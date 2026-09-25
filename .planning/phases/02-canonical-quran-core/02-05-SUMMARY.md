---
phase: 02-canonical-quran-core
plan: 05
subsystem: citations
tags: [quran, citations, quotation, verdict, hard-failure, cli, http, trycmd, adr-0111, d-15]

requires:
  - phase: 02-canonical-quran-core
    provides: plan 02-03's reader-backed operator integrity surfaces and the trycmd CLI harness this plan's snapshot extends
  - phase: 01-foundations
    provides: citations::CitationResolver + QuotationVerdict, the generation-keyed reader, the application composition root, and the HTTP citation handler

provides:
  - QuotationVerdict::is_hard_failure + label and the shared citations::require_exact mapping with typed codes QAI-QUR-0323/0324/0325/0326
  - qai quran verify-quotation — a read-only CLI verb (QuranAction::VerifyQuotation + cmd_verify_quotation) whose mismatch exit is 3 with a typed code
  - verify_canonical_quotation — the single shared application verifier returning the verdict plus the resolved canonical text hash
  - HTTP enforcement of a stored hard-failure verdict in citation_handler (no 200 envelope for a failed verdict)
  - crates/cli/tests/quran/verify_quotation.trycmd + a server hard-failure test
  - a recorded structurally-exempt direct-read answer-path list with per-path basis

affects: [02-07 phase evidence of record / answer-path audit, phase 9 citation verifier reuse, phase 5 citation UI]

actuals:
  tokens: 7063     # chars/4 over the realized diff (28253 diff chars across crates/)
  tasks: 3
  commits: 3       # MEASURED: git rev-list --count 7da6d70..HEAD
  plan_head_before: 7da6d7090baa4b6485cfad5d417e7c39a34d7d9f

tech-stack:
  added: []
  patterns:
    - "One shared verdict mapping: a domain helper (require_exact) turns a hard-failure verdict into a typed coded error; every operator surface delegates to it so a mismatch cannot be softened on one path (D-15, ADR-0111)."
    - "Resolved-hash provenance: a verifying surface returns the canonical hash read from the reader, never recomputed from the supplied text, so a caller cannot present a synthesized hash (T-02-19)."
    - "Structural exemption over wrapping: direct-read paths that serve the source of truth are documented as exempt with a per-path basis instead of being wrapped in a comparison that proves nothing (Pitfall 4)."

key-files:
  created:
    - crates/cli/tests/quran/verify_quotation.trycmd
  modified:
    - crates/citations/src/lib.rs
    - crates/application/src/quran_tools.rs
    - crates/application/src/quran_cli.rs
    - crates/application/tests/quran_tools.rs
    - crates/cli/src/quran.rs
    - crates/cli/tests/quran.rs
    - crates/server/src/api.rs
    - crates/server/tests/api.rs

key-decisions:
  - "The shared verdict-to-hard-failure mapping lives in the citations domain crate (QuotationVerdict::is_hard_failure + citations::require_exact); exit-code and HTTP-status concerns stay in the callers so citations stays free of CLI concerns (D-15, ADR-0010)."
  - "verify_canonical_quotation calls the resolver's resolve path directly because it returns the resolved canonical hash in a single fetch; verify_quotation delegates to the same resolve, so the verdict is identical and the hash is never computed from the supplied text (T-02-19)."
  - "The HTTP citation handler is the single enforcement point for stored verdicts, so any backend's hard-failing verdict returns a typed error instead of a 200 envelope; no new route, stored-citation field, or error-code map was added (T-02-18)."
  - "The direct-read answer paths (tool quran.get_ayah/get_context, CLI direct reads, HTTP direct reads) are structurally exempt from verify_quotation because they serve canonical text and cannot mismatch by construction; wrapping them would compare canonical text to itself (Pitfall 4)."
  - "MatchAfterDeclaredNormalization is unreachable in v1 (the resolver has no normalization-rules parameter); the tests assert the reachable verdict set excludes it rather than claiming behaviour for it."

patterns-established:
  - "Coded hard-failure mapping: each hard-failure verdict maps to a unique, stable QAI-QUR-* code; existing codes are never reused or renumbered."
  - "Single verifying implementation: the CLI verb and the HTTP path both go through verify_canonical_quotation / citations::require_exact, so the answer-path decision cannot diverge."
  - "Exempt-by-construction audit: a read path that serves the canonical source of truth is recorded as exempt with its file:line basis instead of being needlessly wrapped."

requirements-completed: [REQ-quran-corpus]

