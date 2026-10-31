# Reference Analysis — `temp-inspire/`

## Method

The four generated sites were copied to a scratch directory (the originals are
read-only) and each was installed and started with Vite on its own port. Where a
site rendered, desktop screenshots were captured with Playwright; where it did
not, the source was read instead. Q-ai was inspected at `web/` (Vite + React 19
+ Tailwind v4 + shadcn primitives).

| Folder | Runtime | Evidence used |
|---|---|---|
| `complete-phase-13-roadmap/` | renders | Desktop screenshot (Reader), `index.css` (6.6k lines), `App.tsx` |
| `complete-phase-13-roadmap (1)/` | renders | Desktop screenshot (Overview), `index.css` (5.2k lines), 4 views |
| `complete-phase-13-roadmap (2)/` | **crashes on mount** (`useEffect` returns a promise → React error) | `index.css` tokens, 16 views, 4 components (`Shell`, `Layers`, `Background`, `ui`) |
| `complete-phase-13-roadmap (3) 2/` | **crashes on mount** (same bug) | near-identical to (2) |

The two crashes are developmental bugs in the references, not a blocker: their
design language is fully legible from the CSS token block and the shell source.

## What each template actually is

**`complete-phase-13-roadmap/` — “warm scholarly console”.**
Forest-green rail (`#506747`), off-white `#fcfcfa`, DM Sans, a 230 px sidebar
with a workspace switcher, and a reader that puts canonical Arabic, translation
and notes in separate labelled blocks. Dense but calm. The canvas has generous
margins and the reading column is deliberately narrow.

**`complete-phase-13-roadmap (1)/` — “editorial overview”.**
Same green family, but built around a landing-style Overview: large serif-feeling
headline (“A space for deeper understanding.”), a “continue reading” hero plate,
two-column path lists, and a search/prompt card. The most product-marketing of
the four; the least instrument-like.

**`complete-phase-13-roadmap (2)` / `(3) 2` — “provenance dashboard”.**
Cool grey `#f5f6f8` canvas, white cards, Inter, and a well-defined provenance
palette: canon `#047857`, translation `#2563eb`, annotation `#b45309`,
computed `#7c3aed`. 16 views, hash routing, ambient background orbs, and
shell-level mobile drawer. The most domain-aware information architecture.

## Comparative read

| Dimension | Strongest | Why |
|---|---|---|
| Atmosphere | (0) | The green + warm paper pairing reads as scholarship, not SaaS |
| Shell discipline | (1) | Clear rail → header → workspace hierarchy and a real mobile drawer |
| Domain vocabulary | (2)/(3) | Provenance layers are part of the layout grammar, not a legend afterthought |
| Data density | (2)/(3) | Rows, chips and ruled sections handle text-heavy results well |
| Typographic scale | (0)/(1) | Confident display sizes and quiet small caps |
| Weaknesses | all | Every one is a **left-sidebar dashboard**; three share the same forest-green/cream mood; two do not run |

## Why this pass does not use one as the base

The previous Q-ai pass had already drifted into that shared silhouette — ink
sidebar, parchment canvas, gold rules, sidebar-grouped navigation. Blending more
of the same would reproduce a look that the references already own. This pass
therefore treats them as **evidence about the problem**, not as a template:

**Adopted (idea, re-expressed):**

- Layered provenance as a structural vocabulary — but as hairline rules,
  dots and mono labels instead of tinted cards (2/3).
- Margin/legend column beside the primary text, so layer meaning is visible
  while reading (0 + 2).
- A real display scale with one hero size per page and mono metadata (0/1).
- A command surface (⌘K) and a mobile drawer with proper focus behaviour (1).

**Deliberately rejected:**

- Left sidebar navigation for five destinations; it steals reading width and is
  the references' most repeated trait.
- Forest green, parchment, gold ornament, ambient gradient orbs.
- Card-per-item layouts for verses and search hits.
- Decorative KPI/landing hero compositions that imply features the API lacks.

**Resulting direction:** a horizontal top rail, cool porcelain ground, one deep
indigo instrument accent, serif for the written word, mono for references, and
hairline structure. Documented in `UI-SPEC.md`.
