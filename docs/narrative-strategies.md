# Narrative strategies: prior-art survey

## Purpose

A narrative strategy is an ordered sequence of shared **roles**. A role is a
reusable building block: it carries a purpose and a default layout
([data/roles.json](../data/roles.json)). Strategies only pick and order roles,
so a new strategy costs one JSON file ([data/narratives/scqa.json](../data/narratives/scqa.json)
is the schema example). `wb-slide narrative list|show|outline|recommend`
exposes them (see `wb-slide narrative --help`).

This note surveys well-known structures and maps each one onto our roles.
All summaries are our own words. Read the cited works for the real thing.

**Notation.** Roles are written `a → b → c`. `b*` means repeat. `[b]` means
optional. Role ids are exactly the keys in `roles.json`.

## Summary table

| # | Strategy | Family | Best for | Length |
|---|---|---|---|---|
| 1 | Minto Pyramid | logic | decide, inform executives | 5-30 min |
| 2 | SCQA | logic | decide, persuade | 5-30 min |
| 3 | Duarte sparkline | story | inspire, change minds | 15-45 min |
| 4 | slide:ology principles | craft | any deck (overlay) | any |
| 5 | Pixar story spine | story | case story, culture | 5-15 min |
| 6 | Hero's journey | story | transformation, keynote | 15-45 min |
| 7 | Monroe's motivated sequence | persuasion | call to action | 5-20 min |
| 8 | AIDA | persuasion | sales, launch | 3-10 min |
| 9 | Problem-solution-benefit | persuasion | product, proposal | 5-15 min |
| 10 | Before-after-bridge | persuasion | short pitch | 2-5 min |
| 11 | STAR | report | case study, review | 3-10 min |
| 12 | Three-act structure | story | general talk | 10-45 min |
| 13 | Inverted pyramid | report | update, news | 2-10 min |
| 14 | BLUF decision memo | logic | busy decision maker | 2-10 min |
| 15 | Startup pitch deck | pitch | investors | 10-20 min |
| 16 | Consulting storyline | logic | analysis, steering | 15-60 min |
| 17 | Toulmin argument | logic | contested claim | 5-15 min |
| 18 | Freytag pyramid | story | dramatic case | 10-30 min |
| 19 | PAS | persuasion | urgent fix | 2-5 min |
| 20 | FAB | persuasion | product demo | 5-15 min |
| 21 | Kishōtenketsu | story | reframing, insight | 5-15 min |
| 22 | Nested loops | story | long keynote | 20-60 min |
| 23 | Tell-show-tell | teach | training, lecture | 10-60 min |
| 24 | Rule of three | craft | any (overlay) | any |
| 25 | Assertion-evidence | logic | science, engineering | 10-30 min |
| 26 | Lessig / Takahashi | format | energetic talk | 5-20 min |
| 27 | Pecha Kucha / Ignite | format | timed showcase | 5-7 min |

## Logic family

### 1. Minto Pyramid Principle
- **Reference.** Barbara Minto, *The Pyramid Principle* (1987; earlier editions in the 1970s).
- **Essence.** Start from one governing thought. Below it sit a few key lines
  that each support it. Below each key line sit the facts that prove it.
  Ideas at one level form a group that is either a logical chain (deductive)
  or a set of parallel points of the same kind (inductive). Groups should not
  overlap and should together cover the case.
- **Fits.** Executives and reviewers who want the answer first. Decision and
  briefing decks, 5-30 minutes.
- **Roles.** `title → bluf → argument* → evidence* → recommendation → ask`
- **Pitfall.** Key lines that mix kinds (a reason, a step and a risk side by side). Each level must hold one kind of idea.

### 2. SCQA
- **Reference.** Minto, as the introduction to a pyramid (1987).
- **Essence.** Begin from facts the audience already accepts. Add the change
  that disturbs them. Name the question this raises. Then answer it, and
  let the pyramid support the answer.
