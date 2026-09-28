---
phase: 03-quran-search-linguistics
plan: 07
subsystem: quran-search-linguistics
tags: [rust, sqlite, doctor, g-08, g-06, g-11, d-13, performance-budget, tool-registry, attribution, tdd, od-11]

# Dependency graph
requires:
  - phase: 03-quran-search-linguistics
    provides: 03-01 the tracer + canonical identity suites and the search services; 03-06 the license gate whose `import_params`/capture fields this plan's morphology setup satisfies
  - phase: 02-canonical-quran-core
    provides: the 19 doctor index checks (quran_doctor_indexes), the index pointer/generation lifecycle, the activation-gated morphology import, and the SearchHit stale-index warning (QAI-IDX-0101)
provides:
  - "crates/application/tests/doctor_indexes.rs: an end-to-end soak (import+activate -> forms -> index -> morphology import+activate -> one search per mode) asserting 19 stable check ids with no failure, plus a drift case that produces QAI-IDX-0101"
  - "fixtures/quran/performance/budgets.json: machine-readable budget table (ADR-0207 concatenated p99<=150 ms; plan 17.1 p50/p99 rows; fixture bound 2000 ms), each row carrying applies_to + rationale"
  - "crates/application/tests/search_latency.rs: loads the artifact (include_str!), gates the fixture runs against fixture_bound_ms, asserts it is never loosened, and records the full-corpus rows as OD-11-dependent"
  - "crates/tool-registry/src/lib.rs: 7 registered tool names + QuranBackend backend_search/root/lemma/morphology/family (default typed-unsupported) + empty-selector guards"
  - "crates/application/src/quran_tools.rs: ReaderToolBackend implements the five new backends building attributed ToolResult envelopes (normalization_rules from the hit trace; analysis_sources from canonical refs / dataset)"
  - "crates/cli/src/lib.rs: serve wires ReaderToolBackend::registry_with_index_root so the D-13 tool surface is live"
affects: [03-08, phase-04-graph, phase-05-result-contract, phase-10-agent-runtime]

# Actuals (#2632) — same estimateTokens scale as the plan's estimate (chars/4 over the realized diff)
actuals:
  tokens: 17971
  tasks: 3
  commits: 4   # plan's own atomic commits: a1f907d, a0cb699, 196cf87 (RED), ca91230 (GREEN)
  # NOTE (#3968): the ledger window plan_head_before..HEAD measures 8 commits; 4 belong
  # to concurrent workstreams on the shared tree (01-04 fix 4745df7, 01 docs, 01-05 x2).
  plan_head_before: 5dddba874eb2e659d52d4b3b761fdf823c0b0117

tech-stack:
  added: []
  patterns:
    - "Doctor soak = the whole Phase-3 derived state in one test: the 19 checks are asserted for stable ids + no failure, and the only permitted non-pass is the token-index check that has no backend by design"
    - "Stale-index code (QAI-IDX-0101) is proven where it is actually emitted — the SearchHit warning after the serving generation drifts from the active edition — while doctor reports the drift as a warn and never repairs"
    - "Budgets live in one machine-readable artifact loaded with include_str! so it cannot silently disappear; the fixture bound is asserted equal to the pre-existing constant (never loosened)"
    - "Tool backends are trait methods with default typed-unsupported impls, so existing fakes (server tests) compile unchanged while the real adapter serves the full attributed surface"

key-files:
  created:
    - fixtures/quran/performance/budgets.json
  modified:
    - crates/application/tests/doctor_indexes.rs
    - crates/application/tests/search_latency.rs
    - crates/tool-registry/src/lib.rs
    - crates/application/src/quran_tools.rs
    - crates/cli/src/lib.rs
    - crates/application/tests/quran_tools.rs
    - .planning/WINDOWS.md

