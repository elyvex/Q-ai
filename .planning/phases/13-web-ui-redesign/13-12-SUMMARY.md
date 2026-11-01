---
phase: 13-web-ui-redesign
plan: 12
subsystem: ui
tags: [react, vitest, tools, tool-ledger, parity, cli-equivalence, policy]

# Dependency graph
requires:
  - phase: 13-web-ui-redesign
    provides: Shared ToolLedger and tool-result helpers (13-10-SUMMARY)
  - phase: 13-web-ui-redesign
    provides: ROUTES/DIRECTORY/NAVIGATION/isActive-also route-wiring pattern (13-11-SUMMARY)
provides:
  - `/tools` catalogue of all 27 callable tools with typed generated forms
  - `tools.ts` derivations (`paramsJson`, `cliCommand`, `httpRequest`) plus the parity panel pattern
  - `DENIED` list (7 RAG + 5 isnad) reused by any later view that names refused tools
  - `ApiRequestError.remedy` carrying the server remedy beside code and summary
affects: [13-13, 13-14]

# Actuals (#2632) — pairs with the plan's `estimate` to calibrate future estimates.
# Same estimateTokens scale (chars/4 over the realized diff), never a harness token count.
actuals:
  tokens: 13660
  tasks: 3
  commits: 3

# Tech tracking
tech-stack:
  added: []
  patterns: [catalogue-generated typed forms, sent-params parity panels, optional-remedy ApiRequestError]

key-files:
  created:
    - web/src/lib/tools.ts
    - web/src/lib/tools.test.ts
    - web/src/views/ToolsView.tsx
    - web/src/views/ToolsView.test.tsx
  modified:
    - web/src/router.tsx
    - web/src/router.test.tsx
    - web/src/components/layout/site-header.tsx
    - web/src/components/command/command-menu.tsx
    - web/src/api/client.ts

key-decisions:
  - "cli/http panels derive from the params kept beside the last successful ledger, never recomputed from the live form"
  - "ApiRequestError gains an optional remedy rather than the view refetching the error body"
  - "Unchecked bool means the param is omitted (server default); explicit false is never sent"

patterns-established:
  - "A catalogue view groups tools by DOMAIN_ORDER, generates its form from fields, and blocks Run with `required: <label>`"
  - "Copy uses navigator.clipboard.writeText and surfaces a failure message — never a silent no-op"

requirements-completed: [REQ-ux-web, REQ-cli-api, REQ-quran-result-contract, REQ-agents-tools-runtime]

# Coverage metadata (#1602) — one entry per shipped deliverable. Drives DETERMINISTIC UAT routing in verify-work.
coverage:
  - id: D1
    description: "/tools appears in ROUTES, DIRECTORY and NAVIGATION, and the Research rail entry stays active while /tools is open"
    requirement: "REQ-ux-web"
    verification:
      - kind: unit
        ref: "src/router.test.tsx tools route wiring (2 tests)"
        status: pass
    human_judgment: false
  - id: D2
    description: "Catalogue names the 27-name callable set exactly with registry-exact field keys, and derivations print the same params three ways"
    requirement: "REQ-agents-tools-runtime"
    verification:
      - kind: unit
        ref: "src/lib/tools.test.ts (13 tests)"
        status: pass
    human_judgment: false
  - id: D3
    description: "Denied tools render as inert rows with QAI-RAG-0326 / QAI-ISNAD-0326; a refused tool renders code, summary and remedy with no ledger table"
    requirement: "REQ-quran-result-contract"
    verification:
      - kind: unit
        ref: "src/views/ToolsView.test.tsx denied + honest-failure blocks (3 tests)"
        status: pass
    human_judgment: false
  - id: D4
    description: "Running quran.get_ayah 2:1 in the browser prints the same research_checksum the CLI prints for the copied --json line"
    verification: []
    human_judgment: true
    rationale: "Needs a running Rust engine with a loaded edition; jsdom only stubs fetch"
  - id: D5
    description: "Cold-loading /tools, the rail highlight, the ⌘K entry, the reader directory link and the copy buttons work in the served build"
    requirement: "REQ-ux-web"
    verification: []
    human_judgment: true
    rationale: "jsdom covers route wiring only; reaching /tools cold needs the served build plus a running engine"

