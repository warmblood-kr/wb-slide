# Migration Guide: v0.6 → v0.7

v0.7 changes how slide layouts are rendered. Previously, layouts were JavaScript
Web Components rendered client-side in the browser. Starting with v0.7, layouts
are HTML templates rendered server-side by wb-slide (Rust). This eliminates a
class of intermittent rendering bugs (especially in PDF export and `file://`
contexts) and makes the exported HTML self-contained without depending on
JavaScript execution timing.

This is a breaking change for **custom layouts** (in themes or your own deck's
`layouts/` directory). Slide content (`slides.md`), built-in layouts, and CSS
all keep working as before.

## TL;DR — Do I need to change anything?

| You have | Action needed |
|---|---|
| Just `slides.md` using built-in layouts | Nothing. Update wb-slide, you're done. |
| A `styles/custom.css` | Nothing. |
| A theme from the official registry (e.g. `monocle-brochure`) | Nothing. The registry is already updated. |
| A custom theme you built | Convert your `.js` layouts to `.html` templates. See below. |
| Custom `.js` layouts in your deck's `layouts/` directory | Convert to `.html`, or leave as `.js` (still works as an "escape hatch" with known limitations). |

## Why the change

In v0.6 and earlier, layouts were JS Web Components like:

```js
class SlideFeature extends SlideBase {
  layoutTemplate(content, slots) {
    const heading = this.getAttribute('heading') || '';
    return `<div><h1>${heading}</h1>${content}</div>`;
  }
}
customElements.define('slide-feature', SlideFeature);
```

The browser parsed each `<slide-feature>` element, upgraded it via JS, then
ran `connectedCallback()` to apply the layout. This worked, but with edge cases:

- **PDF export / `@media print`**: the browser snapshots the DOM at a point
  where Custom Elements may not have upgraded yet, leaving raw `<slide-feature>`
  tags visible as text.
- **HTML round-trip**: when `el.innerHTML = body` is set and then read back in
  `connectedCallback`, the browser parses then re-serializes the content.
  Complex content (inline SVG, deeply nested HTML) doesn't always round-trip
  cleanly — some slides rendered correctly, others showed garbled HTML.
- **`file://` protocol**: Custom Element upgrades can be delayed or skipped in
  some browsers depending on script timing.

v0.7 sidesteps all of this by rendering layouts in Rust before the HTML is sent
to the browser. Slides are pure static HTML; the only client-side JS handles
keyboard navigation and viewport scaling.

## What changed

### Layout format: `.js` → `.html` templates

A layout used to be a JavaScript class. Now it's an HTML template with mustache-
style placeholders.

#### Before (v0.6)

`layouts/slide-feature.js`:
```js
class SlideFeature extends SlideBase {
  layoutTemplate(content, slots) {
    const heading = this.getAttribute('heading') || '';
    const subtitle = this.getAttribute('subtitle') || '';
    return `
      <div class="ms-feature-layout">
        <h1 class="ms-slide-title">${heading}</h1>
        ${subtitle ? `<p class="ms-slide-subtitle">${subtitle}</p>` : ''}
        <div class="ms-slot-area">${content}</div>
      </div>
    `;
  }
}
customElements.define('slide-feature', SlideFeature);
```

#### After (v0.7)

`layouts/slide-feature.html`:
```html
<div class="ms-feature-layout">
  <h1 class="ms-slide-title">{{heading}}</h1>
  {{#subtitle}}<p class="ms-slide-subtitle">{{subtitle}}</p>{{/subtitle}}
  <div class="ms-slot-area">{{{content}}}</div>
</div>
```

### Template syntax cheat sheet

| Mustache syntax | Old JS equivalent | Use |
|---|---|---|
| `{{name}}` | `${this.getAttribute('name')}` (HTML-escaped) | Insert an attribute value as text |
| `{{{name}}}` | `${this.getAttribute('name')}` (raw) | Insert without HTML escaping |
| `{{{content}}}` | `${content}` | Insert the default slot (slide body) |
| `{{slot:name}}` | `${slots.name || ''}` | Insert a named slot |
| `{{#name}}...{{/name}}` | `${name ? '...' : ''}` | Render block if `name` is non-empty |
| `{{^name}}...{{/name}}` | `${!name ? '...' : ''}` | Render block if `name` is empty |

### Chrome (watermark, footer, page number)

