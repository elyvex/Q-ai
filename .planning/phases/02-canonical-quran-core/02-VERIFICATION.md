---
phase: 02-canonical-quran-core
verified: 2026-10-04T15:31:19Z
status: passed
score: 8/9 must-haves verified
covered_files:

  - .planning/REQUIREMENTS.md
  - .planning/phases/02-canonical-quran-core/02-01-PLAN.md
  - .planning/phases/02-canonical-quran-core/02-01-SUMMARY.md
  - .planning/phases/02-canonical-quran-core/02-02-PLAN.md
  - .planning/phases/02-canonical-quran-core/02-02-SUMMARY.md
  - .planning/phases/02-canonical-quran-core/02-03-PLAN.md
  - .planning/phases/02-canonical-quran-core/02-03-SUMMARY.md
  - .planning/phases/02-canonical-quran-core/02-04-PLAN.md
  - .planning/phases/02-canonical-quran-core/02-04-SUMMARY.md
  - .planning/phases/02-canonical-quran-core/02-05-PLAN.md
  - .planning/phases/02-canonical-quran-core/02-05-SUMMARY.md
  - .planning/phases/02-canonical-quran-core/02-06-PLAN.md
  - .planning/phases/02-canonical-quran-core/02-06-SUMMARY.md
  - .planning/phases/02-canonical-quran-core/02-07-PLAN.md
  - .planning/phases/02-canonical-quran-core/02-07-SUMMARY.md
  - .planning/phases/02-canonical-quran-core/02-08-PLAN.md
  - .planning/phases/02-canonical-quran-core/02-08-SUMMARY.md
  - crates/application/src/quran_tools.rs
  - crates/application/tests/answer_path_ledger.rs
  - docs/05-followups/phase-02-owner-gates.md
  - docs/06-progress/phase-02-evidence.md

covered_digest: "v1:sha256:a4490959afb556550c7eac46e17b4f3793bc13f701e731ecc084b8068296584a"
behavior_unverified: 0
overrides_applied: 0
re_verification:
  previous_status: passed
  previous_score: 5/5
  gaps_closed:
    - "G-02-3: Every answer path that emits quoted canonical text is either enforced or recorded as structurally exempt with a per-path basis"
  gaps_remaining: []
  regressions: []
advisory:

  - finding: "Pre-existing WR-03: search_meta(None) attributes the quran.search envelope to the active edition while the indexed edition serves hits"
    category: architectural
    reason: "Reviewer-recorded, evidenced in quran_tools.rs/search_normalized; it is phase-03 legacy code outside the 02-08 delta (base d3a0a99 predates phase 03), so it is not a Phase 2 regression and is not actionable here"
    evidence_status: "reviewer finding, out of this phase's contract"
human_verification:

  - test: "Decide the answer-path ledger guard's coverage scope (WR-01/WR-02) and reconcile it with the guard's completeness claim"
    expected: "Either (a) extend crates/application/tests/answer_path_ledger.rs to scan quran_cli.rs/quran_tools.rs emission markers and add citation_handler/resolve_handler to LEDGER, or (b) narrow the guard module doc (answer_path_ledger.rs:6-8) and the `ReaderCitationSource` doc (quran_tools.rs:486-488) + evidence §3 wording from 'the complete sweep is guarded mechanically' to the actually-scanned api.rs frame + CLI cmd_search. Then re-run `cargo test -p application --test answer_path_ledger`."
    why_human: "The guard is green and fails closed for the HTTP api.rs emission frame (where G-02-3 actually occurred), but its own module doc claims any new marker-emitting handler fails the build — demonstrably false for quran_cli.rs (4 `.arabic_text()` sites) and quran_tools.rs (1), and LEDGER omits `citation_handler` (the enforced HTTP path) and `resolve_handler` that the ledger docs name. Choosing between extending coverage and narrowing the claim is a scope/ratification decision, not a code-uniformity check."
---

