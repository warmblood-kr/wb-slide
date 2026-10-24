# wb-slide markup: authoring guide

Reference for writing wb-slide markup. Follow it exactly. Design rationale is
in [markup-dsl.md](markup-dsl.md).

## 1. Command

```sh
wb-slide export --markup deck.html [--theme default|classic|night|graphite|signal|tokens.css] -o /absolute/out.html
python tools/render.py /absolute/out.html --output /absolute/new-dir \
  --font-file /path/to/licensed-variable.ttf [--pdf] [--scale 1|2] [--browser /path/to/chrome]
```

- `export` exits 1 and prints `Error: <message>` on invalid input; existing output is kept.
- `render.py` needs a new `--output` directory. It writes one PNG per slide, optional PDF and `manifest.json`.
  Each `slides[]` entry has `text` (visible text), `notes` (`""` if none) and `overflow` (elements outside the 960×540 slide).

## 2. Tags

| Tag | Attributes | Where | Becomes |
|---|---|---|---|
| `<wb-slide>` | `layout` (optional: `cover` or `section`) | top level | one slide |
| `<wb-section number="…">Title</wb-section>` | `number` (optional; 1–8 characters, e.g. `01`) | top level; text only, one collapsed line (the title) | one section-divider slide (`layout: section`, `heading`, `number`, no blocks) |
| `<wb-quote by="…">Text</wb-quote>` | `by` (optional; attribution, e.g. `Name, role`) | top level; text only, one collapsed line (≤25 words) | one pull-quote slide (`layout: quote`, `heading` = text, `subtitle` = `by`, no blocks) |
| `<wb-split image="…" alt="…" side="…">` | `image`, `alt` (both required, non-blank), `side` (optional: `left` default, or `right`) | top level; `<wb-title>` first, then the same content as `<wb-slide>` | one image/text split slide (`layout: split`, slide `image` {`src`,`alt`}, `side`) |
| `<wb-agenda>` | none | top level; `<wb-title>` first, optional `<wb-subtitle>`, then 3–6 `<wb-item>`s and optional `<wb-notes>`; no other text or tags | one agenda slide (`layout: agenda`, one `list` block of the items) |
| `<wb-title>` | none | first child of a slide | `heading` (title-top layout) |
| `<wb-subtitle>` | none | directly after `<wb-title>` | `subtitle` |
| `<wb-list>` / `<wb-item>` | none | body; list holds only items, items hold only text (`<wb-agenda>` takes `<wb-item>`s directly) | `list` block |
| `<wb-card label="…">` | `label` (optional; if present, non-blank) | body; text only, blank lines split paragraphs (≤16) | `panel` block |
| `<wb-callout tone="…">` | `tone` (required; `info` or `warning`) | body; text only, blank lines split paragraphs (≤16) | `callout` block |
| `<wb-stat value="…">caption</wb-stat>` | `value` (required, non-blank), `note` (optional; if present, non-blank) | body; text only, one collapsed line (the caption); needs `</wb-stat>` | `kpi` block (`value`, caption → `label`, `note`) |
| `<wb-columns>` / `<wb-column>` | none | slide body; columns hold only 2–3 `<wb-column>` | `columns` block |
| `<wb-image src="…" alt="…" />` | `src`, `alt` (both required, non-blank) | body; no content; must end `/>` or have `</wb-image>` | `image` block |
| `<wb-diagram type="…" alt="…">{JSON}</wb-diagram>` | `type` (required: `quadrant`, `hub`, `cycle`, `timeline`, `fishbone`, `funnel`, `pyramid`, `flowchart`), `alt` (required, non-blank) | slide body only (not in `<wb-columns>` or `<wb-split>`); content is the diagram data as one JSON object (schemas: §6); no tags | `diagram` block (`diagram` = type, `alt`, `data`), drawn as inline SVG |
| `<wb-footer>` | none | top level, outside slides | deck `footer` on every slide |
| `<wb-folio>` | none | top level, outside slides | deck `folio`: small caps label top-left |
| `<wb-notes>` | none | directly in a slide; text only | slide `notes` (hidden) |

Body = plain text, `<wb-list>`, `<wb-card>`, `<wb-callout>`, `<wb-stat>`, `<wb-columns>`, `<wb-image>`, `<wb-diagram>`. Put 2–3 `<wb-stat>` in separate `<wb-column>`s to show figures side by side. A `<wb-column>` takes the same body content except `<wb-columns>` and `<wb-diagram>`.

