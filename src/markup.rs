//! Semantic slide markup -> typed spec v1 JSON, plus the themeable slide preset.
//! Closed vocabulary: unknown tags, attributes or stray text are errors.
use html5gum::{DefaultEmitter, Token, Tokenizer};
use serde_json::json;

type Result<T> = std::result::Result<T, String>;

const ROLE_CSS: &str = include_str!("../framework/presets/slide.css");
const DEFAULT_THEME: &str = include_str!("../framework/presets/themes/default.css");
const NIGHT_THEME: &str = include_str!("../framework/presets/themes/night.css");
const MAX_THEME_BYTES: usize = 8 * 1024;

fn collapse(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn finish(heading: Option<String>, body: &str, index: usize) -> Result<serde_json::Value> {
    let heading = heading.ok_or_else(|| {
        format!("wb-slide[{index}]: <wb-title> is required for now (centered title-less slides need a spec layout; see #23)")
    })?;
    // Blank lines separate paragraphs; other whitespace collapses as in HTML.
    let blocks: Vec<_> = body
        .replace("\r\n", "\n")
        .split("\n\n")
        .map(collapse)
        .filter(|p| !p.is_empty())
        .map(|text| json!({"type": "text", "text": text}))
        .collect();
    if blocks.is_empty() {
        return Err(format!("wb-slide[{index}]: body text is required"));
    }
    Ok(json!({"heading": heading, "blocks": blocks}))
}

/// Compile markup to typed spec JSON. Same input, same bytes.
pub fn compile(markup: &str) -> Result<String> {
    enum At { Deck, Slide, Title }
    let mut at = At::Deck;
    let (mut slides, mut heading, mut title, mut body) = (Vec::new(), None, String::new(), String::new());
    let emitter = DefaultEmitter::<usize>::new_with_span();
    for token in Tokenizer::new_with_emitter(markup, emitter) {
        let token = token.map_err(|e| e.to_string())?;
        let (name, pos, is_end) = match &token {
            Token::String(s) => {
                let text = String::from_utf8_lossy(&s.value);
                match at {
                    At::Deck if text.trim().is_empty() => {}
                    At::Deck => return Err(format!("text outside <wb-slide> at byte {}", s.span.start)),
                    At::Slide => body.push_str(&text),
                    At::Title => title.push_str(&text),
                }
                continue;
            }
            Token::Comment(_) | Token::Doctype(_) => continue,
            Token::Error(e) => return Err(format!("markup error at byte {}: {}", e.span.start, e.value)),
            Token::StartTag(t) => {
                if !t.attributes.is_empty() || t.self_closing {
                    return Err(format!("unsupported attributes or self-closing tag at byte {}", t.span.start));
                }
                (String::from_utf8_lossy(&t.name).to_string(), t.span.start, false)
            }
            Token::EndTag(t) => (String::from_utf8_lossy(&t.name).to_string(), t.span.start, true),
        };
        at = match (at, name.as_str(), is_end) {
            (At::Deck, "wb-slide", false) => At::Slide,
            (At::Slide, "wb-title", false) if heading.is_none() && body.trim().is_empty() => At::Title,
            (At::Title, "wb-title", true) => {
                let t = collapse(&title);
                if t.is_empty() {
                    return Err(format!("empty <wb-title> at byte {pos}"));
                }
                heading = Some(t);
                At::Slide
            }
            (At::Slide, "wb-slide", true) => {
                slides.push(finish(heading.take(), &body, slides.len())?);
                (title, body) = (String::new(), String::new());
                At::Deck
            }
            _ => return Err(format!("unexpected {}<{name}> at byte {pos}", if is_end { "closing " } else { "" })),
        };
    }
    if !matches!(at, At::Deck) {
        return Err("unclosed <wb-slide> or <wb-title> at end of input".into());
    }
    let deck_title = slides.first().ok_or("no <wb-slide> found")?["heading"].clone();
    let spec = json!({"schema_version": 1, "title": deck_title, "slides": slides});
    Ok(serde_json::to_string_pretty(&spec).unwrap())
}

fn token_names() -> Vec<&'static str> {
    DEFAULT_THEME.lines().filter_map(|l| l.split(':').next()).map(str::trim).filter(|n| !n.is_empty()).collect()
}