- **Fits.** Decisions and persuasion for any audience that needs context first. 5-30 minutes.
- **Roles.** `title → situation → complication → question → answer → argument* → [evidence] → ask`
- **Pitfall.** A long situation. It is shared ground; two slides at most.

### 14. BLUF decision memo
- **Reference.** "Bottom line up front", a US military writing convention.
- **Essence.** The first slide states the decision needed and the
  recommendation. Everything after is backup the reader can stop reading at
  any point.
- **Fits.** Senior, time-poor readers; pre-reads; 2-10 minutes.
- **Roles.** `bluf → recommendation → argument* → risk → plan → ask`
- **Pitfall.** A bottom line that is a topic ("Q3 budget") instead of a decision ("Approve 10% more for Q3").

### 16. Consulting action-title storyline
- **Reference.** Common practice at strategy consultancies; builds on Minto.
  See [consulting-style-research.md](consulting-style-research.md).
- **Essence.** Every title is a full-sentence conclusion. Reading only the
  titles in order tells the whole argument (horizontal logic). Each slide body
  proves its own title and nothing else (vertical logic). Teams draft a
  "ghost deck" first: titles plus empty bodies, to test the story before any
  analysis is done.
- **Fits.** Analytical work, steering committees, long decks read without a speaker. 15-60 minutes.
- **Roles.** `title → bluf → agenda → (section → argument → evidence*)* → recommendation → plan → ask`
- **Pitfall.** Titles that restate the chart ("Sales by region") instead of the conclusion it supports.

### 17. Toulmin argument
- **Reference.** Stephen Toulmin, *The Uses of Argument* (1958).
- **Essence.** A claim rests on grounds (data). A warrant explains why the
  data supports the claim, and backing supports the warrant. A qualifier sets
  how strongly the claim holds. A rebuttal names when it would not hold.
- **Fits.** Contested or technical claims; reviewers who will push back. 5-15 minutes.
- **Roles.** `title → answer → evidence → argument → [evidence] → risk → recommendation`
  (answer = claim, first evidence = grounds, argument = warrant, second evidence = backing, risk = rebuttal and qualifier).
- **Pitfall.** Skipping the warrant. Data alone rarely shows why it supports the claim.

### 25. Assertion-evidence
- **Reference.** Michael Alley, *The Craft of Scientific Presentations* (2003; 2nd ed. 2013).
- **Essence.** Each slide has a one-sentence assertion as its headline and
  visual evidence as its body: a chart, photo or diagram, not bullet lists.
  The speaker explains; the slide shows.
- **Fits.** Scientific, engineering and technical talks. 10-30 minutes.
- **Roles.** `title → agenda → (argument → evidence)* → recap → close`
  (in practice most body slides are `evidence` with an assertion title).
- **Pitfall.** Assertions that are too long to read at a glance. Keep them to two lines.

## Story family

### 3. Duarte sparkline
- **Reference.** Nancy Duarte, *Resonate* (2010).
- **Essence.** A talk moves back and forth between what is and what could
  be. Each swing widens the gap the audience wants closed. It starts with the
  status quo, ends with a call to action, and closes on a picture of the
  better world once the idea is adopted (the "new bliss").
- **Fits.** Change, vision and keynote talks where the audience must move. 15-45 minutes.
- **Roles.** `title → current-state → future-state → (current-state → future-state)* → ask → future-state → close`
- **Pitfall.** Swings that repeat the same point. Each pair should add a new reason.

### 5. Pixar story spine
- **Reference.** Kenn Adams' improv "story spine" exercise; widely shared via
  Emma Coats' list of Pixar story tips (2011).
- **Essence.** A fill-in pattern: once there was a normal world, and every day
  something happened. One day something changed. Because of that, a chain of
  consequences followed. Until finally it resolved, and since then life is different.
- **Fits.** Customer stories, project retrospectives, culture talks. 5-15 minutes.
- **Roles.** `title → character → story → turning-point → story* → resolution → lesson`
- **Pitfall.** A chain of "and then" events. Each beat should be caused by the one before.

