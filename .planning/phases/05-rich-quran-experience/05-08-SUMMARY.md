---
phase: 05-rich-quran-experience
plan: 08
subsystem: ui
tags: [citation, URN, deep-link, re-verify, hard-fail, ADR-0111, D-17, SC2]

# Dependency graph
requires:
  - phase: 05-rich-quran-experience plan 04
    provides: web/ scaffold
  - phase: 05-rich-quran-experience plan 06
    provides: typed Envelope/Meta client with resolve/citation methods, reading route
  - phase: 05-rich-quran-experience plan 09
    provides: router/nav registry this plan extends
provides:
  - Stable citation URN on every resolved citation + server hard-fail open gate
  - Citation copy/open SPA flow mounted on the frozen deep link
affects: [phase-10-agent-runtime]

# Actuals (#2632) — pairs with the plan's `estimate` to calibrate future estimates.
actuals:
  tokens: 2400
  tasks: 2
  commits: 1

# Tech tracking
tech-stack:
  added: []
  patterns: [frozen-helper URN populated at DTO construction, citation id carried as deep-link query param, fake-gated routing tests for enforcement]

key-files:
  created: [web/src/components/CitationCopy.tsx, web/src/components/CitationCopy.test.tsx, web/src/views/CitationView.tsx, crates/server/tests/citation_open.rs]
  modified: [crates/citations/src/lib.rs, crates/server/tests/api.rs, web/src/router.tsx, web/src/App.tsx]

key-decisions:
  - "URN populated inside resolve_stored/resolve beside deep_link (same frozen helper, same call sites) rather than parsed back from deep_link in api.rs — no format round-trip"
  - "Citation id travels as ?citation= on the frozen deep link; Router mounts CitationView alongside ReadingView when present"
  - "New citation_open.rs target carries minimal generated stubs (FakeApi implements only api_citation); existing api.rs fakes updated for the new field"

patterns-established:
  - "Citation responses always carry both urn and deep_link (None together on failure verdicts)"

requirements-completed: [REQ-quran-display]

# Coverage metadata (#1602)
coverage:
  - id: D1
    description: "Citation open re-verifies server-side; mismatch hard-fails typed non-200; URN surfaced from frozen helper"
    requirement: "REQ-quran-display"
    verification:
      - kind: integration
        ref: "cargo test -p server --test citation_open (3/3)"
        status: pass
    human_judgment: false
  - id: D2
    description: "Citation copy echoes API URN/deep link verbatim; deep link with ?citation= mounts CitationView"
    requirement: "REQ-quran-display"
    verification:
      - kind: unit
        ref: "npx vitest run src/components/CitationCopy.test.tsx (3/3)"
        status: pass
    human_judgment: false
  - id: D3
    description: "Locked D-17/ADR-0111 contract confirmed before flow wiring"
    requirement: "REQ-quran-display"
    verification:
      - kind: other
        ref: "owner checkpoint answer: proceed-as-locked"
        status: pass
    human_judgment: true
    rationale: "One-way citation-identity embodiment requires explicit owner confirmation."

# Metrics
duration: ~1.5h wall
completed: 2026-10-09
status: complete
---

# Phase 05 Plan 08 Summary

**SC2 citation flow ships: copy emits the frozen URN + deep link, opening re-verifies server-side with mismatch as a typed hard failure.**

## Performance

- **Duration:** ~1.5h wall
- **Started:** 2026-10-09
- **Completed:** 2026-10-09
- **Tasks:** 2 completed
- **Files modified:** 8

## Accomplishments

- `ResolvedCitation.urn` populated from `citations::citation_urn` at both success construction sites
  (`resolve`, `resolve_stored`); `None` on all failure verdicts — the citation response now exposes the URN
  with zero SPA formatting
- `citation_open.rs` gate: verified open returns URN + deep link (200), tampered open hard-fails typed
  non-200, missing id 404s; existing `api.rs` fakes updated for the new field
- `CitationCopy` echoes API values verbatim (test asserts no reconstructed variant); `CitationView` opens
  `?citation=` deep links against `/api/v1/quran/citations/{id}` rendering verified/mismatch; router mounts
  it alongside the reading view; full web suite 18/18 + build green

## Task Commits

1. **Task 1: Confirm citation-identity contract (blocking-decision)** - owner answer recorded here (no code)
2. **Task 2: Copy + open flow** - `HEAD` feat commit (this run)

**Plan metadata:** `HEAD` (docs: this file)

## Files Created/Modified

- `crates/citations/src/lib.rs` - `urn` field + population at both success sites
- `crates/server/tests/citation_open.rs` - 3 routing/enforcement tests with minimal stubs
- `crates/server/tests/api.rs` - existing fakes gain `urn`
- `web/src/components/CitationCopy.tsx` (+ test) - verbatim echo + route-mount test
- `web/src/views/CitationView.tsx` - verify/mismatch against the enforcing endpoint
- `web/src/router.tsx` - `parseCitationId` + conditional `CitationView` mount + nav (unchanged entries)
- `web/src/App.tsx` - query string passed to the router

## Decisions Made

- No new dependencies on either side (T-05-SC).
- `resolve` stays parse-only; the open path relies solely on the enforcing `citations/{id}` endpoint.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing] URN lives in the citations DTO, not api.rs**
- **Found during:** Task 2 (ResolvedCitation lacks any URN; handler only sees the DTO)
- **Issue:** Plan suggested adding URN to the api.rs response DTO; parsing it back from deep_link there would be a format round-trip.
- **Fix:** Field added to `ResolvedCitation` and populated beside `deep_link` from the same frozen helper at both success sites — the api.rs response gains it by serialization with no handler change.

## Verification Results

- `cargo test -p server --test citation_open` — 3/3 pass; full `citations` + `server` suites green
- `npx vitest run` — 18/18 pass; `npm run build` — success
