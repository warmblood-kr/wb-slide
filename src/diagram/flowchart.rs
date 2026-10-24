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
    id: String,
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
        nodes.push(Node { id: id.clone(), label: req_str(o, "label", &p, 80)?, shape });
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

/// (main_len, cross_len, m_max, c_max, m_min, c_min): canvas room and slot size bounds per direction.
fn geom(tb: bool) -> (f64, f64, f64, f64, f64, f64) {
    if tb { (H - 2.0 * PAD, W - 2.0 * PAD, 56.0, 176.0, 24.0, 48.0) } else { (W - 2.0 * PAD, H - 2.0 * PAD, 160.0, 96.0, 48.0, 24.0) }
}

/// Slot length along the main axis for `nr` ranks, and across it for `k` slots per rank.
fn slot_m(tb: bool, nr: usize, gap: f64) -> f64 {
    let (main_len, _, m_max, ..) = geom(tb);
    snap(((main_len - (nr - 1) as f64 * gap) / nr as f64).min(m_max))
}
fn slot_c(tb: bool, k: usize) -> f64 {
    let (_, cross_len, _, c_max, ..) = geom(tb);
    snap(((cross_len - (k - 1) as f64 * GAP) / k as f64).min(c_max))
}

/// Most ranks, and most slots in one rank, that still leave slots of the minimum size (LR 9 x 7, TB 8 x 8).
fn caps(tb: bool) -> (usize, usize) {
    let (.., m_min, c_min) = geom(tb);
    ((1..).take_while(|&r| slot_m(tb, r, MIN_GAP) >= m_min).last().unwrap(), (1..).take_while(|&k| slot_c(tb, k) >= c_min).last().unwrap())
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
    // Capacity: say which count is over, by how much, and what to change.
    let (dir, other) = if tb { ("TB", "LR") } else { ("LR", "TB") };
    let ((max_nr, max_k), (alt_nr, alt_k)) = (caps(tb), caps(!tb));
    if nr > max_nr {
        let alt = if nr <= alt_nr { format!(" or use direction {other}") } else { String::new() };
        return Err(format!("$.edges: the longest chain has {nr} ranks, {dir} fits at most {max_nr}; shorten the longest chain{alt}"));
    }
    if let Some((r, l)) = layers.iter().enumerate().find(|(_, l)| l.len() > max_k) {
        let real: Vec<&str> = l.iter().filter(|&&i| i < nodes.len()).map(|&i| nodes[i].id.as_str()).collect();
        let d = l.len() - real.len();
        let held = if d == 0 { format!("{} nodes", real.len()) } else { format!("{} nodes + {d} edges passing through", real.len()) };
        let skip = if d > 0 { ", drop edges that pass through this rank" } else { "" };
        let alt = if l.len() <= alt_k { format!(", or use direction {other}") } else { String::new() };
        return Err(format!("$.nodes: rank {} holds {held}, {dir} fits at most {max_k}; chain some of {} instead of placing them side by side{skip}{alt}", r + 1, real.join(", ")));
    }
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
    let (main_len, cross_len, _, _, m_min, _) = geom(tb);
    let k = layers.iter().map(|l| l.len()).max().unwrap();
    // Prefer room between ranks for arrows and edge labels; squeeze to MIN_GAP only if needed (caps() vouches it fits).
    let m = [if tb { 32.0 } else { 40.0 }, MIN_GAP].map(|g| slot_m(tb, nr, g)).into_iter().find(|&m| m >= m_min).unwrap();
    let c = slot_c(tb, k);
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
                svg.nodes.push(r); // the whole slot, so the check sees an edge across a point
                Rect { x: r.x + d, y: r.y + 4.0, w: r.w - 2.0 * d, h: r.h - 8.0 }
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
                svg.nodes.push(r); // bounding box: conservative, but no edge clips a corner unseen
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
    // Dummy slots per chain: another edge's direct run must not take their lane.
    let slot_rect = |i: usize| { let (x, y) = pt(at[i].0, at[i].1); if tb { Rect { x, y, w: c, h: m } } else { Rect { x, y, w: m, h: c } } };
    let dummies: Vec<Vec<Rect>> = chains.iter().map(|ch| ch[1..ch.len() - 1].iter().map(|&d| slot_rect(d)).collect()).collect();
    for (e, (link, chain)) in links.iter().zip(&chains).enumerate() {
        // Points on (main, cross); every change of cross becomes an elbow in the middle of the gap.
        let mid = |i: usize| at[i].1 + c / 2.0;
        let mut mc = vec![(at[link.from].0 + m, mid(link.from))];
        for &d in &chain[1..chain.len() - 1] {
            mc.extend([(at[d].0, mid(d)), (at[d].0 + m, mid(d))]);
        }
        mc.push((at[link.to].0, mid(link.to)));
        // Unrelated edges never share a trunk: each bend takes the first free lane near the gap middle.
        let ends = [base + link.from, base + link.to];
        let others: Vec<&[(f64, f64)]> = svg.edges[base_edge..].iter()
            .filter(|e| !e.ends.iter().any(|x| ends.contains(x))).map(|e| e.pts.as_slice()).collect();
        let free = |p: &[(f64, f64)]| !others.iter().any(|o| runs_overlap(p, o));
        let ortho = |mc: &[(f64, f64)]| -> Option<Vec<(f64, f64)>> {
            let mut out = vec![pt(mc[0].0, mc[0].1)];
            for w in mc.windows(2) {
                let last = *out.last().unwrap();
                if w[0].1 != w[1].1 {
                    let lanes = [0.0, -8.0, 8.0, -16.0, 16.0, -24.0, 24.0].into_iter().filter(|o: &f64| *o == 0.0 || o.abs() <= g / 2.0 - 8.0);
                    // Two edges swapping lanes in one gap cannot both elbow without sharing a run: cross diagonally.
                    if let Some(b) = lanes.map(|o| w[1].0 - g / 2.0 + o)
                        .find(|&b| free(&[last, pt(b, w[0].1), pt(b, w[1].1), pt(w[1].0, w[1].1)])) {
                        out.extend([pt(b, w[0].1), pt(b, w[1].1)]);
                    }
                }
                out.push(pt(w[1].0, w[1].1));
            }
            Some(out).filter(|o| free(o))
        };
        // A long edge runs straight and bends only in the last gap when nothing is in the way.
        let direct = ortho(&[mc[0], mc[mc.len() - 1]]).filter(|d| {
            let clear = |r: &[Rect], skip: &dyn Fn(usize) -> bool| r.iter().enumerate().all(|(j, b)| skip(j) || d.windows(2).all(|s| !b.hit_by(s[0], s[1])));
            mc.len() > 2 && clear(&svg.nodes, &|j| ends.contains(&j)) && clear(&svg.texts, &|_| false)
                && dummies.iter().enumerate().all(|(k, r)| k == e || clear(r, &|_| false))
        });
        let pts = direct.or_else(|| ortho(&mc)).ok_or("too many crossing edges: the flow does not fit; split it")?;
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
        // Longest segment first: a label sits best on a long run, not beside an elbow.
        let mut segs: Vec<&[(f64, f64)]> = pts.windows(2).collect();
        let len = |w: &[(f64, f64)]| (w[1].0 - w[0].0).abs() + (w[1].1 - w[0].1).abs();
        segs.sort_by(|a, b| len(b).total_cmp(&len(a)));
        let spot = segs.into_iter().flat_map(|w| [0.5, 0.3, 0.7, 0.2, 0.8].map(|f| (w[0].0 + (w[1].0 - w[0].0) * f, w[0].1 + (w[1].1 - w[0].1) * f)))
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

    /// Ok, or a "does not fit" / capacity error; never a layout-check failure.
    fn fits_or_says_so(d: &Value) {
        if let Err(e) = render(d) {
            assert!(e.contains("does not fit") || e.contains("fits at most"), "{e}");
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
    fn check_catches_edge_through_decision_corner() {
        // Repro: a decision registered only its inner text box, so an edge across its point went unnoticed.
        let t = super::Theme::load(None).unwrap();
        for dir in ["LR", "TB"] {
            let mut d = sample();
            d["direction"] = json!(dir);
            let mut s = super::Svg::new(&t);
            super::draw(&d, &mut s).unwrap();
            s.check().unwrap();
            let pts = s.body.split("<polygon points=\"").nth(1).unwrap().split('"').next().unwrap();
            let (vx, vy) = pts.split(' ').map(|p| { let (x, y) = p.split_once(',').unwrap(); (x.parse::<f64>().unwrap(), y.parse::<f64>().unwrap()) })
                .min_by(|a, b| a.0.total_cmp(&b.0)).unwrap();
            s.edge(&[(vx + 2.0, vy), (vx + 10.0, vy)], [usize::MAX, usize::MAX], "#000");
            assert!(s.check().unwrap_err().contains("through node"), "{dir}");
        }
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
            // Known-good near the limits must render, so an all-error regression cannot pass as "says so".
            render(&json!({"direction": dir, "nodes": nodes(8, "단계"), "edges": &chain[..7]})).unwrap();
            render(&json!({"direction": dir, "nodes": nodes(8, "분기"), "edges": &fan[..7]})).unwrap();
            // Max counts: 12 nodes, 24 edges (4 ranks of 3, consecutive ranks linked).
            let grid: Vec<Value> = (0..3).flat_map(|r| (0..3).flat_map(move |a| (0..3).map(move |b| (r * 3 + a, r * 3 + 3 + b)))).take(24)
                .map(|(i, j)| json!({"from": format!("n{i}"), "to": format!("n{j}")})).collect();
            render(&json!({"direction": dir, "nodes": nodes(12, "검토"), "edges": grid})).unwrap();
        }
    }

    fn chain(n: usize) -> Vec<Value> {
        (1..n).map(|i| json!({"from": format!("n{}", i - 1), "to": format!("n{i}")})).collect()
    }

    fn fan(n: usize) -> Vec<Value> {
        (1..n).map(|i| json!({"from": "n0", "to": format!("n{i}")})).collect()
    }

    #[test]
    fn capacity_limits_are_explicit_errors() {
        // Repro: a 10-step LR chain failed only with a generic "does not fit".
        for (dir, deep, wide) in [("LR", 9, 7), ("TB", 8, 8)] {
            render(&json!({"direction": dir, "nodes": nodes(deep, "단계"), "edges": chain(deep)})).unwrap();
            render(&json!({"direction": dir, "nodes": nodes(wide + 1, "분기"), "edges": fan(wide + 1)})).unwrap();
            let e = render(&json!({"direction": dir, "nodes": nodes(deep + 1, "단계"), "edges": chain(deep + 1)})).unwrap_err();
            assert!(e.starts_with(&format!("$.edges: the longest chain has {} ranks, {dir} fits at most {deep};", deep + 1)), "{e}");
            let e = render(&json!({"direction": dir, "nodes": nodes(wide + 2, "분기"), "edges": fan(wide + 2)})).unwrap_err();
            assert!(e.starts_with(&format!("$.nodes: rank 2 holds {} nodes, {dir} fits at most {wide};", wide + 1)), "{e}");
        }
        // Edges passing through a rank take slots too.
        let mut e = fan(7);
        e.extend([json!({"from": "n1", "to": "n7"}), json!({"from": "n0", "to": "n7"}), json!({"from": "n7", "to": "n8"}), json!({"from": "n0", "to": "n8"})]);
        let err = render(&json!({"nodes": nodes(9, "분기"), "edges": e})).unwrap_err();
        assert!(err.starts_with("$.nodes: rank 2 holds 6 nodes + 2 edges passing through, LR fits at most 7;"), "{err}");
        assert!(err.contains("n1, n2"), "{err}");
    }

    #[test]
    fn inputs_within_the_limits_never_fail_for_capacity() {
        // Sweep: whatever passes the rank and slot checks lays out; only label and crossing limits remain.
        let mut seed = 11u64;
        let mut rnd = |k: u64| { seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); (seed >> 33) % k };
        let (mut ok, mut cap, mut spot) = (0, 0, 0);
        for _ in 0..1000 {
            let n = 2 + rnd(11) as usize;
            let mut edges = Vec::new();
            for j in 1..n {
                for i in 0..j {
                    if edges.len() < 24 && rnd(4) == 0 { edges.push(json!({"from": format!("n{i}"), "to": format!("n{j}"), "label": "예"})); }
                }
            }
            let dir = if rnd(2) == 0 { "LR" } else { "TB" };
            match render(&json!({"direction": dir, "nodes": nodes(n, "단계"), "edges": edges})) {
                Ok(_) => ok += 1,
                Err(e) if e.contains("fits at most") => cap += 1,
                Err(e) if e.contains("no clear spot") => spot += 1,
                Err(e) => panic!("{e}"),
            }
        }
        eprintln!("capacity sweep: ok {ok}, capacity {cap}, label spot {spot}");
        assert!(ok >= 500, "only {ok}/1000 rendered");
    }

    #[test]
    fn random_flows_never_draw_one_edge_over_another() {
        // Repro: unrelated edges bending in the same gap used to share one trunk x.
        let mut seed = 7u64;
        let mut rnd = |k: u64| { seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); (seed >> 33) % k };
        let mut ok = 0;
        for _ in 0..300 {
            let n = 4 + rnd(6) as usize;
            let mut edges = Vec::new();
            for j in 1..n {
                for i in 0..j {
                    if rnd(3) == 0 { edges.push(json!({"from": format!("n{i}"), "to": format!("n{j}")})); }
                }
            }
            let dir = if rnd(2) == 0 { "LR" } else { "TB" };
            let d = json!({"direction": dir, "nodes": nodes(n, "단계"), "edges": edges});
            fits_or_says_so(&d);
            ok += render(&d).is_ok() as usize;
        }
        assert!(ok >= 290, "only {ok}/300 rendered");
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
        d["nodes"][2]["label"] = json!("재고 수량 및 위치 확인");
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
        let mut d = sample();
        d["nodes"][1]["id"] = json!("start");
        assert_eq!(err(d), "$.nodes[1].id: duplicate id \"start\"");
        let mut d = sample();
        d["edges"][0]["to"] = json!("start");
        assert_eq!(err(d), "$.edges[0]: self-loop on \"start\"");
        let mut d = sample();
        d["edges"].as_array_mut().unwrap().push(json!({"from": "pack", "to": "done", "label": "다시"}));
        assert_eq!(err(d), "$.edges[5]: duplicate edge pack -> done");
        let many: Vec<Value> = (0..25).map(|_| json!({"from": "n0", "to": "n1"})).collect();
        assert_eq!(err(json!({"nodes": nodes(2, "x"), "edges": many})), "$.edges: expected an array of 0-24 entries");
    }
}
