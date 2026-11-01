---
phase: 13-web-ui-redesign
plan: 11
subsystem: ui
tags: [react, vitest, words, morphology, lexicon, counts, tool-ledger, deep-links]

# Dependency graph
requires:
  - phase: 13-web-ui-redesign
    provides: Shared ToolLedger and tool-result helpers from plan 10 (13-10-SUMMARY)
provides:
  - `/words` route with morphology, root, lemma, family and all five count instruments
  - `words.ts` input parser (`parseWordInput`, `tokenId`, `familyKindFor`) with unit tests
  - `DIRECTORY` export and `NAVIGATION` export both covering `/words`
  - `isActive` with `also` ordering pinned by test
affects: [13-12, 13-13, 13-14]

# Actuals (#2632) — pairs with the plan's `estimate` to calibrate future estimates.
# Same estimateTokens scale (chars/4 over the realized diff), never a harness token count.
actuals:
  tokens: 13490
  tasks: 3
  commits: 3

# Tech tracking
tech-stack:
  added: []
  patterns: [shared ToolLedger reuse for lexicon-adjacent views, pinned-edition deep links, rules-beside-number count output]

key-files:
  created:
    - web/src/lib/words.ts
    - web/src/lib/words.test.ts
    - web/src/views/WordsView.tsx
    - web/src/views/WordsView.test.tsx
    - web/src/router.test.tsx
  modified:
    - web/src/router.tsx
    - web/src/components/layout/site-header.tsx
    - web/src/components/command/command-menu.tsx

key-decisions:
  - "Lexicon kind/id/target overrides reset on main-input change so defaults always reflect the current query"
  - "Co-occurrence shows the window reason first when both target and window are invalid"
  - "Unrecognized count shapes render an honest fallback line instead of crashing the section"
  - "Counts share one target/profile/window row with five independent run controls and labelled outputs"

patterns-established:
  - "Deep links build only from the single pinned edition in edition_versions; otherwise plain text plus the margin note"
  - "A count without visible rules is not rendered — zero counts keep the rules block beside the empty state"

requirements-completed: [REQ-ux-web, REQ-quran-research-tools, REQ-quran-result-contract, REQ-quran-display]

# Coverage metadata (#1602) — one entry per shipped deliverable. Drives DETERMINISTIC UAT routing in verify-work.
coverage:
  - id: D1
    description: "/words appears in ROUTES, DIRECTORY and NAVIGATION, and the Research rail entry stays active while /words is open"
    requirement: "REQ-ux-web"
    verification:
      - kind: unit
        ref: "src/router.test.tsx (3 tests)"
        status: pass
    human_judgment: false
  - id: D2
    description: "Token/ayah submit runs quran.morphology and renders the analyses through the shared ToolLedger"
    requirement: "REQ-quran-result-contract"
    verification: []
    human_judgment: true
    rationale: "No jsdom test covers the morphology submit path; needs a live engine returning analyses for 2:1:3"
  - id: D3
    description: "Root, lemma and family results render with dataset attribution and pinned-edition reader deep links"
    requirement: "REQ-quran-display"
    verification:
      - kind: unit
        ref: "src/views/WordsView.test.tsx lexicon block (4 tests)"
        status: pass
    human_judgment: false
  - id: D4
    description: "All five count instruments render their number beside the rules block, checksum and by-surah table"
    requirement: "REQ-quran-research-tools"
    verification:
      - kind: unit
        ref: "src/views/WordsView.test.tsx counts block (5 tests)"
        status: pass
    human_judgment: false
  - id: D5
    description: "Cold-loading /words, the rail highlight, the ⌘K entry and the reader directory link work against a live engine"
    requirement: "REQ-ux-web"
    verification: []
    human_judgment: true
    rationale: "jsdom covers route wiring only; reaching /words cold and running tools needs the served build plus a running engine"

# Metrics
duration: 13min
completed: 2026-11-01
status: complete
---

# Phase 13 Plan 11: Words & Roots Summary

**New `/words` route where morphology, root, lemma, family and all five counts run from the browser, every number printed beside the rules that produced it**

## Performance

- **Duration:** ~13 min
- **Started:** 2026-11-01T06:12Z
- **Completed:** 2026-11-01T06:25Z
- **Tasks:** 3
- **Files modified:** 8

## Accomplishments

- `words.ts` parses `surah:ayah:position` / `surah:ayah` / free text into a discriminated query, with `tokenId` and `familyKindFor` defaults — all branch-tested including `0:0` and `2:1:x`
- `/words` wired into all four navigation surfaces: `ROUTES`, exported `DIRECTORY`, rail `also` on Research (nine destinations unchanged), and exported `NAVIGATION` for ⌘K
- `WordsView` runs `quran.morphology` through the shared `ToolLedger`, then root/lemma/family through occurrence tables and full-text family lists with pinned-edition deep links
- Counts section runs all five count routes with shared target/profile/window inputs and five independent ruled outputs; zero is an explicit ruled empty state