# Metrics
duration: 25min
completed: 2026-11-01
status: complete
---

# Phase 13 Plan 12: Research Tools Summary

**One catalogue for all 27 callable tools — a typed form per tool, the shared ledger, and copyable CLI/HTTP equivalents for the exact params that produced the checksum**

## Performance

- **Duration:** ~25 min
- **Completed:** 2026-11-01
- **Tasks:** 3
- **Files modified:** 9

## Accomplishments

- `tools.ts` catalogue types (`ToolField`, `ToolSpec`, `DENIED`) with `paramsJson` (skip empty optionals, `Number()` coercion, `JSON.parse` per field, `'true'`/`'false'` bools, typed `ToolParamError` naming the field), `cliCommand` (single-quote escape as `'\''`, global `--json`), and `httpRequest` (`POST /api/v1/quran/tool/<name>`, pretty body)
- All 27 callable tools catalogued with field keys matching the registry structs (`reference`, `translations`, `surah/ayah/position`, `seeds`/`steps` as json, …); `scripture.editions` takes no fields
- `/tools` wired into all four surfaces: `ROUTES`, exported `DIRECTORY`, route-switch branch, rail `also` on Research (`['/words', '/tools']`), and exported `NAVIGATION` for ⌘K
- `ToolsView` runs any callable tool through `runTool` into the shared `ToolLedger`, blocks `Run` with `required: <label>` or the live json parse message, and mirrors the last successful run as copyable `cli`/`http` panels derived from the sent params
- `DENIED` complete (7 RAG + 5 isnad identity tools) rendering as `aria-disabled` rows with policy codes and the durable-state reason; no form, no run control
- Honest failure path: `ApiRequestError` now carries the optional server `remedy`, and the view renders code, summary and remedy in `ErrorNote` with no ledger table

## Task Commits

Each task was committed atomically:

1. **Task 1: End-to-end `/tools` — one tool, one form, three surfaces** - `e998a47` (feat)
2. **Task 2: Typed forms for every callable tool** - `af1beb7` (feat)
3. **Task 3: Denied tools, honest errors, and the checksum parity note** - `31a8e90` (feat)

## Files Created/Modified

- `web/src/lib/tools.ts` - Catalogue, `DENIED`, `paramsJson`/`cliCommand`/`httpRequest`, `ToolParamError`
- `web/src/lib/tools.test.ts` - Catalogue exactness (27 names, no dupes), derivations, param coercion and typed failures
- `web/src/views/ToolsView.tsx` - Domain-grouped catalogue, generated form, ui/cli/http surface control, denied rows
- `web/src/views/ToolsView.test.tsx` - Field rendering, disabled-Run reason, denied rows, QAI-SCR-0310 path, CLI line
- `web/src/router.tsx` - `/tools` route, `DIRECTORY` entry, switch branch
- `web/src/router.test.tsx` - `/tools` membership and rail-highlight assertions
- `web/src/components/layout/site-header.tsx` - Research `also` gains `/tools`
- `web/src/components/command/command-menu.tsx` - `NAVIGATION` gains Research Tools (Terminal icon)
- `web/src/api/client.ts` - `ApiRequestError` carries optional `remedy` in both `get` and `post`

## Decisions Made

- The `cli` and `http` panels derive from the `{name, params}` object kept in state beside the last successful ledger — never recomputed from the live form — so the checksum beside them is provably the checksum of the params shown (T-13-12).
- `ApiRequestError` gains an optional third constructor argument rather than the view refetching or the test bypassing the client: existing callers pass one or two args and are unaffected.
- An unchecked bool field omits the param (server default applies) rather than sending `false`; the toggle only ever emits `'true'` or absence.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing critical functionality] `ApiRequestError` dropped the server remedy the plan requires the view to render**
- **Found during:** Task 3 (honest failure path)
- **Issue:** The plan requires the view to render the server's `error.code`, `error.summary` and `error.remedy`, but `runTool` threw `ApiRequestError` carrying only code and summary — the remedy died in `client.ts`.
- **Fix:** Added optional `remedy` to `ApiRequestError` and populated it from `body?.error?.remedy` in both `get` and `post`. Fully backward compatible; no caller signature changed.
- **Files modified:** web/src/api/client.ts
- **Committed in:** 31a8e90 (part of task commit)