### 6. Hero's journey (simplified)
- **Reference.** Joseph Campbell, *The Hero with a Thousand Faces* (1949);
  Christopher Vogler, *The Writer's Journey* (1992), a screenwriter's version.
- **Essence.** A hero leaves the ordinary world after a call, meets a mentor,
  faces trials and a central ordeal, and returns changed with something of
  value. In a talk, the audience is the hero and the speaker is the mentor.
- **Fits.** Transformation stories, product adoption, keynotes. 15-45 minutes.
- **Roles.** `title → character → current-state → problem → solution → story* → turning-point → resolution → lesson → ask`
- **Pitfall.** Casting the speaker or the product as the hero. The audience should be.

### 12. Three-act structure
- **Reference.** Aristotle, *Poetics* (beginning, middle, end); Syd Field, *Screenplay* (1979).
- **Essence.** Act one sets up the world and the conflict. Act two develops
  and escalates it. Act three resolves it. Most talks fit this shape.
- **Fits.** General-purpose talks of almost any length. 10-45 minutes.
- **Roles.** `title → hook → situation → complication → section → argument* → turning-point → resolution → close`
- **Pitfall.** A long, flat middle act. Raise the stakes inside it.

### 18. Freytag pyramid
- **Reference.** Gustav Freytag, *Die Technik des Dramas* (1863).
- **Essence.** Five parts: exposition, rising action, climax, falling action,
  and the final unravelling. The climax sits in the middle, not at the end.
- **Fits.** Dramatic case studies, incident reviews told as stories. 10-30 minutes.
- **Roles.** `title → situation → complication → story* → turning-point → story → resolution → lesson`
- **Pitfall.** A long falling action. Once the peak passes, close quickly.

### 21. Kishōtenketsu
- **Reference.** A four-part structure from classical Chinese poetry, later common across East Asian writing.
- **Essence.** Introduce (ki), develop (shō), twist with an unexpected
  angle (ten), then reconcile both views (ketsu). It needs no conflict; the
  force comes from the shift in perspective.
- **Fits.** Insight and reframing talks; audiences tired of problem-solution pitches. 5-15 minutes.
- **Roles.** `title → situation → story → turning-point → lesson`
- **Pitfall.** A twist that does not connect back. The last part must join the twist to the opening.

### 22. Nested loops and sparkline variants
- **Reference.** Common speaking technique; Duarte's sparkline is a related shape.
- **Essence.** Open story A, then story B, then put the core message in the
  middle. Close B, then close A. Open loops hold attention across a long talk.
  Sparkline variants change how many swings there are and where the peak falls.
- **Fits.** Long keynotes with several stories. 20-60 minutes.
- **Roles.** `title → story → story → answer → evidence* → resolution → resolution → close`
- **Pitfall.** Forgetting to close a loop. Track each opened story.

## Persuasion family

### 7. Monroe's motivated sequence
- **Reference.** Alan H. Monroe, Purdue University, in his speech textbooks from the 1930s.
- **Essence.** Five steps: get attention, show a need, satisfy it with a
  solution, help the audience see the result, then ask for action.
- **Fits.** Calls to action, fundraising, policy proposals. 5-20 minutes.
- **Roles.** `title → hook → problem → stakes → solution → how-it-works → future-state → ask`
- **Pitfall.** A weak need step. If the audience does not feel the need, the rest fails.

### 8. AIDA
- **Reference.** Advertising model usually credited to E. St. Elmo Lewis (around 1898).
- **Essence.** Attention, interest, desire, action. Catch the eye, hold it
  with something relevant, make the audience want the outcome, then ask.
- **Fits.** Sales and launch decks, short demos. 3-10 minutes.
- **Roles.** `hook → problem → solution → benefit → traction → ask`
- **Pitfall.** Treating attention as the goal. A clever hook unrelated to the offer wastes it.