key-decisions:
  - "G-08: 'all 19 checks green' is realized as 19 stable ids with no failure. The token-level index (quran.fts.token_index) stays skipped by design — there is no `quran.token.v1` backend — and a structural assertion pins it as the only non-pass, so it can never be a silent pass or a false fail."
  - "QAI-IDX-0101 is asserted on the SearchHit stale-index warning (the only emitter of that code) after a v2 edition is activated while the serving generation still points at v1; doctor reports quran.index.drift as warn with a next command and performs no repair (T-03-27)."
  - "G-06: budgets are codified with applies_to per row; the full-corpus p50/p99 rows are recorded as OD-11-dependent and never executed against the 14-ayah fixture. The fixture bound stays 2000 ms and the harness asserts the artifact value equals the pre-existing constant (no silent loosening)."
  - "G-11/D-13: the QuranBackend trait gains backend_search/root/lemma/morphology/family with default typed-unsupported impls, so no backend is forced to implement them and server fakes compile unchanged; the real ReaderToolBackend builds attributed envelopes."
  - "quran.search runs normalized L3 search, so its envelope always carries the served hit's rule ids (I9) as normalization_rules; the lexicon tools carry dataset attribution in analysis_sources and no normalization rules (no trace applies)."
  - "An unavailable dataset reaches the tool caller as the typed backend error QAI-MORPH-0004 (never an empty result), matching the CLI/HTTP surface contract established in 03-05."
  - "OD-11 and OD-12 stay BLOCKED: every soak/tool assertion runs on the synthetic `synthetic_test_only` lexicon, and the full-corpus latency rows await a licensed corpus."

patterns-established:
  - "Derived-state soak: doctor's 19 checks asserted end-to-end on activated morphology + trigram postings, with the drift case proving the stale-index code path"
  - "Budget artifact + harness gate: one JSON table (ADR + plan table + fixture bound) loaded at compile time, with the full-corpus rows explicitly deferred"
  - "Attributed tool envelope: search carries normalization_rules + canonical sources; lexicon tools carry dataset attribution; unavailable capability is a typed error"

requirements-completed: [REQ-quran-normalization, REQ-quran-linguistics]

# Coverage metadata (#1602)
coverage:
  - id: D1
    description: "G-08: after a full rebuild -> morphology activate -> one search per mode, `qai doctor --indexes` reports all 19 checks with stable ids and no failure (morphology + lexicon + trigram + canonical + smoke green; token-index skipped by design), and an injected drift (activate v2 while the serving generation points at v1) produces QAI-IDX-0101 and a warn on quran.index.drift. Doctor stays read-only."
    requirement: REQ-quran-linguistics
    verification:
      - kind: integration
        ref: "cargo test -p application --test doctor_indexes (7 pass; doctor_indexes_soak_green_on_activated_morphology_and_trigram, injected_drift_reports_stale_index_code)"
        status: pass
    human_judgment: false
  - id: D2
    description: "G-06: fixtures/quran/performance/budgets.json exists, parses, codifies ADR-0207's concatenated p99 <= 150 ms plus the plan 17.1 p50/p99 table, and every row carries applies_to + rationale; the fixture harness loads it, gates against fixture_bound_ms (asserted never loosened), and records the full-corpus rows as OD-11-dependent."
    requirement: REQ-quran-normalization
    verification:
      - kind: integration
        ref: "cargo test -p application --test search_latency (2 pass; budget_artifact_codifies_adr_0207_and_fixture_bound, search_latency_within_fixture_bounds)"
        status: pass
    human_judgment: false
  - id: D3
    description: "G-11/D-13: tool_names() lists quran.search/root/lemma/morphology/family plus the two reader tools; every new result carries analysis_sources (and normalization_rules for search), an empty selector is a typed invalid-input error, an unavailable dataset is the typed QAI-MORPH-0004 backend error, and the cli serve registry is wired with the serving-index root."
    requirement: REQ-quran-linguistics
    verification:
      - kind: unit
        ref: "cargo test -p tool-registry (6 pass; registry_lists_all_tools, empty_selector_tools_are_typed_invalid_input, new_tools_return_attributed_envelopes)"
        status: pass
      - kind: integration
        ref: "cargo test -p application --test quran_tools (9 pass; search_tool_returns_attributed_envelope, lexicon_tools_fail_closed_without_a_dataset, lexicon_tools_are_attributed_with_an_active_dataset)"
        status: pass
      - kind: other
        ref: "cargo build -p cli; cargo run -q -p xtask -- arch-check"
        status: pass
    human_judgment: false
  - id: D4
    description: "Linguistic correctness of the lexicon results served through the new tools, and ratification of the performance budgets' dataset applicability (OD-11/OD-12)."
    requirement: REQ-quran-linguistics
    verification: []
    human_judgment: true
    rationale: "The soak and the tool conformance tests run on the synthetic `synthetic_test_only` lexicon; the full-corpus p50/p99 rows cannot execute until a licensed corpus is active (OD-11 BLOCKED). No result asserts linguistic correctness (OD-12 BLOCKED, ADR-0204/0205/0210/0211/0215 unratified)."

