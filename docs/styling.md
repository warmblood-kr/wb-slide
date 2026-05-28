# Styling

## Theme Variables

The default theme uses CSS custom properties. Override them in `styles/custom.css`:

```css
:root {
  --color-accent: #FF6600;        /* Titles, highlights */
  --color-text-dark: #1F2937;     /* Body text */
  --color-text-gray: #6B7280;     /* Secondary text */
  --color-header-gray: #D1D5DB;   /* Watermark color */
  --color-white: #FFFFFF;         /* Slide background */
  --font-family: 'Pretendard', sans-serif;
}
```

## Custom CSS

Create any `.css` file in the `styles/` directory. All files are loaded
automatically, sorted by filename.

```
styles/
  00-fonts.css      # Load custom fonts
  custom.css        # Override theme + add classes
```

## Utility Classes

These Tailwind-like utilities are available out of the box:

**Layout:** `flex`, `flex-col`, `flex-wrap`, `items-center`, `justify-center`

**Spacing:** `gap-3` to `gap-6`, `mb-2`, `mb-8`, `mt-12`

**Sizing:** `w-10`, `h-7` to `h-16`, `h-full`

**Typography:** `font-bold`, `text-4xl`

## Print / PDF Export

Use `Ctrl+P` in the browser or `wb-slide export` for a self-contained HTML file.

The `@media print` stylesheet automatically:
- Shows all slides stacked, one per page
- Removes navigation UI
- Sets page size to 960x540 (16:9)