# Phase 02: Canonical Quran Core Verification Report

**Phase Goal:** One validated Quran edition is importable, addressable, and provably immutable.
**Verified:** 2026-10-04T15:31:19Z
**Status:** human_needed
**Re-verification:** Yes — after gap closure (plan 02-08 closed UAT gap G-02-3)

## Goal Achievement

### Observable Truths

| #   | Truth   | Status     | Evidence       |
| --- | ------- | ---------- | -------------- |
| 1 | SC-1: Operator can import a Quran edition through staging → validation → atomic activation with rollback | ✓ VERIFIED | Regression re-run: `cargo test -p application --test quran_import` 16 passed (incl. rollback-rejection branches); `cargo test -p storage-sqlite --test quran` 11 passed; `cargo run -q -p xtask -- migrate-check` → OK, 22 ordered, checksums stable |
| 2 | SC-2: User can look up any surah:ayah and receive byte-exact canonical Arabic with pinned edition reference | ✓ VERIFIED | Regression re-run: `cargo test -p application --test quran_reader` 13 passed; `--test quran_identity` 7 passed; pinned `quran:{slug}@{version}:{surah}:{ayah}` + STORED text_hash print path intact |
| 3 | SC-3: Corpus integrity checks (counts, addressing, Unicode, checksums, round-trip, reference comparison) pass | ✓ VERIFIED | Regression re-run: `cargo test -p application --test quran_doctor` 4 passed; `cargo test -p quran-corpus --test fixtures` 13 passed; six-family `qai quran verify` surface unchanged by gap closure |
| 4 | SC-4: Canonical tables reject all non-approved writes; importer has no code path to canonical tables | ✓ VERIFIED | `migrations/sqlite/0021_canonical_write_fence.up.sql` carries 16 `RAISE(ABORT…` handlers (re-counted); `cargo test -p quran-corpus --test import_path` 4 passed (importer audit); `cargo run -q -p xtask -- arch-check` → OK, no forbidden edges |
| 5 | SC-5: Every quotation verifies via `verify_quotation` with mismatch as a hard failure | ✓ VERIFIED | `citations::require_exact` called at `crates/application/src/quran_tools.rs:639` and `crates/server/src/api.rs:606` (citation_handler); `cargo test -p citations --lib` 13 passed; `--test quran_tools` 9 passed; `cargo test -p server --test api` 22 passed (G-02-3 touched no enforcement path) |
| 6 | G-02-3: Every answer path that emits quoted canonical text is enforced or recorded structurally exempt (or explicitly debug-only) with a per-path basis | ✓ VERIFIED | Corrected emission-frame ledger present in all 3 evidence files (`docs/06-progress/phase-02-evidence.md` §3, `docs/05-followups/phase-02-owner-gates.md` §D-15, `02-05-SUMMARY.md` §Structural Exemptions): 2 enforced, 10 `AyahView.canonical`, 1 hand-built token surface (`tokens_handler`), 7 `SearchHit.quotation`, `resolve_handler` no-text, `debug_reader_handler` debug-only, plus a not-emitters list. All nine previously-missing paths (5 search handlers, surah/divisions, cmd_search, quran.search) now recorded |
| 7 | Every ledger reference is a drift-proof `path#symbol` reference that resolves at HEAD | ✓ VERIFIED | Checked all 13 api.rs symbols (`surah_handler`, `divisions_handler`, `search_exact_handler`, `search_normalized_handler`, `search_phrase_handler`, `search_concatenated_handler`, `search_regex_handler`, `ayahs_handler`, `context_handler`, `tokens_handler`, `debug_reader_handler`, `resolve_handler`, `citation_handler`), all 6 CLI symbols (`cmd_get`, `cmd_context`, `cmd_surah`, `cmd_division`, `cmd_search`, `cmd_verify_quotation`), and `quran_tools.rs` (`verify_canonical_quotation` async fn; `ReaderToolBackend`, `ReaderCitationSource` structs) — all resolve at HEAD. Emission counts in `api.rs`: `.arabic_text()`=3, `search_response(&headers,`=5, `ok_envelope(result.results`=2 |
| 8 | A mechanical guard fails if a new emitting path is added without being recorded in the checked list | ⚠️ UNCERTAIN | `crates/application/tests/answer_path_ledger.rs` exists and `cargo test -p application --test answer_path_ledger` → 6 passed (incl. self-tests `scanner_reports_an_unlisted_emitting_handler`, `kind_aware_resolution_rejects_a_mismatched_definition`). It fails closed for a new api.rs marker-emitting handler and resolves all 20 `LEDGER` symbols kind-aware. BUT it scans only `api.rs` (WR-02): `.arabic_text()` occurs 4× in `quran_cli.rs` and 1× in `quran_tools.rs` unscanned, so a new CLI/tools emitting handler does **not** fail the build — contradicting the module doc at `answer_path_ledger.rs:6-8`. And `LEDGER` omits `citation_handler` (the enforced HTTP path) and `resolve_handler` (WR-01), so their rename/removal is undetected. Scope/claim decision surfaced to human — see Human Verification |
| 9 | The false blanket completeness claim is corrected in 02-05-SUMMARY D4, evidence §3, owner-gates §D-15, and the in-code `ReaderCitationSource` doc | ✓ VERIFIED | §3 intro now frames on the emission frame + points at the guard; `02-05-SUMMARY.md` D4 (line 104) rewritten; `ReaderCitationSource` doc (`quran_tools.rs:456-488`) carries the complete sweep; no file still claims the narrowed list is "every answer path" |

