# Typed slide input, schema version 1

Write JSON data, then call the trusted renderer. No generated Python or
JavaScript is needed. The full two-slide Korean example is
[`samples/static-components/deck.json`](../samples/static-components/deck.json),
equivalent to the existing Markdown deck in that directory.

```sh
wb-slide export --spec request.json -o /absolute/deck.html
python /installed/wb-slide/tools/render.py /absolute/deck.html \
  --output /absolute/new-preview --font-file /installed/approved-ko-variable.ttf --pdf
```

The second command runs maintained renderer code. The operator supplies the
installed renderer/browser/font, outside the JSON. See [renderer setup](rendering.md).
`--spec` conflicts with `--dir`, `--embed`, and `--refresh-themes`. Relative
output paths resolve from the working directory, not the spec directory.

```json
{
  "schema_version": 1,
  "title": "가상 검토",
  "slides": [{
    "heading": "운영 조건을 확인합니다",
    "subtitle": "기능 확인을 위한 가상 자료",
    "blocks": [{"type":"panel","label":"검토 사항",
                "paragraphs":["야간 운영 비용은 미확인입니다."]}]
  }]
}
```

Root requires integer `schema_version: 1`, `title` and `slides`. Optional
`footer` is literal text, at most 200 characters, shown bottom-left on every
slide. Optional `folio` is a short running label (literal text, 1–60
characters, not blank) shown top-left in the margin band, in small muted caps,
on every slide except the cover and section dividers. Each slide
requires `blocks` (unless it has `layout`, below); `heading` and `subtitle` are optional. A slide with a
heading uses the built-in `slide-feature` layout. A slide without one shows its
blocks centered (`slide-section`); `subtitle` then is an error. Optional
`notes` holds speaker notes (literal text, 1–2000 characters). They are never
painted: the HTML carries them inside the slide as escaped
`<aside class="wb-notes" hidden>`, and `tools/render.py` writes them to each
slide's `notes` in `manifest.json` (`""` when absent). Notes are not private:
anyone with the HTML file can read them. Unknown fields
are errors at every level.

Optional slide `layout` picks a layout variant from a closed set:

| `layout` | Framework layout | Look (preset) |
|---|---|---|
| `cover` | `slide-cover` (no chrome) | title group bottom-left, accent bar above the title |
| `section` | `slide-section` | divider title left, from mid-height, accent bar; blocks at body size |
| `quote` | `slide-quote` | pull quote: `heading` is the quoted text, `subtitle` the attribution; no blocks |
| `split` | `slide-split` | image/text split: slide `image` bleeds to `side`, head and blocks in the other half |
| `agenda` | `slide-agenda` | head in the left third, one numbered list (3–6 items) in the right two-thirds |

A slide with `layout` requires `heading`, allows `subtitle`, and may have no
blocks (`blocks` absent or `[]`, at most 64). The body starts with an escaped
heading group, `<header class="wb-head"><h1 class="wb-head-title">…</h1><p
class="wb-head-subtitle">…</p></header>` (subtitle only when present), then the
blocks and notes. Unknown values fail with
`$.slides[i].layout: expected cover or section or quote or split or agenda`; a missing heading with
`$.slides[i].heading: required for layout cover`. Without `layout`, behaviour is
unchanged.

Optional slide `number` (1–8 characters, not blank, e.g. `"01"`) is allowed only
with `layout: "section"` (`$.slides[i].number: requires layout section`). It is
emitted escaped first in the head group, `<span class="wb-head-number">01</span>`,
and shown large in the accent at the top-left. A section slide with no blocks
renders inverse (`--wb-inverse-bg`, `--wb-inverse-text`, `--wb-inverse-accent`).

`layout: "quote"` takes no blocks (`$.slides[i].blocks: not allowed with layout
quote`). Its body is `<figure class="wb-quote"><blockquote><p>heading</p></blockquote>
<figcaption class="wb-quote-by">subtitle</figcaption></figure>` (escaped; the
figcaption only with a subtitle), with no heading group. The preset sets the text at
32–40px (from `--wb-title-size`) within `--wb-measure`, hangs an accent `“` in the
left margin, sets the attribution in `--wb-label-size` muted caps, and centres the
figure vertically, slightly high.

