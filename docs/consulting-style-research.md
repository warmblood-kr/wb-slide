# Consulting & Slide:ology Style Research

> Source material for `consulting-base` theme, McKinsey/BCG/Bain color variants,
> and Tier 2 standalone layouts. Synthesizes the structural conventions of top
> management consulting decks with Nancy Duarte's *Slide:ology* principles.

## 1. The McKinsey / BCG / Bain canon

### 1.1 The action title (the single most important convention)

Consulting slides put a **full sentence conclusion as the title** — not a topic
label. The body just visually proves it.

| ❌ Topic title (legacy) | ✅ Action title (consulting) |
|---|---|
| "Revenue trends" | "Revenue grew 38% in 2025, driven by EU expansion" |
| "Market share" | "We are #2 with room to overtake the leader in 18 months" |
| "Cost structure" | "60% of costs are fixed and unmanaged" |

Properties:
- 1–2 lines, ≤ 18 words
- Past tense for what happened; present/future for recommendations
- Specific (a number, a name, a verb of consequence)
- The "so what" lives in the title; the body is evidence
- Reader can skim title-only and still get the story

### 1.2 Page chrome (consulting frame)

```
+------------------------------------------------------------+
| Action title — full sentence headline           [Tracker]  |
| Optional sub-title for nuance                              |
+------------------------------------------------------------+
|                                                            |
|        BODY: visual proof of the title (chart,             |
|        diagram, framework, table — not bullets)            |
|                                                            |
|                                                            |
+------------------------------------------------------------+
| Source: [data sources]                          [Logo]  N  |
+------------------------------------------------------------+
```