**Score:** 8/9 truths verified (0 present, behavior-unverified; 1 uncertain → human verification)

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | -------- | ------ | ------- |
| `crates/application/tests/answer_path_ledger.rs` | checked-list source-scan guard (new) | ✓ VERIFIED | 155 lines; 6 tests pass; kind-aware symbol resolution + 3-marker scan + 2 fail-closed self-tests; ceiling documented (though narrower than the doc claim — WR-02) |
| `docs/06-progress/phase-02-evidence.md` §3 | complete emission-frame ledger | ✓ VERIFIED | 2 enforced + 10 `AyahView.canonical` + 1 token-surface + 7 `SearchHit.quotation` + no-text + debug-only + not-emitters; all `path#symbol` |
| `docs/05-followups/phase-02-owner-gates.md` §D-15 | mirrored complete ledger | ✓ VERIFIED | Same sweep as evidence §3; owner-ratifiable interpretation preserved |
| `.planning/phases/02-canonical-quran-core/02-05-SUMMARY.md` | corrected D4 + §Structural Exemptions | ✓ VERIFIED | D4 rewritten to the emission frame (line 104); table complete (lines 170-222) |
| `crates/application/src/quran_tools.rs` `ReaderCitationSource` doc | complete sweep + guard pointer | ✓ VERIFIED | `quran_tools.rs:456-488`; doc-comment-only change (verified: non-comment diff lines = 0) |
| `.planning/phases/02-canonical-quran-core/02-RESEARCH.md` §F-6 | search rows + frame-correction note | ✓ VERIFIED | Search-path row (line 499) + `quran.search` row (line 500) + `**Frame correction (G-02-3)**` note (line 503) |
| `.planning/phases/02-canonical-quran-core/02-05-PLAN.md` Task 3 | G-02-3 correction note | ✓ VERIFIED | `<!-- Correction (G-02-3): ...` at line 163; historical action/AC unchanged |

### Key Link Verification

