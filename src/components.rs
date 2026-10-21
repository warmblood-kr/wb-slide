//! Optional authoring tags lowered to plain HTML, without client registration.
//! This is a component compiler for trusted decks, not an HTML sanitizer.
use lol_html::{element, end_tag, rewrite_str, RewriteStrSettings};
use std::{cell::RefCell, collections::HashSet, rc::Rc};

const MAX_BYTES: usize = 2 * 1024 * 1024;
const MAX_DEPTH: usize = 16;
const MAX_COMPONENTS: usize = 1000;

#[derive(Default)]
struct State {
    stack: Vec<(String, usize)>,
    count: usize,
    direct_columns: HashSet<usize>,
}

// A streaming rewrite handler sees matched elements, but cannot see orphan
// end tags. Tokenize first so unknown/unmatched component closes cannot pass
// through. Raw-text states keep script/style/textarea examples out of the
// component grammar; comments and escaped code are not tags.
fn check_pairs(html: &str) -> Result<(), String> {
    use html5gum::{DefaultEmitter, Token, Tokenizer};
    let mut emitter = DefaultEmitter::<usize>::new_with_span();
    emitter.naively_switch_states(true);
    let mut stack = Vec::new();
    for token in Tokenizer::new_with_emitter(html, emitter) {
        let token = token.map_err(|error| error.to_string())?;
        let (name, close, position) = match &token {
            Token::StartTag(tag) => (&tag.name, false, tag.span.start),
            Token::EndTag(tag) => (&tag.name, true, tag.span.start),
            _ => continue,
        };
        if !name.starts_with(b"wb-") {
            continue;
        }
        let name = String::from_utf8_lossy(name).to_string();
        if !matches!(
            name.as_str(),
            "wb-columns" | "wb-column" | "wb-panel" | "wb-kpi" | "wb-callout"
        ) {
            return Err(format!(
                "unknown component {}<{name}> at HTML byte {position}",
                if close { "closing tag for " } else { "" }
            ));
        }
        if close {
            if stack.pop().as_deref() != Some(name.as_str()) {
                return Err(format!("unmatched </{name}> at HTML byte {position}"));
            }
        } else {
            stack.push(name);
            if stack.len() > MAX_DEPTH {
                return Err("component nesting limit exceeded (16)".into());
            }
        }
    }
    if let Some(name) = stack.last() {
        return Err(format!(
            "unclosed component <{name}>; use an explicit closing tag"
        ));
    }
    Ok(())
}

// Attribute values returned by lol_html retain character references. Keep
// those references (decoded once by the browser), but escape literal markup.
fn text(value: &str) -> String {
    value.replace('<', "&lt;").replace('>', "&gt;")
}

