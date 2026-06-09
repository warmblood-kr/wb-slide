# Architecture

wb-slide is a single Rust binary that renders Markdown slide decks into HTML.
Layouts are HTML templates rendered server-side; the browser only handles
navigation and scaling. There are no required runtime dependencies (Node.js,
npm, Python, …) on the user's machine.

## The big picture

```
   slides.md, styles/, layouts/, assets/         Theme registry (GitHub Pages)
                  │                                       │
                  ▼                                       ▼
            +-----------+                          +--------------+
            |  parse    |                          | fetch theme  |
            |  YAML +   |                          | (cached)     |
            |  markdown |                          +--------------+
            +-----------+                                 │
                  │                                       │
                  └───────────────┬───────────────────────┘
                                  ▼
                          +----------------+
                          |  resolve       |
                          |  layouts       |
                          |  (LayoutSet)   |
                          +----------------+
                                  │
                                  ▼
                          +----------------+
                          |  render each   |
                          |  slide via     |
                          |  template      |
                          +----------------+
                                  │
                                  ▼
                          +----------------+
                          | assemble HTML  |
                          | + framework JS |
                          | + framework CSS|
                          +----------------+
                                  │
                                  ▼
                          serve / export

                          ┌─────────────┐
                          │   browser   │
                          │             │
                          │  navigate   │
                          │  scale      │
                          │  fullscreen │
                          └─────────────┘
```

The only client-side JavaScript is keyboard navigation and viewport scaling
(~70 lines). All layout rendering happens server-side.

## Pipeline detail

### 1. Parse `slides.md`

The deck is a Markdown file with YAML frontmatter blocks separating slides:

```markdown
---
title: My Deck
theme: monocle-brochure
---

Cover content.

---
layout: slide-feature
heading: First feature
---

Body of first feature slide.
```

`parse_slides()` returns:
- A `global_meta` map (from the first block)
- A `Vec<Slide>`, each with its own `frontmatter`, `body` markdown, and
  named `slots`