| From | To  | Via | Status | Details |
| ---- | --- | --- | ------ | ------- |
| guard `LEDGER` | 3 evidence-of-record ledger tables | identical emitting-path set | ⚠️ PARTIAL | All 18 emitting paths in the tables are in `LEDGER`; `LEDGER` additionally omits 2 non-emitter names that the tables list (`citation_handler`, `resolve_handler`) — WR-01. Drift protection for those 2 paths is not wired |
| `ReaderCitationSource` doc | `docs/06-progress/phase-02-evidence.md` §3 | identical sweep | ✓ WIRED | Both carry the same 2 enforced / 10 + 1 + 7 exempt / no-text / debug-only taxonomy |
| `answer_path_ledger.rs` self-test | scanner fail-closed behaviour | synthetic unlisted handler | ✓ WIRED | `scanner_reports_an_unlisted_emitting_handler` + `kind_aware_resolution_rejects_a_mismatched_definition` pass |
| G-02-3 enforcement paths (unchanged) | `verify_canonical_quotation` / `citation_handler` | `citations::require_exact` | ✓ WIRED | quran_tools.rs:639, api.rs:606; no enforcement behaviour changed by 02-08 |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| `answer_path_ledger.rs` `scan_marker` | enclosing `async fn` name | `include_str!` of api.rs / quran_cli.rs / quran_tools.rs | Yes (real source text) | ✓ FLOWING (api.rs only — WR-02) |
| `every_ledger_symbol_resolves_under_its_declared_kind` | symbol resolution | `include_str!` sources | Yes | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| ledger guard (new) | `cargo test -p application --test answer_path_ledger` | 6 passed | ✓ PASS |
| import/activation/rollback | `cargo test -p application --test quran_import` | 16 passed | ✓ PASS |
| byte-exact reader + identity | `cargo test -p application --test quran_reader --test quran_identity` | 13 + 7 passed | ✓ PASS |
| quotation tools + verdict hardening | `cargo test -p citations --lib`; `--test quran_tools` | 13 + 9 passed | ✓ PASS |
| translation layer / reference / doctor | `--test quran_translation --test quran_reference --test quran_doctor` | 13 + 1 + 4 passed | ✓ PASS |
| write fence + importer audit | `cargo test -p storage-sqlite --test quran`; `-p quran-corpus --test import_path` | 11 + 4 passed | ✓ PASS |
| HTTP enforcement | `cargo test -p server --test api` | 22 passed | ✓ PASS |
| integrity fixtures | `cargo test -p quran-corpus --test fixtures` | 13 passed | ✓ PASS |
| migrations ordered | `cargo run -q -p xtask -- migrate-check` | OK — 22 ordered, checksums stable | ✓ PASS |
| purity fence | `cargo run -q -p xtask -- arch-check` | OK — no forbidden edges | ✓ PASS |

### Probe Execution

No `scripts/*/tests/probe-*.sh` probes declared by this phase's plans. Skipped (not applicable).

### Requirements Coverage

| Requirement | Source Plans | Description | Status | Evidence |
| ----------- | ------------ | ----------- | ------ | -------- |
| REQ-quran-corpus | 02-01, 02-02, 02-03, 02-04, 02-05, 02-07, 02-08 | Validated canonical representation: hierarchy, edition model, immutable text, stable addressing | ✓ SATISFIED | SC-1…SC-5 verified; gap-closure guard green; `REQUIREMENTS.md:20` marked [x]/Complete |
| REQ-data-separation-layers | 02-01, 02-06, 02-07 | Trust layers A–E; translation as distinct non-canonical layer | ✓ SATISFIED | `quran_translation` 13 passed; layer-separation negative test; `REQUIREMENTS.md:19` marked [x]/Complete |
| REQ-ingestion-validation-eval | 02-02, 02-03, 02-04, 02-06, 02-07, 02-08 | Discover→stage pipeline; validation; evaluation | ✓ SATISFIED | six-family verify surface; QV-015 evaluation path; integrity artifact; `REQUIREMENTS.md:21` marked [x]/Complete |

