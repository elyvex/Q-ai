---
phase: 03-quran-search-linguistics
plan: 01
subsystem: testing
tags: [rust, sqlite, fts5, quran-search, normalization, canonical-identity, reader-service, tracer]

# Dependency graph
requires:
  - phase: 02-canonical-quran-core
    provides: canonical edition import/activation, forms rebuild (MV-018), index rebuild, reader service, exact/normalized/phrase/concatenated/regex search services, validating SearchHit constructor
provides:
  - "alpha_smoke.rs — the single Phase-3 tracer path proven end-to-end on the synthetic fixture (migrate → import → activate → forms → index → normalized search → hit with trace + canonical span + verified quotation)"
  - "canonical_display_identity.rs — SC5 pinned per hit across all five search modes: displayed canonical text is byte-identical to the reader-resolved canonical row at every hit's canonical span, and canonical hashes are stable across searches"
  - "Corrected normalization rule-catalog module doc (G-12): heuristic_rules() implements N18–N22; by_id() returns None only for N23/N24"
  - "Recorded SC1 repeatable-check evidence (400-row golden suite + 5,000-substring parity gate)"
affects: [03-02, 03-03, 03-04, 03-05, 03-06, 03-07, 03-08, phase-04-graph, phase-05-result-contract]

# Actuals (#2632) — same estimateTokens scale as the plan's estimate (chars/4 over the realized diff)
actuals:
  tokens: 8576
  tasks: 3
  commits: 3
plan_head_before: c2d6b6853e41951a0fa2ba138b29bd9f36c8a36c

tech-stack:
  added: []
  patterns:
    - "Per-hit canonical byte-identity pin: resolve the hit's ayah through the reader service and compare the displayed quotation bytes to the canonical row bytes at the hit's canonical span"
    - "Vacuity guard: every search-mode assertion first requires >= 1 hit, so a silently-empty result can never pass"
    - "Dispose a fallback edge at the surface the operator actually calls (CLI guard), not at the unguarded service"

key-files:
  created:
    - crates/application/tests/alpha_smoke.rs
    - crates/application/tests/canonical_display_identity.rs
  modified:
    - crates/quran-normalization/src/rules/mod.rs

key-decisions:
  - "SC5 is pinned per hit, not only at storage level: the displayed quotation is compared byte-for-byte to the reader-resolved canonical row, and the canonical stored hash is asserted unchanged across all five searches (I8/SC5)."
  - "The empty/whitespace query edge is disposed at the CLI boundary: cmd_search returns exit::USAGE with 'provide query text' before opening the service; search_normalized has no empty guard and legitimately returns Ok with zero hits."

patterns-established:
  - "SC5 per-hit pin: reader-resolved canonical bytes == SearchHit.quotation bytes at canonical_span, for every mode"
  - "Alpha smoke tracer: one pasted command proves the whole Phase-3 search path end-to-end"

requirements-completed: [REQ-quran-normalization]

# Coverage metadata (#1602)
coverage:
  - id: D1
    description: "alpha_smoke.rs: end-to-end tracer (migrate → import → activate → forms rebuild → index rebuild → normalized L3 search) returns a hit with a non-empty ordered rule trace, a valid canonical span, and a quotation byte-identical to the canonical row; empty/whitespace queries are rejected by the real cmd_search CLI guard with a typed usage error."
    requirement: REQ-quran-normalization
    verification:
      - kind: integration
        ref: "cargo test -p application --test alpha_smoke -- alpha_smoke_normalized_search_end_to_end"
        status: pass
      - kind: integration
        ref: "cargo test -p application --test alpha_smoke -- empty_and_whitespace_queries_are_rejected_at_the_cli_boundary"
        status: pass
    human_judgment: false
  - id: D2
    description: "canonical_display_identity.rs: SC5 per-hit byte-identity across exact/normalized/phrase/concatenated/regex, concatenated segmentation tiling the canonical text, and unchanged canonical hashes across all five searches."
    requirement: REQ-quran-normalization
    verification:
      - kind: integration
        ref: "cargo test -p application --test canonical_display_identity"
        status: pass
    human_judgment: false
  - id: D3
    description: "Normalization rule-catalog module doc corrected (N18–N22 implemented, N23/N24 reserved) with no behavioural change, and the SC1 repeatable check (400-row golden suite + 5,000-substring parity gate) recorded green."
    requirement: REQ-quran-normalization
    verification:
      - kind: integration
        ref: "cargo test -p application --test search_goldens"
        status: pass
      - kind: integration
        ref: "cargo test -p quran-search --test search_parity"
        status: pass
      - kind: other
        ref: "cargo clippy -p quran-normalization -- -D warnings && cargo fmt -p quran-normalization -- --check"
        status: pass
      - kind: integration
        ref: "cargo test -p cli --test quran"
        status: fail
    human_judgment: true
    rationale: "The CLI snapshot tree is red for a pre-existing, unrelated reason: the concurrent/committed 01-04 enqueue-only import refactor (c2d6b68) changed the import CLI wording but did not refresh crates/cli/tests/quran/*.trycmd. The 03-01 change is doc-only and provably non-causal (reverting it and re-running quran_search_snapshots reproduces the identical failure). A human must decide whether to refresh the snapshots here or in the owning 01-04 plan."

