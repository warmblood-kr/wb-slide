# Architecture

## Overview

```
wb-slide (single binary, ~3MB)
  +-- Embedded: framework/         Web Components + CSS (rust-embed)
  |     +-- monocle-slide.js       Core engine (<monocle-slide> element)
  |     +-- slide-base.js          Layout base class
  |     +-- layouts/*.js           Built-in layouts (8)
  |     +-- theme.css              Default theme
  |     +-- utilities.css          Minimal utility classes
  |     +-- print.css              PDF print styles
  +-- Runtime:
        +-- Reads slides.md from working directory
        +-- Renders markdown server-side (comrak)
        +-- Auto-scans styles/ and layouts/
        +-- Serves assets from working directory
```

## Pipeline

1. **Parse**: Read `slides.md`, split on `---`, extract YAML frontmatter per slide
2. **Render**: Convert markdown bodies to HTML using comrak (server-side)
3. **Assemble**: Inject slides as JSON, framework JS/CSS, and user styles/layouts into HTML template
4. **Serve**: Embedded HTTP server (axum) serves the assembled page + static assets
5. **Present**: Browser-side Web Components handle layout, keyboard navigation, and scaling

## Directory Convention

```
my-presentation/
  slides.md           # Required
  styles/*.css         # Auto-loaded after framework CSS
  layouts/*.js         # Auto-loaded as Web Component layouts
  assets/              # Served as static files
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| axum | HTTP server |
| tokio | Async runtime |
| comrak | Markdown to HTML |
| rust-embed | Embed framework files in binary |
| clap | CLI argument parsing |
| open | Open browser |

## Browser-side

The browser receives a self-contained HTML page with:
- Inlined framework CSS + user CSS
- Inlined framework JS + layout JS + user layout JS
- Slide data as JSON (`window.__MONOCLE_SLIDES__`)

The `<monocle-slide>` Web Component reads the JSON, creates layout elements,
injects pre-rendered HTML, and handles navigation/scaling. No external
dependencies are loaded at runtime.
