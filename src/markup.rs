//! Semantic slide markup -> typed spec v1 JSON, plus the themeable slide preset.
//! Closed vocabulary: unknown tags, attributes or stray text are errors.
use html5gum::{DefaultEmitter, Token, Tokenizer};
use serde_json::json;

type Result<T> = std::result::Result<T, String>;

const ROLE_CSS: &str = include_str!("../framework/presets/slide.css");
const DEFAULT_THEME: &str = include_str!("../framework/presets/themes/default.css");
const NIGHT_THEME: &str = include_str!("../framework/presets/themes/night.css");
const MAX_THEME_BYTES: usize = 8 * 1024;

/// Tags and the attributes each accepts. Everything else is an error.
const TAGS: &[(&str, &[&str])] = &[("wb-slide", &[]), ("wb-title", &[]), ("wb-subtitle", &[]), ("wb-list", &[]), ("wb-item", &[]), ("wb-card", &["label"]), ("wb-columns", &[]), ("wb-column", &[])];

enum Node {
    Text(String),
    El(El),
}

struct El {
    name: String,
    pos: usize,
    attrs: Vec<(String, String)>,
    kids: Vec<Node>,
}

fn unexpected(el: &El) -> String {
    format!("unexpected <{}> at byte {}", el.name, el.pos)
}

/// Tokenize into a tree of known elements and text. Checks vocabulary and nesting only.
fn tree(markup: &str) -> Result<Vec<Node>> {
    let mut stack = vec![El { name: String::new(), pos: 0, attrs: Vec::new(), kids: Vec::new() }];
    let emitter = DefaultEmitter::<usize>::new_with_span();
    for token in Tokenizer::new_with_emitter(markup, emitter) {
        match token.map_err(|e| e.to_string())? {
            Token::String(s) => {
                let text = String::from_utf8_lossy(&s.value).to_string();
                if stack.len() == 1 && !blank(&text) {
                    return Err(format!("text outside <wb-slide> at byte {}", s.span.start));
                }
                // Merge runs split by comments so line breaks are judged on whole lines.
                match stack.last_mut().unwrap().kids.last_mut() {
                    Some(Node::Text(prev)) => prev.push_str(&text),
                    _ => stack.last_mut().unwrap().kids.push(Node::Text(text)),
                }
            }
            Token::Comment(_) | Token::Doctype(_) => {}
            Token::Error(e) => return Err(format!("markup error at byte {}: {}", e.span.start, e.value)),
            Token::StartTag(t) => {
                let name = String::from_utf8_lossy(&t.name).to_string();
                let allowed = TAGS.iter().find(|(n, _)| *n == name).map(|(_, a)| *a);
                let allowed = allowed.ok_or(format!("unexpected <{name}> at byte {}", t.span.start))?;
                let attrs: Vec<(String, String)> = t.attributes.iter()
                    .map(|(k, v)| (String::from_utf8_lossy(k).to_string(), String::from_utf8_lossy(&v.value).to_string()))
                    .collect();
                if attrs.iter().any(|(k, _)| !allowed.contains(&k.as_str())) || t.self_closing {
                    return Err(format!("unsupported attributes or self-closing tag at byte {}", t.span.start));
                }
                stack.push(El { name, pos: t.span.start, attrs, kids: Vec::new() });
            }
            Token::EndTag(t) => {
                let name = String::from_utf8_lossy(&t.name).to_string();
                if stack.len() == 1 || stack.last().unwrap().name != name {
                    return Err(format!("unexpected closing <{name}> at byte {}", t.span.start));
                }
                let el = stack.pop().unwrap();
                stack.last_mut().unwrap().kids.push(Node::El(el));
            }
        }
    }
    if stack.len() > 1 {
        return Err(format!("unclosed <{}> at end of input", stack.last().unwrap().name));
    }
    Ok(stack.pop().unwrap().kids)
}

/// HTML whitespace is ASCII only; U+00A0 (&nbsp;) is content.
fn blank(s: &str) -> bool {
    s.bytes().all(|b| b.is_ascii_whitespace())
}

fn collapse(s: &str) -> String {
    s.split_ascii_whitespace().collect::<Vec<_>>().join(" ")
}

