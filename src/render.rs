//! Server-side rendering of slides to static HTML.
//!
//! Replaces the previous Custom-Element-based client rendering. Layouts are
//! HTML templates (mustache subset); the Rust server resolves the layout,
//! fills the template, wraps with chrome, and emits plain HTML.
//!
//! Layout resolution order (highest precedence first):
//!   1. Local `./layouts/<name>.html` in the deck working directory
//!   2. Theme-provided layouts (via theme.json `layouts` list)
//!   3. Framework built-in (embedded in the binary)

use crate::template::{render as render_template, Context};
use crate::theme::LoadedTheme;
use crate::FrameworkAssets;
use std::collections::HashMap;
use std::path::Path;

/// A layout's source: either an HTML template (rendered server-side) or a
/// JavaScript Web Component (rendered client-side; escape hatch for layouts
/// that genuinely need JS).
#[derive(Debug, Clone)]
pub enum LayoutSource {
    Template(String),
    Script(String),
}

/// A layout + its metadata.
#[derive(Debug, Clone)]
pub struct Layout {
    pub name: String,
    pub source: LayoutSource,
    pub chrome: bool,
}

impl Layout {
    /// Parse an HTML layout file: optional YAML-ish frontmatter then template body.
    pub fn parse_template(name: &str, source: &str) -> Self {
        let (meta, body) = split_frontmatter(source);
        let chrome = meta.get("chrome").map(|v| v != "false").unwrap_or(true);
        Self {
            name: name.to_string(),
            source: LayoutSource::Template(body),
            chrome,
        }
    }

    /// Parse a JS layout file. Chrome defaults to true; layouts can opt out at
    /// runtime via the JS class (showChrome) — but we still wrap chrome with
    /// HTML to keep SSR / client paths consistent.
    pub fn parse_script(name: &str, source: &str) -> Self {
        // Heuristic: look for "showChrome() { return false" in the source.
        let chrome = !source.contains("showChrome()") || !source.contains("return false");
        Self {
            name: name.to_string(),
            source: LayoutSource::Script(source.to_string()),
            chrome,
        }
    }
}

fn split_frontmatter(s: &str) -> (HashMap<String, String>, String) {
    let trimmed_start = s.trim_start_matches('\u{feff}');
    if !trimmed_start.starts_with("---\n") && !trimmed_start.starts_with("---\r\n") {
        return (HashMap::new(), s.to_string());
    }
    let after_open = &trimmed_start[trimmed_start.find('\n').unwrap() + 1..];
    if let Some(end_idx) = find_frontmatter_close(after_open) {
        let meta_text = &after_open[..end_idx];
        let body_start = end_idx + after_open[end_idx..].find('\n').map(|i| i + 1).unwrap_or(0);
        let body = if body_start <= after_open.len() {
            &after_open[body_start..]
        } else {
            ""
        };
        let mut meta = HashMap::new();
        for line in meta_text.lines() {
            if let Some(idx) = line.find(':') {
                let k = line[..idx].trim().to_string();
                let v = line[idx + 1..].trim().to_string();
                if !k.is_empty() {
                    meta.insert(k, v);
                }
            }
        }
        return (meta, body.to_string());
    }
    (HashMap::new(), s.to_string())
}

fn find_frontmatter_close(s: &str) -> Option<usize> {
    let mut pos = 0;
    while pos < s.len() {
        let rest = &s[pos..];
        if rest.starts_with("---\n") || rest.starts_with("---\r\n") || rest == "---" {
            return Some(pos);
        }
        match rest.find('\n') {
            Some(i) => pos += i + 1,
            None => return None,
        }
    }
    None
}

/// A repository of available layouts, resolved in precedence order.
#[derive(Debug, Default, Clone)]
pub struct LayoutSet {
    pub local: HashMap<String, Layout>,
    pub theme: HashMap<String, Layout>,
    pub builtin: HashMap<String, Layout>,
}

impl LayoutSet {
    pub fn resolve(&self, name: &str) -> Option<&Layout> {
        self.local
            .get(name)
            .or_else(|| self.theme.get(name))
            .or_else(|| self.builtin.get(name))
    }

    /// Load built-in layouts from the embedded `framework/layouts/*.html`.
    pub fn load_builtin(&mut self) {
        for asset_name in FrameworkAssets::iter() {
            if !asset_name.starts_with("layouts/") {
                continue;
            }
            let file = match FrameworkAssets::get(&asset_name) {
                Some(f) => f,
                None => continue,
            };
            let content = String::from_utf8_lossy(&file.data).to_string();
            let stem = asset_name.strip_prefix("layouts/").unwrap_or("");
            if let Some(name) = stem.strip_suffix(".html") {
                self.builtin
                    .insert(name.to_string(), Layout::parse_template(name, &content));
            }
            // We intentionally ignore built-in .js layouts now — built-ins are
            // SSR-only via the template format.
        }
    }

