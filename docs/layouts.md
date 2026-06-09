# Layouts

A layout is the visual frame around a slide's content — title position, body
grid, where the watermark and page number go. wb-slide ships eight built-in
layouts and you can author your own.

Since v0.7, layouts are **HTML templates** rendered server-side. (For the
escape hatch — `.js` layouts rendered client-side — see [Escape hatch](#escape-hatch).)

## Built-in layouts

| Layout | Description |
|---|---|
| `slide-cover` | Centered content. No chrome (watermark/footer/page number hidden). For title slides. |
| `slide-section` | Large centered text. For chapter dividers between sections. |
| `slide-feature` | Title + optional subtitle on top, content area below. The everyday "title + body" slide. |
| `slide-two-column` | Two-column body with `::left::` / `::right::` slots. Optional title above. |
| `slide-image-full` | Full-bleed image. No chrome. |
| `slide-quote` | Blockquote rendering. Use `quote` and `author` frontmatter fields. |
| `slide-contact` | Left-aligned content. Good for contact / sources / endnotes. |
| `slide-default` | Padded body with chrome but no title structure. Fallback for unknown layouts. |

### Common frontmatter

Most layouts accept these attributes in the slide frontmatter:

| Field | Used by | Description |
|---|---|---|
| `layout` | all | Which layout to use. Default: `slide-default`. |
| `heading` | `slide-feature`, `slide-two-column` | Slide title. |
| `subtitle` | `slide-feature`, `slide-two-column` | Optional sub-title shown below the heading. |
| `quote` | `slide-quote` | Quotation text. |
| `author` | `slide-quote` | Attribution. |
| `watermark` | all (chrome layouts) | Top-right corner text. Often set globally in the deck's first frontmatter block so every slide inherits. |
| `footer` | all (chrome layouts) | Bottom-left HTML (e.g. logo, brand name). |

The watermark, footer, and page number are automatically added to every layout
unless the layout opts out with `chrome: false` in its template frontmatter.

## Template format

A layout template is an HTML file with mustache-style placeholders. Save it as
`<layout-name>.html` either in your deck's `layouts/` directory or in a theme's
`layouts/` directory.

### Placeholders

| Syntax | Meaning |
|---|---|
| `{{name}}` | Insert the attribute `name`, HTML-escaped. |
| `{{{name}}}` | Insert the attribute `name` without escaping (use for trusted HTML). |
| `{{{content}}}` | Insert the slide's body content (raw HTML, post-markdown). |
| `{{slot:name}}` | Insert the named slot's content (e.g. `::left::` body). |
| `{{#name}}…{{/name}}` | Render the block only if `name` is non-empty. |
| `{{^name}}…{{/name}}` | Render the block only if `name` is empty. |

### Example: a minimal layout

`layouts/slide-greeting.html`:

```html
<div class="slide-greeting">
  <h1>{{heading}}</h1>
  {{#subtitle}}
  <p class="muted">{{subtitle}}</p>
  {{/subtitle}}
  <div class="body">{{{content}}}</div>
</div>
```

Use it from `slides.md`:

```markdown
---
layout: slide-greeting
heading: Hello, world!
subtitle: A friendly opening
---

This is the body content.
```

### Frontmatter on templates

A template can declare metadata in its own frontmatter block:

```html
---
chrome: false
---
<div class="cover">{{{content}}}</div>
```

Supported fields:

| Field | Default | Meaning |
|---|---|---|
| `chrome` | `true` | When `false`, the watermark, footer, and page number are not added around the layout. |

### Examples from the built-ins

`slide-cover.html`:

```html
---
chrome: false
---
<div class="ms-cover-layout">{{{content}}}</div>
```

`slide-feature.html`:

```html
<div class="ms-feature-layout">
  <h1 class="ms-slide-title">{{heading}}</h1>
  {{#subtitle}}<p class="ms-slide-subtitle">{{subtitle}}</p>{{/subtitle}}
  <div class="ms-slot-area">{{{content}}}</div>
</div>
```

`slide-two-column.html`:

```html
<div class="ms-two-column-outer">
  {{#heading}}<h1 class="ms-slide-title">{{heading}}</h1>{{/heading}}
  {{#subtitle}}<p class="ms-slide-subtitle">{{subtitle}}</p>{{/subtitle}}
  <div class="ms-two-column-layout">
    <div class="ms-col">{{#slot:left}}{{slot:left}}{{/slot:left}}{{^slot:left}}{{{content}}}{{/slot:left}}</div>
    <div class="ms-col">{{slot:right}}</div>
  </div>
</div>
```

The two-column layout uses an interesting pattern: if a `left` slot exists, use
it; otherwise fall back to the default content. The right column always comes
from the `right` slot.

## Resolution precedence

When a slide says `layout: slide-foo`, wb-slide searches in this order:

1. **Deck-local** — `<deck>/layouts/slide-foo.html` (or `.js`)
2. **Theme** — layouts listed in the theme's `theme.json`
3. **Built-in** — embedded in the wb-slide binary

The first match wins. To override a built-in, drop a same-named `.html` into
your deck's `layouts/`.

If both `slide-foo.html` and `slide-foo.js` exist at the same level, the
`.html` wins.

If the name doesn't match any layout, wb-slide falls back to `slide-default`.

## Slots

Slots let a slide body provide multiple regions of content to a layout. They're
declared in `slides.md` using the `::slot-name::` syntax:

```markdown
---
layout: slide-two-column
heading: Plan
---

::left::

## Before

Some bullet points about the current state.

::right::

## After

What we want it to look like.
```

In the layout, reference each slot with `{{slot:left}}` (or `{{slot:right}}`).
Slots are raw HTML — wb-slide already runs the slot contents through the
markdown renderer before passing them to the template.

The default slot (anything before the first `::name::` marker) is available as
`{{{content}}}`.

## Escape hatch

If a layout genuinely needs JavaScript at render time — for example, to compute
a dynamic value, run an animation, or wire up event handlers — you can still
ship a `.js` Custom Element. wb-slide will emit `<slide-foo>` for that layout
and rely on the browser to upgrade it.

### Trade-offs

`.js` layouts have known issues that `.html` templates avoid:

- May fail to upgrade in PDF export contexts, leaving raw `<slide-foo>` text.
- Round-trip serialization of complex content (especially inline SVG) may
  produce garbled output.
- The exported HTML depends on `customElements` being available — file:// URLs
  in some browsers don't run the JS.

If you can express the layout as an `.html` template, do so. The escape hatch
exists only for cases where templates aren't enough.

### Format

A `.js` layout is a class extending `SlideBase`:

```js
class SlideAnimated extends SlideBase {
  layoutTemplate(content, slots) {
    const heading = this.getAttribute('heading') || '';
    // ... arbitrary JS, can call this.querySelector, etc.
    return `<div>...${content}...</div>`;
  }

  showChrome() { return true; }
}
customElements.define('slide-animated', SlideAnimated);
```

Save as `layouts/slide-animated.js`. wb-slide bundles `SlideBase` automatically
and protects against double-registration; you don't need to import anything.

## See also

- [Slide format](./slide-format.md) — the `slides.md` syntax including frontmatter and slots
- [Styling](./styling.md) — theme tokens, utility classes
- [Theme contract](./theme-contract.md) — the standard CSS custom properties every theme should define
- [Architecture](./architecture.md) — how the SSR pipeline works
