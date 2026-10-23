//! 2x2 / four-quadrant (also SWOT): quadrants in order TL, TR, BL, BR.
use super::*;

const GAP: f64 = 8.0;
const INSET: f64 = 16.0;
const INSET_Y: f64 = 12.0;
const AXIS_BAND: f64 = 32.0;

struct Axis {
    label: Option<String>,
    low: Option<String>,
    high: Option<String>,
}

fn axis(m: &Map<String, Value>, key: &str) -> Result<Option<Axis>> {
    let Some(v) = m.get(key) else { return Ok(None) };
    let path = format!("$.{key}");
    let a = obj(v, &path, &["label", "low", "high"])?;
    Ok(Some(Axis { label: opt_str(a, "label", &path, 40)?, low: opt_str(a, "low", &path, 40)?, high: opt_str(a, "high", &path, 40)? }))
}

fn snap(v: f64) -> f64 {
    (v / 8.0).floor() * 8.0
}

pub fn draw(data: &Value, svg: &mut Svg) -> Result<()> {
    let m = obj(data, "$", &["x", "y", "quadrants", "highlight"])?;
    let (x, y) = (axis(m, "x")?, axis(m, "y")?);
    let highlight = match m.get("highlight") {
        None => None,
        Some(v) => Some(v.as_u64().filter(|h| *h < 4).ok_or("$.highlight: expected 0-3")? as usize),
    };
    let quads = arr(m, "quadrants", "$", 4, 4)?;

    // Left gutter sized to the widest y label (one line at 14px), capped at 96px.
    let gutter = y.as_ref().map_or(0.0, |a| {
        let w = [&a.label, &a.low, &a.high].iter().filter_map(|s| s.as_deref()).map(|s| width(s, 14.0)).fold(0.0, f64::max);
        (((w + 40.0) / 8.0).ceil() * 8.0).clamp(56.0, 96.0)
    });
    let (x0, y0) = (PAD + gutter, PAD);
    let (x1, y1) = (W - PAD, H - PAD - if x.is_some() { AXIS_BAND } else { 0.0 });
    let (cw, ch) = (snap((x1 - x0 - GAP) / 2.0), snap((y1 - y0 - GAP) / 2.0));
    let t = svg.t;

    for (i, q) in quads.iter().enumerate() {
        let path = format!("$.quadrants[{i}]");
        let q = obj(q, &path, &["title", "items"])?;
        let title = req_str(q, "title", &path, 40)?;
        let items = match q.get("items") {
            None => Vec::new(),
            Some(_) => arr(q, "items", &path, 1, 4)?.iter().enumerate()
                .map(|(j, v)| str_items(v, &format!("{path}.items[{j}]"), 80)).collect::<Result<_>>()?,
        };
        let cell = Rect { x: x0 + (i % 2) as f64 * (cw + GAP), y: y0 + (i / 2) as f64 * (ch + GAP), w: cw, h: ch };
        let on = highlight == Some(i);
        svg.node(cell, &t.card, on.then_some((t.accent.as_str(), 2.0)), 8.0);
        let inner = cw - 2.0 * INSET;
        let title = fit(&title, 20.0, inner, 2)?;
        let (items, gap) = fit_items(&items, inner - 16.0, ch - 2.0 * INSET_Y - title.height() - 8.0)
            .map_err(|e| format!("{path}.items: {e}"))?;
        let mut cy = cell.y + INSET_Y;
        cy += svg.text(&title, Anchor::Start, cell.x + INSET, cy, 700, if on { &t.accent } else { &t.text }) + 8.0;
        for l in &items {
            let dot_y = cy + l.line_h() / 2.0;
            svg.dot(cell.x + INSET + 3.0, dot_y, 3.0, if on { &t.accent } else { &t.muted });
            cy += svg.text(l, Anchor::Start, cell.x + INSET + 16.0, cy, 400, &t.text) + gap;
        }
    }

    let (gx1, gy1) = (x0 + 2.0 * cw + GAP, y0 + 2.0 * ch + GAP);
    if let Some(a) = x {
        let ly = gy1 + 8.0;
        svg.line(x0, ly, gx1, ly, &t.border, true);
        let third = (gx1 - x0) / 3.0 - 8.0;
        for (s, anchor, ax) in [(&a.low, Anchor::Start, x0), (&a.label, Anchor::Middle, (x0 + gx1) / 2.0), (&a.high, Anchor::End, gx1)] {
            if let Some(s) = s {
                let l = fit(s, 14.0, third, 1).map_err(|e| format!("$.x: {e}"))?;
                svg.text(&l, anchor, ax, ly + 6.0, if anchor == Anchor::Middle { 700 } else { 400 }, &t.muted);
            }
        }
    }
    if let Some(a) = y {
        let lx = x0 - 16.0;
        svg.line(lx, gy1, lx, y0, &t.border, true);
        let (gw, ex) = (x0 - PAD - 32.0, lx - 12.0);
        if let Some(s) = &a.high {
            let l = fit(s, 14.0, gw, 2).map_err(|e| format!("$.y: {e}"))?;
            svg.text(&l, Anchor::End, ex, y0, 400, &t.muted);
        }
        if let Some(s) = &a.label {
            let l = fit(s, 14.0, gw, 2).map_err(|e| format!("$.y: {e}"))?;
            svg.text(&l, Anchor::End, ex, (y0 + gy1 - l.height()) / 2.0, 700, &t.muted);
        }
        if let Some(s) = &a.low {
            let l = fit(s, 14.0, gw, 2).map_err(|e| format!("$.y: {e}"))?;
            svg.text(&l, Anchor::End, ex, gy1 - l.height(), 400, &t.muted);
        }
    }
    Ok(())
}