# Metrics
duration: 19 min
completed: 2026-09-28
status: complete
---

# Phase 3 Plan 01: Quran Search & Linguistics — Alpha Smoke + SC5 Canonical-Display Pin Summary

**One pasted command (`cargo test -p application --test alpha_smoke`) now proves the whole Phase-3 search path end-to-end on the synthetic fixture, and a new per-hit test pins SC5 — displayed canonical text is byte-identical to the stored canonical row at every hit's span across all five search modes.**

## Performance

- **Duration:** 19 min
- **Started:** 2026-09-28T00:20:00Z (approx.)
- **Completed:** 2026-09-28T00:39:00Z (approx.)
- **Tasks:** 3/3
- **Files modified:** 3 (2 created, 1 doc-edited)

## Accomplishments

- **End-to-end tracer (SC1/I9/I10/D-11):** `alpha_smoke.rs` drives migrate → import → activate → forms rebuild → index rebuild → normalized `L3.diacritics` search of a diacritic-free fixture token, and asserts every hit carries a non-empty ordered rule trace, a valid canonical span, and a quotation byte-identical to the independently reader-resolved canonical row.
- **Empty-input edge disposed at the real surface:** the same binary asserts `cmd_search` returns a typed usage error (`exit::USAGE`, `provide query text`) for `""`, `" "`, `"   "`, and `"\t\n"` — never a panic and never an all-match. This is the guard at `crates/application/src/quran_cli.rs:3406`; the service (`search_normalized`) legitimately returns `Ok` with zero hits and is not asserted to error.
- **SC5 pinned per hit (Q5 / assumptions A1/A2 closed):** `canonical_display_identity.rs` exercises exact, normalized, phrase, concatenated, and regex modes; for every hit it resolves the canonical ayah through the reader service and asserts byte-identity of the displayed quotation, byte-identity of the span slice in displayed vs canonical text, and equality of the quoted hash with the canonical row hash. Concatenated hits must carry a non-empty segmentation whose `canonical_surface` values occur inside the canonical ayah text. A final test asserts the canonical stored hashes are unchanged across all five searches (I8).
- **Rule-catalog doc corrected (G-12):** the stale "N18–N24 … are not implemented here" claim is replaced with the accurate statement that `heuristic_rules()` implements N18–N22 and `by_id()` returns `None` only for the reserved N23/N24. Doc-only; no behavioural function changed.
- **SC1 evidence recorded:** 400/400 synthetic golden rows pass (`cargo test -p application --test search_goldens` → 2 passed) and the 5,000-substring tokenizer-parity gate passes (`cargo test -p quran-search --test search_parity` → 3 passed).

## Task Commits

Each task was committed atomically:

1. **Task 1: End-to-end "diacritic-free query → ranked canonical hit with explainability"** - `43c21fb` (test)
2. **Task 2: Pin SC5 — displayed canonical slice is byte-identical for every search mode** - `a2856d5` (test)
3. **Task 3: Correct the normalization rule doc (G-12) and record SC1 evidence** - `0034fe0` (docs)

**Plan metadata:** (this commit) (docs: complete plan)

_Note: Task 2 is a test-only `tdd="true"` task. The behaviour it asserts is pre-existing/structural (`SearchHit.quotation` is a `QuranQuotation` built from the canonical row), so GREEN was immediate — the discipline applied here is the anti-vacuity guard: each mode asserts `>= 1` hit before any identity assertion, so a silently-empty result cannot pass._

## Files Created/Modified

- `crates/application/tests/alpha_smoke.rs` — end-to-end tracer + CLI empty-input guard assertions.
- `crates/application/tests/canonical_display_identity.rs` — SC5 per-hit byte-identity across all five modes + canonical-hash stability.
- `crates/quran-normalization/src/rules/mod.rs` — module doc corrected (doc-only).
- `.planning/phases/03-quran-search-linguistics/deferred-items.md` — out-of-scope pre-existing CLI snapshot failure logged.

## Decisions Made