/// Inline text of an element that may contain only text.
fn text_of(el: &El) -> Result<String> {
    let mut text = String::new();
    for kid in &el.kids {
        match kid {
            Node::Text(t) => text.push_str(t),
            Node::El(e) => return Err(unexpected(e)),
        }
    }
    let text = collapse(&text);
    if text.is_empty() {
        return Err(format!("empty <{}> at byte {}", el.name, el.pos));
    }
    Ok(text)
}

/// Text -> paragraph blocks. Blank lines separate paragraphs; other whitespace collapses.
fn paragraphs(text: &str, out: &mut Vec<serde_json::Value>) {
    let mut paragraphs = vec![String::new()];
    for (i, line) in text.split('\n').enumerate() {
        if i > 0 && blank(line) {
            paragraphs.push(String::new());
        } else {
            let last = paragraphs.last_mut().unwrap();
            last.push(' ');
            last.push_str(line);
        }
    }
    out.extend(paragraphs.iter().map(|p| collapse(p)).filter(|p| !p.is_empty())
        .map(|text| json!({"type": "text", "text": text})));
}

/// Element children that must all be `name` (whitespace between them is ignored).
fn children<'a>(el: &'a El, name: &str) -> Result<Vec<&'a El>> {
    el.kids.iter().filter(|k| !matches!(k, Node::Text(t) if blank(t))).map(|k| match k {
        Node::El(e) if e.name == name => Ok(e),
        Node::El(e) => Err(unexpected(e)),
        Node::Text(_) => Err(format!("<{}> at byte {} takes only <{name}> children", el.name, el.pos)),
    }).collect()
}

/// Body content -> spec blocks, in order.
fn body(kids: &[Node]) -> Result<Vec<serde_json::Value>> {
    let mut blocks = Vec::new();
    for kid in kids {
        match kid {
            Node::Text(t) => paragraphs(t, &mut blocks),
            Node::El(e) if e.name == "wb-list" => {
                let items = children(e, "wb-item")?.into_iter().map(text_of).collect::<Result<Vec<_>>>()?;
                if items.is_empty() {
                    return Err(format!("empty <wb-list> at byte {}", e.pos));
                }
                blocks.push(json!({"type": "list", "items": items}));
            }
            Node::El(e) if e.name == "wb-card" => {
                let mut texts = Vec::new();
                for kid in &e.kids {
                    match kid {
                        Node::Text(t) => paragraphs(t, &mut texts),
                        Node::El(inner) => return Err(unexpected(inner)),
                    }
                }
                if texts.is_empty() {
                    return Err(format!("empty <wb-card> at byte {}", e.pos));
                }
                let mut panel = json!({"type": "panel", "paragraphs": texts.iter().map(|t| &t["text"]).collect::<Vec<_>>()});
                if let Some((_, label)) = e.attrs.iter().find(|(k, _)| k == "label") {
                    panel["label"] = json!(collapse(label));
                }
                blocks.push(panel);
            }
            Node::El(e) if e.name == "wb-columns" => {
                let columns = children(e, "wb-column")?
                    .into_iter()
                    .map(|c| match body(&c.kids)? {
                        b if b.is_empty() => Err(format!("empty <wb-column> at byte {}", c.pos)),
                        b => Ok(json!({"blocks": b})),
                    })
                    .collect::<Result<Vec<_>>>()?;
                // The spec enforces 2–3 columns and no nesting, with a field path.
                blocks.push(json!({"type": "columns", "columns": columns}));
            }
            Node::El(e) => return Err(unexpected(e)),
        }
    }
    Ok(blocks)
}

fn slide(el: &El, index: usize) -> Result<serde_json::Value> {
    let mut slide = json!({});
    // Optional head, before any body: <wb-title>, then optional <wb-subtitle>.
    let mut at = 0;
    let mut take = |name: &str| {
        let j = at + el.kids[at..].iter().position(|k| !matches!(k, Node::Text(t) if blank(t)))?;
        match &el.kids[j] {
            Node::El(e) if e.name == name => {
                at = j + 1;
                Some(e)
            }
            _ => None,
        }
    };
    if let Some(title) = take("wb-title") {
        slide["heading"] = json!(text_of(title)?);
        if let Some(sub) = take("wb-subtitle") {
            slide["subtitle"] = json!(text_of(sub)?);
        }
    }
    let blocks = body(&el.kids[at..])?;
    if blocks.is_empty() {
        return Err(format!("wb-slide[{index}]: body text is required"));
    }
    slide["blocks"] = json!(blocks);
    Ok(slide)
}