## 3. Nesting and ordering rules

- Top level holds only `<wb-slide>`, `<wb-section>`, `<wb-quote>`, `<wb-split>`, `<wb-agenda>`, `<wb-footer>`, `<wb-folio>`, whitespace and comments. No stray text.
- `<wb-title>` must be the first element in a slide; `<wb-subtitle>` only right after it.
- No `<wb-title>` → the slide body is centered on both axes. A subtitle without a title is an error.
- Every slide needs body content. A title alone fails, unless the slide has `layout`.
- `layout="cover"`: title slide, no footer or page number; title and subtitle at bottom-left. `layout="section"`: section divider; title and subtitle at left from mid-height, optional short body under them. Both require `<wb-title>` first; `<wb-subtitle>` and body are optional. Use only these two values.
- `<wb-section>`: a pure divider: inverse colours (`--wb-inverse-*`), the `number` large in `--wb-inverse-accent` at the top-left, the title at left from mid-height, footer and page number kept. Text only: no child tags (no `<wb-notes>`). A `layout="section"` slide with only a title (and subtitle) gets the same inverse look; one with body keeps the deck colours.
- `<wb-quote>`: one quotation, ≤25 words, vertically centred on the left axis with an accent mark hung in the margin; `by` shows under it in small muted caps. Text only: no child tags, no quote marks of your own (the slide draws them). Use `<wb-quote>`, not `<wb-slide layout="quote">`.
- `<wb-split>`: the image fills 45% of the width at full height on `side` (edge to edge, cropped to fill, square corners); title and body sit top-aligned in the other half (from 52%). Footer and page number move to the text side. Requires `<wb-title>` first; `<wb-subtitle>`, body and `<wb-notes>` as in `<wb-slide>`. The text column is ~400px wide: keep it to a title of ≤12 words and ≤3 short paragraphs or a ≤5-item list; avoid `<wb-columns>` (too narrow). The image counts toward the 8-image limit and follows the image `src` rules below. Use a photo or full-frame picture; a wide picture gets cropped. `<wb-diagram>` is not allowed here (too narrow to read): give the diagram its own `<wb-slide>`.
- `<wb-agenda>`: title in the left third, the items in the right two-thirds, numbered `01`–`06` in the accent (the slide draws the numbers: do not write them), a hairline between items. 3–6 `<wb-item>`s, each one line of ≤8 words (a long item wraps to two lines). Put the items directly in `<wb-agenda>`, not in a `<wb-list>`. Requires `<wb-title>` first (e.g. `목차`, `Agenda`).
- `<wb-columns>`: 2–3 `<wb-column>` children, each non-empty. Columns cannot nest.
- `<wb-footer>`: at most once, at deck level.
- `<wb-folio>`: at most once, at deck level. A short running label (deck or chapter name, e.g. `2026 파일럿 리뷰`) in small muted caps at the top-left of every slide except the cover and section dividers. Use it instead of a logo or deck name repeated in each slide.
- `<wb-notes>`: at most once per slide, directly inside `<wb-slide>`, any position. Not inside cards, callouts, stats or columns.
- `<wb-callout>` `tone` is `info` (accent-tinted box) or `warning` (red edge). Any other value fails at the spec step (see §5).
- `<wb-stat>`: `value` is the figure (e.g. `+38%`), the content is its caption (keep it ≤10 words), `note` is an optional small source line.
- `<wb-diagram>`: the content is JSON, kept as written (whitespace is not collapsed; entities still decode). Never write a raw `<` in it: the tokenizer reads it as a tag. Write `\u003c` (or `&lt;`). The drawing is 16:9, `--wb-diagram-height` (300px) tall, left-aligned. Only directly in a `<wb-slide>` body: inside `<wb-columns>` or `<wb-split>` its labels would shrink below a readable size, so that is an error. Prefer it over `<wb-image>` for these 8 shapes: it follows the theme colours.
- Text collapses whitespace; a single newline joins lines into one paragraph. A blank (whitespace-only) line separates paragraphs (one `text` block each), in the body and inside `<wb-card>` and `<wb-callout>`. Text before and after a list, card, callout, stat, columns or image becomes separate blocks.
- `<wb-section>`, `<wb-quote>`, `<wb-title>`, `<wb-subtitle>`, `<wb-item>`, `<wb-stat>`, `<wb-footer>`, `<wb-folio>` and `<wb-notes>` take one collapsed line of text.
- Entities decode (`&amp;` → `&`, `&nbsp;` stays a non-breaking space). All text is literal; nothing is HTML.
- Only the tags above with only their listed attributes. Any other tag (`<p>`, `<b>`, `<svg>`…), attribute (`class`, `style`, `alt` on a card…) or self-closing tag except `<wb-image />` is an error (`<wb-stat />`, `<wb-callout />` included).
- Deck title (not shown on slides) = the first `<wb-title>` outside section, quote and agenda slides (`<wb-section>`, `layout="section"`, `<wb-quote>`, `<wb-agenda>`); else the first section or agenda title; else the first non-blank string in the first slide; else the first quote.
- Image `src` (and `<wb-split image>`): plain `dir/file.ext` under the markup file's directory. No leading `./`, no `/`-absolute path, no `..`, no `:` or `\`. The file must exist as a regular file and must not resolve (via symlinks) outside that directory. Types `.png .jpg .jpeg .webp .svg` (any case).

