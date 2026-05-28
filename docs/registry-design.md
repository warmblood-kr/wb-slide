# Theme & Layout Registry — Design

> Status: Draft / RFC
> Goal: Distribute and dynamically load themes/layouts via GitHub Pages —
> no install step, no local cache management.

## Principles

1. **GitHub Pages as the CDN.** Themes are served as static files. No download/install, just fetch.
2. **Self-contained output.** Rust server fetches at startup, inlines into the generated HTML. The served page (and `wb-slide export` output) has no runtime dependencies.
3. **Single binary, zero deps.** No npm, no package manager. Just `wb-slide` + a URL.
4. **Convention over configuration.** A theme is a directory with a known manifest.
5. **Composable.** A deck can pull multiple themes + individual layouts.

## Distribution Model

### Themes live on GitHub Pages

A theme repo:
- Has GitHub Pages enabled
- Serves a manifest at a known path

URL pattern:
```
https://<org>.github.io/<repo>/<theme-name>/theme.json
```

Example:
```
https://warmblood-kr.github.io/wb-slide-registry/themes/monocle/theme.json
```

### Theme structure (served via GitHub Pages)

```
wb-slide-registry/                            (repo root)
  README.md
  index.json                                  → /index.json
  themes/
    monocle/
      theme.json                              → /themes/monocle/theme.json
      layouts/
        slide-brochure-feature.js             → /themes/monocle/layouts/slide-brochure-feature.js
      styles/
        custom.css                            → /themes/monocle/styles/custom.css
        fonts.css                             → /themes/monocle/styles/fonts.css
    minimal/
      theme.json
      layouts/
      styles/
  layouts/                                    (standalone layouts)
    slide-timeline/
      layout.json                             → /layouts/slide-timeline/layout.json
      slide-timeline.js
      slide-timeline.css
```

### theme.json (manifest)

```json
{
  "name": "monocle",
  "version": "0.1.0",
  "description": "Brochure-style theme for Monocle decks",
  "author": "Warmblood",
  "license": "MIT",
  "wb-slide": ">=0.5.0",
  "layouts": [
    "layouts/slide-brochure-feature.js",
    "layouts/slide-brochure-cover.js"
  ],
  "styles": [
    "styles/fonts.css",
    "styles/custom.css"
  ]
}
```

Explicit manifest > directory listing because GitHub Pages doesn't provide directory listings. Authors list their files.

### layout.json (standalone layouts)

```json
{
  "name": "slide-timeline",
  "version": "0.1.0",
  "component": "slide-timeline",
  "description": "Horizontal timeline layout",
  "files": {
    "js": "slide-timeline.js",
    "css": "slide-timeline.css"
  }
}
```

### index.json (registry catalog)

The central registry repo (`warmblood-kr/wb-slide-registry`) also serves an index of all curated themes/layouts:

```
https://warmblood-kr.github.io/wb-slide-registry/index.json
```

```json
{
  "themes": {
    "monocle": "https://warmblood-kr.github.io/wb-slide-registry/themes/monocle/theme.json",
    "minimal": "https://warmblood-kr.github.io/wb-slide-registry/themes/minimal/theme.json",
    "corporate-blue": "https://some-org.github.io/wb-theme-corporate-blue/theme.json"
  },
  "layouts": {
    "slide-timeline": "https://warmblood-kr.github.io/wb-slide-registry/layouts/slide-timeline/layout.json"
  }
}
```

Third-party authors submit a PR adding their entry. Their repo can live anywhere on GitHub Pages.

## Frontmatter Reference

```yaml
---
title: My Deck

# Single theme by short name (resolved through index.json)
theme: monocle

# Or by full URL (skips the index)
theme: https://warmblood-kr.github.io/wb-slide-registry/themes/monocle/theme.json

# Multiple themes — later overrides earlier in the cascade
themes:
  - minimal
  - monocle

# Individual layouts (added on top of theme)
layouts:
  - slide-timeline
  - https://example.org/wb-layouts/slide-fancy/layout.json
---
```

## Resolution Order (precedence high → low)

For both layouts and styles:

1. **Deck-local `./layouts/` and `./styles/`** (highest — explicit override)
2. **Frontmatter `layouts:` list** (declaration order)
3. **Frontmatter `theme:` / `themes:`** (declaration order, later overrides earlier)
4. **Built-in** in the wb-slide binary (lowest)

CSS cascade follows insertion order, so this falls out naturally.

## Loading Pipeline

```
wb-slide show
   │
   ├── Parse slides.md frontmatter
   │     theme: monocle
   │
   ├── Resolve theme name:
   │     1. Fetch index.json (or use bundled fallback for offline)
   │     2. Look up "monocle" → URL to theme.json
   │
   ├── Fetch theme.json
   │     → Get list of layouts + styles files
   │
   ├── Fetch each listed file (parallel HTTP GETs)
   │
   ├── Inline everything into the generated HTML page
   │     <style>...framework css...</style>
   │     <style>...theme css...</style>
   │     <style>...local custom.css...</style>
   │     <script>...framework js + layouts...</script>
   │
   └── Serve (self-contained HTML, no runtime fetches)
```

