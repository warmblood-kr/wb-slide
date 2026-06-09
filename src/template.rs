//! Minimal mustache-style template engine for wb-slide layout templates.
//!
//! Supports:
//! - `{{name}}`               HTML-escaped variable
//! - `{{{name}}}`             raw variable (no escape)
//! - `{{#name}}...{{/name}}`  section: renders block if `name` is non-empty
//! - `{{^name}}...{{/name}}`  inverted section: renders block if `name` is empty
//! - `{{slot:name}}`          named slot, raw (alias for `{{{slot:name}}}`)
//!
//! Whitespace is preserved literally. The engine is single-pass tokenize then
//! recursive section parsing.

use std::collections::HashMap;

/// Variables and slot content available to a template render.
#[derive(Debug, Default, Clone)]
pub struct Context {
    pub vars: HashMap<String, String>,
    pub slots: HashMap<String, String>,
}

impl Context {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, k: &str, v: impl Into<String>) -> &mut Self {
        self.vars.insert(k.to_string(), v.into());
        self
    }

    pub fn set_slot(&mut self, name: &str, html: impl Into<String>) -> &mut Self {
        self.slots.insert(name.to_string(), html.into());
        self
    }

    fn lookup_var(&self, name: &str) -> Option<&str> {
        if let Some(rest) = name.strip_prefix("slot:") {
            self.slots.get(rest).map(|s| s.as_str())
        } else {
            self.vars.get(name).map(|s| s.as_str())
        }
    }

    fn is_truthy(&self, name: &str) -> bool {
        match self.lookup_var(name) {
            None => false,
            Some(v) => !v.trim().is_empty(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Token<'a> {
    Text(&'a str),
    Var(&'a str),     // {{x}}     — escape
    Raw(&'a str),     // {{{x}}}   — no escape; also {{slot:name}}
    SecOpen(&'a str), // {{#x}}
    InvOpen(&'a str), // {{^x}}
    SecClose(&'a str), // {{/x}}
}

fn tokenize(s: &str) -> Vec<Token<'_>> {
    let mut tokens = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0usize;
    let mut text_start = 0usize;

    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'{' && bytes[i + 1] == b'{' {
            // Flush any preceding text
            if text_start < i {
                tokens.push(Token::Text(&s[text_start..i]));
            }

            // Raw triple-brace: {{{name}}}
            if i + 2 < bytes.len() && bytes[i + 2] == b'{' {
                if let Some(end) = find_triple_close(s, i + 3) {
                    let name = s[i + 3..end].trim();
                    tokens.push(Token::Raw(name));
                    i = end + 3;
                    text_start = i;
                    continue;
                } else {
                    // Malformed; treat the rest as text
                    tokens.push(Token::Text(&s[i..]));
                    return tokens;
                }
            }

            // Standard double-brace: {{...}}
            if let Some(end) = find_double_close(s, i + 2) {
                let inner = s[i + 2..end].trim();
                let tok = if let Some(rest) = inner.strip_prefix('#') {
                    Token::SecOpen(rest.trim())
                } else if let Some(rest) = inner.strip_prefix('^') {
                    Token::InvOpen(rest.trim())
                } else if let Some(rest) = inner.strip_prefix('/') {
                    Token::SecClose(rest.trim())
                } else if inner.starts_with("slot:") {
                    // slot references default to raw
                    Token::Raw(inner)
                } else {
                    Token::Var(inner)
                };
                tokens.push(tok);
                i = end + 2;
                text_start = i;
                continue;
            } else {
                tokens.push(Token::Text(&s[i..]));
                return tokens;
            }
        }
        i += 1;
    }

    if text_start < bytes.len() {
        tokens.push(Token::Text(&s[text_start..]));
    }
    tokens
}

fn find_double_close(s: &str, from: usize) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut i = from;
    while i + 1 < bytes.len() {
        if bytes[i] == b'}' && bytes[i + 1] == b'}' {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn find_triple_close(s: &str, from: usize) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut i = from;
    while i + 2 < bytes.len() {
        if bytes[i] == b'}' && bytes[i + 1] == b'}' && bytes[i + 2] == b'}' {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

pub fn render(template: &str, ctx: &Context) -> String {
    let tokens = tokenize(template);
    let mut out = String::with_capacity(template.len());
    render_tokens(&tokens, 0, ctx, &mut out);
    out
}

/// Render tokens starting at `start`. Returns the index just after the last
/// token consumed at this nesting level.
fn render_tokens(tokens: &[Token], start: usize, ctx: &Context, out: &mut String) -> usize {
    let mut i = start;
    while i < tokens.len() {
        match tokens[i] {
            Token::Text(s) => out.push_str(s),
            Token::Var(name) => {
                if let Some(v) = ctx.lookup_var(name) {
                    out.push_str(&html_escape(v));
                }
            }
            Token::Raw(name) => {
                if let Some(v) = ctx.lookup_var(name) {
                    out.push_str(v);
                }
            }
            Token::SecOpen(name) => {
                // Find matching close (respecting nesting)
                let close = find_section_close(tokens, i + 1, name);
                if ctx.is_truthy(name) {
                    render_tokens(tokens, i + 1, ctx, out);
                }
                i = close;
            }
            Token::InvOpen(name) => {
                let close = find_section_close(tokens, i + 1, name);
                if !ctx.is_truthy(name) {
                    render_tokens(tokens, i + 1, ctx, out);
                }
                i = close;
            }
            Token::SecClose(_) => {
                return i; // bubble up
            }
        }
        i += 1;
    }
    i
}

fn find_section_close(tokens: &[Token], from: usize, name: &str) -> usize {
    let mut depth: i32 = 1;
    let mut i = from;
    while i < tokens.len() {
        match tokens[i] {
            Token::SecOpen(n) | Token::InvOpen(n) if n == name => depth += 1,
            Token::SecClose(n) if n == name => {
                depth -= 1;
                if depth == 0 {
                    return i;
                }
            }
            _ => {}
        }
        i += 1;
    }
    tokens.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_text() {
        let ctx = Context::new();
        assert_eq!(render("hello world", &ctx), "hello world");
    }

    #[test]
    fn test_variable_substitution() {
        let mut ctx = Context::new();
        ctx.set("name", "Alice");
        assert_eq!(render("Hello {{name}}!", &ctx), "Hello Alice!");
    }

    #[test]
    fn test_html_escape() {
        let mut ctx = Context::new();
        ctx.set("x", "<script>alert('x')</script>");
        let out = render("{{x}}", &ctx);
        assert!(out.contains("&lt;script&gt;"));
        assert!(!out.contains("<script>"));
    }

    #[test]
    fn test_raw_no_escape() {
        let mut ctx = Context::new();
        ctx.set("x", "<b>bold</b>");
        assert_eq!(render("{{{x}}}", &ctx), "<b>bold</b>");
    }

    #[test]
    fn test_missing_var_empty() {
        let ctx = Context::new();
        assert_eq!(render("Hello {{name}}!", &ctx), "Hello !");
    }

    #[test]
    fn test_section_truthy() {
        let mut ctx = Context::new();
        ctx.set("x", "yes");
        assert_eq!(render("{{#x}}shown{{/x}}", &ctx), "shown");
    }

    #[test]
    fn test_section_empty_string() {
        let mut ctx = Context::new();
        ctx.set("x", "");
        assert_eq!(render("{{#x}}shown{{/x}}", &ctx), "");
    }

    #[test]
    fn test_section_missing() {
        let ctx = Context::new();
        assert_eq!(render("{{#x}}shown{{/x}}", &ctx), "");
    }

    #[test]
    fn test_inverted_section() {
        let ctx = Context::new();
        assert_eq!(render("{{^x}}no{{/x}}", &ctx), "no");

        let mut ctx2 = Context::new();
        ctx2.set("x", "y");
        assert_eq!(render("{{^x}}no{{/x}}", &ctx2), "");
    }

    #[test]
    fn test_nested_var_in_section() {
        let mut ctx = Context::new();
        ctx.set("title", "Hello");
        ctx.set("show", "1");
        assert_eq!(
            render("{{#show}}<h1>{{title}}</h1>{{/show}}", &ctx),
            "<h1>Hello</h1>"
        );
    }

    #[test]
    fn test_nested_sections() {
        let mut ctx = Context::new();
        ctx.set("a", "1");
        ctx.set("b", "1");
        let tpl = "{{#a}}A{{#b}}B{{/b}}A2{{/a}}";
        assert_eq!(render(tpl, &ctx), "ABA2");
    }

    #[test]
    fn test_slot_lookup() {
        let mut ctx = Context::new();
        ctx.set_slot("left", "<p>left content</p>");
        assert_eq!(
            render("col: {{slot:left}}", &ctx),
            "col: <p>left content</p>"
        );
    }

    #[test]
    fn test_slot_section() {
        let mut ctx = Context::new();
        ctx.set_slot("left", "x");
        let tpl = "{{#slot:left}}has left{{/slot:left}}";
        assert_eq!(render(tpl, &ctx), "has left");
    }

    #[test]
    fn test_real_layout_template() {
        let tpl = r#"<div class="ms-feature-layout">
  <h1 class="ms-slide-title">{{heading}}</h1>
  {{#subtitle}}<p class="ms-slide-subtitle">{{subtitle}}</p>{{/subtitle}}
  <div class="ms-slot-area">{{{content}}}</div>
</div>"#;
        let mut ctx = Context::new();
        ctx.set("heading", "Multi Vendor");
        ctx.set("subtitle", "");
        ctx.set("content", "<img src=\"foo.png\" />");
        let out = render(tpl, &ctx);
        assert!(out.contains("<h1 class=\"ms-slide-title\">Multi Vendor</h1>"));
        assert!(!out.contains("ms-slide-subtitle"));
        assert!(out.contains("<img src=\"foo.png\" />"));
    }

    #[test]
    fn test_unmatched_open_renders_literal() {
        let ctx = Context::new();
        assert_eq!(render("{{x", &ctx), "{{x");
    }
}
