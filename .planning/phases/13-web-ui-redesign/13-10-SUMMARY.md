---
phase: 13-web-ui-redesign
plan: 10
subsystem: ui
tags: [react, vitest, rag-debug, tool-ledger, api-explorer, typescript]

# Dependency graph
requires:
  - phase: 13-web-ui-redesign
    provides: Concordance design contract (13-UI-SPEC), typed API client, sealed RAG view
provides:
  - Shared ToolResult contract and ToolLedger renderer every later gap-closure plan reuses
  - Working read-only rag.debug / rag.query browser path with route-decision display
  - Explorer preset list constrained to reachable routes with a failing-if-regressed test
affects: [13-11, 13-12, 13-13, 13-14]

# Actuals (#2632) — pairs with the plan's `estimate` to calibrate future estimates.
# Same estimateTokens scale (chars/4 over the realized diff), never a harness token count.
actuals:
  tokens: 9756
  tasks: 3
  commits: 3

# Tech tracking
tech-stack:
  added: []
  patterns: [shared ToolResult renderer, allow-list preset reachability test, typed ApiRequestError carrying server code]

key-files:
  created:
    - web/src/lib/tool-result.ts
    - web/src/lib/tool-result.test.ts
    - web/src/components/ToolLedger.tsx
    - web/src/components/ToolLedger.test.tsx
    - web/src/views/ExplorerView.test.tsx
  modified:
    - web/src/api/client.ts
    - web/src/views/RagDebugView.tsx
    - web/src/views/RagDebugView.test.tsx
    - web/src/views/ExplorerView.tsx

key-decisions:
  - "Denial note renders always, not only after a run — it is static policy information"
  - "ApiRequestError keeps message=summary so existing callers render identically, and adds optional code"

patterns-established:
  - "ToolLedger is the only renderer of a ToolResult in this phase; later views reuse it, never duplicate it"
  - "Route allow-list membership (not shape regex) keeps invented paths out of the Explorer"

requirements-completed: [REQ-ux-web, REQ-multi-rag, REQ-retrieval-governance, REQ-quran-result-contract]

# Coverage metadata (#1602) — one entry per shipped deliverable. Drives DETERMINISTIC UAT routing in verify-work.
coverage:
  - id: D1
    description: "rag.debug runs from the browser and its full ToolResult renders through one shared ledger"
    requirement: "REQ-quran-result-contract"
    verification:
      - kind: unit
        ref: "src/lib/tool-result.test.ts + src/components/ToolLedger.test.tsx (17 tests)"
        status: pass
      - kind: unit
        ref: "src/views/RagDebugView.test.tsx (5 tests)"
        status: pass
    human_judgment: false
  - id: D2
    description: "RAG screen explains every route-decision field, prints absent fields as absent, and states the policy reason the mutation tools have no control"
    requirement: "REQ-multi-rag"
    verification:
      - kind: unit
        ref: "src/views/RagDebugView.test.tsx prints-absent + disabled-rows tests"
        status: pass
    human_judgment: false
  - id: D3
    description: "No Explorer preset targets a path outside the documented route allow-list or a tool outside the callable set"
    requirement: "REQ-retrieval-governance"
    verification:
      - kind: unit
        ref: "src/views/ExplorerView.test.tsx (4 tests)"
        status: pass
    human_judgment: false
  - id: D4
    description: "Submitting /rag-debug against a live engine shows a checksum or a typed error code rather than a fabricated panel"
    verification: []
    human_judgment: true
    rationale: "Needs a running Rust engine with a configured RAG backend; CI/jsdom only stubs fetch"

# Metrics
duration: 10min
completed: 2026-11-01
status: complete
---

# Phase 13 Plan 10: RAG Debug + Explorer Presets Summary

**Shared ToolLedger renders read-only rag.debug/rag.query results with route-decision rows and absent-as-absent, and the Explorer only lists reachable routes with a test that fails on invented paths**

## Performance

- **Duration:** ~10 min
- **Started:** 2026-11-01T06:00Z
- **Completed:** 2026-11-01T06:10Z
- **Tasks:** 3
- **Files modified:** 9

## Accomplishments

- `ToolResult` contract (`web/src/lib/tool-result.ts`) with checksum/rows/deep-link/edition helpers, all unit-tested including null and non-array edges
- `ToolLedger` single renderer: header identity with server checksum, warnings, deep-linked references, ruled row table (or pretty `<pre>`, or named empty state), reproducibility footer
- `RagDebugView` runs `rag.debug`/`rag.query` end-to-end, renders the route-decision block field-by-field (`absent` for missing keys), and lists the seven mutating RAG tools as `aria-disabled` rows carrying `QAI-RAG-0326`
- `ExplorerView` presets replaced with reachable tool-dispatcher paths; reachability test pins the seven allow-list templates plus the 27-name callable set and bans the four removed literals

## Task Commits

Each task was committed atomically:

