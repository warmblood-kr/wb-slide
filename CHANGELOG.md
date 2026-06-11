# Changelog

All notable changes to wb-slide.

## v0.9.0 — 2026-06-12

### Added

- **Content component classes** (issue #1) in `framework/utilities.css` — reusable
  slide-body building blocks so authors stop hand-rolling nested divs + inline
  styles (which bloat decks and break print/PDF). Theme-token-keyed:
  `.cols`/`.col`, `.panel`/`.panel-label`, `.info-box`/`.warn-box`,
  `.step-row`/`.step-num`/`.step-content`, `.compare-bad`/`.compare-good`,
  `.flow`/`.flow-step`/`.flow-connector`, `.cluster-grid`/`.cluster`,
  `.risk-item`, `.attn-bar`, `.mono-chip`.
- **`.cover-stack`** helper + docs for layered covers (background / decoration /
  content layers that each fill the cover).
- A **components gallery** showcase and [`docs/components.md`](docs/components.md).

### Fixed

- **`slide-cover`** (issue #2): `.ms-cover-layout` is now `position: relative`, so
  layered / full-bleed children fill the cover instead of collapsing under the
  centering grid. Simple centered covers are unchanged.
- **`build.rs`**: rebuild and re-embed framework assets whenever `framework/`
  changes. Release builds embed assets at compile time, so an edited CSS/HTML/JS
  asset could previously be silently missing from an incremental build.

## v0.8.0 — 2026-06-12

### Added

- **`wb-slide validate`** — a static deck checker (no browser). Flags unknown
  `layout:` names, indented/dropped frontmatter, blank lines inside raw HTML/SVG
  blocks, missing local assets (an error), and slides likely to overflow the
  fixed 960×540 canvas. `--strict` fails on warnings too — CI-friendly.
- **`wb-slide export --embed`** — inline local images (`<img src>` and CSS
  `url()`) as base64 data URIs, producing a single, fully portable HTML file.
- **Presentation helper classes** in the framework, theme-independent and keyed
  to the theme-contract tokens so they adopt any theme's accent: `.kicker`,
  `.stat` / `.stat--accent` / `.stat-label`, `.takeaway`, `.source`, `.pos` /
  `.neg`, and `.table-clean` (+ `.num` for right-aligned numerals).
- **GFM Markdown extensions**: pipe tables, strikethrough (`~~x~~`), autolinks,
  and task lists (previously Markdown tables rendered as literal `| text |`).
- A rendered **layout gallery** plus ready-to-copy **consulting** and
  **marketing** templates, published to GitHub Pages
  (<https://warmblood-kr.github.io/wb-slide/>).

### Fixed

- **`slide-two-column`**: the heading rendered at the bottom of the slide and the
  two columns collapsed to the left — the container was missing its
  `grid-template-areas`, so the heading/subtitle fell into implicit rows.

## v0.7.0 — 2026-06-09

### Breaking changes

Layouts are now HTML templates rendered server-side by wb-slide instead of
JavaScript Web Components rendered client-side. See [docs/migration-v0.7.md](docs/migration-v0.7.md)
for migration steps.

This fixes a class of intermittent bugs where some slides (particularly those
with inline SVG or deeply nested HTML) showed raw HTML instead of rendering,
especially in PDF export and on `file://` URLs.

### Added

- Mustache-style template engine (Rust-side) with `{{var}}`, `{{{raw}}}`,
  `{{#section}}`, `{{slot:name}}` support. 15 unit tests.
- New module `src/render.rs`: `LayoutSet` resolves layouts in order
  local → theme → built-in, with `.html` files taking precedence over `.js`.
- `.js` layouts still supported as an escape hatch for layouts that genuinely
  need client-side execution. Mixed `.html` and `.js` in the same deck works.

### Changed

- Built-in `framework/layouts/*.js` files replaced with `*.html` templates.
- `framework/slide-base.js` removed (no longer needed).
- `framework/monocle-slide.js` slimmed from ~110 to ~70 lines — only handles
  keyboard navigation, viewport scaling, and hash routing.
- Theme manifest layouts list now expects `.html` files. `.js` entries are
  skipped with a warning.
- Output HTML no longer contains `<slide-foo>` Custom Elements; slides are
  rendered as plain `<div class="ms-slide-container slide-foo">`.

### Migration

- Official `monocle-brochure` theme (in the registry) has been migrated to
  template format.
- For custom themes or local `layouts/*.js` files: convert to `.html` using
  the cheat sheet in [docs/migration-v0.7.md](docs/migration-v0.7.md).

---

## v0.6.1 — 2026-06-01

### Fixed

- `wb-slide gui` no longer pulls `wayland-sys` build deps. Replaced the `rfd`
  crate with OS-native shell-outs (`osascript` on macOS, `zenity`/`kdialog`
  on Linux, PowerShell + WinForms on Windows). Restores musl/headless Linux
  builds in CI.

## v0.6.0 — 2026-06-01

### Added

- `wb-slide gui` command. Opens a native folder picker dialog, then starts
  the presentation server. Aimed at non-CLI users.
- `install.sh` creates a desktop launcher (`~/Applications/WB Slide.app` on
  macOS, `~/.local/share/applications/wb-slide.desktop` on Linux).
- `install.ps1` creates a Start Menu and Desktop shortcut on Windows.

## v0.5.3 — 2026-06-07

### Changed

- `install.ps1` now auto-registers `%LOCALAPPDATA%\Programs\wb-slide` into the
  User PATH (no admin needed) and updates the current shell's `$env:Path`.

## v0.5.2 — 2026-06-01

### Fixed

- PowerShell installer architecture detection. Falls back to
  `PROCESSOR_ARCHITECTURE` and `PROCESSOR_ARCHITEW6432` when
  `RuntimeInformation.OSArchitecture` returns empty (some Windows configs).

## v0.5.1 — 2026-05-28

### Fixed

- PDF export and `@media print` now render all slides (was: only the first
  slide). The previous print CSS referenced `.viewport` / `.slide-container`
  but the actual class names were `.ms-viewport` / `.ms-slide-container`.

### Added

- `install.sh` defaults to `~/.local/bin` (no sudo) and warns if not on PATH.
- `install.ps1` — Windows PowerShell installer for the `irm | iex` workflow.

### Changed

- `wb-slide update` no longer prints "Updated to vX!" when the binary replace
  failed.

## v0.5.0 — 2026-05-28

### Added

- Theme support via `theme:` frontmatter. Themes are fetched from a GitHub
  Pages registry (default: `warmblood-kr/wb-slide-registry`), cached in
  `~/.cache/wb-slide/`, and inlined into the generated HTML.
- Theme contract: 6 standardized CSS custom properties every theme should
  define (`--color-accent`, `--color-text`, `--color-text-muted`,
  `--color-border`, `--color-background`, `--font-family`).
- `--refresh-themes` flag bypasses the cache.

### Changed

- CSS variable names standardized (with backward-compatible aliases for the
  legacy `--color-text-dark`, `--color-text-gray`, etc.).

## v0.4.0 — 2026-05-28

### Added

- Built-in layouts now use CSS Grid for more robust layout behavior.
- Version sync check in the release workflow (Cargo.toml vs git tag must
  match).
- Continuous integration workflow runs `cargo test` and `cargo build` on
  every push.

## v0.3.0 — 2026-05-28

### Added

- `::slot-name::` syntax for named slots in `slides.md`.
- `slide-two-column` layout supports `::left::` and `::right::` slots.
- Eight unit tests for slot/frontmatter parsing.

## v0.2.0 — 2026-05-28

### Added

- `wb-slide version` checks GitHub for a newer release.
- `wb-slide update` downloads and replaces the running binary.

## v0.1.0 — 2026-05-28

Initial release.

- Single Rust binary, no runtime dependencies.
- `wb-slide show` / `wb-slide export` commands.
- Eight built-in layouts.
- GitHub Actions releases for macOS ARM64, Linux x64 (musl), Windows x64.
- One-line installer.
