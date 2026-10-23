# Diagram library: design note

**Goal.** An author, often an LLM, picks a diagram type and writes compact
JSON data. A deterministic renderer lays it out and draws SVG. No model draws
anything, so the same data and theme always give the same bytes.

The CLI ships in v1:

```sh
wb-slide diagram render --type funnel --in data.json --theme default -o funnel.svg
```

```text
data.json ──validate──▶ typed model ──layout (pure)──▶ boxes + paths ──check bboxes──▶ SVG (preset tokens)
```

The look follows the slide preset: `--wb-*` tokens
([theme-contract.md](theme-contract.md)), Pretendard, a strict 8px grid, one
accent colour, plenty of whitespace. No gradients, shadows, 3D or clip-art.

Planned (agreed with the markup owner): a `<wb-diagram type="..." alt="...">`
tag whose body is the JSON model ([markup-dsl.md](markup-dsl.md)), lowered to
a spec `diagram` block ([slide-spec.md](slide-spec.md)). Export writes the SVG
inline (`<svg class="wb-diagram" role="img" aria-label>`), so the render font,
font-coverage check and `render.py --check` see diagram text. At most 16
diagrams per deck; each type caps its own item counts.

## Prior art

| Source | Lesson taken |
|---|---|
| Mermaid | Text in, diagram out, and LLMs already write it. But the default look is generic and hard to bring in line with a deck. Layout is dagre, with ELK as an option. We keep text in and fix one look. |
| D2 | A diagram language with swappable layout engines (dagre, ELK, TALA) and named themes. Keep the data model separate from layout, and themes separate from both. |
| Graphviz | `dot` is layered layout (Sugiyama): rank assignment, dummy nodes on long edges, crossing reduction, then coordinates. Our flowchart uses the same steps, kept small. |
| PlantUML | Many diagram kinds from one text syntax, some laid out through Graphviz. Breadth is useful, but in our view the default style looks dated on slides. A catalog needs one visual system. |
| Excalidraw | Hand-drawn, whiteboard style with manual placement. Good for sketching, the opposite of what we want: clean, exact, generated. |
| Office SmartArt | A closed catalog grouped by intent: List, Process, Cycle, Hierarchy, Relationship, Matrix, Pyramid, Picture. Content comes from a bullet outline. We copy the intent-first catalog and drop the 3D and gradient styles. |
| Duarte, *slide:ology* | Diagrams are chosen by the relationship they show (flow, structure, cluster, radiate, network) and then simplified. Pick by meaning, not by decoration. |
| Napkin AI | Text to business visuals, suggested by intent. Shows the demand, but output is model-generated. We keep intent-to-visual and make it deterministic. |
| Whimsical | Flowcharts and mind maps that snap and align for you, with a calm, flat look. Alignment should be the tool's job, not the author's. |

## Common rules

- Input is one JSON object. Unknown fields are errors at every level.
- Strings are nonblank. Labels are at most 80 characters. Short fields are
  at most 40: `id`, `category`, `date`, quadrant `title`, column, lane, role
  and set names, and axis labels (`label`, `low`, `high`). Korean is fine.
- `highlight` is a 0-based index into the type's main array.
- No `title` field. The slide owns the heading.
- Canvas is a 640×360 viewBox. Every coordinate and size is a multiple of 8px. Text sizes are exempt.
- Limits below are hard. Out of range is an error, never a silent cut.

## Shared primitives

Every type is built from the same five pieces. A new type adds layout, not
new drawing code.

| Primitive | What it is | Notes |
|---|---|---|
| node | box, pill or circle holding one wrapped label | fill `card-bg`, stroke `border`; highlighted node uses `accent` |
| edge | straight, orthogonal or polyline path, optional arrowhead, optional label | stroke `border`, label placed in a gap so it never sits on a node |
| label | wrapped text block with a max width and max lines | width estimate below; never clipped |
| axis | line with a label and low/high end labels | quadrant, timeline, gantt, radar |
| grid cell | rectangle in a row/column grid, aligned to 8px | comparison, raci, heatmap, quadrant |

**Text width estimate.** Conservative, with no font metrics at layout time:
CJK and full-width characters count 1.0em, Latin letters, digits and
punctuation 0.62em, spaces 0.32em. Lines break at spaces; a word wider than the
box breaks between characters. A label that needs more than its max lines
shrinks 2px at a time down to 12px; if it still does not fit, the render fails
with the label text. Text is never clipped or ellipsized.

