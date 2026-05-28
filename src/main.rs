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

#[derive(Embed)]
#[folder = "framework/"]
struct FrameworkAssets;

#[derive(Parser)]
#[command(name = "wb-slide", about = "Lightweight slide presentation framework")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
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
    },
    /// Export to a self-contained HTML file
    Export {
        #[arg(short, long)]
        dir: Option<PathBuf>,
        #[arg(short, long, default_value = "export.html")]
        output: PathBuf,
    },
}

struct Slide {
    frontmatter: Vec<(String, String)>,
    body_html: String,
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

fn render_markdown(text: &str) -> String {
    if text.trim().is_empty() {
        return String::new();
    }
    let mut options = Options::default();
    options.render.unsafe_ = true;
    markdown_to_html(text, &options)
}

fn parse_slides(raw: &str) -> (Vec<(String, String)>, Vec<Slide>) {
    let blocks: Vec<&str> = raw.split("\n---\n").collect();

    // blocks[0] = global frontmatter
    // blocks[1] = slide 1 body
    // blocks[2] = slide 2 frontmatter, blocks[3] = slide 2 body
    // blocks[2n] = slide n+1 frontmatter, blocks[2n+1] = slide n+1 body

    let first_block = blocks[0].trim_start_matches("---\n").trim_start_matches("---\r\n");
    let (global_meta, _) = parse_frontmatter(first_block);

    let mut slides = Vec::new();

    // Slide 1: layout from global frontmatter, body from blocks[1]
    if blocks.len() > 1 {
        let mut fm: Vec<(String, String)> = Vec::new();
        if let Some(layout) = global_meta.iter().find(|(k, _)| k == "layout") {
            fm.push(layout.clone());
        }
        slides.push(Slide {
            frontmatter: fm,
            body_html: render_markdown(blocks[1].trim()),
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

        slides.push(Slide {
            frontmatter: fm,
            body_html: render_markdown(&body),
        });

        i += 2;
    }

    (global_meta, slides)
}

fn get_fm(slide: &Slide, key: &str) -> Option<String> {
    slide.frontmatter.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
}

fn build_slides_json(slides: &[Slide], global_meta: &[(String, String)]) -> String {
    let global_watermark = global_meta.iter().find(|(k, _)| k == "watermark").map(|(_, v)| v.as_str()).unwrap_or("");
    let global_footer = global_meta.iter().find(|(k, _)| k == "footer").map(|(_, v)| v.as_str()).unwrap_or("");

    let arr: Vec<String> = slides.iter().enumerate().map(|(i, slide)| {
        let layout = get_fm(slide, "layout").unwrap_or_else(|| "slide-default".to_string());
        let mut attrs: Vec<String> = slide.frontmatter.iter()
            .filter(|(k, _)| k != "layout")
            .map(|(k, v)| {
                let escaped_v = v.replace('\\', "\\\\").replace('"', "\\\"");
                format!("\"{}\":\"{}\"", k, escaped_v)
            })
            .collect();
        if !global_watermark.is_empty() && get_fm(slide, "watermark").is_none() {
            attrs.push(format!("\"watermark\":\"{}\"", global_watermark));
        }
        if !global_footer.is_empty() && get_fm(slide, "footer").is_none() {
            attrs.push(format!("\"footer\":\"{}\"", global_footer));
        }
        let body_escaped = serde_json::to_string(&slide.body_html).unwrap();
        format!(
            "{{\"layout\":\"{}\",\"index\":{},\"attrs\":{{{}}},\"body\":{}}}",
            layout,
            i + 1,
            attrs.join(","),
            body_escaped
        )
    }).collect();
    format!("[{}]", arr.join(","))
}

struct HtmlOptions<'a> {
    title: &'a str,
    slides_json: &'a str,
    framework_css: &'a str,
    framework_js: &'a str,
    layout_js: &'a str,
    user_css: Option<&'a str>,
    user_layouts: Option<&'a str>,
}

fn build_index_html(opts: &HtmlOptions) -> String {
    let user_css_tag = opts.user_css
        .map(|css| format!("<style>{css}</style>"))
        .unwrap_or_default();
    let user_layouts_tag = opts.user_layouts
        .map(|js| format!("\n{js}"))
        .unwrap_or_default();

    format!(
        r#"<!DOCTYPE html>
<html lang="ko">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{title}</title>
  <style>{framework_css}</style>
  {user_css_tag}
</head>
<body>
  <monocle-slide></monocle-slide>
  <script>window.__MONOCLE_SLIDES__ = {slides_json};</script>
  <script type="module">{layout_js}{user_layouts_tag}

{framework_js}</script>
</body>
</html>"#,
        title = opts.title,
        framework_css = opts.framework_css,
        slides_json = opts.slides_json,
        layout_js = opts.layout_js,
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

fn collect_layout_js() -> String {
    let mut js = String::new();
    if let Some(file) = FrameworkAssets::get("slide-base.js") {
        js.push_str(&String::from_utf8_lossy(&file.data));
        js.push('\n');
    }
    for name in FrameworkAssets::iter() {
        if name.starts_with("layouts/") && name.ends_with(".js") {
            if let Some(file) = FrameworkAssets::get(&name) {
                let content = String::from_utf8_lossy(&file.data);
                let content = content.replace("import { SlideBase } from '../slide-base.js';", "");
                js.push_str(&content);
                js.push('\n');
            }
        }
    }
    js
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

async fn serve_index(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Html<String> {
    let raw = std::fs::read_to_string(&state.slides_path)
        .unwrap_or_else(|_| "---\ntitle: No slides found\n---\n\n# No slides.md found".to_string());

    let (global_meta, slides) = parse_slides(&raw);
    let slides_json = build_slides_json(&slides, &global_meta);
    let title = global_meta.iter().find(|(k, _)| k == "title")
        .map(|(_, v)| v.as_str()).unwrap_or("WB Slide");

    let user_css = collect_user_css(&state.work_dir);
    let user_layouts = collect_user_layouts(&state.work_dir);
    let framework_css = collect_framework_css();
    let framework_js = collect_framework_js();
    let layout_js = collect_layout_js();

    Html(build_index_html(&HtmlOptions {
        title,
        slides_json: &slides_json,
        framework_css: &framework_css,
        framework_js: &framework_js,
        layout_js: &layout_js,
        user_css: user_css.as_deref(),
        user_layouts: user_layouts.as_deref(),
    }))
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
}

fn resolve_state(dir: Option<PathBuf>) -> AppState {
    let work_dir = dir.unwrap_or_else(|| std::env::current_dir().unwrap());
    let work_dir = work_dir.canonicalize().unwrap_or(work_dir);

    let slides_path = if work_dir.join("slides.md").exists() {
        work_dir.join("slides.md")
    } else {
        work_dir.join("index.md")
    };

    AppState { work_dir, slides_path }
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

fn collect_user_layouts(work_dir: &std::path::Path) -> Option<String> {
    let layouts_dir = work_dir.join("layouts");
    if !layouts_dir.is_dir() {
        return None;
    }
    let mut js = String::new();
    if let Ok(entries) = std::fs::read_dir(&layouts_dir) {
        let mut paths: Vec<_> = entries.filter_map(|e| e.ok()).map(|e| e.path()).collect();
        paths.sort();
        for path in paths {
            if path.extension().map_or(false, |ext| ext == "js") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let content = content.replace("import { SlideBase } from '../slide-base.js';", "");
                    let content = content.replace("import { SlideBase } from './slide-base.js';", "");
                    js.push_str(&content);
                    js.push('\n');
                }
            }
        }
    }
    if js.is_empty() { None } else { Some(js) }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Show { port, dir, no_open } => {
            let state = resolve_state(dir);

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

        Commands::Export { dir, output } => {
            let state = resolve_state(dir);

            if !state.slides_path.exists() {
                eprintln!("Error: No slides.md found in {}", state.work_dir.display());
                std::process::exit(1);
            }

            let raw = std::fs::read_to_string(&state.slides_path).unwrap();
            let (global_meta, slides) = parse_slides(&raw);
            let slides_json = build_slides_json(&slides, &global_meta);
            let title = global_meta.iter().find(|(k, _)| k == "title")
                .map(|(_, v)| v.as_str()).unwrap_or("WB Slide");

            let user_css = collect_user_css(&state.work_dir);
            let user_layouts = collect_user_layouts(&state.work_dir);
            let framework_css = collect_framework_css();
            let framework_js = collect_framework_js();
            let layout_js = collect_layout_js();

            let html = build_index_html(&HtmlOptions {
                title,
                slides_json: &slides_json,
                framework_css: &framework_css,
                framework_js: &framework_js,
                layout_js: &layout_js,
                user_css: user_css.as_deref(),
                user_layouts: user_layouts.as_deref(),
            });

            let output_path = if output.is_absolute() {
                output
            } else {
                state.work_dir.join(output)
            };

            std::fs::write(&output_path, html).unwrap();
            eprintln!("Exported to: {}", output_path.display());
        }
    }
}
