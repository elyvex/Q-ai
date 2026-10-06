---
status: complete
phase: 02-canonical-quran-core
source: [02-01-SUMMARY.md, 02-02-SUMMARY.md, 02-03-SUMMARY.md, 02-04-SUMMARY.md, 02-05-SUMMARY.md, 02-06-SUMMARY.md, 02-07-SUMMARY.md, 02-08-SUMMARY.md]
started: 2026-10-03T16:12:52Z
updated: 2026-10-06T01:04:30Z
---

## Current Test

[testing complete]

## Tests

### 1. Cold Start Smoke Test
expected: Kill any running server/service, clear ephemeral state (temp DBs, caches, lock files), start the app from scratch — server boots without errors, migrations complete, and a primary query (health check / homepage / basic API call) returns live data.
result: pass

### 2. Auto-Covered Deliverables Confirmation
expected: |
  31 of 34 deliverables are auto-covered by passing tests (tests 6-36, source: automated): 02-01 identity/license/primary (5), 02-02 synthetic fixtures (4), 02-03 quran verify surfaces (6), 02-04 write fence + approval gate (4), 02-05 verify-quotation + citation enforcement (4), 02-06 translation hashing/license (4), 02-07 integrity artifact + evidence + coverage gate + task closure (4). Confirm you accept that test evidence as sufficient — no manual checking needed.
result: pass

### 3. Answer-Path Citation Enforcement Audit
expected: Every answer path that emits quoted canonical text is either enforced or recorded as structurally exempt with a per-path file:line basis (tool get_ayah/get_context; CLI cmd_get/context/surah/division; HTTP direct reads). The exemption set is auditable and complete.
result: pass
reported: "Closed by gap-closure plan 02-08 (G-02-3). Emission-frame ledger rebuilt in all three evidence-of-record files + in-code ReaderCitationSource doc; every path#symbol ref resolves at HEAD; the false blanket completeness claim corrected; drift guard crates/application/tests/answer_path_ledger.rs added (6 tests green). Original report: ledger was incomplete — 9 emitting answer paths in neither list plus drifted line anchors."
note: "Residual scope decision on the new guard's coverage claim surfaced as test 37 (WR-01/WR-02)."
severity: major
resolved_by: 02-08
resolved: 2026-10-04

### 4. Blocked-Gate Ledger Audit (OD-01/OD-02/OD-03)
expected: OD-01/OD-02/OD-03 are recorded as blocked human-only gates with their question, what they block, the closing action, and the exact command; the record is agent-uncloseable and no gate is marked resolved.
result: pass

### 5. D-15 Read-Path Exemption Ratification
expected: The D-15 read-path exemption is recorded as an explicit owner-ratifiable interpretation with a per-path file:line basis, and is NOT presented as a satisfied locked decision.
result: pass

### 6. [02-01 D1] Manifest-declared upstream_edition_slug / qai_edition_id / is_primary survive import → staging → activation and are readable through the canonical row, the reader view, and the operator edition-show surface.
expected: Manifest-declared upstream_edition_slug / qai_edition_id / is_primary survive import → staging → activation and are readable through the canonical row, the reader view, and the operator edition-show surface.
result: pass
source: automated
coverage_id: D1

### 7. [02-01 D2] Manifest-declared license is persisted verbatim; an undeclared license stays explicit Unknown with no invented redistribution permission.
expected: Manifest-declared license is persisted verbatim; an undeclared license stays explicit Unknown with no invented redistribution permission.
result: pass
source: automated
coverage_id: D2

### 8. [02-01 D3] Append-only migration 0020 adds the three columns to both canonical and staging tables; migrations 0007–0019 are untouched and the checksum ledger gains exactly one entry.
expected: Append-only migration 0020 adds the three columns to both canonical and staging tables; migrations 0007–0019 are untouched and the checksum ledger gains exactly one entry.
result: pass
source: automated
coverage_id: D3

