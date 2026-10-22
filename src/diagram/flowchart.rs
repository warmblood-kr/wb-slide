//! Flowchart: a DAG in ranks (simplified Sugiyama), left-to-right or top-to-bottom.
//! Rank = longest path from a source; edges spanning ranks route through dummy
//! slots; a few barycenter sweeps order each rank. Positions are computed on a
//! (main, cross) axis pair and mapped to (x, y) by the direction.
use super::*;

const GAP: f64 = 24.0; // between slots of one rank (cross axis)
const MIN_GAP: f64 = 16.0; // between ranks (main axis)
const MAX_GAP: f64 = 96.0;

#[derive(Clone, Copy, PartialEq)]
enum Shape {
    Process,
    Decision,
    Terminal,
}

struct Node {
    label: String,
    shape: Shape,
}

struct Link {
    from: usize,
    to: usize,
    label: Option<String>,
}

fn snap(v: f64) -> f64 {
    (v / 8.0).floor() * 8.0
}

fn parse(data: &Value) -> Result<(bool, Vec<Node>, Vec<Link>)> {
    let m = obj(data, "$", &["direction", "nodes", "edges"])?;
    let tb = match opt_str(m, "direction", "$", 2)?.as_deref() {
        None | Some("LR") => false,
        Some("TB") => true,
        _ => return Err("$.direction: expected LR or TB".into()),
    };
    let (mut ids, mut nodes) = (Vec::<String>::new(), Vec::new());
    for (i, v) in arr(m, "nodes", "$", 2, 12)?.iter().enumerate() {
        let p = format!("$.nodes[{i}]");
        let o = obj(v, &p, &["id", "label", "shape"])?;
        let id = req_str(o, "id", &p, 40)?;
        if ids.contains(&id) {
            return Err(format!("{p}.id: duplicate id \"{id}\""));
        }
        let shape = match o.get("shape").map(|s| s.as_str()) {
            None | Some(Some("process")) => Shape::Process,
            Some(Some("decision")) => Shape::Decision,
            Some(Some("terminal")) => Shape::Terminal,
            _ => return Err(format!("{p}.shape: expected process, decision or terminal")),
        };
        nodes.push(Node { label: req_str(o, "label", &p, 80)?, shape });
        ids.push(id);
    }
    let mut links: Vec<Link> = Vec::new();
    for (i, v) in arr(m, "edges", "$", 0, 24)?.iter().enumerate() {
        let p = format!("$.edges[{i}]");
        let o = obj(v, &p, &["from", "to", "label"])?;
        let end = |k: &str| -> Result<usize> {
            let id = req_str(o, k, &p, 40)?;
            ids.iter().position(|x| *x == id).ok_or(format!("{p}.{k}: unknown id \"{id}\""))
        };
        let (from, to) = (end("from")?, end("to")?);
        if from == to {
            return Err(format!("{p}: self-loop on \"{}\"", ids[from]));
        }
        if links.iter().any(|l| l.from == from && l.to == to) {
            return Err(format!("{p}: duplicate edge {} -> {}", ids[from], ids[to]));
        }
        links.push(Link { from, to, label: opt_str(o, "label", &p, 40)? });
    }
    Ok((tb, nodes, links))
}

/// Longest-path rank per node (Kahn's topological order); a leftover node means a cycle.
fn ranks(n: usize, links: &[Link]) -> Result<Vec<usize>> {
    let mut indeg = vec![0; n];
    links.iter().for_each(|l| indeg[l.to] += 1);
    let mut rank = vec![0; n];
    let mut queue: Vec<usize> = (0..n).filter(|&v| indeg[v] == 0).collect();
    let mut seen = 0;
    while let Some(u) = queue.pop() {
        seen += 1;
        for l in links.iter().filter(|l| l.from == u) {
            rank[l.to] = rank[l.to].max(rank[u] + 1);
            indeg[l.to] -= 1;
            if indeg[l.to] == 0 {
                queue.push(l.to);
            }
        }
    }
    if seen < n {
        return Err("$.edges: cycles are not supported; use the cycle type".into());
    }
    Ok(rank)
}

