# Monocle Slide

Lightweight slide presentation framework. Markdown in, slides out.

No npm. No bundler. Single binary.

## Quick Start

```bash
# Build
cargo install --path .

# Create slides
cat > slides.md << 'EOF'
---
title: My Presentation
watermark: Company Name
footer: <b>Author</b>
layout: slide-cover
---

# Hello World

Welcome to my presentation.

---
layout: slide-feature
heading: Feature One
subtitle: This is the first feature.
---

<img src="assets/screenshot.png" class="screenshot-frame" />

---
layout: slide-section
---

# Thank You
EOF

# Present
monocle-slide show
```

Browser opens at `http://localhost:3030`. Navigate with arrow keys.

## How It Works

Write `slides.md` with YAML frontmatter. Each `---` starts a new slide.
The CLI reads it, renders markdown to HTML, and serves a self-contained
presentation with built-in layouts and keyboard navigation.

```
slides.md  ──→  Rust CLI  ──→  Browser
                  │
                  ├── Parses YAML frontmatter
                  ├── Renders markdown (comrak)
                  ├── Applies Web Component layouts
                  └── Serves via embedded HTTP server
```

## Slide Format

```markdown
---
title: Deck Title
watermark: Top Right Text
footer: <i>W</i><span>armblood</span>
layout: slide-cover
---

Cover slide content here.

---
layout: slide-feature
heading: Slide Title
subtitle: One-line description.
---

Slide body content (HTML or Markdown).
```

### Global Frontmatter

The first `---` block sets deck-wide defaults:

| Key | Description |
|-----|-------------|
| `title` | Presentation title (used in `<title>` tag) |
| `watermark` | Text shown top-right on every slide |
| `footer` | HTML shown bottom-left on every slide |
| `layout` | Layout for the first slide |

### Per-Slide Frontmatter

| Key | Description |
|-----|-------------|
| `layout` | Web Component tag name (e.g., `slide-feature`) |
| `heading` | Slide title (for `slide-feature`) |
| `subtitle` | Slide subtitle (for `slide-feature`) |
| Any key | Passed as HTML attribute to the layout component |

## Built-in Layouts

| Layout | Description |
|--------|-------------|
| `slide-default` | Padded content area with chrome |
| `slide-cover` | Centered content, no watermark/footer/page number |
| `slide-feature` | Title + subtitle + content area (most common) |
| `slide-section` | Large centered text for section dividers |
| `slide-contact` | Left-aligned content for contact info |
| `slide-two-column` | Two-column grid layout |
| `slide-image-full` | Full-bleed image, no chrome |
| `slide-quote` | Blockquote with attribution |

## Commands

### `monocle-slide show`

Start the presentation server.

```
monocle-slide show [OPTIONS]

Options:
  -p, --port <PORT>    Port [default: 3030]
  -d, --dir <DIR>      Working directory [default: .]
      --no-open        Don't open browser
```

### `monocle-slide export`

Export to a self-contained HTML file.

```
monocle-slide export [OPTIONS]

Options:
  -d, --dir <DIR>        Working directory [default: .]
  -o, --output <FILE>    Output file [default: export.html]
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `→` `↓` `Space` `PageDown` | Next slide |
| `←` `↑` `PageUp` | Previous slide |
| `Home` | First slide |
| `End` | Last slide |
| `F` | Toggle fullscreen |

## Directory Convention

```
my-presentation/
  slides.md           # Slide content (required)
  styles/
    custom.css         # Theme overrides, custom classes (optional)
  assets/              # Images, icons, etc. (optional)
    screenshot.png
  layouts/             # Custom layout components (optional, future)
    my-layout.js
```

## Custom Styling

Create `styles/custom.css` to override theme variables or add custom classes:

```css
:root {
  --color-accent: #FF6600;
  --font-family: 'Pretendard', sans-serif;
}

.screenshot-frame {
  border-radius: 8px;
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.12);
  max-width: 80%;
}
```

## Architecture

```
monocle-slide (Rust binary, ~3MB)
  ├── Embedded: framework/     ← Web Components + CSS (via rust-embed)
  │     ├── monocle-slide.js   ← Core engine (<monocle-slide> element)
  │     ├── slide-base.js      ← Layout base class
  │     ├── layouts/*.js       ← Built-in layouts
  │     ├── theme.css          ← Default theme
  │     ├── utilities.css      ← Minimal utility classes
  │     └── print.css          ← PDF print styles
  └── Runtime:
        ├── Reads slides.md from working directory
        ├── Renders markdown server-side (comrak)
        ├── Injects slides as JSON into HTML template
        └── Serves assets from working directory
```

## License

MIT
