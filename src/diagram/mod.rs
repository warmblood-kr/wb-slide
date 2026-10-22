//! Deterministic diagrams: compact JSON data -> SVG, themed by the slide preset tokens.
//! Every render runs `Svg::check`: text inside the canvas, no overlaps, no edge
//! through a label. A layout that cannot fit is an error, never clipped output.
use serde_json::{Map, Value};

mod cycle;
mod hub;
mod quadrant;

pub type Result<T> = std::result::Result<T, String>;

pub const W: f64 = 960.0;
pub const H: f64 = 540.0;
pub const PAD: f64 = 32.0;
const MIN_SIZE: f64 = 12.0;

pub const TYPES: &[&str] = &["quadrant", "hub", "cycle"];

/// Render `data` as diagram `kind` with a preset theme (None, "default", "night" or a token file).
pub fn render(kind: &str, data: &Value, theme: Option<&str>) -> Result<String> {
    let t = Theme::load(theme)?;
    let mut svg = Svg::new(&t);
    match kind {
        "quadrant" => quadrant::draw(data, &mut svg)?,
        "hub" => hub::draw(data, &mut svg)?,
        "cycle" => cycle::draw(data, &mut svg)?,
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
}

/// Where the ray from `a`'s centre toward `b`'s centre leaves `a`, pushed `gap` further out.
fn exit(a: &Rect, b: &Rect, gap: f64) -> (f64, f64) {
    let (ax, ay) = (a.x + a.w / 2.0, a.y + a.h / 2.0);
    let (dx, dy) = (b.x + b.w / 2.0 - ax, b.y + b.h / 2.0 - ay);
    let k = (a.w / 2.0 / dx.abs()).min(a.h / 2.0 / dy.abs()) + gap / dx.hypot(dy);
    (ax + dx * k, ay + dy * k)
}

/// Round to 0.5px so output is byte-stable across platforms.
fn n(v: f64) -> String {
    let r = (v * 2.0).round() / 2.0;
    if r.fract() == 0.0 { format!("{}", r as i64) } else { format!("{r:.1}") }
}

impl<'a> Svg<'a> {
    pub fn new(t: &'a Theme) -> Self {
        Svg { t, body: String::new(), texts: Vec::new(), nodes: Vec::new() }
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

    /// A straight connector between two node boxes, clipped at both borders with a small gap.
    pub fn link(&mut self, a: Rect, b: Rect, color: &str, arrow: bool) {
        let (x1, y1) = exit(&a, &b, 6.0);
        let (x2, y2) = exit(&b, &a, 6.0);
        self.line(x1, y1, x2, y2, color, arrow);
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
    /// and a text either inside one node or clear of all nodes.
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

/// Round up to the 8px grid.
pub fn up8(v: f64) -> f64 {
    (v / 8.0).ceil() * 8.0
}

/// Uniform boxes of `w` x `h` centred on an ellipse filling the canvas, clockwise from 12 o'clock.
pub fn ring(n: usize, w: f64, h: f64) -> Vec<Rect> {
    let (rx, ry) = (W / 2.0 - PAD - w / 2.0, H / 2.0 - PAD - h / 2.0);
    (0..n)
        .map(|i| {
            let a = std::f64::consts::TAU * i as f64 / n as f64;
            let (x, y) = (W / 2.0 + rx * a.sin(), H / 2.0 - ry * a.cos());
            Rect { x: ((x - w / 2.0) / 8.0).round() * 8.0, y: ((y - h / 2.0) / 8.0).round() * 8.0, w, h }
        })
        .collect()
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
    fn theme_values_are_escaped_and_night_differs() {
        let d = Theme::load(None).unwrap();
        let n = Theme::load(Some("night")).unwrap();
        assert_eq!(d.accent, "#2563EB");
        assert_ne!(d.bg, n.bg);
        assert!(d.font.starts_with("Pretendard"));
    }
}
