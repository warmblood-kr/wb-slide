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
#[folder = "../framework/"]
struct FrameworkAssets;

#[derive(Parser)]
#[command(name = "monocle-slide", about = "Lightweight slide presentation framework")]
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
    if text.trim_start().starts_with('<') {
        return text.to_string();
    }
    let mut options = Options::default();
    options.render.unsafe_ = true;
    markdown_to_html(text, &options)
}

fn parse_slides(raw: &str) -> (Vec<(String, String)>, Vec<Slide>) {
    let blocks: Vec<&str> = raw.split("\n---\n").collect();

    let first_block = blocks[0].trim_start_matches("---\n").trim_start_matches("---\r\n");
    let (global_meta, first_body) = parse_frontmatter(first_block);

    let mut slides = Vec::new();

    if !first_body.is_empty() {
        slides.push(Slide {
            frontmatter: Vec::new(),
            body_html: render_markdown(&first_body),
        });
    }

    for block in &blocks[1..] {
        let (fm, body) = parse_frontmatter(block);
        if !body.is_empty() || !fm.is_empty() {
            slides.push(Slide {
                frontmatter: fm,
                body_html: render_markdown(&body),
            });
        }
    }

    (global_meta, slides)
}

fn get_fm(slide: &Slide, key: &str) -> Option<String> {
    slide.frontmatter.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
}

fn build_slides_json(slides: &[Slide]) -> String {
    let arr: Vec<String> = slides.iter().enumerate().map(|(i, slide)| {
        let layout = get_fm(slide, "layout").unwrap_or_else(|| "slide-default".to_string());
        let attrs: Vec<String> = slide.frontmatter.iter()
            .filter(|(k, _)| k != "layout")
            .map(|(k, v)| {
                let escaped_v = v.replace('\\', "\\\\").replace('"', "\\\"");
                format!("\"{}\":\"{}\"", k, escaped_v)
            })
            .collect();
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

fn build_index_html(slides_json: &str, custom_css: Option<&str>, framework_css: &str, framework_js: &str, layout_js: &str) -> String {
    let custom_css_tag = custom_css
        .map(|css| format!("<style>{css}</style>"))
        .unwrap_or_default();

    format!(
        r#"<!DOCTYPE html>
<html lang="ko">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Monocle Slide</title>
  <style>{framework_css}</style>
  {custom_css_tag}
</head>
<body>
  <monocle-slide></monocle-slide>
  <script>window.__MONOCLE_SLIDES__ = {slides_json};</script>
  <script type="module">{layout_js}

{framework_js}</script>
</body>
</html>"#
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

    let (_, slides) = parse_slides(&raw);
    let slides_json = build_slides_json(&slides);

    let custom_css = state.custom_css_path
        .as_ref()
        .and_then(|p| std::fs::read_to_string(p).ok());

    let framework_css = collect_framework_css();
    let framework_js = collect_framework_js();
    let layout_js = collect_layout_js();

    Html(build_index_html(
        &slides_json,
        custom_css.as_deref(),
        &framework_css,
        &framework_js,
        &layout_js,
    ))
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
    custom_css_path: Option<PathBuf>,
}

fn resolve_state(dir: Option<PathBuf>) -> AppState {
    let work_dir = dir.unwrap_or_else(|| std::env::current_dir().unwrap());
    let work_dir = work_dir.canonicalize().unwrap_or(work_dir);

    let slides_path = if work_dir.join("slides.md").exists() {
        work_dir.join("slides.md")
    } else {
        work_dir.join("index.md")
    };

    let custom_css_path = {
        let p = work_dir.join("styles/custom.css");
        if p.exists() { Some(p) } else { None }
    };

    AppState { work_dir, slides_path, custom_css_path }
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
            eprintln!("  Monocle Slide v{}", env!("CARGO_PKG_VERSION"));
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
            let (_, slides) = parse_slides(&raw);
            let slides_json = build_slides_json(&slides);

            let custom_css = state.custom_css_path
                .as_ref()
                .and_then(|p| std::fs::read_to_string(p).ok());

            let framework_css = collect_framework_css();
            let framework_js = collect_framework_js();
            let layout_js = collect_layout_js();

            let html = build_index_html(
                &slides_json,
                custom_css.as_deref(),
                &framework_css,
                &framework_js,
                &layout_js,
            );

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