/// Fit bullet items into `max_h`: largest size first, then tighter item spacing, all items together.
fn fit_items(items: &[String], w: f64, max_h: f64) -> Result<(Vec<Label>, f64)> {
    for size in [16.0, 14.0, 12.0] {
        let labels: Vec<Label> = items.iter().map(|s| fit(s, size, w, 2)).collect::<Result<_>>()?;
        let text_h: f64 = labels.iter().map(Label::height).sum();
        let gaps = labels.len().saturating_sub(1) as f64;
        if let Some(gap) = [8.0, 4.0, 2.0].into_iter().find(|g| text_h + g * gaps <= max_h) {
            return Ok((labels, gap));
        }
    }
    Err("items do not fit in the quadrant; shorten or remove some".into())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    fn swot() -> serde_json::Value {
        json!({"quadrants":[
            {"title":"강점","items":["가상 브랜드 인지도 높음","운영 비용 낮음"]},
            {"title":"약점","items":["채널 다양성 부족"]},
            {"title":"기회","items":["신규 시장 확대 가능성","파트너십 제안 증가","규제 완화 예상"]},
            {"title":"위협","items":["경쟁사 가격 인하"]}]})
    }

    #[test]
    fn swot_renders_and_is_deterministic() {
        let a = super::super::render("quadrant", &swot(), None).unwrap();
        assert_eq!(a, super::super::render("quadrant", &swot(), None).unwrap());
        assert!(a.contains("강점") && a.starts_with("<svg"));
    }

    #[test]
    fn axes_and_highlight_render() {
        let mut d = swot();
        d["x"] = json!({"label":"Effort","low":"low","high":"high"});
        d["y"] = json!({"label":"Impact","low":"low","high":"high"});
        d["highlight"] = json!(1);
        let s = super::super::render("quadrant", &d, Some("night")).unwrap();
        assert!(s.contains("#F59E0B") && s.contains("Impact"));
    }

    #[test]
    fn max_content_fits_or_errors_never_overlaps() {
        // Longest legal items in every quadrant at every limit: render() runs the layout check.
        let long = "가나다라마바사아자차카타파하 ".repeat(5).chars().take(80).collect::<String>();
        let q = json!({"title": "제목 ".repeat(8).trim(), "items": [long, long, long, long]});
        let d = json!({"x":{"label":"x","low":"l","high":"h"},"y":{"label":"y","low":"l","high":"h"},"quadrants":[q,q,q,q]});
        match super::super::render("quadrant", &d, None) {
            Ok(_) => {}
            Err(e) => assert!(e.contains("do not fit") || e.contains("does not fit"), "{e}"),
        }
    }

    #[test]
    fn four_short_items_fit_with_and_without_axes() {
        let q = json!({"title":"강점","items":["a","b","c","d"]});
        let mut d = json!({"quadrants":[q,q,q,q]});
        super::super::render("quadrant", &d, None).unwrap();
        d["x"] = json!({"label":"노력","low":"낮음","high":"높음"});
        d["y"] = json!({"label":"영향","low":"낮음","high":"높음"});
        let s = super::super::render("quadrant", &d, None).unwrap();
        assert!(!s.contains(r#"font-size="12""#), "4 short items should stay >=14px with axes");
    }

    #[test]
    fn item_counts_and_lengths_never_break_layout() {
        for axes in [false, true] {
            for n in 1..=4 {
                for len in 1..=80 {
                    let item: String = "가나다 라마 바사아".chars().cycle().take(len).collect();
                    let q = json!({"title":"제목","items":vec![item; n]});
                    let mut d = json!({"quadrants":[q,q,q,q]});
                    if axes {
                        d["x"] = json!({"label":"x","low":"l","high":"h"});
                        d["y"] = json!({"label":"y","low":"l","high":"h"});
                    }
                    if let Err(e) = super::super::render("quadrant", &d, None) {
                        assert!(!e.starts_with("layout:"), "n={n} len={len} axes={axes}: {e}");
                    }
                }
            }
        }
    }

    #[test]
    fn strict_fields() {
        let mut d = swot();
        d["quadrants"][0]["color"] = json!("red");
        assert_eq!(super::super::render("quadrant", &d, None).unwrap_err(), "$.quadrants[0].color: unknown field");
        assert!(super::super::render("quadrant", &json!({"quadrants":[]}), None).unwrap_err().contains("4-4"));
        assert!(super::super::render("quadrant", &json!({"quadrants":swot()["quadrants"],"highlight":4}), None).is_err());
    }
}