Same for `wb-slide export`: fetch + inline once → emit single HTML file.

## Caching

Light caching only — we don't want to manage a complex install state.

```
~/.cache/wb-slide/
  index.json                         # last fetched registry index
  index.json.etag
  themes/
    monocle@0.1.0.json               # cached theme.json
    monocle@0.1.0.layouts.bundle     # concatenated JS for fast reload
  fetched-at
```

- HTTP `If-None-Match` (etag) used for revalidation
- TTL: 1 hour default, can be overridden by `--no-cache` or `--refresh`
- Pinned versions (`@v0.1.0`) cached indefinitely

If GitHub is unreachable AND no cache, fail with a clear error.
If GitHub is unreachable AND cache exists, use cache silently.

## CLI Commands

```bash
# Just run — themes auto-fetched
wb-slide show

# Force refresh
wb-slide show --refresh-themes

# Inspect / browse
wb-slide theme list                  # list themes in the registry index
wb-slide theme info monocle          # show theme.json contents
wb-slide theme search corporate      # text search in registry

# Maintenance
wb-slide registry update             # refresh local index.json cache
wb-slide cache clear                 # clear all caches

# For theme authors
wb-slide theme new my-theme          # scaffold a new theme repo
wb-slide theme validate              # check theme.json structure in CWD
wb-slide theme preview <url>         # render a sample deck with the given theme

# Optional: clone a theme for local editing/override
wb-slide theme clone monocle ./themes/monocle    # downloads to local dir
```

Notice: no `install`, no `update`, no `remove`. Themes are not "installed" — they're fetched-on-demand.

## Why GitHub Pages?

| Aspect | Pages-served | Repo clone (npm-style) |
|--------|--------------|------------------------|
| Theme author setup | Push to repo, enable Pages | Push to repo, publish to npm |
| User install | Nothing — just `theme: name` | `wb-slide theme install name` |
| Updates | Instant on next fetch | Explicit `update` command |
| Offline use | Works after first fetch (cached) | Works after install |
| Versioning | URL with `@version` or git tag | Semver via package manager |
| Trust | Static files only, no scripts run | Same |
| Cost | Free for OSS (GitHub Pages) | Free (GitHub) |
| Discoverability | One registry repo + index.json | Same |

Pages wins on simplicity for both authors and users.

## Trust & Security

- Themes are fetched via HTTPS only.
- No install-time code execution. Theme files are static (`.js`, `.css`, `.json`).
- The JS files are loaded by the browser (same as the framework JS). They run in the page context — same trust model as any web page using a CDN.
- Optional: SHA-256 checksum of theme.json in the registry index for tamper detection.
- Out of scope (deferred): signed themes, sandboxed JS.

## Versioning

- Theme `theme.json` declares its own version (semver).
- Version pinning via URL: `https://.../theme.json` can have query string or be served from `/v0.1.0/theme.json` (depends on theme author's preference).
- Default: fetch the latest version. The registry index lists the latest known version.
- Pinned: `theme: monocle@0.1.0` resolves through index.json's version-specific URL.

## Export Behavior

`wb-slide export` does the same fetch + inline. The resulting `export.html`:
- Is fully self-contained
- Works offline
- Works on file:// (no fetch at runtime)
- Can be shared as a single file

This is a strong property: the brochure recipient never needs internet or wb-slide installed.

## Future: Showcase Site

A static GitHub Pages site at `warmblood-kr.github.io/wb-slide-registry/`:
- Index page listing themes with screenshots and descriptions
- Generated from `index.json`
- Each theme has its own preview page (live iframe loading a sample deck)
- "Use this theme" snippet: copy-paste the frontmatter line

Defer until 5+ themes exist.

## Migration Path

- **v0.5**: `theme:` frontmatter + fetch from a single hardcoded registry URL. In-tree themes only (inside the registry repo). Cache to disk.
- **v0.6**: `themes:` array (composition). Third-party theme URLs (any GitHub Pages site).
- **v0.7**: `layouts:` frontmatter for standalone layouts. CLI: `theme list/info/search`.
- **v0.8**: `theme new` scaffolder + `theme validate`. `theme clone` for local editing.
- **v0.9**: Showcase site (static GitHub Pages).
- **v1.0**: Lock formats. Document theme author guidelines.

## Open Questions

- **Slug collisions**: `org/name` style for full disambiguation, or just rely on the curated index?
- **Versioned URLs**: How does a theme author serve `@v0.1.0` vs latest? Probably via separate paths in the repo, or per-tag GitHub Pages branches.
- **Asset paths in themes**: A theme's CSS may reference fonts/images via relative URLs. Since we inline the CSS, these need to be rewritten to absolute URLs pointing back at the theme's GitHub Pages origin.
- **Hot reload during dev**: `wb-slide show --dev` watches local files. For external themes, refresh on `--refresh-themes`.
- **Network unreachable**: Behavior when GitHub Pages is down. Fail fast or fall back to cache + warn?
- **CORS**: GitHub Pages serves with permissive CORS; fetching from Rust server side doesn't have CORS issues. Browser-side direct loading would need to verify CORS headers. (Inlining sidesteps this entirely.)