**Guarantees.** For every rendered diagram:

1. Every text bbox lies inside the canvas.
2. No two text bboxes overlap.
3. No two node boxes overlap, and every label lies inside its own node.
4. Edge labels sit on a `--wb-bg` background plate and do not overlap nodes
   or other text. No edge passes through a label.

The same layout check covers all four. Tests run it by computing the bboxes from the layout output for each
type, across fixture data at the min and max of every limit, including long
Korean labels. A layout that breaks a guarantee is a bug, not a warning.

## Catalog

A closed catalog grouped by intent. `v1` types are implemented. `planned`
types have a draft model; fields may still change slightly. `?` marks an
optional field.

| Type | Intent | Data model | Limits | Use when / don't use when |
|---|---|---|---|---|
| quadrant `v1` | compare | `{"x"?:{"label"?,"low"?,"high"?}, "y"?:{"label"?,"low"?,"high"?}, "quadrants":[TL,TR,BL,BR] each {"title","items"?:[str]}, "highlight"?:0-3}` | 4 quadrants, ≤4 items each | Two axes split options into four groups; SWOT is a quadrant with titles S/W/O/T. Not for precise values. |
| venn `planned` | compare | `{"sets":[str], "overlaps":[{"sets":[i,j], "label"}]}` | 2–3 sets; `overlaps[].sets` are 0-based indexes into `sets` | Shared versus distinct traits. Not for more than 3 groups. |
| comparison `planned` | compare | `{"columns":[str], "rows":[{"label","cells":[str]}]}` | 2–4 columns, ≤6 rows | Options side by side on the same criteria. Not for long prose cells. |
| before_after `planned` | compare | `{"before":[str], "after":[str]}` | 1–5 items each | A change of state. Not when there are more than two states. |
| flowchart `v1` | process | `{"direction"?:"LR\|TB", "nodes":[{"id","label","shape"?:"process\|decision\|terminal"}], "edges":[{"from","to","label"?}]}` | 2–12 nodes; DAG only, a cycle is an error | Steps with branches. Layered layout with dummy nodes. Not for loops (use cycle or causal_loop). |
| chevrons `planned` | process | `{"steps":[{"label","note"?}], "highlight"?:index}` | 3–6 steps | A short linear sequence. Not when steps branch. |
| stairs `planned` | process | `{"steps":[{"label","note"?}]}` bottom step first | 3–5 steps | Progressive maturity or build-up. Not when steps are equal in weight. |
| swimlane `planned` | process | `{"lanes":[str], "steps":[{"id","lane","label"}], "edges":[{"from","to"}]}` | 2–5 lanes, ≤12 steps | Who does what in a handoff process. Not with one actor. |
| value_chain `planned` | process | `{"primary":[str], "support":[str]}` | 3–6 primary, 0–4 support | Porter-style primary and support activities. Not for a generic process. |
| funnel `v1` | process | `{"stages":[{"label","value"?}]}` | 2–6 stages | Narrowing stages such as conversion. `value` is a presentation string; widths are fixed steps, not to scale. Not for growth. |
| cycle `v1` | cycle | `{"steps":[{"label","note"?}]}` clockwise from 12 o'clock | 3–8 steps | A repeating loop with no end. Not when there is a clear start and finish. |
| causal_loop `planned` | cycle | `{"nodes":[{"id","label"}], "edges":[{"from","to","sign":"+\|-"}]}` | 3–8 nodes | Reinforcing and balancing feedback. Not for one-way causes (use fishbone). |
| pyramid `v1` | hierarchy | `{"levels":[{"label","note"?}]}` top level first | 2–6 levels | Ranked layers where the base supports the top. Not for unranked groups. |
| tree `planned` | hierarchy | `{"root":{"label","children"?:[node]}}` recursive | ≤4 levels including root, ≤20 nodes | Org charts and breakdowns. Not for many-to-many links. |
| mindmap `planned` | hierarchy | `{"center", "branches":[{"label","items"?:[str]}]}` | 2–6 branches, ≤4 items each | Topics around one idea. Not when order matters. |
| hub `v1` | relationship | `{"hub", "spokes":[str], "arrows"?:"none\|out\|in"}` | 3–8 spokes | One centre related to many parts (hub-and-spoke). Not when spokes relate to each other. |
| fishbone `v1` | relationship | `{"effect", "causes":[{"category","items":[str]}]}` | 2–6 categories, ≤4 items each | Root causes of one effect (Ishikawa). Not for feedback loops. |
| raci `planned` | relationship | `{"roles":[str], "tasks":[{"label","r","a","c"?:[str],"i"?:[str]}]}` | 2–6 roles, ≤8 tasks; `r` and `a` are one role name each, `c` and `i` lists of role names | Responsibility by task and role. Not for a plain org chart. |
| layers `planned` | structure | `{"layers":[{"label","items"?:[str]}]}` top first | 2–6 layers, ≤5 items each | A stack such as a tech architecture. Not when layers are ranked by size (use pyramid). |
| concentric `planned` | structure | `{"rings":[{"label","note"?}]}` centre first | 2–5 rings | Core to periphery (target, onion). Not for independent groups. |
| timeline `v1` | time | `{"events":[{"date","label","note"?}], "highlight"?:index}` | 2–8 events | Dated milestones in order. Not for durations (use gantt). |
| gantt `planned` | time | `{"start","end", "tasks":[{"label","from","to","group"?}]}` ISO dates | ≤12 tasks | Task durations and overlap. Not for a few point events. |
| roadmap `planned` | time | `{"lanes":[str], "periods":[str], "items":[{"lane","label","from","to"}]}` period indexes | 2–5 lanes, 2–8 periods | Themes over coarse periods (quarters). Not for day-level plans. |
| journey `planned` | time | `{"stages":[{"label","score":-2..2,"note"?}]}` | 3–7 stages; `score` is an integer −2..2 | Experience highs and lows across stages. Not for metrics with units. |
| waterfall `planned` | quantity | `{"start":{"label","value"}, "steps":[{"label","delta"}], "end"?:{"label"}}` numbers | 2–8 steps | How a total moves from start to end. Not without real numbers. |
| radar `planned` | quantity | `{"axes":[str], "series":[{"label","values":[number]}]}` | 3–8 axes, ≤2 series, values are numbers 0–5, floats allowed | Profile across many criteria. Not for exact comparison (use comparison). |
| heatmap `planned` | quantity | `{"rows":[str], "columns":[str], "cells":[[0-3]]}` | ≤8 rows × 8 columns | Intensity in a matrix (risk, coverage). Four steps of `accent` tint; not for exact values. |