coverage:
  - id: D1
    description: "The shared verdict-to-hard-failure mapping exists on the citations domain crate: exact/whitespace matches pass, and Mismatch/LocationNotFound/EditionNotFound/AccessDenied map to distinct typed QAI-QUR-* codes; MatchAfterDeclaredNormalization is not in the reachable set."
    requirement: "REQ-quran-corpus"
    verification:
      - kind: unit
        ref: "crates/citations/src/lib.rs#require_exact_maps_every_hard_failure_to_a_distinct_code"
        status: pass
      - kind: unit
        ref: "crates/citations/src/lib.rs#tampered_text_against_a_real_citation_is_a_hard_failure"
        status: pass
      - kind: unit
        ref: "crates/citations/src/lib.rs#reachable_verdict_set_excludes_declared_normalization"
        status: pass
    human_judgment: false
  - id: D2
    description: "qai quran verify-quotation is read-only, takes an explicit pinned slug@version, prints the verdict and the RESOLVED canonical hash, exits 0 on an exact match and 3 with a typed code on a mismatch (5 for a location/edition not found)."
    requirement: "REQ-quran-corpus"
    verification:
      - kind: integration
        ref: "crates/application/tests/quran_tools.rs#verify_canonical_quotation_succeeds_on_exact_and_hard_fails_on_mismatch"
        status: pass
      - kind: e2e
        ref: "crates/cli/tests/quran/verify_quotation.trycmd (exit 0 exact / ? 3 mismatch / ? 5 unknown edition)"
        status: pass
    human_judgment: false
  - id: D3
    description: "The HTTP citation answer path refuses a 200 success envelope for a stored hard-failing verdict (typed error, no data envelope) while an exact-match stored citation still returns its resolved citation unchanged; no new route is introduced."
    requirement: "REQ-quran-corpus"
    verification:
      - kind: integration
        ref: "crates/server/tests/api.rs#stored_citation_hard_failure_is_not_a_success_envelope"
        status: pass
      - kind: integration
        ref: "crates/server/tests/api.rs#listings_divisions_tokens_resolve_citations (exact-match citation still 200)"
        status: pass
      - kind: other
        ref: "cargo run -q -p xtask -- arch-check (OK); no route line added under /api/v1/quran/citations"
        status: pass
    human_judgment: false
  - id: D4
    description: "Every answer path that emits quoted canonical text is either enforced or recorded as structurally exempt with a per-path file:line basis (tool get_ayah/get_context; CLI cmd_get/context/surah/division; HTTP direct reads)."
    requirement: "REQ-quran-corpus"
    verification:
      - kind: other
        ref: "crates/application/src/quran_tools.rs#ReaderCitationSource doc (exemption list) + the Structural Exemptions table in this Summary"
        status: pass
    human_judgment: true
    rationale: "Whether the exemption set is complete and correctly reasoned is an architectural audit judgment no test asserts; plan 02-07 consumes this evidence of record."
  - id: D5
    description: "No frozen verdict vocabulary, quotation type, endpoint, or crate-boundary change: the work is additive helpers, one read-only verb, and one handler enforcement."
    requirement: "REQ-quran-corpus"
    verification:
      - kind: other
        ref: "cargo run -q -p xtask -- arch-check (OK — no forbidden dependency edges)"
        status: pass
      - kind: other
        ref: "QuotationVerdict variants unchanged; no new stored-citation field; no new HTTP route"
        status: pass
    human_judgment: false

duration: 11 min
completed: 2026-09-25
status: complete
---

# Phase 02 Plan 05: Quotation hard-failure wiring Summary

**`verify_quotation` now has production call sites: a shared `citations::require_exact` mapping turns every hard-failure verdict into a typed code, `qai quran verify-quotation` exercises it with a non-zero mismatch exit, and the HTTP citation path refuses a 200 envelope for a stored failed verdict.**

## Performance

- **Duration:** 11 min
- **Started:** 2026-09-25T16:06:39Z
- **Completed:** 2026-09-25T16:17:11Z
- **Tasks:** 3
- **Files modified:** 9 (1 created, 8 modified)

## Accomplishments