    /// Load deck-local layouts from `<work_dir>/layouts/*.{html,js}`. `.html`
    /// has precedence over `.js` for the same name.
    pub fn load_local(&mut self, work_dir: &Path) {
        let layouts_dir = work_dir.join("layouts");
        if !layouts_dir.is_dir() {
            return;
        }
        if let Ok(entries) = std::fs::read_dir(&layouts_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                let stem = match path.file_stem().and_then(|s| s.to_str()) {
                    Some(s) => s.to_string(),
                    None => continue,
                };
                let content = match std::fs::read_to_string(&path) {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                match ext {
                    "html" => {
                        self.local
                            .insert(stem.clone(), Layout::parse_template(&stem, &content));
                    }
                    "js" => {
                        // .js only inserted if no .html for the same name was loaded.
                        if !self.local.contains_key(&stem) {
                            self.local
                                .insert(stem.clone(), Layout::parse_script(&stem, &content));
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    /// Load theme layouts from a `LoadedTheme`. Themes list .html (preferred)
    /// or .js (escape hatch) files in their manifest; theme.rs already
    /// distinguishes by extension.
    pub fn load_theme(&mut self, theme: &LoadedTheme) {
        for (name, source) in &theme.layouts {
            // Theme entries are stored with their original extension implicit.
            // Detect: starts with frontmatter or HTML tag → template, otherwise script.
            let layout = if looks_like_template(source) {
                Layout::parse_template(name, source)
            } else {
                Layout::parse_script(name, source)
            };
            self.theme.insert(name.clone(), layout);
        }
    }
}

fn looks_like_template(src: &str) -> bool {
    let trimmed = src.trim_start();
    trimmed.starts_with("---")
        || trimmed.starts_with('<')
        || trimmed.starts_with("{{")
}

/// Result of rendering all slides: the HTML to inline + any client-side JS
/// snippets (for `.js` layouts using the escape hatch).
#[derive(Debug, Default)]
pub struct RenderedSlides {
    pub html: String,
    /// Extra JS that must run on the client (Custom Element definitions for
    /// `.js` escape-hatch layouts).
    pub client_js: String,
}

/// Render one slide. Returns (slide_html, optional_client_js).
pub fn render_slide(
    layout_name: &str,
    attrs: &[(String, String)],
    slots: &[(String, String)],
    body_html: &str,
    index: usize,
    total: usize,
    layouts: &LayoutSet,
) -> (String, Option<String>) {
    let layout = layouts
        .resolve(layout_name)
        .or_else(|| layouts.resolve("slide-default"));

    let watermark = attrs
        .iter()
        .find(|(k, _)| k == "watermark")
        .map(|(_, v)| v.as_str())
        .unwrap_or("");
    let footer = attrs
        .iter()
        .find(|(k, _)| k == "footer")
        .map(|(_, v)| v.as_str())
        .unwrap_or("");

    let show_chrome = layout.map(|l| l.chrome).unwrap_or(true);
    let chrome_html = if show_chrome {
        let mut s = String::new();
        if !watermark.is_empty() {
            s.push_str(&format!(r#"<div class="ms-watermark">{}</div>"#, watermark));
        }
        if !footer.is_empty() {
            s.push_str(&format!(r#"<div class="ms-footer-logo">{}</div>"#, footer));
        }
        s.push_str(&format!(r#"<div class="ms-page-number">{}</div>"#, index));
        s
    } else {
        String::new()
    };

    let layout_class = layout
        .map(|l| l.name.clone())
        .unwrap_or_else(|| "slide-default".to_string());
    let active_class = if index == 1 { " active" } else { "" };

    match layout.map(|l| &l.source) {
        Some(LayoutSource::Template(tpl)) => {
            let mut ctx = Context::new();
            for (k, v) in attrs {
                ctx.set(k, v.clone());
            }
            for (name, html) in slots {
                ctx.set_slot(name, html.clone());
            }
            ctx.set("content", body_html.to_string());
            ctx.set("slide-index", index.to_string());
            ctx.set("total-slides", total.to_string());
            ctx.set_slot("content", body_html.to_string());

            let body_html_out = render_template(tpl, &ctx);
            let html = format!(
                r#"<div class="ms-slide-container {layout_class}{active_class}" data-slide-index="{index}">{chrome_html}{body_html_out}</div>"#,
            );
            (html, None)
        }
        Some(LayoutSource::Script(js)) => {
            // Escape hatch: emit a Custom Element. Chrome is emitted as a
            // wrapper so that even if the CE fails to upgrade, the chrome
            // still shows.
            let attr_str: String = attrs
                .iter()
                .filter(|(k, _)| k != "layout")
                .map(|(k, v)| {
                    format!(r#" {}="{}""#, k, html_attr_escape(v))
                })
                .collect();
            let html = format!(
                r#"<div class="ms-slide-container {layout_class}{active_class}" data-slide-index="{index}">{chrome_html}<{layout_name}{attr_str} slide-index="{index}" total-slides="{total}">{body_html}</{layout_name}></div>"#,
            );
            (html, Some(js.clone()))
        }
        None => {
            // No layout found at all. Render body as-is.
            let html = format!(
                r#"<div class="ms-slide-container {layout_class}{active_class}" data-slide-index="{index}">{chrome_html}{body_html}</div>"#,
            );
            (html, None)
        }
    }
}

fn html_attr_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('"', "&quot;").replace('<', "&lt;")
}
