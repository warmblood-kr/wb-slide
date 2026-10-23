//! Closed, text-only slide data. No Markdown parsing or deck-local code loading.
use crate::{template::html_escape as escape, Slide};
use serde_json::{Map, Value};
use std::{fs::File, io::Read, path::Path};

const MAX_IMAGE_BYTES: u64 = 2 * 1024 * 1024;
const MAX_IMAGES: usize = 8;

/// Where local images may come from, and how many were embedded so far.
struct Ctx<'a> {
    base: Option<&'a Path>,
    images: usize,
}

const MAX_BYTES: usize = 64 * 1024;

/// Slide layout variants: spec name -> framework layout. A new variant is one row
/// here plus one CSS block in `framework/presets/slide.css`.
const LAYOUTS: &[(&str, &str)] = &[("cover", "slide-cover"), ("section", "slide-section"), ("quote", "slide-quote"), ("split", "slide-split")];
type Object = Map<String, Value>;
type Result<T> = std::result::Result<T, String>;

#[derive(Debug)]
enum Block {
    Text(String),
    Panel(Option<String>, Vec<String>),
    Callout(String, Vec<String>),
    Kpi(String, String, Option<String>),
    List(Vec<String>),
    Image(String, String),
    Columns(Vec<Vec<Block>>),
}

fn error(path: &str, reason: &str) -> String {
    format!("{path}: {reason}")
}

fn object<'a>(v: &'a Value, path: &str, keys: &[&str]) -> Result<&'a Object> {
    let o = v
        .as_object()
        .ok_or_else(|| error(path, "expected object"))?;
    for key in o.keys() {
        if !keys.contains(&key.as_str()) {
            return Err(error(&format!("{path}.{key}"), "unknown field"));
        }
    }
    Ok(o)
}

fn field<'a>(o: &'a Object, key: &str, path: &str) -> Result<&'a Value> {
    o.get(key)
        .ok_or_else(|| error(&format!("{path}.{key}"), "missing required field"))
}

fn string(o: &Object, key: &str, path: &str) -> Result<String> {
    let v = field(o, key, path)?
        .as_str()
        .ok_or_else(|| error(&format!("{path}.{key}"), "expected string"))?;
    if v.trim().is_empty() || v.chars().count() > 2000 {
        return Err(error(
            &format!("{path}.{key}"),
            "expected 1–2000 characters, not blank",
        ));
    }
    Ok(v.to_string())
}

fn optional(o: &Object, key: &str, path: &str) -> Result<Option<String>> {
    o.contains_key(key)
        .then(|| string(o, key, path))
        .transpose()
}

fn items<'a>(
    o: &'a Object,
    key: &str,
    path: &str,
    min: usize,
    max: usize,
) -> Result<&'a Vec<Value>> {
    let p = format!("{path}.{key}");
    let a = field(o, key, path)?
        .as_array()
        .ok_or_else(|| error(&p, "expected array"))?;
    if !(min..=max).contains(&a.len()) {
        return Err(error(&p, &format!("expected {min}–{max} items")));
    }
    Ok(a)
}

fn paragraphs(o: &Object, path: &str) -> Result<Vec<String>> {
    strings(o, "paragraphs", path)
}

fn strings(o: &Object, key: &str, path: &str) -> Result<Vec<String>> {
    items(o, key, path, 1, 16)?
        .iter()
        .enumerate()
        .map(|(i, value)| {
            let p = format!("{path}.{key}[{i}]");
            let text = value.as_str().ok_or_else(|| error(&p, "expected string"))?;
            if text.trim().is_empty() || text.chars().count() > 2000 {
                return Err(error(&p, "expected 1–2000 characters, not blank"));
            }
            Ok(text.to_string())
        })
        .collect()
}