pub fn expand(html: &str) -> Result<String, String> {
    // Existing raw-HTML decks retain their exact output and parsing behavior.
    if !html
        .as_bytes()
        .windows(4)
        .any(|s| s.eq_ignore_ascii_case(b"<wb-"))
        && !html
            .as_bytes()
            .windows(5)
            .any(|s| s.eq_ignore_ascii_case(b"</wb-"))
    {
        return Ok(html.to_string());
    }
    if html.len() > MAX_BYTES {
        return Err("component input exceeds 2 MiB per slide".into());
    }
    check_pairs(html)?;
    let state = Rc::new(RefCell::new(State::default()));
    let shared = state.clone();
    let direct = state.clone();
    let source = Rc::new(html.to_string());
    let output = rewrite_str(
        html,
        RewriteStrSettings::new()
            // Use the HTML parser's actual direct-parent relationship, rather
            // than the component-only ancestor stack used for expansion limits.
            .append_element_content_handler(element!("wb-columns > wb-column", move |el| {
                direct
                    .borrow_mut()
                    .direct_columns
                    .insert(el.source_location().bytes().start);
                Ok(())
            }))
            .append_element_content_handler(element!("wb-columns > :not(wb-column)", |_| {
                Err(
                    "<wb-columns> accepts only direct <wb-column> elements; remove HTML wrappers"
                        .into(),
                )
            }))
            .append_element_content_handler(element!("*", move |el| {
                let name = el.tag_name();
                if !name.starts_with("wb-") {
                    return Ok(());
                }
                let allowed: &[&str] = match name.as_str() {
                    "wb-columns" | "wb-column" => &[],
                    "wb-panel" => &["label"],
                    "wb-kpi" => &["value", "label"],
                    "wb-callout" => &["tone"],
                    _ => return Err(format!("unknown component <{name}>").into()),
                };
                for attr in el.attributes() {
                    if !allowed.contains(&attr.name().as_str()) {
                        return Err(
                            format!("unknown property `{}` on <{name}>", attr.name()).into()
                        );
                    }
                }
                if el.is_self_closing() || el.namespace_uri() != "http://www.w3.org/1999/xhtml" {
                    return Err(format!("<{name}> requires an explicit HTML closing tag").into());
                }
                let mut state = shared.borrow_mut();
                state.count += 1;
                if state.stack.len() >= MAX_DEPTH || state.count > MAX_COMPONENTS {
                    return Err("component nesting/count limit exceeded (16 / 1000)".into());
                }
                if name == "wb-column" {
                    if !state
                        .direct_columns
                        .remove(&el.source_location().bytes().start)
                    {
                        return Err("<wb-column> requires a direct <wb-columns> parent".into());
                    }
                    match state.stack.last_mut() {
                        Some((parent, count)) if parent == "wb-columns" => *count += 1,
                        _ => return Err("<wb-column> requires a <wb-columns> parent".into()),
                    }
                } else if state
                    .stack
                    .last()
                    .is_some_and(|(parent, _)| parent == "wb-columns")
                {
                    return Err("wrap each component in <wb-columns> in <wb-column>".into());
                }
                state.stack.push((name.clone(), 0));
                drop(state);

                let class = match name.as_str() {
                    "wb-columns" => "cols",
                    "wb-column" => "col",
                    "wb-panel" => "panel",
                    "wb-kpi" => "wb-kpi",
                    "wb-callout" => match el.get_attribute("tone").as_deref().unwrap_or("info") {
                        "info" => "info-box",
                        "warning" => "warn-box",
                        _ => return Err("<wb-callout> tone must be info or warning".into()),
                    },
                    _ => unreachable!(),
                };
                let mut prefix = String::new();
                if name == "wb-kpi" {
                    for key in ["value", "label"] {
                        let value = el
                            .get_attribute(key)
                            .filter(|v| !v.trim().is_empty())
                            .ok_or_else(|| format!("<wb-kpi> requires `{key}`"))?;
                        let class = if key == "value" { "stat" } else { "stat-label" };
                        prefix.push_str(&format!("<div class=\"{class}\">{}</div>", text(&value)));
                    }
                } else if name == "wb-panel" {
                    if let Some(label) = el.get_attribute("label") {
                        prefix = format!("<div class=\"panel-label\">{}</div>", text(&label));
                    }
                }
                for attr in allowed {
                    el.remove_attribute(attr);
                }
                el.set_tag_name("div")?;
                el.set_attribute("class", class)?;
                el.prepend(&prefix, lol_html::html_content::ContentType::Html);
                let closing = shared.clone();
                let original = source.clone();
                el.on_end_tag(end_tag!(move |end| {
                    // A tolerant HTML parser can implicitly close an element at
                    // another tag. Component authoring requires the actual pair.
                    let raw = &original[end.source_location().bytes()];
                    let actual = raw
                        .trim_start_matches("</")
                        .trim_end_matches('>')
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .to_ascii_lowercase();
                    if actual != name {
                        return Err(format!("<{name}> requires </{name}> before {raw}").into());
                    }
                    let mut state = closing.borrow_mut();
                    let (opened, children) =
                        state.stack.pop().ok_or("unexpected component close")?;
                    if opened != name {
                        return Err(format!("<{opened}> must close before </{name}>").into());
                    }
                    if name == "wb-columns" && !(2..=3).contains(&children) {
                        return Err(
                            "<wb-columns> requires two or three <wb-column> children".into()
                        );
                    }
                    Ok(())
                }))?;
                Ok(())
            })),
    )
    .map_err(|error| error.to_string())?;
    if let Some((name, _)) = state.borrow().stack.last() {
        return Err(format!(
            "unclosed component <{name}>; use an explicit closing tag"
        ));
    }
    if output.len() > MAX_BYTES {
        return Err("component output exceeds 2 MiB per slide".into());
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_components_lower_without_runtime() {
        let html = expand(r#"<wb-columns><wb-column><wb-panel label="비교"><b>본문</b></wb-panel></wb-column><wb-column><wb-kpi value="12%" label="가상 증가"></wb-kpi></wb-column></wb-columns>"#).unwrap();
        assert!(html.contains(r#"<div class="cols"><div class="col"><div class="panel">"#));
        assert!(html.contains(r#"<div class="stat-label">가상 증가</div>"#));
        assert!(!html.contains("wb-columns"));
        assert!(html.contains("<b>본문</b>"));
    }

    #[test]
    fn attribute_text_preserves_entities_without_injecting_html() {
        let html =
            expand(r#"<wb-panel label='A &amp; B &lt; C <img src=x>'>Body</wb-panel>"#).unwrap();
        assert!(html.contains("A &amp; B &lt; C &lt;img src=x&gt;"));
        assert!(!html.contains("<img"));
        assert!(!html.contains("&amp;amp;"));
    }

    #[test]
    fn rejects_unknown_props_and_invalid_values() {
        for input in [
            "<wb-missing></wb-missing>",
            "<wb-panel onclick='x'></wb-panel>",
            "<wb-callout tone='danger'></wb-callout>",
            "<wb-kpi value='1'></wb-kpi>",
            "<wb-column></wb-column>",
            "<wb-columns></wb-columns>",
            "<wb-panel/>",
            "<wb-panel>missing",
            "<wb-panel><wb-callout></wb-panel>",
        ] {
            assert!(expand(input).is_err(), "must reject: {input}");
        }
    }

    #[test]
    fn ordinary_html_code_and_svg_are_preserved() {
        let html = r#"<div id="raw"><pre>&lt;wb-unknown&gt;</pre><svg><text>가상</text></svg><!-- <wb-unknown> --></div>"#;
        assert_eq!(expand(html).unwrap(), html);
    }

    #[test]
    fn rejects_html_wrappers_between_columns_and_their_parent() {
        for input in [
            "<wb-columns><div><wb-column>A</wb-column><wb-column>B</wb-column></div></wb-columns>",
            "<wb-columns><wb-column>A</wb-column><div><wb-column>B</wb-column></div></wb-columns>",
        ] {
            assert!(expand(input).unwrap_err().contains("direct"));
        }
        assert!(expand(
            "<wb-columns><wb-column><div>A</div></wb-column><wb-column>B</wb-column></wb-columns>"
        )
        .is_ok());
    }

    #[test]
    fn rejects_orphan_unknown_and_extra_component_end_tags() {
        for input in [
            "<wb-panel>body</wb-panel></wb-missing>",
            "<wb-panel>body</wb-panel></wb-panel>",
            "</wb-panel>",
            "</wb-missing>",
            "</WB-PANEL>",
        ] {
            assert!(expand(input).is_err(), "must reject: {input}");
        }
        let literal = "<wb-panel>body</wb-panel><!-- </wb-missing> --><script>const example = '</wb-panel>';</script><style>/* </wb-missing> */</style><textarea></wb-missing></textarea><div title='</wb-missing>'>text</div>";
        assert!(expand(literal).is_ok());
    }

    #[test]
    fn bounded_expansion() {
        assert!(expand(&format!("<wb-panel>{}</wb-panel>", "x".repeat(MAX_BYTES))).is_err());
        assert!(expand(&format!(
            "{}{}",
            "<wb-panel>".repeat(17),
            "</wb-panel>".repeat(17)
        ))
        .is_err());
        assert!(expand(&"<wb-panel></wb-panel>".repeat(MAX_COMPONENTS + 1)).is_err());
    }
}