### 9. [02-01 D4] EditionSelector::Primary resolves the flagged edition and returns typed errors when none or more than one is flagged, never falling back to the active pointer.
expected: EditionSelector::Primary resolves the flagged edition and returns typed errors when none or more than one is flagged, never falling back to the active pointer.
result: pass
source: automated
coverage_id: D4

### 10. [02-01 D5] docs/schemas/quran-edition-source.v1.schema.json declares the matching edition.is_primary boolean property and the committed schema is CI-quiet under gen-schema.
expected: docs/schemas/quran-edition-source.v1.schema.json declares the matching edition.is_primary boolean property and the committed schema is CI-quiet under gen-schema.
result: pass
source: automated
coverage_id: D5

### 11. [02-02 D1] Richer synthetic edition (test-edition-rich, 6 surahs / 24 ayahs) with a CSV mirror that reproduces its JSON ayahs field-for-field and a golden set covering every ayah.
expected: Richer synthetic edition (test-edition-rich, 6 surahs / 24 ayahs) with a CSV mirror that reproduces its JSON ayahs field-for-field and a golden set covering every ayah.
result: pass
source: automated
coverage_id: D1

### 12. [02-02 D2] Companion synthetic reference edition (reference.json) with a declared synthetic reference_corpus_id and a sha256 reference_text_hash pin, byte-identical to the rich ayah stream.
expected: Companion synthetic reference edition (reference.json) with a declared synthetic reference_corpus_id and a sha256 reference_text_hash pin, byte-identical to the rich ayah stream.
result: pass
source: automated
coverage_id: D2

### 13. [02-02 D3] Synthetic identity/primary/license fixture (test-edition-identity) declaring is_primary: true, a non-null upstream_edition_slug/qai_edition_id, and a concrete declared license.
expected: Synthetic identity/primary/license fixture (test-edition-identity) declaring is_primary: true, a non-null upstream_edition_slug/qai_edition_id, and a concrete declared license.
result: pass
source: automated
coverage_id: D3

