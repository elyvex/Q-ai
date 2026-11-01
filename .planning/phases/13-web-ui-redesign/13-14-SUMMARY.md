---
phase: 13-web-ui-redesign
plan: 14
subsystem: ui
tags: [react, vitest, citations, isnad, byte-diff, tool-ledger, deep-links]

# Dependency graph
requires:
  - phase: 13-web-ui-redesign
    provides: Shared ToolLedger and tool-result helpers (13-10-SUMMARY)
  - phase: 13-web-ui-redesign
    provides: ROUTES/DIRECTORY/NAVIGATION/isActive-also route-wiring pattern (13-11-SUMMARY)
  - phase: 13-web-ui-redesign
    provides: ApiRequestError.code/remedy and catalogue-driven denied rows (13-12-SUMMARY)
provides:
  - `/citations` workbench with server verdict, corpus hash, URN, re-verify and a labelled local tamper simulation over a code-point-exact diff
  - `byte-diff` code-point comparison helper with unit tests
  - `/isnad` read-only reading room (trace, narrator, rijal) with the five identity mutations visible and inert
affects: []

# Actuals (#2632) — pairs with the plan's `estimate` to calibrate future estimates.
# Same estimateTokens scale (chars/4 over the realized diff), never a harness token count.
actuals:
  tokens: 10885
  tasks: 3
  commits: 3

# Tech tracking
tech-stack:
  added: []
  patterns: [code-point diff via Array.from with empty-string missing side, dual-variable baseline (canonical vs localCopy state), catalogue-filtered denied rows for a single tool family]

key-files:
  created:
    - web/src/lib/byte-diff.ts
    - web/src/lib/byte-diff.test.ts
    - web/src/views/CitationsWorkbenchView.tsx
    - web/src/views/CitationsWorkbenchView.test.tsx
    - web/src/views/IsnadView.tsx
    - web/src/views/IsnadView.test.tsx
  modified:
    - web/src/router.tsx
    - web/src/router.test.tsx
    - web/src/components/layout/site-header.tsx
    - web/src/components/command/command-menu.tsx

key-decisions:
  - "Isnad denied rows reuse the DENIED catalogue filtered to isnad.identity_* for names and codes, with the plan's qai-isnad sentence written in the view"
  - "The verdict counter increments on every completed citation request, success or typed failure — a re-verify is a verification performed"
  - "The equality baseline compares two distinct variables (fetched canonical vs localCopy state) so the assertion can fail"

patterns-established:
  - "A query-carrying route branches on `path.split('?', 1)[0]` before the reference-null branch, and its test pins the also-before-/ ordering"
  - "Local tampers render in dir=rtl/lang=ar while the diff readout stays dir=ltr mono — one screen, two directions, each labelled"

requirements-completed: [REQ-ux-web, REQ-quran-result-contract, REQ-isnad-narrator-graph]

# Coverage metadata (#1602) — one entry per shipped deliverable. Drives DETERMINISTIC UAT routing in verify-work.
coverage:
  - id: D1
    description: "/citations opens an id to the server verdict, text_hash, URN and deep link, re-verifies on demand, and compares the deep-linked text byte-exactly"
    requirement: "REQ-quran-result-contract"
    verification:
      - kind: unit
        ref: "src/lib/byte-diff.test.ts + src/views/CitationsWorkbenchView.test.tsx verdict panel block + src/router.test.tsx citations wiring (16 tests)"
        status: pass
    human_judgment: false
  - id: D2
    description: "Tamper simulator shows the code-point index and both labels on edit, restores cleanly, issues no request, and states what it cannot do"
    requirement: "REQ-ux-web"
    verification:
      - kind: unit
        ref: "src/views/CitationsWorkbenchView.test.tsx simulator block (3 tests)"
        status: pass
    human_judgment: false
  - id: D3
    description: "isnad.trace/narrator/rijal run through the shared ledger and every identity mutation renders inert with QAI-ISNAD-0326"
    requirement: "REQ-isnad-narrator-graph"
    verification:
      - kind: unit
        ref: "src/views/IsnadView.test.tsx (5 tests) + src/router.test.tsx isnad wiring"
        status: pass
    human_judgment: false
  - id: D4
    description: "Open /citations?citation=<id> against a live engine, confirm the byte-identical baseline, change one character and confirm the verdict is untouched"
    requirement: "REQ-quran-result-contract"
    verification: []
    human_judgment: true
    rationale: "Needs a running Rust engine with a stored citation; jsdom only stubs fetch"
  - id: D5
    description: "Open /isnad against a live engine, run a trace, and confirm the denied rows carry no control"
    requirement: "REQ-isnad-narrator-graph"
    verification: []
    human_judgment: true
    rationale: "Needs a running engine with an isnad backend; jsdom only stubs fetch"

