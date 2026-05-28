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

## Commands

```bash
wb-slide show                          # Present (opens browser)
wb-slide show --port 8080              # Custom port
wb-slide show --dir path/to/deck       # Different directory

wb-slide export                        # Export to export.html
wb-slide export -o presentation.html   # Custom output name
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
