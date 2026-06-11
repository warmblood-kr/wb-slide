# Content components

Reusable building blocks for slide bodies, so you don't hand-roll nested `div`s
and inline styles (which bloat decks and break print/PDF rendering). They ship
in the framework (`framework/utilities.css`), are **theme-independent** — keyed
to the [theme-contract tokens](theme-contract.md), so they adopt the active
theme's accent and semantic colors — and are flat/restrained by default.

See them rendered: [components gallery](gallery/components/live.html).

![Components](templates/img/components.png)

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