# Metrics
duration: 28 min
completed: 2026-09-28
status: complete
---

# Phase 3 Plan 07: Doctor Soak + Performance Budgets + Attributed Tool Surface Summary

**The three Phase-3 exit-evidence gaps are closed: `doctor --indexes` is proven end-to-end on activated morphology + trigram postings (19 stable ids, no failure, drift yields `QAI-IDX-0101`), the ADR-0207 `p99 ≤ 150 ms` budget and the plan §17.1 table are codified in a machine-readable artifact that the fixture harness gates against, and the tool registry now exposes `quran.search`/`root`/`lemma`/`morphology`/`family` with mandatory attribution and normalization rules.**

## Performance

- **Duration:** 28 min
- **Started:** 2026-09-28T05:45:20Z (plan ledger base `5dddba8`)
- **Completed:** 2026-09-28 (SUMMARY written)
- **Tasks:** 3/3
- **Files modified:** 7 code/fixture files (1 created, 6 modified) + `.planning/WINDOWS.md`

## Accomplishments

- **G-08 closed end-to-end.** The soak test performs the full sequence on the synthetic fixture (import + activate edition → `rebuild_forms` → `rebuild_index` → morphology import + `activate_morphology` → one exact/normalized/concatenated search) and then asserts all 19 check ids are present in plan order with **no failure**: normalization, forms, canonical-unchanged, ayah FTS, skeleton/trigram, index drift/orphans, all six morphology/lexicon checks, and the 12-probe `quran.search.smoke` are green. The only non-pass is `quran.fts.token_index`, which is `skipped` by design (no `quran.token.v1` backend) and structurally pinned as the sole exception.
- **Drift is detected, and detection is proven where the code exists.** `injected_drift_reports_stale_index_code` imports and activates a v2 edition (new edition id + corpus generation) while the serving generation still points at v1: every search hit then carries `QAI-IDX-0101`, and doctor reports `quran.index.drift` as `warn` with a next command. Doctor never repairs (T-03-27) — the soak also re-asserts the byte-identical read-only property, including with `--deep`.
- **G-06 closed.** `fixtures/quran/performance/budgets.json` is a machine-readable table: ADR-0207's concatenated `p99 ≤ 150 ms`, the plan §17.1 p50/p99 rows for exact/normalized/adhoc/phrase/concatenated/regex/root/family/morphology/frequency/normalize-explain, and `fixture_bound_ms: 2000`. Every row carries `value_ms`, `applies_to` (`full-corpus`/`fixture`) and a non-empty `rationale`. The `search_latency` harness loads it with `include_str!`, gates the fixture runs against `fixture_bound_ms`, asserts that value equals the pre-existing 2000 ms constant (never loosened), and records the full-corpus rows as **OD-11-dependent** (no licensed corpus), never executing them on the fixture.
- **G-11/D-13 closed.** `tool_names()` now returns all seven tools. `QuranBackend` gained `backend_search`/`backend_root`/`backend_lemma`/`backend_morphology`/`backend_family` with default typed-unsupported impls (so existing fakes compile unchanged), and the registry entry points reject empty selectors with typed invalid-input errors. `ReaderToolBackend` implements the five backends against the real services: `quran.search` runs normalized L3 search and populates `normalization_rules` from the served hit's `NormalizationTrace` (I9) plus canonical `analysis_sources`; the lexicon tools populate `analysis_sources` from the active dataset, and an unavailable dataset surfaces the typed `QAI-MORPH-0004` backend error rather than an empty result.
- **The serve surface is live.** `crates/cli/src/lib.rs` wires `ReaderToolBackend::registry_with_index_root(reader, index_root_for_db(db_path))` so the running server exposes the full D-13 tool set.
- **No regressions.** The full `application` suite is green (51 lib + every integration binary, including `quran_identity` which now passes 7/7 after a concurrent 01-04 fix), `server` 22/22, `cli` 14/14 for the quran flow suite, `tool-registry` 6/6, `arch-check` OK, and clippy clean with `-D warnings` on the touched crates.

