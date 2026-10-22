//! Funnel: stacked bands narrowing downward (not to scale; widths step down evenly).
use super::*;

const GAP: f64 = 8.0;
const TOP_W: f64 = W - 2.0 * PAD - 48.0; // 544
const BOTTOM_W: f64 = 224.0;

pub fn draw(data: &Value, svg: &mut Svg) -> Result<()> {
    let m = obj(data, "$", &["stages"])?;
    let stages = arr(m, "stages", "$", 2, 6)?;
    let k = stages.len();
    let h = (((H - 2.0 * PAD - GAP * (k - 1) as f64) / k as f64 / 8.0).floor() * 8.0).min(80.0);
    let y0 = ((H - h * k as f64 - GAP * (k - 1) as f64) / 2.0).round();
    let step = (TOP_W - BOTTOM_W) / k as f64;
    let (cx, t) = (W / 2.0, svg.t);

    for (i, s) in stages.iter().enumerate() {
        let path = format!("$.stages[{i}]");
        let s = obj(s, &path, &["label", "value"])?;
        let label = req_str(s, "label", &path, 80)?;
        let value = opt_str(s, "value", &path, 40)?;
        let (top_w, bot_w) = (TOP_W - step * i as f64, TOP_W - step * (i + 1) as f64);
        let y = y0 + i as f64 * (h + GAP);
        // The trapezoid is the visual; its inner rectangle (bottom width, full height) is the node.
        let inner = Rect { x: cx - bot_w / 2.0, y, w: bot_w, h };
        let last = i + 1 == k;
        let (fill, ink) = if last { (&t.accent, &t.bg) } else { (&t.card, &t.text) };
        svg.polygon(&[(cx - top_w / 2.0, y), (cx + top_w / 2.0, y), (cx + bot_w / 2.0, y + h), (cx - bot_w / 2.0, y + h)], fill, (!last).then_some(t.border.as_str()), Some(inner));

        let (iw, ih) = (bot_w - 32.0, h - 16.0);
        let vbig = value.as_ref().map(|v| fit(v, 24.0, iw, 1)).transpose().map_err(|e| format!("{path}.value: {e}"))?;
        let vh = vbig.as_ref().map_or(0.0, |v| v.height() + 4.0);
        let mut l = fit(&label, 16.0, iw, 2).map_err(|e| format!("{path}.label: {e}"))?;
        if l.height() + vh > ih {
            l = fit(&label, 16.0, iw, 1).map_err(|e| format!("{path}.label: {e}"))?;
        }
        if l.height() + vh > ih {
            // Thin band: label and value side by side (label ends, value starts, at the centre).
            let (Some(v), half) = (&value, iw / 2.0 - 8.0) else {
                return Err(format!("{path}: label does not fit in the band; shorten"));
            };
            let l = fit(&label, 16.0, half, 1).map_err(|e| format!("{path}.label: {e}"))?;
            let v = fit(v, 20.0, half, 1).map_err(|e| format!("{path}.value: {e}"))?;
            if v.height() > h - 8.0 {
                return Err(format!("{path}: label and value do not fit in the band; shorten"));
            }
            svg.text(&l, Anchor::End, cx - 8.0, y + (h - l.height()) / 2.0, 400, ink);
            svg.text(&v, Anchor::Start, cx + 8.0, y + (h - v.height()) / 2.0, 700, ink);
            continue;
        }
        let mut ty = y + (h - l.height() - vh) / 2.0;
        ty += svg.text(&l, Anchor::Middle, cx, ty, 400, ink) + 4.0;
        if let Some(v) = &vbig {
            svg.text(v, Anchor::Middle, cx, ty, 700, ink);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    fn render(d: &serde_json::Value, theme: Option<&str>) -> super::Result<String> {
        super::super::render("funnel", d, theme)
    }

    fn sample() -> serde_json::Value {
        json!({"stages":[
            {"label":"방문","value":"12,400명"},
            {"label":"회원가입","value":"3,100명"},
            {"label":"체험 시작","value":"1,240명"},
            {"label":"유료 전환","value":"310명"}]})
    }

    #[test]
    fn sample_renders_and_is_deterministic() {
        let a = render(&sample(), None).unwrap();
        assert_eq!(a, render(&sample(), None).unwrap());
        assert!(a.contains("유료 전환") && a.contains("<polygon"));
        assert!(render(&sample(), Some("night")).unwrap().contains("#F59E0B"));
    }

    #[test]
    fn min_and_max_counts_with_long_labels() {
        let long: String = "가나다라마바사아자차카타파하 ".repeat(6).chars().take(80).collect();
        let v: String = "1".repeat(40);
        for k in [2, 6] {
            let st: Vec<_> = (0..k).map(|_| json!({"label": long, "value": v})).collect();
            match render(&json!({ "stages": st }), None) {
                Ok(_) => {}
                Err(e) => assert!(e.contains("not fit"), "{e}"),
            }
            let st: Vec<_> = (0..k).map(|i| json!({"label": format!("단계 {i}")})).collect();
            render(&json!({ "stages": st }), None).unwrap();
        }
    }

    #[test]
    fn six_stages_put_value_beside_label() {
        let st: Vec<_> = ["노출", "클릭", "방문", "가입", "체험", "결제"].iter().map(|l| json!({"label": l, "value": "1,200"})).collect();
        assert!(render(&json!({ "stages": st }), None).unwrap().contains("text-anchor=\"end\""));
    }

    #[test]
    fn strict_fields() {
        let mut d = sample();
        d["stages"][1]["color"] = json!("red");
        assert_eq!(render(&d, None).unwrap_err(), "$.stages[1].color: unknown field");
        assert!(render(&json!({"stages":[{"label":"a"}]}), None).unwrap_err().contains("2-6"));
        assert!(render(&json!({"stages":[{"label":"a"},{"value":"1"}]}), None).unwrap_err().contains("$.stages[1].label"));
    }
}