### 9. Problem-solution-benefit
- **Reference.** General sales and proposal practice; no single author.
- **Essence.** Name the pain, offer the fix, then say what the audience gains.
- **Fits.** Product and internal proposals. 5-15 minutes.
- **Roles.** `title → problem → cause → solution → how-it-works → benefit → ask`
- **Pitfall.** A solution looking for a problem. Validate the pain first.

### 10. Before-after-bridge
- **Reference.** Copywriting formula; no single author.
- **Essence.** Show the world today, show it after, then present your idea as
  the bridge between them.
- **Fits.** Very short pitches, one-pagers, lightning talks. 2-5 minutes.
- **Roles.** `current-state → future-state → solution → ask` (or one `contrast` for the first two)
- **Pitfall.** An "after" that is not believable. Ground it in a result or example.

### 19. PAS (problem-agitate-solve)
- **Reference.** Copywriting formula; no single author.
- **Essence.** Name the problem, make its cost felt, then solve it.
- **Fits.** Urgent fixes, security or reliability asks. 2-5 minutes.
- **Roles.** `problem → stakes → data-point → solution → ask`
- **Pitfall.** Over-agitating. Fear without a credible fix breeds denial.

### 20. FAB (features-advantages-benefits)
- **Reference.** Sales training practice; no single author.
- **Essence.** For each feature, say what it does better (advantage) and why
  this audience should care (benefit).
- **Fits.** Product demos, release notes for customers. 5-15 minutes.
- **Roles.** `title → solution → (how-it-works → benefit)* → competition → ask`
- **Pitfall.** Stopping at the feature. The benefit must be in the audience's terms.

## Report and teaching family

### 11. STAR
- **Reference.** Behavioral interviewing practice.
- **Essence.** Situation, task, action, result. Short and factual, with the result measured.
- **Fits.** Case studies, project reviews, award nominations. 3-10 minutes.
- **Roles.** `title → situation → problem → how-it-works → result → lesson`
  (problem = task, how-it-works = actions taken).
- **Pitfall.** Too much situation, too little result.

### 13. Inverted pyramid
- **Reference.** Newspaper writing practice from the 19th century.
- **Essence.** Most important facts first, then supporting detail, then
  background. The reader can stop at any point and keep the key news.
- **Fits.** Status updates, incident notices, newsletters. 2-10 minutes.
- **Roles.** `title → result → data-point → evidence* → situation`
- **Pitfall.** Burying the key change under context. Lead with what is new.

### 23. Tell-show-tell
- **Reference.** Traditional speaking advice; the exact origin is uncertain.
- **Essence.** Preview what you will cover, cover it, then sum up what you covered.
- **Fits.** Training, lectures, onboarding. 10-60 minutes.
- **Roles.** `title → agenda → (section → argument → evidence)* → recap → close`
- **Pitfall.** A recap that repeats the agenda word for word. Recap the takeaways, not the headings.

## Pitch family

### 15. Startup pitch deck
- **Reference.** Sequoia Capital's public pitch template; Guy Kawasaki,
  *The Art of the Start* (2004), with his 10/20/30 rule: about ten slides,
  twenty minutes, nothing smaller than 30-point type.
- **Essence.** A fixed order investors expect: purpose, problem, solution,
  why now, market, competition, product, business model, team, financials, and the ask.
- **Fits.** Investor meetings and demo days. 10-20 minutes.
- **Roles.** `title → problem → solution → how-it-works → market → competition → business-model → traction → team → plan → ask`
- **Pitfall.** Missing "why now". Add a `complication` slide when timing is the argument.

## Overlays and formats

These do not set a sequence. They constrain how any strategy is drawn.

### 4. Duarte slide:ology principles
- **Reference.** Nancy Duarte, *slide:ology* (2008).
- **Essence.** One idea per slide. Cut noise so the signal stands out. A slide
  should pass a glance test: the point is clear in about three seconds.
  Sketch on paper before opening the tool.