Previously the chrome was added by `SlideBase.connectedCallback()`. Now it's
added automatically by wb-slide around every layout that opts in. By default,
chrome is shown. To opt out (e.g. for a cover slide), add a frontmatter block
at the top of the template:

```html
---
chrome: false
---
<div class="ms-cover-layout">{{{content}}}</div>
```

### Theme manifest (`theme.json`)

Update file paths from `.js` to `.html`:

```json
{
  "name": "my-theme",
  "version": "1.0.0",
  "wb-slide": ">=0.7.0",
  "layouts": [
    "layouts/slide-foo.html"
  ],
  "styles": [
    "styles/theme.css"
  ]
}
```

If you list a `.js` file, wb-slide will skip it with a warning. To keep a `.js`
layout intentionally (escape hatch), list it explicitly — the slide will use
the legacy Custom Element path with the caveats above.

### Output HTML structure

The HTML that wb-slide now produces uses plain `<div>` containers:

```html
<!-- v0.7 -->
<div id="monocle-slide-deck">
  <div class="ms-viewport">
    <div class="ms-slide-container slide-feature active" data-slide-index="1">
      <div class="ms-watermark">My Brand</div>
      <div class="ms-feature-layout">
        <h1 class="ms-slide-title">Hello</h1>
        ...
      </div>
      <div class="ms-page-number">1</div>
    </div>
    ...
  </div>
</div>
```

There are no `<slide-feature>`, `<slide-cover>`, etc. Custom Elements anywhere.

## How to migrate

### Case 1: A custom theme

1. For each `layouts/*.js` file in your theme:
   a. Read the `layoutTemplate(content, slots)` method.
   b. Translate the template literal into mustache syntax (see cheat sheet).
   c. Save as `layouts/<same-name>.html`.
   d. Delete the `.js` file.
2. Update `theme.json` to list the new `.html` paths.
3. Bump `wb-slide` to `">=0.7.0"` in `theme.json`.
4. Commit and (if hosted on GitHub Pages) wait for it to publish.

### Case 2: A custom layout in your deck's `layouts/` directory

Same as a theme layout — translate the JS template into an HTML template
file. Or leave the `.js` file in place if it must be JavaScript; the slide
will use the Custom Element path with the original limitations.

### Case 3: Anything else

You don't need to do anything. Update wb-slide and you're done.

```bash
# macOS / Linux
wb-slide update

# Or reinstall fresh
curl -fsSL https://raw.githubusercontent.com/warmblood-kr/wb-slide/main/install.sh | sh
```

## Why an escape hatch?

Some layouts genuinely need JavaScript at render time — for example, a layout
that animates between states, listens to events, or computes values that
templates can't express. For those rare cases, you can still ship a `.js`
layout (which renders client-side as a Custom Element) and accept the trade-offs.

For static layouts (which is almost all of them), `.html` templates are
strictly better: simpler, safer (auto-escaping), and immune to the upgrade-
timing bugs.

If both `slide-foo.html` and `slide-foo.js` exist for the same name, the
`.html` takes precedence.

## What didn't change

- `slides.md` syntax. Frontmatter, slot syntax (`::slot-name::`), markdown
  rendering: all the same.
- Built-in layout names. `slide-cover`, `slide-feature`, `slide-section`,
  `slide-two-column`, `slide-image-full`, `slide-quote`, `slide-contact`,
  `slide-default` — same names, same attribute and slot contracts.
- CSS files. Your `styles/*.css` works unchanged. The theme contract tokens
  (`--color-accent`, `--font-family`, etc.) are unchanged.
- Theme registry URL format and `wb-slide theme show/install/list` commands.
- Frontmatter fields: `title`, `theme`, `layout`, `watermark`, `footer`, etc.
- All keyboard shortcuts and CLI flags.

## Verifying the migration

After updating, run:

```bash
wb-slide show
```

Open the page source in your browser's devtools and search for `<slide-`. If
you find no matches, your deck is fully SSR'd and immune to the v0.6 timing
bugs. If you see `<slide-foo>` elements, those are layouts using the `.js`
escape hatch.

Then test PDF export by pressing `Ctrl+P` (or `wb-slide export`) and verify all
slides render correctly in the print preview.

## Getting help

If you hit a migration issue, open an issue at
https://github.com/warmblood-kr/wb-slide/issues — please include:
- Your `layouts/*.js` file contents
- The behavior you expected vs. what you see
- Output of `wb-slide --version`