**2. [Rule 1 - Bug] Test asserted a single `quran.get_ayah` text node but three render**
- **Found during:** Task 2 (ToolsView tests)
- **Issue:** `getByText('quran.get_ayah')` throws on multiple matches — the catalogue button, the kicker, and the Run button all render the name.
- **Fix:** Assert `getAllByText('quran.get_ayah').length` is greater than 0.
- **Files modified:** web/src/views/ToolsView.test.tsx
- **Verification:** `npx vitest run src/lib/tools.test.ts src/views/ToolsView.test.tsx` — 17 passed
- **Committed in:** af1beb7 (part of task commit)

**3. [Rule 1 - Bug] Checksum lookup used the bare hex but the ledger prints `sha256:<hex>`**
- **Found during:** Task 3 (cli surface test)
- **Issue:** `findByText('deadbeef')` never matches the single `research_checksum (server) sha256:deadbeef` text node.
- **Fix:** Match `/deadbeef/` instead.
- **Files modified:** web/src/views/ToolsView.test.tsx
- **Verification:** `npx vitest run src/lib/tools.test.ts src/views/ToolsView.test.tsx src/router.test.tsx` — 26 passed
- **Committed in:** 31a8e90 (part of task commit)

---

**Total deviations:** 3 auto-fixed (1 missing-functionality addition to the shared client, 2 test-correctness bugs)
**Impact on plan:** No scope change; the client addition is what makes the plan's remedy requirement testable against the real server.

## Issues Encountered

- An early `Edit` on `router.tsx` targeted `./ExplorerView` imports but the file uses `./views/…` paths; corrected immediately, no bad state committed.
- A test-file `Edit` briefly consumed the `words route wiring` describe header; repaired and verified the file reads both blocks before running tests.

## Threat Flags

| Flag | File | Description |
|------|------|-------------|
| threat_flag: reviewed | web/src/views/ToolsView.tsx | New DOM sinks rendering server tool payloads and catalogue strings; mitigated per T-13-09/T-13-11 — all cells render as React text children, json bodies go through `JSON.stringify` into `<pre>`, no `dangerouslySetInnerHTML` in any new file (verified by scan) |

T-13-09 mitigated: `DENIED` is a separate list with no form and no run control, and the UI note names the CLI as the trusted surface with the `policy.tool_execution_default = "allow"` opt-in. T-13-10 mitigated: single quotes escape as `'\''` before wrapping, covered by unit test; the line is displayed and copied, never executed. T-13-11 mitigated: `JSON.parse` inside `paramsJson` with a typed per-field failure plus a live blocking check under the field; the value is re-serialized by `post()`, never concatenated. T-13-12 mitigated: `cli`/`http` derive from the sent params kept beside the ledger. T-13-SC clean: no dependencies added (`package.json` untouched).

## Known Stubs

None. The only `placeholder` matches in new code are input hint attributes, not stubs. No empty arrays/objects flow to rendered output; the pre-run state renders a named `EmptyState`.

## Self-Check: PASSED

- All nine files exist on disk; all three task commits are ancestors of HEAD (`e998a47`, `af1beb7`, `31a8e90` verified via `git log`).
- `npm run build` passes; full `npx vitest run`: 14 files / 92 tests pass.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `TOOLS`/`DENIED`/`paramsJson` give plans 13-13–13-14 the finished catalogue: any later surface that names a tool reuses these instead of re-listing names.
- `ApiRequestError.remedy` is available to every view that renders a typed server failure.
- Manual checks still open: run `quran.get_ayah` `2:1` against a live engine and compare the browser checksum with the CLI checksum for the copied line (covered as D4/D5 human-judgment items).

---
*Phase: 13-web-ui-redesign*
*Completed: 2026-11-01*
