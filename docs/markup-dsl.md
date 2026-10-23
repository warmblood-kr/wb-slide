# Slide markup: design note (issue #23)

Authoring reference (for LLMs writing decks): [markup-dsl-guide.md](markup-dsl-guide.md).

**Goal.** An author, often an LLM, writes small semantic markup. A slide
preset supplies the opinions: type scale, palette, grid, spacing. A
deterministic compiler lowers the markup to the typed spec
([slide-spec.md](slide-spec.md)). The spec stays the only contract, so the
existing validation, escaping and render checks apply unchanged.

```text
markup ──compile (pure, deterministic)──▶ typed spec v1 ──export──▶ HTML (+ preset CSS) ──render.py──▶ PNG/PDF
```

## Prior art

| Source | Lesson taken |
|---|---|
| Marp | Plain text plus one theme CSS gives good results. Defaults belong in a stylesheet, not in the author's text. |
| reveal.js | `<section>` per slide works, but authors still hand-tune layout. Too low-level on its own. |
| Slidev | Frontmatter `layout:` names intent and the layout decides placement. Semantic names win. |
| Typst | Set rules (`#set text(...)`) apply defaults once and content stays clean. This is our preset. |
| Engelbart H-LAM/T | Human plus language, artifacts, methodology and training. The language should fit the human (or LLM). Tools carry the mechanical work: layout, fit and repair. |
| Alexander | Structure-preserving steps. Each tag is one small transformation that keeps the existing spec whole. Grow one tag at a time. |

## Three candidate syntaxes

**A. Custom HTML elements (chosen)**
```html
<wb-slide><wb-title>Heading</wb-title>hello</wb-slide>
```

**B. Plain HTML with classes**
```html
<section class="slide"><h1>Heading</h1><p>hello</p></section>
```

**C. Markdown with directives (Marp-like)**
```md
---
# Heading
hello
```

**Why A.**
- It matches the owner's examples, and HTML tokenizers parse it with no new grammar. html5gum is already a dependency.
- The vocabulary is closed. Any unknown `wb-*` tag or attribute is an error with a location. B hides intent in free-form classes, and C has ambiguous whitespace and inline HTML.
- `wb-` tags are already the component vocabulary (`wb-panel`, `wb-columns`), so each future tag maps 1:1 to a spec block.
- It reads naturally to LLMs, and plain tags stay greppable for repair tools.

## The slide preset and themes (first-class)

The preset has two layers:

- **Role styles**, in `framework/presets/slide.css`, set title, body, safe area
  and gap. They read only `--wb-*` tokens and map them onto the theme contract
  ([theme-contract.md](theme-contract.md)), so existing components follow the
  theme too.
- **A theme**: a swappable set of token values covering fonts, type scale,
  palette, spacing and per-role styles. Each theme is a file of
  `--wb-name: value;` lines. The built-in themes are `themes/default.css` (the design
  study's light "paper" look; `paper` is an alias), `classic.css` (the earlier
  white default), `night.css`, `graphite.css` (dark) and `signal.css` (bold).

`wb-slide export --markup deck.html [--theme default|classic|night|graphite|signal|tokens.css]` emits
the default tokens, then the chosen theme's tokens (the later value wins), then
the role styles. A token file can override only some tokens.

Theme files use a closed vocabulary. Token names must exist in the default
theme. Values are plain, with no functions, `@`, braces, comments or double quotes, so a
theme cannot load URLs or inject rules. The limit is 8 KiB. Built-in themes are
checked by the same parser. The plain `--spec` output is unchanged.

| Token | default (paper) | classic | night | graphite | signal |
|---|---|---|---|---|---|
| `--wb-bg` / `--wb-accent` | #F5F2EA / #C2410C | #FFFFFF / #2563EB | #111827 / #F59E0B | #0B0C0E / #7C83F0 | #3B2BDB / #D7FF3A |
| `--wb-title-size` / `-weight` | 40px / 600 | 32px / 700 | 36px / 800 | 40px / 600 | 44px / 700 |
| `--wb-body-size` / `--wb-center-size` | 20 / 64px | 20 / 40px | 22 / 44px | 20 / 64px | 22 / 72px |
| `--wb-title-tracking` / `-leading` | -0.02em / 1.1 | normal / 1.25 | normal / 1.25 | -0.02em / 1.1 | -0.02em / 1.05 |
| `--wb-measure` (body width under a title) | 60% | 100% | 100% | 60% | 60% |
| `--wb-vertical` (`start`, `safe center` or `space-evenly`; others are errors) | start | safe center | safe center | start | start |

Every theme sets all tokens; see the files for the rest (`--wb-font` is Pretendard in all).

