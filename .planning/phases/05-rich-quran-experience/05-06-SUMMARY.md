---
phase: 05-rich-quran-experience
plan: 06
subsystem: ui
tags: [spa, reading-view, layer-tokens, rtl, vitest, typed-client, ADR-0112, D-09]

# Dependency graph
requires:
  - phase: 05-rich-quran-experience plan 04
    provides: web/ scaffold, pinned toolchain (vitest/jsdom/TL), embedded serving shell
provides:
  - Layer-token set + themes + canonical/translation/annotation components with type-level ADR-0112 guard
  - RTL reading view (3×3 display modes, deep-link route) + typed Envelope/Meta client with resolve/citation/tool methods
affects: [05-09, 05-08, 05-07]

# Actuals (#2632) — pairs with the plan's `estimate` to calibrate future estimates.
actuals:
  tokens: 4200
  tasks: 3
  commits: 2

# Tech tracking
tech-stack:
  added: []
  patterns: [canonical-only props as ADR-0112 type guard, ts-expect-error tripwire, view-model AyahView subset client-side]

key-files:
  created: [web/src/tokens/layers.css, web/src/tokens/themes.css, web/src/components/AyahColumn.tsx, web/src/components/TranslationPanel.tsx, web/src/components/WordInspector.tsx, web/src/components/LayerBadge.tsx, web/src/components/AyahColumn.test.tsx, web/src/display/DisplayModeControl.tsx, web/src/views/ReadingView.tsx, web/src/views/ReadingView.test.tsx, web/src/api/client.ts, web/src/router.tsx, web/vitest.config.ts]
  modified: [web/src/App.tsx]

key-decisions:
  - "Canonical slot takes {arabicText, reference} only (mirrors Example 5); translations travel as {translator, text} pairs in a separate prop — no shared prop, ever"
  - "Client AyahView is a structural subset (canonical + translations); full server shapes flow through as typed unknowns where unneeded"
  - "ReadingView renders the given order verbatim (no client sort); WordInspector popover/panel/pin mirrors the TUI modes"

patterns-established:
  - "@ts-expect-error tripwire: if a translation prop is ever added to AyahColumn, tsc -b fails"
  - "Dataset text renders as text (React escaping); no dangerouslySetInnerHTML anywhere"

requirements-completed: [REQ-quran-display]

# Coverage metadata (#1602)
coverage:
  - id: D1
    description: "Layer tokens + themes exist; components render under matching token classes; canonical slot rejects translation props"
    requirement: "REQ-quran-display"
    verification:
      - kind: unit
        ref: "npx vitest run src/components/AyahColumn.test.tsx (5/5)"
        status: pass
      - kind: unit
        ref: "npm run build (tsc -b typechecks the @ts-expect-error guard)"
        status: pass
    human_judgment: false
  - id: D2
    description: "Reading view: 3 translation × 3 inspector modes, RTL, adjacency separation, empty-translation state, byte-exact canonical"
    requirement: "REQ-quran-display"
    verification:
      - kind: unit
        ref: "npx vitest run src/views/ReadingView.test.tsx (4/4)"
        status: pass
    human_judgment: false
  - id: D3
    description: "Locked D-09/ADR-0112 contract confirmed before view structure landed"
    requirement: "REQ-quran-display"
    verification:
      - kind: other
        ref: "owner checkpoint answer: proceed-as-locked"
        status: pass
    human_judgment: true
    rationale: "One-way structural embodiment of a user-locked contract requires explicit owner confirmation; automation cannot ratify irreversibility."

# Metrics
duration: ~1.5h wall
completed: 2026-10-09
status: complete
---

# Phase 05 Plan 06 Summary

**SPA reading experience ships: layer-token themes, canonical-only components with a type-level ADR-0112 guard, RTL reading view with 3×3 switchable modes, deep-link routing, and a typed Envelope/Meta client.**

## Performance

- **Duration:** ~1.5h wall
- **Started:** 2026-10-09
- **Completed:** 2026-10-09
- **Tasks:** 3 completed
- **Files modified:** 14

## Accomplishments

- Blocking decision checkpoint cleared by the owner (`proceed-as-locked` — D-09/ADR-0112 implemented exactly)
- `layers.css`/`themes.css` (light/dark/sepia/contrast) + `AyahColumn` (canonical-only props), `TranslationPanel`,
  `WordInspector` (popover/panel/pin), `LayerBadge` — 5/5 Vitest green, `tsc -b` locks the no-translation-prop guard
- `ReadingView` (centered RTL column, collapsible side panels, adjacency separation, explicit empty-translation
  state, byte-exact canonical) + `DisplayModeControl` + `/read/{slug}@{version}/{s}:{a}` route + typed
  `api/client.ts` (ayahs/resolve/citation/tool) — 4/4 Vitest green, `npm run build` green
- `App.tsx` remounted on the router with theme selector; no `dangerouslySetInnerHTML`, no external URLs

## Task Commits

1. **Task 1: Confirm layer-separation contract (blocking-decision)** - owner answer recorded here (no code)
2. **Task 2: Layer tokens + components** - `eb8ae78` (feat)
3. **Task 3: Reading view + client** - `3a2c5ca` (feat)

**Plan metadata:** `HEAD` (docs: this file)

## Files Created/Modified

- `web/src/tokens/layers.css`, `web/src/tokens/themes.css` - three layer families + 3 theme overrides
- `web/src/components/*.tsx` - canonical/translation/inspector/badge components
- `web/src/display/DisplayModeControl.tsx` - 3×3 mode switcher
- `web/src/views/ReadingView.tsx` (+ test) - RTL column, modes, deep-link data, empty states
- `web/src/api/client.ts` - `Envelope<T>`/`Meta`/`AyahView` types + ayahs/resolve/citation/tool fetchers
- `web/src/router.tsx` - `/read/…` route with loading/error states
- `web/src/App.tsx` - router mount + theme selector + token imports
- `web/vitest.config.ts` - jsdom + src tests

## Decisions Made

- Test-toolchain versions come from the 05-04 checkpoint; no new npm packages in this plan (T-05-SC).
- `WordInspector` "•" placeholder trigger in the reading view (real token wiring is research-view work in 05-09).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Vitest jsdom accumulates renders across tests without cleanup**
- **Found during:** Task 3 (order/adjacency assertions saw 5 units instead of 2)
- **Issue:** No global cleanup setup; `document` accumulated renders file-wide.
- **Fix:** `afterEach(() => cleanup())` in the view test (component test passed only because it ran solo first).

**2. [Rule 2 - Missing] Stray placeholder token left in router.tsx**
- **Found during:** Task 3 (self-review before commit)
- **Issue:** `funcref_placeholder() {}` artifact after the import.
- **Fix:** Removed before running tests.

## Verification Results

- `npx vitest run` — 9/9 pass (5 component + 4 view)
- `npm run build` (`tsc -b` + vite) — success, `dist/index.html` produced
- No `dangerouslySetInnerHTML`; no external URLs (scanned)
