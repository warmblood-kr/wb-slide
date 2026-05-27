# Monocle AI Brochure Design Guide

Design specification extracted from the official **Monocle AI 사용자 기능 소개서** (2026-05-14).
Source: PowerPoint (960x540pt, 16:9), exported via Slidev.

---

## Color Palette

### Primary

| Role | Hex | RGB | Usage |
|------|-----|-----|-------|
| Accent Orange | `#FF6600` | 255, 102, 0 | Slide titles, section headings, highlight borders, TOC title |
| Text Body | `#404040` | 64, 64, 64 | Subtitle text, descriptions |
| Watermark Gray | `#CECECE` | 206, 206, 206 | "Monocle AI" top-right watermark |
| Contact Navy | `#243B5D` | 36, 59, 93 | Contact page title |
| Background | `#FFFFFF` | 255, 255, 255 | All slide backgrounds |

### Secondary

| Role | Hex | Usage |
|------|-----|-------|
| UI Tag Green | `#00B894` | Product UI green tags (COLLECTION labels) |
| UI Tag Red | `#E74C3C` | Product UI red tags (참고자료 labels) |
| UI Tag Orange | `#F39C12` | Product UI orange tags |
| Light Gray BG | `#F9F9F9` | Card/panel backgrounds in product screenshots |

---

## Typography

| Element | Font | Weight | Size | Color |
|---------|------|--------|------|-------|
| Slide Title | Pretendard | Bold (700) | 2.2rem | `#FF6600` |
| Subtitle | Pretendard | Regular (400) | 1.1rem | `#404040` |
| Watermark | Pretendard | Light (300) | 0.9rem | `#CECECE` |
| Footer Logo | Warmblood custom | Italic W + Bold | 0.85rem | `#404040` |
| Page Number | Pretendard | Regular (400) | 0.8rem | `#636E72` |
| Contact Title | Pretendard | Bold (700) | 1.8rem | `#243B5D` |
| Contact Body | Pretendard | Regular (400) | 1.2rem | `#404040` |
| TOC Title | Pretendard | Bold (700) | 2.2rem | `#FF6600` |
| TOC Items | Pretendard | Regular (400) | 1.1rem | `#404040` |

**Font CDN:**
```
https://cdn.jsdelivr.net/gh/orioncactus/pretendard/dist/web/static/pretendard.css
```
**Fallback:** -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif

---

## Slide Dimensions

- **Aspect Ratio:** 16:9
- **Canvas:** 960 x 540 (Slidev canvasWidth: 960)
- **Original PPT:** 960 x 539.88pt

---

## Layout System

### Common Elements (all slides except cover)

```
+--------------------------------------------------+
|                              Monocle AI   <- watermark, top-right
|                                               (0.9rem, #CECECE)
|  [content area]
|
|
|
|
|  Warmblood                              15  <- page number
|  ^ footer logo, bottom-left     bottom-right ^
+--------------------------------------------------+
```

- **Watermark:** "Monocle AI", absolute top-right, 24px from top, 40px from right
- **Footer Logo:** "Warmblood" (italic W, bold rest), absolute bottom-left, 24px from bottom, 40px from left
- **Page Number:** absolute bottom-right, 24px from bottom, 40px from right
- **Padding:** 40px all sides

### Slide Types

#### 1. Cover
- No watermark, no footer, no page number
- Centered vertically: Logo -> Title -> Laptop mockup
- Logo: `monocle-ai-logo-full.png` (eye icon + "Monocle AI" text), h=64px
- Title: "Monocle AI 기능 소개서", 2.5rem bold, `#2D3436`
- Mockup: max-width 600px, with rounded shadow frame

#### 2. TOC (Table of Contents)
- Centered: title + list
- Title: "목차", `#FF6600`, bold, 2.2rem, centered
- Category label: bold, 1.1rem
- List items: bullet (disc), 1.1rem, line-height 2

#### 3. Section Divider
- Single line centered vertically + horizontally
- Text: section name, `#FF6600`, bold, 2.5rem

#### 4. Feature (most common)
- Top-left: heading in `#FF6600`, bold, 2.2rem
- Below heading: one-line subtitle in `#404040`, regular, 1.1rem
- Center: product screenshot(s)
  - Single: centered, max-width 80%, rounded shadow
  - Dual overlap: left image z-1 at 5% left, right image z-2 at 5% right, offset 10% top

#### 5. Contact
- Top-left: "Contact" in `#243B5D`, bold, 1.8rem
- Below: company name (bold), URLs, email, phone
- Line-height: 2

---

## Screenshot Presentation

### Single Screenshot
```css
border-radius: 8px;
box-shadow: 0 4px 24px rgba(0, 0, 0, 0.12);
max-width: 80%;
margin: 1.5rem auto 0;
```

