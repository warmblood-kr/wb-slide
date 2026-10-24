use axum::{
    Router,
    extract::Path,
    http::{StatusCode, header},
    response::{Html, IntoResponse, Response},
};
use clap::{Parser, Subcommand};
use comrak::{markdown_to_html, Options};
use rust_embed::Embed;
use std::net::SocketAddr;
use std::path::PathBuf;

mod theme;
mod template;
mod render;
mod components;
mod spec;
mod markup;
mod diagram;
mod narrative;
use theme::{LoadedTheme, ThemeSpec};

#[derive(Embed)]
#[folder = "framework/"]
struct FrameworkAssets;

#[derive(Parser)]
#[command(name = "wb-slide", about = "Lightweight slide presentation framework", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum DiagramCommand {
    /// Render one diagram to SVG (stdout unless -o)
    Render {
        /// Diagram type, e.g. quadrant
        #[arg(long = "type")]
        kind: String,
        /// JSON data file (the type's data model, docs/diagram-library.md)
        #[arg(long = "in")]
        input: PathBuf,
        /// Preset theme: a built-in name (default, night) or a token file
        #[arg(long)]
        theme: Option<String>,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum Commands {
    /// Start the slide presentation server
    Show {
        #[arg(short, long, default_value = "3030")]
        port: u16,
        #[arg(short, long)]
        dir: Option<PathBuf>,
        #[arg(long)]
        no_open: bool,
        /// Bypass theme cache and re-fetch from registry
        #[arg(long)]
        refresh_themes: bool,
    },
    /// Export to a self-contained HTML file
    Export {
        #[arg(short, long)]
        dir: Option<PathBuf>,
        /// Export a closed, versioned JSON slide spec using built-ins only
        #[arg(long, conflicts_with_all = ["dir", "refresh_themes", "embed"])]
        spec: Option<PathBuf>,
        /// Export semantic slide markup (<wb-slide>, <wb-title>) with the slide preset
        #[arg(long, conflicts_with_all = ["dir", "spec", "refresh_themes", "embed"])]
        markup: Option<PathBuf>,
        /// Preset theme for --markup: a built-in name (default, night, paper, graphite, signal) or a token file
        #[arg(long, requires = "markup")]
        theme: Option<String>,
        #[arg(short, long, default_value = "export.html")]
        output: PathBuf,
        /// Bypass theme cache and re-fetch from registry
        #[arg(long)]
        refresh_themes: bool,
        /// Inline local assets (images referenced via <img src> and CSS url())
        /// as base64 data URIs, producing a single fully portable HTML file
        #[arg(long)]
        embed: bool,
    },
    /// Statically check a deck for common problems (unknown layouts, dropped
    /// frontmatter, raw-HTML blank lines, missing assets, likely overflow)
    Validate {
        #[arg(short, long)]
        dir: Option<PathBuf>,
        /// Bypass theme cache and re-fetch from registry
        #[arg(long)]
        refresh_themes: bool,
        /// Exit non-zero on warnings too, not only errors
        #[arg(long)]
        strict: bool,
    },
    /// Deterministic diagrams from compact JSON data
    Diagram {
        #[command(subcommand)]
        command: DiagramCommand,
    },
    /// Built-in narrative strategies (story structures for decks)
    Narrative {
        #[command(subcommand)]
        action: NarrativeAction,
    },
    /// Show version and check for updates
    Version,
    /// Update to the latest version
    Update,
    /// Open a folder picker dialog, then start the presentation server
    /// (for non-CLI users — wire this to a desktop shortcut)
    Gui {
        #[arg(short, long, default_value = "3030")]
        port: u16,
    },
}

#[derive(Subcommand)]
enum NarrativeAction {
    /// List strategy ids
    List,
    /// Print one strategy as JSON
    Show { id: String },
    /// Print a skeleton deck in wb-slide markup, one slide per role
    Outline {
        #[arg(long)]
        strategy: String,
        #[arg(long)]
        topic: String,
    },
    /// Rank strategies for an audience, goal and duration, with reasons
    Recommend {
        /// executive, team, technical, customer, investor, general, academic
        #[arg(long)]
        audience: Option<String>,
        /// decide, persuade, inform, inspire, sell, report, teach, fund
        #[arg(long)]
        goal: Option<String>,
        /// Talk length in minutes
        #[arg(long)]
        minutes: Option<u64>,
    },
}

struct Slide {
    frontmatter: Vec<(String, String)>,
    body_html: String,
    slots: Vec<(String, String)>,
}

fn parse_frontmatter(block: &str) -> (Vec<(String, String)>, String) {
    let mut meta = Vec::new();
    let mut body_start = 0;
    let lines: Vec<&str> = block.trim().lines().collect();

    for (i, line) in lines.iter().enumerate() {
        if line.starts_with(char::is_whitespace) {
            body_start = i + 1;
            continue;
        }
        if let Some(colon_pos) = line.find(':') {
            let key = line[..colon_pos].trim();
            if !key.is_empty() && key.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                let mut value = line[colon_pos + 1..].trim().to_string();
                if (value.starts_with('\'') && value.ends_with('\''))
                    || (value.starts_with('"') && value.ends_with('"'))
                {
                    value = value[1..value.len() - 1].to_string();
                }
                if !value.is_empty() {
                    meta.push((key.to_string(), value));
                }
                body_start = i + 1;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    let body = lines[body_start..].join("\n").trim().to_string();
    (meta, body)
}

fn parse_slots(body: &str) -> (String, Vec<(String, String)>) {
    let mut slots = Vec::new();
    let mut default_parts = Vec::new();
    let mut current_slot: Option<String> = None;
    let mut current_content = Vec::new();

    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("::") && trimmed.ends_with("::") && trimmed.len() > 4 {
            let name = trimmed[2..trimmed.len() - 2].trim().to_string();
            if let Some(prev_slot) = current_slot.take() {
                slots.push((prev_slot, current_content.join("\n").trim().to_string()));
            } else if !current_content.is_empty() {
                default_parts.extend(current_content.drain(..));
            }
            current_content.clear();
            current_slot = Some(name);
        } else {
            current_content.push(line.to_string());
        }
    }

    if let Some(slot_name) = current_slot {
        slots.push((slot_name, current_content.join("\n").trim().to_string()));
    } else {
        default_parts.extend(current_content);
    }

    (default_parts.join("\n").trim().to_string(), slots)
}

fn render_markdown(text: &str) -> String {
    if text.trim().is_empty() {
        return String::new();
    }
    let mut options = Options::default();
    options.render.unsafe_ = true;
    // GFM extensions: tables (pipe syntax), strikethrough, autolinks, and
    // task lists — the markdown features authors (and LLMs) expect by default.
    options.extension.table = true;
    options.extension.strikethrough = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    markdown_to_html(text, &options)
}

fn parse_slides(raw: &str) -> (Vec<(String, String)>, Vec<Slide>) {
    // The deck delimiter is line-oriented; Windows line endings must not
    // silently turn a valid deck into zero slides.
    let normalized = raw.replace("\r\n", "\n");
    let raw = normalized.as_str();
    let blocks: Vec<&str> = raw.split("\n---\n").collect();

    // blocks[0] = global frontmatter
    // blocks[1] = slide 1 body
    // blocks[2] = slide 2 frontmatter, blocks[3] = slide 2 body
    // blocks[2n] = slide n+1 frontmatter, blocks[2n+1] = slide n+1 body

    let first_block = blocks[0].trim_start_matches("---\n").trim_start_matches("---\r\n");
    let (global_meta, _) = parse_frontmatter(first_block);

    let mut slides = Vec::new();

    // The first block contains deck settings AND the first slide's context.
    // Only title/theme are deck-only; custom layout attributes must survive.
    if blocks.len() > 1 {
        let fm = global_meta.iter()
            .filter(|(key, _)| !matches!(key.as_str(), "title" | "theme"))
            .cloned()
            .collect();
        let (default_body, slot_parts) = parse_slots(blocks[1].trim());
        let slots = slot_parts.into_iter()
            .map(|(name, content)| (name, render_markdown(&content)))
            .collect();
        slides.push(Slide {
            frontmatter: fm,
            body_html: render_markdown(&default_body),
            slots,
        });
    }

    // Remaining slides: pairs of (frontmatter, body)
    let mut i = 2;
    while i < blocks.len() {
        let (fm, inline_body) = parse_frontmatter(blocks[i]);

        let body = if i + 1 < blocks.len() {
            let next = blocks[i + 1].trim();
            if inline_body.is_empty() {
                next.to_string()
            } else {
                format!("{}\n{}", inline_body, next)
            }
        } else {
            inline_body
        };

        let (default_body, slot_parts) = parse_slots(&body);
        let slots = slot_parts.into_iter()
            .map(|(name, content)| (name, render_markdown(&content)))
            .collect();

        slides.push(Slide {
            frontmatter: fm,
            body_html: render_markdown(&default_body),
            slots,
        });

        i += 2;
    }

    (global_meta, slides)
}

fn get_fm(slide: &Slide, key: &str) -> Option<String> {
    slide.frontmatter.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
}

/// Server-side render every slide. Returns the assembled HTML + any client JS
/// needed for `.js` escape-hatch layouts.
fn render_all_slides(
    slides: &[Slide],
    global_meta: &[(String, String)],
    layouts: &render::LayoutSet,
) -> Result<render::RenderedSlides, String> {
    let global_watermark = global_meta.iter().find(|(k, _)| k == "watermark").map(|(_, v)| v.as_str()).unwrap_or("");
    let global_footer = global_meta.iter().find(|(k, _)| k == "footer").map(|(_, v)| v.as_str()).unwrap_or("");
    let global_folio = global_meta.iter().find(|(k, _)| k == "folio").map(|(_, v)| v.as_str()).unwrap_or("");

    let total = slides.len();
    let mut out = render::RenderedSlides::default();
    // Deduplicate: same .js layout used by many slides should be inlined once.
    let mut seen_js: std::collections::HashSet<String> = std::collections::HashSet::new();

    for (i, slide) in slides.iter().enumerate() {
        let layout_name = get_fm(slide, "layout").unwrap_or_else(|| "slide-default".to_string());

        let mut attrs: Vec<(String, String)> = slide.frontmatter.iter()
            .filter(|(k, _)| k != "layout")
            .cloned()
            .collect();
        if !global_watermark.is_empty() && get_fm(slide, "watermark").is_none() {
            attrs.push(("watermark".to_string(), global_watermark.to_string()));
        }
        if !global_footer.is_empty() && get_fm(slide, "footer").is_none() {
            attrs.push(("footer".to_string(), global_footer.to_string()));
        }
        if !global_folio.is_empty() && get_fm(slide, "folio").is_none() {
            attrs.push(("folio".to_string(), global_folio.to_string()));
        }

        let (html, maybe_js) = render::render_slide(
            &layout_name,
            &attrs,
            &slide.slots,
            &slide.body_html,
            i + 1,
            total,
            layouts,
        );
        let html = components::expand(&html).map_err(|error| format!("slide {}: {error}", i + 1))?;
        out.html.push_str(&html);
        out.html.push('\n');

        if let Some(js) = maybe_js {
            if seen_js.insert(layout_name.clone()) {
                // Need SlideBase if any .js layout is used. Emit it once at first sight.
                if seen_js.len() == 1 {
                    if let Some(file) = FrameworkAssets::get("slide-base.js") {
                        let base = String::from_utf8_lossy(&file.data);
                        out.client_js.push_str(&base);
                        out.client_js.push('\n');
                    }
                }
                let stripped = js
                    .replace("import { SlideBase } from '../slide-base.js';", "")
                    .replace("import { SlideBase } from './slide-base.js';", "");
                let guarded = theme::guard_custom_elements_define(&stripped);
                out.client_js.push_str(&theme::wrap_in_iife(&guarded));
                out.client_js.push('\n');
            }
        }
    }
    Ok(out)
}

struct HtmlOptions<'a> {
    title: &'a str,
    slides_html: &'a str,
    framework_css: &'a str,
    framework_js: &'a str,
    client_js: &'a str,
    theme_css: Option<&'a str>,
    user_css: Option<&'a str>,
}

fn build_index_html(opts: &HtmlOptions) -> String {
    let theme_css_tag = opts.theme_css
        .map(|css| format!("<style data-source=\"theme\">{css}</style>"))
        .unwrap_or_default();
    let user_css_tag = opts.user_css
        .map(|css| format!("<style data-source=\"user\">{css}</style>"))
        .unwrap_or_default();
    let client_js_tag = if opts.client_js.is_empty() {
        String::new()
    } else {
        format!("<script type=\"module\">{}</script>", opts.client_js)
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="ko">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{title}</title>
  <style>{framework_css}</style>
  {theme_css_tag}
  {user_css_tag}
</head>
<body>
  <div id="monocle-slide-deck">
    <div class="ms-viewport">
{slides_html}
    </div>
  </div>
  {client_js_tag}
  <script>{framework_js}</script>
</body>
</html>"#,
        title = template::html_escape(opts.title),
        framework_css = opts.framework_css,
        slides_html = opts.slides_html,
        framework_js = opts.framework_js,
    )
}

fn collect_framework_css() -> String {
    let mut css = String::new();
    for name in &["theme.css", "utilities.css", "print.css"] {
        if let Some(file) = FrameworkAssets::get(name) {
            css.push_str(&String::from_utf8_lossy(&file.data));
            css.push('\n');
        }
    }
    css
}

fn collect_framework_js() -> String {
    if let Some(file) = FrameworkAssets::get("monocle-slide.js") {
        let content = String::from_utf8_lossy(&file.data);
        let mut result = String::new();
        for line in content.lines() {
            if line.starts_with("import ") {
                continue;
            }
            result.push_str(line);
            result.push('\n');
        }
        return result;
    }
    String::new()
}

/// Turn a local asset reference into a `data:` URI, or return None to leave it
/// untouched (remote URLs, existing data URIs, anchors, or unreadable files).
fn asset_to_data_uri(work_dir: &std::path::Path, raw: &str) -> Option<String> {
    use base64::Engine as _;
    let p = raw.trim();
    if p.is_empty()
        || p.starts_with("data:")
        || p.starts_with("http://")
        || p.starts_with("https://")
        || p.starts_with("//")
        || p.starts_with('#')
    {
        return None;
    }
    let rel = p.strip_prefix("./").unwrap_or(p);
    let path = work_dir.join(rel);
    let bytes = std::fs::read(&path).ok()?;
    let mime = mime_guess::from_path(&path).first_or_octet_stream();
    let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
    Some(format!("data:{mime};base64,{b64}"))
}

/// Replace every `<marker><value><close>` occurrence's value with a data URI
/// when it points at a readable local asset (used for `src="`, `src='`).
fn rewrite_attr(input: &str, marker: &str, close: char, work_dir: &std::path::Path) -> String {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(pos) = rest.find(marker) {
        let (before, after) = rest.split_at(pos + marker.len());
        out.push_str(before);
        if let Some(end) = after.find(close) {
            let value = &after[..end];
            match asset_to_data_uri(work_dir, value) {
                Some(uri) => out.push_str(&uri),
                None => out.push_str(value),
            }
            rest = &after[end..]; // leave the closing delimiter for the next copy
        } else {
            out.push_str(after);
            return out;
        }
    }
    out.push_str(rest);
    out
}

/// Replace `url(...)` references (with optional quotes) inside inlined CSS.
fn rewrite_css_urls(input: &str, work_dir: &std::path::Path) -> String {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(pos) = rest.find("url(") {
        let (before, after) = rest.split_at(pos + 4);
        out.push_str(before);
        if let Some(end) = after.find(')') {
            let inner = after[..end].trim();
            let quoted = inner.starts_with('"') || inner.starts_with('\'');
            let value = if quoted && inner.len() >= 2 {
                &inner[1..inner.len() - 1]
            } else {
                inner
            };
            match asset_to_data_uri(work_dir, value) {
                Some(uri) if quoted => {
                    out.push('"');
                    out.push_str(&uri);
                    out.push('"');
                }
                Some(uri) => out.push_str(&uri),
                None => out.push_str(&after[..end]),
            }
            rest = &after[end..];
        } else {
            out.push_str(after);
            return out;
        }
    }
    out.push_str(rest);
    out
}

/// Inline all local assets in the exported HTML as base64 data URIs so the
/// output is a single, fully portable file. Remote URLs and existing data
/// URIs are left untouched.
fn embed_assets(html: &str, work_dir: &std::path::Path) -> String {
    let html = rewrite_attr(html, "src=\"", '"', work_dir);
    let html = rewrite_attr(&html, "src='", '\'', work_dir);
    rewrite_css_urls(&html, work_dir)
}

// ---------------------------------------------------------------------------
// Deck validation (`wb-slide validate`)
// ---------------------------------------------------------------------------

#[derive(PartialEq, Clone, Copy)]
enum Severity {
    Error,
    Warning,
}

struct Diagnostic {
    severity: Severity,
    slide: Option<usize>,
    message: String,
}

/// True if a blank line appears while inside an unclosed block-level HTML tag
/// (svg/table/div/ul/ol/g). comrak ends the HTML block at that blank line, so
/// everything after it renders as literal text (the inline-SVG footgun).
fn raw_html_blank_line(body: &str) -> bool {
    let opens = ["<svg", "<table", "<div", "<ul", "<ol", "<g ", "<g>"];
    let closes = ["</svg>", "</table>", "</div>", "</ul>", "</ol>", "</g>"];
    let mut depth: i32 = 0;
    for line in body.lines() {
        let t = line.trim();
        if t.is_empty() {
            if depth > 0 {
                return true;
            }
            continue;
        }
        for o in opens {
            if t.contains(o) {
                depth += 1;
            }
        }
        for c in closes {
            if t.contains(c) {
                depth -= 1;
            }
        }
        if depth < 0 {
            depth = 0;
        }
    }
    false
}

fn is_local_ref(p: &str) -> bool {
    let p = p.trim();
    !(p.is_empty()
        || p.starts_with("http://")
        || p.starts_with("https://")
        || p.starts_with("//")
        || p.starts_with("data:")
        || p.starts_with('#')
        || p.starts_with("mailto:"))
}

fn refs_between(text: &str, marker: &str, close: char) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(pos) = rest.find(marker) {
        let after = &rest[pos + marker.len()..];
        if let Some(end) = after.find(close) {
            out.push(after[..end].to_string());
            rest = &after[end..];
        } else {
            break;
        }
    }
    out
}

