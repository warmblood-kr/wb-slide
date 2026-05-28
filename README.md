# WB Slide

[한국어](README.ko.md)

Lightweight slide presentation framework. Markdown in, slides out.

## Why WB Slide?

**Single binary, zero dependencies.** Download one file and you're done.
No npm, no Python, no Ruby, no build step. Nothing to install, nothing to break.
Non-developers shouldn't need a package manager to make a presentation.

**Markdown-first.** Write slides in plain text. No drag-and-drop, no proprietary format.
Easy to version control, easy to diff, easy to collaborate on.

**Standardized output.** Define layout templates once, reuse everywhere. Every deck
from your team looks consistent -- same spacing, same typography, same structure.
No more "creative" font choices on slide 7.

**AI/LLM-friendly.** Plain markdown is the format LLMs understand best.
Ask an AI to write your slides and paste the output directly into `slides.md`.
No conversion step, no copy-paste from a chat window into PowerPoint.

**Extensible when you need it.** Custom layouts are Web Components -- drop a `.js` file
in the `layouts/` folder. Custom styles go in `styles/`. The defaults work for 90% of
cases; the escape hatch is there for the rest.

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/warmblood-kr/wb-slide/main/install.sh | sh
```

Or download from [Releases](https://github.com/warmblood-kr/wb-slide/releases).

Supports macOS (Apple Silicon), Linux (x64), and Windows (x64).

## Quick Start

```bash
mkdir my-deck && cd my-deck

cat > slides.md << 'EOF'
---
title: Hello World
layout: slide-cover
---

# Hello World

---
layout: slide-feature
heading: First Slide
subtitle: Getting started with wb-slide.
---

Write content in **Markdown** or HTML.

---
layout: slide-section
---

# Thank You
EOF

wb-slide show
```

Browser opens at `http://localhost:3030`. Navigate with arrow keys.

## Commands

```bash
wb-slide show                          # Present (opens browser)
wb-slide show --port 8080              # Custom port
wb-slide show --dir path/to/deck       # Different directory

wb-slide export                        # Export to export.html
wb-slide export -o presentation.html   # Custom output name

wb-slide version                       # Check for updates
wb-slide update                        # Self-update to latest
```

## Keyboard

`->` / `Space` next | `<-` previous | `Home` / `End` first/last | `F` fullscreen

## Layouts

| Layout | Use |
|--------|-----|
| `slide-cover` | Title slide (no chrome) |
| `slide-feature` | Heading + subtitle + content |
| `slide-section` | Section divider |
| `slide-default` | Generic content |
| `slide-contact` | Contact info |
| `slide-two-column` | Two columns |
| `slide-image-full` | Full-bleed image |
| `slide-quote` | Blockquote |

Need a custom layout? Drop a `.js` file in `layouts/`. See [docs/layouts.md](docs/layouts.md).

## Customization

Override colors, fonts, or add CSS classes in `styles/`:

```css
/* styles/custom.css */
:root {
  --color-accent: #FF6600;
  --font-family: 'Pretendard', sans-serif;
}
```

## Directory Structure

```
my-deck/
  slides.md        # Slide content (required)
  styles/          # CSS overrides (auto-loaded)
  layouts/         # Custom layouts (auto-loaded)
  assets/          # Images, icons, etc.
```

## Docs

- [Slide Format](docs/slide-format.md) -- frontmatter, markdown, content rules
- [Layouts](docs/layouts.md) -- built-in layouts, creating custom layouts
- [Styling](docs/styling.md) -- theme variables, CSS utilities, print/PDF
- [Architecture](docs/architecture.md) -- how it works under the hood

## License

[MIT](LICENSE)
