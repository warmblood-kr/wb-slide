---
title: wb-slide layout gallery
footer: <b>wb-slide</b> · layout gallery
layout: slide-cover
---

# wb-slide

A markdown-driven slide tool — one slide per built-in layout follows.

---
layout: slide-section
---

# slide-section

---
layout: slide-feature
heading: slide-feature is the workhorse — an action title plus a body
subtitle: heading + optional subtitle + Markdown/HTML content
---

- One idea per slide, stated as a full-sentence title
- Body holds bullets, a table, a stat, or one visual
- The most common layout in a real deck

---
layout: slide-two-column
heading: slide-two-column places content side by side
subtitle: use ::left:: / ::right:: slot markers
---

::left::

## Before
- Single column of text
- No comparison structure

::right::

## After
- Two balanced columns
- Ideal for before/after, pros/cons

---
layout: slide-image-full
---

![A full-bleed image](assets/sample.png)

---
layout: slide-quote
quote: Above all else show the data.
author: Edward R. Tufte
---

---
layout: slide-contact
---

# slide-contact

Use it for closing details, contacts, and sources.

- hello@example.com
- github.com/warmblood-kr/wb-slide

---
layout: slide-default
---

# slide-default

A generic padded body — the fallback for any unknown layout name.