- **Fits.** Every deck. In wb-slide it maps to `density.one_idea_per_slide` and short titles.
- **Roles.** Any. Favour `statement` layouts (`hook`, `data-point`, `turning-point`) for single ideas.
- **Pitfall.** Splitting one idea across many sparse slides. One idea per slide also means one slide per idea.

### 24. Rule of three
- **Reference.** Classical rhetoric (the tricolon); a common speaking habit.
- **Essence.** Lists of three feel complete and are easy to remember.
- **Fits.** Any deck. Group `argument`, `benefit` and `recap` items in threes.
- **Roles.** `answer → argument → argument → argument → recap`
- **Pitfall.** Forcing a fourth point into three, or padding two into three.

### 26. Lessig and Takahashi styles
- **Reference.** Lawrence Lessig's talks (early 2000s); Masayoshi Takahashi's method (2001).
- **Essence.** Many slides, each with a few words or one huge phrase,
  advancing in time with the speech. Takahashi uses large plain text only.
- **Fits.** Energetic talks where the speaker carries the content. Not for reading decks.
- **Roles.** Mostly roles on the `statement` layout: `hook`, `data-point`, `turning-point`, `lesson`, `close`.
- **Pitfall.** The deck means nothing without the speaker. Do not send it as a pre-read.

### 27. Pecha Kucha and Ignite
- **Reference.** Pecha Kucha: Astrid Klein and Mark Dytham, Tokyo (2003), 20 slides × 20 seconds.
  Ignite: 20 slides × 15 seconds, started in 2006.
- **Essence.** Fixed slide count and auto-advancing timing. The constraint forces focus and rehearsal.
- **Fits.** Showcases and community events. 5-7 minutes.
- **Roles.** Any strategy compressed to 20 slides; Pixar spine and sparkline fit well.
- **Pitfall.** Dense slides. At 15-20 seconds each, use `statement` and `title-image` layouts.

## Cross-cutting guidance

### Title styles

| Style | Example | Use when |
|---|---|---|
| Action title | "Sunday trial pays back in 12 weeks" (fictional) | Logic and pitch strategies; decks read without a speaker. |
| Topic title | "Sunday trial" | Section dividers, agendas, Lessig-style decks. |
| Question title | "Will regulars follow us to Sunday?" | `question` role, teaching, opening a sparkline swing. |

An action title is one full sentence, under about 15 words, with the conclusion
in it. Reading only the titles should tell the story. Story strategies may use
short evocative titles instead, but each still names one idea.

### Density rules

| Rule | Default |
|---|---|
| Ideas per slide | One |
| Bullets | 3-5 (`title-list`), never nested deeper than one level |
| Words per slide | About 50 for spoken decks; reading decks may go higher |
| Cards or columns | 2-3 |
| Statement slides | One sentence or one number |
| Glance test | Point readable in about three seconds |

### Choosing a strategy

| Audience | Goal | Duration | First choice | Alternative |
|---|---|---|---|---|
| Executive | Decide | ≤ 10 min | BLUF | Minto Pyramid |
| Executive | Decide | 10-30 min | SCQA | Consulting storyline |
| Team | Inform | ≤ 10 min | Inverted pyramid | STAR |
| Team | Teach | 10-60 min | Tell-show-tell | Assertion-evidence |
| Customer | Sell | ≤ 10 min | AIDA | Before-after-bridge |
| Customer | Persuade | 10-20 min | Problem-solution-benefit | FAB |
| Investor | Fund | 10-20 min | Startup pitch deck | SCQA |
| Broad / conference | Inspire | 15-45 min | Duarte sparkline | Hero's journey |
| Broad / conference | Entertain, share | 5-7 min | Pecha Kucha + Pixar spine | Kishōtenketsu |
| Technical reviewers | Convince | 10-30 min | Assertion-evidence | Toulmin |
| Any | Mobilise action | 5-20 min | Monroe's sequence | PAS |

Overlays (slide:ology, rule of three) apply on top of any choice.

## How roles are shared

