//! Semantic slide markup -> typed spec v1 JSON, plus the themeable slide preset.
//! Closed vocabulary: unknown tags, attributes or stray text are errors.
use html5gum::{DefaultEmitter, Token, Tokenizer};
use serde_json::json;

type Result<T> = std::result::Result<T, String>;

const ROLE_CSS: &str = include_str!("../framework/presets/slide.css");
const DEFAULT_THEME: &str = include_str!("../framework/presets/themes/default.css");
/// Built-in themes beside default; each sets every token.
const THEMES: &[(&str, &str)] = &[
    ("night", include_str!("../framework/presets/themes/night.css")),
    ("classic", include_str!("../framework/presets/themes/classic.css")),
    ("graphite", include_str!("../framework/presets/themes/graphite.css")),
    ("signal", include_str!("../framework/presets/themes/signal.css")),
];
const MAX_THEME_BYTES: usize = 8 * 1024;
const MAX_MARKUP_BYTES: usize = 64 * 1024;
const MAX_DEPTH: usize = 8;

/// Tags and the attributes each accepts. Everything else is an error.
const TAGS: &[(&str, &[&str])] = &[("wb-slide", &["layout"]), ("wb-section", &["number"]), ("wb-quote", &["by"]), ("wb-title", &[]), ("wb-subtitle", &[]), ("wb-list", &[]), ("wb-item", &[]), ("wb-card", &["label"]), ("wb-columns", &[]), ("wb-column", &[]), ("wb-image", &["src", "alt"]), ("wb-stat", &["value", "note"]), ("wb-callout", &["tone"]), ("wb-footer", &[]), ("wb-notes", &[])];

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
    if markup.len() > MAX_MARKUP_BYTES {
        return Err("markup exceeds 64 KiB".into());
    }
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
                if attrs.iter().any(|(k, _)| !allowed.contains(&k.as_str())) || t.self_closing && name != "wb-image" {
                    return Err(format!("unsupported attributes or self-closing tag at byte {}", t.span.start));
                }
                if stack.len() > MAX_DEPTH {
                    return Err(format!("nesting deeper than {MAX_DEPTH} at byte {}", t.span.start));
                }
                let el = El { name, pos: t.span.start, attrs, kids: Vec::new() };
                if t.self_closing {
                    stack.last_mut().unwrap().kids.push(Node::El(el));
                } else {
                    stack.push(el);
                }
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

/// Text-only element (<wb-card>, <wb-callout>) -> paragraph strings.
fn prose_of(e: &El) -> Result<Vec<serde_json::Value>> {
    let mut texts = Vec::new();
    for kid in &e.kids {
        match kid {
            Node::Text(t) => paragraphs(t, &mut texts),
            Node::El(inner) => return Err(unexpected(inner)),
        }
    }
    if texts.is_empty() {
        return Err(format!("empty <{}> at byte {}", e.name, e.pos));
    }
    Ok(texts.into_iter().map(|t| t["text"].clone()).collect())
}

fn attr(e: &El, k: &str) -> Result<String> {
    e.attrs.iter().find(|(n, _)| n == k).map(|(_, v)| v.clone())
        .ok_or(format!("<{}> at byte {} needs {k}=\"…\"", e.name, e.pos))
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
                let mut panel = json!({"type": "panel", "paragraphs": prose_of(e)?});
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
            Node::El(e) if e.name == "wb-image" => {
                if let Some(Node::El(inner)) = e.kids.iter().find(|k| !matches!(k, Node::Text(t) if blank(t))) {
                    return Err(unexpected(inner));
                }
                if e.kids.iter().any(|k| !matches!(k, Node::Text(t) if blank(t))) {
                    return Err(format!("<wb-image> at byte {} takes no content; use alt=\"…\"", e.pos));
                }
                blocks.push(json!({"type": "image", "src": attr(e, "src")?, "alt": collapse(&attr(e, "alt")?)}));
            }
            Node::El(e) if e.name == "wb-stat" => {
                // The caption is the spec kpi label.
                let mut kpi = json!({"type": "kpi", "value": collapse(&attr(e, "value")?), "label": text_of(e)?});
                if let Ok(note) = attr(e, "note") {
                    kpi["note"] = json!(collapse(&note));
                }
                blocks.push(kpi);
            }
            Node::El(e) if e.name == "wb-callout" => {
                // The spec rejects a tone other than info|warning with a field path.
                let tone = attr(e, "tone")?.trim().to_string();
                blocks.push(json!({"type": "callout", "tone": tone, "paragraphs": prose_of(e)?}));
            }
            Node::El(e) => return Err(unexpected(e)),
        }
    }
    Ok(blocks)
}