/// Embed a local image as a data URI. Relative path inside `base` only; no URLs.
fn image(src: &str, p: &str, ctx: &mut Ctx) -> Result<String> {
    let at = format!("{p}.src");
    let base = ctx.base.ok_or_else(|| error(&at, "images need a spec file location"))?;
    let rel = Path::new(src);
    if src.contains(':') || src.contains('\\') || !rel.is_relative()
        || rel.components().any(|c| !matches!(c, std::path::Component::Normal(_)))
    {
        return Err(error(&at, "expected a relative local path without .. or a scheme"));
    }
    let ext = rel.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    let mime = match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        _ => return Err(error(&at, "expected .png, .jpg, .webp or .svg")),
    };
    ctx.images += 1;
    if ctx.images > MAX_IMAGES {
        return Err(error(&at, "at most 8 images per deck"));
    }
    let base = base.canonicalize().map_err(|e| error(&at, &format!("cannot resolve base: {e}")))?;
    let full = base.join(rel).canonicalize().map_err(|e| error(&at, &format!("cannot read image: {e}")))?;
    if !full.starts_with(&base) {
        return Err(error(&at, "image resolves outside the spec directory"));
    }
    // A FIFO or device would block or never end; only regular files are read.
    if !full.metadata().map(|m| m.is_file()).unwrap_or(false) {
        return Err(error(&at, "expected a regular file"));
    }
    let mut bytes = Vec::new();
    File::open(&full)
        .and_then(|f| f.take(MAX_IMAGE_BYTES + 1).read_to_end(&mut bytes))
        .map_err(|e| error(&at, &format!("cannot read image: {e}")))?;
    if bytes.len() as u64 > MAX_IMAGE_BYTES {
        return Err(error(&at, "image exceeds 2 MiB"));
    }
    use base64::Engine as _;
    Ok(format!("data:{mime};base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes)))
}

fn blocks(o: &Object, path: &str, columns_allowed: bool, count: &mut usize, ctx: &mut Ctx) -> Result<Vec<Block>> {
    items(o, "blocks", path, 1, 64)?
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let p = format!("{path}.blocks[{i}]");
            let raw = v
                .as_object()
                .ok_or_else(|| error(&p, "expected block object"))?;
            let kind = string(raw, "type", &p)?;
            let keys: &[&str] = match kind.as_str() {
                "text" => &["type", "text"],
                "panel" => &["type", "label", "paragraphs"],
                "callout" => &["type", "tone", "paragraphs"],
                "kpi" => &["type", "value", "label", "note"],
                "list" => &["type", "items"],
                "image" => &["type", "src", "alt"],
                "columns" if columns_allowed => &["type", "columns"],
                "columns" => return Err(error(&format!("{p}.type"), "columns cannot nest")),
                _ => {
                    return Err(error(
                        &format!("{p}.type"),
                        "expected text, panel, callout, kpi, list, image or columns",
                    ))
                }
            };
            let b = object(v, &p, keys)?;
            if kind != "columns" {
                *count += 1;
                if *count > 64 {
                    return Err(error(&p, "at most 64 leaf blocks per slide"));
                }
            }
            match kind.as_str() {
                "text" => Ok(Block::Text(string(b, "text", &p)?)),
                "panel" => Ok(Block::Panel(optional(b, "label", &p)?, paragraphs(b, &p)?)),
                "callout" => {
                    let tone = string(b, "tone", &p)?;
                    if !matches!(tone.as_str(), "info" | "warning") {
                        return Err(error(&format!("{p}.tone"), "expected info or warning"));
                    }
                    Ok(Block::Callout(tone, paragraphs(b, &p)?))
                }
                "kpi" => Ok(Block::Kpi(
                    string(b, "value", &p)?,
                    string(b, "label", &p)?,
                    optional(b, "note", &p)?,
                )),
                "list" => Ok(Block::List(strings(b, "items", &p)?)),
                "image" => {
                    let alt = string(b, "alt", &p)?;
                    Ok(Block::Image(image(&string(b, "src", &p)?, &p, ctx)?, alt))
                }
                "columns" => {
                    let cols = items(b, "columns", &p, 2, 3)?
                        .iter()
                        .enumerate()
                        .map(|(j, value)| {
                            let cp = format!("{p}.columns[{j}]");
                            blocks(object(value, &cp, &["blocks"])?, &cp, false, count, ctx)
                        })
                        .collect::<Result<Vec<_>>>()?;
                    Ok(Block::Columns(cols))
                }
                _ => unreachable!(),
            }
        })
        .collect()
}

fn prose(texts: &[String]) -> String {
    texts
        .iter()
        .map(|t| format!("<p>{}</p>", escape(t)))
        .collect()
}