| Role | Used by |
|---|---|
| `title` | Most sequence strategies (not AIDA, BAB, PAS, BLUF) |
| `hook` | Three-act, Monroe, AIDA, Lessig |
| `agenda` | Consulting storyline, assertion-evidence, tell-show-tell |
| `bluf` | Minto, BLUF, consulting storyline |
| `situation` | SCQA, three-act, Freytag, Kishōtenketsu, STAR, inverted pyramid |
| `complication` | SCQA, three-act, Freytag, pitch deck (why now) |
| `question` | SCQA |
| `answer` | SCQA, Toulmin, nested loops, rule of three |
| `argument` | Minto, SCQA, BLUF, consulting, Toulmin, assertion-evidence, three-act, tell-show-tell, rule of three |
| `evidence` | Minto, SCQA, consulting, Toulmin, assertion-evidence, nested loops, inverted pyramid, tell-show-tell |
| `problem` | Hero's journey, Monroe, AIDA, PSB, PAS, STAR, pitch deck |
| `cause` | PSB |
| `stakes` | Monroe, PAS |
| `current-state` | Sparkline, hero's journey, BAB |
| `future-state` | Sparkline, Monroe, BAB |
| `contrast` | BAB (compact form) |
| `solution` | Hero's journey, Monroe, AIDA, PSB, BAB, PAS, FAB, pitch deck |
| `how-it-works` | Monroe, PSB, FAB, STAR, pitch deck |
| `benefit` | AIDA, PSB, FAB |
| `options` | None yet; candidate for consulting and BLUF variants |
| `recommendation` | Minto, BLUF, consulting, Toulmin |
| `risk` | BLUF, Toulmin |
| `plan` | BLUF, consulting, pitch deck |
| `ask` | Minto, SCQA, BLUF, consulting, sparkline, hero's journey, Monroe, AIDA, PSB, BAB, PAS, FAB, pitch deck |
| `story` | Pixar spine, hero's journey, Freytag, Kishōtenketsu, nested loops |
| `character` | Pixar spine, hero's journey |
| `turning-point` | Pixar spine, hero's journey, three-act, Freytag, Kishōtenketsu, Lessig |
| `resolution` | Pixar spine, hero's journey, three-act, Freytag, nested loops |
| `lesson` | Pixar spine, hero's journey, Freytag, Kishōtenketsu, STAR, Lessig |
| `result` | STAR, inverted pyramid |
| `data-point` | PAS, inverted pyramid, Lessig |
| `market` | Pitch deck |
| `traction` | AIDA, pitch deck |
| `business-model` | Pitch deck |
| `competition` | FAB, pitch deck |
| `team` | Pitch deck |
| `section` | Consulting, three-act, tell-show-tell |
| `recap` | Assertion-evidence, tell-show-tell, rule of three |
| `close` | Sparkline, three-act, nested loops, assertion-evidence, tell-show-tell, Lessig |

`ask`, `argument` and `evidence` are the most shared roles, so their
purposes and layouts deserve the most care. `options` is unused here; keep it
for decision variants rather than remove it.

## Out of scope and caveats

- **Simplified maps.** Each sequence is one reasonable reading, not the
  canonical form. Real decks merge, split and reorder roles.
- **No claims of effect.** This note does not say any strategy performs
  better than another. We cite no studies.
- **Attribution.** Several formulas (PAS, BAB, FAB, PSB) have no clear
  author. Where origin is disputed we say so instead of guessing.
- **Original wording.** Summaries are paraphrased in our own words. Do not
  paste book text into strategy JSON or examples.
- **Not covered.** Speaker delivery, rehearsal, visual design and data
  charting. Themes and layouts cover the visual side ([layouts.md](layouts.md)).
- **Culture.** Answer-first logic suits many business settings but not all;
  Kishōtenketsu and story strategies can fit better where directness reads as abrupt.
- **Examples.** Strategy JSON examples must use fictional topics and mark
  any numbers as synthetic, as `scqa.json` does.
