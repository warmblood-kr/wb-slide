# Slide Format

## File Structure

Write a `slides.md` file. Each slide is separated by `---`.

```markdown
---
title: Deck Title
watermark: Top Right Text
footer: <b>Author Name</b>
layout: slide-cover
---

Cover slide content.

---
layout: slide-feature
heading: Feature Title
subtitle: One-line description.
---

Body content (HTML or Markdown).
```

## Global Frontmatter

The first `---` block sets deck-wide defaults and the first slide's layout
attributes (for example `heading`, `subtitle`, and custom template keys).
`title` and `theme` are deck-only. Other first-block keys are passed to the
first slide; `watermark` and `footer` also remain defaults for later slides.
Both LF and Windows CRLF line endings are supported.

| Key | Description |
|-----|-------------|
| `title` | Presentation title (appears in browser tab) |
| `watermark` | Text shown top-right on every slide |
| `footer` | HTML shown bottom-left on every slide |
| `layout` | Layout for the first slide |

## Per-Slide Frontmatter

| Key | Description |
|-----|-------------|
| `layout` | Layout name (e.g., `slide-feature`) |
| `heading` | Slide title (for `slide-feature`) |
| `subtitle` | Slide subtitle (for `slide-feature`) |
| `quote` | Quote text (for `slide-quote`) |
| `author` | Attribution (for `slide-quote`) |

Any additional key is available to the layout template as `{{key}}` (or as an
HTML attribute for a local JavaScript layout). Metadata uses flat `key: value`
lines; nested YAML is not supported.

## Content

Slide body content supports both HTML and Markdown.

HTML is passed through as-is:

```html
<img src="assets/photo.png" class="screenshot-frame" />
```

Markdown is rendered server-side:

```markdown
## Key Points

- First point
- Second point with **bold**
- Third point with [link](https://example.com)
```

You can mix both in the same slide.