1. **Task 1: End-to-end `rag.debug` — one request, one ledger** - `02617c8` (feat)
2. **Task 2: Route decision panel, query/debug switch, and the denied-path note** - `960fcf0` (feat)
3. **Task 3: Explorer presets corrected to reachable routes, with a reachability test** - `a2304e7` (feat)

## Files Created/Modified

- `web/src/lib/tool-result.ts` - ToolResult contract + checksumLabel/rowsOf/deepLinkFor/editionOf helpers
- `web/src/lib/tool-result.test.ts` - Unit tests for all four helpers and edge cases
- `web/src/components/ToolLedger.tsx` - The single ToolResultData renderer
- `web/src/components/ToolLedger.test.tsx` - Header/warnings/links/table/pre/empty-state tests
- `web/src/api/client.ts` - `runTool` widened to `Envelope<ToolResultData>`; `ragDebug`/`ragQuery` added; `ApiRequestError` carries the server code
- `web/src/views/RagDebugView.tsx` - Read-only debug/query form + ledger + route decision + denial note
- `web/src/views/RagDebugView.test.tsx` - Success/error/disabled/absent/mode tests on stubbed fetch
- `web/src/views/ExplorerView.tsx` - `PRESETS` exported; four invented paths replaced with tool-dispatcher entries
- `web/src/views/ExplorerView.test.tsx` - Allow-list + callable-set + negative-list + no-mutating-tool assertions

## Decisions Made

- Denial note renders unconditionally (not only after a successful run): it is static policy information, and the task-2 test requires the rows on first paint.
- `ApiRequestError` keeps `message` equal to the old summary string so every existing caller renders identically; the server code rides on an optional `code` field the RAG view reads. No other client signature changed.
- `decisionText` renders numbers as-is, arrays comma-joined, objects JSON-stringified, and missing keys as `absent` — a missing field never appears as `0` or `[]`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Test asserted the omitted route-decision label disappears**
- **Found during:** Task 2 (RagDebugView tests)
- **Issue:** The test asserted `queryByText('reranked')` is null after omitting the field, but the plan requires labelled rows where the *value* prints `absent` — the label correctly stays.
- **Fix:** Assert the `reranked` row's text matches `absent` and not `: 0`.
- **Files modified:** web/src/views/RagDebugView.test.tsx
- **Verification:** `npx vitest run src/views/RagDebugView.test.tsx` — 5 passed
- **Committed in:** 960fcf0 (part of task commit)

**2. [Rule 1 - Bug] TS tuple indexing error in the mode-switch test**
- **Found during:** Task 3 verification (`npm run build`)
- **Issue:** `fetchMock.mock.calls[0][0]` fails under `noUncheckedIndexedAccess`-style tuple typing (`Tuple type '[]' has no element at index '0'`); vitest passed but `tsc -b` failed.
- **Fix:** Capture the requested URL in a local inside the fetch stub instead of reading mock call tuples.
- **Files modified:** web/src/views/RagDebugView.test.tsx
- **Verification:** `npm run build` passes; 26/26 tests pass across the four touched files
- **Committed in:** a2304e7 (part of task commit)

---

**Total deviations:** 2 auto-fixed (both correctness bugs in new test code)
**Impact on plan:** No scope change; both fixes keep the plan's verification green.

## Issues Encountered

- Vitest output piping: `npx vitest run ... | grep` swallowed the result table in this shell; rerunning with output redirected to a file showed the failures. No code impact.

## Threat Flags

| Flag | File | Description |
|------|------|-------------|
| threat_flag: reviewed | web/src/components/ToolLedger.tsx | New DOM sink rendering server tool payload; mitigated per T-13-04 — all cells render as React text children, no `dangerouslySetInnerHTML` anywhere in new files (verified by scan) |

T-13-02 mitigated: checksum printed exactly as received and labelled `research_checksum (server)`; never recomputed client-side. T-13-SC clean: no dependencies added (`package.json` untouched). T-13-01/T-13-03 accepted per plan (loopback dev surface, read-only diagnostics).

## Known Stubs

None. The only `placeholder` match in new code is an input hint attribute (`placeholder="mercy"`), not a stub. No empty arrays/objects flow to rendered output; the empty-rows case renders a named `EmptyState`.

## Self-Check: PASSED

- All nine files exist on disk; all three task commits are ancestors of HEAD (`02617c8`, `960fcf0`, `a2304e7` verified via `git merge-base --is-ancestor`).
- `npm run build` passes; full `npx vitest run`: 9 files / 44 tests pass.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `ToolLedger` + `tool-result` helpers are ready for plans 13-11–13-14 to reuse for every other tool surface.
- `PRESETS` export and the `ExplorerView.test.tsx` allow-list give later plans a pattern for constraining new presets.
- Manual check still open: `/rag-debug` against a live engine (checksum vs `QAI-RAG-0310`); covered as D4 human-judgment item.

---
*Phase: 13-web-ui-redesign*
*Completed: 2026-11-01*