### 14. [02-02 D4] All three new fixtures are import-viable: validate_edition yields no Fatal/Error finding, so run_import (and plan 02-03's import/activate snapshots) succeed.
expected: All three new fixtures are import-viable: validate_edition yields no Fatal/Error finding, so run_import (and plan 02-03's import/activate snapshots) succeed.
result: pass
source: automated
coverage_id: D4

### 15. [02-03 D1] qai quran verify reports the six integrity families for the active edition from persisted state, exits 3 when any family fails, and is read-only (corpus_generation unchanged).
expected: qai quran verify reports the six integrity families for the active edition from persisted state, exits 3 when any family fails, and is read-only (corpus_generation unchanged).
result: pass
source: automated
coverage_id: D1

### 16. [02-03 D2] The reference-comparison family is state-derived: skipped when the persisted QV-015 Info skip applies, pass only on a real comparison, fail on a QV-015 Fatal; a skip is never a pass.
expected: The reference-comparison family is state-derived: skipped when the persisted QV-015 Info skip applies, pass only on a real comparison, fail on a QV-015 Fatal; a skip is never a pass.
result: pass
source: automated
coverage_id: D2

### 17. [02-03 D3] The pinned edition reference and the STORED per-ayah canonical hash are printed together on the qai quran get human line (the hash is read from AyahView, never recomputed).
expected: The pinned edition reference and the STORED per-ayah canonical hash are printed together on the qai quran get human line (the hash is read from AyahView, never recomputed).
result: pass
source: automated
coverage_id: D3

### 18. [02-03 D4] qai quran import --reference evaluates QV-015 end-to-end: a matching reference persists outcome pass with one DifferenceClass per difference from the existing vocabulary; a mismatch fails closed with the stage untouched and no active edition.
expected: qai quran import --reference evaluates QV-015 end-to-end: a matching reference persists outcome pass with one DifferenceClass per difference from the existing vocabulary; a mismatch fails closed with the stage untouched and no active edition.
result: pass
source: automated
coverage_id: D4

### 19. [02-03 D5] qai quran edition show (human and --json) reports the manifest-declared upstream identity, primary marker, and declared license status for an activated identity fixture, with no volatile ids on the human line.
expected: qai quran edition show (human and --json) reports the manifest-declared upstream identity, primary marker, and declared license status for an activated identity fixture, with no volatile ids on the human line.
result: pass
source: automated
coverage_id: D5

### 20. [02-03 D6] No migration or architecture boundary change: the plan adds operator surfaces only.
expected: No migration or architecture boundary change: the plan adds operator surfaces only.
result: pass
source: automated
coverage_id: D6

### 21. [02-04 D1] Append-only migration 0021 fences every previously untriggered canonical table and verb with a unique coded abort, plus DELETE on quran_editions and immutability of the 0020 identity/default columns.
expected: Append-only migration 0021 fences every previously untriggered canonical table and verb with a unique coded abort, plus DELETE on quran_editions and immutability of the 0020 identity/default columns.
result: pass
source: automated
coverage_id: D1

### 22. [02-04 D2] ApprovalToken is only mintable from a persisted granted approval row and names its exact subject; ApprovalGate implements CanonicalWriter and gates activate/rollback/verify/deprecate inside the existing single UnitOfWork.
expected: ApprovalToken is only mintable from a persisted granted approval row and names its exact subject; ApprovalGate implements CanonicalWriter and gates activate/rollback/verify/deprecate inside the existing single UnitOfWork.
result: pass
source: automated
coverage_id: D2

### 23. [02-04 D3] The importer has no code path to canonical tables: no ApprovalToken/CanonicalWriter reference, no activation/rollback call, no canonical-table INSERT, and a terminal run state of Staged.
expected: The importer has no code path to canonical tables: no ApprovalToken/CanonicalWriter reference, no activation/rollback call, no canonical-table INSERT, and a terminal run state of Staged.
result: pass
source: automated
coverage_id: D3

### 24. [02-04 D4] Rollback rejection is proven (missing/denied/mismatched approval and rollback-to-active leave pointer, canonical rows, and generation unchanged), rollback invalidates the reader cache (no stale v2 text, generation strictly increases), and the quran_segments scope plus trigger-code map are recorded.
expected: Rollback rejection is proven (missing/denied/mismatched approval and rollback-to-active leave pointer, canonical rows, and generation unchanged), rollback invalidates the reader cache (no stale v2 text, generation strictly increases), and the quran_segments scope plus trigger-code map are recorded.
result: pass
source: automated
coverage_id: D4

### 25. [02-05 D1] The shared verdict-to-hard-failure mapping exists on the citations domain crate: exact/whitespace matches pass, and Mismatch/LocationNotFound/EditionNotFound/AccessDenied map to distinct typed QAI-QUR-* codes; MatchAfterDeclaredNormalization is not in the reachable set.
expected: The shared verdict-to-hard-failure mapping exists on the citations domain crate: exact/whitespace matches pass, and Mismatch/LocationNotFound/EditionNotFound/AccessDenied map to distinct typed QAI-QUR-* codes; MatchAfterDeclaredNormalization is not in the reachable set.
result: pass
source: automated
coverage_id: D1

### 26. [02-05 D2] qai quran verify-quotation is read-only, takes an explicit pinned slug@version, prints the verdict and the RESOLVED canonical hash, exits 0 on an exact match and 3 with a typed code on a mismatch (5 for a location/edition not found).
expected: qai quran verify-quotation is read-only, takes an explicit pinned slug@version, prints the verdict and the RESOLVED canonical hash, exits 0 on an exact match and 3 with a typed code on a mismatch (5 for a location/edition not found).
result: pass
source: automated
coverage_id: D2

### 27. [02-05 D3] The HTTP citation answer path refuses a 200 success envelope for a stored hard-failing verdict (typed error, no data envelope) while an exact-match stored citation still returns its resolved citation unchanged; no new route is introduced.
expected: The HTTP citation answer path refuses a 200 success envelope for a stored hard-failing verdict (typed error, no data envelope) while an exact-match stored citation still returns its resolved citation unchanged; no new route is introduced.
result: pass
source: automated
coverage_id: D3

### 28. [02-05 D5] No frozen verdict vocabulary, quotation type, endpoint, or crate-boundary change: the work is additive helpers, one read-only verb, and one handler enforcement.
expected: No frozen verdict vocabulary, quotation type, endpoint, or crate-boundary change: the work is additive helpers, one read-only verb, and one handler enforcement.
result: pass
source: automated
coverage_id: D5

### 29. [02-06 D1] Every imported translation carries a real order-independent content hash produced by a new domain-separated recipe; the three frozen v1 recipes are byte-identical.
expected: Every imported translation carries a real order-independent content hash produced by a new domain-separated recipe; the three frozen v1 recipes are byte-identical.
result: pass
source: automated
coverage_id: D1

### 30. [02-06 D2] A translation's declared license is persisted verbatim; an undeclared license yields an explicit unknown status with no invented redistribution permission.
expected: A translation's declared license is persisted verbatim; an undeclared license yields an explicit unknown status with no invented redistribution permission.
result: pass
source: automated
coverage_id: D2

### 31. [02-06 D3] A served view's canonical slot is the canonical quotation (canonical Arabic + canonical hash); the translation rides only in the attributed sidecar; no canonical view constructor accepts a translator/language pair.
expected: A served view's canonical slot is the canonical quotation (canonical Arabic + canonical hash); the translation rides only in the attributed sidecar; no canonical view constructor accepts a translator/language pair.
result: pass
source: automated
coverage_id: D3

### 32. [02-06 D4] No frozen recipe, no migration, and no canonical type changed: the additive work is confined to a new recipe and the translation import path, and the architecture boundary still holds.
expected: No frozen recipe, no migration, and no canonical type changed: the additive work is confined to a new recipe and the translation import path, and the architecture boundary still holds.
result: pass
source: automated
coverage_id: D4

### 33. [02-07 D3] A committed corpus-integrity artifact classifies exactly the six integrity families, carries the pinned edition identity and the persisted QV-015 evidence, records the reference family honestly, and contains no volatile identity.
expected: A committed corpus-integrity artifact classifies exactly the six integrity families, carries the pinned edition identity and the persisted QV-015 evidence, records the reference family honestly, and contains no volatile identity.
result: pass
source: automated
coverage_id: D3

### 34. [02-07 D4] The evidence of record maps each of the five roadmap success criteria to an exact command, observed result, and proving artifact/test; dispositions all six spec-less edge-probe rows; records the D-15 exemption and the legacy snapshot/fmt baseline with a quiet-worktree instruction.
expected: The evidence of record maps each of the five roadmap success criteria to an exact command, observed result, and proving artifact/test; dispositions all six spec-less edge-probe rows; records the D-15 exemption and the legacy snapshot/fmt baseline with a quiet-worktree instruction.
result: pass
source: automated
coverage_id: D4

### 35. [02-07 D5] The coverage gate enforces published Quran-crate floors (quran-core/corpus at 90%) and records the citations shortfall to the published 85% floor as a named follow-up; existing rows and unrelated CI steps are unchanged.
expected: The coverage gate enforces published Quran-crate floors (quran-core/corpus at 90%) and records the citations shortfall to the published 85% floor as a named follow-up; existing rows and unrelated CI steps are unchanged.
result: pass
source: automated
coverage_id: D5

### 36. [02-07 D6] TASK-002-canonical-quran-core is closed through the AGENTS.md lifecycle: it exists under docs/04-tasks/completed/ (not active), maps plans 02-01…02-07 and the five success criteria, has a dated completion section, and a newest-first rollup entry.
expected: TASK-002-canonical-quran-core is closed through the AGENTS.md lifecycle: it exists under docs/04-tasks/completed/ (not active), maps plans 02-01…02-07 and the five success criteria, has a dated completion section, and a newest-first rollup entry.
result: pass
source: automated
coverage_id: D6

### 37. Decide the answer-path ledger guard's coverage scope (WR-01/WR-02)
expected: |
  Either (a) extend crates/application/tests/answer_path_ledger.rs to scan quran_cli.rs/quran_tools.rs emission markers and add citation_handler/resolve_handler to LEDGER, or (b) narrow the guard module doc (answer_path_ledger.rs:6-8) and the ReaderCitationSource doc (quran_tools.rs:486-488) plus evidence §3 wording to the actually-scanned api.rs frame + CLI cmd_search. Then re-run `cargo test -p application --test answer_path_ledger`.
why_human: "The guard is green and fails closed for the HTTP api.rs emission frame (where G-02-3 actually occurred), but its module doc claims any new marker-emitting handler fails the build — demonstrably false for quran_cli.rs (4 .arabic_text() sites) and quran_tools.rs (1), and LEDGER omits citation_handler/resolve_handler that the ledger docs name. Choosing extend-vs-narrow is a scope/ratification decision."
result: pass
decided: "Accepted the guard's current scope (api.rs HTTP emission frame, where G-02-3 occurred). WR-01/WR-02 recorded as an advisory guard-coverage ceiling, not a phase blocker — a future broadening can extend the scan to quran_cli.rs/quran_tools.rs and add citation_handler/resolve_handler to LEDGER."

## Summary

total: 37
passed: 37
issues: 0
pending: 0
skipped: 0
blocked: 0

## Auto-Covered
- [02-01 D1] Manifest-declared upstream_edition_slug / qai_edition_id / is_primary survive import → staging → activation and are readable through the canonical row, the reader view, and the operator edition-show surface.
- [02-01 D2] Manifest-declared license is persisted verbatim; an undeclared license stays explicit Unknown with no invented redistribution permission.
- [02-01 D3] Append-only migration 0020 adds the three columns to both canonical and staging tables; migrations 0007–0019 are untouched and the checksum ledger gains exactly one entry.
- [02-01 D4] EditionSelector::Primary resolves the flagged edition and returns typed errors when none or more than one is flagged, never falling back to the active pointer.
- [02-01 D5] docs/schemas/quran-edition-source.v1.schema.json declares the matching edition.is_primary boolean property and the committed schema is CI-quiet under gen-schema.
- [02-02 D1] Richer synthetic edition (test-edition-rich, 6 surahs / 24 ayahs) with a CSV mirror that reproduces its JSON ayahs field-for-field and a golden set covering every ayah.
- [02-02 D2] Companion synthetic reference edition (reference.json) with a declared synthetic reference_corpus_id and a sha256 reference_text_hash pin, byte-identical to the rich ayah stream.
- [02-02 D3] Synthetic identity/primary/license fixture (test-edition-identity) declaring is_primary: true, a non-null upstream_edition_slug/qai_edition_id, and a concrete declared license.
- [02-02 D4] All three new fixtures are import-viable: validate_edition yields no Fatal/Error finding, so run_import (and plan 02-03's import/activate snapshots) succeed.
- [02-03 D1] qai quran verify reports the six integrity families for the active edition from persisted state, exits 3 when any family fails, and is read-only (corpus_generation unchanged).
- [02-03 D2] The reference-comparison family is state-derived: skipped when the persisted QV-015 Info skip applies, pass only on a real comparison, fail on a QV-015 Fatal; a skip is never a pass.
- [02-03 D3] The pinned edition reference and the STORED per-ayah canonical hash are printed together on the qai quran get human line (the hash is read from AyahView, never recomputed).
- [02-03 D4] qai quran import --reference evaluates QV-015 end-to-end: a matching reference persists outcome pass with one DifferenceClass per difference from the existing vocabulary; a mismatch fails closed with the stage untouched and no active edition.
- [02-03 D5] qai quran edition show (human and --json) reports the manifest-declared upstream identity, primary marker, and declared license status for an activated identity fixture, with no volatile ids on the human line.
- [02-03 D6] No migration or architecture boundary change: the plan adds operator surfaces only.
- [02-04 D1] Append-only migration 0021 fences every previously untriggered canonical table and verb with a unique coded abort, plus DELETE on quran_editions and immutability of the 0020 identity/default columns.
- [02-04 D2] ApprovalToken is only mintable from a persisted granted approval row and names its exact subject; ApprovalGate implements CanonicalWriter and gates activate/rollback/verify/deprecate inside the existing single UnitOfWork.
- [02-04 D3] The importer has no code path to canonical tables: no ApprovalToken/CanonicalWriter reference, no activation/rollback call, no canonical-table INSERT, and a terminal run state of Staged.
- [02-04 D4] Rollback rejection is proven (missing/denied/mismatched approval and rollback-to-active leave pointer, canonical rows, and generation unchanged), rollback invalidates the reader cache (no stale v2 text, generation strictly increases), and the quran_segments scope plus trigger-code map are recorded.
- [02-05 D1] The shared verdict-to-hard-failure mapping exists on the citations domain crate: exact/whitespace matches pass, and Mismatch/LocationNotFound/EditionNotFound/AccessDenied map to distinct typed QAI-QUR-* codes; MatchAfterDeclaredNormalization is not in the reachable set.
- [02-05 D2] qai quran verify-quotation is read-only, takes an explicit pinned slug@version, prints the verdict and the RESOLVED canonical hash, exits 0 on an exact match and 3 with a typed code on a mismatch (5 for a location/edition not found).
- [02-05 D3] The HTTP citation answer path refuses a 200 success envelope for a stored hard-failing verdict (typed error, no data envelope) while an exact-match stored citation still returns its resolved citation unchanged; no new route is introduced.
- [02-05 D5] No frozen verdict vocabulary, quotation type, endpoint, or crate-boundary change: the work is additive helpers, one read-only verb, and one handler enforcement.
- [02-06 D1] Every imported translation carries a real order-independent content hash produced by a new domain-separated recipe; the three frozen v1 recipes are byte-identical.
- [02-06 D2] A translation's declared license is persisted verbatim; an undeclared license yields an explicit unknown status with no invented redistribution permission.
- [02-06 D3] A served view's canonical slot is the canonical quotation (canonical Arabic + canonical hash); the translation rides only in the attributed sidecar; no canonical view constructor accepts a translator/language pair.
- [02-06 D4] No frozen recipe, no migration, and no canonical type changed: the additive work is confined to a new recipe and the translation import path, and the architecture boundary still holds.
- [02-07 D3] A committed corpus-integrity artifact classifies exactly the six integrity families, carries the pinned edition identity and the persisted QV-015 evidence, records the reference family honestly, and contains no volatile identity.
- [02-07 D4] The evidence of record maps each of the five roadmap success criteria to an exact command, observed result, and proving artifact/test; dispositions all six spec-less edge-probe rows; records the D-15 exemption and the legacy snapshot/fmt baseline with a quiet-worktree instruction.
- [02-07 D5] The coverage gate enforces published Quran-crate floors (quran-core/corpus at 90%) and records the citations shortfall to the published 85% floor as a named follow-up; existing rows and unrelated CI steps are unchanged.
- [02-07 D6] TASK-002-canonical-quran-core is closed through the AGENTS.md lifecycle: it exists under docs/04-tasks/completed/ (not active), maps plans 02-01…02-07 and the five success criteria, has a dated completion section, and a newest-first rollup entry.

## Gaps

[none yet]

- gap_id: G-02-3
  truth: "Every answer path that emits quoted canonical text is either enforced or recorded as structurally exempt with a per-path file:line basis"
  status: resolved
  resolved_by: 02-08
  resolved: 2026-10-04
  resolution: "Emission-frame ledger rebuilt in all three evidence-of-record files + in-code doc; all path#symbol refs resolve at HEAD; false completeness claim corrected in all copies; drift guard added (6 tests green). Residual guard-coverage scope decision tracked as test 37 (WR-01/WR-02), not a reopening of G-02-3."
  reason: "User reported: ledger is incomplete — 9 emitting answer paths are in neither list (HTTP surah_handler, divisions_handler, 5x search_*; CLI cmd_search; tool quran.search) plus 1 borderline debug route, and the recorded line anchors have drifted against HEAD"
  severity: major
  test: 3
  root_cause: "Frame error at authoring time: the D-15 audit enumerated paths by 'direct read of the canonical reader' instead of 'every path whose response contains quoted canonical text'. 02-RESEARCH.md §F-6 (the enumeration source) has no search row, and 02-05-PLAN Task 3 <action> narrowed the deliverable to tool get_ayah/get_context + CLI/HTTP direct reads even though its own <success_criteria> demanded 'every answer path'. Separately, HTTP surah_handler/divisions_handler WERE listed in §F-6 but were dropped in transcription to the ledger; the tool quran.search (added 03-07, ca91230) was never dispositioned; all file:line anchors were frozen at 06f651f and have since drifted. 8 of the 9 missing paths already existed at the audit commit — authoring omissions, not later evolution."
  artifacts:
    - path: ".planning/phases/02-canonical-quran-core/02-05-SUMMARY.md"
      issue: "§Structural Exemptions table (lines 170-181) omits search/surah/divisions paths; false 'every answer path' completeness claim at line 172"
    - path: "docs/06-progress/phase-02-evidence.md"
      issue: "§3 table (lines 172-192) repeats the same omission; all 33 unique refs across the three evidence files are stale at HEAD"
    - path: "docs/05-followups/phase-02-owner-gates.md"
      issue: "§Owner-ratifiable interpretation table (lines 155-175) repeats the same omission"
    - path: "crates/application/src/quran_tools.rs"
      issue: "ReaderCitationSource doc (lines 456-474) is an in-code copy of the narrowed list"
    - path: ".planning/phases/02-canonical-quran-core/02-RESEARCH.md"
      issue: "§F-6 (lines 491-498) enumeration source has no search row; D-15 restated broadly at line 34"
    - path: ".planning/phases/02-canonical-quran-core/02-05-PLAN.md"
      issue: "Task 3 <action> (line 146) narrows what <success_criteria> (line 198) demands"
    - path: "crates/server/src/api.rs"
      issue: "surah_handler :369 (emit :384) and divisions_handler :498 (emit :523) emit view.canonical.arabic_text(); search_*_handler :1021/:1049/:1077/:1113/:1145 emit SearchHit.quotation"
  missing:
    - "Exempt rows for HTTP surah_handler and divisions_handler"
    - "Exempt rows for the five HTTP search handlers (exact/normalized/phrase/concatenated/regex)"
    - "Exempt row for CLI cmd_search"
    - "Exempt row for tool quran.search"
    - "Explicit disposition for HTTP debug_reader_handler (/debug/read/{edition}/{surah}, api.rs:2002) — exempt or documented debug-only out-of-scope — so the set is exhaustive"
    - "Emission-mechanism basis per path (AyahView.canonical vs SearchHit.quotation — both validated QuranQuotations built from canonical rows, exempt-by-construction)"
    - "Re-anchor or symbol-convert all refs (path#symbol is drift-proof); correct the false 'every answer path' completeness claim in 02-05-SUMMARY D4 and the ReaderCitationSource doc"
  debug_session: ".planning/debug/answer-path-exemption-ledger.md"