- Closed QC-07 / T-02-17: the shared mapping from quotation verdict to hard failure now exists in the `citations` domain crate — `QuotationVerdict::is_hard_failure` plus the free function `citations::require_exact`, which maps `Mismatch` → `QAI-QUR-0323`, `LocationNotFound` → `0324`, `EditionNotFound` → `0325`, and `AccessDenied` → `0326`, while the reachable success verdicts pass. `MatchAfterDeclaredNormalization` is documented as unreachable in v1 (no normalization-rules parameter) and a test asserts the resolver's reachable verdict set rather than claiming behaviour for it.
- Gave `verify_quotation` a real production operator surface (criterion 5): the read-only `qai quran verify-quotation --edition slug@version --surah N --ayah N --text …` verb builds the citation from the pinned edition string, verifies the externally supplied text through the reader-backed resolver, and prints the verdict together with the RESOLVED canonical hash. A tampered quotation exits 3 with the typed code; an exact match exits 0; an unknown pinned edition exits 5. Proven by a trycmd snapshot and an application test.
- Enforced the same mapping on the existing HTTP answer path (T-02-18): `citation_handler` (`GET /api/v1/quran/citations/{id}`) now applies `citations::require_exact` to the resolved stored verdict, so a hard-failing stored verdict returns a typed error response instead of a 200 envelope that re-serves an untrusted verdict; an exact-match citation is returned unchanged. No new route, endpoint, or stored-citation field was added.
- Recorded, rather than wrapped, the structurally exempt read paths (Pitfall 4): the direct-read answer paths serve canonical text from the source of truth and cannot mismatch by construction, so `verify_quotation` there would compare canonical text against itself. The exemption list is documented on `ReaderCitationSource` and tabulated below with its file:line basis for plan 02-07's answer-path audit.

## Task Commits

Each task was committed atomically:

1. **Task 1: Shared verdict-to-hard-failure mapping in the citations crate** - `85ddb67` (feat)
2. **Task 2: `qai quran verify-quotation` production surface with an exit-code contract** - `a0d4b14` (feat)
3. **Task 3: HTTP answer-path enforcement + recorded read-path exemptions** - `1768044` (feat)

**Plan metadata:** committed with this SUMMARY (docs: complete plan)

## Files Created/Modified

- `crates/citations/src/lib.rs` - `QuotationVerdict::{is_hard_failure, label}`, `citations::require_exact`, four new coded `CitationError` variants, and three unit tests
- `crates/application/src/quran_tools.rs` - `verify_canonical_quotation` (the single shared verifier) and the `ReaderCitationSource` verification-scope/exemption doc
- `crates/application/src/quran_cli.rs` - `cmd_verify_quotation` plus `map_citation_error` exit mapping
- `crates/application/tests/quran_tools.rs` - exact/mismatch/missing-location/missing-edition verifier test
- `crates/cli/src/quran.rs` - `QuranAction::VerifyQuotation` variant and its read-only dispatch arm
- `crates/cli/tests/quran.rs` - `quran_verify_quotation_snapshots` harness fn
- `crates/cli/tests/quran/verify_quotation.trycmd` - exact (exit 0), tampered (exit 3), unknown edition (exit 5) snapshot
- `crates/server/src/api.rs` - `citation_handler` applies the shared mapping to a resolved stored verdict
- `crates/server/tests/api.rs` - FakeApi hard-failure case + `stored_citation_hard_failure_is_not_a_success_envelope`

## Structural Exemptions (answer-path audit evidence for plan 02-07)

`verify_quotation` is meaningful only for an externally supplied quotation. Every answer path that emits quoted canonical text is either enforced or recorded exempt below with its file:line basis:

| Answer path | Kind | Basis | Why exempt / enforced |
|---|---|---|---|
| `qai quran verify-quotation` | enforced | `crates/application/src/quran_tools.rs:253` (`verify_canonical_quotation`), `crates/application/src/quran_cli.rs:362` (`cmd_verify_quotation`) | Externally supplied text — the only surface that can mismatch |
| `GET /api/v1/quran/citations/{id}` | enforced | `crates/server/src/api.rs:584` (`citation_handler`), route at `crates/server/src/api.rs:1314` | Re-serves a stored verdict; `require_exact` rejects a hard failure |
| tool `quran.get_ayah` | exempt | `crates/application/src/quran_tools.rs:88-95` (`ReaderToolBackend`) → `crates/application/src/quran_reader.rs:355` | Serves canonical rows from the source of truth; cannot mismatch by construction |
| tool `quran.get_context` | exempt | `crates/application/src/quran_tools.rs:99-105` (`ReaderToolBackend`) | Same — canonical rows, no externally supplied quotation |
| CLI direct reads | exempt | `crates/application/src/quran_cli.rs:161` (`cmd_get`), `:214` (`cmd_context`), `:255` (`cmd_surah`), `:290` (`cmd_division`) | Read the canonical source of truth; no quotation is supplied to compare |
| HTTP direct reads | exempt | `crates/server/src/api.rs:451` (`context_handler`), `:524` (`tokens_handler`), `:560` (`resolve_handler`), `:1388` (`ReaderBackend`); ayahs route at `:1309` | Serve `AyahView` from the canonical reader; cannot mismatch |

## Decisions Made

