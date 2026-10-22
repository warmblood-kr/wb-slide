//! Closed, text-only slide data. No Markdown parsing or deck-local code loading.
use crate::{template::html_escape as escape, Slide};
use serde_json::{Map, Value};
use std::{fs::File, io::Read, path::Path};

const MAX_IMAGE_BYTES: u64 = 2 * 1024 * 1024;
const MAX_IMAGES: usize = 8;
const MAX_DIAGRAMS: usize = 16;

/// Where local images may come from, the diagram preset theme, and how many
/// images/diagrams were embedded so far.
struct Ctx<'a> {
    base: Option<&'a Path>,
    theme: Option<&'a str>,
    images: usize,
    diagrams: usize,
}

const MAX_BYTES: usize = 64 * 1024;
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
    Diagram(String),
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

/// Render a diagram block to inline SVG; data errors gain the block path.
fn diagram(b: &Object, p: &str, ctx: &mut Ctx) -> Result<String> {
    let alt = string(b, "alt", p)?;
    let kind = string(b, "diagram", p)?;
    if !crate::diagram::TYPES.contains(&kind.as_str()) {
        return Err(error(&format!("{p}.diagram"), &format!("expected one of {}", crate::diagram::TYPES.join(", "))));
    }
    ctx.diagrams += 1;
    if ctx.diagrams > MAX_DIAGRAMS {
        return Err(error(p, "at most 16 diagrams per deck"));
    }
    let data = field(b, "data", p)?;
    let svg = crate::diagram::render(&kind, data, ctx.theme).map_err(|e| match e.strip_prefix('$') {
        Some(rest) => format!("{p}.data{rest}"),
        None => error(&format!("{p}.data"), &e),
    })?;
    let attrs = format!("<svg class=\"wb-diagram\" role=\"img\" aria-label=\"{}\" ", escape(&alt));
    Ok(svg.trim_end().replacen("<svg ", &attrs, 1))
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
                "diagram" => &["type", "diagram", "alt", "data"],
                "columns" if columns_allowed => &["type", "columns"],
                "columns" => return Err(error(&format!("{p}.type"), "columns cannot nest")),
                _ => {
                    return Err(error(
                        &format!("{p}.type"),
                        "expected text, panel, callout, kpi, list, image, diagram or columns",
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
                "diagram" => Ok(Block::Diagram(diagram(b, &p, ctx)?)),
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
            Self::Diagram(svg) => svg.clone(),
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
    parse_in(bytes, None, None)
}

/// Parse with local images resolved under `base` (the spec or markup directory)
/// and diagrams drawn with preset `theme` (None = default).
pub fn parse_in(bytes: &[u8], base: Option<&Path>, theme: Option<&str>) -> Result<(Vec<(String, String)>, Vec<Slide>)> {
    let mut ctx = Ctx { base, theme, images: 0, diagrams: 0 };
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
            let s = object(v, &p, &["heading", "subtitle", "blocks", "notes"])?;
            let heading = optional(s, "heading", &p)?;
            let subtitle = optional(s, "subtitle", &p)?;
            let mut body_html = html(&blocks(s, &p, true, &mut 0, &mut ctx)?);
            // Speaker notes ride along in the slide, hidden: no layout, no paint, no innerText.
            if let Some(notes) = optional(s, "notes", &p)? {
                body_html.push_str(&format!("<aside class=\"wb-notes\" hidden>{}</aside>", escape(&notes)));
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
    parse_in(&bytes, Some(path.parent().filter(|d| !d.as_os_str().is_empty()).unwrap_or(Path::new("."))), None)
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
        let (_, slides) = parse_in(spec("img/a.svg").as_bytes(), Some(&inside), None).unwrap();
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
            let err = parse_in(spec(src).as_bytes(), Some(&inside), None).err().unwrap();
            assert!(err.contains("$.slides[0].blocks[0].src") && err.contains(msg), "{src}: {err}");
        }
        let mut v = base();
        v["slides"][0]["blocks"] = json!(vec![json!({"type":"image","src":"img/a.svg","alt":"a"}); 9]);
        assert!(parse_in(v.to_string().as_bytes(), Some(&inside), None).err().unwrap().contains("at most 8 images"));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn diagrams_render_inline_svg_with_limits() {
        let q = json!({"quadrants":[{"title":"가"},{"title":"나"},{"title":"다"},{"title":"라"}]});
        let block = |extra: Value| {
            let mut b = json!({"type":"diagram","diagram":"quadrant","alt":"사분면 \"<1>\"","data":q});
            b.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
            let mut v = base();
            v["slides"][0]["blocks"] = json!([b]);
            v.to_string()
        };
        let (_, slides) = parse(block(json!({})).as_bytes()).unwrap();
        let html = &slides[0].body_html;
        assert!(html.starts_with("<svg class=\"wb-diagram\" role=\"img\" aria-label=\"사분면 &quot;&lt;1&gt;&quot;\" xmlns="), "{html}");
        assert!(html.contains(&format!("viewBox=\"0 0 {} {}\"", crate::diagram::W, crate::diagram::H)) && html.ends_with("</svg>"));
        assert_eq!(parse(block(json!({})).as_bytes()).unwrap().1[0].body_html, *html);
        let night = parse_in(block(json!({})).as_bytes(), None, Some("night")).unwrap().1;
        assert_ne!(night[0].body_html, *html);
        for (extra, msg) in [
            (json!({"diagram":"pie"}), "$.slides[0].blocks[0].diagram: expected one of quadrant"),
            (json!({"data":{"quadrants":[{"title":"가","x":1},{"title":"나"},{"title":"다"},{"title":"라"}]}}), "$.slides[0].blocks[0].data.quadrants[0].x: unknown field"),
            (json!({"src":"a.svg"}), "$.slides[0].blocks[0].src: unknown field"),
            (json!({"alt":" "}), "$.slides[0].blocks[0].alt"),
        ] {
            let err = parse(block(extra).as_bytes()).err().unwrap();
            assert!(err.contains(msg), "{err}");
        }
        let one = json!({"type":"diagram","diagram":"quadrant","alt":"a","data":q});
        let mut v = base();
        v["slides"] = json!([{"blocks":vec![one.clone(); 8]}, {"blocks":vec![one.clone(); 8]}]);
        assert!(parse(v.to_string().as_bytes()).is_ok());
        v["slides"][1]["blocks"] = json!(vec![one; 9]);
        assert!(parse(v.to_string().as_bytes()).err().unwrap().contains("$.slides[1].blocks[8]: at most 16 diagrams"));
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