/// All local asset paths referenced in a slide body: `<img src>`, CSS `url()`,
/// and Markdown images `![](path)`.
fn collect_local_refs(body: &str) -> Vec<String> {
    let mut refs = Vec::new();
    refs.extend(refs_between(body, "src=\"", '"'));
    refs.extend(refs_between(body, "src='", '\''));
    for u in refs_between(body, "url(", ')') {
        let v = u.trim().trim_matches('"').trim_matches('\'').to_string();
        refs.push(v);
    }
    // Markdown images: ![alt](path "title")
    let mut rest = body;
    while let Some(bang) = rest.find("![") {
        let after = &rest[bang..];
        if let Some(op) = after.find("](") {
            let tail = &after[op + 2..];
            if let Some(cp) = tail.find(')') {
                let raw = tail[..cp].trim();
                let path = raw.split_whitespace().next().unwrap_or(raw);
                refs.push(path.to_string());
                rest = &tail[cp..];
                continue;
            }
        }
        rest = &after[2..];
    }
    refs.into_iter().filter(|r| is_local_ref(r)).collect()
}

/// Warn about indented frontmatter lines (silently dropped — flat keys only).
fn check_frontmatter_indent(block: &str, slide: Option<usize>, diags: &mut Vec<Diagnostic>) {
    for line in block.lines() {
        if line.trim().is_empty() {
            break; // end of the frontmatter region
        }
        if line.starts_with(' ') || line.starts_with('\t') {
            let t = line.trim();
            if t.contains(':') && t.chars().next().map_or(false, |c| c.is_alphanumeric()) {
                diags.push(Diagnostic {
                    severity: Severity::Warning,
                    slide,
                    message: format!(
                        "indented frontmatter is ignored (use flat `key: value` only): `{t}`"
                    ),
                });
            }
        }
    }
}