No orphaned requirements: all three Phase-2 `REQUIREMENTS.md` rows (lines 19-21) are claimed across the eight plans, and none is mapped to this phase in `REQUIREMENTS.md` without a plan claim.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| `crates/application/tests/answer_path_ledger.rs` | 37-58 | `LEDGER` omits `citation_handler` (enforced HTTP) and `resolve_handler`, both named in the ledger docs it claims to guard | ⚠️ WARNING | Rename/removal of either won't fail the build; the "checked list ↔ ledger tables identical" claim (02-08 key link) is not fully wired (WR-01) |
| `crates/application/tests/answer_path_ledger.rs` | 112-127 | Markers scanned only in `api.rs`; module doc (`:6-8`) claims any new marker-emitting handler fails the build, false for `quran_cli.rs` (4 `.arabic_text()`) and `quran_tools.rs` (1) | ⚠️ WARNING | New CLI/tools emitting handler can bypass the guard, giving false assurance (WR-02) |
| — | — | `TBD/FIXME/XXX` in phase-touched impl files | — | None. The one `TBD` (`docs/05-followups/phase-02-owner-gates.md:12`) is descriptive owner-gate prose ("stays `_unassigned_` / `TBD` until a human"), not a debt marker on implementation |

### Known Pre-Existing Issues (NOT phase gaps)

1. **WR-03 — `search_meta(None)` edition mis-attribution** (`quran_tools.rs:129-141`, reached from `backend_search`): the envelope names the active edition while `quran.search` serves the indexed edition. Confirmed pre-existing phase-03 code; the declared review base `d3a0a99` predates phase 03 and the 02-08 delta does not touch it. Recorded as `advisory`.
2. **Legacy CLI snapshot baseline** — `cargo test -p cli --test quran` reports known `quran_search_snapshots`/`quran_normalize_snapshots` failures (non-deterministic index manifest hash), attributed to roadmap Phase 3 in `phase-02-evidence.md` §4. Not a Phase 2 gap.

### Human Verification Required

#### 1. Answer-path ledger guard coverage scope (WR-01/WR-02)

**Test:** Decide whether the guard at `crates/application/tests/answer_path_ledger.rs` must cover all three source files and all swept paths, then reconcile the code with its claim.
**Expected:** Either (a) extend the marker scan to `quran_cli.rs`/`quran_tools.rs` with per-file expected counts and add `citation_handler`/`resolve_handler` to `LEDGER`; or (b) narrow the module doc (`answer_path_ledger.rs:6-8`) and the `ReaderCitationSource` doc (`quran_tools.rs:486-488`) plus evidence §3's "guarded mechanically by …" wording to state the guard covers the `api.rs` emission frame + CLI `cmd_search`.
**Why human:** The guard is green and fails closed for the api.rs frame where G-02-3 actually occurred, but its documented "any new emitting handler fails the build" claim overstates the wired scan, and `LEDGER` omits two swept paths. Choosing extend-vs-narrow is a scope/ratification decision.

### Gaps Summary

No FAILED must-have truths, no MISSING/STUB artifacts, no NOT_WIRED links, and no unresolved debt markers. UAT gap **G-02-3 is closed**: the D-15 answer-path exemption ledger was rebuilt on the emission frame in all three evidence-of-record files, the in-code doc, and the research enumeration source; every `path#symbol` reference resolves at HEAD; the false blanket completeness claim is corrected in all four copies; and a checked-list source-scan guard now fails closed for the `api.rs` emission frame (6 tests green).

The single open item is the guard's **coverage scope** (WR-01/WR-02): its module doc claims detection for any new marker-emitting handler, but the scan is `api.rs`-only and `LEDGER` omits two swept paths. This does not block the phase goal (the five roadmap success criteria all hold on the synthetic-test path) and is not a regression introduced by the gap closure; it is an artifact-vs-claim scope decision routed to human verification. All five roadmap success criteria and all three mapped requirements are otherwise satisfied.

---

_Verified: 2026-10-04T15:31:19Z_
_Verifier: the agent (gsd-verifier)_