`layout: "split"` requires a slide-level `image: {"src": "…", "alt": "…"}` (no other
keys; same validation, embedding and 8-per-deck count as an `image` block, errors at
`$.slides[i].image.src`) and allows `side`: `"left"` (default) or `"right"`
(`$.slides[i].side: expected left or right`). `image` or `side` on another layout fail
with `…: requires layout split`; a split without `image` with `$.slides[i].image:
required for layout split`. The body is `<img class="wb-split-image wb-split-left|right"
src="data:…" alt="…">` then the usual head group, blocks and notes. The preset sizes
the image 45% × 100% at the edge (`object-fit: cover`, no radius), starts the text
column at 52%, and moves footer/page number to the text side.

`layout: "agenda"` requires exactly one `list` block (`$.slides[i].blocks: expected one
list block with layout agenda`) with 3–6 items (`$.slides[i].blocks[0].items: expected
3–6 items with layout agenda`). The body is the usual head group then the list; the
preset numbers the items `01`–`06` with a CSS counter (accent, tabular), sets them at
`--wb-subtitle-size` with hairlines between, and places the head in the left third.

| Block type | Required fields | Optional fields |
|---|---|---|
| text | text | none |
| panel | paragraphs: string array | label |
| callout | tone: info/warning, paragraphs | none |
| kpi | value, label (both presentation strings) | note |
| list | items: string array (bulleted) | none |
| image | src (relative local path), alt | none |
| diagram | diagram (type), alt, data (that type's JSON model) | none |
| columns | columns: array of objects, each with blocks | none |

Columns require 2–3 entries and cannot nest. The compiler maps these objects
to the existing component styles. It performs no calculations; preserve units,
source/period and unknown conditions in the supplied labels/notes/prose.

Limits: input 64 KiB; 1–20 slides; 1–64 blocks per list; at most 64 leaf blocks
per slide across all columns; 1–16 paragraphs per panel/callout; 1–16 list items. Strings must
be nonblank and at most 2,000 Unicode characters each. Limits bound processing;
they do not establish that a slide fits or has an appropriate reading density.

All strings are literal text, escaped for their HTML context. Markdown,
template markers and HTML-looking strings are displayed as text. The spec path
does not load a nearby slides.md, local layouts/styles/JavaScript, registry
themes, URLs or imports. Existing Markdown/tag input remains supported.

The only files it reads are `image` blocks (#22). `src` must be a relative path
with no `..` and no scheme. Its resolved location, after following symlinks,
must stay inside the spec file's directory. The type must be .png, .jpg, .webp
or .svg, each file at most 2 MiB, with at most 8 images per deck. The file is
embedded as a `data:` URI in an `<img>` with the escaped `alt`, so SVG scripts
and external references do not run. Text inside an SVG uses whatever fonts the
browser has, not the render font check.

A `diagram` block draws one deterministic diagram from strict JSON
([diagram-library.md](diagram-library.md) lists the types and data models):

```json
{"type":"diagram","diagram":"quadrant","alt":"가상 우선순위 사분면",
 "data":{"quadrants":[{"title":"빠른 성과"},{"title":"주요 과제"},
                      {"title":"보류"},{"title":"재검토"}]}}
```

Export renders the SVG in-process with the deck's preset theme (`--theme` for
markup, default for `--spec`) and writes it inline as
`<svg class="wb-diagram" role="img" aria-label="…">` (escaped `alt`), so the
render font and checks see its text. No file is read. The slide preset
(`export --markup`) sizes it with `--wb-diagram-height` (300px), keeping the
aspect ratio. At most 16 diagrams
per deck. An unknown type lists the valid ones; data errors carry the block
path, e.g. `$.slides[0].blocks[0].data.quadrants[0].title: …`.

The complete request is validated before any output is published. Invalid
input returns nonzero with a field path and reason, for example:

```text
$.slides[0].blocks[0].columns[1].blocks[0].tone: expected info or warning
```

Malformed JSON reports `$` plus line/column information. Valid HTML is staged
beside the destination and renamed only after a complete write. Invalid specs
preserve existing output files. A failed replacement may require choosing a
new output path on platforms that cannot atomically replace an existing file.

This path produces HTML, PNG and PDF through the existing tools. Native PPTX
editability, DOCX and XLSX are not provided by this schema. Inspect rendered
slides and verify facts; data validation and image bounds cannot prove quality.