fn check_slide(
    no: usize,
    layout: Option<&str>,
    body: &str,
    work_dir: &std::path::Path,
    layouts: &render::LayoutSet,
    diags: &mut Vec<Diagnostic>,
) {
    let lname = layout.unwrap_or("slide-default");
    if layouts.resolve(lname).is_none() {
        diags.push(Diagnostic {
            severity: Severity::Warning,
            slide: Some(no),
            message: format!("unknown layout `{lname}` — wb-slide falls back to slide-default"),
        });
    }
    if let Some(render::Layout { source: render::LayoutSource::Script(js), .. }) = layouts.resolve(lname) {
        // A conservative source check, not a JavaScript parser. The old
        // SlideBase asset is no longer bundled; whitespace may span lines.
        let words: Vec<&str> = js.split_whitespace().collect();
        if words.windows(2).any(|pair| pair[0] == "extends" && pair[1].trim_end_matches('{') == "SlideBase") {
            diags.push(Diagnostic {
                severity: Severity::Warning,
                slide: Some(no),
                message: format!("layout `{lname}` references legacy SlideBase, which is no longer bundled — migrate to layouts/{lname}.html (see docs/migration-v0.7.md)"),
            });
        }
    }
    if raw_html_blank_line(body) {
        diags.push(Diagnostic {
            severity: Severity::Warning,
            slide: Some(no),
            message: "blank line inside a raw HTML/SVG block — content after it renders as literal text".to_string(),
        });
    }
    for r in collect_local_refs(body) {
        let rel = r.trim_start_matches("./");
        if !work_dir.join(rel).exists() {
            diags.push(Diagnostic {
                severity: Severity::Error,
                slide: Some(no),
                message: format!("asset not found: `{r}`"),
            });
        }
    }
    // Overflow heuristic — the canvas is a fixed 960x540 with overflow:hidden.
    let sparse = matches!(
        lname,
        "slide-cover" | "slide-section" | "slide-image-full" | "slide-quote"
    );
    let chars = body.chars().count();
    let lines = body.lines().filter(|l| !l.trim().is_empty()).count();
    if !sparse && (chars > 1100 || lines > 22) {
        diags.push(Diagnostic {
            severity: Severity::Warning,
            slide: Some(no),
            message: "slide looks dense and may overflow the 960x540 canvas (heuristic) — consider splitting".to_string(),
        });
    }
}