# Metrics
duration: 7min
completed: 2026-11-01
status: complete
---

# Phase 13 Plan 14: Citations Workbench + Isnad Reading Room Summary

**`/citations` shows the server's verification evidence with a labelled local tamper simulation over a code-point-exact diff, and `/isnad` reads the narrator graph while the five identity mutations sit visible and inert**

## Performance

- **Duration:** ~7 min
- **Started:** 2026-11-01T03:17:17Z
- **Completed:** 2026-11-01T03:24:26Z
- **Tasks:** 3
- **Files modified:** 10

## Accomplishments

- `byte-diff.ts` compares by Unicode code point (`Array.from`), reports lengths as code-point counts, names the index and both `U+XXXX` labels on mismatch (`end of text` for the missing side), and renders one readout line per state
- `CitationsWorkbenchView` (new; singular `CitationView.tsx` untouched) renders the server's `citation_id`, verdict (string and single-key-object variants), `text_hash`, `urn` and deep link verbatim, with `Re-verify` and a per-session verification counter; typed failures render code, summary and remedy with no softened success
- The deep-linked text is fetched via `fetchAyahs` on the trailing `surah:ayah`, labelled with the actually-returned edition slug and version, with a margin note when the citation points at a different edition than the active one
- Tamper simulation edits a browser-held copy (`dir="rtl" lang="ar"`), recomputes `byteDiff` per keystroke into a `dir="ltr"` mono readout with 8-code-point context on mismatch, restores cleanly, issues no request, and states it cannot write the corpus, change the verdict, or recompute `text_hash`
- `IsnadView` runs `isnad.trace` (from/to + optional k, omitted when blank), `isnad.narrator` and `isnad.rijal` through the shared `ToolLedger`, with `Run` blocked by a named `required:` reason, `LoadingLine` in flight and coded `ErrorNote` on failure
- The five identity mutations render as `aria-disabled` rows with `QAI-ISNAD-0326` and the `qai isnad …` CLI sentence; both routes wired into all four surfaces (switch branch, ROUTES, DIRECTORY, NAV plus command-menu NAVIGATION)

## Task Commits

Each task was committed atomically:

1. **Task 1: End-to-end `/citations` — verdict, hash, URN, re-verify, one diff** - `10cc033` (feat)
2. **Task 2: Tamper simulator with a byte-exact diff, labelled as local** - `7796fbd` (feat)
3. **Task 3: Read-only isnad view — trace, narrator, rijal, and the denied mutations** - `5f74d5e` (feat)

## Files Created/Modified

- `web/src/lib/byte-diff.ts` - `ByteDiff`, `byteDiff`, `codePointLabel`, `describeDiff`
- `web/src/lib/byte-diff.test.ts` - Equality, UTF-16-boundary, length-mismatch, label and readout tests (11 tests)
- `web/src/views/CitationsWorkbenchView.tsx` - Verdict panel, re-verify counter, text pane, tamper simulator
- `web/src/views/CitationsWorkbenchView.test.tsx` - Verdict/hash/URN/link, re-verify count, typed error, edit-flip, restore, no-request, boundary sentences (6 tests)
- `web/src/views/IsnadView.tsx` - Three-way seg, per-instrument forms, ledger rendering, denied rows
- `web/src/views/IsnadView.test.tsx` - Trace ledger, rijal params, disabled-Run reason, denied rows, error code (5 tests)
- `web/src/router.tsx` - `/citations` (query-split branch) and `/isnad` branches, ROUTES and DIRECTORY entries
- `web/src/router.test.tsx` - `/citations` and `/isnad` membership plus rail-highlight assertions
- `web/src/components/layout/site-header.tsx` - `/citations` on the Reader `also`, `/isnad` on the Hadith `also`
- `web/src/components/command/command-menu.tsx` - Citations (ShieldCheck) and Isnad (GitBranch) NAVIGATION entries

## Decisions Made

- Isnad denied rows reuse the `DENIED` catalogue filtered to `isnad.identity_*` so names and policy codes stay in sync with `/tools`, while the view writes the plan's isnad-specific `qai isnad …` sentence the shared reason string does not carry.
- The verification counter increments on every completed citation request rather than successes only — a re-verify that hard-fails is still a verification performed, and the failure path shows no counter at all.
- The equality baseline deliberately compares two distinct variables (fetched `canonical` vs `localCopy` state) so the byte-identical line is a real comparison the test can flip, not a literal.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Test read mock calls off `vi.stubGlobal`'s return value**
- **Found during:** Task 2 (CitationsWorkbenchView tests)
- **Issue:** `vi.stubGlobal` does not return the mock the way the test assumed (`mock.calls` was undefined) — the same tuple-indexing trap plan 10 hit with `tsc`.
- **Fix:** Capture requested URLs in a local `calls` array inside the fetch stub, matching the established plan-10 pattern.
- **Files modified:** web/src/views/CitationsWorkbenchView.test.tsx
- **Verification:** `npx vitest run src/views/CitationsWorkbenchView.test.tsx` — 6 passed
- **Committed in:** 7796fbd (part of task commit)

