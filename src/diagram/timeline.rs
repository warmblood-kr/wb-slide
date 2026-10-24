//! Timeline: evenly spaced events on a horizontal axis; date above, label + note below.
use super::*;

struct Event {
    date: String,
    label: String,
    note: Option<String>,
}

pub fn draw(data: &Value, svg: &mut Svg) -> Result<()> {
    let m = obj(data, "$", &["events", "highlight"])?;
    let events = arr(m, "events", "$", 2, 8)?.iter().enumerate().map(|(i, e)| {
        let path = format!("$.events[{i}]");
        let e = obj(e, &path, &["date", "label", "note"])?;
        Ok(Event { date: req_str(e, "date", &path, 40)?, label: req_str(e, "label", &path, 80)?, note: opt_str(e, "note", &path, 80)? })
    }).collect::<Result<Vec<_>>>()?;
    let n = events.len();
    let highlight = match m.get("highlight") {
        None => None,
        Some(v) => Some(v.as_u64().filter(|h| (*h as usize) < n).ok_or(format!("$.highlight: expected 0-{}", n - 1))? as usize),
    };

    // 4+ events alternate above/below the axis, so each text block spans two steps.
    let alt = n >= 4;
    let k = if alt { 2.0 } else { 1.0 };
    let gaps = (n - 1) as f64;
    let tw = ((k * (W - 2.0 * PAD) - 16.0 * gaps) / (gaps + k)).min(240.0).floor();
    let step = (W - 2.0 * PAD - tw) / gaps;
    let dates = events.iter().map(|e| fit(&e.date, 18.0, tw, 2)).collect::<Result<Vec<_>>>()?;
    let date_h = dates.iter().map(Label::height).fold(0.0, f64::max);
    // Vertical budget for label + note: one side minus its date (alt), or all below the dates.
    let room = if alt { (H - 2.0 * PAD - 48.0) / 2.0 - date_h - 8.0 } else { H - 2.0 * PAD - date_h - 40.0 };
    let below = fit_below(&events, tw, room)?;
    let block = |i: usize| {
        let (l, nt) = &below[i];
        l.height() + nt.as_ref().map_or(0.0, |x| 4.0 + x.height())
    };
    let (up, down) = (0..n).fold((0.0f64, 0.0f64), |(u, d), i| match (alt, i % 2) {
        (false, _) => (date_h, d.max(block(i))),
        (true, 0) => (u.max(dates[i].height() + 8.0 + block(i)), d),
        _ => (u, d.max(dates[i].height() + 8.0 + block(i))),
    });
    let gap = if alt { 24.0 } else { 20.0 };
    let axis = ((PAD + (H - 2.0 * PAD - up - down - 2.0 * gap) / 2.0 + up + gap) / 8.0).round() * 8.0;
    let t = svg.t;

    svg.line(PAD, axis, W - PAD, axis, &t.border, false);
    for (i, (date, (label, note))) in dates.iter().zip(&below).enumerate() {
        let cx = PAD + tw / 2.0 + step * i as f64;
        let on = highlight == Some(i);
        let color = if on { &t.accent } else { &t.muted };
        svg.dot(cx, axis, if on { 8.0 } else { 5.0 }, color);
        let y = if !alt {
            svg.text(date, Anchor::Middle, cx, axis - gap - date.height(), 700, color);
            axis + gap
        } else {
            let top = if i % 2 == 0 { axis - gap - (date.height() + 8.0 + block(i)) } else { axis + gap };
            svg.line(cx, if i % 2 == 0 { top + date.height() + 8.0 + block(i) + 4.0 } else { axis + 10.0 },
                cx, if i % 2 == 0 { axis - 10.0 } else { top - 4.0 }, &t.border, false);
            top + svg.text(date, Anchor::Middle, cx, top, 700, color) + 8.0
        };
        let y = y + svg.text(label, Anchor::Middle, cx, y, 600, &t.text);
        if let Some(note) = note {
            svg.text(note, Anchor::Middle, cx, y + 4.0, 400, &t.muted);
        }
    }
    Ok(())
}

/// Fit every label (and note, 2px smaller) into `w`, shrinking all together until the tallest fits.
fn fit_below(events: &[Event], w: f64, max_h: f64) -> Result<Vec<(Label, Option<Label>)>> {
    let mut size = 20.0;
    loop {
        let out = events.iter().map(|e| {
            Ok((fit(&e.label, size, w, 6)?, e.note.as_ref().map(|s| fit(s, (size - 2.0).max(12.0), w, 6)).transpose()?))
        }).collect::<Result<Vec<_>>>()?;
        let h = out.iter().map(|(l, nt)| l.height() + nt.as_ref().map_or(0.0, |x| 4.0 + x.height())).fold(0.0, f64::max);
        if h <= max_h {
            return Ok(out);
        }
        if size <= 12.0 {
            return Err("timeline labels and notes do not fit; shorten them".into());
        }
        size -= 2.0;
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    fn render(d: &serde_json::Value, theme: Option<&str>) -> Result<String, String> {
        super::super::render("timeline", d, theme)
    }

    fn sample() -> serde_json::Value {
        json!({"highlight":2,"events":[
            {"date":"2025 1분기","label":"가상 서비스 기획","note":"요구사항 정리"},
            {"date":"2025 2분기","label":"베타 출시"},
            {"date":"2025 3분기","label":"정식 출시","note":"주요 파트너 3곳 연동"},
            {"date":"2026 1분기","label":"해외 확장","note":"일본 시장 진출"}]})
    }

    #[test]
    fn renders_deterministically_in_both_themes() {
        let a = render(&sample(), None).unwrap();
        assert_eq!(a, render(&sample(), None).unwrap());
        assert!(a.contains("정식 출시") && a.contains("#C2410C"));
        assert!(render(&sample(), Some("night")).unwrap().contains("#F59E0B"));
    }

    #[test]
    fn min_and_max_counts_with_long_text_fit_or_error() {
        let long = "가나다라마바사아자차카타파하 ".repeat(6).chars().take(80).collect::<String>();
        let date = "가나다라 ".repeat(8).chars().take(40).collect::<String>();
        for n in [2, 8] {
            let e = json!({"date": date, "label": long, "note": long});
            let short = json!({"date": "2026.01", "label": "가상 이벤트 이름", "note": "짧은 메모"});
            assert!(render(&json!({"events": vec![short; n]}), None).is_ok());
            match render(&json!({"events": vec![e; n], "highlight": n - 1}), None) {
                Ok(_) => {}
                Err(e) => assert!(e.contains("not fit"), "{e}"),
            }
        }
    }

    #[test]
    fn strict_fields() {
        let mut d = sample();
        d["events"][1]["color"] = json!("red");
        assert_eq!(render(&d, None).unwrap_err(), "$.events[1].color: unknown field");
        assert!(render(&json!({"events":[{"date":"a","label":"b"}]}), None).unwrap_err().contains("2-8"));
        let mut d = sample();
        d["highlight"] = json!(4);
        assert_eq!(render(&d, None).unwrap_err(), "$.highlight: expected 0-3");
        d["highlight"] = json!(0);
        d["events"][0]["label"] = json!("가".repeat(81));
        assert!(render(&d, None).unwrap_err().starts_with("$.events[0].label"));
    }
}
