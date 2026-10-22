//! Closed, text-only slide data. No Markdown parsing or deck-local code loading.
use crate::{template::html_escape as escape, Slide};
use serde_json::{Map, Value};
use std::{fs::File, io::Read, path::Path};

const MAX_BYTES: usize = 64 * 1024;
type Object = Map<String, Value>;
type Result<T> = std::result::Result<T, String>;

#[derive(Debug)]
enum Block {
    Text(String),
    Panel(Option<String>, Vec<String>),
    Callout(String, Vec<String>),
    Kpi(String, String, Option<String>),
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
    items(o, "paragraphs", path, 1, 16)?
        .iter()
        .enumerate()
        .map(|(i, value)| {
            let p = format!("{path}.paragraphs[{i}]");
            let text = value.as_str().ok_or_else(|| error(&p, "expected string"))?;
            if text.trim().is_empty() || text.chars().count() > 2000 {
                return Err(error(&p, "expected 1–2000 characters, not blank"));
            }
            Ok(text.to_string())
        })
        .collect()
}

fn blocks(o: &Object, path: &str, columns_allowed: bool, count: &mut usize) -> Result<Vec<Block>> {
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
                "columns" if columns_allowed => &["type", "columns"],
                "columns" => return Err(error(&format!("{p}.type"), "columns cannot nest")),
                _ => {
                    return Err(error(
                        &format!("{p}.type"),
                        "expected text, panel, callout, kpi or columns",
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
                "columns" => {
                    let cols = items(b, "columns", &p, 2, 3)?
                        .iter()
                        .enumerate()
                        .map(|(j, value)| {
                            let cp = format!("{p}.columns[{j}]");
                            blocks(object(value, &cp, &["blocks"])?, &cp, false, count)
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

pub fn parse(bytes: &[u8]) -> Result<(Vec<(String, String)>, Vec<Slide>)> {
    if bytes.len() > MAX_BYTES {
        return Err(error("$", "spec exceeds 64 KiB"));
    }
    let v: Value =
        serde_json::from_slice(bytes).map_err(|e| error("$", &format!("invalid JSON: {e}")))?;
    let root = object(&v, "$", &["schema_version", "title", "slides"])?;
    if field(root, "schema_version", "$")?.as_u64() != Some(1) {
        return Err(error("$.schema_version", "expected integer 1"));
    }
    let title = string(root, "title", "$")?;
    let slides = items(root, "slides", "$", 1, 20)?
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let p = format!("$.slides[{i}]");
            let s = object(v, &p, &["heading", "subtitle", "blocks"])?;
            let heading = optional(s, "heading", &p)?;
            let subtitle = optional(s, "subtitle", &p)?;
            let body = blocks(s, &p, true, &mut 0)?;
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
                body_html: html(&body),
                slots: Vec::new(),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok((vec![("title".into(), title)], slides))
}

pub fn load(path: &Path) -> Result<(Vec<(String, String)>, Vec<Slide>)> {
    let mut bytes = Vec::new();
    File::open(path)
        .and_then(|f| f.take((MAX_BYTES + 1) as u64).read_to_end(&mut bytes))
        .map_err(|e| error("$", &format!("cannot read spec: {e}")))?;
    parse(&bytes)
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