/// Validate a token-only theme: `--wb-name: value;` lines with known names.
/// No functions, at-rules or braces, so a theme cannot load URLs or inject rules.
fn tokens(source: &str, origin: &str) -> Result<String> {
    let names = token_names();
    let mut out = String::new();
    for (i, line) in source.lines().enumerate().filter(|(_, l)| !l.trim().is_empty()) {
        let at = format!("{origin}:{}", i + 1);
        let (name, value) = line.trim().strip_suffix(';').and_then(|l| l.split_once(':'))
            .ok_or(format!("{at}: expected `--wb-name: value;`"))?;
        let (name, value) = (name.trim(), value.trim());
        if !names.contains(&name) {
            return Err(format!("{at}: unknown token {name}"));
        }
        if value.is_empty() || value.contains(|c| "(){};<>@\\\"".contains(c)) {
            return Err(format!("{at}: value must be plain (no functions, quotes other than ', or braces)"));
        }
        out.push_str(&format!("  {name}: {value};\n"));
    }
    Ok(out)
}

/// Preset CSS: default tokens, then the chosen theme (built-in name or token file), then role styles.
pub fn preset_css(theme: Option<&str>) -> Result<String> {
    let overrides = match theme {
        None | Some("default") => String::new(),
        Some("night") => tokens(NIGHT_THEME, "night")?,
        Some(path) => {
            let source = std::fs::read(path).map_err(|e| format!("cannot read theme {path}: {e}"))?;
            if source.len() > MAX_THEME_BYTES {
                return Err(format!("theme {path} exceeds 8 KiB"));
            }
            tokens(&String::from_utf8(source).map_err(|_| format!("theme {path}: not UTF-8"))?, path)?
        }
    };
    Ok(format!(":root {{\n{}{overrides}}}\n{ROLE_CSS}", tokens(DEFAULT_THEME, "default")?))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(m: &str) -> serde_json::Value {
        serde_json::from_str(&compile(m).unwrap()).unwrap()
    }

    #[test]
    fn title_and_body_compile_to_valid_spec() {
        let s = spec("<wb-slide><wb-title> 가상   검토 </wb-title>\n hello &amp; <b>\n\n둘째 문단 </wb-slide>"
            .replace("<b>", "&lt;b&gt;").as_str());
        assert_eq!(s["title"], "가상 검토");
        assert_eq!(s["slides"][0]["heading"], "가상 검토");
        assert_eq!(s["slides"][0]["blocks"][0]["text"], "hello & <b>");
        assert_eq!(s["slides"][0]["blocks"][1]["text"], "둘째 문단");
        let json = compile("<wb-slide><wb-title>A</wb-title>x</wb-slide><wb-slide><wb-title>B</wb-title>y</wb-slide>").unwrap();
        assert!(crate::spec::parse(json.as_bytes()).is_ok());
        assert_eq!(json, compile("<wb-slide><wb-title>A</wb-title>x</wb-slide><wb-slide><wb-title>B</wb-title>y</wb-slide>").unwrap());
    }

    #[test]
    fn closed_vocabulary_errors() {
        for (m, msg) in [
            ("<wb-slide>hello</wb-slide>", "<wb-title> is required"),
            ("<wb-slide><wb-title>A</wb-title><p>x</p></wb-slide>", "unexpected <p>"),
            ("<wb-slide class=x><wb-title>A</wb-title>x</wb-slide>", "unsupported attributes"),
            ("stray<wb-slide><wb-title>A</wb-title>x</wb-slide>", "text outside"),
            ("<wb-slide><wb-title>A</wb-title>x", "unclosed"),
            ("<wb-slide>x<wb-title>A</wb-title></wb-slide>", "unexpected <wb-title>"),
            ("<wb-slide><wb-title>A</wb-title></wb-slide>", "body text is required"),
            ("", "no <wb-slide>"),
        ] {
            assert!(compile(m).unwrap_err().contains(msg), "{m}");
        }
    }

    #[test]
    fn preset_defaults_and_theme_override() {
        let css = preset_css(None).unwrap();
        assert!(css.contains("--wb-title-size: 32px;") && css.contains("--wb-font: Pretendard"));
        assert!(css.contains(".ms-slide-title") && css.contains("var(--wb-title-size)"));
        let night = preset_css(Some("night")).unwrap();
        // Overrides come after defaults inside :root, so the later value wins.
        assert!(night.rfind("--wb-title-size: 36px;").unwrap() > night.find("--wb-title-size: 32px;").unwrap());
        assert!(tokens(NIGHT_THEME, "night").unwrap().lines().count() == token_names().len());
        for (bad, msg) in [
            ("--wb-bg: url(http://x);", "must be plain"),
            ("--wb-nope: 1px;", "unknown token"),
            ("--wb-bg: red", "expected"),
            ("--wb-bg: red;} body{x:y;", "must be plain"),
        ] {
            assert!(tokens(bad, "t").unwrap_err().contains(msg), "{bad}");
        }
    }
}
