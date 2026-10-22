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

Root requires integer `schema_version: 1`, `title` and `slides`. Each slide
requires `blocks`; `heading` and `subtitle` are optional. A slide with a
heading uses the built-in `slide-feature` layout. A slide without one shows its
blocks centered (`slide-section`); `subtitle` then is an error. Unknown fields
are errors at every level.

| Block type | Required fields | Optional fields |
|---|---|---|
| text | text | none |
| panel | paragraphs: string array | label |
| callout | tone: info/warning, paragraphs | none |
| kpi | value, label (both presentation strings) | note |
| list | items: string array (bulleted) | none |
| image | src (relative local path), alt | none |
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