### Dual Overlap Screenshots
```
+--------------------------------------------------+
|  +------------------+                              |
|  |  Screenshot 1    |                              |
|  |  (z-index: 1)    +------------------+          |
|  |                   |  Screenshot 2    |          |
|  +-------------------+  (z-index: 2)    |          |
|                      |                   |          |
|                      +-------------------+          |
+--------------------------------------------------+
```
- Container: `position: relative; width: 100%; height: 100%`
- Image 1: `position: absolute; top: 0; left: 5%; max-width: 55%`
- Image 2: `position: absolute; top: 10%; right: 5%; max-width: 55%`
- Both: `border-radius: 8px; box-shadow: 0 4px 24px rgba(0,0,0,0.12)`

### Highlight Box (e.g., Craft page)
- Orange border box drawn on screenshot
- Color: `#FF6600`, 2px solid border

---

## Asset Inventory

### Logos (`assets/logos/`)

| File | Size | Description |
|------|------|-------------|
| `monocle-ai-logo-full.png` | 1654x328 | Eye icon + "Monocle AI" text (transparent BG) |
| `monocle-ai-text.png` | 1070x145 | "Monocle AI" text only (transparent BG) |
| `warmblood-logo.png` | 1236x180 | Warmblood company logo (transparent BG) |

### Screenshots (`assets/screenshots/`)

| File | Slide | Size | Content |
|------|-------|------|---------|
| `cover-laptop-mockup.png` | 1 | 2169x1314 | Laptop with multi-vendor chat |
| `p04-multi-vendor-model-selection.png` | 4 | 2508x1890 | Model dropdown (GPT, Claude, Gemini) |
| `p05-multi-vendor-simultaneous-call.png` | 5 | 2048x1368 | 3-model simultaneous response |
| `p06-project-team-collaboration.png` | 6 | 2784x2144 | Project management page |
| `p07-m365-description-card.png` | 7 | 736x707 | M365 connector info card |
| `p08-web-search-in-progress.png` | 8 | 2922x1872 | Web search loading |
| `p08-web-search-result-with-citation.png` | 8 | 2922x1872 | Web search with URL citation |
| `p09-file-attachment.png` | 9 | 2922x1872 | File attached to chat |
| `p10-reference-materials-list.png` | 10 | 2922x1872 | Reference collection library |
| `p10-reference-materials-in-chat.png` | 10 | 2922x1872 | Collections used in chat |
| `p11-custom-models-agents.png` | 11 | 2922x1872 | 10 custom agents list |
| `p12-craft-document-generation.png` | 12 | 1319x925 | Craft mode workflow |
| `p13-image-generation-model-select.png` | 13 | 1766x1144 | Image model dropdown |
| `p14-memory-management.png` | 14 | 2266x1788 | Memory dialog |

### Icons (`assets/icons/`)

| File | Size | Description |
|------|------|-------------|
| `microsoft-365.png` | 111x111 | Microsoft 365 diamond icon |
| `office-365-logo.png` | 374x86 | "Office 365" text with icon |
| `m365-integration-bar.png` | 735x112 | Chat input bar (Code Interpreter + M365) |
| `powerpoint.png` | 91x85 | PowerPoint icon |
| `word.png` | 98x91 | Word icon |
| `excel.png` | 94x88 | Excel icon |
| `outlook.png` | 111x103 | Outlook icon |
| `onedrive.png` | 120x120 | OneDrive icon |
| `sharepoint.png` | 74x75 | SharePoint icon |
| `teams.png` | 111x103 | Teams icon |

All icons extracted from PPTX with original transparency preserved.

---

## Slide Map

| # | Layout | Heading | Screenshot Count |
|---|--------|---------|-----------------|
| 1 | cover | Monocle AI 기능 소개서 | 1 (mockup) |
| 2 | toc | 목차 | 0 |
| 3 | section-divider | 사용자용 기능 | 0 |
| 4 | feature | 멀티 벤더 | 1 |
| 5 | feature | 멀티 벤더 동시 호출 | 1 |
| 6 | feature | 프로젝트 (팀 협업 공간) | 1 |
| 7 | feature | M365 (오피스웨어) 연동 | 1 card + 9 icons + 1 bar |
| 8 | feature | 웹 자료 활용 | 2 (overlap) |
| 9 | feature | 파일 첨부 | 1 |
| 10 | feature | 참고 자료 | 2 (overlap) |
| 11 | feature | 커스텀 모델 | 1 |
| 12 | feature | 크래프트 (Word·PPT·PDF·Excel 생성) | 1 |
| 13 | feature | 이미지 생성 | 1 |
| 14 | feature | 메모리 | 1 |
| 15 | contact | Contact | 0 |

---

## Tech Stack

- **Slidev** v52 with `@slidev/theme-default`
- **Custom layouts:** `feature.vue`, `section-divider.vue`, `contact.vue`
- **CSS engine:** UnoCSS (Tailwind-compatible utilities)
- **Global styles:** `styles/index.css` (CSS custom properties + component classes)
- **Static assets:** `public/assets` -> symlink to `assets/`
- **Export:** `npx slidev export` (requires `playwright-chromium`)