**2. [Rule 1 - Bug] Error-path assertion matched the page lede**
- **Found during:** Task 2 (CitationsWorkbenchView tests)
- **Issue:** `queryByText(/verdict /)` matched the lede ("The verdict below comes from the server…"), so the no-verdict assertion could never hold.
- **Fix:** Assert `queryByText(/verdict ExactMatch/)` is null and the `Re-verify` control is absent.
- **Files modified:** web/src/views/CitationsWorkbenchView.test.tsx
- **Verification:** `npx vitest run src/views/CitationsWorkbenchView.test.tsx` — 6 passed
- **Committed in:** 7796fbd (part of task commit)

**3. [Rule 1 - Bug] Test expected the wrong hex for `d`**
- **Found during:** Task 2 (CitationsWorkbenchView tests)
- **Issue:** Editing `abc` → `adc` was asserted as `found U+0063` (`c`), but the changed character is `d` = `U+0064`; the implementation output was correct.
- **Fix:** Corrected the expectation to `expected U+0062, found U+0064`.
- **Files modified:** web/src/views/CitationsWorkbenchView.test.tsx
- **Verification:** `npx vitest run src/views/CitationsWorkbenchView.test.tsx` — 6 passed
- **Committed in:** 7796fbd (part of task commit)

**4. [Rule 1 - Bug] Denied-sentence assertion hit five identical nodes**
- **Found during:** Task 3 (IsnadView tests)
- **Issue:** `getByText(/trusted operator surface/)` throws on five matching rows — the same multiple-match trap plans 11 and 12 hit.
- **Fix:** Assert `getAllByText(...)` length equals the five identity names.
- **Files modified:** web/src/views/IsnadView.test.tsx
- **Verification:** `npx vitest run src/views/IsnadView.test.tsx src/router.test.tsx` — 14 passed
- **Committed in:** 5f74d5e (part of task commit)

---

**Total deviations:** 4 auto-fixed (all test-correctness bugs; no production-code deviation)
**Impact on plan:** No scope change; all fixes keep the plan's verification green.

## Issues Encountered

- Vitest output piping (`| tail`, `| grep`) swallows the result table in this shell (same as plans 10–13); redirecting to a file and reading it shows the failures.

## Threat Flags

| Flag | File | Description |
|------|------|-------------|
| threat_flag: reviewed | web/src/views/CitationsWorkbenchView.tsx, web/src/views/IsnadView.tsx | New DOM sinks rendering server verdict/hash/URN/tool payloads and catalogue strings; mitigated per T-13-17/T-13-19 — verdict panel renders only server data and stays separate from the simulator, all cells render as React text children, no `dangerouslySetInnerHTML` in any new file (verified by scan) |

T-13-17 mitigated: verdict panel is server-data-only and visually separate; the simulator states it cannot change the verdict and the test asserts the verdict text is unchanged after an edit. T-13-18 mitigated: no hashing code in the browser — the only hash shown is the endpoint's `text_hash` string, labelled as the server's corpus fingerprint. T-13-19 mitigated: the denied list has no form and no control; names and `QAI-ISNAD-0326` come from the shared catalogue. T-13-20 accepted per plan (loopback-only opaque ids). T-13-SC clean: no dependencies added (`package.json` untouched).

## Known Stubs

None. The stub scan (`dangerouslySetInnerHTML|TODO|FIXME|placeholder|coming soon|not available`) is clean across both new views and `byte-diff.ts`; the only `placeholder`-free inputs carry hint-free labels, and the no-deep-link case renders a named `EmptyState`.

## Self-Check: PASSED

- All ten files exist on disk; all three task commits are ancestors of HEAD (`10cc033`, `7796fbd`, `5f74d5e` verified via `git log`).
- `npm run build` passes; full plan verification `npx vitest run src/lib/byte-diff.test.ts src/views/CitationsWorkbenchView.test.tsx src/views/IsnadView.test.tsx src/router.test.tsx`: 4 files / 29 tests pass.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `byte-diff` is reusable by any later surface that compares local text against canonical text; the dual-variable baseline + labelled-simulator shape is the pattern for any future "local edit vs server truth" screen.
- The `DENIED`-filter idiom covers any later view that needs one tool family's refused names with catalogue-exact codes.
- Manual checks still open: `/citations?citation=<id>` baseline and one-character flip against a live engine (D4), `/isnad` trace against a live isnad backend (D5).

---
*Phase: 13-web-ui-redesign*
*Completed: 2026-11-01*
