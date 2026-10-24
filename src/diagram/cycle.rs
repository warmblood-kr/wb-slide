//! Cycle: 3-8 steps on an ellipse clockwise from 12 o'clock, each arrowed to the next, last back to first.
use super::*;

const NW: f64 = 144.0;
const INSET: f64 = 12.0;

pub fn draw(data: &Value, svg: &mut Svg) -> Result<()> {
    let m = obj(data, "$", &["steps"])?;
    let steps = arr(m, "steps", "$", 3, 8)?.iter().enumerate()
        .map(|(i, v)| {
            let path = format!("$.steps[{i}]");
            let s = obj(v, &path, &["label", "note"])?;
            let inner = NW - 2.0 * INSET;
            let label = fit(&req_str(s, "label", &path, 80)?, 16.0, inner, 2).map_err(|e| format!("{path}.label: {e}"))?;
            let note = match opt_str(s, "note", &path, 80)? {
                Some(n) => Some(fit(&n, 14.0, inner, 3).map_err(|e| format!("{path}.note: {e}"))?),
                None => None,
            };
            Ok((label, note))
        })
        .collect::<Result<Vec<_>>>()?;
    let content = |(l, n): &(Label, Option<Label>)| l.height() + n.as_ref().map_or(0.0, |n| 4.0 + n.height());
    let nh = up8(steps.iter().map(content).fold(0.0, f64::max) + 16.0).max(64.0); // 8px top/bottom: 8 steps need nh <= 80
    let mut boxes = ring(steps.len(), NW, nh);
    // Odd counts leave the bottom empty: centre the ring's bounding box vertically.
    let (top, bottom) = boxes.iter().fold((H, 0.0f64), |(t, b), r| (t.min(r.y), b.max(r.y + r.h)));
    let dy = ((H - top - bottom) / 2.0 / 8.0).round() * 8.0;
    boxes.iter_mut().for_each(|r| r.y += dy);
    if boxes.iter().enumerate().any(|(i, a)| boxes[i + 1..].iter().any(|b| a.overlaps(b))) {
        return Err("steps do not fit around the cycle; shorten labels or notes, or use fewer steps".into());
    }
    let t = svg.t;
    for (i, a) in boxes.iter().enumerate() {
        svg.link(*a, boxes[(i + 1) % boxes.len()], &t.muted, true);
    }
    for (i, (r, s)) in boxes.iter().zip(&steps).enumerate() {
        let stroke = if i == 0 { (t.accent.as_str(), 2.0) } else { (t.border.as_str(), 1.0) };
        svg.node(*r, &t.card, Some(stroke), 8.0);
        let (cx, mut y) = (r.x + r.w / 2.0, r.y + (r.h - content(s)) / 2.0);
        y += svg.text(&s.0, Anchor::Middle, cx, y, 700, &t.text) + 4.0;
        if let Some(n) = &s.1 {
            svg.text(n, Anchor::Middle, cx, y, 400, &t.muted);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    fn sample() -> serde_json::Value {
        json!({"steps":[{"label":"계획","note":"목표와 지표 정의"},{"label":"실행","note":"작은 단위로 배포"},
            {"label":"측정","note":"사용 데이터 수집"},{"label":"학습","note":"가설 검증 후 다음 계획에 반영"}]})
    }

    fn render(d: &serde_json::Value) -> Result<String, String> {
        super::super::render("cycle", d, None)
    }

    #[test]
    fn renders_deterministically() {
        let a = render(&sample()).unwrap();
        assert_eq!(a, render(&sample()).unwrap());
        assert!(a.contains("측정") && a.matches("marker-end").count() == 4);
    }

    #[test]
    fn min_and_max_steps_with_long_text() {
        let long = "가나다라마바 ".repeat(12).chars().take(80).collect::<String>();
        for n in [3, 8] {
            assert!(render(&json!({"steps": vec![json!({"label":"가상 단계","note":"짧은 설명 문구"}); n]})).is_ok());
            for step in [json!({"label": long}), json!({"label": "단계", "note": long}), json!({"label": long, "note": long})] {
                match render(&json!({"steps": vec![step; n]})) {
                    Ok(_) => {}
                    Err(e) => assert!(e.contains("not fit"), "{e}"),
                }
            }
        }
    }

    #[test]
    fn fuzz_counts_and_lengths_never_layout_error() {
        for n in 3..=8 {
            for len in 1..=80 {
                let spaced: String = format!("가{}", "나다 라마바사 아자".repeat(9)).chars().take(len).collect();
                for s in [spaced.trim().to_string(), "가".repeat(len)] {
                    for st in [json!({"label": &s}), json!({"label": "단계", "note": &s}), json!({"label": &s, "note": &s})] {
                        if let Err(e) = render(&json!({"steps": vec![st; n]})) {
                            assert!(!e.starts_with("layout:"), "n={n} len={len}: {e}");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn strict_fields() {
        let mut d = sample();
        d["steps"][1]["icon"] = json!("x");
        assert_eq!(render(&d).unwrap_err(), "$.steps[1].icon: unknown field");
        assert!(render(&json!({"steps":[{"label":"a"},{"label":"b"}]})).unwrap_err().contains("3-8"));
        assert!(render(&json!({"steps":[{"label":"a"},{"label":"b"},{"note":"c"}]})).unwrap_err().contains("$.steps[2].label"));
    }

    #[test]
    fn night_theme_renders() {
        assert!(super::super::render("cycle", &sample(), Some("night")).unwrap().contains("#F59E0B"));
    }
}
