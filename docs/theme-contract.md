# Theme Contract

Standard CSS custom properties (tokens) every wb-slide theme should define.
Built-in layouts and shared components read these tokens, so any theme that
honors the contract is interchangeable.

## Required tokens

These five tokens MUST be defined in the theme's CSS (typically inside `:root`).

| Token | Description | Default fallback |
|-------|-------------|------------------|
| `--color-accent` | Primary brand color. Used for titles, highlights, accents. | `#2563EB` |
| `--color-text` | Primary body text color. | `#1F2937` |
| `--color-text-muted` | Secondary / muted text (captions, page numbers, fine print). | `#6B7280` |
| `--color-border` | Subtle borders and chrome elements (watermark, dividers). | `#D1D5DB` |
| `--color-background` | Slide background. | `#FFFFFF` |
| `--font-family` | Primary font stack for body text. | system-ui stack |

Built-in framework styles use only these tokens. A theme that defines them
correctly will fully restyle every built-in layout.

## Optional tokens (recommended)

These don't have framework defaults but are common enough to standardize:

| Token | Description |
|-------|-------------|
| `--color-accent-secondary` | Secondary brand color (e.g., for sub-sections, navy/teal accents) |
| `--font-family-heading` | Heading-specific font (defaults to `--font-family`) |
| `--font-family-mono` | Code / monospace font |
| `--color-success` | Green/positive feedback color |
| `--color-warning` | Yellow/caution color |
| `--color-error` | Red/error color |

If your theme provides any of these, document them in your theme's README so
deck authors know they can reference them.

## Theme-specific extras

Themes may define any number of additional tokens for theme-specific purposes
(e.g., `--color-navy-dark` in `monocle-brochure`). These are not portable —
decks that use them are locked to that theme.

## Example: minimal theme

```css
/* my-theme/styles/theme.css */

@import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;600;700&display=swap');

:root {
  --color-accent: #2563EB;
  --color-text: #1F2937;
  --color-text-muted: #6B7280;
  --color-border: #E5E7EB;
  --color-background: #FFFFFF;
  --font-family: 'Inter', system-ui, sans-serif;
}
```

That's the minimum. It overrides framework defaults and is enough to give
every built-in layout the new look.

## Example: branded theme with extras

```css
:root {
  /* Required contract */
  --color-accent: #FF6600;
  --color-text: #404040;
  --color-text-muted: #636E72;
  --color-border: #CECECE;
  --color-background: #FFFFFF;
  --font-family: 'Pretendard', sans-serif;

  /* Recommended */
  --color-accent-secondary: #243B5D;
  --font-family-mono: 'JetBrains Mono', monospace;

  /* Theme-specific */
  --color-navy-dark: #243B5D;
  --brand-orange-highlight: #FFE0CC;
}
```

## Deprecated tokens (v0.4 and earlier)

The following names were used before v0.5 and are kept as aliases for
backward compatibility. They will be removed in a future version.

| Deprecated | Use instead |
|------------|-------------|
| `--color-text-dark` | `--color-text` |
| `--color-text-gray` | `--color-text-muted` |
| `--color-header-gray` | `--color-border` |
| `--color-white` | `--color-background` |

The framework defines these aliases as `var(--color-text)` etc., so existing
decks/themes keep working. Migrate when convenient.

## Validation (planned)

A future `wb-slide theme validate` command will check that a theme:
- Declares all 6 required tokens
- Has a valid `theme.json` schema
- Lists files that all exist
- Uses no removed/deprecated tokens (warning only)

For now, the contract is enforced by convention.