/// Run all static checks over a deck's raw `slides.md`.
fn validate_deck(
    raw: &str,
    work_dir: &std::path::Path,
    layouts: &render::LayoutSet,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let normalized = raw.replace("\r\n", "\n");
    let raw = normalized.as_str();
    let blocks: Vec<&str> = raw.split("\n---\n").collect();

    let global_block = blocks
        .first()
        .copied()
        .unwrap_or("")
        .trim_start_matches("---\n")
        .trim_start_matches("---\r\n");
    let (global_meta, _) = parse_frontmatter(global_block);
    let global_layout = global_meta
        .iter()
        .find(|(k, _)| k == "layout")
        .map(|(_, v)| v.clone());
    check_frontmatter_indent(global_block, None, &mut diags);

    let mut slide_no = 0;
    // Slide 1: body is blocks[1], layout inherited from global frontmatter.
    if blocks.len() > 1 {
        slide_no += 1;
        check_slide(
            slide_no,
            global_layout.as_deref(),
            blocks[1].trim(),
            work_dir,
            layouts,
            &mut diags,
        );
    }
    // Remaining slides: (frontmatter, body) pairs.
    let mut i = 2;
    while i < blocks.len() {
        slide_no += 1;
        let (fm, inline_body) = parse_frontmatter(blocks[i]);
        check_frontmatter_indent(blocks[i], Some(slide_no), &mut diags);
        let layout = fm.iter().find(|(k, _)| k == "layout").map(|(_, v)| v.as_str());
        let body = if i + 1 < blocks.len() {
            let next = blocks[i + 1].trim();
            if inline_body.is_empty() {
                next.to_string()
            } else {
                format!("{inline_body}\n{next}")
            }
        } else {
            inline_body
        };
        check_slide(slide_no, layout, &body, work_dir, layouts, &mut diags);
        i += 2;
    }

    diags
}