fn slide(mut el: El, index: usize) -> Result<serde_json::Value> {
    let mut slide = json!({});
    // Layout variant: the value is checked by the spec, which owns the closed set.
    let layout = el.attrs.iter().find(|(k, _)| k == "layout").map(|(_, v)| v.trim().to_string());
    if let Some(layout) = &layout {
        slide["layout"] = json!(layout);
    }
    // Speaker notes: at most one <wb-notes>, anywhere directly in the slide.
    let (notes, kids) = el.kids.into_iter().partition::<Vec<_>, _>(|k| matches!(k, Node::El(e) if e.name == "wb-notes"));
    // Rejoin text the notes split, as with comments.
    el.kids = Vec::new();
    for kid in kids {
        match (el.kids.last_mut(), kid) {
            (Some(Node::Text(prev)), Node::Text(t)) => {
                // A line that held only the notes vanishes with its line break.
                let cut = prev.rfind('\n').filter(|&i| prev[i..].trim().is_empty());
                if let (Some(i), Some(j)) = (cut, t.find('\n')) {
                    if t[..j].trim().is_empty() {
                        prev.truncate(i);
                    }
                }
                prev.push_str(&t)
            }
            (_, kid) => el.kids.push(kid),
        }
    }
    for (i, note) in notes.iter().enumerate() {
        let Node::El(note) = note else { unreachable!() };
        if i > 0 {
            return Err(unexpected(note));
        }
        slide["notes"] = json!(text_of(note)?);
    }
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
    if layout.is_some() && slide.get("heading").is_none() {
        return Err(format!("wb-slide[{index}]: <wb-title> is required with layout at byte {}", el.pos));
    }
    if blocks.is_empty() && layout.is_none() {
        return Err(format!("wb-slide[{index}]: body text is required"));
    }
    slide["blocks"] = json!(blocks);
    Ok(slide)
}

/// First non-blank string value, depth-first, ignoring block `type` tags.
fn first_text(v: &serde_json::Value) -> Option<&serde_json::Value> {
    match v {
        serde_json::Value::String(s) => (!s.trim().is_empty()).then_some(v),
        serde_json::Value::Array(a) => a.iter().find_map(first_text),
        serde_json::Value::Object(o) => o.iter().filter(|(k, _)| *k != "type").find_map(|(_, v)| first_text(v)),
        _ => None,
    }
}

/// Compile markup to typed spec JSON. Same input, same bytes.
pub fn compile(markup: &str) -> Result<String> {
    let (mut slides, mut footer) = (Vec::new(), None);
    for node in tree(markup)? {
        match node {
            Node::El(e) if e.name == "wb-slide" => slides.push(slide(e, slides.len())?),
            // Section divider: title text only; the spec checks the number.
            Node::El(e) if e.name == "wb-section" => {
                let mut s = json!({"layout": "section", "heading": text_of(&e)?, "blocks": []});
                if let Ok(number) = attr(&e, "number") {
                    s["number"] = json!(number.trim());
                }
                slides.push(s);
            }
            // Pull quote: text only; `by` is the attribution, checked by the spec.
            Node::El(e) if e.name == "wb-quote" => {
                let mut s = json!({"layout": "quote", "heading": text_of(&e)?, "blocks": []});
                if let Ok(by) = attr(&e, "by") {
                    s["subtitle"] = json!(by.trim());
                }
                slides.push(s);
            }
            // One deck-level footer, shown on every slide.
            Node::El(e) if e.name == "wb-footer" && footer.is_none() => footer = Some(text_of(&e)?),
            Node::El(e) => return Err(unexpected(&e)),
            Node::Text(_) => {}
        }
    }
    // Deck title: first heading outside dividers and quotes, else a section title, else the first text, else a quote.
    let first = slides.first().ok_or("no <wb-slide> found")?;
    let heading = |skip: &[&str]| slides.iter().filter(|s| !skip.iter().any(|l| s["layout"] == *l)).find_map(|s| s.get("heading"));
    let deck_title =
        heading(&["section", "quote"]).or_else(|| heading(&["quote"])).or_else(|| first_text(&first["blocks"])).or_else(|| heading(&[])).cloned();
    let mut spec = json!({"schema_version": 1, "title": deck_title, "slides": slides});
    if let Some(footer) = footer {
        spec["footer"] = json!(footer);
    }
    Ok(serde_json::to_string_pretty(&spec).unwrap())
}