## Task Commits

Task 3 is `tdd="true"`: RED → GREEN.

1. **Task 1: `doctor --indexes` end-to-end soak (G-08)** — `a1f907d` (test)
2. **Task 2: Codify performance budgets + threshold gate (G-06)** — `a0cb699` (test)
3. **Task 3: Register attributed search/lexicon tools (G-11, D-13)**
   - `196cf87` (test — RED: `registry_lists_all_tools` + `new_tools_return_attributed_envelopes` failing)
   - `ca91230` (feat — GREEN: 7 tool names, real backends, cli serve wiring, application conformance tests)

**Plan metadata:** (this commit) `docs(03-07): complete [plan-name] plan`

## Files Created/Modified

- `fixtures/quran/performance/budgets.json` (new) — the G-06 machine-readable budget table (ADR-0207 concatenated p99 ≤ 150 ms, plan §17.1 p50/p99 rows, fixture bound, regression tolerance, OD-11 note). 23 named rows.
- `crates/application/tests/doctor_indexes.rs` — added the soak (`soak_db`, `aligned_morph_document`, `morph_import_params`, `doctor_indexes_soak_green_on_activated_morphology_and_trigram`) and the drift case (`injected_drift_reports_stale_index_code`); seeded `appr-v2` + `appr-morph` approvals in the shared harness; added the `storage::Database` import.
- `crates/application/tests/search_latency.rs` — `BUDGETS_JSON` (include_str!), `budgets()`/`budget_ms()` helpers, `budget_artifact_codifies_adr_0207_and_fixture_bound`, and the fixture-bound gate derived from the artifact (asserted equal to `FIXTURE_BOUND`).
- `crates/tool-registry/src/lib.rs` — five param structs + version consts; five `QuranBackend` methods with default typed-unsupported impls; `unsupported()` helper; `ToolRegistry::TOOL_NAMES` (7) and the `search/root/lemma/morphology/family` entry points with empty-selector guards; fake-backend overrides + `attributed()` test helper.
- `crates/application/src/quran_tools.rs` — `index_root` on `ReaderToolBackend` with `with_index_root`/`registry_with_index_root`; `meta_from_edition`/`active_meta`/`pinned_ref`/`dataset_source`; `search_tool_error`/`morphology_tool_error`; the five backend implementations with attributed envelopes.
- `crates/cli/src/lib.rs` — serve wiring for `registry_with_index_root`.
- `crates/application/tests/quran_tools.rs` — real-backend conformance tests (search envelope + empty guard; lexicon fail-closed without a dataset; attributed lexicon results with an active dataset) + their setup helpers.
- `.planning/WINDOWS.md` — recorded the three 03-07 deviations.

## Decisions Made

- **"All 19 green" = 19 stable ids with no failure.** The token-index check has no backend in this phase; it stays `skipped` (asserted as the only non-pass). Any future `pass` there will require a real token index — the test cannot silently accept one.
- **QAI-IDX-0101 is asserted on the SearchHit warning.** That code is only emitted by search (`Warning::stale_index`); the drift test therefore injects a serving-edition/active-edition mismatch (activate v2) and asserts both the hit warning and the doctor `warn`. Doctor does not repair.
- **Budgets: one artifact, no silent loosening.** Rows are typed with `applies_to`; the full-corpus p50/p99 rows are deferred as OD-11-dependent rather than run against fixture data that cannot meet or violate them meaningfully.
- **Trait defaults keep the blast radius small.** Default `QuranBackend` methods returning a typed error let the server fakes and reader-only convenience functions compile unchanged, while `ReaderToolBackend` provides the real implementations.
- **Search tool = normalized L3.** This guarantees `normalization_rules` is populated from a real trace (I9), satisfying the "no result without explainability" requirement; the lexicon tools carry dataset attribution instead.
- **OD-11/OD-12 remain BLOCKED.** Every assertion runs on the synthetic lexicon; nothing here is asserted as linguistically correct.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] The Task-3 conformance tests need a test file the plan did not list**
- **Found during:** Task 3 (GREEN)
- **Issue:** Task 3's acceptance criteria require "tool-conformance tests assert attribution + normalization rules on the new tools" and its `verify` runs `cargo test -p application --test quran_tools`, but `files_modified` lists only the three production files. Without extending the test file there is no real-backend conformance evidence.
- **Fix:** Added three tests to `crates/application/tests/quran_tools.rs` (attributed search envelope + empty guard; lexicon fail-closed without a dataset; attributed lexicon results with an active dataset) plus their setup helpers.
- **Files modified:** `crates/application/tests/quran_tools.rs`
- **Verification:** `cargo test -p application --test quran_tools` → 9 pass.
- **Committed in:** `ca91230` (Task 3 GREEN)

