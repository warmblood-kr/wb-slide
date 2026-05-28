use std::path::PathBuf;

const DEFAULT_REGISTRY_INDEX: &str =
    "https://warmblood-kr.github.io/wb-slide-registry/index.json";

const CACHE_TTL_SECS: u64 = 3600;

#[derive(Debug, Clone)]
pub enum ThemeSpec {
    /// theme: monocle-brochure
    Name(String),
    /// theme: https://example.com/theme.json
    Url(String),
}

impl ThemeSpec {
    pub fn parse(s: &str) -> Self {
        if s.starts_with("http://") || s.starts_with("https://") {
            ThemeSpec::Url(s.to_string())
        } else {
            ThemeSpec::Name(s.to_string())
        }
    }
}

/// A loaded theme: aggregated JS and CSS ready to inline.
#[derive(Debug, Default, Clone)]
pub struct LoadedTheme {
    pub name: String,
    pub version: String,
    pub js: String,
    pub css: String,
}

fn cache_dir() -> PathBuf {
    if let Some(dir) = dirs_cache_dir() {
        dir.join("wb-slide")
    } else {
        std::env::temp_dir().join("wb-slide")
    }
}

fn dirs_cache_dir() -> Option<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
        if !xdg.is_empty() {
            return Some(PathBuf::from(xdg));
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        return Some(PathBuf::from(home).join(".cache"));
    }
    None
}

fn cache_path_for(url: &str) -> PathBuf {
    let safe: String = url
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    cache_dir().join(format!("{safe}.cache"))
}

fn is_fresh(path: &std::path::Path, ttl: u64) -> bool {
    if let Ok(meta) = std::fs::metadata(path) {
        if let Ok(modified) = meta.modified() {
            if let Ok(age) = modified.elapsed() {
                return age.as_secs() < ttl;
            }
        }
    }
    false
}

async fn fetch_text(url: &str) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .user_agent(concat!("wb-slide/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("client init: {e}"))?;

    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("fetch {url}: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("fetch {url}: HTTP {}", resp.status()));
    }
    resp.text().await.map_err(|e| format!("read {url}: {e}"))
}

/// Fetch a URL, using local cache if fresh, falling back to cache on network failure.
async fn fetch_cached(url: &str, refresh: bool) -> Result<String, String> {
    let cache = cache_path_for(url);
    let _ = std::fs::create_dir_all(cache_dir());

    if !refresh && is_fresh(&cache, CACHE_TTL_SECS) {
        if let Ok(text) = std::fs::read_to_string(&cache) {
            return Ok(text);
        }
    }

    match fetch_text(url).await {
        Ok(text) => {
            let _ = std::fs::write(&cache, &text);
            Ok(text)
        }
        Err(e) => {
            if let Ok(text) = std::fs::read_to_string(&cache) {
                eprintln!("warning: using stale cache for {url} ({e})");
                Ok(text)
            } else {
                Err(e)
            }
        }
    }
}

fn extract_string_field(json: &str, key: &str) -> Option<String> {
    let pat = format!("\"{key}\"");
    let start = json.find(&pat)?;
    let after = &json[start + pat.len()..];
    let colon = after.find(':')?;
    let mut rest = after[colon + 1..].trim_start();
    if !rest.starts_with('"') {
        return None;
    }
    rest = &rest[1..];
    let mut out = String::new();
    let mut chars = rest.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(next) = chars.next() {
                out.push(next);
            }
            continue;
        }
        if c == '"' {
            return Some(out);
        }
        out.push(c);
    }
    None
}

fn extract_string_array(json: &str, key: &str) -> Vec<String> {
    let pat = format!("\"{key}\"");
    let Some(start) = json.find(&pat) else { return Vec::new() };
    let after = &json[start + pat.len()..];
    let Some(colon) = after.find(':') else { return Vec::new() };
    let rest = after[colon + 1..].trim_start();
    if !rest.starts_with('[') {
        return Vec::new();
    }
    let Some(end) = rest.find(']') else { return Vec::new() };
    let inside = &rest[1..end];

    let mut items = Vec::new();
    let mut in_str = false;
    let mut esc = false;
    let mut buf = String::new();
    for c in inside.chars() {
        if esc {
            buf.push(c);
            esc = false;
            continue;
        }
        if in_str {
            if c == '\\' {
                esc = true;
            } else if c == '"' {
                items.push(buf.clone());
                buf.clear();
                in_str = false;
            } else {
                buf.push(c);
            }
        } else if c == '"' {
            in_str = true;
        }
    }
    items
}

/// Resolve a theme name → theme.json URL using the registry index.
async fn resolve_theme_url(name: &str, refresh: bool) -> Result<String, String> {
    let index_text = fetch_cached(DEFAULT_REGISTRY_INDEX, refresh).await?;

    let themes_start = index_text
        .find("\"themes\"")
        .ok_or_else(|| "registry index has no \"themes\" field".to_string())?;
    let themes_section = &index_text[themes_start..];

    let pat = format!("\"{name}\"");
    let entry_start = themes_section
        .find(&pat)
        .ok_or_else(|| format!("theme \"{name}\" not found in registry"))?;
    let entry_section = &themes_section[entry_start..];

    extract_string_field(entry_section, "url")
        .ok_or_else(|| format!("theme \"{name}\" has no url field in registry"))
}