async fn serve_index(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Response {
    let raw = std::fs::read_to_string(&state.slides_path)
        .unwrap_or_else(|_| "---\ntitle: No slides found\n---\n\n# No slides.md found".to_string());

    let (global_meta, slides) = parse_slides(&raw);
    let title = global_meta.iter().find(|(k, _)| k == "title")
        .map(|(_, v)| v.as_str()).unwrap_or("WB Slide");

    let theme = load_theme_from_meta(&global_meta, state.refresh_themes).await;
    let user_css = collect_user_css(&state.work_dir);
    let framework_css = collect_framework_css();
    let framework_js = collect_framework_js();

    let mut layouts = render::LayoutSet::default();
    layouts.load_builtin();
    if let Some(t) = &theme {
        layouts.load_theme(t);
    }
    layouts.load_local(&state.work_dir);

    let rendered = match render_all_slides(&slides, &global_meta, &layouts) {
        Ok(rendered) => rendered,
        Err(error) => return (StatusCode::BAD_REQUEST, format!("Deck render error: {error}")).into_response(),
    };

    Html(build_index_html(&HtmlOptions {
        title,
        slides_html: &rendered.html,
        framework_css: &framework_css,
        framework_js: &framework_js,
        client_js: &rendered.client_js,
        theme_css: theme.as_ref().map(|t| t.css.as_str()),
        user_css: user_css.as_deref(),
    })).into_response()
}

async fn load_theme_from_meta(
    global_meta: &[(String, String)],
    refresh: bool,
) -> Option<LoadedTheme> {
    let theme_value = global_meta.iter().find(|(k, _)| k == "theme")?.1.clone();
    let spec = ThemeSpec::parse(&theme_value);
    match theme::load_theme(&spec, refresh).await {
        Ok(t) => {
            eprintln!("  Theme: {} v{}", t.name, t.version);
            Some(t)
        }
        Err(e) => {
            eprintln!("warning: failed to load theme \"{theme_value}\": {e}");
            None
        }
    }
}

async fn serve_framework(Path(path): Path<String>) -> Response {
    match FrameworkAssets::get(&path) {
        Some(content) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            (StatusCode::OK, [(header::CONTENT_TYPE, mime.as_ref())], content.data.to_vec()).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn serve_static(
    axum::extract::State(state): axum::extract::State<AppState>,
    Path(path): Path<String>,
) -> Response {
    let file_path = state.work_dir.join(&path);
    match tokio::fs::read(&file_path).await {
        Ok(content) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            (StatusCode::OK, [(header::CONTENT_TYPE, mime.as_ref())], content).into_response()
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

#[derive(Clone)]
struct AppState {
    work_dir: PathBuf,
    slides_path: PathBuf,
    refresh_themes: bool,
}

fn resolve_state(dir: Option<PathBuf>, refresh_themes: bool) -> AppState {
    let work_dir = dir.unwrap_or_else(|| std::env::current_dir().unwrap());
    let work_dir = work_dir.canonicalize().unwrap_or(work_dir);

    let slides_path = if work_dir.join("slides.md").exists() {
        work_dir.join("slides.md")
    } else {
        work_dir.join("index.md")
    };

    AppState { work_dir, slides_path, refresh_themes }
}

fn collect_user_css(work_dir: &std::path::Path) -> Option<String> {
    let styles_dir = work_dir.join("styles");
    if !styles_dir.is_dir() {
        return None;
    }
    let mut css = String::new();
    if let Ok(entries) = std::fs::read_dir(&styles_dir) {
        let mut paths: Vec<_> = entries.filter_map(|e| e.ok()).map(|e| e.path()).collect();
        paths.sort();
        for path in paths {
            if path.extension().map_or(false, |ext| ext == "css") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    css.push_str(&content);
                    css.push('\n');
                }
            }
        }
    }
    if css.is_empty() { None } else { Some(css) }
}

fn export_markup(path: &std::path::Path, theme: Option<&str>, output: &std::path::Path) -> Result<(), String> {
    use std::io::Read;
    let mut bytes = Vec::new();
    std::fs::File::open(path).and_then(|f| f.take(64 * 1024 + 1).read_to_end(&mut bytes))
        .map_err(|e| format!("cannot read markup: {e}"))?;
    if bytes.len() > 64 * 1024 {
        return Err("markup exceeds 64 KiB".into());
    }
    let source = String::from_utf8(bytes).map_err(|_| "markup is not UTF-8".to_string())?;
    // Images resolve relative to the markup file, like the spec path.
    let base = path.parent().filter(|d| !d.as_os_str().is_empty()).unwrap_or(std::path::Path::new("."));
    let (meta, slides) = spec::parse_in(markup::compile(&source)?.as_bytes(), Some(base), theme)?;
    write_spec_html(meta, slides, Some(&markup::preset_css(theme)?), output)
}

fn render_diagram(kind: &str, input: &std::path::Path, theme: Option<&str>, output: Option<&std::path::Path>) -> Result<(), String> {
    let bytes = std::fs::read(input).map_err(|e| format!("cannot read {}: {e}", input.display()))?;
    if bytes.len() > 64 * 1024 {
        return Err("diagram data exceeds 64 KiB".into());
    }
    let data: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|e| format!("$: invalid JSON at line {} column {}: {e}", e.line(), e.column()))?;
    let svg = diagram::render(kind, &data, theme)?;
    match output {
        Some(path) => std::fs::write(path, svg).map_err(|e| format!("cannot write {}: {e}", path.display())),
        None => {
            use std::io::Write;
            let _ = std::io::stdout().write_all(svg.as_bytes()); // a closed pipe is not an error
            Ok(())
        }
    }
}

fn export_spec(path: &std::path::Path, output: &std::path::Path) -> Result<(), String> {
    let (meta, slides) = spec::load(path)?;
    write_spec_html(meta, slides, None, output)
}

fn write_spec_html(meta: Vec<(String, String)>, slides: Vec<Slide>, theme_css: Option<&str>, output: &std::path::Path) -> Result<(), String> {
    use std::io::Write;
    let mut layouts = render::LayoutSet::default();
    layouts.load_builtin();
    let rendered = render_all_slides(&slides, &meta, &layouts)?;
    let html = build_index_html(&HtmlOptions {
        title: &meta[0].1, slides_html: &rendered.html,
        framework_css: &collect_framework_css(), framework_js: &collect_framework_js(),
        client_js: "", theme_css, user_css: None,
    });
    let mut name = output.file_name().ok_or("output must name a file")?.to_os_string();
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos();
    name.push(format!(".tmp-{}-{nonce}", std::process::id()));
    let temporary = output.with_file_name(name);
    let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(&temporary)
        .map_err(|e| format!("cannot stage output: {e}"))?;
    let result = file.write_all(html.as_bytes()).and_then(|_| file.sync_all());
    drop(file);
    let result = result.and_then(|_| std::fs::rename(&temporary, output));
    if result.is_err() { let _ = std::fs::remove_file(&temporary); }
    result.map_err(|e| format!("cannot publish output: {e}"))
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Show { port, dir, no_open, refresh_themes } => {
            let state = resolve_state(dir, refresh_themes);

            if !state.slides_path.exists() {
                eprintln!("Warning: No slides.md or index.md found in {}", state.work_dir.display());
            }

            let app = Router::new()
                .route("/", axum::routing::get(serve_index))
                .route("/_framework/{*path}", axum::routing::get(serve_framework))
                .route("/{*path}", axum::routing::get(serve_static))
                .with_state(state.clone());

            let addr = SocketAddr::from(([0, 0, 0, 0], port));
            eprintln!("  WB Slide v{}", env!("CARGO_PKG_VERSION"));
            eprintln!("  Serving: {}", state.work_dir.display());
            eprintln!("  URL: http://localhost:{port}/");

            if !no_open {
                let _ = open::that(format!("http://localhost:{port}/"));
            }

            let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
            axum::serve(listener, app).await.unwrap();
        }

        Commands::Export { dir, spec, markup, theme, output, refresh_themes, embed } => {
            let typed = match (&spec, &markup) {
                (Some(path), _) => Some(export_spec(path, &output)),
                (_, Some(path)) => Some(export_markup(path, theme.as_deref(), &output)),
                _ => None,
            };
            if let Some(result) = typed {
                if let Err(error) = result {
                    eprintln!("Error: {error}");
                    std::process::exit(1);
                }
                eprintln!("Exported to: {}", output.display());
                return;
            }
            let state = resolve_state(dir, refresh_themes);

            if !state.slides_path.exists() {
                eprintln!("Error: No slides.md found in {}", state.work_dir.display());
                std::process::exit(1);
            }

            let raw = std::fs::read_to_string(&state.slides_path).unwrap();
            let (global_meta, slides) = parse_slides(&raw);
            let title = global_meta.iter().find(|(k, _)| k == "title")
                .map(|(_, v)| v.as_str()).unwrap_or("WB Slide");

            let theme = load_theme_from_meta(&global_meta, refresh_themes).await;
            let user_css = collect_user_css(&state.work_dir);
            let framework_css = collect_framework_css();
            let framework_js = collect_framework_js();

            let mut layouts = render::LayoutSet::default();
            layouts.load_builtin();
            if let Some(t) = &theme {
                layouts.load_theme(t);
            }
            layouts.load_local(&state.work_dir);

            let rendered = match render_all_slides(&slides, &global_meta, &layouts) {
                Ok(rendered) => rendered,
                Err(error) => {
                    eprintln!("Error: {error}");
                    std::process::exit(1);
                }
            };

            let html = build_index_html(&HtmlOptions {
                title,
                slides_html: &rendered.html,
                framework_css: &framework_css,
                framework_js: &framework_js,
                client_js: &rendered.client_js,
                theme_css: theme.as_ref().map(|t| t.css.as_str()),
                user_css: user_css.as_deref(),
            });

            let html = if embed {
                embed_assets(&html, &state.work_dir)
            } else {
                html
            };

            let output_path = if output.is_absolute() {
                output
            } else {
                state.work_dir.join(output)
            };

            std::fs::write(&output_path, &html).unwrap();
            if embed {
                eprintln!("Exported to: {} (assets embedded)", output_path.display());
            } else {
                eprintln!("Exported to: {}", output_path.display());
            }
        }

        Commands::Validate { dir, refresh_themes, strict } => {
            let state = resolve_state(dir, refresh_themes);

            if !state.slides_path.exists() {
                eprintln!("Error: No slides.md found in {}", state.work_dir.display());
                std::process::exit(1);
            }

            let raw = std::fs::read_to_string(&state.slides_path).unwrap();
            let (global_meta, _) = parse_slides(&raw);

            let theme = load_theme_from_meta(&global_meta, refresh_themes).await;
            let mut layouts = render::LayoutSet::default();
            layouts.load_builtin();
            if let Some(t) = &theme {
                layouts.load_theme(t);
            }
            layouts.load_local(&state.work_dir);

            let diags = validate_deck(&raw, &state.work_dir, &layouts);
            let (_, slides) = parse_slides(&raw);
            if let Err(error) = render_all_slides(&slides, &global_meta, &layouts) {
                eprintln!("Error: {error}");
                std::process::exit(1);
            }
            let errors = diags.iter().filter(|d| d.severity == Severity::Error).count();
            let warnings = diags.iter().filter(|d| d.severity == Severity::Warning).count();

            for d in &diags {
                let tag = match d.severity {
                    Severity::Error => "error",
                    Severity::Warning => "warn ",
                };
                let loc = d.slide.map(|n| format!("slide {n}")).unwrap_or_else(|| "global".to_string());
                eprintln!("  [{tag}] {loc}: {}", d.message);
            }

            if diags.is_empty() {
                println!("\u{2713} {} — no issues found", state.slides_path.display());
            } else {
                eprintln!("\n{errors} error(s), {warnings} warning(s)");
            }

            if errors > 0 || (strict && warnings > 0) {
                std::process::exit(1);
            }
        }

        Commands::Diagram { command: DiagramCommand::Render { kind, input, theme, output } } => {
            if let Err(e) = render_diagram(&kind, &input, theme.as_deref(), output.as_deref()) {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
        Commands::Narrative { action } => {
            let out = match action {
                NarrativeAction::List => narrative::list(),
                NarrativeAction::Show { id } => narrative::show(&id),
                NarrativeAction::Outline { strategy, topic } => narrative::outline(&strategy, &topic),
                NarrativeAction::Recommend { audience, goal, minutes } => {
                    narrative::recommend(audience.as_deref(), goal.as_deref(), minutes)
                }
            };
            match out {
                Ok(text) => println!("{}", text.trim_end()),
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::Version => {
            let current = env!("CARGO_PKG_VERSION");
            println!("wb-slide v{current}");

            eprint!("Checking for updates... ");
            match check_latest_version().await {
                Ok(latest) => {
                    let latest_clean = latest.trim_start_matches('v');
                    if latest_clean == current {
                        eprintln!("up to date.");
                    } else {
                        eprintln!("v{latest_clean} available!");
                        eprintln!();
                        eprintln!("  Run `wb-slide update` to upgrade.");
                    }
                }
                Err(e) => eprintln!("could not check ({e})"),
            }
        }

        Commands::Gui { port } => {
            // Use the OS-native folder picker via shell-out. No GUI deps.
            let dir = match pick_folder_native() {
                Some(p) => p,
                None => {
                    eprintln!("No folder selected.");
                    return;
                }
            };

            let canonical = dir.canonicalize().unwrap_or(dir.clone());
            let slides_path = if canonical.join("slides.md").exists() {
                canonical.join("slides.md")
            } else {
                canonical.join("index.md")
            };

            if !slides_path.exists() {
                let msg = format!(
                    "No slides.md or index.md found in:\n{}",
                    canonical.display()
                );
                show_message_native(&msg);
                return;
            }

            let state = AppState {
                work_dir: canonical.clone(),
                slides_path,
                refresh_themes: false,
            };

            let app = Router::new()
                .route("/", axum::routing::get(serve_index))
                .route("/_framework/{*path}", axum::routing::get(serve_framework))
                .route("/{*path}", axum::routing::get(serve_static))
                .with_state(state.clone());

            let addr = SocketAddr::from(([0, 0, 0, 0], port));
            eprintln!("  WB Slide v{}", env!("CARGO_PKG_VERSION"));
            eprintln!("  Serving: {}", state.work_dir.display());
            eprintln!("  URL: http://localhost:{port}/");

            let _ = open::that(format!("http://localhost:{port}/"));

            let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
            axum::serve(listener, app).await.unwrap();
        }

        Commands::Update => {
            let current = env!("CARGO_PKG_VERSION");
            eprint!("Checking latest version... ");

            let latest = match check_latest_version().await {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("failed ({e})");
                    std::process::exit(1);
                }
            };

            let latest_clean = latest.trim_start_matches('v');
            if latest_clean == current {
                eprintln!("already at v{current}.");
                return;
            }

            eprintln!("v{latest_clean} (current: v{current})");

            let platform = detect_platform();
            let asset = match platform.as_str() {
                "macos-arm64" => "wb-slide-macos-arm64.tar.gz",
                "linux-x64" => "wb-slide-linux-x64.tar.gz",
                "windows-x64" => "wb-slide-windows-x64.zip",
                _ => {
                    eprintln!("Unsupported platform: {platform}");
                    std::process::exit(1);
                }
            };

            let url = format!(
                "https://github.com/warmblood-kr/wb-slide/releases/download/{latest}/{asset}"
            );

            eprintln!("Downloading {asset}...");
            let self_path = std::env::current_exe().unwrap();
            let tmp_dir = std::env::temp_dir().join("wb-slide-update");
            let _ = std::fs::remove_dir_all(&tmp_dir);
            std::fs::create_dir_all(&tmp_dir).unwrap();

            let resp = reqwest::get(&url).await;
            let resp = match resp {
                Ok(r) if r.status().is_success() => r,
                Ok(r) => {
                    eprintln!("Download failed: HTTP {}", r.status());
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Download failed: {e}");
                    std::process::exit(1);
                }
            };

            let bytes = resp.bytes().await.unwrap();
            let archive_path = tmp_dir.join(asset);
            std::fs::write(&archive_path, &bytes).unwrap();

            if asset.ends_with(".tar.gz") {
                let status = std::process::Command::new("tar")
                    .args(["xzf", &archive_path.to_string_lossy(), "-C", &tmp_dir.to_string_lossy()])
                    .status()
                    .expect("failed to run tar");
                if !status.success() {
                    eprintln!("Failed to extract archive");
                    std::process::exit(1);
                }
                let new_binary = tmp_dir.join("wb-slide");
                if !self_replace(&new_binary, &self_path) {
                    eprintln!();
                    eprintln!("Tip: install to a user-owned location instead, e.g. ~/.local/bin");
                    std::process::exit(1);
                }
            } else {
                eprintln!("Windows: extract {asset} manually and replace the binary.");
                eprintln!("Archive saved to: {}", archive_path.display());
                return;
            }

            let _ = std::fs::remove_dir_all(&tmp_dir);
            eprintln!("Updated to v{latest_clean}!");
        }
    }
}

async fn check_latest_version() -> Result<String, String> {
    let client = reqwest::Client::builder()
        .user_agent("wb-slide")
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get("https://api.github.com/repos/warmblood-kr/wb-slide/releases/latest")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let text = resp.text().await.map_err(|e| e.to_string())?;

    text.split("\"tag_name\"")
        .nth(1)
        .and_then(|s| s.split('"').nth(1))
        .map(|s| s.to_string())
        .ok_or_else(|| "could not parse response".to_string())
}

fn detect_platform() -> String {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    match (os, arch) {
        ("macos", "aarch64") => "macos-arm64".to_string(),
        ("linux", "x86_64") => "linux-x64".to_string(),
        ("windows", "x86_64") => "windows-x64".to_string(),
        _ => format!("{os}-{arch}"),
    }
}

/// Open a native folder picker dialog by shelling out to the OS.
/// Returns None if the user cancels or no helper is available.
fn pick_folder_native() -> Option<PathBuf> {
    use std::process::Command;

    #[cfg(target_os = "macos")]
    {
        let output = Command::new("osascript")
            .args([
                "-e",
                "set f to choose folder with prompt \"Select a folder containing slides.md\"",
                "-e",
                "POSIX path of f",
            ])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if path.is_empty() { None } else { Some(PathBuf::from(path)) }
    }

    #[cfg(target_os = "linux")]
    {
        // Try zenity, then kdialog, then env fallback.
        for cmd in &["zenity", "kdialog"] {
            let args: Vec<&str> = if *cmd == "zenity" {
                vec!["--file-selection", "--directory", "--title=Select a folder containing slides.md"]
            } else {
                vec!["--getexistingdirectory", ".", "--title", "Select a folder containing slides.md"]
            };
            if let Ok(output) = Command::new(cmd).args(&args).output() {
                if output.status.success() {
                    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !path.is_empty() {
                        return Some(PathBuf::from(path));
                    }
                }
                // exit code != 0 means user cancelled; stop trying
                return None;
            }
        }
        eprintln!("No folder picker available. Install zenity or kdialog, or use `wb-slide show --dir <path>`.");
        None
    }

    #[cfg(target_os = "windows")]
    {
        let script = r#"
Add-Type -AssemblyName System.Windows.Forms
$f = New-Object System.Windows.Forms.FolderBrowserDialog
$f.Description = "Select a folder containing slides.md"
$f.ShowNewFolderButton = $false
if ($f.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
    [Console]::Out.WriteLine($f.SelectedPath)
}
"#;
        let output = Command::new("powershell")
            .args(["-NoProfile", "-STA", "-Command", script])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if path.is_empty() { None } else { Some(PathBuf::from(path)) }
    }
}

/// Show a small message dialog via the OS.
fn show_message_native(msg: &str) {
    use std::process::Command;

    #[cfg(target_os = "macos")]
    {
        let script = format!(
            "display dialog \"{}\" with title \"WB Slide\" buttons {{\"OK\"}} default button \"OK\"",
            msg.replace('"', "\\\"")
        );
        let _ = Command::new("osascript").args(["-e", &script]).output();
    }

    #[cfg(target_os = "linux")]
    {
        for cmd in &["zenity", "kdialog"] {
            let args: Vec<String> = if *cmd == "zenity" {
                vec!["--info".to_string(), "--title=WB Slide".to_string(), format!("--text={msg}")]
            } else {
                vec!["--title".to_string(), "WB Slide".to_string(), "--msgbox".to_string(), msg.to_string()]
            };
            if Command::new(cmd).args(&args).status().is_ok() {
                return;
            }
        }
        eprintln!("{msg}");
    }

    #[cfg(target_os = "windows")]
    {
        let script = format!(
            r#"Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.MessageBox]::Show("{}", "WB Slide") | Out-Null"#,
            msg.replace('"', "`\"").replace('\n', "`n")
        );
        let _ = Command::new("powershell")
            .args(["-NoProfile", "-Command", &script])
            .output();
    }
}

/// Replace the running binary with a new one. Returns true on success.
fn self_replace(new_binary: &std::path::Path, self_path: &std::path::Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(new_binary, std::fs::Permissions::from_mode(0o755));
    }

    let backup = self_path.with_extension("old");
    let _ = std::fs::remove_file(&backup);

    if std::fs::rename(self_path, &backup).is_err() {
        eprintln!("Could not replace binary at {}.", self_path.display());
        eprintln!("Either re-run with sudo, or reinstall to a user-owned path:");
        eprintln!("  curl -fsSL https://raw.githubusercontent.com/warmblood-kr/wb-slide/main/install.sh | sh");
        return false;
    }

    if let Err(e) = std::fs::rename(new_binary, self_path) {
        let _ = std::fs::rename(&backup, self_path);
        eprintln!("Could not install new binary at {}: {e}", self_path.display());
        return false;
    }

    let _ = std::fs::remove_file(&backup);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_html_blank_line() {
        assert!(raw_html_blank_line("<svg>\n<rect/>\n\n<rect/>\n</svg>"));
        assert!(!raw_html_blank_line("<svg>\n<rect/>\n<rect/>\n</svg>"));
        assert!(!raw_html_blank_line("# Heading\n\nA paragraph.\n\nAnother."));
    }

    #[test]
    fn test_collect_local_refs() {
        let body = r#"<img src="assets/a.png"> ![x](assets/b.png) <img src="https://h/c.png">
<style>.x{background:url('assets/d.png')}</style> [link](https://x)"#;
        let refs = collect_local_refs(body);
        assert!(refs.contains(&"assets/a.png".to_string()));
        assert!(refs.contains(&"assets/b.png".to_string()));
        assert!(refs.contains(&"assets/d.png".to_string()));
        assert!(!refs.iter().any(|r| r.starts_with("http")));
    }

    #[test]
    fn test_validate_deck_flags_issues() {
        let mut layouts = render::LayoutSet::default();
        layouts.load_builtin();
        let work_dir = std::env::temp_dir(); // assets won't exist here
        let raw = "---\ntitle: T\nfonts:\n  sans: Inter\nlayout: slide-cover\n---\n\n# Hi\n\n---\nlayout: slide-nope\n---\n\n![c](assets/missing.png)";
        let diags = validate_deck(raw, &work_dir, &layouts);
        assert!(diags.iter().any(|d| d.message.contains("indented frontmatter")));
        assert!(diags.iter().any(|d| d.message.contains("unknown layout")));
        assert!(diags.iter().any(|d| d.severity == Severity::Error && d.message.contains("asset not found")));
    }

    #[test]
    fn test_validate_used_legacy_layout_and_template_override() {
        let mut layouts = render::LayoutSet::default();
        layouts.load_builtin();
        layouts.local.insert("slide-x".to_string(), render::Layout::parse_script(
            "slide-x", "class SlideX extends\n  SlideBase{ }",
        ));
        let raw = "---\nlayout: slide-default\n---\n\nBody\n\n---\nlayout: slide-x\n---\n\nBody";
        let work_dir = std::env::temp_dir();
        let diags = validate_deck(raw, &work_dir, &layouts);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].slide, Some(2));
        assert!(diags[0].severity == Severity::Warning);
        assert!(diags[0].message.contains("SlideBase"));

        // Unused legacy files must not warn, and a resolved HTML override is safe.
        let default_deck = "---\nlayout: slide-default\n---\n\nBody";
        assert!(validate_deck(default_deck, &work_dir, &layouts).is_empty());
        layouts.local.insert("slide-x".to_string(), render::Layout::parse_template(
            "slide-x", "<div>{{{content}}}</div>",
        ));
        assert!(validate_deck(raw, &work_dir, &layouts).is_empty());

        // Self-contained Custom Elements do not require the removed base class.
        layouts.local.insert("slide-x".to_string(), render::Layout::parse_script(
            "slide-x", "class SlideX extends HTMLElement { }",
        ));
        assert!(validate_deck(raw, &work_dir, &layouts).is_empty());
    }

    #[test]
    fn test_embed_skips_remote_and_data_uris() {
        let dir = std::env::temp_dir();
        let html = r#"<img src="https://x/y.png"><img src="data:image/png;base64,AAAA">"#;
        // nothing local to read → unchanged
        assert_eq!(embed_assets(html, &dir), html);
    }

    #[test]
    fn test_embed_inlines_local_image() {
        // a 1x1 transparent PNG
        let png: &[u8] = &[
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D,
            0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
            0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00,
            0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
            0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
            0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
        ];
        let dir = std::env::temp_dir().join("wbslide_embed_test");
        std::fs::create_dir_all(dir.join("assets")).unwrap();
        std::fs::write(dir.join("assets/pixel.png"), png).unwrap();

        let html = r#"<img src="assets/pixel.png"><img src='./assets/pixel.png'>"#;
        let out = embed_assets(html, &dir);
        assert!(!out.contains("assets/pixel.png"), "local refs should be replaced");
        assert_eq!(out.matches("data:image/png;base64,").count(), 2);

        let css = "background: url(assets/pixel.png); mask: url('assets/pixel.png');";
        let out_css = embed_assets(css, &dir);
        assert_eq!(out_css.matches("data:image/png;base64,").count(), 2);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_parse_frontmatter_simple() {
        let (meta, body) = parse_frontmatter("layout: slide-feature\nheading: Hello\n\nBody here");
        assert_eq!(meta, vec![
            ("layout".to_string(), "slide-feature".to_string()),
            ("heading".to_string(), "Hello".to_string()),
        ]);
        assert_eq!(body, "Body here");
    }

    #[test]
    fn test_parse_frontmatter_quoted_value() {
        let (meta, _) = parse_frontmatter("title: 'My Title'");
        assert_eq!(meta[0].1, "My Title");
    }

    #[test]
    fn test_parse_frontmatter_colon_in_value() {
        let (meta, _) = parse_frontmatter("subtitle: M365: Office Integration");
        assert_eq!(meta[0].1, "M365: Office Integration");
    }

    #[test]
    fn test_parse_frontmatter_nested_yaml_skipped() {
        let (meta, body) = parse_frontmatter("title: Test\nfonts:\n  sans: Pretendard\n\nBody");
        assert_eq!(meta.len(), 1);
        assert_eq!(meta[0], ("title".to_string(), "Test".to_string()));
        assert_eq!(body, "Body");
    }

    #[test]
    fn test_parse_slots_no_slots() {
        let (default, slots) = parse_slots("<img src=\"test.png\" />");
        assert_eq!(default, "<img src=\"test.png\" />");
        assert!(slots.is_empty());
    }

    #[test]
    fn test_parse_slots_two_slots() {
        let input = "::left::\n\n## Before\n\nOld way\n\n::right::\n\n## After\n\nNew way";
        let (default, slots) = parse_slots(input);
        assert!(default.is_empty());
        assert_eq!(slots.len(), 2);
        assert_eq!(slots[0].0, "left");
        assert!(slots[0].1.contains("Before"));
        assert!(slots[0].1.contains("Old way"));
        assert_eq!(slots[1].0, "right");
        assert!(slots[1].1.contains("After"));
        assert!(slots[1].1.contains("New way"));
    }

    #[test]
    fn test_parse_slots_default_before_slot() {
        let input = "Default content\n\n::sidebar::\n\nSidebar content";
        let (default, slots) = parse_slots(input);
        assert_eq!(default, "Default content");
        assert_eq!(slots.len(), 1);
        assert_eq!(slots[0].0, "sidebar");
        assert!(slots[0].1.contains("Sidebar content"));
    }

    #[test]
    fn test_parse_slides_basic() {
        let raw = "---\ntitle: Test\nlayout: slide-cover\n---\n\n# Cover\n\n---\nlayout: slide-feature\nheading: Feature\n---\n\nBody content";
        let (meta, slides) = parse_slides(raw);
        assert_eq!(meta.iter().find(|(k, _)| k == "title").unwrap().1, "Test");
        assert_eq!(slides.len(), 2);
        assert_eq!(slides[0].frontmatter.iter().find(|(k, _)| k == "layout").unwrap().1, "slide-cover");
        assert!(slides[0].body_html.contains("Cover"));
        assert_eq!(slides[1].frontmatter.iter().find(|(k, _)| k == "layout").unwrap().1, "slide-feature");
        assert!(slides[1].body_html.contains("Body content"));
    }

    #[test]
    fn test_parse_slides_with_slots() {
        let raw = "---\ntitle: Test\n---\n\nDefault\n\n---\nlayout: slide-two-column\n---\n\n::left::\n\nLeft content\n\n::right::\n\nRight content";
        let (_, slides) = parse_slides(raw);
        assert_eq!(slides.len(), 2);
        assert_eq!(slides[1].slots.len(), 2);
        assert_eq!(slides[1].slots[0].0, "left");
        assert!(slides[1].slots[0].1.contains("Left content"));
        assert_eq!(slides[1].slots[1].0, "right");
        assert!(slides[1].slots[1].1.contains("Right content"));
    }

    #[test]
    fn spec_folio_is_chrome_except_on_cover() {
        let src = r#"{"schema_version":1,"title":"t","folio":"A & B","slides":[{"layout":"cover","heading":"c","blocks":[]},{"heading":"h","blocks":[{"type":"text","text":"b"}]}]}"#;
        let mut layouts = render::LayoutSet::default();
        layouts.load_builtin();
        let (meta, slides) = spec::parse(src.as_bytes()).unwrap();
        let html = render_all_slides(&slides, &meta, &layouts).unwrap().html;
        assert_eq!(html.matches(r#"<div class="ms-folio">A &amp; B</div>"#).count(), 1);
        let (meta, slides) = spec::parse(src.replace(r#""folio":"A & B","#, "").as_bytes()).unwrap();
        assert!(!render_all_slides(&slides, &meta, &layouts).unwrap().html.contains("ms-folio"));
    }

    #[test]
    fn spec_export_sizes_diagrams_and_images_without_a_preset() {
        let dir = std::env::temp_dir().join(format!("wbslide_spec_size_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let (input, output) = (dir.join("deck.json"), dir.join("deck.html"));
        std::fs::write(&input, r#"{"schema_version":1,"title":"가상 덱","slides":[{"heading":"사분면","blocks":[{"type":"diagram","diagram":"quadrant","alt":"가상 사분면","data":{"quadrants":[{"title":"가"},{"title":"나"},{"title":"다"},{"title":"라"}]}}]}]}"#).unwrap();
        export_spec(&input, &output).unwrap();
        let html = std::fs::read_to_string(&output).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        assert!(html.contains("<svg class=\"wb-diagram\""));
        assert!(html.contains(".wb-diagram { display: block; justify-self: start; width: 100%; max-width: calc(var(--wb-diagram-height, 300px) * 16 / 9); height: auto; aspect-ratio: 16 / 9; }"), "diagram 16:9 sizing rule missing");
        assert!(html.contains("max-height: var(--wb-image-height, 300px)"), "image height rule missing");
    }

    #[test]
    fn test_first_slide_context_reaches_template() {
        let raw = "---\ntitle: Deck tab\ntheme: test-theme\nlayout: slide-feature\nheading: 첫 장 제목\nsubtitle: 설명\nlabel: Custom\nfooter: Team\n---\n\nBody\n\n---\nlayout: slide-feature\nheading: Next\n---\n\nSecond body";
        let (meta, slides) = parse_slides(raw);
        let mut layouts = render::LayoutSet::default();
        layouts.load_builtin();
        let rendered = render_all_slides(&slides, &meta, &layouts).unwrap();
        assert!(rendered.html.contains("첫 장 제목"));
        assert!(rendered.html.contains("설명"));
        assert_eq!(get_fm(&slides[0], "label").as_deref(), Some("Custom"));
        assert!(get_fm(&slides[0], "title").is_none());
        assert!(get_fm(&slides[0], "theme").is_none());
        assert!(get_fm(&slides[1], "label").is_none());
        assert_eq!(rendered.html.matches("ms-footer-logo\">Team").count(), 2);
    }

    #[test]
    fn test_windows_line_endings_match_rendering_and_validation() {
        let lf = "---\ntitle: Test\nlayout: slide-feature\nheading: 첫 장\n---\n\nBody\n\n---\nlayout: missing-layout\n---\n\n::left::\n\n왼쪽\n\n::right::\n\n오른쪽";
        let crlf = lf.replace('\n', "\r\n");
        let mut layouts = render::LayoutSet::default();
        layouts.load_builtin();
        let (lf_meta, lf_slides) = parse_slides(lf);
        let (crlf_meta, crlf_slides) = parse_slides(&crlf);
        assert_eq!(crlf_slides.len(), 2);
        assert_eq!(lf_meta, crlf_meta);
        assert_eq!(
            render_all_slides(&lf_slides, &lf_meta, &layouts).unwrap().html,
            render_all_slides(&crlf_slides, &crlf_meta, &layouts).unwrap().html,
        );
        let dir = std::env::temp_dir();
        let lf_diags = validate_deck(lf, &dir, &layouts);
        let crlf_diags = validate_deck(&crlf, &dir, &layouts);
        assert_eq!(lf_diags.len(), 1);
        assert_eq!(lf_diags[0].message, crlf_diags[0].message);
        assert_eq!(crlf_diags[0].slide, Some(2));
    }

    #[test]
    fn test_render_markdown_html_passthrough() {
        let html = "<div class=\"flex\"><img src=\"test.png\" /></div>";
        let result = render_markdown(html);
        assert!(result.contains("<img"));
        assert!(result.contains("test.png"));
    }

    #[test]
    fn test_component_tags_in_slots_and_code_examples() {
        let raw = "---\nlayout: slide-two-column\n---\n\n::left::\n\n<wb-panel label=\"왼쪽\"><b>근거</b></wb-panel>\n\n::right::\n\n```html\n<wb-unknown></wb-unknown>\n```";
        let (meta, slides) = parse_slides(raw);
        let mut layouts = render::LayoutSet::default();
        layouts.load_builtin();
        let rendered = render_all_slides(&slides, &meta, &layouts).unwrap();
        assert!(rendered.html.contains("panel-label\">왼쪽</div><b>근거</b>"));
        assert!(!rendered.html.contains("<wb-panel"));
        assert!(rendered.html.contains("&lt;wb-unknown&gt;"));
        let (meta, slides) = parse_slides("---\n---\n\n<wb-unknown></wb-unknown>");
        assert!(render_all_slides(&slides, &meta, &layouts).unwrap_err().contains("slide 1"));
    }

    #[test]
    fn test_render_markdown_basic() {
        let md = "## Hello\n\n**bold** text";
        let result = render_markdown(md);
        assert!(result.contains("<h2>"));
        assert!(result.contains("<strong>bold</strong>"));
    }

    #[test]
    fn test_render_markdown_gfm_table() {
        let md = "| A | B |\n| --- | ---: |\n| 1 | 2 |";
        let result = render_markdown(md);
        assert!(result.contains("<table>"), "GFM tables should render: {result}");
        assert!(result.contains("<th"));
        assert!(result.contains("<td"));
    }

    #[test]
    fn test_render_markdown_gfm_strikethrough() {
        assert!(render_markdown("~~gone~~").contains("<del>"));
    }
}