fn token_names() -> Vec<&'static str> {
    DEFAULT_THEME.lines().filter_map(|l| l.split(':').next()).map(str::trim).filter(|n| !n.is_empty()).collect()
}

/// Validate a token-only theme: `--wb-name: value;` lines with known names.
/// No functions, at-rules or braces, so a theme cannot load URLs or inject rules.
/// `safe center` keeps tall content from being clipped above the body area.
const VERTICAL: [&str; 3] = ["start", "safe center", "space-evenly"];

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
        // An invalid keyword would silently fall back to top alignment.
        if name == "--wb-vertical" && !VERTICAL.contains(&value) {
            return Err(format!("{at}: --wb-vertical must be one of {}", VERTICAL.join(", ")));
        }
        out.push_str(&format!("  {name}: {value};\n"));
    }
    Ok(out)
}

/// Preset CSS: default tokens, then the chosen theme (built-in name or token file), then role styles.
pub fn preset_css(theme: Option<&str>) -> Result<String> {
    Ok(format!(":root {{\n{}{}}}\n{ROLE_CSS}", tokens(DEFAULT_THEME, "default")?, overrides(theme)?))
}

/// Resolved token values (default, then the chosen theme; later wins), for renderers outside CSS.
pub fn token_values(theme: Option<&str>) -> Result<Vec<(String, String)>> {
    let css = tokens(DEFAULT_THEME, "default")? + &overrides(theme)?;
    let mut out: Vec<(String, String)> = Vec::new();
    for (name, value) in css.lines().filter_map(|l| l.trim().strip_suffix(';')?.split_once(':')) {
        out.retain(|(n, _)| n != name.trim());
        out.push((name.trim().to_string(), value.trim().to_string()));
    }
    Ok(out)
}

