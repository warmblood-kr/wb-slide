//! Fishbone (Ishikawa): spine into the effect box on the right; category bones
//! alternate top/bottom, items stacked as text left of each bone.
use super::*;

const DX: f64 = 48.0; // horizontal lean of a bone, head (category) to foot (spine)
const EFFECT_W: f64 = 136.0;

pub fn draw(data: &Value, svg: &mut Svg) -> Result<()> {
    let m = obj(data, "$", &["effect", "causes"])?;
    let effect = req_str(m, "effect", "$", 80)?;
    let causes = arr(m, "causes", "$", 2, 6)?.iter().enumerate().map(|(i, c)| {
        let path = format!("$.causes[{i}]");
        let c = obj(c, &path, &["category", "items"])?;
        let items = match c.get("items") {
            None => Vec::new(),
            Some(_) => arr(c, "items", &path, 0, 4)?.iter().enumerate()
                .map(|(j, v)| str_items(v, &format!("{path}.items[{j}]"), 80)).collect::<Result<_>>()?,
        };
        Ok((req_str(c, "category", &path, 40)?, items))
    }).collect::<Result<Vec<_>>>()?;

    // Slots per side; slot width sw; items get sw - 24 so they clear the neighbouring bone.
    let slots = causes.len().div_ceil(2) as f64;
    let sw = ((W - 2.0 * PAD - DX - EFFECT_W + 16.0) / slots).min(280.0).floor();
    let iw = sw - 24.0;
    let total = slots * sw - 16.0 + DX + EFFECT_W;
    let left = PAD + ((W - 2.0 * PAD - total) / 2.0 / 8.0).floor() * 8.0;
    let foot = |j: f64| left + iw + 8.0 + DX + j * sw;
    let ex = foot(slots - 1.0) + 24.0;
    let spine = H / 2.0;
    let t = svg.t;

    let cats = causes.iter().map(|(c, _)| fit(c, 16.0, sw - 48.0, 2)).collect::<Result<Vec<_>>>()?;
    let bh = cats.iter().map(Label::height).fold(0.0, f64::max) + 16.0;
    let region = spine - PAD - bh - 20.0; // item space between a category box and the spine

    let eff = fit(&effect, 20.0, EFFECT_W - 32.0, 4)?;
    let er = Rect { x: ex, y: spine - (eff.height() + 32.0) / 2.0, w: EFFECT_W, h: eff.height() + 32.0 };
    svg.line(left, spine, ex, spine, &t.muted, true);
    svg.node(er, &t.accent, None, 8.0);
    svg.text(&eff, Anchor::Middle, ex + EFFECT_W / 2.0, er.y + 16.0, 700, &t.bg);

    for (i, ((_, items), cat)) in causes.iter().zip(&cats).enumerate() {
        let (fx, top) = (foot((i / 2) as f64), i % 2 == 0);
        let hx = fx - DX;
        let by = if top { PAD } else { H - PAD - bh };
        let head_y = if top { by + bh } else { by };
        let bw = cat.width() + 32.0;
        svg.node(Rect { x: hx - bw / 2.0, y: by, w: bw, h: bh }, &t.card, Some((&t.border, 1.0)), 6.0);
        svg.text(cat, Anchor::Middle, hx, by + (bh - cat.height()) / 2.0, 700, &t.text);
        svg.line(hx, head_y, fx, spine, &t.border, false);

        let labels = fit_items(items, iw, region).map_err(|e| format!("$.causes[{i}].items: {e}"))?;
        // Bone x at height y; an item's right edge clears the bone at both its top and bottom.
        let bone_x = |y: f64| hx + DX * (y - head_y) / (spine - head_y);
        // Items hug the category end on both sides, so top and bottom read as mirror images.
        let stack: f64 = labels.iter().map(|l| l.height() + 6.0).sum::<f64>() - 6.0;
        let mut y = if top { head_y + 12.0 } else { head_y - 12.0 - stack };
        for l in &labels {
            let right = bone_x(y).min(bone_x(y + l.height())) - 8.0;
            y += svg.text(l, Anchor::End, right, y, 400, &t.muted) + 6.0;
        }
    }
    Ok(())
}

/// Fit items into `max_h`, shrinking all of a bone's items together.
fn fit_items(items: &[String], w: f64, max_h: f64) -> Result<Vec<Label>> {
    let mut size = 14.0;
    loop {
        let labels: Vec<Label> = items.iter().map(|s| fit(s, size, w, 3)).collect::<Result<_>>()?;
        if labels.iter().map(|l| l.height() + 6.0).sum::<f64>() <= max_h {
            return Ok(labels);
        }
        if size <= 12.0 {
            return Err("items do not fit along the bone; shorten or remove some".into());
        }
        size -= 2.0;
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    fn render(d: &serde_json::Value, theme: Option<&str>) -> Result<String, String> {
        super::super::render("fishbone", d, theme)
    }

    fn sample() -> serde_json::Value {
        json!({"effect":"가상 앱 이탈률 증가","causes":[
            {"category":"사람","items":["신규 상담 인력 부족","교육 자료 노후화"]},
            {"category":"프로세스","items":["가입 단계 과다","환불 처리 지연","문의 응답 지연"]},
            {"category":"기술","items":["초기 로딩 지연","결제 오류 빈발"]},
            {"category":"환경","items":["경쟁 앱 프로모션"]}]})
    }

    #[test]
    fn renders_deterministically_in_both_themes() {
        let a = render(&sample(), None).unwrap();
        assert_eq!(a, render(&sample(), None).unwrap());
        assert!(a.contains("프로세스") && a.contains("#2563EB"));
        assert!(render(&sample(), Some("night")).unwrap().contains("#F59E0B"));
    }

    #[test]
    fn min_and_max_counts_fit_or_error() {
        let short = json!({"category":"가상 분류","items":["짧은 원인 항목 하나","짧은 원인 둘","원인 셋","원인 넷"]});
        for n in [2, 6] {
            assert!(render(&json!({"effect":"결과","causes": vec![short.clone(); n]}), None).is_ok(), "{n}");
        }
        let long = "가나다라마바사아자차카타파하 ".repeat(6).chars().take(80).collect::<String>();
        let latin = "the quick brown fox jumps over ".repeat(3).chars().take(80).collect::<String>();
        for item in [long.clone(), latin] {
            let c = json!({"category": "가나다라 ".repeat(8).chars().take(40).collect::<String>(), "items": [item, item, item, item]});
            for n in [2, 6] {
                match render(&json!({"effect": long, "causes": vec![c.clone(); n]}), None) {
                    Ok(_) => {}
                    Err(e) => assert!(e.contains("not fit"), "{e}"),
                }
            }
        }
    }

    #[test]
    fn empty_items_render_as_bare_category() {
        let mut d = sample();
        d["causes"][0]["items"] = json!([]);
        assert!(render(&d, None).is_ok());
    }

    #[test]
    fn strict_fields() {
        let mut d = sample();
        d["causes"][0]["weight"] = json!(1);
        assert_eq!(render(&d, None).unwrap_err(), "$.causes[0].weight: unknown field");
        let mut d = sample();
        d["causes"][1]["items"] = json!(["a", "b", "c", "d", "e"]);
        assert!(render(&d, None).unwrap_err().starts_with("$.causes[1].items"));
        assert!(render(&json!({"effect":"x","causes":[]}), None).unwrap_err().contains("2-6"));
        assert_eq!(render(&json!({"causes":sample()["causes"]}), None).unwrap_err(), "$.effect: required");
    }
}