impl Block {
    fn html(&self) -> String {
        match self {
            Self::Text(t) => prose(std::slice::from_ref(t)),
            Self::Panel(label, p) => {
                let attr = label
                    .as_ref()
                    .map(|s| format!(" label=\"{}\"", escape(s)))
                    .unwrap_or_default();
                format!("<wb-panel{attr}>{}</wb-panel>", prose(p))
            }
            Self::Callout(tone, p) => {
                format!("<wb-callout tone=\"{tone}\">{}</wb-callout>", prose(p))
            }
            Self::Kpi(value, label, note) => {
                let note = note
                    .as_ref()
                    .map(|s| format!("<p class=\"source\">{}</p>", escape(s)))
                    .unwrap_or_default();
                format!(
                    "<wb-kpi value=\"{}\" label=\"{}\">{note}</wb-kpi>",
                    escape(value),
                    escape(label)
                )
            }
            Self::Image(uri, alt) => {
                format!("<img class=\"wb-image\" src=\"{}\" alt=\"{}\">", escape(uri), escape(alt))
            }
            Self::List(items) => {
                let li: String = items.iter().map(|t| format!("<li>{}</li>", escape(t))).collect();
                format!("<ul class=\"wb-list\">{li}</ul>")
            }
            Self::Columns(cols) => {
                let children = cols
                    .iter()
                    .map(|bs| format!("<wb-column>{}</wb-column>", html(bs)))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("<wb-columns>\n{children}\n</wb-columns>\n")
            }
        }
    }
}

fn html(blocks: &[Block]) -> String {
    blocks.iter().map(Block::html).collect()
}

#[cfg(test)]
pub fn parse(bytes: &[u8]) -> Result<(Vec<(String, String)>, Vec<Slide>)> {
    parse_in(bytes, None)
}