fn overrides(theme: Option<&str>) -> Result<String> {
    Ok(match theme {
        // paper is the default look; the name stays as an alias.
        None | Some("default") | Some("paper") => String::new(),
        Some(name) if THEMES.iter().any(|(n, _)| *n == name) => {
            tokens(THEMES.iter().find(|(n, _)| *n == name).unwrap().1, name)?
        }
        Some(path) => {
            let source = std::fs::read(path).map_err(|e| {
                // A bare word is almost always a mistyped built-in name, not a file.
                if !path.contains('/') && !path.ends_with(".css") {
                    let names: Vec<_> = THEMES.iter().map(|(n, _)| *n).collect();
                    format!("unknown theme {path}: use paper, {} or a token file path", names.join(", "))
                } else {
                    format!("cannot read theme {path}: {e}")
                }
            })?;
            if source.len() > MAX_THEME_BYTES {
                return Err(format!("theme {path} exceeds 8 KiB"));
            }
            tokens(&String::from_utf8(source).map_err(|_| format!("theme {path}: not UTF-8"))?, path)?
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_theme_name_lists_built_ins() {
        let err = preset_css(Some("papr")).unwrap_err();
        assert!(err.contains("unknown theme papr") && err.contains("graphite"), "{err}");
        assert!(preset_css(Some("./nope.css")).unwrap_err().contains("cannot read theme"));
    }

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
    fn slide_layout_passes_through_and_allows_an_empty_body() {
        let json = compile("<wb-slide layout=\" cover \"><wb-title>표지</wb-title><wb-subtitle>부제</wb-subtitle></wb-slide>").unwrap();
        let s: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(s["slides"][0], json!({"layout": "cover", "heading": "표지", "subtitle": "부제", "blocks": []}));
        let (_, slides) = crate::spec::parse(json.as_bytes()).unwrap();
        assert_eq!(slides[0].frontmatter[0].1, "slide-cover");
        assert_eq!(spec("<wb-slide layout=section><wb-title>장</wb-title>본문</wb-slide>")["slides"][0]["blocks"][0]["text"], "본문");
        // The spec owns the value set.
        let json = compile("<wb-slide layout=pull-quote><wb-title>T</wb-title></wb-slide>").unwrap();
        assert_eq!(crate::spec::parse(json.as_bytes()).err().unwrap(), "$.slides[0].layout: expected cover or section or quote");
        assert!(compile("<wb-slide layout=cover>x</wb-slide>").unwrap_err().contains("<wb-title> is required with layout at byte 0"));
        assert!(compile("<wb-slide><wb-title>T</wb-title></wb-slide>").unwrap_err().contains("body text is required"));
    }

    #[test]
    fn section_is_a_numbered_divider() {
        let titled = spec("<wb-section number=\"01\">1부</wb-section><wb-slide><wb-title>진짜 제목</wb-title>x</wb-slide>");
        assert_eq!(titled["title"], "진짜 제목");
        assert_eq!(spec("<wb-section>1부</wb-section><wb-slide>x</wb-slide>")["title"], "1부");
        let json = compile("<wb-section number=\" 01 \">\n  첫   장\n</wb-section><wb-slide>x</wb-slide>").unwrap();
        let s: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(s["title"], "첫 장");
        assert_eq!(s["slides"][0], json!({"layout": "section", "heading": "첫 장", "number": "01", "blocks": []}));
        let (_, slides) = crate::spec::parse(json.as_bytes()).unwrap();
        assert!(slides[0].body_html.starts_with("<header class=\"wb-head\"><span class=\"wb-head-number\">01</span>"));
        assert!(spec("<wb-section>장</wb-section>")["slides"][0].get("number").is_none());
        // Slide indices in errors count sections too.
        assert!(compile("<wb-section>장</wb-section><wb-slide></wb-slide>").unwrap_err().contains("wb-slide[1]: body text is required"));
        let json = compile("<wb-section number=\"123456789\">장</wb-section>").unwrap();
        assert_eq!(crate::spec::parse(json.as_bytes()).err().unwrap(), "$.slides[0].number: expected 1–8 characters, not blank");
        for (m, msg) in [
            ("<wb-section> </wb-section>", "empty <wb-section> at byte 0"),
            ("<wb-section>장<wb-notes>n</wb-notes></wb-section>", "unexpected <wb-notes>"),
            ("<wb-section layout=cover>장</wb-section>", "unsupported attributes"),
            ("<wb-slide><wb-section>장</wb-section></wb-slide>", "unexpected <wb-section>"),
        ] {
            assert!(compile(m).unwrap_err().contains(msg), "{m}");
        }
    }

    #[test]
    fn quote_is_a_pull_quote_slide() {
        let titled = spec("<wb-quote by=\"김 대표\">인용</wb-quote><wb-section>장</wb-section><wb-slide><wb-title>진짜 제목</wb-title>x</wb-slide>");
        assert_eq!(titled["title"], "진짜 제목");
        assert_eq!(spec("<wb-quote>인용</wb-quote><wb-section>장</wb-section>")["title"], "장");
        assert_eq!(spec("<wb-quote>인용</wb-quote><wb-slide>본문</wb-slide>")["title"], "인용");
        let json = compile("<wb-quote by=\" A &amp; B, CEO \">\n  좋은   &lt;말&gt;\n</wb-quote>").unwrap();
        let s: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(s["title"], "좋은 <말>");
        assert_eq!(s["slides"][0], json!({"layout": "quote", "heading": "좋은 <말>", "subtitle": "A & B, CEO", "blocks": []}));
        let (_, slides) = crate::spec::parse(json.as_bytes()).unwrap();
        assert_eq!(slides[0].body_html, "<figure class=\"wb-quote\"><blockquote><p>좋은 &lt;말&gt;</p></blockquote><figcaption class=\"wb-quote-by\">A &amp; B, CEO</figcaption></figure>");
        assert!(spec("<wb-quote>말</wb-quote>")["slides"][0].get("subtitle").is_none());
        let json = compile("<wb-quote by=\"\">말</wb-quote>").unwrap();
        assert_eq!(crate::spec::parse(json.as_bytes()).err().unwrap(), "$.slides[0].subtitle: expected 1–2000 characters, not blank");
        assert!(compile("<wb-quote>말</wb-quote><wb-slide></wb-slide>").unwrap_err().contains("wb-slide[1]: body text is required"));
        for (m, msg) in [
            ("<wb-quote> </wb-quote>", "empty <wb-quote> at byte 0"),
            ("<wb-quote>말<wb-notes>n</wb-notes></wb-quote>", "unexpected <wb-notes>"),
            ("<wb-quote author=x>말</wb-quote>", "unsupported attributes"),
            ("<wb-slide><wb-quote>말</wb-quote></wb-slide>", "unexpected <wb-quote>"),
        ] {
            assert!(compile(m).unwrap_err().contains(msg), "{m}");
        }
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
        let json = compile("<wb-slide><wb-card label=\" \">x</wb-card></wb-slide>").unwrap();
        assert!(crate::spec::parse(json.as_bytes()).err().unwrap().contains("$.slides[0].blocks[0].label"));
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
        // Input size and depth are bounded before any recursion.
        let deep = "<wb-columns><wb-column>".repeat(20);
        assert!(compile(&format!("<wb-slide>{deep}")).unwrap_err().contains("nesting deeper"));
        assert!(compile(&"x".repeat(MAX_MARKUP_BYTES + 1)).unwrap_err().contains("64 KiB"));
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
    fn image_takes_src_and_alt() {
        let s = spec("<wb-slide><wb-image src=\"img/a.svg\" alt=\" 구성  도 \" /><wb-image src=\"b.png\" alt=\"b\"></wb-image></wb-slide>");
        assert_eq!(s["slides"][0]["blocks"], json!([
            {"type":"image","src":"img/a.svg","alt":"구성 도"}, {"type":"image","src":"b.png","alt":"b"}]));
        for (m, msg) in [
            ("<wb-slide><wb-image src=\"a.png\"/></wb-slide>", "needs alt="),
            ("<wb-slide><wb-image alt=\"a\"/></wb-slide>", "needs src="),
            ("<wb-slide><wb-image src=\"a.png\" alt=\"a\">설명</wb-image></wb-slide>", "takes no content"),
            ("<wb-slide><wb-card label=\"x\"/></wb-slide>", "self-closing"),
        ] {
            assert!(compile(m).unwrap_err().contains(msg), "{m}");
        }
    }

    #[test]
    fn footer_is_deck_level() {
        let s = spec("<wb-footer> 가상 &amp; 팀 </wb-footer><wb-slide>x</wb-slide>");
        assert_eq!(s["footer"], "가상 & 팀");
        assert!(spec("<wb-slide>x</wb-slide>").get("footer").is_none());
        for m in ["<wb-footer>a</wb-footer><wb-footer>b</wb-footer><wb-slide>x</wb-slide>", "<wb-slide><wb-footer>a</wb-footer>x</wb-slide>"] {
            assert!(compile(m).unwrap_err().contains("unexpected <wb-footer>"), "{m}");
        }
    }

    #[test]
    fn stat_is_a_kpi_with_caption() {
        let s = spec("<wb-slide><wb-columns><wb-column><wb-stat value=\"+38%\" note=\" 가상  수치 \"> 재방문\n 증가 </wb-stat></wb-column>\n<wb-column><wb-stat value=\"3주\">준비 기간</wb-stat></wb-column></wb-columns></wb-slide>");
        assert_eq!(s["slides"][0]["blocks"][0]["columns"], json!([
            {"blocks":[{"type":"kpi","value":"+38%","label":"재방문 증가","note":"가상 수치"}]},
            {"blocks":[{"type":"kpi","value":"3주","label":"준비 기간"}]}]));
        for (m, msg) in [
            ("<wb-slide><wb-stat>x</wb-stat></wb-slide>", "<wb-stat> at byte 10 needs value="),
            ("<wb-slide><wb-stat value=\"1\"> </wb-stat></wb-slide>", "empty <wb-stat> at byte 10"),
            ("<wb-slide><wb-stat value=\"1\">a<wb-list></wb-list></wb-stat></wb-slide>", "unexpected <wb-list>"),
            ("<wb-slide><wb-stat value=\"1\"/></wb-slide>", "self-closing"),
            ("<wb-slide><wb-stat value=\"1\" label=\"a\">x</wb-stat></wb-slide>", "unsupported attributes"),
        ] {
            assert!(compile(m).unwrap_err().contains(msg), "{m}");
        }
        assert!(crate::spec::parse(compile("<wb-slide><wb-stat value=\"1\">a</wb-stat></wb-slide>").unwrap().as_bytes()).is_ok());
    }

    #[test]
    fn callout_has_tone_and_paragraphs() {
        let s = spec("<wb-slide><wb-callout tone=\"warning\">첫째\n\n둘째</wb-callout><wb-callout tone=\"info\">단독</wb-callout></wb-slide>");
        assert_eq!(s["slides"][0]["blocks"], json!([
            {"type":"callout","tone":"warning","paragraphs":["첫째","둘째"]},
            {"type":"callout","tone":"info","paragraphs":["단독"]}]));
        for (m, msg) in [
            ("<wb-slide><wb-callout>x</wb-callout></wb-slide>", "<wb-callout> at byte 10 needs tone="),
            ("<wb-slide><wb-callout tone=\"info\"> </wb-callout></wb-slide>", "empty <wb-callout>"),
            ("<wb-slide><wb-callout tone=\"info\"><wb-list></wb-list></wb-callout></wb-slide>", "unexpected <wb-list>"),
            ("<wb-slide><wb-callout tone=\"info\"/></wb-slide>", "self-closing"),
        ] {
            assert!(compile(m).unwrap_err().contains(msg), "{m}");
        }
        // Another tone is left for the spec to reject with a field path.
        let json = compile("<wb-slide><wb-callout tone=\"danger\">x</wb-callout></wb-slide>").unwrap();
        assert!(crate::spec::parse(json.as_bytes()).err().unwrap().contains("$.slides[0].blocks[0].tone: expected info or warning"));
    }

    #[test]
    fn notes_lower_to_slide_notes() {
        let s = spec("<wb-slide><wb-notes> 먼저\n  &lt;결론&gt; </wb-notes><wb-title>T</wb-title>x</wb-slide>");
        assert_eq!((s["slides"][0]["notes"].as_str(), s["slides"][0]["heading"].as_str()), (Some("먼저 <결론>"), Some("T")));
        assert_eq!(s["slides"][0]["blocks"], json!([{"type":"text","text":"x"}]));
        let s = spec("<wb-slide>앞 <wb-notes>n</wb-notes> 뒤</wb-slide>");
        assert_eq!(s["slides"][0]["blocks"], json!([{"type":"text","text":"앞 뒤"}]));
        // A line holding only the notes vanishes; a real blank line still splits.
        let s = spec("<wb-slide>p1\n  <wb-notes>n</wb-notes>\n  p2\n\np3</wb-slide>");
        assert_eq!(s["slides"][0]["blocks"], json!([{"type":"text","text":"p1 p2"}, {"type":"text","text":"p3"}]));
        for (m, msg) in [
            ("<wb-slide>x<wb-notes>a</wb-notes><wb-notes>b</wb-notes></wb-slide>", "unexpected <wb-notes> at byte 33"),
            ("<wb-slide>x<wb-notes>a<wb-list></wb-list></wb-notes></wb-slide>", "unexpected <wb-list>"),
            ("<wb-slide>x<wb-notes> </wb-notes></wb-slide>", "empty <wb-notes>"),
            ("<wb-slide><wb-card>x<wb-notes>a</wb-notes></wb-card></wb-slide>", "unexpected <wb-notes>"),
            ("<wb-notes>a</wb-notes><wb-slide>x</wb-slide>", "unexpected <wb-notes>"),
        ] {
            assert!(compile(m).unwrap_err().contains(msg), "{m}");
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
        assert!(css.contains("--wb-title-size: 40px;") && css.contains("--wb-font: Pretendard"));
        assert!(css.contains(".ms-slide-title") && css.contains("var(--wb-title-size)"));
        let night = preset_css(Some("night")).unwrap();
        // Overrides come after defaults inside :root, so the later value wins.
        assert!(night.rfind("--wb-title-size: 36px;").unwrap() > night.find("--wb-title-size: 40px;").unwrap());
        for (name, source) in THEMES {
            assert_eq!(tokens(source, name).unwrap().lines().count(), token_names().len(), "{name}");
        }
        assert_eq!(preset_css(Some("paper")).unwrap(), css);
        assert!(preset_css(Some("classic")).unwrap().contains("--wb-title-size: 32px;"));
        assert!(tokens("--wb-vertical: start;", "t").is_ok());
        for (bad, msg) in [
            ("--wb-bg: url(http://x);", "must be plain"),
            ("--wb-nope: 1px;", "unknown token"),
            ("--wb-bg: red", "expected"),
            ("--wb-bg: red;} body{x:y;", "must be plain"),
            ("--wb-bg: red /*;", "must be plain"),
            ("--wb-vertical: center;", "must be one of"),
        ] {
            assert!(tokens(bad, "t").unwrap_err().contains(msg), "{bad}");
        }
    }
}
