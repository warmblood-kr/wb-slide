//! Deterministic diagrams: compact JSON data -> SVG, themed by the slide preset tokens.
//! Every render runs `Svg::check`: text inside the canvas, no overlaps, no edge
//! through a label. A layout that cannot fit is an error, never clipped output.
use serde_json::{Map, Value};

mod flowchart;
mod quadrant;

pub type Result<T> = std::result::Result<T, String>;

pub const W: f64 = 960.0;
pub const H: f64 = 540.0;
pub const PAD: f64 = 32.0;
const MIN_SIZE: f64 = 12.0;

pub const TYPES: &[&str] = &["flowchart", "quadrant"];

/// Render `data` as diagram `kind` with a preset theme (None, "default", "night" or a token file).
pub fn render(kind: &str, data: &Value, theme: Option<&str>) -> Result<String> {
    let t = Theme::load(theme)?;
    let mut svg = Svg::new(&t);
    match kind {
        "flowchart" => flowchart::draw(data, &mut svg)?,
        "quadrant" => quadrant::draw(data, &mut svg)?,
        _ => return Err(format!("unknown diagram type {kind}; expected one of {}", TYPES.join(", "))),
    }
    svg.check()?;
    Ok(svg.finish())
}

pub struct Theme {
    pub font: String,
    pub text: String,
    pub muted: String,
    pub accent: String,
    pub bg: String,
    pub card: String,
    pub border: String,
}

impl Theme {
    pub fn load(theme: Option<&str>) -> Result<Self> {
        let values = crate::markup::token_values(theme)?;
        let get = |n: &str| values.iter().find(|(k, _)| k == n).map(|(_, v)| esc(v)).unwrap_or_default();
        Ok(Theme {
            font: get("--wb-font"), text: get("--wb-text"), muted: get("--wb-muted"), accent: get("--wb-accent"),
            bg: get("--wb-bg"), card: get("--wb-card-bg"), border: get("--wb-border"),
        })
    }
}

pub fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// Conservative advance width in em: wide (CJK/Hangul) 1.0, Latin 0.62, space 0.32.
/// ponytail: no font metrics; overestimates Pretendard so fallback fonts still fit.
fn char_em(c: char) -> f64 {
    match c {
        ' ' => 0.32,
        '\u{1100}'..='\u{11FF}' | '\u{2E80}'..='\u{9FFF}' | '\u{AC00}'..='\u{D7AF}' | '\u{F900}'..='\u{FAFF}' | '\u{FF00}'..='\u{FFEF}' => 1.0,
        _ => 0.62,
    }
}

pub fn width(s: &str, size: f64) -> f64 {
    s.chars().map(char_em).sum::<f64>() * size
}