- **SC5 is a per-hit contract, not just a storage check.** The test independently resolves canonical through the reader and compares bytes, so a future change that re-rendered displayed text from a normalized form would fail loudly.
- **Dispose the `empty` fallback edge at the CLI boundary.** The plan's must_have flagged this as an assumption; it is now asserted at the real operator surface (`cmd_search`) rather than the unguarded service.
- **Offset/equality semantics (flagged assumptions):** canonical spans are char ranges mapping to byte ranges in canonical NFC text (ADR-0104); equality of normalized forms is code-point equality, not grapheme count. Both hold in the new tests.
- **REQ-quran-linguistics not touched by this plan** (plan 03-01 declares only `REQ-quran-normalization`); OD-12 (normalization catalog + named linguist) remains BLOCKED.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] SC5 assertion wording was inconsistent with the hit contract**
- **Found during:** Task 2 (`canonical_display_identity.rs`)
- **Issue:** The plan states "assert byte equality between the sliced canonical text and `hit.quotation`". In the code, `SearchHit.quotation().arabic_text()` is the **full canonical ayah** (`assemble_hit` passes `ayah_row.text`), while `canonical_span` indexes *into* it — so a literal comparison of the span slice to the whole quotation would always fail.
- **Fix:** Implemented the semantically-correct SC5 pin that satisfies the plan's must_have ("displayed canonical slice is byte-identical to the canonical row at the hit's canonical span"): (a) displayed quotation bytes == reader-resolved canonical row bytes (full row), and (b) the span slice of the displayed text is byte-identical to the span slice of the canonical row. Both readings of the criterion are covered.
- **Files modified:** `crates/application/tests/canonical_display_identity.rs`
- **Verification:** `cargo test -p application --test canonical_display_identity` → 6 passed.
- **Committed in:** `a2856d5` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 — plan wording reconciled to the real code contract; no behavioural change to production code).
**Impact on plan:** None beyond the assertion shape. No scope creep: no production symbol was added or changed by this plan.

## Issues Encountered

- **Reader requires a UUID-shaped edition id.** The plan said to build the harness "verbatim from `search_goldens.rs`", which uses a slug-like `run_id` (`"run-goldens-1"`). Because this plan resolves canonical through the reader service, the import `run_id` must be UUID-shaped (the edition id == run id, and the reader's typed id mapping rejects a non-UUID). Changed `RUN_ID` to `aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee` in both new tests (mirrors `tests/common/mod.rs`). No production change.
- **Concurrent work in the shared working tree.** While this plan ran, another plan committed `c2d6b68 feat(01-04): make quran import enqueue-only with queued job result` on `main`, changing `crates/application/src/quran.rs`, `quran_cli.rs`, `cli/src/quran.rs`, and `cli/tests/quran.rs`. Those files were never staged by this plan; only the three declared `files_modified` were committed.

## Deferred Issues

- **`cargo test -p cli --test quran` is red (pre-existing, out of scope).** 9/13 snapshot tests fail because `c2d6b68` (plan 01-04) changed the import CLI wording to the queued/host contract without refreshing `crates/cli/tests/quran/*.trycmd`. Verified non-causal for this plan by reverting the doc-only change and reproducing the identical failure. Logged in `deferred-items.md`; not fixed here (outside `files_modified`). This is why coverage item D3 routes to human review.

## User Setup Required

None — no external service configuration required.

## Known Stubs

None — this plan adds test evidence and a doc correction; no stub, placeholder, or hardcoded-empty value flows to any UI or result.

## Threat Flags

None — no new network endpoint, auth path, file-access pattern, or trust-boundary schema change. The tests bind query text through the existing normalized/FTS path and assert the empty-input rejection (T-03-01), per-hit canonical byte identity (T-03-02), trace-mandatory hits (T-03-05), and clamped `limit` (T-03-04).

## Owner Gates (BLOCKED — not silently passed)

- **OD-12 (normalization catalog + named linguist, ADR-0204/0205)** — BLOCKED. The golden suite run here is synthetic (`reviewed_by: pending-linguist`). Closing step: name a qualified Arabic linguist and record sign-off in `docs/reviews/`, then flip ADR-0204/0205 to Accepted.
- **OD-11 (morphology dataset/license)** — BLOCKED (not touched by this plan; recorded for phase coherence).

## Next Phase Readiness

- Phase-5 consumers now have a pinned contract: for every hit and every mode, the displayed canonical text is the stored canonical bytes at the hit's canonical span.
- SC1 has a named repeatable check (golden suite + parity gate) that is green; SC5 has a repeatable per-hit assertion.
- Open concern for the phase: the CLI snapshot tree (`cargo test -p cli --test quran`) is red due to the parallel 01-04 enqueue-only import refactor leaving trycmd snapshots stale — the owning plan must refresh them before the phase gate.
