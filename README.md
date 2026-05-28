# WB Slide

Lightweight slide presentation framework. Markdown in, slides out.

No npm. No bundler. Single binary.

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/warmblood-kr/wb-slide/main/install.sh | sh
```

Or download from [Releases](https://github.com/warmblood-kr/wb-slide/releases).

## Quick Start

```bash
mkdir my-deck && cd my-deck

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

Content goes here.

---
layout: slide-section
---

# Thank You
EOF

wb-slide show
```

Browser opens at `http://localhost:3030`. Navigate with arrow keys.

## How It Works

Write `slides.md` with YAML frontmatter. Each `---` starts a new slide.
The CLI reads it, renders markdown to HTML, and serves a self-contained
presentation with built-in layouts and keyboard navigation.

```
slides.md  -->  wb-slide  -->  Browser
                  |
                  +-- Parses YAML frontmatter
                  +-- Renders markdown (comrak)
                  +-- Applies Web Component layouts
                  +-- Serves via embedded HTTP server
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

| Key | Description |
|-----|-------------|
| `title` | Presentation title |
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
| `slide-feature` | Title + subtitle + content area |
| `slide-section` | Large centered text for section dividers |
| `slide-contact` | Left-aligned content for contact info |
| `slide-two-column` | Two-column grid layout |
| `slide-image-full` | Full-bleed image, no chrome |
| `slide-quote` | Blockquote with `quote` and `author` attributes |

## Commands

### `wb-slide show`

```
wb-slide show [OPTIONS]

Options:
  -p, --port <PORT>    Port [default: 3030]
  -d, --dir <DIR>      Working directory [default: .]
      --no-open        Don't open browser
```

### `wb-slide export`

```
wb-slide export [OPTIONS]

Options:
  -d, --dir <DIR>        Working directory [default: .]
  -o, --output <FILE>    Output file [default: export.html]
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `->` `v` `Space` `PageDown` | Next slide |
| `<-` `^` `PageUp` | Previous slide |
| `Home` | First slide |
| `End` | Last slide |
| `F` | Toggle fullscreen |

## Directory Convention

```
my-presentation/
  slides.md           # Slide content (required)
  styles/             # Auto-loaded: all *.css files (optional)
    custom.css
    fonts.css
  assets/             # Static files served as-is (optional)
    screenshot.png
  layouts/            # Auto-loaded: all *.js Web Components (optional)
    my-layout.js
```

The CLI auto-scans `styles/` and `layouts/` directories:

- **`styles/`** -- all `.css` files injected after framework CSS (overrides defaults)
- **`layouts/`** -- all `.js` files loaded as custom Web Component layouts
- **`assets/`** -- served as static files, referenced via relative paths

## Custom Styling

Create any `.css` file in `styles/`:

```css
:root {
  --color-accent: #FF6600;
  --font-family: 'Pretendard', sans-serif;
}
```

## Architecture

```
wb-slide (single binary, ~3MB)
  +-- Embedded: framework/     <-- Web Components + CSS (rust-embed)
  |     +-- monocle-slide.js        Core engine
  |     +-- slide-base.js           Layout base class
  |     +-- layouts/*.js            Built-in layouts (8)
  |     +-- theme.css               Default theme
  |     +-- utilities.css           Minimal utility classes
  |     +-- print.css               PDF print styles
  +-- Runtime:
        +-- Reads slides.md from working directory
        +-- Renders markdown server-side (comrak)
        +-- Auto-scans styles/ and layouts/
        +-- Serves assets from working directory
```

## License

MIT