## 4. Limits

- Markup: 64 KiB, UTF-8, nesting depth 8. The compiled spec JSON is also capped at 64 KiB (`$: spec exceeds 64 KiB`): split a deck that hits it.
- Slides: 1–20 per deck.
- Blocks: 1–64 per slide or column (0–64 on a slide with `layout`); at most 64 leaf blocks per slide, columns included.
- List items: 1–16 (agenda: 3–6). Card and callout paragraphs: 1–16.
- Section `number`: 1–8 characters, not blank.
- Every string (title, item, paragraph, label, caption, value, note, alt, notes): 1–2000 characters, not blank.
- Footer: at most 200 characters.
- Folio: 1–60 characters, not blank (keep it ≤6 words: it is one line at 12px).
- Images: at most 8 per deck, each at most 2 MiB.
- Diagrams: at most 16 per deck (separate from the image limit). Per-type data caps: §6.
- Theme file: 8 KiB.

## 5. Errors

Markup errors (byte offsets count from the start of the file):

| Message | Fix |
|---|---|
| `unexpected <x> at byte N` | Tag not allowed there (unknown tag, wrong parent, title not first, second footer/folio/notes). Move or remove it. |
| `unexpected closing <x> at byte N` | Mismatched close tag. Close tags in reverse order. |
| `unclosed <x> at end of input` | Add the missing `</x>`. |
| `unsupported attributes or self-closing tag at byte N` | Remove the attribute not listed for that tag, or write `<x></x>` instead of `<x/>` (e.g. `<wb-stat value="…">caption</wb-stat>`, never `<wb-stat value="…"/>`). |
| `text outside <wb-slide> at byte N` | Move the text into a slide. |
| `empty <x> at byte N` | Give it text (or items/columns). |
| `<x> at byte N takes only <y> children` | Remove text directly inside `<wb-list>`/`<wb-columns>`. |
| `<wb-image> at byte N needs src="…"` / `needs alt="…"` | Add the attribute. |
| `<wb-stat> at byte N needs value="…"` / `<wb-callout> at byte N needs tone="…"` | Add the attribute. |
| `<wb-image> at byte N takes no content; use alt="…"` | Move the text into `alt`. |
| `<wb-diagram> at byte N needs type="…"` / `needs alt="…"` | Add the attribute. |
| `<wb-columns> at byte N holds a <wb-diagram>: too narrow to read; …` (or `<wb-split>`) | Move the diagram to its own `<wb-slide>`; keep the text in the columns or split. |
| `empty <wb-diagram> at byte N` | Put the JSON data inside. |
| `<wb-diagram> at byte N: invalid JSON: …` | Fix the JSON (quotes on keys and strings, commas, braces); a raw `<` becomes `\u003c`. |
| `cannot read markup: …` | Fix the input path. |
| `wb-slide[I]: body text is required` | Add body to slide I (0-based; `<wb-section>`, `<wb-quote>`, `<wb-split>` and `<wb-agenda>` slides count). |
| `wb-slide[I]: <wb-title> is required with layout at byte N` / `wb-split[I]: …` / `wb-agenda[I]: …` | Put a `<wb-title>` first in that slide, or drop `layout`. |
| `<wb-split> at byte N needs image="…"` / `needs alt="…"` | Add the attribute. |
| `<wb-agenda> at byte N takes only <wb-title>, <wb-subtitle>, <wb-item> and <wb-notes>` | Move other text or tags out; one `<wb-item>` per agenda line. |
| `no <wb-slide> found` | Add a slide. |
| `nesting deeper than 8 at byte N` / `markup exceeds 64 KiB` | Flatten or shorten. |
| `markup error at byte N: …` | Malformed syntax at N. Write a literal `<` as `&lt;`, `&` as `&amp;`. Inside `<wb-diagram>` JSON any raw `<` (even `a < b`) causes this (e.g. `invalid-first-character-of-tag-name`): write `\u003c`. |
| `markup is not UTF-8` | Save the file as UTF-8. |