At render time `render.py` substitutes the operator's approved font file. Pass
a Pretendard variable `.ttf` (e.g. `PretendardVariable.ttf`) to get Pretendard in PNG/PDF.
The preset sets `word-break: keep-all`, so Korean wraps between words.

The study themes (`default`/`paper`, `graphite`, `signal`) follow the design study's principles:
- a warm off-white or dark ground;
- one accent;
- tight titles (`--wb-title-tracking`, `--wb-title-leading`);
- body text capped at a 60% measure (`--wb-measure`);
- the body hung from the top axis (`--wb-vertical: start`).
`--wb-label-size` sizes card labels. `--wb-stat-size` and `--wb-inverse-bg`/`--wb-inverse-text` are reserved
for the stat and section patterns (next PRs); no rule reads them yet. `classic` and `night` keep the earlier look.

**Alternate theme example.** The `night` theme is built in (`--theme night`).
A custom theme is a token file. This one overrides only some tokens and
inherits the rest from the default:

Saved as `brand.css`:

```css
--wb-accent: #E11D48;
--wb-title-color: #881337;
--wb-title-size: 34px;
```

```sh
wb-slide export --markup deck.html --theme brand.css -o deck.html
```

Rendered samples: `samples/markup/default.png`, `classic.png`, `night.png`, `graphite.png`, `signal.png`, and
`centered.png` (a bare slide).

## Semantics

- A file is one or more `<wb-slide>` elements. Only whitespace and comments may appear between them.
- `<wb-title>` (optional, first child) becomes the slide `heading`, and the slide uses the title-top layout.
- `<wb-subtitle>` (optional, directly after `<wb-title>`) becomes `subtitle`. It is sized by `--wb-subtitle-size` and coloured with `--wb-muted`.
- `<wb-list>` contains only `<wb-item>` elements, each holding text. It becomes a spec `list` block, placed in order among the paragraphs. The bullet marker uses `--wb-accent`.
- `<wb-card label="…">` holds text paragraphs. It becomes a spec `panel` block, with the optional label as `label`. The preset styles it with `--wb-card-bg`, `--wb-border`, `--wb-radius` and an accent-coloured label.
- `<wb-columns>` holds 2–3 `<wb-column>` elements. Each column takes the same body content as a slide: text, lists and cards. It becomes a spec `columns` block. The spec enforces the column count and forbids nesting, reporting a field path. The preset gives columns equal width across the full content area, which fixes the width shrink in #22.
- `<wb-callout tone="info|warning">` holds text paragraphs and becomes a spec `callout` block; `tone` is required, and a value other than `info`/`warning` is left to the spec, which rejects it at `…tone` (the same path the blank-label check uses), so the markup front-end does not duplicate the allowlist. `<wb-stat value="+38%" note="…">caption</wb-stat>` becomes a spec `kpi` block: the one-line caption is the kpi `label`, so the figure's meaning sits in the content, not an attribute. It needs a close tag. Put 2–3 stats in `<wb-column>`s for a row of figures. The preset sizes the figure from `--wb-title-size` in `--wb-accent`, and tints callouts from `--wb-accent` with `--wb-radius`.
- `<wb-image src="img/a.svg" alt="…" />` becomes a spec `image` block. It may be self-closing, and `src` and `alt` are required. `src` resolves relative to the markup file, under the same local-only rules as the spec. The preset limits its height with `--wb-image-height`. Inline `<svg>` is not accepted, because passing raw SVG through would need a sanitizer; put the SVG in a file instead.
- `<wb-footer>text</wb-footer>` sits at deck level, outside any slide, and may appear at most once. It becomes the spec's root `footer`, shown on every slide in `--wb-muted`.
- `<wb-notes>text</wb-notes>` sits directly inside a `<wb-slide>`, at most once. It takes plain text only (whitespace collapses, entities decode) and becomes that slide's spec `notes`: hidden on the slide, listed in the render manifest. A line holding only `<wb-notes>` does not split a paragraph. Notes stay readable in the HTML source, so keep secrets out.
- A slide without `<wb-title>` shows its body centered on both axes (`<wb-slide>hello</wb-slide>`). It maps to a spec slide without `heading`, which renders with `slide-section`. The preset sizes this text with `--wb-center-size`.
- Remaining text becomes body. Whitespace collapses as in HTML, and blank-line-separated runs become separate `text` blocks.
- Deck `title` is the first heading. If no slide has a heading, it is the first slide's first text.
- Markup input is limited to 64 KiB and 8 levels of nesting. Both limits are checked before compiling, and the spec's own limits still apply afterwards.
- Entities are decoded. Text stays literal: the spec escapes it, and no HTML passes through.

## Next slices (one tag each)

None queued. Speaker notes (`wb-notes`) are done.
