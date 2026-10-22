//! Hub and spoke (star topology): a central node, 3-8 spokes on an ellipse clockwise from 12 o'clock.
use super::*;

const SW: f64 = 136.0;
const HW: f64 = 144.0;
const INSET: f64 = 16.0;

pub fn draw(data: &Value, svg: &mut Svg) -> Result<()> {
    let m = obj(data, "$", &["hub", "spokes", "arrows"])?;
    let hub = req_str(m, "hub", "$", 80)?;
    let arrows = match m.get("arrows").map(Value::as_str) {
        None | Some(Some("none")) => None,
        Some(Some("out")) => Some(true),
        Some(Some("in")) => Some(false),
        _ => return Err(r#"$.arrows: expected "none", "out" or "in""#.into()),
    };
    let labels: Vec<Label> = arr(m, "spokes", "$", 3, 8)?.iter().enumerate()
        .map(|(i, v)| {
            let path = format!("$.spokes[{i}]");
            fit(&str_items(v, &path, 80)?, 16.0, SW - 2.0 * INSET, 3).map_err(|e| format!("{path}: {e}"))
        })
        .collect::<Result<_>>()?;
    let hl = fit(&hub, 20.0, HW - 2.0 * INSET, 3).map_err(|e| format!("$.hub: {e}"))?;

    let sh = up8(labels.iter().map(Label::height).fold(0.0, f64::max) + 2.0 * INSET).max(64.0);
    let hh = up8(hl.height() + 2.0 * INSET).max(80.0);
    let hub_r = Rect { x: (W - HW) / 2.0, y: up8((H - hh) / 2.0), w: HW, h: hh };
    let spokes = ring(labels.len(), SW, sh);
    let t = svg.t;

    for r in &spokes {
        match arrows {
            None => svg.link(hub_r, *r, &t.border, false),
            Some(true) => svg.link(hub_r, *r, &t.muted, true),
            Some(false) => svg.link(*r, hub_r, &t.muted, true),
        }
    }
    svg.node(hub_r, &t.accent, None, 12.0);
    svg.text(&hl, Anchor::Middle, W / 2.0, hub_r.y + (hh - hl.height()) / 2.0, 700, &t.bg);
    for (r, l) in spokes.iter().zip(&labels) {
        svg.node(*r, &t.card, Some((t.border.as_str(), 1.0)), 8.0);
        svg.text(l, Anchor::Middle, r.x + r.w / 2.0, r.y + (r.h - l.height()) / 2.0, 600, &t.text);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    fn sample() -> serde_json::Value {
        json!({"hub":"고객 데이터 플랫폼","spokes":["마케팅","영업","고객 지원","제품 분석","재무"],"arrows":"out"})
    }

    fn render(d: &serde_json::Value) -> Result<String, String> {
        super::super::render("hub", d, None)
    }

    #[test]
    fn renders_deterministically() {
        let a = render(&sample()).unwrap();
        assert_eq!(a, render(&sample()).unwrap());
        assert!(a.contains("고객 지원") && a.contains("marker-end"));
    }

    #[test]
    fn min_and_max_spokes_with_long_labels() {
        let long = "가나다라마바 ".repeat(12).chars().take(80).collect::<String>();
        for n in [3, 8] {
            for s in ["가상 팀", long.as_str()] {
                let d = json!({"hub": s, "spokes": vec![s; n], "arrows": "in"});
                match render(&d) {
                    Ok(_) => {}
                    Err(e) => assert!(e.contains("does not fit"), "{e}"),
                }
            }
            assert!(render(&json!({"hub":"허브","spokes":vec!["가상 서비스 이름"; n]})).is_ok());
        }
    }

    #[test]
    fn strict_fields() {
        let mut d = sample();
        d["color"] = json!("red");
        assert_eq!(render(&d).unwrap_err(), "$.color: unknown field");
        assert!(render(&json!({"hub":"h","spokes":["a","b"]})).unwrap_err().contains("3-8"));
        assert!(render(&json!({"hub":"h","spokes":["a","b","c"],"arrows":"both"})).unwrap_err().starts_with("$.arrows"));
        assert!(render(&json!({"spokes":["a","b","c"]})).unwrap_err().contains("$.hub"));
    }

    #[test]
    fn night_theme_renders() {
        assert!(super::super::render("hub", &sample(), Some("night")).unwrap().contains("#F59E0B"));
    }
}