Spec errors after compiling name a field path: `$.slides[I].blocks[J]…: reason`,
e.g. `$.slides[0].blocks[0].columns: expected 2–3 items`, `…type: columns cannot nest`,
`…items: expected 1–16 items`, `…: expected 1–2000 characters, not blank`,
`…label: expected 1–2000 characters, not blank` (empty `label=""`/`alt=""`), `$.footer: expected at most 200 characters`, `$.folio: expected 1–60 characters, not blank`,
`…tone: expected info or warning` (a `<wb-callout tone>` other than `info`/`warning`),
`$.slides[I].layout: expected cover or section or quote or split or agenda` (use `cover` or `section` on `<wb-slide>`; quotes use `<wb-quote>`, splits `<wb-split>`, agendas `<wb-agenda>`), `$.slides[I].blocks[0].items: expected 3–6 items with layout agenda` (give `<wb-agenda>` 3–6 `<wb-item>`s), `$.slides[I].side: expected left or right` (fix `<wb-split side>`), `$.slides[I].blocks: not allowed with layout quote` (a quote slide has no body),
`$.slides[I].subtitle: expected 1–2000 characters, not blank` (an empty `<wb-quote by="">`: fill or drop `by`),
`$.slides[I].number: expected 1–8 characters, not blank` (shorten or fill `<wb-section number>`).
Diagram errors name the data path, e.g. `$.slides[I].blocks[J].data.quadrants: …` (fix that field against §6); an unknown `type` and the 16-per-deck cap also fail here.
Image errors on `…src` (`$.slides[I].image.src` for `<wb-split image>`): `expected a relative local path without .. or a scheme`, `cannot read image: …`,
`expected .png, .jpg, .webp or .svg`, `image exceeds 2 MiB`, `expected a regular file`,
`image resolves outside the spec directory`, `at most 8 images per deck`.
Slide I is the I-th `<wb-slide>`, `<wb-section>`, `<wb-quote>`, `<wb-split>` or `<wb-agenda>` counting from 0. The render manifest counts from 1 (`index`, `slide-001.png`): manifest slide N is error slide N−1.

**Repair loop**
1. Run `export`. On a nonzero exit, read the message, fix only the named spot (byte N or slide/field path), rerun.
2. Run `render.py`. It fails with no manifest on a missing glyph or font fallback (use characters the font has). For each manifest slide with non-empty `overflow` (a list of `TAG.class` strings), shorten that slide's text or split it into two slides. Re-export and re-render to a new directory.
3. Look at the PNGs; bounds do not prove readability.

## 6. Diagrams

Placement, raw `<` and the 16-per-deck cap: §2–§4. Data rules, all types:

- One JSON object, only the fields below. Any other field fails, `title` included (the `<wb-title>` is the heading).
- Text values are nonblank one-line JSON strings. At most **40** characters: axis `label`/`low`/`high`, quadrant `title`, timeline `date`, fishbone `category`, funnel `value`, flowchart `id` and edge `label`. At most **80**: all other text.
- Figures are strings too: `"value":"12,400명"`, `"date":"2025.01"`, never `12400`.
- `highlight` (quadrant, timeline only): a JSON integer, 0-based index into the main array.
- Counts are hard limits. Text must also fit its box (shrinks to 12px, then fails): keep labels to 1–3 words.

