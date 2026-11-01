---
phase: 13-web-ui-redesign
plan: 13
subsystem: ui
tags: [react, vitest, localStorage, reader, bookmarks, corpus-trace, rtl, amiri]

# Dependency graph
requires:
  - phase: 13-web-ui-redesign
    provides: Shared ToolResult contract and ToolLedger renderer (13-10-SUMMARY)
  - phase: 13-web-ui-redesign
    provides: ROUTES/DIRECTORY/NAVIGATION/isActive-also route-wiring pattern (13-11-SUMMARY)
  - phase: 13-web-ui-redesign
    provides: ApiRequestError.remedy and catalogue-driven tool discipline (13-12-SUMMARY)
provides:
  - Size/layout-persisted ReadingView with three Arabic sizes and three canonical/translation layouts
  - Browser-local bookmark list with pinned-reference jump links via deepLinkFor
  - Per-verse on-demand tafsir/hadith trace with printed seed and seeded-hits disclaimer
affects: [13-14]

# Actuals (#2632) — pairs with the plan's `estimate` to calibrate future estimates.
# Same estimateTokens scale (chars/4 over the realized diff), never a harness token count.
actuals:
  tokens: 12578
  tasks: 3
  commits: 3

# Tech tracking
tech-stack:
  added: []
  patterns: [storage-object-injected prefs/bookmarks with tolerant per-field fallback, layout-owned translation frame with disabled row, seeded-query corpus trace with printed disclaimer]

key-files:
  created:
    - web/src/lib/reader-storage.ts
    - web/src/lib/reader-storage.test.ts
    - web/src/lib/related.ts
    - web/src/lib/related.test.ts
    - web/src/display/DisplayModeControl.test.tsx
  modified:
    - web/src/components/AyahColumn.tsx
    - web/src/components/AyahColumn.test.tsx
    - web/src/display/DisplayModeControl.tsx
    - web/src/views/ReadingView.tsx
    - web/src/views/ReadingView.test.tsx

key-decisions:
  - "Layout row reports with a `layout ` name prefix so translation names stay exact-matchable in tests"
  - "Empty search results count as their literal length; only non-array non-hits shapes print unknown"
  - "RelatedBlock lives directly under the verse header so button and panel share one per-verse state"

patterns-established:
  - "Browser-writable storage never reaches a class name or href unchecked — closed allow-lists with per-field fallback"
  - "Bookmark hrefs build only through deepLinkFor; unpinned references render as plain mono text"
  - "Corpus-trace panels print the exact seed in mono plus RELATED_NOTE beside every count"

requirements-completed: [REQ-ux-web, REQ-quran-display, REQ-hadith-corpus, REQ-tafsir-comparative]

# Coverage metadata (#1602) — one entry per shipped deliverable. Drives DETERMINISTIC UAT routing in verify-work.
coverage:
  - id: D1
    description: "Three sizes and three layouts work end-to-end, persist across reload, Arabic invariants hold in every combination"
    requirement: "REQ-quran-display"
    verification:
      - kind: unit
        ref: "src/lib/reader-storage.test.ts + src/components/AyahColumn.test.tsx + src/display/DisplayModeControl.test.tsx + src/views/ReadingView.test.tsx (43 tests across the five plan files)"
        status: pass
    human_judgment: false
  - id: D2
    description: "Verses can be bookmarked and un-bookmarked, list persists in localStorage, each entry jumps to its deep link"
    requirement: "REQ-ux-web"
    verification:
      - kind: unit
        ref: "src/lib/reader-storage.test.ts bookmarks block + src/views/ReadingView.test.tsx bookmark toggle/remove tests"
        status: pass
    human_judgment: false
  - id: D3
    description: "A verse traces into the tafsir and hadith corpora on demand with seed and disclaimer visible, counts never fabricated"
    requirement: "REQ-tafsir-comparative"
    verification:
      - kind: unit
        ref: "src/lib/related.test.ts + src/views/ReadingView.test.tsx related lookup block (seed/counts/error/loading/no-prefetch)"
        status: pass
    human_judgment: false
  - id: D4
    description: "Change size and layout, reload and confirm they persist; bookmark two verses and follow both links; press related on one verse and read the seed and the note"
    verification: []
    human_judgment: true
    rationale: "Needs the served build in a real browser with localStorage across a reload; jsdom covers the contract only"

# Metrics
duration: 9min
completed: 2026-11-01
status: complete
---

# Phase 13 Plan 13: Adjustable Reading Room Summary

**Three Arabic sizes, three canonical/translation layouts, browser-local bookmarks with jump links, and an on-demand tafsir/hadith trace that prints its seed — all persisted, all labelled, Arabic invariants intact**

## Performance

- **Duration:** ~9 min
- **Started:** 2026-11-01T03:05:19Z
- **Completed:** 2026-11-01T03:13:53Z
- **Tasks:** 3
- **Files modified:** 10

## Accomplishments

- `reader-storage.ts` holds prefs (`sm/md/lg`, `stacked/side-by-side/interlinear`, translation + inspector modes) and bookmarks under `qai.reader.prefs.v1` / `qai.reader.bookmarks.v1`; loads never throw, bad fields fall back per-field, malformed entries are dropped
- `AyahColumn` takes an optional `size` prop mapped to three explicit pairs (`text-[1.75rem] leading-[2.15]`, `text-[2rem] leading-[2.15]`, `text-[2.25rem] leading-[2.2]`) inside the UI-SPEC §4/§15 bands, with `data-ayah-size` on the `<p>`
- `DisplayModeControl` gains a `layout` row; off-`stacked` the translation row renders `aria-disabled`, keeps buttons focusable, shows the reason, and ignores clicks
- `ReadingView` reads prefs once on mount, writes back on change, carries `data-ayah-size`/`data-ayah-layout`, passes `size` to every column, and branches each verse into stacked / two-cell side-by-side (canonical first in DOM, `dir="ltr"` translation cell) / interlinear (`data-interlinear` hairline-ruled lines)
- Bookmarks toggle from each verse header (`bookmark`/`bookmarked`, `aria-pressed`), land in a margin `Bookmarks` block with count, mono label, `deepLinkFor` jump link and per-verse remove; empty state plus margin note state the browser-local boundary
- `related.ts` exports `seedQuery` (first 40 code points via `Array.from`, never splitting a mark or surrogate pair) and `RELATED_NOTE`; each verse gets an on-demand block firing `tafsir.search` + `hadith.search` concurrently, printing the mono seed, both counts (rows, `hits` array, else `unknown`) and per-domain `code — summary` errors

