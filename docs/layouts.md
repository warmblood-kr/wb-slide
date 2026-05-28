# Layouts

## Built-in Layouts

| Layout | Description |
|--------|-------------|
| `slide-default` | Padded content area with chrome |
| `slide-cover` | Centered content, no watermark/footer/page number |
| `slide-feature` | Title + subtitle + content area |
| `slide-section` | Large centered text for section dividers |
| `slide-contact` | Left-aligned content for contact/info pages |
| `slide-two-column` | Two-column grid layout |
| `slide-image-full` | Full-bleed image, no chrome |
| `slide-quote` | Blockquote with `quote` and `author` attributes |

## Custom Layouts

Create a `layouts/` directory in your slide deck folder. Drop any `.js` file
there and it will be automatically loaded.

A custom layout is a Web Component that extends `SlideBase`:

```javascript
class SlideHighlight extends SlideBase {
  layoutTemplate(content) {
    const color = this.getAttribute('color') || '#FF6600';
    return `
      <div style="padding: 40px; height: 100%; background: ${color}; color: white;">
        ${content}
      </div>
    `;
  }
}

customElements.define('slide-highlight', SlideHighlight);
```

Use it in `slides.md`:

```markdown
---
layout: slide-highlight
color: #2563EB
---

# This slide has a blue background
```

### Layout API

Every layout extends `SlideBase` and can override:

| Method | Default | Description |
|--------|---------|-------------|
| `layoutTemplate(content)` | Wraps in `<div>` | Returns HTML string. `content` is the pre-rendered slide body. |
| `showChrome()` | `true` | Return `false` to hide watermark, footer, and page number. |

Frontmatter keys are accessible via `this.getAttribute('key')`.

### Chrome Elements

When `showChrome()` returns `true`, these elements are added automatically:

- `.ms-watermark` -- top-right text (from global `watermark:`)
- `.ms-footer-logo` -- bottom-left HTML (from global `footer:`)
- `.ms-page-number` -- bottom-right slide number