| Type | Use when | Fields (`?` optional) | Limits |
|---|---|---|---|
| `quadrant` | four groups on two axes (2×2, SWOT) | `quadrants` [{`title`, `items`? [text]}] in order TL, TR, BL, BR; `x`?, `y`? {`label`?, `low`?, `high`?}; `highlight`? | exactly 4 quadrants; `items` 1–4 if present |
| `hub` | one centre related to many parts | `hub` text; `spokes` [text]; `arrows`? `none` (default), `out`, `in` | 3–8 spokes |
| `cycle` | a loop with no end | `steps` [{`label`, `note`?}] clockwise from the top | 3–8 steps |
| `timeline` | dated milestones in order | `events` [{`date`, `label`, `note`?}]; `highlight`? | 2–8 events |
| `fishbone` | causes of one problem | `effect` text; `causes` [{`category`, `items`? [text]}] | 2–6 causes; 0–4 items each |
| `funnel` | drop-off per stage | `stages` [{`label`, `value`?}] top first | 2–6 stages; not to scale |
| `pyramid` | ranked layers | `levels` [{`label`, `note`?}] top first | 2–6 levels |
| `flowchart` | steps with branches (no loops) | `nodes` [{`id`, `label`, `shape`? `process` (default), `decision`, `terminal`}]; `edges` [{`from`, `to`, `label`?}] (required, may be `[]`); `direction`? `LR` (default), `TB` | 2–12 nodes; 0–24 edges; unique ids; no self-loop, duplicate edge or cycle |

Minimal valid examples:

```html
<wb-diagram type="quadrant" alt="노력 대비 영향">{"x":{"label":"노력","low":"낮음","high":"높음"},"y":{"label":"영향","low":"낮음","high":"높음"},"highlight":0,"quadrants":[{"title":"빠른 성과","items":["문구 정리"]},{"title":"주요 과제"},{"title":"보류"},{"title":"재검토"}]}</wb-diagram>
<wb-diagram type="hub" alt="고객 데이터 활용 부서">{"hub":"고객 데이터","arrows":"out","spokes":["마케팅","영업","고객 지원"]}</wb-diagram>
<wb-diagram type="cycle" alt="개선 순환">{"steps":[{"label":"계획","note":"목표 정의"},{"label":"실행"},{"label":"측정"},{"label":"개선"}]}</wb-diagram>
<wb-diagram type="timeline" alt="출시 일정">{"highlight":1,"events":[{"date":"2025.01","label":"기획"},{"date":"2025.06","label":"베타","note":"20명 테스트"},{"date":"2026.01","label":"정식 출시"}]}</wb-diagram>
<wb-diagram type="fishbone" alt="가입 이탈의 원인">{"effect":"가입 이탈 증가","causes":[{"category":"사람","items":["상담 인력 부족"]},{"category":"프로세스","items":["단계 과다","인증 중복"]},{"category":"기술"}]}</wb-diagram>
<wb-diagram type="funnel" alt="전환 퍼널">{"stages":[{"label":"방문","value":"12,400명"},{"label":"가입","value":"3,100명"},{"label":"유료 전환","value":"310명"}]}</wb-diagram>
<wb-diagram type="pyramid" alt="전략 체계">{"levels":[{"label":"비전","note":"가장 신뢰받는 브랜드"},{"label":"전략 목표"},{"label":"실행 계획"}]}</wb-diagram>
<wb-diagram type="flowchart" alt="주문 처리 흐름">{"direction":"LR","nodes":[{"id":"in","label":"주문 접수","shape":"terminal"},{"id":"chk","label":"재고 있음?","shape":"decision"},{"id":"ship","label":"출고"},{"id":"buy","label":"공급사 발주"}],"edges":[{"from":"in","to":"chk"},{"from":"chk","to":"ship","label":"예"},{"from":"chk","to":"buy","label":"아니오"}]}</wb-diagram>
```

Pick the type by the "Use when" column. Anything else (comparison table, org chart, numeric chart): use `<wb-list>`, `<wb-columns>` of `<wb-card>`/`<wb-stat>`, or `<wb-image>`.

**Errors**: `$.slides[I].blocks[J].data.PATH: reason` (or `…data: reason`). Fix that field and rerun (Repair loop, §5).

