# wb-slide markup: authoring guide

Reference for writing wb-slide markup. Follow it exactly. Design rationale is
in [markup-dsl.md](markup-dsl.md).

## 1. Command

```sh
wb-slide export --markup deck.html [--theme default|night|paper|graphite|signal|tokens.css] -o /absolute/out.html
python tools/render.py /absolute/out.html --output /absolute/new-dir \
  --font-file /path/to/licensed-variable.ttf [--pdf] [--scale 1|2] [--browser /path/to/chrome]
```

- `export` exits 1 and prints `Error: <message>` on invalid input; existing output is kept.
- `render.py` needs a new `--output` directory. It writes one PNG per slide, optional PDF and `manifest.json`.
  Each `slides[]` entry has `text` (visible text), `notes` (`""` if none) and `overflow` (elements outside the 960×540 slide).

## 2. Tags

| Tag | Attributes | Where | Becomes |
|---|---|---|---|
| `<wb-slide>` | none | top level | one slide |
| `<wb-title>` | none | first child of a slide | `heading` (title-top layout) |
| `<wb-subtitle>` | none | directly after `<wb-title>` | `subtitle` |
| `<wb-list>` / `<wb-item>` | none | body; list holds only items, items hold only text | `list` block |
| `<wb-card label="…">` | `label` (optional; if present, non-blank) | body; text only, blank lines split paragraphs (≤16) | `panel` block |
| `<wb-columns>` / `<wb-column>` | none | slide body; columns hold only 2–3 `<wb-column>` | `columns` block |
| `<wb-image src="…" alt="…" />` | `src`, `alt` (both required, non-blank) | body; no content; must end `/>` or have `</wb-image>` | `image` block |
| `<wb-footer>` | none | top level, outside slides | deck `footer` on every slide |
| `<wb-notes>` | none | directly in a slide; text only | slide `notes` (hidden) |

Body = plain text, `<wb-list>`, `<wb-card>`, `<wb-columns>`, `<wb-image>`. A `<wb-column>` takes the same body content except `<wb-columns>`.

## 3. Nesting and ordering rules

- Top level holds only `<wb-slide>`, `<wb-footer>`, whitespace and comments. No stray text.
- `<wb-title>` must be the first element in a slide; `<wb-subtitle>` only right after it.
- No `<wb-title>` → the slide body is centered on both axes. A subtitle without a title is an error.
- Every slide needs body content. A title alone fails.
- `<wb-columns>`: 2–3 `<wb-column>` children, each non-empty. Columns cannot nest.
- `<wb-footer>`: at most once, at deck level.
- `<wb-notes>`: at most once per slide, directly inside `<wb-slide>`, any position. Not inside cards or columns.
- Text collapses whitespace; a single newline joins lines into one paragraph. A blank (whitespace-only) line separates paragraphs (one `text` block each), in the body and inside `<wb-card>`. Text before and after a list, card, columns or image becomes separate blocks.
- `<wb-title>`, `<wb-subtitle>`, `<wb-item>`, `<wb-footer>` and `<wb-notes>` take one collapsed line of text.
- Entities decode (`&amp;` → `&`, `&nbsp;` stays a non-breaking space). All text is literal; nothing is HTML.
- Only the tags above with only their listed attributes. Any other tag (`<p>`, `<b>`, `<svg>`…), attribute (`class`, `style`, `alt` on a card…) or self-closing tag except `<wb-image />` is an error.
- Deck title (not shown on slides) = the first `<wb-title>` in the deck; else the first non-blank string in the first slide.
- Image `src`: plain `dir/file.ext` under the markup file's directory. No leading `./`, no `/`-absolute path, no `..`, no `:` or `\`. The file must exist as a regular file and must not resolve (via symlinks) outside that directory. Types `.png .jpg .jpeg .webp .svg` (any case).

## 4. Limits

- Markup: 64 KiB, UTF-8, nesting depth 8. The compiled spec JSON is also capped at 64 KiB (`$: spec exceeds 64 KiB`): split a deck that hits it.
- Slides: 1–20 per deck.
- Blocks: 1–64 per slide or column; at most 64 leaf blocks per slide, columns included.
- List items: 1–16. Card paragraphs: 1–16.
- Every string (title, item, paragraph, label, alt, notes): 1–2000 characters, not blank.
- Footer: at most 200 characters.
- Images: at most 8 per deck, each at most 2 MiB.
- Theme file: 8 KiB.

## 5. Errors

Markup errors (byte offsets count from the start of the file):

