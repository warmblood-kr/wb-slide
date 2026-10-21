# Content components

Reusable building blocks for slide bodies, so you don't hand-roll nested `div`s
and inline styles (which bloat decks and break print/PDF rendering). They ship
in the framework (`framework/utilities.css`), are **theme-independent** — keyed
to the [theme-contract tokens](theme-contract.md), so they adopt the active
theme's accent and semantic colors — and are flat/restrained by default.

See them rendered: [components gallery](gallery/components/live.html).

![Components](templates/img/components.png)

## Optional static tags

`wb-slide` also accepts the following HTML-shaped tags. Rust lowers them to
ordinary `div` elements before the browser opens the deck. No Custom Element
registration, Shadow DOM, hydration or extra client script is used.

| Tag | Properties | Output |
|---|---|---|
| `wb-columns` | none | `.cols`; requires two or three `wb-column` children |
| `wb-column` | none | `.col`; must be a direct child of `wb-columns` |
| `wb-panel` | optional `label` | `.panel` with an optional `.panel-label` |
| `wb-kpi` | required `value`, `label` | `.stat` and `.stat-label`; body may supply context/source |
| `wb-callout` | `tone="info"` (default) or `"warning"` | `.info-box` or `.warn-box` |

```html
<wb-columns>
<wb-column><wb-panel label="근거"><p>가상 비교 자료입니다.</p></wb-panel></wb-column>
<wb-column><wb-callout tone="warning">확인되지 않은 조건을 함께 표시합니다.</wb-callout></wb-column>
</wb-columns>
```

Use explicit closing tags, including for an empty KPI. Attributes are text,
not HTML. Unknown tags/properties, invalid tones, missing KPI values and
unclosed components, unknown/unmatched closing tags and ordinary HTML wrappers
between columns and their parent fail `validate`, `export` and preview.
Keep component blocks on contiguous lines. Children are HTML or text; write
`<strong>` rather than assuming nested `**Markdown**` is parsed. Normal
Markdown outside these blocks and named Markdown slots continue to work.
Fenced code examples are displayed literally. Components in HTML layout
templates also expand. Legacy class authoring remains available.

Component-bearing slides are limited to 2 MiB input/output, 16 component
levels and 1,000 instances. The compiler validates component properties; it
does not sanitize trusted raw HTML, script tags or local JavaScript layouts.
These tags use the existing visual styles, not a new visual theme.

## Classes

| Class(es) | Purpose |
|---|---|
| `.cols` / `.col` | Two-column flex layout inside a slide body |
| `.panel` / `.panel-label` | Bordered content box with an optional uppercase label |
| `.info-box` / `.warn-box` | Accent / caution callout boxes (left border + tint) |
| `.step-row` / `.step-num` / `.step-content` | Numbered step list |
| `.compare-bad` / `.compare-good` | ❌ / ✅ comparison cards (red / green top border) |
| `.flow` / `.flow-step` / `.flow-connector` | Left-to-right flow diagram |
| `.cluster-grid` / `.cluster` | Wrapping grid of small cards |
| `.risk-item` | A risk row (pair with `.pos` / `.neg` for severity) |
| `.attn-bar` | Full-width accent attention / takeaway bar |
| `.mono-chip` | Inline code / keyword chip |

These complement the presentation helpers (`.kicker`, `.stat`, `.takeaway`,
`.source`, `.pos`/`.neg`, `.table-clean`) added in v0.8.0.

## Examples

```html
<!-- Two columns: a panel beside two callouts -->
<div class="cols"><div class="col"><div class="panel"><div class="panel-label">Panel</div>Grouped content.</div></div><div class="col"><div class="info-box">Heads up.</div><div class="warn-box" style="margin-top:.6rem">Be careful.</div></div></div>

<!-- Numbered steps -->
<div class="step-row"><div class="step-num">1</div><div class="step-content">First step.</div></div>

<!-- Comparison -->
<div class="cols"><div class="col"><div class="compare-bad">❌ The old way</div></div><div class="col"><div class="compare-good">✅ The new way</div></div></div>

<!-- Flow -->
<div class="flow"><span class="flow-step">Capture</span><span class="flow-connector">&rarr;</span><span class="flow-step">Render</span></div>

<!-- Attention bar + inline chip -->
<div class="attn-bar">One thing to remember.</div>
<p>Run <span class="mono-chip">wb-slide validate</span> first.</p>
```

> Keep each block on contiguous lines — **a blank line inside a raw HTML block
> ends it** (the rest renders as literal text). `wb-slide validate` flags this.