The renderer does no arithmetic on presentation strings. Only `waterfall`,
`radar`, `heatmap` and `journey` take numbers, and only for geometry.

## Choosing a diagram

| Question | Type |
|---|---|
| Do options fall into four groups on two axes? | quadrant |
| What is shared and what is distinct between 2–3 groups? | venn |
| How do options compare on the same criteria? | comparison |
| What changes from now to then? | before_after |
| What are the steps, and do they branch? | chevrons if linear, flowchart if they branch |
| Who hands off to whom? | swimlane |
| Does it repeat with no end? | cycle; with +/− feedback, causal_loop |
| What causes this problem? | fishbone |
| Is it ranked by importance or size, base to top? | pyramid; maturity steps, stairs |
| Is it a parent/child breakdown? | tree; loose topics, mindmap |
| Is one thing at the centre of many? | hub; core to edge, concentric |
| Are these stacked dependencies, each resting on the one below? | layers |
| Who is responsible for each task? | raci |
| When did things happen? | timeline; durations, gantt; coarse themes, roadmap |
| How does a stage-by-stage experience feel? | journey |
| How many drop out at each stage? | funnel |
| How does a total move? | waterfall |
| How strong is each cell? | heatmap |

If no question fits, use a plain list or panel instead. A diagram that does
not show a relationship is decoration.

## Theming

Diagrams read only preset tokens, so `--theme default|night|tokens.css`
restyles them with the slides. No colour is hard-coded. All tokens below,
including `--wb-card-bg`, `--wb-border` and `--wb-radius`, already exist in
`framework/presets/themes/default.css`.

| Token | Used for |
|---|---|
| `--wb-bg` | canvas background |
| `--wb-text` | labels and node text |
| `--wb-muted` | notes, axis end labels, edge labels, dates |
| `--wb-accent` | the single highlight: `highlight` index, decision outline, active stage. One accent per diagram; heatmap tints are steps of it |
| `--wb-card-bg` | node and cell fill |
| `--wb-border` | edges, node strokes, axes, grid lines |
| `--wb-font` | all text (Pretendard by default) |
| `--wb-radius` | node corner radius |

Text sizes come from the type, not the theme, so a label that fits under one
theme fits under all. A theme with a larger body size does not change diagram
layout.