- The mapping belongs in the domain crate, not in each caller: `require_exact` returns a typed `CitationError`; exit codes and HTTP statuses stay in `quran_cli`/`api`. This keeps `citations` free of CLI concerns while guaranteeing one decision across surfaces (D-15, ADR-0010).
- `verify_canonical_quotation` uses the resolver's `resolve` (which `verify_quotation` delegates to) because `resolve` also yields the resolved canonical hash in one fetch; the hash is always the reader's canonical value, never derived from the supplied text (T-02-19).
- `citation_handler` is the single HTTP enforcement point, so any `QuranApiBackend` implementation's stored verdict is checked; no new route, stored-citation field, or error-code mapping was added (T-02-18). A hard-failure stored verdict surfaces through the existing `tool_error_response` shape.
- The direct-read paths are exempt by construction, not by omission; the exemption is documented with a file:line basis rather than wrapped in a comparison that would always pass (Pitfall 4).
- `MatchAfterDeclaredNormalization` is a declared match but unreachable in v1; tests assert the reachable set excludes it instead of fabricating behaviour for it.

## Deviations from Plan

None - plan executed exactly as written, with two documented implementation choices:

1. **The shared verifier calls `resolve`, not `verify_quotation`, for the hash.** The plan said to call `CitationResolver::verify_quotation` and also return the resolved canonical hash. `verify_quotation` returns only the verdict, so the verifier uses `resolve` — the same code path `verify_quotation` delegates to — once, avoiding a second fetch. Verdict semantics are identical (the probe citation carries the supplied text), and the hash comes from the canonical row. Recorded as a key decision, not a rule deviation.
2. **The server hard-failure test supplies the verdict through the existing FakeApi.** The plan said to "insert a citation whose persisted verdict is a hard failure". `crates/server` cannot depend on `storage-sqlite` (not in its `xtask/allowlist.toml` allow set), so a real-DB integration test is impossible without an architectural change. The enforcement lives in `citation_handler`, which is backend-agnostic; the test drives a hard-failure verdict through the existing FakeApi and asserts the real handler returns a typed error rather than a 200 envelope. This tests the actual enforcement point.

## Issues Encountered

- The plan's `<output>` line names `02-04-SUMMARY.md`; this plan is `02-05` and `02-04` is already complete, so `02-05-SUMMARY.md` was written (the plan file number and phase board are authoritative). Same pre-existing typo noted by the 02-04 and 02-06 executions.
- Pre-existing CLI snapshot failures remain in `cargo test -p cli --test quran` (`quran_normalize_snapshots`, `quran_search_snapshots`) from a non-deterministic index manifest hash, confirmed pre-02-01; their snapshots were not touched (out of scope).
- `crates/server/src/api.rs` `tool_status` maps the new hard-failure codes to the default 500 (it maps only the codes it already knew). The handler still returns a typed, coded error and never a 200 success, which is the D-15/T-02-18 requirement; extending the status map was explicitly out of scope ("keep the existing error-code mapping").

## Known Stubs

None.

## Threat Flags

None — the plan adds a domain helper, one read-only CLI verb, and one handler check on the existing citation route. No new network endpoint, auth path, file-access pattern, schema change, or quotation type was introduced beyond the plan's `<threat_model>`.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Criterion 5 holds on two real production paths: `qai quran verify-quotation` (mismatch → exit 3 with `QAI-QUR-0323`) and `GET /api/v1/quran/citations/{id}` (hard-failing stored verdict → typed error, never a 200 envelope). Plan 02-07 can cite `verify_quotation.trycmd`, `quran_tools.rs`, and `api.rs` as the answer-path audit evidence.
- The frozen verdict vocabulary, `QuranQuotation`, the reader, and the HTTP route table are unchanged; `arch-check` and `clippy -D warnings` are clean on the touched crates.
- Owner gates OD-01/OD-02/OD-03 remain open and recorded, not closed. `REQ-quran-corpus` is shared with sibling plan 02-07 (no SUMMARY yet), so the shared-ID gate keeps it pending in REQUIREMENTS.md.

---

*Phase: 02-canonical-quran-core*
*Completed: 2026-09-25*

## Self-Check: PASSED

- Key files exist: `crates/citations/src/lib.rs`, `crates/application/src/quran_tools.rs`, `crates/application/src/quran_cli.rs`, `crates/cli/src/quran.rs`, `crates/cli/tests/quran/verify_quotation.trycmd`, `crates/server/src/api.rs`, `crates/server/tests/api.rs`, `.planning/phases/02-canonical-quran-core/02-05-SUMMARY.md`.
- Task commits exist: `85ddb67`, `a0d4b14`, `1768044`.
- Plan verification re-run green: `cargo test -p citations` (7 passed), `cargo test -p application --test quran_tools` (6 passed), `cargo test -p cli --test quran -- quran_verify_quotation_snapshots` (1 passed), `cargo test -p server --test api` (18 passed), `cargo run -q -p xtask -- arch-check` (OK), `cargo clippy -p citations -p application -p server -p cli --all-targets -- -D warnings` (clean).
