# Optional PNG/PDF renderer

The Rust CLI exports HTML without Python or a browser dependency. The reviewed
`tools/render.py` helper writes one PNG per slide, optional PDF and a manifest.
Generation runs supply Markdown/data and call these commands; the Python is
maintained renderer implementation code, not model-written generation code.

## Setup

Use Python 3.11+ in a virtual environment:

```sh
python3 -m venv .venv-render
.venv-render/bin/python -m pip install -r tools/requirements-render.txt
.venv-render/bin/python -m playwright install chromium
cargo build --locked
target/debug/wb-slide validate --strict -d samples/static-components
target/debug/wb-slide export -d samples/static-components -o /tmp/compact.html
.venv-render/bin/python tools/render.py /tmp/compact.html --output /tmp/compact-render \
  --font-file /path/to/licensed/NotoSansKR-Variable.ttf --pdf
```

Choose a new output directory. `--browser /path/to/chrome` uses installed
Chromium/Chrome instead of Playwright's bundled browser. The actual version is
recorded; identical output across browser/OS versions is not promised. Viewport:
960×540; PNGs: 1920×1080 (`--scale 1` gives 960×540). PDF uses the deck's print
CSS. Inspect its page count/layout separately.

## First-slice contract

- Static HTML, 1–200 slides; resolve tags before rendering. Runtime custom
  elements are rejected. Browser exceptions/console errors, missing assets and
  network requests fail the job.
- Local assets only. Embed assets or keep HTML beside its local asset tree.
  Registry CSS retaining remote font imports must be made local first.
- Supply a licensed variable TTF with a `wght` axis (at most 32 MiB). This one
  face overrides theme fonts. Check visible DOM text against its Unicode cmap
  and weight range; reject detected system-font fallback. No runtime font fetch
  or repository font copy. Multi-face brands, icon fonts and asynchronous apps
  are outside this slice.
- Font/image readiness has a 15-second bound. Disable animations/transitions;
  fix locale, timezone, viewport and scale. No inferred async-app readiness.
- Stage outputs and publish a new directory only on success. Refuse existing
  directories. Manifest records input/renderer/font hashes, library/browser
  versions, actual fonts, text, PNG hashes and element bounds. Bounds cannot
  prove readable typography, intended overlap, clipping or lack of occlusion;
  visual review is required.

## Layout check

Each manifest slide carries `findings`: layout problems for an automated repair
loop. `--check` prints all findings as one JSON line on stdout (instead of the
"Rendered" line) and exits 3 if any exist, 0 if none; render errors still exit 1.

```json
{"slide": 1, "kind": "overflow", "element": "UL.wb-list", "text": "first 60 chars"}
```

- `overflow`: the outermost element extending past the slide box.
- `clipped`: an element with non-`visible` overflow whose content exceeds its box
  (e.g. a too-tall body in `.ms-slot-area`).
- `overlap`: two text/`img` leaf boxes intersecting by more than 2px; the other
  box is in `"with": {"element", "text"}`.

Hidden elements (zero-size boxes such as notes) are ignored. Findings are
heuristics; visual review is still required.

The helper runs trusted HTML and its scripts in Chromium. It is not a sandbox
for executable third-party templates. This slice does not generate native
Office files; PDF/PNG fidelity does not establish editability or native wrapping.

## Checks

Install Poppler's `pdfinfo` for the PDF page-count test (`poppler-utils` on
Ubuntu, `poppler` on Homebrew). The renderer itself does not require Poppler.

```sh
.venv-render/bin/python -m pip install -r tools/requirements-render-test.txt
WB_RENDER_FONT=/path/to/licensed/NotoSansKR-Variable.ttf \
  .venv-render/bin/python -m unittest discover -s tools -p test_render.py -v
```

Optional environment: `WB_RENDER_BROWSER`, `WB_SLIDE_BIN`. Compare the compact
sample with `samples/static-components-raw`: exported HTML must match exactly;
PNG comparison permits only a one-level RGB difference in at most 32 pixels
per 1920×1080 image, for browser glyph-edge raster noise. Verify Korean text,
PNG dimensions/fonts, actual PDF page count and failure without output for
JavaScript errors, missing/network assets, missing glyphs, empty decks and
unresolved elements. Existing outputs are preserved. CI downloads a hash-pinned
public Noto Sans KR font and uploads only the synthetic render artifact.

Font inspection: [FontTools](https://fonttools.readthedocs.io/en/latest/ttLib/ttFont.html).
Browser screenshots and print PDF: [Playwright](https://playwright.dev/python/docs/api/class-page).
Noto license: [SIL OFL](https://github.com/google/fonts/blob/b38c5c93af322c45f633e17ac440ec1e6c94d489/ofl/notosanskr/OFL.txt).