Within a slide body, the `::slot-name::` convention splits content into
named slots. Markdown is rendered to HTML for each slot using [comrak](https://github.com/kivikakk/comrak).

### 2. Resolve the theme (optional)

If the global frontmatter includes a `theme:` field, wb-slide:

1. Looks up the theme name in the registry's `index.json`
   (default: `https://warmblood-kr.github.io/wb-slide-registry/index.json`).
2. Fetches the theme's `theme.json` manifest.
3. Fetches each listed `layouts/*.html` and `styles/*.css` file.
4. Caches everything under `~/.cache/wb-slide/`.

All HTTP fetches use HTTP `If-None-Match` (etag) for revalidation with a
1-hour TTL. If the network is unreachable but the cache exists, the cached
copy is used silently. The `--refresh-themes` flag bypasses the cache.

A theme can also be specified as a direct URL to a `theme.json`.

### 3. Build the `LayoutSet`

The `LayoutSet` collects layouts from three sources, in precedence order:

| Priority | Source | Path |
|---|---|---|
| 1 (highest) | Deck-local | `<deck>/layouts/*.html` (or `.js`) |
| 2 | Theme | files listed in the theme manifest |
| 3 (fallback) | Built-in | embedded in the binary via `rust-embed` |

When a slide says `layout: slide-foo`, the `LayoutSet::resolve("slide-foo")`
returns the highest-priority match. If `.html` and `.js` both exist at the
same level, `.html` wins.

Built-ins are SSR-only `.html`. Themes and deck-local layouts can ship either
form.

### 4. Render each slide

For each `Slide`, wb-slide calls `render_slide()`:

- If the resolved layout is an `.html` template, the template engine fills in
  the placeholders (`{{var}}`, `{{{content}}}`, `{{slot:name}}`, …) using the
  slide's attributes, body HTML, and slots as the context. The result is
  plain HTML.

- If the resolved layout is a `.js` Custom Element, wb-slide emits a `<slide-foo>`
  element with the attributes and body as child HTML, plus the layout's JS
  inlined into a `<script type="module">` near the top of the document. The
  browser upgrades the element at runtime. (See [layouts.md](layouts.md#escape-hatch)
  for the trade-offs.)

Chrome elements (watermark, footer, page number) are emitted as sibling
`<div>`s around the layout body, unless the layout's template frontmatter sets
`chrome: false`.

Each rendered slide is wrapped in:

```html
<div class="ms-slide-container slide-foo active" data-slide-index="1">
  ...
</div>
```

The `active` class is initially on the first slide; the navigation JS toggles
it on the rest as the user moves around.

### 5. Assemble the page

```html
<!DOCTYPE html>
<html>
<head>
  <style>{framework CSS + theme CSS + user CSS}</style>
</head>
<body>
  <div id="monocle-slide-deck">
    <div class="ms-viewport">
      {rendered slides…}
    </div>
  </div>
  {client JS for any .js layouts (rare)}
  <script>{framework JS for navigation/scaling}</script>
</body>
</html>
```

CSS is loaded in cascade order: framework defaults → theme overrides → user
overrides. The user's `styles/*.css` therefore always wins.

### 6. Serve or export

- `wb-slide show` serves the assembled HTML on a local port via axum and
  opens the browser.
- `wb-slide export` writes the same HTML to a file. The exported HTML is
  fully self-contained — no remote fetches at view time.

## File layout

```
wb-slide/                       
├── Cargo.toml
├── src/
│   ├── main.rs                 CLI, server, command handlers
│   ├── template.rs             Mustache-style template engine
│   ├── render.rs               LayoutSet, SSR pipeline
│   └── theme.rs                Theme fetching, cache, manifest parsing
├── framework/                  Embedded into the binary via rust-embed
│   ├── monocle-slide.js        Client navigation + scaling (~70 lines)
│   ├── theme.css               Default theme + chrome positioning
│   ├── utilities.css           Minimal utility classes (.flex, .gap, …)
│   ├── print.css               @media print rules
│   └── layouts/
│       ├── slide-cover.html
│       ├── slide-feature.html
│       └── …                   (8 built-in layout templates)
└── docs/
    └── …
```

The `framework/` directory is bundled into the binary at compile time. The
binary has no filesystem dependencies on the user's machine for these assets.

## Why server-side rendering?

The earliest version of wb-slide used Web Components: each layout was a JS
class extending a `SlideBase`. The browser upgraded `<slide-foo>` elements and
ran `connectedCallback()` to apply the layout.

This worked, but with intermittent failures:

- **PDF export**: the browser sometimes snapshots the DOM before the Custom
  Element upgrade completes, leaving raw `<slide-foo>` tags in the printed
  output.
- **HTML round-trip**: setting `el.innerHTML = body` then reading it back in
  `connectedCallback` re-serializes the DOM. Complex content (especially
  inline SVG) didn't always round-trip cleanly.
- **`file://`**: some browsers delay or skip Custom Element upgrades when the
  page is loaded via `file://`.

The symptom was inconsistent: simple slides rendered fine, but slides with
diagrams or deep nesting showed garbled HTML. Moving the render step into Rust
sidesteps all of this. The browser receives plain `<div>` elements; nothing
needs to "upgrade."

The price is that custom layouts can no longer execute arbitrary JavaScript.
For the slide-layout domain — title placement, slot composition, chrome — the
mustache template subset is enough. For the rare layout that genuinely needs
JS, the `.js` escape hatch remains.

## Dependencies

Crate dependencies are deliberately minimal:

| Crate | Used for |
|---|---|
| axum, tokio, tower-http | HTTP server for `wb-slide show` |
| clap | CLI argument parsing |
| comrak | Markdown to HTML |
| rust-embed | Embed `framework/` into the binary |
| reqwest (rustls) | Fetch themes; check for updates |
| serde_json | JSON escaping for theme manifests |
| mime_guess | MIME types for served assets |
| open | Open the browser from `wb-slide show` |

There's no JavaScript runtime, no headless browser, no FFI. The compiled
binary is around 7 MB and statically links musl on Linux for portability.

## See also

- [Slide format](./slide-format.md) — `slides.md` syntax
- [Layouts](./layouts.md) — built-in layouts and authoring custom ones
- [Theme contract](./theme-contract.md) — standard CSS tokens
- [Registry design](./registry-design.md) — theme distribution model
- [Migration guide](./migration-v0.7.md) — upgrading from v0.6 to v0.7