/// First string value, depth-first, ignoring block `type` tags.
fn first_text(v: &serde_json::Value) -> Option<&serde_json::Value> {
    match v {
        serde_json::Value::String(_) => Some(v),
        serde_json::Value::Array(a) => a.iter().find_map(first_text),
        serde_json::Value::Object(o) => o.iter().filter(|(k, _)| *k != "type").find_map(|(_, v)| first_text(v)),
        _ => None,
    }
}

/// Compile markup to typed spec JSON. Same input, same bytes.
pub fn compile(markup: &str) -> Result<String> {
    let slides = tree(markup)?
        .iter()
        .filter_map(|n| match n {
            Node::El(e) if e.name == "wb-slide" => Some(Ok(e)),
            Node::El(e) => Some(Err(unexpected(e))),
            Node::Text(_) => None,
        })
        .enumerate()
        .map(|(i, e)| slide(e?, i))
        .collect::<Result<Vec<_>>>()?;
    // Deck title: first heading, else the first text anywhere in the blocks.
    let first = slides.first().ok_or("no <wb-slide> found")?;
    let deck_title = slides.iter().find_map(|s| s.get("heading")).or_else(|| first_text(&first["blocks"])).cloned();
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
        if value.is_empty() || value.contains(|c| "(){};<>@\\\"/*".contains(c)) {
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
        let s = spec("<wb-slide><wb-title>T</wb-title>a\r\n  \r\nb&nbsp;c</wb-slide>");
        assert_eq!(s["slides"][0]["blocks"][0]["text"], "a");
        assert_eq!(s["slides"][0]["blocks"][1]["text"], "b\u{a0}c");
        let s = spec("<wb-slide><wb-title>T</wb-title>a\n<!-- c -->b\n<!-- c -->\nd</wb-slide>");
        assert_eq!(s["slides"][0]["blocks"], json!([{"type":"text","text":"a b"},{"type":"text","text":"d"}]));
        assert!(compile("&nbsp;<wb-slide>x</wb-slide>").unwrap_err().contains("text outside"));
        let json = compile("<wb-slide><wb-title>A</wb-title>x</wb-slide><wb-slide><wb-title>B</wb-title>y</wb-slide>").unwrap();
        assert!(crate::spec::parse(json.as_bytes()).is_ok());
        assert_eq!(json, compile("<wb-slide><wb-title>A</wb-title>x</wb-slide><wb-slide><wb-title>B</wb-title>y</wb-slide>").unwrap());
    }

    #[test]
    fn bare_slide_is_centered_content() {
        let json = compile("<wb-slide> hello </wb-slide><wb-slide><wb-title>H</wb-title>x</wb-slide>").unwrap();
        let s: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(s["title"], "H");
        assert!(s["slides"][0].get("heading").is_none());
        let (_, slides) = crate::spec::parse(json.as_bytes()).unwrap();
        assert_eq!(slides[0].frontmatter[0].1, "slide-section");
        assert_eq!(spec("<wb-slide>hello</wb-slide>")["title"], "hello");
    }

    #[test]
    fn subtitle_follows_title() {
        let s = spec("<wb-slide><wb-title>T</wb-title>\n <wb-subtitle> 부제 </wb-subtitle>본문</wb-slide>");
        assert_eq!((s["slides"][0]["subtitle"].as_str(), s["slides"][0]["blocks"][0]["text"].as_str()), (Some("부제"), Some("본문")));
        for m in ["<wb-slide><wb-subtitle>S</wb-subtitle>x</wb-slide>", "<wb-slide><wb-title>T</wb-title>x<wb-subtitle>S</wb-subtitle></wb-slide>"] {
            assert!(compile(m).unwrap_err().contains("unexpected <wb-subtitle>"), "{m}");
        }
    }

    #[test]
    fn list_items_between_paragraphs() {
        let s = spec("<wb-slide><wb-title>T</wb-title>앞\n<wb-list>\n <wb-item>하나</wb-item>\n <wb-item>둘 &amp; 셋</wb-item>\n</wb-list>\n뒤</wb-slide>");
        assert_eq!(s["slides"][0]["blocks"], json!([
            {"type":"text","text":"앞"}, {"type":"list","items":["하나","둘 & 셋"]}, {"type":"text","text":"뒤"}]));
        let json = compile("<wb-slide><wb-list><wb-item>x</wb-item></wb-list></wb-slide>").unwrap();
        assert!(crate::spec::parse(json.as_bytes()).is_ok());
        assert_eq!(serde_json::from_str::<serde_json::Value>(&json).unwrap()["title"], "x");
        for (m, msg) in [
            ("<wb-slide><wb-list>x</wb-list></wb-slide>", "takes only <wb-item>"),
            ("<wb-slide><wb-list></wb-list></wb-slide>", "empty <wb-list>"),
            ("<wb-slide><wb-item>x</wb-item></wb-slide>", "unexpected <wb-item>"),
            ("<wb-slide><wb-list><wb-item></wb-item></wb-list></wb-slide>", "empty <wb-item>"),
        ] {
            assert!(compile(m).unwrap_err().contains(msg), "{m}");
        }
    }

    #[test]
    fn card_is_a_labelled_panel() {
        let s = spec("<wb-slide><wb-card label=\" 검토 &amp; 확인 \">첫째\n\n둘째</wb-card><wb-card>단독</wb-card></wb-slide>");
        assert_eq!(s["slides"][0]["blocks"], json!([
            {"type":"panel","label":"검토 & 확인","paragraphs":["첫째","둘째"]},
            {"type":"panel","paragraphs":["단독"]}]));
        assert_eq!(s["title"], "검토 & 확인"); // first string: the label
        for (m, msg) in [
            ("<wb-slide><wb-card></wb-card></wb-slide>", "empty <wb-card>"),
            ("<wb-slide><wb-card><wb-list><wb-item>x</wb-item></wb-list></wb-card></wb-slide>", "unexpected <wb-list>"),
            ("<wb-slide><wb-card tone=info>x</wb-card></wb-slide>", "unsupported attributes"),
        ] {
            assert!(compile(m).unwrap_err().contains(msg), "{m}");
        }
        // A blank label is left for the spec to reject with a field path.
        assert!(compile("<wb-slide><wb-card label=\" \">x</wb-card></wb-slide>").is_ok());
    }

    #[test]
    fn columns_hold_body_content() {
        let s = spec("<wb-slide><wb-title>T</wb-title><wb-columns>\n<wb-column>왼쪽<wb-list><wb-item>a</wb-item></wb-list></wb-column>\n<wb-column><wb-card label=\"L\">오른쪽</wb-card></wb-column>\n</wb-columns>끝</wb-slide>");
        assert_eq!(s["slides"][0]["blocks"], json!([
            {"type":"columns","columns":[
                {"blocks":[{"type":"text","text":"왼쪽"},{"type":"list","items":["a"]}]},
                {"blocks":[{"type":"panel","label":"L","paragraphs":["오른쪽"]}]}]},
            {"type":"text","text":"끝"}]));
        assert!(crate::spec::parse(compile("<wb-slide><wb-columns><wb-column>a</wb-column><wb-column>b</wb-column></wb-columns></wb-slide>").unwrap().as_bytes()).is_ok());
        for (m, msg) in [
            ("<wb-slide><wb-columns>x</wb-columns></wb-slide>", "takes only <wb-column>"),
            ("<wb-slide><wb-columns><wb-column> </wb-column><wb-column>b</wb-column></wb-columns></wb-slide>", "empty <wb-column>"),
            ("<wb-slide><wb-column>a</wb-column></wb-slide>", "unexpected <wb-column>"),
        ] {
            assert!(compile(m).unwrap_err().contains(msg), "{m}");
        }
        // Count and nesting limits come from the spec, with a field path.
        for (m, msg) in [
            ("<wb-slide><wb-columns><wb-column>a</wb-column></wb-columns></wb-slide>", "$.slides[0].blocks[0].columns: expected 2–3"),
            ("<wb-slide><wb-columns><wb-column><wb-columns><wb-column>a</wb-column><wb-column>b</wb-column></wb-columns></wb-column><wb-column>c</wb-column></wb-columns></wb-slide>", "columns cannot nest"),
        ] {
            let json = compile(m).unwrap();
            assert!(crate::spec::parse(json.as_bytes()).err().unwrap().contains(msg), "{m}");
        }
    }

    #[test]
    fn closed_vocabulary_errors() {
        for (m, msg) in [
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
            ("--wb-bg: red /*;", "must be plain"),
        ] {
            assert!(tokens(bad, "t").unwrap_err().contains(msg), "{bad}");
        }
    }
}