**2. [Plan-text interpretation] "All 19 checks green" vs the backendless token index**
- **Found during:** Task 1
- **Issue:** The plan's must-have says "all 19 checks green", but `quran.fts.token_index` has no backend and is `skipped` by design (a pre-existing, intentional state asserted by the Phase-2 suite).
- **Fix:** The soak asserts 19 stable ids, **no failure**, and explicitly that `quran.fts.token_index` is the only `skipped` check. This honors the intent (no hidden or false-green check) without inventing a token index outside this plan's scope.
- **Files modified:** `crates/application/tests/doctor_indexes.rs`
- **Verification:** `cargo test -p application --test doctor_indexes` → 7 pass.
- **Committed in:** `a1f907d`

**3. [Plan-text interpretation] "Injected drift produces QAI-IDX-0101"**
- **Found during:** Task 1
- **Issue:** The plan's drift examples (delete a generation file / bump the recorded generation) make doctor checks `fail`/`warn`, but `QAI-IDX-0101` is only emitted by the search stale-index warning. Deleting a file cannot produce that code.
- **Fix:** The drift case activates a v2 edition (bumping the corpus generation and changing the active edition id) while the serving generation still points at v1, then asserts `QAI-IDX-0101` on the (still-serving) hits and `warn` on `quran.index.drift` — satisfying both halves of G-08's evidence.
- **Files modified:** `crates/application/tests/doctor_indexes.rs`
- **Verification:** `cargo test -p application --test doctor_indexes -- injected_drift_reports_stale_index_code` → pass.
- **Committed in:** `a1f907d`

**4. [TDD note] Task 3 REFACTOR produced no changes**
- **Found during:** Task 3
- **Issue:** After GREEN, clippy (`-D warnings`) and `rustfmt --check` were clean and the design needed no restructuring.
- **Resolution:** No REFACTOR commit; RED (`196cf87`) → GREEN (`ca91230`) is the full cycle. Recorded here rather than fabricated.

---

**Total deviations:** 1 blocking (integration wiring) + 2 plan-text interpretations + 1 TDD note.
**Impact on plan:** No scope creep: no new dependencies (`T-03-SC` stays accepted), no migration, no canonical/hash recipe touched. Every declared `files_modified` entry was touched; the extra file is the test the plan's own acceptance and verify commands require.

## Issues Encountered

- **`serve_hosts_the_worker_and_shuts_down_joined` failed once under full-suite load** (94 s, job `Queued`), then passed standalone in 5.30 s and the full `--test quran` suite passed 14/14 in 11.14 s. This is the identical shared-tree cargo contention artifact 03-05 and 03-06 recorded; not a regression.
- **`cli --test doctor_json::audit_verify_rejects_corrupt_chain_without_modifying_database` remains failing** — the known pre-existing audit-tamper failure (proven unrelated by 03-05 on the pre-plan base). Untouched.
- **`application --test quran_identity` now passes 7/7** — a concurrent workstream committed `4745df7 fix(01-04): drain enqueue-only import queue in quran_identity tests` during this plan, clearing the previously-known 2 `NotStaged` failures. No action taken by this plan.
- **The plan ledger window contains four concurrent-session commits** (`4745df7`, `89e5f03`, `bc0a27f`, `24b2dcf`) on the shared tree, so the raw `rev-list` count is 8 while the plan's own commits are 4. Recorded transparently in `actuals`.

## User Setup Required

None — no external service configuration, no new dependencies, no package installs (`T-03-SC` remains accepted).

## Known Stubs

None new. Two recorded, intentional non-executing entries (not code stubs):