/// Fit a label in a w x h box at `size`, with as many lines as the height allows.
fn fit_box(s: &str, w: f64, h: f64, size: f64) -> Result<Label> {
    let lines = (h / (size * 1.3f64).round()).floor() as usize;
    fit(s, size, w, lines.max(1)).ok().filter(|l| l.size == size && l.height() <= h)
        .ok_or(format!("label \"{s}\" does not fit in {}x{}px; shorten it", w.floor(), h.floor()))
}

pub fn draw(data: &Value, svg: &mut Svg) -> Result<()> {
    let (tb, nodes, links) = parse(data)?;
    let rank = ranks(nodes.len(), &links)?;

    // Items 0..n are nodes, the rest dummy points; each link becomes a chain of items.
    let mut item_rank = rank.clone();
    let chains: Vec<Vec<usize>> = links.iter().map(|l| {
        let mut c = vec![l.from];
        for r in rank[l.from] + 1..rank[l.to] {
            item_rank.push(r);
            c.push(item_rank.len() - 1);
        }
        c.push(l.to);
        c
    }).collect();
    let nr = rank.iter().max().unwrap() + 1;
    let mut layers: Vec<Vec<usize>> = vec![Vec::new(); nr];
    item_rank.iter().enumerate().for_each(|(i, &r)| layers[r].push(i));
    let (mut up, mut down) = (vec![Vec::new(); item_rank.len()], vec![Vec::new(); item_rank.len()]);
    for w in chains.iter().flat_map(|c| c.windows(2)) {
        down[w[0]].push(w[1]);
        up[w[1]].push(w[0]);
    }

    // Barycenter sweeps, alternating down and up; ties keep input order.
    let mut pos = vec![0.0; item_rank.len()];
    for l in &layers {
        l.iter().enumerate().for_each(|(k, &i)| pos[i] = k as f64);
    }
    for sweep in 0..4 {
        let (order, nb): (Vec<usize>, _) = if sweep % 2 == 0 { ((1..nr).collect(), &up) } else { ((0..nr - 1).rev().collect(), &down) };
        for r in order {
            let key: Vec<f64> = (0..item_rank.len()).map(|i| {
                let ns: &Vec<usize> = &nb[i];
                if ns.is_empty() { pos[i] } else { ns.iter().map(|&j| pos[j]).sum::<f64>() / ns.len() as f64 }
            }).collect();
            layers[r].sort_by(|&a, &b| key[a].total_cmp(&key[b]).then(a.cmp(&b)));
            layers[r].iter().enumerate().for_each(|(k, &i)| pos[i] = k as f64);
        }
    }

    // Uniform slot size: m along the main axis, c across it.
    let (main_len, cross_len) = if tb { (H - 2.0 * PAD, W - 2.0 * PAD) } else { (W - 2.0 * PAD, H - 2.0 * PAD) };
    let (m_max, c_max, m_min, c_min) = if tb { (56.0, 176.0, 24.0, 48.0) } else { (160.0, 72.0, 48.0, 24.0) };
    let k = layers.iter().map(|l| l.len()).max().unwrap();
    // Prefer room between ranks for arrows and edge labels; squeeze to MIN_GAP only if needed.
    let slot = |gap: f64| snap(((main_len - (nr - 1) as f64 * gap) / nr as f64).min(m_max));
    let m = [if tb { 40.0 } else { 56.0 }, MIN_GAP].map(slot).into_iter().find(|&m| m >= m_min)
        .ok_or("too many steps in a row: the flow does not fit; split it")?;
    let c = snap(((cross_len - (k - 1) as f64 * GAP) / k as f64).min(c_max));
    if c < c_min {
        return Err("too many nodes in one rank: the flow does not fit; split it".into());
    }
    let g = if nr > 1 { snap(((main_len - nr as f64 * m) / (nr - 1) as f64).min(MAX_GAP)) } else { 0.0 };
    let main0 = snap(PAD + (main_len - nr as f64 * m - (nr - 1) as f64 * g) / 2.0);
    // Slot origin (main, cross) per item.
    let mut at = vec![(0.0, 0.0); item_rank.len()];
    for (r, l) in layers.iter().enumerate() {
        let start = snap(PAD + (cross_len - l.len() as f64 * c - (l.len() - 1) as f64 * GAP) / 2.0);
        l.iter().enumerate().for_each(|(s, &i)| at[i] = (main0 + r as f64 * (m + g), start + s as f64 * (c + GAP)));
    }
    // Centre the real nodes (not the empty dummy slots) across the canvas.
    let (lo, hi) = (0..nodes.len()).fold((f64::MAX, f64::MIN), |(lo, hi), i| (lo.min(at[i].1), hi.max(at[i].1 + c)));
    let (alo, ahi) = at.iter().fold((f64::MAX, f64::MIN), |(lo, hi), a| (lo.min(a.1), hi.max(a.1 + c)));
    // ...but never push a dummy slot (an edge bend) off the canvas.
    let shift = snap(PAD + (cross_len - (hi - lo)) / 2.0 - lo).clamp(PAD - alo, PAD + cross_len - ahi);
    at.iter_mut().for_each(|a| a.1 += shift);
    let pt = |main: f64, cross: f64| if tb { (cross, main) } else { (main, cross) };
    let t = svg.t;

    // Text box inside a process/terminal slot; one font size for all of them (largest that fits every label).
    let (sw, sh) = if tb { (c, m) } else { (m, c) };
    let inner = |shape: Shape, x: f64, y: f64| {
        let ix = if shape == Shape::Terminal { (sw.min(sh) / 2.0).min(16.0) } else if sw >= 96.0 { 12.0 } else { 6.0 };
        Rect { x: x + ix, y: y + 4.0, w: sw - 2.0 * ix, h: sh - 8.0 }
    };
    let boxed = || nodes.iter().filter(|nd| nd.shape != Shape::Decision);
    let size = [16.0, 14.0, 12.0].into_iter()
        .find(|&sz| boxed().all(|nd| { let b = inner(nd.shape, 0.0, 0.0); fit_box(&nd.label, b.w, b.h, sz).is_ok() }))
        .unwrap_or(12.0);

    let base = svg.nodes.len();
    for (i, nd) in nodes.iter().enumerate() {
        let (x, y) = pt(at[i].0, at[i].1);
        let r = if tb { Rect { x, y, w: c, h: m } } else { Rect { x, y, w: m, h: c } };
        let (cx, cy) = (r.x + r.w / 2.0, r.y + r.h / 2.0);
        let b = match nd.shape {
            Shape::Decision if r.w >= 2.0 * r.h => {
                // Wide slot (TB): a diamond would be a sliver, so point only the ends.
                let d = r.h / 2.0;
                svg.body += &format!(
                    r#"<polygon points="{},{} {},{} {},{} {},{} {},{} {},{}" fill="{}" stroke="{}" stroke-width="1"/>"#,
                    n(r.x), n(cy), n(r.x + d), n(r.y), n(r.x + r.w - d), n(r.y), n(r.x + r.w), n(cy),
                    n(r.x + r.w - d), n(r.y + r.h), n(r.x + d), n(r.y + r.h), t.card, t.border
                );
                let b = Rect { x: r.x + d, y: r.y + 4.0, w: r.w - 2.0 * d, h: r.h - 8.0 };
                svg.nodes.push(b);
                b
            }
            Shape::Decision => {
                svg.body += &format!(
                    r#"<polygon points="{},{} {},{} {},{} {},{}" fill="{}" stroke="{}" stroke-width="1"/>"#,
                    n(cx), n(r.y), n(r.x + r.w), n(cy), n(cx), n(r.y + r.h), n(r.x), n(cy), t.card, t.border
                );
                // A centred rect of height h fits the diamond if its width <= w * (1 - h / H).
                let l = [16.0, 14.0, 12.0].iter().filter(|&&sz| sz <= size).flat_map(|&sz| (1..=2).map(move |k| (sz, k))).find_map(|(sz, k)| {
                    let h = (sz * 1.3f64).round() * k as f64 + 4.0;
                    fit(&nd.label, sz, r.w * (1.0 - h / r.h) - 4.0, k).ok().filter(|_| h < r.h)
                });
                let l = l.ok_or(format!("$.nodes[{i}].label: label \"{}\" does not fit in the decision; shorten it", nd.label))?;
                let (iw, ih) = (r.w * (1.0 - (l.height() + 4.0) / r.h), l.height() + 4.0);
                svg.nodes.push(Rect { x: cx - iw / 2.0, y: cy - ih / 2.0, w: iw, h: ih });
                svg.text(&l, Anchor::Middle, cx, cy - l.height() / 2.0, 500, &t.text);
                continue;
            }
            shape => {
                let radius = if shape == Shape::Terminal { r.w.min(r.h) / 2.0 } else { 8.0 };
                svg.node(r, &t.card, Some((&t.border, 1.0)), radius);
                inner(shape, r.x, r.y)
            }
        };
        // Decisions may step down below the shared size; boxes already fit at it unless nothing does.
        let mut l = fit_box(&nd.label, b.w, b.h, size);
        for sz in [14.0, 12.0] {
            if l.is_err() && nd.shape == Shape::Decision && sz < size {
                l = fit_box(&nd.label, b.w, b.h, sz);
            }
        }
        let l = l.map_err(|e| format!("$.nodes[{i}].label: {e}"))?;
        svg.text(&l, Anchor::Middle, cx, cy - l.height() / 2.0, 500, &t.text);
    }

    let base_edge = svg.edges.len();
    for (link, chain) in links.iter().zip(&chains) {
        let mid = |i: usize| at[i].1 + c / 2.0;
        let mut pts = vec![pt(at[link.from].0 + m, mid(link.from))];
        for &d in &chain[1..chain.len() - 1] {
            pts.push(pt(at[d].0, mid(d)));
            pts.push(pt(at[d].0 + m, mid(d)));
        }
        pts.push(pt(at[link.to].0, mid(link.to)));
        // A long edge goes straight when nothing is in the way; dummy slots only keep the room free.
        let direct = [pts[0], pts[pts.len() - 1]];
        let ends = [base + link.from, base + link.to];
        let clear = |r: &[Rect], skip: &dyn Fn(usize) -> bool| r.iter().enumerate().all(|(j, b)| skip(j) || !b.hit_by(direct[0], direct[1]));
        if pts.len() > 2 && clear(&svg.nodes, &|j| ends.contains(&j)) && clear(&svg.texts, &|_| false) {
            pts = direct.to_vec();
        }
        svg.edge(&pts, ends, &t.muted);
    }

    // Labels after all edges, so each can dodge every line: the first clear spot along its own edge.
    for (e, link) in links.iter().enumerate() {
        let Some(s) = &link.label else { continue };
        let idx = base_edge + e;
        let max_w = if tb { c } else { g - 8.0 };
        let l = fit(s, 12.0, max_w, 2).map_err(|err| format!("$.edges[{e}].label: {err}"))?;
        let (lw, lh) = (l.width(), l.height());
        let pts = svg.edges[idx].pts.clone();
        let plate_at = |(mx, my): (f64, f64)| Rect { x: mx - lw / 2.0 - 4.0, y: my - lh / 2.0 - 2.0, w: lw + 8.0, h: lh + 4.0 };
        let free = |p: &Rect| {
            !svg.texts.iter().chain(&svg.nodes).any(|o| o.overlaps(p))
                && svg.edges.iter().enumerate().all(|(j, o)| j == idx || o.pts.windows(2).all(|w| !p.hit_by(w[0], w[1])))
        };
        let spot = pts.windows(2).flat_map(|w| [0.5, 0.3, 0.7, 0.2, 0.8].map(|f| (w[0].0 + (w[1].0 - w[0].0) * f, w[0].1 + (w[1].1 - w[0].1) * f)))
            .find(|&q| free(&plate_at(q)))
            .ok_or(format!("$.edges[{e}].label: no clear spot along the edge; the label does not fit, shorten or drop it"))?;
        let p = plate_at(spot);
        svg.body += &format!(r#"<rect x="{}" y="{}" width="{}" height="{}" rx="4" fill="{}"/>"#, n(p.x), n(p.y), n(p.w), n(p.h), t.bg);
        svg.edges[idx].label = Some(svg.texts.len());
        svg.text(&l, Anchor::Middle, spot.0, spot.1 - lh / 2.0, 400, &t.muted);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};

    fn render(d: &Value) -> super::Result<String> {
        super::super::render("flowchart", d, None)
    }

    fn sample() -> Value {
        serde_json::from_str(include_str!("../../samples/diagrams/flowchart.json")).unwrap()
    }

    /// Ok, or a "does not fit" error; never a layout-check failure.
    fn fits_or_says_so(d: &Value) {
        if let Err(e) = render(d) {
            assert!(e.contains("does not fit"), "{e}");
        }
    }

    fn nodes(n: usize, label: &str) -> Vec<Value> {
        (0..n).map(|i| json!({"id": format!("n{i}"), "label": label})).collect()
    }

    #[test]
    fn sample_renders_deterministically_in_both_directions_and_themes() {
        let a = render(&sample()).unwrap();
        assert_eq!(a, render(&sample()).unwrap());
        assert!(a.contains("있음?") && a.contains("<polygon") && a.contains("polyline"));
        let mut tb = sample();
        tb["direction"] = json!("TB");
        render(&tb).unwrap();
        assert_ne!(super::super::render("flowchart", &sample(), Some("night")).unwrap(), a);
    }

    #[test]
    fn min_and_max_counts() {
        render(&json!({"nodes": nodes(2, "시작"), "edges": []})).unwrap();
        for dir in ["LR", "TB"] {
            // 12-step chain (deepest) and 1 -> 11 fan (widest).
            let chain: Vec<Value> = (0..11).map(|i| json!({"from": format!("n{i}"), "to": format!("n{}", i + 1)})).collect();
            fits_or_says_so(&json!({"direction": dir, "nodes": nodes(12, "단계"), "edges": chain}));
            let fan: Vec<Value> = (1..12).map(|i| json!({"from": "n0", "to": format!("n{i}")})).collect();
            fits_or_says_so(&json!({"direction": dir, "nodes": nodes(12, "분기"), "edges": fan}));
            // Dense: 24 forward edges over 12 nodes, labelled, through dummy slots.
            let dense: Vec<Value> = (0..12).flat_map(|i| [(i, i + 1), (i, i + 3)]).filter(|&(_, j)| j < 12)
                .chain([(0, 6), (1, 7), (2, 8), (3, 9), (4, 10)]).take(24)
                .map(|(i, j)| json!({"from": format!("n{i}"), "to": format!("n{j}"), "label": "예"})).collect();
            fits_or_says_so(&json!({"direction": dir, "nodes": nodes(12, "검토"), "edges": dense}));
        }
    }

    #[test]
    fn longest_labels_fit_or_say_so() {
        let long: String = "가나다라마바사아자 ".repeat(8).chars().take(80).collect();
        let mut d = sample();
        for n in d["nodes"].as_array_mut().unwrap() {
            n["label"] = json!(long);
        }
        fits_or_says_so(&d);
        let mut d = sample();
        d["edges"][3]["label"] = json!("가".repeat(40));
        fits_or_says_so(&d);
        let mut d = sample();
        d["nodes"][1]["label"] = json!("재고 수량 및 위치 확인");
        render(&d).unwrap();
    }

    #[test]
    fn strict_input() {
        let err = |d: Value| render(&d).unwrap_err();
        let mut d = sample();
        d["nodes"][0]["color"] = json!("red");
        assert_eq!(err(d), "$.nodes[0].color: unknown field");
        let mut d = sample();
        d["edges"][0]["to"] = json!("nope");
        assert!(err(d).starts_with("$.edges[0].to: unknown id"));
        let mut d = sample();
        d["edges"].as_array_mut().unwrap().push(json!({"from": "done", "to": "start"}));
        assert!(err(d).contains("cycles are not supported"));
        let mut d = sample();
        d["direction"] = json!("RL");
        assert!(err(d).starts_with("$.direction"));
        let mut d = sample();
        d["nodes"][0]["shape"] = json!("circle");
        assert!(err(d).starts_with("$.nodes[0].shape"));
        assert!(err(json!({"nodes": nodes(13, "x"), "edges": []})).contains("2-12"));
        assert!(err(json!({"nodes": nodes(2, &"가".repeat(81)), "edges": []})).contains("80"));
    }
}
