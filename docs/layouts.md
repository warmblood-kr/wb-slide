# Layouts

## Built-in Layouts

### `slide-cover`

Title/cover slide. No watermark, footer, or page number.

```
+--------------------------------------------------+
|                                                    |
|                                                    |
|              [centered content]                    |
|                                                    |
|                                                    |
+--------------------------------------------------+
```

```markdown
---
layout: slide-cover
---

# My Presentation

Subtitle goes here
```

### `slide-feature`

The most common layout. Title + subtitle at the top, content below.

```
+--------------------------------------------------+
| Heading                           Monocle AI      |
| Subtitle text                                      |
|                                                    |
|              [content area]                        |
|                                                    |
| Warmblood                                    4     |
+--------------------------------------------------+
```

**Attributes:** `heading`, `subtitle`

```markdown
---
layout: slide-feature
heading: Feature Name
subtitle: One-line description of the feature.
---

<img src="assets/screenshot.png" class="screenshot-frame" />
```

### `slide-section`

Section divider. Large centered text.

```
+--------------------------------------------------+
|                                     Monocle AI     |
|                                                    |
|              [centered content]                    |
|                                                    |
| Warmblood                                    3     |
+--------------------------------------------------+
```

```markdown
---
layout: slide-section
---

# Section Title
```

### `slide-default`

Generic padded content area.

```
+--------------------------------------------------+
| +----------------------------------------------+ |
| |                                     M. AI     | |
| |  [content with 40px padding]                  | |
| |                                               | |
| |                                               | |
| | Warmblood                                 2   | |
| +----------------------------------------------+ |
+--------------------------------------------------+
```

```markdown
---
layout: slide-default
---

Any content here.
```

### `slide-two-column`

Two-column grid layout with named slots.

```
+--------------------------------------------------+
| Heading (optional)                  Monocle AI     |
| Subtitle (optional)                                |
| +---------------------+  +---------------------+  |
| |  ::left::           |  |  ::right::          |  |
| |                     |  |                     |  |
| +---------------------+  +---------------------+  |
| Warmblood                                    5     |
+--------------------------------------------------+
```

**Attributes:** `heading` (optional), `subtitle` (optional)

**Slots:** `left`, `right`

```markdown
---
layout: slide-two-column
heading: Comparison
---

::left::

## Before

- Manual process
- Error-prone
- Slow

::right::

## After

- Automated
- Reliable
- Fast
```

### `slide-contact`

Contact information layout. Left-aligned, top-to-bottom.

```
+--------------------------------------------------+
|                                     Monocle AI     |
|  [content, left-aligned]                           |
|                                                    |
|                                                    |
| Warmblood                                   15     |
+--------------------------------------------------+
```

```markdown
---
layout: slide-contact
---

## Contact

**Company Name**

https://example.com

hello@example.com
```

### `slide-image-full`

Full-bleed image. No watermark, footer, or page number.

```
+--------------------------------------------------+
|                                                    |
|           [image fills entire slide]               |
|                                                    |
+--------------------------------------------------+
```

```markdown
---
layout: slide-image-full
---

<img src="assets/hero-photo.jpg" />
```

### `slide-quote`

Blockquote with optional attribution.

```
+--------------------------------------------------+
|                                     Monocle AI     |
|                                                    |
|    | "Quote text goes here."                       |
|    |                                               |
|    -- Author Name                                  |
|                                                    |
| Warmblood                                   10     |
+--------------------------------------------------+
```

**Attributes:** `quote`, `author`

```markdown
---
layout: slide-quote
quote: The best way to predict the future is to invent it.
author: Alan Kay
---
```

---

## Slots

Slots let you place content into specific areas of a layout.

### Syntax

Use `::slot-name::` on its own line to start a named slot:

```markdown
---
layout: slide-two-column
---

::left::

Content for the left column.

::right::

Content for the right column.
```

Content before the first `::slot::` marker goes to the default slot.

### Available Slots by Layout

| Layout | Slots | Description |
|--------|-------|-------------|
| `slide-two-column` | `left`, `right` | Left and right columns |
| All others | (default only) | Content goes to the main content area |

Custom layouts can define their own slots.

---

## Custom Layouts

Create a `layouts/` directory in your slide deck. Drop any `.js` file there.

### Example: Highlight Layout

```javascript
// layouts/slide-highlight.js
class SlideHighlight extends SlideBase {
  layoutTemplate(content, slots) {
    const color = this.getAttribute('color') || '#2563EB';
    return `
      <div style="padding: 60px; height: 100%; background: ${color}; color: white;">
        ${content}
      </div>
    `;
  }

  showChrome() {
    return false;
  }
}

customElements.define('slide-highlight', SlideHighlight);
```

Use it:

```markdown
---
layout: slide-highlight
color: #1B4332
---

# Key Takeaway

The most important point of the presentation.
```

### Example: Layout with Custom Slots

```javascript
// layouts/slide-demo.js
class SlideDemo extends SlideBase {
  layoutTemplate(content, slots) {
    const heading = this.getAttribute('heading') || '';
    return `
      <div style="padding: 40px; height: 100%; display: flex; flex-direction: column;">
        ${heading ? `<h1 class="ms-slide-title">${heading}</h1>` : ''}
        <div style="flex: 1; display: grid; grid-template-columns: 1fr 1fr; gap: 20px;">
          <div>${slots.code || ''}</div>
          <div>${slots.preview || content}</div>
        </div>
      </div>
    `;
  }
}

customElements.define('slide-demo', SlideDemo);
```

Use it:

```markdown
---
layout: slide-demo
heading: Live Demo
---

::code::

\`\`\`python
print("Hello, world!")
\`\`\`

::preview::

![Screenshot](assets/demo-output.png)
```

### Layout API

| Method | Default | Description |
|--------|---------|-------------|
| `layoutTemplate(content, slots)` | Wraps in `<div>` | Returns HTML. `content` = default slot. `slots` = named slot object. |
| `showChrome()` | `true` | Return `false` to hide watermark, footer, page number. |

Attributes from frontmatter: `this.getAttribute('heading')`, etc.