/// Greedy wrap at spaces; a word wider than `max_w` breaks between characters.
fn wrap(s: &str, size: f64, max_w: f64) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    for word in s.split_whitespace() {
        let joined = if cur.is_empty() { word.to_string() } else { format!("{cur} {word}") };
        if width(&joined, size) <= max_w {
            cur = joined;
            continue;
        }
        if !cur.is_empty() {
            lines.push(std::mem::take(&mut cur));
        }
        for c in word.chars() {
            if !cur.is_empty() && width(&format!("{cur}{c}"), size) > max_w {
                lines.push(std::mem::take(&mut cur));
            }
            cur.push(c);
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}

/// A label fitted to a box: wrapped lines at the largest size (down to 12px) that fits.
pub struct Label {
    pub lines: Vec<String>,
    pub size: f64,
}

impl Label {
    pub fn line_h(&self) -> f64 {
        (self.size * 1.3).round()
    }
    pub fn height(&self) -> f64 {
        self.line_h() * self.lines.len() as f64
    }
    pub fn width(&self) -> f64 {
        self.lines.iter().map(|l| width(l, self.size)).fold(0.0, f64::max)
    }
}

pub fn fit(s: &str, size: f64, max_w: f64, max_lines: usize) -> Result<Label> {
    let mut sz = size;
    loop {
        let lines = wrap(s, sz, max_w);
        if lines.len() <= max_lines {
            return Ok(Label { lines, size: sz });
        }
        if sz - 2.0 < MIN_SIZE {
            return Err(format!("label \"{s}\" does not fit in {max_w}px x {max_lines} lines; shorten it"));
        }
        sz -= 2.0;
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    fn overlaps(&self, o: &Rect) -> bool {
        self.x < o.x + o.w && o.x < self.x + self.w && self.y < o.y + o.h && o.y < self.y + self.h
    }
    fn contains(&self, o: &Rect) -> bool {
        o.x >= self.x && o.y >= self.y && o.x + o.w <= self.x + self.w && o.y + o.h <= self.y + self.h
    }
    /// Liang-Barsky: does segment a-b cross this rect shrunk by 1px (so touching a border is fine)?
    fn hit_by(&self, a: (f64, f64), b: (f64, f64)) -> bool {
        let (x0, y0, x1, y1) = (self.x + 1.0, self.y + 1.0, self.x + self.w - 1.0, self.y + self.h - 1.0);
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let (mut t0, mut t1) = (0.0f64, 1.0f64);
        for (p, q) in [(-dx, a.0 - x0), (dx, x1 - a.0), (-dy, a.1 - y0), (dy, y1 - a.1)] {
            if p == 0.0 {
                if q < 0.0 {
                    return false;
                }
            } else if p < 0.0 {
                t0 = t0.max(q / p);
            } else {
                t1 = t1.min(q / p);
            }
        }
        x0 < x1 && y0 < y1 && t0 <= t1
    }
}

/// A recorded connector: polyline points, the node indices it joins, and its own label's text index.
pub struct Edge {
    pub pts: Vec<(f64, f64)>,
    pub ends: [usize; 2],
    pub label: Option<usize>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Anchor {
    Start,
    Middle,
    End,
}

/// SVG writer that records every text and node box for the layout check.
pub struct Svg<'a> {
    pub t: &'a Theme,
    body: String,
    pub texts: Vec<Rect>,
    pub nodes: Vec<Rect>,
    pub edges: Vec<Edge>,
}

/// Round to 0.5px so output is byte-stable across platforms.
fn n(v: f64) -> String {
    let r = (v * 2.0).round() / 2.0;
    if r.fract() == 0.0 { format!("{}", r as i64) } else { format!("{r:.1}") }
}

impl<'a> Svg<'a> {
    pub fn new(t: &'a Theme) -> Self {
        Svg { t, body: String::new(), texts: Vec::new(), nodes: Vec::new(), edges: Vec::new() }
    }

    /// A filled node box (counts for the overlap check).
    pub fn node(&mut self, r: Rect, fill: &str, stroke: Option<(&str, f64)>, radius: f64) {
        let stroke = stroke.map(|(c, w)| format!(r#" stroke="{c}" stroke-width="{}""#, n(w))).unwrap_or_default();
        self.body += &format!(r#"<rect x="{}" y="{}" width="{}" height="{}" rx="{}" fill="{fill}"{stroke}/>"#, n(r.x), n(r.y), n(r.w), n(r.h), n(radius));
        self.nodes.push(r);
    }

    /// A small decorative circle (bullets, timeline ticks); not a node.
    pub fn dot(&mut self, cx: f64, cy: f64, r: f64, fill: &str) {
        self.body += &format!(r#"<circle cx="{}" cy="{}" r="{}" fill="{fill}"/>"#, n(cx), n(cy), n(r));
    }

    pub fn line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, color: &str, arrow: bool) {
        let head = if arrow { r#" marker-end="url(#arrow)""# } else { "" };
        self.body += &format!(r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{color}" stroke-width="1.5"{head}/>"#, n(x1), n(y1), n(x2), n(y2));
    }

    /// A polyline from node `ends[0]` to node `ends[1]` (indices into `nodes`), arrow at the end.
    /// Returns the edge index so the caller can attach its label.
    pub fn edge(&mut self, pts: &[(f64, f64)], ends: [usize; 2], color: &str) -> usize {
        let p: Vec<String> = pts.iter().map(|(x, y)| format!("{},{}", n(*x), n(*y))).collect();
        self.body += &format!(r#"<polyline points="{}" fill="none" stroke="{color}" stroke-width="1.5" stroke-linejoin="round" marker-end="url(#arrow)"/>"#, p.join(" "));
        self.edges.push(Edge { pts: pts.to_vec(), ends, label: None });
        self.edges.len() - 1
    }

    /// Draw a fitted label with its top at `y`; returns the label height.
    pub fn text(&mut self, l: &Label, anchor: Anchor, x: f64, y: f64, weight: u16, color: &str) -> f64 {
        let (a, left) = match anchor {
            Anchor::Start => ("start", x),
            Anchor::Middle => ("middle", x - l.width() / 2.0),
            Anchor::End => ("end", x - l.width()),
        };
        let lh = l.line_h();
        let mut spans = String::new();
        for (i, line) in l.lines.iter().enumerate() {
            // Baseline sits ~0.8em below the line top for Pretendard-like fonts.
            let base = y + lh * i as f64 + (lh - l.size) / 2.0 + l.size * 0.8;
            spans += &format!(r#"<tspan x="{}" y="{}">{}</tspan>"#, n(x), n(base), esc(line));
        }
        self.body += &format!(r#"<text font-size="{}" font-weight="{weight}" fill="{color}" text-anchor="{a}">{spans}</text>"#, n(l.size));
        self.texts.push(Rect { x: left, y, w: l.width(), h: l.height() });
        l.height()
    }

    /// Layout guarantees: texts inside the canvas, texts/nodes pairwise disjoint,
    /// a text either inside one node or clear of all nodes, and no edge through
    /// a text (other than its own label) or a node other than its endpoints.
    pub fn check(&self) -> Result<()> {
        let canvas = Rect { x: 0.0, y: 0.0, w: W, h: H };
        for (i, a) in self.texts.iter().enumerate() {
            if !canvas.contains(a) {
                return Err(format!("layout: text {i} {a:?} leaves the canvas"));
            }
            if let Some(j) = self.texts[i + 1..].iter().position(|b| a.overlaps(b)) {
                return Err(format!("layout: texts {i} and {} overlap", i + 1 + j));
            }
            if self.nodes.iter().any(|b| a.overlaps(b) && !b.contains(a)) {
                return Err(format!("layout: text {i} straddles a node edge"));
            }
        }
        for (i, a) in self.nodes.iter().enumerate() {
            if self.nodes[i + 1..].iter().any(|b| a.overlaps(b)) {
                return Err(format!("layout: node {i} overlaps another node"));
            }
        }
        for (i, e) in self.edges.iter().enumerate() {
            for s in e.pts.windows(2) {
                if let Some(j) = (0..self.texts.len()).find(|&j| e.label != Some(j) && self.texts[j].hit_by(s[0], s[1])) {
                    return Err(format!("layout: edge {i} passes through text {j}"));
                }
                if let Some(j) = (0..self.nodes.len()).find(|&j| !e.ends.contains(&j) && self.nodes[j].hit_by(s[0], s[1])) {
                    return Err(format!("layout: edge {i} passes through node {j}"));
                }
            }
        }
        Ok(())
    }

    pub fn finish(self) -> String {
        let t = self.t;
        format!(
            concat!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}" font-family="{font}">"#,
                r#"<defs><marker id="arrow" viewBox="0 0 8 8" refX="7" refY="4" markerWidth="8" markerHeight="8" orient="auto"><path d="M0 0L8 4L0 8z" fill="{muted}"/></marker></defs>"#,
                r#"<rect width="{w}" height="{h}" fill="{bg}"/>{body}</svg>"#, "\n"
            ),
            w = W, h = H, font = t.font, muted = t.muted, bg = t.bg, body = self.body
        )
    }
}

/// Strict input helpers: unknown fields are errors, every error names a `$.path`.
pub fn obj<'v>(v: &'v Value, path: &str, allowed: &[&str]) -> Result<&'v Map<String, Value>> {
    let m = v.as_object().ok_or(format!("{path}: expected an object"))?;
    if let Some(k) = m.keys().find(|k| !allowed.contains(&k.as_str())) {
        return Err(format!("{path}.{k}: unknown field"));
    }
    Ok(m)
}

pub fn opt_str(m: &Map<String, Value>, key: &str, path: &str, max: usize) -> Result<Option<String>> {
    match m.get(key) {
        None => Ok(None),
        Some(Value::String(s)) if !s.trim().is_empty() && s.chars().count() <= max => Ok(Some(s.trim().to_string())),
        Some(_) => Err(format!("{path}.{key}: expected nonblank text of at most {max} characters")),
    }
}

pub fn req_str(m: &Map<String, Value>, key: &str, path: &str, max: usize) -> Result<String> {
    opt_str(m, key, path, max)?.ok_or(format!("{path}.{key}: required"))
}

pub fn arr<'v>(m: &'v Map<String, Value>, key: &str, path: &str, min: usize, max: usize) -> Result<&'v Vec<Value>> {
    match m.get(key) {
        Some(Value::Array(a)) if (min..=max).contains(&a.len()) => Ok(a),
        _ => Err(format!("{path}.{key}: expected an array of {min}-{max} entries")),
    }
}

pub fn str_items(v: &Value, path: &str, max: usize) -> Result<String> {
    match v {
        Value::String(s) if !s.trim().is_empty() && s.chars().count() <= max => Ok(s.trim().to_string()),
        _ => Err(format!("{path}: expected nonblank text of at most {max} characters")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_shrinks_then_errors() {
        let l = fit("가나다라마바사아자차", 20.0, 120.0, 1).unwrap();
        assert!(l.size < 20.0 && l.width() <= 120.0);
        assert!(fit("가나다라마바사아자차카타파하", 20.0, 40.0, 1).is_err());
    }

    #[test]
    fn wrap_breaks_long_words() {
        let lines = wrap("supercalifragilistic word", 10.0, 60.0);
        assert!(lines.iter().all(|l| width(l, 10.0) <= 60.0), "{lines:?}");
    }

    #[test]
    fn check_catches_overlap_and_overflow() {
        let t = Theme::load(None).unwrap();
        let mut s = Svg::new(&t);
        let l = fit("hello", 16.0, 200.0, 1).unwrap();
        s.text(&l, Anchor::Start, 10.0, 10.0, 400, "#000");
        s.text(&l, Anchor::Start, 12.0, 12.0, 400, "#000");
        assert!(s.check().unwrap_err().contains("overlap"));
        let mut s = Svg::new(&t);
        s.text(&l, Anchor::End, 5.0, 10.0, 400, "#000");
        assert!(s.check().unwrap_err().contains("canvas"));
    }

    #[test]
    fn check_catches_edge_through_node_or_text() {
        let t = Theme::load(None).unwrap();
        let r = |x| Rect { x, y: 100.0, w: 40.0, h: 40.0 };
        let mut s = Svg::new(&t);
        s.node(r(0.0), "#fff", None, 0.0);
        s.node(r(100.0), "#fff", None, 0.0);
        s.node(r(200.0), "#fff", None, 0.0);
        // 0 -> 2 straight through node 1.
        s.edge(&[(40.0, 120.0), (200.0, 120.0)], [0, 2], "#000");
        assert!(s.check().unwrap_err().contains("edge 0 passes through node 1"));
        // Around node 1 (touching its border within tolerance) is fine; through a foreign label is not.
        s.edges.clear();
        s.edge(&[(40.0, 120.0), (60.0, 99.5), (180.0, 99.5), (200.0, 120.0)], [0, 2], "#000");
        s.check().unwrap();
        let l = fit("label", 12.0, 200.0, 1).unwrap();
        s.text(&l, Anchor::Middle, 70.0, 90.0, 400, "#000"); // in the gap, across the detour
        assert!(s.check().unwrap_err().contains("edge 0 passes through text 0"));
        s.edges[0].label = Some(0);
        s.check().unwrap();
    }

    #[test]
    fn theme_values_are_escaped_and_night_differs() {
        let d = Theme::load(None).unwrap();
        let n = Theme::load(Some("night")).unwrap();
        assert_eq!(d.accent, "#2563EB");
        assert_ne!(d.bg, n.bg);
        assert!(d.font.starts_with("Pretendard"));
    }
}
