---
phase: 02-canonical-quran-core
plan: 08
subsystem: citations
tags: [quran, citations, d-15, answer-path, exemption-ledger, gap-closure, g-02-3, source-scan, guard]

requires:
  - phase: 02-canonical-quran-core
    provides: plan 02-05's structurally-exempt answer-path list and the D-15 enforcement surfaces it audits
  - phase: 02-canonical-quran-core
    provides: plan 02-07's evidence-of-record and owner-gate files that carried the narrowed ledger
provides:
  - a corrected D-15 answer-path exemption ledger on the emission frame in all three evidence-of-record files (2 enforced, 18 structurally-exempt emitting paths, 1 no-text, 1 debug-only, plus a not-emitters list)
  - drift-proof path#symbol references replacing all stale file:line anchors
  - corrected ReaderCitationSource in-code doc, 02-RESEARCH §F-6 search rows + frame-correction note, and a G-02-3 note on the historical 02-05-PLAN Task 3 narrowing
  - crates/application/tests/answer_path_ledger.rs — a checked-list source-scan guard that fails when a recorded path drifts or a new marker-emitting handler is added unrecorded
affects: [phase 02 UAT gap G-02-3 closure, phase 9 citation verifier reuse, phase 5 citation UI, any future answer path that emits canonical text]

actuals:
  tokens: 9027     # chars/4 over the realized diff (36109 diff bytes across the 7 files)
  tasks: 3
  commits: 3       # MEASURED: git rev-list --count 64d5d9ec..HEAD
  plan_head_before: 64d5d9ec7f0f1fb6b7a4fc43b834db38b274824f

tech-stack:
  added: []
  patterns:
    - "Emission-frame answer-path audit: a path is enforced when it accepts an externally supplied quotation and structurally exempt when its response carries canonical text built from canonical rows; the exemption's emission mechanism (AyahView.canonical / SearchHit.quotation / hand-built token surface) is recorded per path."
    - "Drift-proof ledger references: path#symbol refs (grep-resolvable) replace file:line anchors, which drift as later work moves code."
    - "Checked-list source guard: a `(file, symbol, kind)` ledger resolved kind-aware against the tree, plus a marker scan that maps every emitting marker to a recorded symbol, turning a point-in-time snapshot into a failing build."

key-files:
  created:
    - crates/application/tests/answer_path_ledger.rs
  modified:
    - docs/06-progress/phase-02-evidence.md
    - docs/05-followups/phase-02-owner-gates.md
    - .planning/phases/02-canonical-quran-core/02-05-SUMMARY.md
    - crates/application/src/quran_tools.rs
    - .planning/phases/02-canonical-quran-core/02-RESEARCH.md
    - .planning/phases/02-canonical-quran-core/02-05-PLAN.md

key-decisions:
  - "The D-15 ledger is rebuilt on the emission frame (every path whose response contains quoted canonical text), not the direct-read frame that omitted index-backed search paths; the enforcement paths themselves are unchanged — the omitted paths are exempt-by-construction for the same reason the recorded ones are."
  - "Each exempt path carries an emission mechanism (AyahView.canonical, SearchHit.quotation, or the hand-built token surface) because tokens_handler is exempt by construction yet does not emit via AyahView.canonical and therefore needs its own label."
  - "All ledger references are path#symbol; line anchors are secondary and omitted, because every file:line anchor had drifted by HEAD."
  - "The guard is honest about its ceiling: it detects three literal emission markers, so a new hand-built json! text shape (like tokens_handler) is enumerated in LEDGER but not literally detectable and remains a review responsibility."
  - "The historical 02-05-PLAN Task 3 narrowing is annotated with a G-02-3 correction note rather than rewritten, so re-execution semantics are unchanged."

patterns-established:
  - "Answer-path completeness is guarded, not asserted: crates/application/tests/answer_path_ledger.rs resolves every ledger symbol and every marker occurrence, failing closed on drift or an unrecorded emitting handler."
  - "Evidence-of-record copies stay in lockstep: the same corrected sweep appears in the in-code doc, both evidence files, and the summary, each pointing at the guard."

requirements-completed: [REQ-quran-corpus, REQ-ingestion-validation-eval]

