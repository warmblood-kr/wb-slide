//! Pyramid: a triangle cut into horizontal bands, top level first. Each note sits just
//! right of its band, following the slope; a label too wide for its band moves there too.
use super::*;

const GAP: f64 = 8.0;
const BASE: f64 = 352.0;
const COL_GAP: f64 = 32.0;

/// Widest text box centred in (or else hugging the bottom of) the band, as (box, label).
fn inside(s: &str, yt: f64, yb: f64, wid: impl Fn(f64) -> f64) -> Option<(Rect, Label)> {
    for lines in 1..=2 {
        let rh = lines as f64 * 23.0 + 16.0;
        for top in [(yt + yb - rh) / 2.0, yb - rh] {
            let rw = wid(top) - 16.0;
            if top < yt || rw < 48.0 {
                continue;
            }
            if let Ok(l) = fit(s, 18.0, rw - 24.0, lines) {
                if l.size >= 14.0 {
                    return Some((Rect { x: -rw / 2.0, y: top, w: rw, h: rh }, l));
                }
            }
        }
    }
    None
}

pub fn draw(data: &Value, svg: &mut Svg) -> Result<()> {
    let m = obj(data, "$", &["levels"])?;
    let levels = arr(m, "levels", "$", 2, 6)?;
    let k = levels.len();
    let (y0, ht) = (PAD, H - 2.0 * PAD);
    let h = (ht - GAP * (k - 1) as f64) / k as f64;
    let wid = |y: f64| BASE * (y - y0) / ht;
    let t = svg.t;

    // Pass 1: fit every label (inside its band, else into the right column) and note.
    let mut rows = Vec::new();
    for (i, v) in levels.iter().enumerate() {
        let path = format!("$.levels[{i}]");
        let v = obj(v, &path, &["label", "note"])?;
        let label = req_str(v, "label", &path, 80)?;
        let note = opt_str(v, "note", &path, 80)?;
        let yt = y0 + i as f64 * (h + GAP);
        let fitted = inside(&label, yt, yt + h, wid);
        // Text starts COL_GAP right of the band's bottom corner; room assumes the pyramid hugs PAD.
        let col_w = W - 2.0 * PAD - BASE / 2.0 - wid(yt + h) / 2.0 - COL_GAP;
        let out = match fitted {
            None => Some(fit(&label, 16.0, col_w, 2).map_err(|e| format!("{path}.label: {e}"))?),
            Some(_) => None,
        };
        let lh = out.as_ref().map_or(0.0, |l| l.height() + 4.0);
        let note = match note {
            Some(s) => {
                let lines = ((h - lh) / 18.0).floor().min(3.0) as usize;
                if lines == 0 {
                    return Err(format!("{path}: label and note do not fit beside the band; shorten"));
                }
                Some(fit(&s, 14.0, col_w, lines).map_err(|e| format!("{path}.note: {e}"))?)
            }
            None => None,
        };
        rows.push((yt, fitted, out, note));
    }
    // Pass 2: centre the pyramid plus its notes (as wide as their text) on the canvas.
    let right = rows.iter().map(|r| {
        let tw = [&r.2, &r.3].into_iter().flatten().map(|l| l.width()).fold(0.0, f64::max);
        if tw > 0.0 { wid(r.0 + h) / 2.0 + COL_GAP + tw } else { BASE / 2.0 }
    }).fold(BASE / 2.0, f64::max);
    let cx = ((W - BASE / 2.0 - right) / 2.0 / 8.0).round() * 8.0 + BASE / 2.0;
    for (i, (yt, fitted, out, note)) in rows.into_iter().enumerate() {
        let yb = yt + h;
        let (fill, ink) = if i == 0 { (&t.accent, &t.bg) } else { (&t.card, &t.text) };
        let (a, b) = (wid(yt) / 2.0, wid(yb) / 2.0);
        let inner = fitted.as_ref().map(|(r, _)| Rect { x: r.x + cx, ..*r });
        svg.polygon(&[(cx - a, yt), (cx + a, yt), (cx + b, yb), (cx - b, yb)], fill, (i != 0).then_some(t.border.as_str()), inner);
        if let Some((r, l)) = &fitted {
            svg.text(l, Anchor::Middle, cx, r.y + (r.h - l.height()) / 2.0, 700, ink);
        }
        if out.is_none() && note.is_none() {
            continue;
        }
        // Right column block: the label (if it did not fit inside), then the muted note.
        let both = if out.is_some() && note.is_some() { 4.0 } else { 0.0 };
        let block = [&out, &note].iter().filter_map(|l| l.as_ref()).map(|l| l.height()).sum::<f64>() + both;
        let (cy, col_x) = (yt + h / 2.0, cx + b + COL_GAP);
        svg.line(cx + wid(cy) / 2.0 + 8.0, cy, col_x - 8.0, cy, &t.border, false);
        let mut ty = cy - block / 2.0;
        if let Some(l) = &out {
            ty += svg.text(l, Anchor::Start, col_x, ty, 700, &t.text) + both;
        }
        if let Some(l) = &note {
            svg.text(l, Anchor::Start, col_x, ty, 400, &t.muted);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    fn render(d: &serde_json::Value, theme: Option<&str>) -> super::Result<String> {
        super::super::render("pyramid", d, theme)
    }

    fn sample() -> serde_json::Value {
        json!({"levels":[
            {"label":"비전","note":"가상 시장에서 신뢰받는 1위 브랜드"},
            {"label":"전략 목표","note":"3년 내 핵심 지표 두 배 성장"},
            {"label":"핵심 과제","note":"제품 품질, 고객 경험, 운영 효율"},
            {"label":"실행 계획","note":"분기별 로드맵과 담당 조직 지정"}]})
    }

    #[test]
    fn sample_renders_and_is_deterministic() {
        let a = render(&sample(), None).unwrap();
        assert_eq!(a, render(&sample(), None).unwrap());
        assert!(a.contains("실행 계획") && a.contains("<polygon"));
        assert!(render(&sample(), Some("night")).unwrap().contains("#F59E0B"));
    }

    #[test]
    fn narrow_top_label_moves_outside() {
        let d = json!({"levels":[{"label":"아주 긴 최상위 목표 이름"},{"label":"기반"},{"label":"토대"},{"label":"바닥"},{"label":"뿌리"},{"label":"땅"}]});
        let s = render(&d, None).unwrap();
        assert!(s.contains("<line"), "outside label gets a connector");
    }

    #[test]
    fn min_and_max_counts_with_long_labels() {
        let long: String = "가나다라마바사아자차카타파하 ".repeat(6).chars().take(80).collect();
        for k in [2, 6] {
            let lv: Vec<_> = (0..k).map(|_| json!({"label": long, "note": long})).collect();
            match render(&json!({ "levels": lv }), None) {
                Ok(_) => {}
                Err(e) => assert!(e.contains("not fit"), "{e}"),
            }
            let lv: Vec<_> = (0..k).map(|i| json!({"label": format!("단계 {i}")})).collect();
            render(&json!({ "levels": lv }), None).unwrap();
        }
    }

    #[test]
    fn strict_fields() {
        let mut d = sample();
        d["levels"][2]["value"] = json!("1");
        assert_eq!(render(&d, None).unwrap_err(), "$.levels[2].value: unknown field");
        assert!(render(&json!({"levels":[{"label":"a"}]}), None).unwrap_err().contains("2-6"));
    }
}