| Element | Notes |
|---|---|
| Tracker (top-right) | Tiny breadcrumb indicating section, e.g. "2. Recommendations / 2.1" |
| Source line (bottom-left) | Always cited: "Source: Internal data, McKinsey analysis" |
| Page number (bottom-right) | Always present |
| Logo (bottom-right or inline w/ page #) | Firm mark, restrained |
| Watermark (some firms) | "Confidential" or "Draft", subtle gray |

### 1.3 Body principles

- **One idea per slide.** If you need two, make two slides.
- **Visual proof, not text dump.** The body *shows* the title.
- **High structure.** Aligned grids, repeated bucket counts (rule of three).
- **Restrained color.** Color carries meaning. Black/gray for context, accent for emphasis. No rainbow charts.
- **Negative space.** ~25–30% of the slide is blank.
- **Footnotes for caveats** in tiny gray text near the relevant element.

### 1.4 The repertoire of body layouts (Tier 2 candidates)

What consultants actually use, day in day out:

| Pattern | Use case |
|---|---|
| **3-bucket** | "There are three things to fix: A, B, C" |
| **2x2 matrix** | "Position vs. Action", "Effort vs. Impact", growth-share |
| **Process flow** | "Our 4-step approach", funnel |
| **Stat hero** | "$2.4B", "38%", "1 in 3" |
| **Comparison** | Current state vs. future state; before/after |
| **Hierarchy / tree** | Org breakdown, MECE issue tree |
| **Timeline / roadmap** | Phased plan with milestones |
| **Chart frame** | Data viz with insight callout |
| **Quote / testimonial** | Customer voice |
| **Table** | Detail comparison (rare, dense) |
| **Logo wall** | Clients, partners, comparables |

## 2. Color palettes (the canonical firms)

Each firm has a near-trademarked palette. Backgrounds are usually white; the
firm color shows up in titles, accents, and emphasis.

### McKinsey
```
--mck-blue-primary:   #051C2C   /* deep navy, headers */
--mck-blue-secondary: #034B6F   /* mid blue, sub-elements */
--mck-blue-accent:    #00A9F4   /* bright cyan, highlights */
--mck-gray-700:       #4A4A4A   /* body text */
--mck-gray-500:       #757575   /* muted text, sources */
--mck-gray-300:       #D5D5D5   /* dividers, table rows */
--mck-gray-100:       #F2F2F2   /* subtle backgrounds */
--mck-background:     #FFFFFF
```
Characteristics: very dark navy dominant, blue accents only for callouts, lots of
gray, monochrome charts with one blue series highlighted.

### BCG
```
--bcg-green-primary:  #00754A   /* primary forest green */
--bcg-green-mid:      #4DAB6B   /* mid */
--bcg-green-light:    #BFE3CB   /* tints */
--bcg-charcoal:       #1F1F1F   /* text */
--bcg-gray-500:       #6E6E6E
--bcg-gray-300:       #D9D9D9
--bcg-gray-100:       #F4F4F4
--bcg-background:     #FFFFFF
```
Characteristics: distinctive forest green, more breathing room than McKinsey,
matrix-heavy ("BCG Matrix" is theirs).

### Bain
```
--bain-red-primary:   #CC0000   /* Bain crimson */
--bain-red-dark:      #8E1B25
--bain-orange-accent: #F37021
--bain-gray-800:      #2B2B2B
--bain-gray-500:      #6E6E6E
--bain-gray-200:      #E5E5E5
--bain-background:    #FFFFFF
```
Characteristics: bold red accent, headlines often in red, slightly warmer tone.

### Neutral / generic consulting
```
--cons-primary:       #1F3A5F   /* generic deep blue */
--cons-accent:        #2563EB   /* highlight */
--cons-text:          #1F2937
--cons-text-muted:    #6B7280
--cons-border:        #E5E7EB
--cons-background:    #FFFFFF
```
Safe default for `consulting-base` when no firm-specific theme is chosen.

## 3. Typography conventions

| Element | Style |
|---|---|
| Action title | 22–26pt, semi-bold or bold, dark navy/black, max 2 lines |
| Sub-title | 13–15pt, regular, muted gray |
| Section headers (sub-titles inside body) | 14–16pt, semi-bold, accent color |
| Body | 11–13pt, regular |
| Tracker | 9pt, uppercase, letter-spaced, muted |
| Source line | 8–9pt, italic or regular, muted |
| Page number | 9pt, muted |
| Numbers in stat slides | 60–120pt, light or hairline weight |

Font choices (sans-serif, neutral, high-legibility):
- Inter, Source Sans, IBM Plex Sans, Pretendard (Korean)
- McKinsey uses a proprietary "Bower" sans + "Mckinsey Sans"; Inter is a close OSS substitute
- Avoid playful or display fonts in the title position

## 4. Slide:ology principles (Nancy Duarte)

Duarte's book argues slides are **cinema, not document**. Key applicable
principles for our templates:

| Principle | Implication for layouts |
|---|---|
| Slide as "glance media" | Title + 1 dominant element. Readable in 3 seconds. |
| Big visual metaphor | Diagram-heavy layouts: hub-spoke, pyramid, Venn, journey |
| Hierarchy via contrast | Strong type scale, color used sparingly to direct the eye |
| Negative space | ~30% of slide should be empty |
| One thought per slide | Build content across many slides instead of cramming |
| Numbers as design feature | Hero stat slides — huge number, tiny context |
| Quotes as design feature | Pull quote layout, attribution as second-class element |
| Color with intent | Two or three colors, each meaning a thing |
| Progressive disclosure | (Animation — out of scope for our renderer; achieve same with sequenced slides) |

Concrete layouts these principles produce that our `consulting-base` should
include:

- **slide-stat-hero** — Duarte's "make the number the visual"
- **slide-quote-pull** — Quote as the visual, attribution small
- **slide-big-idea** — Single sentence, lots of white space
- **slide-comparison** — Before vs. after (visualize the change)
- **slide-image-statement** — Full-bleed image + overlay statement

## 5. Layout specifications (Tier 2 candidates)

The following are the layouts the `consulting-base` theme will provide. Each has
a defined attribute/slot contract so any theme can override the look without
breaking decks.

### 5.1 `slide-action-title`

The workhorse of consulting decks. Action title at top, body fills the rest.

```
+------------------------------------------------+
| ACTION TITLE — full sentence headline   2 / 14 |
| Optional sub-title for nuance                  |
+------------------------------------------------+
|                                                |
|       [slot: default — body content]           |
|       (chart, diagram, or sub-layout)          |
|                                                |
+------------------------------------------------+
| Source: ...                                 ⬡  |
+------------------------------------------------+
```

| Attribute | Required | Description |
|---|---|---|
| `heading` | yes | Action title (full sentence) |
| `subtitle` | no | Supporting nuance |
| `source` | no | Source citation, bottom-left |
| `tracker` | no | Section breadcrumb, top-right |

Slot: default (body content).

### 5.2 `slide-three-bucket`

The "rule of three" structure. Title at top, three labeled buckets below.

```
+------------------------------------------------+
| Action title                                   |
+------------------------------------------------+
| BUCKET 1     | BUCKET 2      | BUCKET 3        |
| Header       | Header        | Header          |
| ---          | ---           | ---             |
| body 1       | body 2        | body 3          |
+------------------------------------------------+
| Source                                      ⬡  |
+------------------------------------------------+
```

| Attribute | Description |
|---|---|
| `heading` | Action title |
| `source` | Source citation |

Slots: `bucket1`, `bucket2`, `bucket3`. Each bucket expects its own
sub-heading and content.

### 5.3 `slide-matrix-2x2`

The 2×2 framework. Title at top, matrix with quadrant labels.

```
+------------------------------------------------+
| Action title (e.g. "Effort vs. Impact")        |
+------------------------------------------------+
|         Y-axis label →                         |
|  high  | Q2: ...      | Q1: ...                |
|        |              |                        |
|  low   | Q3: ...      | Q4: ...                |
|        | low          | high   X-axis label →  |
+------------------------------------------------+
| Source                                      ⬡  |
+------------------------------------------------+
```

| Attribute | Description |
|---|---|
| `heading` | Action title |
| `x-axis` | X-axis label |
| `y-axis` | Y-axis label |
| `x-low` / `x-high` | X-axis pole labels |
| `y-low` / `y-high` | Y-axis pole labels |

Slots: `q1`, `q2`, `q3`, `q4` (top-right, top-left, bottom-left, bottom-right).

### 5.4 `slide-process-flow`

Horizontal stepped process with arrows.

```
+------------------------------------------------+
| Action title                                   |
+------------------------------------------------+
| ⬢ Step 1  →  ⬢ Step 2  →  ⬢ Step 3  →  ⬢ End |
| caption     caption       caption      caption |
+------------------------------------------------+
| Source                                      ⬡  |
+------------------------------------------------+
```

| Attribute | Description |
|---|---|
| `heading` | Action title |

Slot: `steps` (each step is a `<div class="step">label</div>` block;
arrows are drawn by CSS between them). Default slot can be used too.

### 5.5 `slide-stat-hero`

Slide:ology-style number-as-design.

```
+------------------------------------------------+
|                                                |
|              38%                               |
|                                                |
|              Active users grew                 |
|              quarter over quarter              |
|                                                |
+------------------------------------------------+
| Source                                      ⬡  |
+------------------------------------------------+
```

| Attribute | Description |
|---|---|
| `stat` | The big number (e.g. "38%", "$2.4B") |
| `label` | Short label (1–2 lines) |
| `source` | Source citation |

No body slot; the whole slide is the stat.

### 5.6 `slide-timeline`

Horizontal timeline with milestones.

```
+------------------------------------------------+
| Action title                                   |
+------------------------------------------------+
|  ●────●────●────●────●                        |
| 2024  2025  2026  2027  2028                  |
| Launch  ...  ...   ...   Scale                |
+------------------------------------------------+
| Source                                      ⬡  |
+------------------------------------------------+
```

| Attribute | Description |
|---|---|
| `heading` | Action title |

Slot: `milestones` (each milestone is `<div class="milestone" data-date="2025">label</div>`).

### 5.7 `slide-comparison`

Before/after, current/future, "from X to Y".

```
+------------------------------------------------+
| Action title                                   |
+------------------------------------------------+
|  BEFORE              |    →     |    AFTER     |
|  ----------          |          |    --------- |
|  body left           |          |    body right|
|                      |          |              |
+------------------------------------------------+
| Source                                      ⬡  |
+------------------------------------------------+
```

| Attribute | Description |
|---|---|
| `heading` | Action title |
| `left-label` | Left header (default: "Before") |
| `right-label` | Right header (default: "After") |

Slots: `left`, `right`.

### 5.8 `slide-quote-pull`

Duarte-style pull quote. The quote is the visual.

```
+------------------------------------------------+
|                                                |
|   "Quote text goes here, large and heavy,      |
|    occupying the visual center. Attribution    |
|    is intentionally secondary."                |
|                                                |
|              — Author Name, Title              |
|                                                |
+------------------------------------------------+
|                                             ⬡  |
+------------------------------------------------+
```

| Attribute | Description |
|---|---|
| `quote` | The quote text |
| `author` | Attribution |
| `role` | Author's role/title (optional) |

### 5.9 `slide-big-idea`

The single statement slide. Lots of negative space.

```
+------------------------------------------------+
|                                                |
|                                                |
|       One single sentence, big and bold        |
|                                                |
|                                                |
+------------------------------------------------+
|                                             ⬡  |
+------------------------------------------------+
```

| Attribute | Description |
|---|---|
| `idea` | The statement |

### 5.10 `slide-chart-frame`

Chart container with action title and insight callout. The chart itself goes in
the default slot (SVG, image, or embedded chart lib).

```
+------------------------------------------------+
| Action title                                   |
+------------------------------------------------+
|                                  ┌──────────┐  |
|     [chart in default slot]      │ Callout: │  |
|                                  │ The big  │  |
|                                  │ insight  │  |
|                                  └──────────┘  |
+------------------------------------------------+
| Source                                      ⬡  |
+------------------------------------------------+
```

| Attribute | Description |
|---|---|
| `heading` | Action title |
| `callout` | Insight text (short) |

Slot: default for the chart.

## 6. Theme structure proposal

```
wb-slide-registry/
  themes/
    consulting-base/             # neutral generic, Tier 2 layouts
      theme.json                 # provides standard tokens + new layouts
      layouts/
        slide-action-title.js
        slide-three-bucket.js
        slide-matrix-2x2.js
        slide-process-flow.js
        slide-stat-hero.js
        slide-timeline.js
        slide-comparison.js
        slide-quote-pull.js
        slide-big-idea.js
        slide-chart-frame.js
      styles/
        theme.css                # consulting chrome (tracker, source line)
        layouts.css              # per-layout styles
    mckinsey-light/              # extends consulting-base, McKinsey palette
      theme.json (extends: consulting-base)
      styles/theme.css            # palette override only
    mckinsey-dark/                # same but dark variant
    bcg/                          # BCG green palette
      theme.json (extends: consulting-base)
      styles/theme.css
    bain/                         # Bain red palette
      theme.json (extends: consulting-base)
      styles/theme.css
```

## 7. Open questions for implementation

1. **Tracker rendering**: Just a string from frontmatter, or a smart "2 / 14" from slide index? Probably both — auto when omitted.
2. **Source line vs. footnote**: We'll provide `source` attribute on each layout. Themes that want it bottom-left render it; minimalist themes can hide it.
3. **Charts**: We don't embed a chart lib by default. Slot accepts SVG/image. Later: integrate Chart.js or similar as opt-in.
4. **Sub-layouts**: Some layouts (e.g. timeline, process-flow) need iteration. Solve with markdown convention (list items become steps) or HTML structure inside default slot?

## 8. References

- Nancy Duarte, *Slide:ology* (O'Reilly, 2008)
- Gene Zelazny, *Say It With Charts* (McKinsey-derived)
- Barbara Minto, *The Pyramid Principle* (McKinsey origin)
- BCG, McKinsey, Bain public reports for visual reference
