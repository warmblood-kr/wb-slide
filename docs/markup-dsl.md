# Slide markup: design note (issue #23)

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
  `--wb-name: value;` lines. The built-in themes are `themes/default.css` and
  `themes/night.css`.

`wb-slide export --markup deck.html [--theme default|night|tokens.css]` emits
the default tokens, then the chosen theme's tokens (the later value wins), then
the role styles. A token file can override only some tokens.

Theme files use a closed vocabulary. Token names must exist in the default
theme. Values are plain, with no functions, `@`, braces, comments or double quotes, so a
theme cannot load URLs or inject rules. The limit is 8 KiB. Built-in themes are
checked by the same parser. The plain `--spec` output is unchanged.

| Token | default | night |
|---|---|---|
| `--wb-font` | Pretendard, system sans | same |
| `--wb-title-size` / `-weight` / `-color` | 32px / 700 / #111827 | 36px / 800 / #F9FAFB |
| `--wb-body-size` / `--wb-center-size` | 20px / 40px | 22px / 44px |
| `--wb-text` / `--wb-muted` / `--wb-accent` / `--wb-bg` | light palette | dark palette |
| `--wb-safe-x` / `--wb-safe-y` / `--wb-gap` | 64 / 48 / 16px | 72 / 56 / 20px |

At render time `render.py` substitutes the operator's approved font file. Pass
a Pretendard variable `.ttf` to get Pretendard in PNG/PDF.

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

Rendered samples: `samples/markup/default.png`, `night.png`, and
`centered.png` (a bare slide).

## Semantics

- A file is one or more `<wb-slide>` elements. Only whitespace and comments may appear between them.
- `<wb-title>` (optional, first child) becomes the slide `heading`, and the slide uses the title-top layout.
- A slide without `<wb-title>` shows its body centered on both axes (`<wb-slide>hello</wb-slide>`). It maps to a spec slide without `heading`, which renders with `slide-section`. The preset sizes this text with `--wb-center-size`.
- Remaining text becomes body. Whitespace collapses as in HTML, and blank-line-separated runs become separate `text` blocks.
- Deck `title` is the first heading. If no slide has a heading, it is the first slide's first text.
- Entities are decoded. Text stays literal: the spec escapes it, and no HTML passes through.

## Next slices (one tag each)

`wb-subtitle` → `subtitle`, `wb-list`, `wb-card` → panel, `wb-columns`,
`wb-image`/`wb-svg` (#22), `wb-notes`, `wb-footer`. Notes and footer need spec
fields.