## Task Commits

Each task was committed atomically:

1. **Task 1: End-to-end size and layout — three sizes, three layouts, persisted** - `5082035` (feat)
2. **Task 2: localStorage bookmarks with jump links** - `63a28dc` (feat)
3. **Task 3: Per-verse related-text lookup, labelled for what it is** - `fe33f57` (feat)

## Files Created/Modified

- `web/src/lib/reader-storage.ts` - ReaderPrefs + Bookmark helpers, tolerant loads, two localStorage keys
- `web/src/lib/reader-storage.test.ts` - Prefs defaults/partial/non-object/round-trip; bookmark add/dupe/remove/malformed/dedupe
- `web/src/lib/related.ts` - `seedQuery` + `RELATED_NOTE`
- `web/src/lib/related.test.ts` - 60-code-point truncation with emoji boundary, trim, whitespace-only, note wording
- `web/src/components/AyahColumn.tsx` - Optional `size` prop, three pinned class pairs, `data-ayah-size`
- `web/src/components/AyahColumn.test.tsx` - Default md, each size pinned literally, rtl/lang/class at all sizes
- `web/src/display/DisplayModeControl.tsx` - Layout row with `layout ` prefix, `disabledReason` on ModeRow
- `web/src/display/DisplayModeControl.test.tsx` - Active layout reported; translation row disabled off-stacked, enabled when stacked
- `web/src/views/ReadingView.tsx` - Prefs/bookmarks state + persistence, size stepper, three layout branches, Bookmarks block, RelatedBlock
- `web/src/views/ReadingView.test.tsx` - Layout markers, Arabic invariants per layout, stepper, reload persistence, bookmark toggle/remove, related seed/counts/error/loading/no-prefetch

## Decisions Made

- The layout row uses a `layout ` accessible-name prefix (mirroring the existing `inspector ` prefix) so `getByRole('button', {name: 'side-by-side'})` stays an exact single match for the translation button; the ReadingView translation-mode test was tightened from regex to exact names for the same reason.
- `countHits` treats an empty results array as the literal `0` (`rowsOf` applies) and reserves `unknown` strictly for non-array, non-`{hits}` shapes — the plan's "never a fabricated 0" targets the fallback path, not an honestly empty result.
- `RelatedBlock` sits directly under the verse header row inside the same article so the button, loading state and result panel share one per-verse state object; the header row itself keeps only the bookmark toggle to avoid overloading the ruled index line.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- First `DisplayModeControl.test.tsx` run failed with multiple `inline` matches: the file lacked `afterEach(cleanup)` so renders accumulated across tests. Added cleanup; 3/3 pass.
- `npm run build` failed on the test file after task 3: unused `PREFS_KEY` import (TS6133) and `ToolResultData` imported from `api/client` where it is not re-exported (TS2459). Fixed the imports; build passes, 43/43 tests pass.
- An `Edit` on the bookmark-toggle block missed because the header button sits before the `ml-auto` canonical badge, not after the short ref; re-read the exact lines and placed the `RelatedBlock` mount after the header `</div>` instead.

## Threat Flags

| Flag | File | Description |
|------|------|-------------|
| threat_flag: reviewed | web/src/views/ReadingView.tsx | New sinks: bookmark labels and related counts/seeds render as React text children; hrefs built only through `deepLinkFor`; no `dangerouslySetInnerHTML` in any new file (verified by scan) |

T-13-13 mitigated: prefs and bookmarks validated against closed allow-lists with per-field fallback; malformed entries dropped. T-13-14 mitigated: `RELATED_NOTE` renders beside every count, exact seed in mono, control named `related`. T-13-15 mitigated: bookmark href only via `deepLinkFor`; non-matching references render as plain text with a working remove button. T-13-16 accepted per plan: browser-local note in both the empty state and the margin note. T-13-SC clean: no dependencies added (`package.json` untouched).

## Known Stubs

None. The stub scan (`dangerouslySetInnerHTML|TODO|FIXME|placeholder|coming soon|not available`) is clean across all ten touched files; empty bookmark/translation states render named `EmptyState`s.

## Self-Check: PASSED

- All ten files exist on disk; all three task commits are ancestors of HEAD (`5082035`, `63a28dc`, `fe33f57` verified via `git log`).
- `npm run build` passes; full plan verification `npx vitest run src/lib/reader-storage.test.ts src/lib/related.test.ts src/components/AyahColumn.test.tsx src/display/DisplayModeControl.test.tsx src/views/ReadingView.test.tsx`: 5 files / 43 tests pass.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `reader-storage` prefs/bookmarks and `related` seed/note are reusable by plan 13-14 for any further reader surface; the `layout `-prefix and `disabledReason` patterns cover any future mode row that needs an owned-frame state.
- Manual check still open: change size/layout, reload, follow two bookmark links, press `related` against a live engine (covered as D4 human-judgment item).

---
*Phase: 13-web-ui-redesign*
*Completed: 2026-11-01*