coverage:
  - id: D1
    description: "All three evidence-of-record files carry the complete emitting-path sweep (2 enforced, 10 AyahView.canonical, 1 hand-built token surface, 7 SearchHit.quotation, resolve no-text, debug-only disposition, not-emitters), each path with an emission-mechanism basis and a path#symbol reference, and the false blanket completeness claim is gone."
    requirement: "REQ-quran-corpus"
    verification:
      - kind: other
        ref: "grep gate over docs/06-progress/phase-02-evidence.md, docs/05-followups/phase-02-owner-gates.md, .planning/phases/02-canonical-quran-core/02-05-SUMMARY.md (all nine omitted symbols + quran.search + #cmd_search present) -> LEDGER_OK"
        status: pass
    human_judgment: true
    rationale: "Whether the exemption set is complete and correctly reasoned remains an architectural audit judgment; the guard proves the recorded set is drift-proof, not that a novel emission shape was conceived."
  - id: D2
    description: "The ReaderCitationSource in-code doc carries the complete sweep with all three emission shapes and the guard pointer; 02-RESEARCH §F-6 gains the search-path rows and a frame-correction note; the historical 02-05-PLAN Task 3 narrowing is annotated with a G-02-3 note without altering the executed instruction."
    requirement: "REQ-ingestion-validation-eval"
    verification:
      - kind: other
        ref: "cargo build -p application + rg gate (search_exact_handler and quran.search in quran_tools.rs; search_exact_handler in 02-RESEARCH.md; G-02-3 in 02-05-PLAN.md) -> DOCS_OK"
        status: pass
    human_judgment: false
  - id: D3
    description: "A mechanical source-scan guard exists that resolves every ledger symbol kind-aware (async fn and struct), maps every detectable emitting marker to a recorded path, and contains self-tests proving it fails closed on an unlisted emitting handler and on a mismatched definition kind."
    requirement: "REQ-quran-corpus"
    verification:
      - kind: integration
        ref: "crates/application/tests/answer_path_ledger.rs#every_ledger_symbol_resolves_under_its_declared_kind"
        status: pass
      - kind: integration
        ref: "crates/application/tests/answer_path_ledger.rs#emitting_markers_resolve_to_recorded_paths"
        status: pass
      - kind: integration
        ref: "crates/application/tests/answer_path_ledger.rs#scanner_reports_an_unlisted_emitting_handler"
        status: pass
      - kind: integration
        ref: "crates/application/tests/answer_path_ledger.rs#kind_aware_resolution_rejects_a_mismatched_definition"
        status: pass
      - kind: integration
        ref: "crates/application/tests/answer_path_ledger.rs#cli_search_quotation_extraction_is_inside_cmd_search"
        status: pass
      - kind: integration
        ref: "crates/application/tests/answer_path_ledger.rs#tool_search_path_is_present"
        status: pass
    human_judgment: false

duration: 5 min
completed: 2026-10-04
status: complete
---

# Phase 02 Plan 08: Answer-path exemption ledger (G-02-3) Summary

**Rebuilt the D-15 answer-path ledger on the emission frame — 2 enforced, 18 structurally-exempt emitting paths, plus explicit no-text/debug-only/not-emitter dispositions — with drift-proof `path#symbol` refs, the false completeness claim corrected in all four copies, and a checked-list source-scan guard that fails when a new marker-emitting path goes unrecorded.**

## Performance

- **Duration:** 5 min
- **Started:** 2026-10-04T14:13:34Z
- **Completed:** 2026-10-04T14:18:27Z
- **Tasks:** 3
- **Files modified:** 7 (1 created, 6 modified)

## Accomplishments

- Closed UAT gap G-02-3 (`docs/06-progress/phase-02-evidence.md` §3, `docs/05-followups/phase-02-owner-gates.md` §D-15, and `02-05-SUMMARY.md` §Structural Exemptions now all carry the complete emission-frame sweep): 2 enforced paths, 10 `AyahView.canonical` paths (including the previously-dropped HTTP `surah_handler`/`divisions_handler`), 1 hand-built token-surface path (`tokens_handler`), 7 `SearchHit.quotation` paths (the five HTTP search handlers, CLI `cmd_search`, tool `quran.search`), `resolve_handler` as no-text, `debug_reader_handler` explicitly debug-only, and a not-emitters list that makes the sweep provably exhaustive.
- Replaced every stale `file:line` anchor with a drift-proof `path#symbol` reference across the three evidence files, and removed the false blanket "every answer path" completeness claim from `02-05-SUMMARY.md` D4 and §Structural Exemptions, the evidence §3, the owner-gates record, and the `ReaderCitationSource` doc.
- Corrected the enumeration source: `02-RESEARCH.md` §F-6 gains rows for the index-backed search paths and `quran.search` plus a frame-correction note; the historical `02-05-PLAN.md` Task 3 narrowing is annotated with a G-02-3 note without altering the executed instruction.
- Added `crates/application/tests/answer_path_ledger.rs`: a checked-list source-scan guard that resolves every `(file, symbol, kind)` kind-aware, asserts every occurrence of the three emission markers maps to a recorded path (`arabic_text` 3, `search_response` 5, `ok_envelope` 2), and self-tests fail-closed behaviour on an unlisted emitting handler and a mismatched definition kind.

