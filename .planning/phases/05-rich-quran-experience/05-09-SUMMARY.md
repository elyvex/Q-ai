---
phase: 05-rich-quran-experience
plan: 09
subsystem: ui
tags: [spa, research-view, graph-view, rag-debug, typed-client, ADR-0217, D-11]

# Dependency graph
requires:
  - phase: 05-rich-quran-experience plan 04
    provides: web/ scaffold, pinned toolchain
  - phase: 05-rich-quran-experience plan 06
    provides: generic Envelope/Meta request helper, layer tokens, router shell
provides:
  - Research/graph/RAG-debug SPA views with routes + nav
  - Full typed research/graph client method set
affects: [05-08, 05-07]

# Actuals (#2632) — pairs with the plan's `estimate` to calibrate future estimates.
actuals:
  tokens: 2600
  tasks: 2
  commits: 1

# Tech tracking
tech-stack:
  added: []
  patterns: [presentational views over injected props with live client wiring alongside, ResearchResults split for direct unit testing]

key-files:
  created: [web/src/views/ResearchView.tsx, web/src/views/ResearchView.test.tsx, web/src/views/GraphView.tsx, web/src/views/GraphView.test.tsx, web/src/views/RagDebugView.tsx, web/src/views/RagDebugView.test.tsx]
  modified: [web/src/api/client.ts, web/src/router.tsx]

key-decisions:
  - "Research root/lemma go through the generic tool() method (no bespoke /root /lemma routes exist); family/counts use their dedicated routes"
  - "GraphView drops the unneeded path-query control (neighbors live + props-driven lines); graphPath stays available in the client for later use"
  - "Nav entries added for all three views; research route renders with empty query (explicit empty state, not blank)"

patterns-established:
  - "View = props-driven presentational core + client-wired container in one file (test the core directly)"

requirements-completed: [REQ-quran-research-tools, REQ-quran-display]

# Coverage metadata (#1602)
coverage:
  - id: D1
    description: "Research view renders service-ordered hits with explicit empty state over typed client methods"
    requirement: "REQ-quran-research-tools"
    verification:
      - kind: unit
        ref: "npx vitest run src/views/ResearchView.test.tsx (3/3)"
        status: pass
    human_judgment: false
  - id: D2
    description: "Graph view surfaces truncated/incomplete_reason verbatim; RAG view renders typed unavailable with no retrieval path"
    requirement: "REQ-quran-research-tools"
    verification:
      - kind: unit
        ref: "npx vitest run src/views/GraphView.test.tsx src/views/RagDebugView.test.tsx (3/3)"
        status: pass
    human_judgment: false

# Metrics
duration: ~1h wall
completed: 2026-10-09
status: complete
---

# Phase 05 Plan 09 Summary

**SPA research surface ships: service-ordered research hits with explicit empty states, verbatim-truncation graph view, and an honest no-retrieval RAG debug view — all over the extended typed client.**

## Performance

- **Duration:** ~1h wall
- **Started:** 2026-10-09
- **Completed:** 2026-10-09
- **Tasks:** 2 completed
- **Files modified:** 8

## Accomplishments

- `ResearchView` (+ kind switcher + live run): search/root/lemma/family/frequency/distribution/co-occurrence
  through declared client methods; order-preserving render; explicit empty state; text-only rendering
- `GraphView` (+ live neighbors): verbatim `truncated`/`incomplete_reason` block, never "no path"
- `RagDebugView`: typed unavailable, zero `.rag-result` nodes, no retrieval code path
- `client.ts`: search/family/rootFrequency/lemmaFrequency/frequency/distribution/cooccurrence +
  graphNeighbors/graphPath over the generic helper; router + nav for `/research`, `/graph`, `/rag-debug`
- 15/15 Vitest green; `npm run build` green; no `dangerouslySetInnerHTML`

## Task Commits

1. **Task 1: Research view** + **Task 2: Graph + RAG views** - `56228b1` (feat, shared client/router)

**Plan metadata:** `HEAD` (docs: this file)

## Files Created/Modified

- `web/src/views/ResearchView.tsx` (+ test) - kind switcher, live run, `ResearchResults` (order/empty/text)
- `web/src/views/GraphView.tsx` (+ test) - live neighbors, verbatim truncation
- `web/src/views/RagDebugView.tsx` (+ test) - typed unavailable
- `web/src/api/client.ts` - 9 research/graph methods over `get`/`post` helpers
- `web/src/router.tsx` - routes + nav for the three views

## Decisions Made

- Single commit for both tasks (shared client.ts/router.tsx cannot compile per-task alone).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Dead `openPath` helper + stray `void` markers**
- **Found during:** Task 2 (self-review)
- **Issue:** Unwired path-query helper and leftover `void` statements.
- **Fix:** Removed; `graphPath` remains available in the client for later use.

## Verification Results

- `npx vitest run` — 15/15 pass (9 existing + 6 new)
- `npm run build` (`tsc -b` + vite) — success
- No `dangerouslySetInnerHTML` (scanned)