| Message | Fix |
|---|---|
| `unexpected <x> at byte N` | Tag not allowed there (unknown tag, wrong parent, title not first, second footer/notes). Move or remove it. |
| `unexpected closing <x> at byte N` | Mismatched close tag. Close tags in reverse order. |
| `unclosed <x> at end of input` | Add the missing `</x>`. |
| `unsupported attributes or self-closing tag at byte N` | Remove the attribute not listed for that tag, or write `<x></x>` instead of `<x/>`. |
| `text outside <wb-slide> at byte N` | Move the text into a slide. |
| `empty <x> at byte N` | Give it text (or items/columns). |
| `<x> at byte N takes only <y> children` | Remove text directly inside `<wb-list>`/`<wb-columns>`. |
| `<wb-image> at byte N needs src="…"` / `needs alt="…"` | Add the attribute. |
| `<wb-image> at byte N takes no content; use alt="…"` | Move the text into `alt`. |
| `cannot read markup: …` | Fix the input path. |
| `wb-slide[I]: body text is required` | Add body to slide I (0-based). |
| `no <wb-slide> found` | Add a slide. |
| `nesting deeper than 8 at byte N` / `markup exceeds 64 KiB` | Flatten or shorten. |
| `markup error at byte N: …` | Malformed syntax at N. Write a literal `<` as `&lt;`, `&` as `&amp;`. |
| `markup is not UTF-8` | Save the file as UTF-8. |

Spec errors after compiling name a field path: `$.slides[I].blocks[J]…: reason`,
e.g. `$.slides[0].blocks[0].columns: expected 2–3 items`, `…type: columns cannot nest`,
`…items: expected 1–16 items`, `…: expected 1–2000 characters, not blank`,
`…label: expected 1–2000 characters, not blank` (empty `label=""`/`alt=""`), `$.footer: expected at most 200 characters`.
Image errors on `…src`: `expected a relative local path without .. or a scheme`, `cannot read image: …`,
`expected .png, .jpg, .webp or .svg`, `image exceeds 2 MiB`, `expected a regular file`,
`image resolves outside the spec directory`, `at most 8 images per deck`.
Slide I is the I-th `<wb-slide>` counting from 0. The render manifest counts from 1 (`index`, `slide-001.png`): manifest slide N is error slide N−1.

**Repair loop**
1. Run `export`. On a nonzero exit, read the message, fix only the named spot (byte N or slide/field path), rerun.
2. Run `render.py`. It fails with no manifest on a missing glyph or font fallback (use characters the font has). For each manifest slide with non-empty `overflow` (a list of `TAG.class` strings), shorten that slide's text or split it into two slides. Re-export and re-render to a new directory.
3. Look at the PNGs; bounds do not prove readability.

## 6. Themes

`--theme default` (or omitted), a built-in `night`, `paper` (warm light editorial), `graphite` (dark product) or
`signal` (bold indigo), or `--theme path/to/tokens.css`. The study themes hang the body from the top (`--wb-vertical: start`).
A token file is one `--wb-name: value;` per line, each ending in `;`; blank lines allowed; no comments; unset tokens keep the default.
Values must be plain: none of `( ) { } ; < > @ \ " / *`.
Errors: `FILE:LINE: expected \`--wb-name: value;\``, `FILE:LINE: unknown token NAME`, `FILE:LINE: value must be plain …`,
`FILE:LINE: --wb-vertical must be one of start, safe center, space-evenly`, `cannot read theme PATH: …`,
`theme PATH exceeds 8 KiB`, `theme PATH: not UTF-8`.

Tokens: `--wb-font`, `--wb-title-size`, `--wb-title-weight`, `--wb-title-color`,
`--wb-subtitle-size`, `--wb-body-size`, `--wb-center-size`, `--wb-text`, `--wb-muted`,
`--wb-accent`, `--wb-bg`, `--wb-card-bg`, `--wb-border`, `--wb-radius`, `--wb-safe-x`,
`--wb-safe-y`, `--wb-gap`, `--wb-image-height`, `--wb-vertical`, `--wb-title-tracking`,
`--wb-title-leading`, `--wb-measure` (max width of body text under a title), `--wb-stat-size`,
`--wb-label-size`, `--wb-inverse-bg`, `--wb-inverse-text`.

`--wb-vertical` accepts only `start`, `safe center` or `space-evenly`.

## 7. Example

```html
<wb-footer>Acme pilot review · fictional data</wb-footer>
<wb-slide>
  <wb-title>Pilot review: Q3</wb-title>
  <wb-subtitle>Fictional team, synthetic numbers</wb-subtitle>
  The pilot ran for six weeks in two offices.

  Costs &amp; outcomes are summarised below.
  <wb-notes>Say up front that all numbers are synthetic.</wb-notes>
</wb-slide>
<wb-slide>
  <wb-title>Two options</wb-title>
  <wb-columns>
    <wb-column>
      <wb-card label="Option A">Start small in one office.</wb-card>
      <wb-list><wb-item>2 weeks to prepare</wb-item><wb-item>Low cost</wb-item></wb-list>
    </wb-column>
    <wb-column>
      <wb-card label="Option B">Roll out to both offices.</wb-card>
      <wb-list><wb-item>6 weeks to prepare</wb-item><wb-item>Larger effect</wb-item></wb-list>
    </wb-column>
  </wb-columns>
</wb-slide>
<wb-slide>Questions?</wb-slide>
```

Images: `<wb-image src="img/flow.svg" alt="Flow from markup to slides" />` (see `samples/markup/deck.html`).