/// Parse with local images resolved under `base` (the spec or markup directory).
pub fn parse_in(bytes: &[u8], base: Option<&Path>) -> Result<(Vec<(String, String)>, Vec<Slide>)> {
    let mut ctx = Ctx { base, images: 0 };
    if bytes.len() > MAX_BYTES {
        return Err(error("$", "spec exceeds 64 KiB"));
    }
    let v: Value =
        serde_json::from_slice(bytes).map_err(|e| error("$", &format!("invalid JSON: {e}")))?;
    let root = object(&v, "$", &["schema_version", "title", "footer", "slides"])?;
    if field(root, "schema_version", "$")?.as_u64() != Some(1) {
        return Err(error("$.schema_version", "expected integer 1"));
    }
    let title = string(root, "title", "$")?;
    let slides = items(root, "slides", "$", 1, 20)?
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let p = format!("$.slides[{i}]");
            let s = object(v, &p, &["layout", "heading", "subtitle", "number", "image", "side", "blocks", "notes"])?;
            let layout = optional(s, "layout", &p)?
                .map(|name| {
                    LAYOUTS.iter().find(|(n, _)| *n == name).map(|(_, l)| *l).ok_or_else(|| {
                        let names: Vec<_> = LAYOUTS.iter().map(|(n, _)| *n).collect();
                        error(&format!("{p}.layout"), &format!("expected {}", names.join(" or ")))
                    })
                })
                .transpose()?;
            let heading = optional(s, "heading", &p)?;
            let subtitle = optional(s, "subtitle", &p)?;
            // Section number ("01"): a short label shown over the divider title.
            let short = |n: String| (n.chars().count() <= 8).then_some(n);
            let number = match optional(s, "number", &p).map(|n| n.map(short)) {
                Ok(None) => None,
                Ok(Some(Some(n))) => Some(n),
                _ => return Err(error(&format!("{p}.number"), "expected 1–8 characters, not blank")),
            };
            if number.is_some() && layout != Some("slide-section") {
                return Err(error(&format!("{p}.number"), "requires layout section"));
            }
            // Split: one slide-level image bleeding to `side` (default left); head and blocks fill the other side.
            let split = layout == Some("slide-split");
            let media = match s.get("image") {
                Some(img) if split => {
                    let side = match optional(s, "side", &p)?.as_deref() {
                        None | Some("left") => "left",
                        Some("right") => "right",
                        _ => return Err(error(&format!("{p}.side"), "expected left or right")),
                    };
                    let ip = format!("{p}.image");
                    let o = object(img, &ip, &["src", "alt"])?;
                    let alt = string(o, "alt", &ip)?;
                    let uri = image(&string(o, "src", &ip)?, &ip, &mut ctx)?;
                    format!("<img class=\"wb-split-image wb-split-{side}\" src=\"{}\" alt=\"{}\">", escape(&uri), escape(&alt))
                }
                None if split => return Err(error(&format!("{p}.image"), "required for layout split")),
                None if !s.contains_key("side") => String::new(),
                _ => {
                    let k = if s.contains_key("image") { "image" } else { "side" };
                    return Err(error(&format!("{p}.{k}"), "requires layout split"));
                }
            };
            // A variant may have no blocks: absent or [] means none.
            let no_blocks = s.get("blocks").map_or(true, |b| b.as_array().is_some_and(Vec::is_empty));
            if layout == Some("slide-quote") && !no_blocks {
                return Err(error(&format!("{p}.blocks"), "not allowed with layout quote"));
            }
            let mut body_html = if layout.is_some() && no_blocks {
                String::new()
            } else {
                html(&blocks(s, &p, true, &mut 0, &mut ctx)?)
            };
            // Speaker notes ride along in the slide, hidden: no layout, no paint, no innerText.
            if let Some(notes) = optional(s, "notes", &p)? {
                body_html.push_str(&format!("<aside class=\"wb-notes\" hidden>{}</aside>", escape(&notes)));
            }
            // A variant's template renders only content, so the heading group goes in the body.
            if let Some(layout) = layout {
                let heading = heading.ok_or_else(|| {
                    error(&format!("{p}.heading"), &format!("required for layout {}", s["layout"].as_str().unwrap()))
                })?;
                let sub = subtitle.as_ref().map_or(String::new(), |t| format!("<p class=\"wb-head-subtitle\">{}</p>", escape(&t)));
                let num = number.map_or(String::new(), |n| format!("<span class=\"wb-head-number\">{}</span>", escape(&n)));
                let head = if layout == "slide-quote" {
                    // Quote: heading is the quoted text, subtitle its attribution (outside the blockquote).
                    let by = subtitle.map_or(String::new(), |t| format!("<figcaption class=\"wb-quote-by\">{}</figcaption>", escape(&t)));
                    format!("<figure class=\"wb-quote\"><blockquote><p>{}</p></blockquote>{by}</figure>", escape(&heading))
                } else {
                    format!("<header class=\"wb-head\">{num}<h1 class=\"wb-head-title\">{}</h1>{sub}</header>", escape(&heading))
                };
                return Ok(Slide {
                    frontmatter: vec![("layout".into(), layout.into())],
                    body_html: media + &head + &body_html,
                    slots: Vec::new(),
                });
            }
            // No heading: content alone, centered (slide-section).
            let mut fm = match heading {
                Some(heading) => vec![
                    ("layout".into(), "slide-feature".into()),
                    ("heading".into(), heading),
                ],
                None if subtitle.is_some() => {
                    return Err(error(&format!("{p}.subtitle"), "requires heading"))
                }
                None => vec![("layout".into(), "slide-section".into())],
            };
            if let Some(subtitle) = subtitle {
                fm.push(("subtitle".into(), subtitle));
            }
            Ok(Slide {
                frontmatter: fm,
                body_html,
                slots: Vec::new(),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let mut meta = vec![("title".into(), title)];
    // The renderer inserts footer as HTML, so escape the literal text here.
    if let Some(footer) = optional(root, "footer", "$")? {
        if footer.chars().count() > 200 {
            return Err(error("$.footer", "expected at most 200 characters"));
        }
        meta.push(("footer".into(), escape(&footer)));
    }
    Ok((meta, slides))
}

pub fn load(path: &Path) -> Result<(Vec<(String, String)>, Vec<Slide>)> {
    let mut bytes = Vec::new();
    File::open(path)
        .and_then(|f| f.take((MAX_BYTES + 1) as u64).read_to_end(&mut bytes))
        .map_err(|e| error("$", &format!("cannot read spec: {e}")))?;
    parse_in(&bytes, Some(path.parent().filter(|d| !d.as_os_str().is_empty()).unwrap_or(Path::new("."))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn base() -> Value {
        json!({"schema_version":1,"title":"가상 검토","slides":[{"heading":"검토","blocks":[{"type":"text","text":"자료"}]}]})
    }

    #[test]
    fn errors_identify_fields_and_enforce_limits() {
        let mut cases = Vec::new();
        let mut v = base();
        v["schema_version"] = json!(2);
        cases.push((v, "$.schema_version"));
        let mut v = base();
        v["theme"] = json!("local");
        cases.push((v, "$.theme"));
        let mut v = base();
        v["slides"][0]["blocks"][0]["text"] = json!(42);
        cases.push((v, "$.slides[0].blocks[0].text"));
        let mut v = base();
        v["slides"][0]["blocks"][0]["html"] = json!("bad");
        cases.push((v, ".html"));
        let mut v = base();
        v["slides"] = json!([]);
        cases.push((v, "$.slides"));
        let mut v = base();
        v["slides"] = json!(vec![v["slides"][0].clone(); 21]);
        cases.push((v, "$.slides"));
        let mut v = base();
        v["slides"][0]["blocks"][0]["text"] = json!("가".repeat(2001));
        cases.push((v, ".text"));
        let mut v = base();
        v["slides"][0]["blocks"] = json!(vec![json!({"type":"text","text":"x"}); 65]);
        cases.push((v, ".blocks"));
        for (v, path) in cases {
            assert!(parse(v.to_string().as_bytes())
                .err()
                .unwrap()
                .contains(path));
        }
        assert!(parse(&vec![b' '; MAX_BYTES + 1])
            .err()
            .unwrap()
            .contains("64 KiB"));
    }

    #[test]
    fn column_structure_and_aggregate_leaf_limit() {
        let leaf = json!({"type":"text","text":"근거"});
        let column = json!({"blocks":[leaf.clone()]});
        let mut v = base();
        for columns in [json!([column.clone()]), json!(vec![column.clone(); 4])] {
            v["slides"][0]["blocks"] = json!([{"type":"columns","columns":columns}]);
            assert!(parse(v.to_string().as_bytes())
                .err()
                .unwrap()
                .contains(".columns"));
        }
        v["slides"][0]["blocks"] = json!([{"type":"columns","columns":[
            {"blocks":[{"type":"columns","columns":[column.clone(),column]}]},
            {"blocks":[leaf.clone()]}
        ]}]);
        assert!(parse(v.to_string().as_bytes())
            .err()
            .unwrap()
            .contains(".blocks[0].type: columns cannot nest"));
        v["slides"][0]["blocks"] = json!([{"type":"columns","columns":[
            {"blocks":vec![leaf.clone();33]}, {"blocks":vec![leaf;32]}
        ]}]);
        assert!(parse(v.to_string().as_bytes())
            .err()
            .unwrap()
            .contains("at most 64 leaf"));
    }

    #[test]
    fn list_items_are_escaped_and_bounded() {
        let mut v = base();
        v["slides"][0]["blocks"] = json!([{"type":"list","items":["a<b","둘"]}]);
        let (_, slides) = parse(v.to_string().as_bytes()).unwrap();
        assert_eq!(slides[0].body_html, "<ul class=\"wb-list\"><li>a&lt;b</li><li>둘</li></ul>");
        v["slides"][0]["blocks"] = json!([{"type":"list","items":[]}]);
        assert!(parse(v.to_string().as_bytes()).err().unwrap().contains(".items: expected 1–16"));
        v["slides"][0]["blocks"] = json!([{"type":"list","items":[" "]}]);
        assert!(parse(v.to_string().as_bytes()).err().unwrap().contains(".items[0]"));
    }

    #[test]
    fn images_embed_local_files_only() {
        let dir = std::env::temp_dir().join(format!("wb-spec-img-{}", std::process::id()));
        let inside = dir.join("deck");
        std::fs::create_dir_all(inside.join("img")).unwrap();
        std::fs::write(inside.join("img/a.svg"), "<svg xmlns='http://www.w3.org/2000/svg'/>").unwrap();
        std::fs::write(dir.join("secret.png"), "x").unwrap();
        std::fs::write(inside.join("big.png"), vec![0u8; MAX_IMAGE_BYTES as usize + 1]).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(dir.join("secret.png"), inside.join("link.png")).ok();
        #[cfg(unix)]
        std::process::Command::new("mkfifo").arg(inside.join("pipe.png")).status().unwrap();
        std::fs::create_dir_all(inside.join("dir.png")).unwrap();
        let spec = |src: &str| {
            let mut v = base();
            v["slides"][0]["blocks"] = json!([{"type":"image","src":src,"alt":"그림 \"<1>\""}]);
            v.to_string()
        };
        let (_, slides) = parse_in(spec("img/a.svg").as_bytes(), Some(&inside)).unwrap();
        assert!(slides[0].body_html.starts_with("<img class=\"wb-image\" src=\"data:image/svg+xml;base64,"));
        assert!(slides[0].body_html.ends_with("alt=\"그림 &quot;&lt;1&gt;&quot;\">"));
        assert!(parse(spec("img/a.svg").as_bytes()).err().unwrap().contains("need a spec file location"));
        for (src, msg) in [
            ("../secret.png", "relative local path"),
            ("/etc/x.png", "relative local path"),
            ("http://x/y.png", "relative local path"),
            ("img/a.gif", "expected .png"),
            ("missing.png", "cannot read image"),
            ("big.png", "exceeds 2 MiB"),
            #[cfg(unix)]
            ("link.png", "outside the spec directory"),
            #[cfg(unix)]
            ("pipe.png", "regular file"),
            ("dir.png", "regular file"),
        ] {
            let err = parse_in(spec(src).as_bytes(), Some(&inside)).err().unwrap();
            assert!(err.contains("$.slides[0].blocks[0].src") && err.contains(msg), "{src}: {err}");
        }
        let mut v = base();
        v["slides"][0]["blocks"] = json!(vec![json!({"type":"image","src":"img/a.svg","alt":"a"}); 9]);
        assert!(parse_in(v.to_string().as_bytes(), Some(&inside)).err().unwrap().contains("at most 8 images"));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn footer_is_escaped_deck_text() {
        let mut v = base();
        v["footer"] = json!("<b>팀</b> & 가상");
        let (meta, _) = parse(v.to_string().as_bytes()).unwrap();
        assert_eq!(meta[1], ("footer".to_string(), "&lt;b&gt;팀&lt;/b&gt; &amp; 가상".to_string()));
        v["footer"] = json!("x".repeat(201));
        assert!(parse(v.to_string().as_bytes()).err().unwrap().contains("$.footer"));
    }

    #[test]
    fn notes_are_hidden_escaped_slide_text() {
        let mut v = base();
        v["slides"][0]["notes"] = json!("<b>말할 것</b> & 확인");
        let (_, slides) = parse(v.to_string().as_bytes()).unwrap();
        assert_eq!(slides[0].body_html, "<p>자료</p><aside class=\"wb-notes\" hidden>&lt;b&gt;말할 것&lt;/b&gt; &amp; 확인</aside>");
        for bad in [json!(" "), json!("x".repeat(2001)), json!(1)] {
            v["slides"][0]["notes"] = bad;
            assert!(parse(v.to_string().as_bytes()).err().unwrap().contains("$.slides[0].notes"));
        }
    }

    #[test]
    fn heading_is_optional_and_centers_content() {
        let mut v = base();
        v["slides"][0].as_object_mut().unwrap().remove("heading");
        let (_, slides) = parse(v.to_string().as_bytes()).unwrap();
        assert_eq!(slides[0].frontmatter, vec![("layout".to_string(), "slide-section".to_string())]);
        v["slides"][0]["subtitle"] = json!("부제");
        assert!(parse(v.to_string().as_bytes()).err().unwrap().contains("$.slides[0].subtitle: requires heading"));
    }

    #[test]
    fn layout_variants_map_and_emit_an_escaped_head() {
        let mut v = base();
        v["slides"][0]["layout"] = json!("cover");
        v["slides"][0]["heading"] = json!("<가상> & 표지");
        v["slides"][0]["subtitle"] = json!("부제 & 설명");
        v["slides"][0]["notes"] = json!("메모");
        let (_, slides) = parse(v.to_string().as_bytes()).unwrap();
        assert_eq!(slides[0].frontmatter, vec![("layout".to_string(), "slide-cover".to_string())]);
        assert_eq!(
            slides[0].body_html,
            "<header class=\"wb-head\"><h1 class=\"wb-head-title\">&lt;가상&gt; &amp; 표지</h1><p class=\"wb-head-subtitle\">부제 &amp; 설명</p></header><p>자료</p><aside class=\"wb-notes\" hidden>메모</aside>"
        );
        // Section, no subtitle, no blocks (absent or empty).
        v["slides"][0] = json!({"layout": "section", "heading": "장", "blocks": []});
        let (_, slides) = parse(v.to_string().as_bytes()).unwrap();
        assert_eq!(slides[0].frontmatter, vec![("layout".to_string(), "slide-section".to_string())]);
        assert_eq!(slides[0].body_html, "<header class=\"wb-head\"><h1 class=\"wb-head-title\">장</h1></header>");
        v["slides"][0].as_object_mut().unwrap().remove("blocks");
        assert!(parse(v.to_string().as_bytes()).is_ok());
    }

    #[test]
    fn section_number_heads_the_divider() {
        let mut v = base();
        v["slides"][0] = json!({"layout": "section", "heading": "장", "number": "<01>"});
        let (_, slides) = parse(v.to_string().as_bytes()).unwrap();
        assert_eq!(slides[0].body_html, "<header class=\"wb-head\"><span class=\"wb-head-number\">&lt;01&gt;</span><h1 class=\"wb-head-title\">장</h1></header>");
        for (n, msg) in [(json!("123456789"), "expected 1–8 characters, not blank"), (json!(" "), "expected 1–8 characters, not blank"), (json!(1), "expected 1–8 characters, not blank")] {
            v["slides"][0]["number"] = n;
            assert_eq!(parse(v.to_string().as_bytes()).err().unwrap(), format!("$.slides[0].number: {msg}"));
        }
        for slide in [json!({"layout": "cover", "heading": "h", "number": "01"}), json!({"heading": "h", "number": "01", "blocks": [{"type":"text","text":"x"}]})] {
            v["slides"][0] = slide;
            assert_eq!(parse(v.to_string().as_bytes()).err().unwrap(), "$.slides[0].number: requires layout section");
        }
    }

    #[test]
    fn quote_is_a_figure_with_attribution() {
        let mut v = base();
        v["slides"][0] = json!({"layout": "quote", "heading": "<좋은> 말", "subtitle": "A & B, CEO"});
        let (_, slides) = parse(v.to_string().as_bytes()).unwrap();
        assert_eq!(slides[0].frontmatter, vec![("layout".to_string(), "slide-quote".to_string())]);
        assert_eq!(slides[0].body_html, "<figure class=\"wb-quote\"><blockquote><p>&lt;좋은&gt; 말</p></blockquote><figcaption class=\"wb-quote-by\">A &amp; B, CEO</figcaption></figure>");
        v["slides"][0] = json!({"layout": "quote", "heading": "말"});
        assert_eq!(parse(v.to_string().as_bytes()).unwrap().1[0].body_html, "<figure class=\"wb-quote\"><blockquote><p>말</p></blockquote></figure>");
        for (slide, msg) in [
            (json!({"layout": "quote"}), "$.slides[0].heading: required for layout quote"),
            (json!({"layout": "quote", "heading": "말", "subtitle": " "}), "$.slides[0].subtitle: expected 1–2000 characters, not blank"),
            (json!({"layout": "quote", "heading": "말", "number": "01"}), "$.slides[0].number: requires layout section"),
            (json!({"layout": "quote", "heading": "말", "blocks": [{"type": "text", "text": "x"}]}), "$.slides[0].blocks: not allowed with layout quote"),
        ] {
            v["slides"][0] = slide;
            assert_eq!(parse(v.to_string().as_bytes()).err().unwrap(), msg);
        }
    }

    #[test]
    fn split_puts_one_checked_image_beside_the_head() {
        let dir = std::env::temp_dir().join(format!("wb-spec-split-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.svg"), "<svg xmlns='http://www.w3.org/2000/svg'/>").unwrap();
        let mut v = base();
        v["slides"][0] = json!({"layout": "split", "image": {"src": "a.svg", "alt": "<팀> \"사진\""}, "side": "right",
            "heading": "A & B", "blocks": [{"type": "text", "text": "본문"}]});
        let (_, slides) = parse_in(v.to_string().as_bytes(), Some(&dir)).unwrap();
        assert_eq!(slides[0].frontmatter, vec![("layout".to_string(), "slide-split".to_string())]);
        let html = &slides[0].body_html;
        assert!(html.starts_with("<img class=\"wb-split-image wb-split-right\" src=\"data:image/svg+xml;base64,"), "{html}");
        assert!(html.ends_with("alt=\"&lt;팀&gt; &quot;사진&quot;\"><header class=\"wb-head\"><h1 class=\"wb-head-title\">A &amp; B</h1></header><p>본문</p>"), "{html}");
        v["slides"][0] = json!({"layout": "split", "image": {"src": "a.svg", "alt": "a"}, "heading": "T"});
        assert!(parse_in(v.to_string().as_bytes(), Some(&dir)).unwrap().1[0].body_html.contains("wb-split-left"));
        // The slide image counts toward the deck limit like any image block.
        v["slides"][0]["blocks"] = json!(vec![json!({"type":"image","src":"a.svg","alt":"a"}); 8]);
        assert!(parse_in(v.to_string().as_bytes(), Some(&dir)).err().unwrap().contains("at most 8 images"));
        for (slide, msg) in [
            (json!({"layout": "split", "heading": "T"}), "$.slides[0].image: required for layout split"),
            (json!({"layout": "split", "heading": "T", "image": "a.svg"}), "$.slides[0].image: expected object"),
            (json!({"layout": "split", "heading": "T", "image": {"src": "a.svg"}}), "$.slides[0].image.alt: missing required field"),
            (json!({"layout": "split", "heading": "T", "image": {"src": "a.svg", "alt": "a", "w": 1}}), "$.slides[0].image.w: unknown field"),
            (json!({"layout": "split", "heading": "T", "image": {"src": "/a.svg", "alt": "a"}}), "$.slides[0].image.src: expected a relative local path without .. or a scheme"),
            (json!({"layout": "split", "heading": "T", "image": {"src": "a.svg", "alt": "a"}, "side": "top"}), "$.slides[0].side: expected left or right"),
            (json!({"layout": "split", "image": {"src": "a.svg", "alt": "a"}}), "$.slides[0].heading: required for layout split"),
            (json!({"heading": "T", "image": {"src": "a.svg", "alt": "a"}, "blocks": [{"type": "text", "text": "x"}]}), "$.slides[0].image: requires layout split"),
            (json!({"layout": "cover", "heading": "T", "side": "left"}), "$.slides[0].side: requires layout split"),
        ] {
            v["slides"][0] = slide;
            assert_eq!(parse_in(v.to_string().as_bytes(), Some(&dir)).err().unwrap(), msg);
        }
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn layout_errors() {
        let mut v = base();
        v["slides"][0]["layout"] = json!("pull-quote");
        assert_eq!(parse(v.to_string().as_bytes()).err().unwrap(), "$.slides[0].layout: expected cover or section or quote or split");
        v["slides"][0] = json!({"layout": "cover", "blocks": [{"type":"text","text":"x"}]});
        assert_eq!(parse(v.to_string().as_bytes()).err().unwrap(), "$.slides[0].heading: required for layout cover");
        // Without a layout, blocks stay required (1–64).
        v["slides"][0] = json!({"heading": "h", "blocks": []});
        assert!(parse(v.to_string().as_bytes()).err().unwrap().contains("$.slides[0].blocks: expected 1–64"));
        v["slides"][0] = json!({"heading": "h"});
        assert!(parse(v.to_string().as_bytes()).err().unwrap().contains("$.slides[0].blocks: missing"));
    }

    #[test]
    fn text_is_literal_and_never_template_or_markdown_source() {
        let mut v = base();
        v["slides"][0]["blocks"][0]["text"] = json!("<script> & **bold** {{heading}} 한글");
        let (_, slides) = parse(v.to_string().as_bytes()).unwrap();
        assert_eq!(
            slides[0].body_html,
            "<p>&lt;script&gt; &amp; **bold** {{heading}} 한글</p>"
        );
    }
}