- `fixtures/quran/performance/budgets.json` → the `full-corpus` p50/p99 rows are recorded but **not executed**: they require a licensed corpus (OD-11 BLOCKED). Only `fixture_bound_ms` gates the harness. Asserted and documented in the artifact's `notes`.
- `quran.fts.token_index` → `skipped` by design (no `quran.token.v1` backend), pre-existing, and pinned by both the Phase-2 suite and the new soak.

## Threat Flags

None beyond the plan's own register. `T-03-26` (results built from verified hits/attributed analyses — the fake and real backends both populate `analysis_sources`, and lexicon tools cannot return results without the active dataset), `T-03-27` (doctor is read-only; the soak re-asserts byte-identical state and adds no repair path), `T-03-28` (budgets gate the fixture; the tool surface reuses the clamped search paths and the empty-selector guards bound input), and `T-03-29` (`analysis_sources`/`normalization_rules` asserted on every new tool) are each mitigated and tested. `T-03-SC` stays accepted (no packages added).

## Owner Gates (BLOCKED — not silently passed)

- **OD-11 (morphology dataset selection & licensing, ADR-0203)** — BLOCKED. The doctor soak and every tool conformance test run on the synthetic `synthetic_test_only` lexicon; the full-corpus latency rows are recorded as dataset-gated and never executed. Closing step: capture the QAC license to `licenses/qac/` per `licenses/README.md`; ratify ADR-0203 Option A when `redistribution_allowed: true` with all fields, else keep Option B; record in `docs/05-followups/owner-decisions.md`.
- **OD-12 (normalization catalog + named linguist, ADR-0204/0205/0210/0211/0215)** — BLOCKED. Surfacing tools and proving the doctor soak does not ratify any tagset, relation semantics, or counting convention. Closing step: name a qualified Arabic linguist, record sign-off in `docs/reviews/`, flip ADR-0204/0205 to Accepted (and ratify ADR-0210/0211/0215).

## Next Phase Readiness

- **03-08** and later phases can rely on: a proven doctor soak harness (`soak_db`) for end-to-end derived-state assertions, a machine-readable budget artifact to extend for full-corpus runs once a corpus lands, and a 7-tool registry whose results always carry attribution and (for search) rule traces.
- The tool surface is a stable seam: `QuranBackend`'s new methods are optional-but-typed, so future backends (HTTP, agent runtime) can adopt them without breaking existing implementors.
- **Open follow-ups:** the full-corpus latency rows remain OD-11-gated; the three 03-07 deviations are in `.planning/WINDOWS.md`; the pre-existing `cli --test doctor_json` audit-tamper failure remains for its owner.
- Verification snapshot: `cargo test -p application` (51 lib + all integration binaries green, incl. doctor_indexes 7, search_latency 2, quran_tools 9), `cargo test -p server` (22), `cargo test -p tool-registry` (6), `cargo test -p cli --test quran` (14), `cargo build -p cli` (ok), `cargo clippy -p tool-registry -p application -p cli --all-targets -- -D warnings` (clean), `rustfmt --edition 2024 --check` on all touched files (clean), `xtask arch-check` (OK).

---

*Phase: 03-quran-search-linguistics*
*Completed: 2026-09-28*

## Self-Check: PASSED

- All 7 declared/created files exist: `fixtures/quran/performance/budgets.json`, `crates/application/tests/doctor_indexes.rs`, `crates/application/tests/search_latency.rs`, `crates/tool-registry/src/lib.rs`, `crates/application/src/quran_tools.rs`, `crates/cli/src/lib.rs`, plus the integration-wiring `crates/application/tests/quran_tools.rs`.
- All 4 plan task commits exist: `a1f907d` (Task 1), `a0cb699` (Task 2), `196cf87` (Task 3 RED), `ca91230` (Task 3 GREEN).
- Verification commands re-run and green: `cargo test -p application --test doctor_indexes` (7), `--test search_latency` (2), `--test quran_tools` (9), `cargo test -p tool-registry` (6), `cargo test -p server` (22), `cargo test -p cli --test quran` (14), `cargo build -p cli` (ok), `cargo clippy -p tool-registry -p application -p cli --all-targets -- -D warnings` (clean), `rustfmt --check` on touched files (clean), `xtask arch-check` (OK).
- Out-of-scope failures left alone and documented: `cli --test doctor_json` (pre-existing audit-tamper).
