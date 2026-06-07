# Slide Design Principles

> Concrete, implementation-ready specifications for the `consulting-base` theme
> and all derived themes. Where `consulting-style-research.md` defines the
> *what*, this doc defines the *exactly how*: grid, spacing scale, type ramp,
> color application, information hierarchy.

All dimensions are given for the wb-slide canvas: **960×540 px (16:9)**.

---

## Part 1 — Visual Design System

### 1.1 The grid

A 12-column grid with a 24px outer margin and 16px gutter.

```
canvas: 960 × 540
outer margin (gutter to edge):  24 px (all sides)
content area:                   912 × 492 px
columns:                        12 (each 60 px) + 11 gutters (each 16 px)
column width:                   60 px
gutter:                         16 px
```

| Region | Vertical position | Height |
|---|---|---|
| Top chrome (tracker) | y = 16 → 36 | 20 px (tiny breadcrumb) |
| Title block | y = 40 → 100 | 60 px (1–2 lines @ 24pt) |
| Sub-title (optional) | y = 100 → 120 | 20 px |
| Body content | y = 130 → 488 | ~358 px |
| Bottom chrome (source + page#) | y = 492 → 524 | 24 px |
| Outer margin bottom | y = 524 → 540 | 16 px |

The body region is the only one that varies between layouts. Title and chrome
positions are fixed across the theme — gives every slide the same eye-resting
points.

### 1.2 Spacing scale (8-pt grid)

All spacing is a multiple of 4 (half-step) or 8 (full step):

```
xs   = 4 px
sm   = 8 px
md   = 16 px
lg   = 24 px
xl   = 32 px
2xl  = 48 px
3xl  = 64 px
```

CSS:
```css
:root {
  --space-xs: 4px;
  --space-sm: 8px;
  --space-md: 16px;
  --space-lg: 24px;
  --space-xl: 32px;
  --space-2xl: 48px;
  --space-3xl: 64px;
}
```

### 1.3 Typographic scale

Built on a **1.25 modular ratio** (major third). All sizes are in points (pt)
but rendered as rem in CSS for scalability.

| Token | Size | Weight | Use |
|---|---|---|---|
| `text-tracker` | 9pt / 0.75rem | 500, +1.2 letter-spacing | Tracker, page number |
| `text-source` | 9pt / 0.75rem | 400, italic | Source line, footnotes |
| `text-caption` | 10pt / 0.83rem | 400 | Captions under elements |
| `text-body-sm` | 11pt / 0.9rem | 400 | Small body, table cells |
| `text-body` | 13pt / 1.0rem | 400 | Default body |
| `text-body-lg` | 15pt / 1.15rem | 400 | Slightly larger body |
| `text-bucket-label` | 11pt / 0.85rem | 600, uppercase, +1.5 ls | "BUCKET 1" headers |
| `text-bucket-heading` | 14pt / 1.05rem | 600 | Within bucket title |
| `text-subtitle` | 14pt / 1.1rem | 400 | Slide subtitle line |
| `text-title` | 24pt / 1.85rem | 600 | Action title |
| `text-section` | 38pt / 2.85rem | 700 | Section divider |
| `text-display` | 56pt / 4.2rem | 300 | Big idea statement |
| `text-stat-hero` | 96pt / 7.5rem | 200 (hairline) | Stat hero number |

Rules:
- Line height: 1.15 for titles, 1.4 for body, 1.0 for stat-hero
- Title weight is the heaviest commonly used; reserve 700 for section dividers only
- Stat hero uses *light* weight (200–300) because the size carries the emphasis
- Captions and body text never go below 9pt — anything smaller fails the glance test

CSS:
```css
:root {
  --font-tracker:        0.75rem;
  --font-source:         0.75rem;
  --font-caption:        0.83rem;
  --font-body-sm:        0.9rem;
  --font-body:           1.0rem;
  --font-body-lg:        1.15rem;
  --font-bucket-label:   0.85rem;
  --font-bucket-heading: 1.05rem;
  --font-subtitle:       1.1rem;
  --font-title:          1.85rem;
  --font-section:        2.85rem;
  --font-display:        4.2rem;
  --font-stat-hero:      7.5rem;

  --weight-hairline:  200;
  --weight-light:     300;
  --weight-regular:   400;
  --weight-medium:    500;
  --weight-semibold:  600;
  --weight-bold:      700;

  --leading-tight:  1.15;
  --leading-normal: 1.4;
  --leading-none:   1.0;
}
```

### 1.4 Color application rules (not just palette)

A palette is meaningless without rules for *where* each color goes. We enforce
**4 color roles**, regardless of the theme variant:

| Role | Used for | Approximate % of slide |
|---|---|---|
| `--color-text` (charcoal/black) | All standing text, axis labels | 60–70% of inked area |
| `--color-text-muted` (mid gray) | Sources, captions, secondary text, dividers | 10–15% |
| `--color-border` (light gray) | Subtle separators, table borders, footer rule | 5–10% |
| `--color-accent` (firm color) | Action title, emphasis, one chart series, callouts | 10–15% |

**Forbidden:**
- More than one accent color on a body slide (excluding charts).
- Accent color on body text (loud).
- Accent on the page background (overwhelms).
- A second accent for the title vs body — keep them visually consistent.

**Charts:**
- Single highlighted series in accent color.
- All other series in graduated grays (`#1F1F1F` → `#9A9A9A`, 3–4 stops).
- No rainbow categorical scales.
- Negative values: `--color-negative: #C53030` (semantic red), reserved for this purpose only.

### 1.5 Lines, shapes, and iconography

| Element | Style |
|---|---|
| Hairline / divider | 1 px solid `--color-border` |
| Frame border | 1 px solid `--color-border`, 4 px radius |
| Emphasis box | 2 px solid `--color-accent`, 0 px radius (sharp = serious) |
| Arrow stroke | 1.5 px, `--color-text-muted` |
| Icon stroke (Lucide-style) | 1.5 px, `currentColor` |
| Icon size (inline) | 16 × 16 px (text) or 20 × 20 px (button-like) |
| Icon size (feature) | 32 × 32 px or 40 × 40 px (concept slides) |
| Corner radius | 0 px (consulting), 4 px (neutral), 8 px (friendly) |

The `consulting-base` theme uses **0 px corners** — sharp, serious, structured.
Derived friendly themes can override to 4 or 8.

### 1.6 Visual hierarchy (where the eye lands first)

Slides read in a Z-pattern by default. The four hot spots are:

```
┌──────────────────────────────────────┐
│ ① ────────────────────────────── ②  │  ← top: title + tracker
│                                       │
│                                       │
│  ④ ────────────────────────────── ③  │  ← bottom: source + page#
└──────────────────────────────────────┘
```

Order of design importance:
1. **①** Action title — must be the heaviest visual element on the page
2. **③** Page number + logo — tertiary, but anchors the eye for navigation
3. **②** Tracker — tertiary, orients but doesn't compete
4. **④** Source — least visual weight; readable but recedes

Body content lives in the implied center but should still align to either
left margin or column structure. Centered bodies are reserved for
`slide-cover`, `slide-section`, `slide-quote-pull`, `slide-big-idea`,
`slide-stat-hero`.

### 1.7 Whitespace philosophy

Negative space is *not* leftover. We target ~30% of the canvas to be empty.

Concrete rules:
- Title block always has at least 32 px below it before body starts.
- Any two body elements have ≥ 16 px between them.
- Chart legends are inline with the chart, not stacked far away.
- Bullet lists indent 24 px; nested bullets indent another 24 px.
- A slide that *looks* full is too full.

---

## Part 2 — Information Design Principles

### 2.1 MECE and Pyramid (Minto)

Body content should obey:
- **MECE**: Mutually Exclusive, Collectively Exhaustive. The 3 buckets in a
  three-bucket layout cover everything *once*, no overlap.
- **Pyramid**: Title is the conclusion; body is the supporting arguments; finer
  detail lives in footnotes or sub-slides.

What this means for templates:
- `slide-three-bucket` enforces an N=3 structure. If your content is 5, you
  need a different layout (`slide-five-bucket` not yet provided — likely too
  dense — use 2 slides).
- Body elements should never need a "etc." or "...and more" at the end. If they
  do, the bucket count is wrong.

### 2.2 Information hierarchy

Every slide has at most 3 levels of textual hierarchy:

```
Level 1: Action title       (text-title,    semibold,  --color-text)
Level 2: Bucket / section   (text-body-lg,  semibold,  --color-accent)
Level 3: Body / caption     (text-body,     regular,   --color-text)
                            (text-source,   regular,   --color-text-muted)
```

If you need a 4th level, the slide is too dense — split it.

### 2.3 Chunking (rule of 3)

Working memory holds 7±2 items, comfortably 3–4. Layouts default to **3 buckets**:
- `slide-three-bucket` — exactly 3
- `slide-process-flow` — typically 3–5 steps; 4 is the sweet spot
- `slide-stat-hero` — exactly 1

Never offer more than 5 parallel items in one slide. Use a follow-up slide.

### 2.4 Chart and data-viz principles (Tufte-inspired)

For any chart embedded in `slide-chart-frame`:

**Data-ink ratio**: maximize the ratio of pixels that encode data to total
pixels. Strip everything else.

- **Remove**: gridlines (or hairline gray), redundant legends, 3D effects,
  drop shadows, chart titles (use the slide title), category icons.
- **Keep**: axis labels (concise), data labels on key points only, one
  highlighted series, source line, scale indicator if non-zero baseline.
- **Annotate**: arrows or text callouts pointing at the insight the title claims.
- **Color**: single accent for the highlighted series, grays for context.
- **Scale**: always start y-axis at 0 unless the deception is intentional
  (and called out).

Bar charts: bars wider than gaps. Line charts: line weight 2 px, data points
4 px circles only at notable values. Pie charts: avoid; use bar or stacked bar.

### 2.5 Reading patterns and focal points

Consulting decks are read in 3 modes:
- **Print/portfolio**: linear, top-to-bottom, time spent on body.
- **Live presentation**: eyes follow speaker's pointer; title scanned, body
  decoded on the fly.
- **Skim** (the most common): glance the title only.

Design implication: **the title must independently convey the message.**
If a reader only reads titles in sequence, they get the full deck argument.
This is the "executive read" test.

### 2.6 Information density tolerance

Density tolerance varies by audience:

| Audience | Density | Implication |
|---|---|---|
| C-suite / 5-min review | Low | Title + 1 stat or 1 diagram. Lots of whitespace. |
| Board / investors | Medium | Title + chart + brief context. |
| Team / working session | High | Title + structured content + sources. Tables OK. |
| Detailed analysis appendix | Very high | Tables, dense charts, footnotes. |

`consulting-base` defaults to **medium density**. Themes can adjust by
changing spacing tokens (`--space-md` and friends).

### 2.7 Color as information

When color carries meaning, it must be consistent across the deck:

| Semantic | Hex (default theme) | When used |
|---|---|---|
| Highlight / focus | `--color-accent` | The one thing the slide wants you to see |
| Positive / growth | `#1F8A3E` (green) | Positive deltas, improvements |
| Negative / decline | `#C53030` (red) | Negative deltas, risks, losses |
| Neutral / steady | `--color-text-muted` | No-change baseline |
| Forecast / projection | dashed line, accent | Future / uncertain |

Never use accent for "good" and red for "bad" simultaneously. Pick one
encoding system per deck.

### 2.8 The 3-second glance test

Show any slide for 3 seconds, then ask: "What was that slide about?"

A passing slide allows a viewer to answer with the action title.
A failing slide leaves them recalling the dominant visual but not the point.

Templates should be designed so the title is *unmissable* in 3 seconds:
- Title is the heaviest type weight on the page.
- Title is in the top 18% of the canvas (above eye-rest fold).
- Title is left-aligned (Z-pattern entry point).
- No competing element above or beside the title.

---

## Part 3 — Patterns across all templates

### 3.1 Consistent chrome rendering

Every template (except cover, section, image-full) includes:

```html
<header class="slide-chrome-top">
  <div class="tracker">{tracker}</div>
</header>
<footer class="slide-chrome-bottom">
  <div class="source">{source}</div>
  <div class="page-number">{index} / {total}</div>
</footer>
```

Themes style this; layouts do not duplicate the HTML.

### 3.2 Title block

Every body layout has:
```html
<div class="slide-title-block">
  <h1 class="action-title">{heading}</h1>
  <p class="subtitle">{subtitle}</p>  <!-- optional -->
</div>
```

Positioned at fixed y=40–120. Body region starts at y=130.

### 3.3 Body region grid

The body region (912 × 358 px) is the **only** part each layout customizes.
Layouts subdivide it using the 12-column grid:

| Layout | Body subdivision |
|---|---|
| `slide-feature` | 1 cell × 12 cols (full body) |
| `slide-three-bucket` | 3 cells × 4 cols each, gap 16 px |
| `slide-matrix-2x2` | 4 cells × 6 cols × 2 rows, gap 16 px |
| `slide-process-flow` | 4 cells × 3 cols + 4 arrows (CSS), single row |
| `slide-comparison` | 2 cells × 6 cols, center connector |
| `slide-chart-frame` | 8 cols chart + 4 cols callout |
| `slide-quote-pull` | full width, centered |
| `slide-stat-hero` | full width, vertically centered |

### 3.4 Footnote and source conventions

- Source format: `Source: <Name> (<Year>); <Method/Note>` — italicized, gray.
- Multiple sources separated by `;` not commas.
- "Note:" prefix for analyst commentary distinct from citations.
- Footnote markers: superscript numerals `¹ ² ³` inline, expanded in the
  source line.

### 3.5 Emphasis patterns

Within a body, emphasis hierarchy:
1. Color (accent only on the focal element)
2. Weight (semibold on the focal label)
3. Size (the stat is the focal element by definition)
4. Box / border (callout box with 2px accent border)

Never use italics for emphasis on slides — reserved for source lines and
quotes.

---

## Part 4 — Anti-patterns (don't ship templates that do these)

| Anti-pattern | Why it fails | Fix |
|---|---|---|
| Bullet list as the whole body | No visual structure; flunks glance test | Replace bullets with three-bucket or matrix |
| Topic title ("Revenue") | Reader has to read body to learn anything | Action title with the conclusion |
| Two equal-weight focal elements | Eye doesn't know where to land | One accent at a time |
| Multiple accent colors | Color stops meaning anything | One accent role per theme |
| Stock photos as decoration | Visual noise without information value | Drop them; use whitespace |
| Rainbow chart series | Categorical color overuse | Highlight one series, gray the rest |
| Centered body text on consulting slides | Doesn't align to grid | Left-align to col-1 |
| Footer with company name in body color | Competes with title | Mute footer to `--color-text-muted` |
| Subtitle longer than the title | Title should be the takeaway | Tighten the title; drop the subtitle |
| Bullets with `etc.` or `...` ending | Means content isn't MECE | Restructure or split |
| Tiny text (<9pt) | Fails 3-second glance test | Cut content or split slide |

---

## Part 5 — Per-layout exact specifications

The following spec assumes the canvas, grid, type ramp, and spacing scale above.
Numbers given are pixel coordinates and dimensions on the 960×540 canvas.

### 5.1 `slide-action-title`

```
Tracker:        x=824, y=16, w=120, text-tracker, right-aligned
Title block:    x=24, y=40, w=912, h=80
  Action title: text-title, --color-text, line-height 1.15
  Subtitle:     text-subtitle, --color-text-muted, mt=4
Body:           x=24, y=130, w=912, h=358
Source line:    x=24, y=492, w=600, text-source, --color-text-muted
Page number:    x=824, y=492, w=120, text-tracker, right-aligned
```

### 5.2 `slide-three-bucket`

```
Title block:    x=24, y=40, w=912, h=80 (same as action-title)
Body:           x=24, y=130, w=912, h=358
  Bucket grid:  3 cols × 1 row, gap 16
  Each bucket:  w=294, h=358
    Number badge: x=0, y=0, 28×28 px circle, --color-accent fill, white digit
    Bucket label: text-bucket-label, --color-accent, mt=44
    Heading:      text-bucket-heading, --color-text, mt=4
    Hairline rule: 1px --color-border, mt=8, mb=12
    Body content: text-body, --color-text
```

### 5.3 `slide-matrix-2x2`

```
Body:           x=24, y=130, w=912, h=358
  Matrix area:  x=80, y=160, w=720, h=308 (room for axis labels)
  Quadrant size: 360 × 154 each, 1px --color-border separators
  Axis labels:
    Y-axis label: x=24, y=314, rotated -90deg, text-bucket-label, --color-text-muted
    Y-pole top:   x=80, y=130, text-source, --color-text-muted
    Y-pole bot:   x=80, y=468, text-source, --color-text-muted
    X-axis label: x=440, y=478, text-bucket-label, --color-text-muted
    X-pole left:  x=80, y=474, text-source, --color-text-muted
    X-pole right: x=800, y=474, text-source, right-aligned
  Each quadrant content: text-body, padded 16px
```

### 5.4 `slide-process-flow`

```
Body:           x=24, y=130, w=912, h=358
  Step row:     y=200, 4 steps inline, gap auto
  Step:         circle 56×56 (1.5px stroke --color-accent, white fill,
                centered digit text-bucket-heading --color-accent)
                Label below: text-body, --color-text, semibold, centered
                Sub-caption: text-caption, --color-text-muted, centered
  Arrow:        between steps, 1.5px stroke --color-text-muted, arrowhead
```

### 5.5 `slide-stat-hero`

```
Whole canvas (no body subdivision):
  Stat number:  centered horizontally, y=180, text-stat-hero (96pt),
                weight 200, --color-accent
  Label:        centered, y=320 (below stat), text-body-lg, --color-text,
                max 2 lines, max-width 600px
  Source line:  bottom-left, x=24, y=492, text-source
```

### 5.6 `slide-comparison`

```
Body:           x=24, y=130, w=912, h=358
  Left panel:   x=24, y=130, w=432, h=358
    Header:     text-bucket-label, --color-text-muted, mb=16
    Content:    text-body
  Connector:    x=464, y=290, 32×32, arrow-right --color-text-muted
  Right panel:  x=504, y=130, w=432, h=358
    Header:     text-bucket-label, --color-accent, mb=16
    Content:    text-body
```

### 5.7 `slide-quote-pull`

```
Whole body:    x=120, y=130, w=720, h=358 (extra side margin for breathing)
  Decorative quote mark: 60pt accent, top-left of content
  Quote text:  text-display weight 300, --color-text, max-width 720
  Author:      text-bucket-label, --color-text-muted, mt=32
  Role:        text-source, --color-text-muted, mt=4
```

### 5.8 `slide-big-idea`

```
Whole body, vertically centered:
  Idea text:    text-display, weight 300, --color-text, max-width 760,
                centered horizontally, line-height 1.15
  Optional:     small em-dash + speaker name at bottom, text-source
```

### 5.9 `slide-timeline`

```
Body:           x=24, y=130, w=912, h=358
  Horizontal axis line: y=300, x=80-880, 2px --color-border
  Milestones:   evenly spaced on the line
    Dot:        12px circle, --color-accent fill
    Date:       text-source above the dot, --color-text-muted
    Label:      text-body below the dot, --color-text, max 2 lines, centered
  Today marker: optional vertical 1px dashed line at "now" position
```

### 5.10 `slide-chart-frame`

```
Body:           x=24, y=130, w=912, h=358
  Chart area:   8 cols → 592 px wide, h=358
  Callout:      4 cols → 304 px wide, h=180
    Position:   x=632, y=130 (top-right of body)
    Border:     2px solid --color-accent, no radius
    Padding:    16px
    Heading:    text-bucket-label, --color-accent, mb=8
    Body:       text-body, --color-text
  Optional reading note: text-source under callout
```

---

## Part 6 — Accessibility floor

All themes must satisfy:

| Check | Requirement |
|---|---|
| Body text contrast | ≥ 4.5:1 against background (WCAG AA) |
| Title contrast | ≥ 3:1 (WCAG AA Large) |
| Don't rely on color alone | Charts use shape + color, not color alone |
| Min font size at render | 9pt at 100% (text-tracker / text-source) |
| Focus indicators (interactive) | Keyboard-visible 2px outline in `--color-accent` |

McKinsey-dark variant must pass contrast with the dark navy background.

---

## Part 7 — How to extend this system

When adding a new layout:

1. Position the body region inside x=24..936, y=130..488 (358px tall).
2. Use the 12-column grid for sub-divisions; pick a column count that divides 12 evenly (1, 2, 3, 4, 6, 12).
3. Pick one focal element per slide; apply `--color-accent` to it only.
4. Use the type ramp; do not invent new sizes.
5. Use the spacing scale; do not invent ad-hoc paddings.
6. Title block is always at y=40–120; don't move it.
7. Chrome (tracker, source, page#) renders from theme; don't duplicate.
8. Test the 3-second glance: cover everything but the title. Does it still
   communicate the takeaway?

When adding a new color variant theme (e.g., a custom brand):

1. Override only the 6 standard tokens (`--color-accent`, `--color-text`,
   `--color-text-muted`, `--color-border`, `--color-background`, `--font-family`).
2. Verify body contrast still ≥ 4.5:1.
3. Verify the accent works for chart highlight (not the same hue as muted gray).
4. Do not override layout structures unless changing actual layout, not color.

---

## Open implementation notes

- The grid (column system) is enforced by `consulting-base` styles. Decks that
  don't want a strict grid use the looser `monocle-brochure` theme.
- A linter (`wb-slide theme validate`) could check that a theme doesn't override
  type sizes outside the ramp. Not yet implemented.
- The stat-hero size (96pt) may need a downscale to 72pt for ultra-long numbers
  ("$2,400,000"); consider an attribute `stat-size="compact"`.