## Task Commits

Each task was committed atomically:

1. **Task 1: Rebuild the answer-path ledger in the three evidence-of-record files** — `f0c3d45` (docs)
2. **Task 2: Correct the in-code doc, the research enumeration source, and the historical plan narrowing** — `9b8280c` (docs)
3. **Task 3: Add the source-scan ledger guard test** — `d341b3c` (test)

**Plan metadata:** committed with this SUMMARY (docs: complete plan).

## Files Created/Modified

- `docs/06-progress/phase-02-evidence.md` — §3 rebuilt on the emission frame with per-path mechanism + `path#symbol` basis and the guard pointer.
- `docs/05-followups/phase-02-owner-gates.md` — §D-15 tables rebuilt to the same complete sweep.
- `.planning/phases/02-canonical-quran-core/02-05-SUMMARY.md` — D4 description corrected; §Structural Exemptions rebuilt with the complete sweep.
- `crates/application/src/quran_tools.rs` — `ReaderCitationSource` doc expanded to the complete sweep (doc-only change; no code touched).
- `.planning/phases/02-canonical-quran-core/02-RESEARCH.md` — §F-6 search rows + frame-correction note.
- `.planning/phases/02-canonical-quran-core/02-05-PLAN.md` — G-02-3 correction note after Task 3 without changing the historical action/AC.
- `crates/application/tests/answer_path_ledger.rs` — the source-scan guard test (new).

## Decisions Made

- **Emission frame, not direct-read frame.** The ledger now enumerates every path whose response contains quoted canonical text; this is what D-15's wording and the plan's success criterion required. The enforced paths are unchanged and the omitted paths are exempt-by-construction for the same reason the recorded ones are.
- **Per-path emission mechanism.** `AyahView.canonical`, `SearchHit.quotation`, and the hand-built token surface are all exempt by construction; `tokens_handler` needs its own label because it does not emit via `AyahView.canonical`.
- **`path#symbol` refs only.** Every `file:line` anchor had drifted by HEAD, so symbol references are primary and line anchors are omitted.
- **Guard honesty.** The test detects three literal markers; a new hand-built `json!` text shape is enumerated in `LEDGER` but not literally detectable, and the doc comment records that as a review responsibility.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- The plan's Task 3 threshold comments ("occurrence count is 3"/"5"/"2") matched the current tree exactly, so the count assertions are pinned to the present counts; a future change to any of those counts will (correctly) fail the guard and force a ledger review.

## Threat Flags

None — no new network endpoint, auth path, file-access pattern, schema change, or quotation type. The changes are documentation, one in-code doc comment, and one test file. `crates/application/src/quran_tools.rs` changed in doc comments only (verified: `git diff` shows no non-comment lines).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- G-02-3 truth now holds and is mechanically guarded: every emitting answer path is enforced or recorded (structurally exempt / no-text / debug-only) with a basis and emission mechanism, and `cargo test -p application --test answer_path_ledger` fails closed on drift or a new marker-emitting handler.
- No production enforcement behaviour, route, verdict vocabulary, or quotation type changed: `git diff --stat` since the plan base touches only the six evidence/doc files and the one new test file.
- The evidence-of-record copies stay in lockstep and point at the guard; the D-15 read-path exemption remains an owner-ratifiable interpretation, not a closed decision.

---
*Phase: 02-canonical-quran-core*
*Completed: 2026-10-04*

## Self-Check: PASSED

- Key files exist: `docs/06-progress/phase-02-evidence.md`, `docs/05-followups/phase-02-owner-gates.md`, `.planning/phases/02-canonical-quran-core/02-05-SUMMARY.md`, `crates/application/src/quran_tools.rs`, `.planning/phases/02-canonical-quran-core/02-RESEARCH.md`, `.planning/phases/02-canonical-quran-core/02-05-PLAN.md`, `crates/application/tests/answer_path_ledger.rs`.
- Task commits exist: `f0c3d45`, `9b8280c`, `d341b3c`.