## Task Commits

Each task was committed atomically:

1. **Task 1: End-to-end `/words` — one route, one reference, one morphology result** - `ebf1305` (feat)
2. **Task 2: Root, lemma and family lookups with canonical deep links** - `01df352` (feat)
3. **Task 3: Five count instruments with their rules block** - `7fdf019` (feat)

## Files Created/Modified

- `web/src/lib/words.ts` - `parseWordInput`, `tokenId`, `familyKindFor` with the `WordQuery` union
- `web/src/lib/words.test.ts` - Branch tests for every parse shape plus both helpers
- `web/src/views/WordsView.tsx` - Morphology form, lexicon selector with kind/id controls, counts section with ruled outputs
- `web/src/views/WordsView.test.tsx` - Lexicon (4) and counts (5) tests on stubbed fetch
- `web/src/router.test.tsx` - ROUTES/NAVIGATION/DIRECTORY membership and `also`-before-`/` ordering
- `web/src/router.tsx` - `/words` route, exported `DIRECTORY` with the Words entry, route-switch branch
- `web/src/components/layout/site-header.tsx` - Optional `also` on every `NAV` entry, ordered `isActive`, `also` passed at both render sites
- `web/src/components/command/command-menu.tsx` - `navigation` exported as `NAVIGATION` with the Words & Roots entry

## Decisions Made

- Lexicon kind/id/target overrides reset whenever the main input changes, so the selector default (root for text, family for token) and the prefilled family id always reflect the current query rather than a stale one.
- The co-occurrence run control reports the window reason first when both target and window are invalid — the window is the instrument-specific constraint.
- Count outputs type-guard the server payload and render an honest "unrecognized shape" line on mismatch instead of crashing the section.
- The task-1 notice for text queries ("run the lexicon lookup below") replaced the transient "attaches here next" wording once task 2 landed in the same plan.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Test asserted a single `stored-forms` text node but rows plus attribution render three**
- **Found during:** Task 2 (WordsView lexicon tests)
- **Issue:** `getByText('stored-forms')` throws on multiple matches — two occurrence rows plus the dataset-attribution list
- **Fix:** Assert `getAllByText('stored-forms').length` is at least 2
- **Files modified:** web/src/views/WordsView.test.tsx
- **Verification:** `npx vitest run src/views/WordsView.test.tsx` — 4 passed (9 after task 3)
- **Committed in:** 01df352 (part of task commit)

---

**Total deviations:** 1 auto-fixed (test correctness)
**Impact on plan:** No scope change; the fix keeps the verification green.

## Issues Encountered

- A mid-task edit briefly consumed the `export const ROUTES = [` header line while extracting `DIRECTORY`; repaired immediately and verified via `tsc -b` before committing. No commit ever contained the broken state.
- No worktree is present in this checkout, so all three task commits landed on `main` — following the prior wave's established practice (`02617c8`, `960fcf0`, `a2304e7` all on `main`).
- `npx vitest run ... | grep` swallows the result table in this shell (same as plan 10); rerunning with `tail` showed the failures.

## Threat Flags

| Flag | File | Description |
|------|------|-------------|
| threat_flag: reviewed | web/src/views/WordsView.tsx | New DOM sinks rendering server occurrence/family/count rows and `href`s built only from server `edition_versions`; no `dangerouslySetInnerHTML` in any new file (verified by scan) |

T-13-05 mitigated: links build only from the single pinned edition, otherwise plain text plus the margin note. T-13-06 mitigated: `RulesBlock` renders unconditionally beside every count including zeros, checksum in mono. T-13-07 mitigated: params go through `JSON.stringify` in the existing `post()` client. T-13-08 mitigated: window parsed as a positive integer, control disabled otherwise. T-13-SC clean: no dependencies added (`package.json` untouched).

## Known Stubs

None. The only `placeholder` matches in new code are input hint attributes, not stubs. No empty arrays/objects flow to rendered output; empty results render named empty states.

## Self-Check: PASSED

- All eight files exist on disk; all three task commits are ancestors of HEAD (`ebf1305`, `01df352`, `7fdf019` verified via `git merge-base --is-ancestor`).
- `npm run build` passes; full plan verification `npx vitest run src/lib/words.test.ts src/router.test.tsx src/views/WordsView.test.tsx`: 3 files / 25 tests pass.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `DIRECTORY` and `NAVIGATION` exports give plans 13-12–13-14 the pattern for adding routes to all four surfaces without touching the rail count.
- `pinnedEdition`/`DatasetAttribution` inside `WordsView.tsx` are the reusable shape for any later view that deep-links server rows; extract to `lib/` if a second view needs them.
- Manual checks still open: cold-load `/words` and run a token, a root and each count against a live engine (covered as D2/D5 human-judgment items).

---
*Phase: 13-web-ui-redesign*
*Completed: 2026-11-01*
