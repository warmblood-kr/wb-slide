//! Deterministic diagrams: compact JSON data -> SVG, themed by the slide preset tokens.
//! Every render runs `Svg::check`: text inside the canvas, no text or node overlaps.
//! A layout that cannot fit is an error, never clipped output.
use serde_json::{Map, Value};

mod quadrant;

pub type Result<T> = std::result::Result<T, String>;

pub const W: f64 = 640.0;
pub const H: f64 = 360.0;
pub const PAD: f64 = 24.0;
const MIN_SIZE: f64 = 12.0;

pub const TYPES: &[&str] = &["quadrant"];

/// Render `data` as diagram `kind` with a preset theme (None, "default", "night" or a token file).
pub fn render(kind: &str, data: &Value, theme: Option<&str>) -> Result<String> {
    let t = Theme::load(theme)?;
    let mut svg = Svg::new(&t);
    match kind {
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
        let get = |n: &str| values.iter().find(|(k, _)| k == n).map(|(_, v)| esc(v)).ok_or(format!("theme lacks {n}"));
        Ok(Theme {
            font: get("--wb-font")?, text: get("--wb-text")?, muted: get("--wb-muted")?, accent: get("--wb-accent")?,
            bg: get("--wb-bg")?, card: get("--wb-card-bg")?, border: get("--wb-border")?,
        })
    }
}

pub fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// Conservative advance width in em, sized for bold Pretendard: wide glyphs
/// (Hangul jamo, symbols/dingbats U+2600-27BF and U+2B00-2BFF, U+2E80 and up: CJK, Hangul, fullwidth, emoji) 1.0, wide Latin 1.0,
/// other capitals 0.75, everything else 0.62, space 0.32.
/// ponytail: no font metrics; overestimates so fallback fonts still fit.
fn char_em(c: char) -> f64 {
    match c {
        ' ' => 0.32,
        '\u{1100}'..='\u{11FF}' | '\u{2600}'..='\u{27BF}' | '\u{2B00}'..='\u{2BFF}' | '\u{2E80}'.. => 1.0,
        'M' | 'W' | 'm' | 'w' | '@' | '%' => 1.0,
        'A'..='Z' => 0.75,
        _ => 0.62,
    }
}

/// Marker id keyed by its colour, so diagrams inlined into one page never
/// pick up another theme's arrowhead (same colour = identical def, harmless).
fn arrow_id(color: &str) -> String {
    let tag: String = color.chars().filter(char::is_ascii_alphanumeric).collect();
    format!("wb-arrow-{tag}")
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
        let l = Label { lines: wrap(s, sz, max_w), size: sz };
        // wrap keeps >=1 char per line, so a glyph wider than max_w can still overflow.
        if l.lines.len() <= max_lines && l.width() <= max_w {
            return Ok(l);
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
        let head = if arrow { format!(r#" marker-end="url(#{})""#, arrow_id(&self.t.muted)) } else { String::new() };
        self.body += &format!(r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{color}" stroke-width="1.5"{head}/>"#, n(x1), n(y1), n(x2), n(y2));
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
                r#"<defs><marker id="{arrow}" viewBox="0 0 8 8" refX="7" refY="4" markerWidth="8" markerHeight="8" orient="auto"><path d="M0 0L8 4L0 8z" fill="{muted}"/></marker></defs>"#,
                r#"<rect width="{w}" height="{h}" fill="{bg}"/>{body}</svg>"#, "\n"
            ),
            w = W, h = H, font = t.font, muted = t.muted, arrow = arrow_id(&t.muted), bg = t.bg, body = self.body
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
    m.get(key).map(|v| str_items(v, &format!("{path}.{key}"), max)).transpose()
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

/// One-line label text: nonblank, at most `max` characters, no control characters
/// (they are not valid XML).
pub fn str_items(v: &Value, path: &str, max: usize) -> Result<String> {
    match v {
        Value::String(s) if s.chars().any(|c| c.is_control() || c == '\u{FFFE}' || c == '\u{FFFF}') => {
            Err(format!("{path}: control characters are not allowed"))
        }
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
    fn fit_rejects_glyph_wider_than_box() {
        assert!(fit("검토", 16.0, 4.5, 2).err().unwrap().contains("does not fit"));
        assert!(fit("검토", 16.0, -8.0, 2).is_err());
    }

    #[test]
    fn controls_rejected_and_wide_glyphs_counted() {
        let m = serde_json::json!({"a": "x\u{1}y"});
        assert!(opt_str(m.as_object().unwrap(), "a", "$", 40).unwrap_err().contains("control"));
        let nc = serde_json::json!({"a": "x\u{FFFF}"});
        assert!(opt_str(nc.as_object().unwrap(), "a", "$", 40).unwrap_err().contains("control"));
        assert_eq!(width("😀", 10.0), 10.0);
        assert_eq!(width("⭐✅", 10.0), 20.0);
        assert_eq!(arrow_id("#9CA3AF"), "wb-arrow-9CA3AF");
        assert_eq!(width("W", 10.0), 10.0);
    }

    #[test]
    fn theme_values_load_and_night_differs() {
        let d = Theme::load(None).unwrap();
        let n = Theme::load(Some("night")).unwrap();
        assert_eq!(d.accent, "#2563EB");
        assert_ne!(d.bg, n.bg);
        assert!(d.font.starts_with("Pretendard"));
    }
}