/// Load a theme: fetch manifest, fetch all listed JS and CSS files, concatenate.
pub async fn load_theme(spec: &ThemeSpec, refresh: bool) -> Result<LoadedTheme, String> {
    let manifest_url = match spec {
        ThemeSpec::Url(url) => url.clone(),
        ThemeSpec::Name(name) => resolve_theme_url(name, refresh).await?,
    };

    let manifest = fetch_cached(&manifest_url, refresh).await?;

    let name = extract_string_field(&manifest, "name").unwrap_or_else(|| "unknown".to_string());
    let version = extract_string_field(&manifest, "version").unwrap_or_default();

    let base_url = manifest_url
        .rsplit_once('/')
        .map(|(prefix, _)| prefix.to_string())
        .unwrap_or_default();

    let layouts = extract_string_array(&manifest, "layouts");
    let styles = extract_string_array(&manifest, "styles");

    let mut loaded = LoadedTheme {
        name: name.clone(),
        version,
        js: String::new(),
        css: String::new(),
    };

    for rel in &layouts {
        let url = format!("{base_url}/{rel}");
        let content = fetch_cached(&url, refresh).await
            .map_err(|e| format!("theme {name}: layout {rel}: {e}"))?;
        let stripped = strip_slidebase_import(&content);
        let guarded = guard_custom_elements_define(&stripped);
        loaded.js.push_str(&wrap_in_iife(&guarded));
        loaded.js.push('\n');
    }

    for rel in &styles {
        let url = format!("{base_url}/{rel}");
        let content = fetch_cached(&url, refresh).await
            .map_err(|e| format!("theme {name}: style {rel}: {e}"))?;
        loaded.css.push_str(&content);
        loaded.css.push('\n');
    }

    Ok(loaded)
}

fn strip_slidebase_import(js: &str) -> String {
    js.lines()
        .filter(|line| !line.trim_start().starts_with("import "))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Wrap a layout JS file in an IIFE so class declarations don't leak into the
/// surrounding module scope (preventing redeclaration errors when multiple
/// sources define a layout with the same class name).
pub fn wrap_in_iife(js: &str) -> String {
    format!("(() => {{\n{js}\n}})();\n")
}

/// Wrap `customElements.define(name, ...)` calls so they no-op when the
/// element is already registered. Lets users override theme-provided
/// layouts by defining their own first.
pub fn guard_custom_elements_define(js: &str) -> String {
    let mut out = String::with_capacity(js.len());
    let needle = "customElements.define(";
    let mut rest = js;
    loop {
        match rest.find(needle) {
            None => {
                out.push_str(rest);
                break;
            }
            Some(idx) => {
                out.push_str(&rest[..idx]);
                let after_open = &rest[idx + needle.len()..];

                // Find the matching close paren, respecting nested parens and strings.
                let mut depth = 1usize;
                let mut in_str: Option<char> = None;
                let mut escape = false;
                let mut end = None;
                for (i, c) in after_open.char_indices() {
                    if escape {
                        escape = false;
                        continue;
                    }
                    if let Some(q) = in_str {
                        if c == '\\' {
                            escape = true;
                        } else if c == q {
                            in_str = None;
                        }
                        continue;
                    }
                    match c {
                        '"' | '\'' | '`' => in_str = Some(c),
                        '(' => depth += 1,
                        ')' => {
                            depth -= 1;
                            if depth == 0 {
                                end = Some(i);
                                break;
                            }
                        }
                        _ => {}
                    }
                }

                match end {
                    None => {
                        // Malformed input; emit as-is and stop.
                        out.push_str(needle);
                        out.push_str(after_open);
                        break;
                    }
                    Some(close_idx) => {
                        let inner = &after_open[..close_idx];
                        // Try to extract the first argument (the element name)
                        let trimmed = inner.trim_start();
                        let (quote, name_start) = match trimmed.chars().next() {
                            Some(q @ '"') | Some(q @ '\'') | Some(q @ '`') => (q, 1),
                            _ => {
                                out.push_str(needle);
                                out.push_str(inner);
                                out.push(')');
                                rest = &after_open[close_idx + 1..];
                                continue;
                            }
                        };
                        let after_quote = &trimmed[name_start..];
                        let Some(end_quote) = after_quote.find(quote) else {
                            out.push_str(needle);
                            out.push_str(inner);
                            out.push(')');
                            rest = &after_open[close_idx + 1..];
                            continue;
                        };
                        let name = &after_quote[..end_quote];

                        out.push_str("(customElements.get(");
                        out.push(quote);
                        out.push_str(name);
                        out.push(quote);
                        out.push_str(") || customElements.define(");
                        out.push_str(inner);
                        out.push_str("))");

                        rest = &after_open[close_idx + 1..];
                    }
                }
            }
        }
    }
    out
}