| Reason | Fix |
|---|---|
| `unknown field` / `required` / `expected an object` | Remove or respell the field (no `title`) / add it / write `{…}`. |
| `expected an array of N-M entries` | Fix the count (flowchart `edges` is required, even `[]`). |
| `expected nonblank text of at most N characters` / `control characters are not allowed` | Nonblank one-line string within the cap; quote numbers. |
| `expected 0-N` (`highlight`) | Use an integer index below the array length. |
| `expected "none", "out" or "in"` / `expected LR or TB` / `expected process, decision or terminal` | Use one of the listed values. |
| `duplicate id "x"` / `unknown id "x"` / `self-loop on "x"` / `duplicate edge a -> b` | Unique ids; `from`/`to` must name a node; drop the edge. |
| `cycles are not supported; use the cycle type` | Remove the back edge, or use `cycle`. |
| `label "…" does not fit in …; shorten it` | Shorten the quoted label. |
| other `… not fit …` (e.g. `steps do not fit around the cycle`, `items do not fit in the quadrant`, `label does not fit in the band`) | Shorten texts or remove entries. |
| `too many steps in a row` / `… nodes in one rank` / `… crossing edges` | Split the flow over two slides. |
| `no clear spot along the edge; …` | Drop the edge label (edges that skip a rank have little room). |

Not data errors: `$.slides[I].blocks[J].diagram: expected one of quadrant, hub, cycle, timeline, fishbone, funnel, pyramid, flowchart` (fix `type`) and `$.slides[I].blocks[J]: at most 16 diagrams per deck` (use fewer).

## 7. Themes

`--theme default` (or omitted; warm off-white editorial, alias `paper`), a built-in `classic` (earlier white look),
`night`, `graphite` (dark) or `signal` (bold indigo), or `--theme path/to/tokens.css` (a file named like a built-in
needs a `./` prefix). default, graphite and signal hang the body from the top (`--wb-vertical: start`).
A token file is one `--wb-name: value;` per line, each ending in `;`; blank lines allowed; no comments; unset tokens keep the default.
Values must be plain: none of `( ) { } ; < > @ \ " / *`.
Errors: `FILE:LINE: expected \`--wb-name: value;\``, `FILE:LINE: unknown token NAME`, `FILE:LINE: value must be plain …`,
`FILE:LINE: --wb-vertical must be one of start, safe center, space-evenly`, `cannot read theme PATH: …`,
`theme PATH exceeds 8 KiB`, `theme PATH: not UTF-8`.

Tokens: `--wb-font`, `--wb-title-size`, `--wb-title-weight`, `--wb-title-color`,
`--wb-subtitle-size`, `--wb-body-size`, `--wb-center-size`, `--wb-text`, `--wb-muted`,
`--wb-accent`, `--wb-bg`, `--wb-card-bg`, `--wb-border`, `--wb-radius`, `--wb-safe-x`,
`--wb-safe-y`, `--wb-gap`, `--wb-image-height`, `--wb-vertical`, `--wb-title-tracking`,
`--wb-title-leading`, `--wb-measure` (max width of body text under a title),
`--wb-label-size` (also the folio size), `--wb-stat-size` (also the section number size), `--wb-inverse-bg`, `--wb-inverse-text`, `--wb-inverse-accent`, `--wb-diagram-height`
(background, text and accent of a pure section divider; keep the accent readable on the inverse background).

`--wb-vertical` accepts only `start`, `safe center` or `space-evenly`.

## 8. Example

```html
<wb-folio>Acme pilot review</wb-folio>
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
      <wb-stat value="2 weeks">to prepare</wb-stat>
      <wb-card label="Option A">Start small in one office.</wb-card>
      <wb-list><wb-item>2 weeks to prepare</wb-item><wb-item>Low cost</wb-item></wb-list>
    </wb-column>
    <wb-column>
      <wb-stat value="+38%">expected effect</wb-stat>
      <wb-card label="Option B">Roll out to both offices.</wb-card>
      <wb-list><wb-item>6 weeks to prepare</wb-item><wb-item>Larger effect</wb-item></wb-list>
    </wb-column>
  </wb-columns>
  <wb-callout tone="warning">All figures are synthetic.</wb-callout>
</wb-slide>
<wb-section number="02">What we learned</wb-section>
<wb-quote by="Jane Doe, pilot lead">Ship the smallest thing that answers the question.</wb-quote>
<wb-split image="img/site.jpg" alt="The pilot office at opening" side="left">
  <wb-title>What we saw on site</wb-title>
  Teams adopted the new flow within a week.
  <wb-list><wb-item>No extra training</wb-item><wb-item>Fewer handoffs</wb-item></wb-list>
</wb-split>
<wb-slide>Questions?</wb-slide>
```

Images: `<wb-image src="img/flow.svg" alt="Flow from markup to slides" />` (see `samples/markup/deck.html`).
