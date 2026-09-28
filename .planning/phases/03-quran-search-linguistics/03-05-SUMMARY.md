---
phase: 03-quran-search-linguistics
plan: 05
subsystem: quran-lexicon
tags: [rust, cli, http-api, axum, word-family, root-frequency, lemma-frequency, typed-errors, attribution, d-10, g-02, g-10, g-13]

# Dependency graph
requires:
  - phase: 02-canonical-quran-core
    provides: quran_datasets activation + approval-gated activate_morphology, word_family_relations (0017), MorphologyToolError (QAI-MORPH-0004), the AppState/QuranApiBackend route shell and its error bodies
  - phase: 03-quran-search-linguistics
    provides: 03-03 root/lemma frequency + CountingRules/FrequencyReport (the count reports this plan serves); 03-04 every typed family relation kind built, explained and dataset-attributed (the relations this plan serves); 03-02 the host-backed trycmd segment convention
provides:
  - "CLI: top-level `qai quran family <kind> <id>` (command_output with dataset id + typed members) and the D-10 `qai quran lemma <lemma>` alias"
  - "application::quran_cli::cmd_family on the cmd_morphology_root shape, with a non-empty kind/id usage guard and tool_exit mapping (missing dataset = exit 5, QAI-MORPH-0004)"
  - "crates/cli/tests/quran/family_s1.trycmd + family_s2.trycmd host-backed segments (typed unavailability, usage guard, MV-018 setup line) driven by a new quran_family_snapshots test"
  - "application::quran_lexicon_api: LexiconBackend trait (Send + Sync), LexiconApiService, validated FamilyArgs/RootFrequencyArgs/LemmaFrequencyArgs, LexiconApiError with a QAI-LEX-* Diagnostic map"
  - "HTTP: POST /api/v1/quran/family, /api/v1/quran/count/root-frequency, /api/v1/quran/count/lemma-frequency on the shared layer-wrapped router; AppState.lexicon wired in cli serve and in the server test state"
  - "4 server contract tests: attributed relations, no-empty-envelope typed unavailability, the CountingRules block, coded 400s for bad selectors/modes"
affects: [03-06, 03-07, 03-08, phase-04-graph, phase-05-result-contract]

# Actuals (#2632) — same estimateTokens scale as the plan's estimate (chars/4 over the realized diff)
actuals:
  tokens: 9014
  tasks: 2
  commits: 2
  plan_head_before: 2d5878a790cef6375ef441ae59456e99db558cb9

tech-stack:
  added: []
  patterns:
    - "Surface-only plan: both new surfaces wrap the existing service functions (word_family, root_frequency, lemma_frequency) instead of re-deriving results, so CLI and HTTP cannot drift"
    - "Typed unavailability across surfaces: a missing dataset is exit 5 (CLI, via the existing tool_exit) and HTTP 404 with QAI-LEX-0004 — never an empty list / empty 200 envelope"
    - "One shared selector parser: the CLI's --mode parser is pub and reused by the HTTP arg constructors, so a count means the same thing on both surfaces"
    - "Host-backed trycmd segments (s1 = migrate+enqueue, s2 = activate+assert): a queued import must reach its host-owned terminal state before dependent commands run"

key-files:
  created:
    - crates/application/src/quran_lexicon_api.rs
    - crates/cli/tests/quran/family_s1.trycmd
    - crates/cli/tests/quran/family_s2.trycmd
  modified:
    - crates/application/src/quran_cli.rs
    - crates/application/src/lib.rs
    - crates/cli/src/quran.rs
    - crates/cli/src/lib.rs
    - crates/cli/tests/quran.rs
    - crates/server/src/api.rs
    - crates/server/tests/api.rs
    - .planning/phases/03-quran-search-linguistics/deferred-items.md

key-decisions:
  - "`family` and `lemma` are TOP-LEVEL `QuranAction` variants (D-10): nesting `Family` under `MorphologyAction` would render `qai quran morphology family …`, so the plan's explicit instruction to dispatch directly to cmd_family was followed and the command tree is asserted by a snapshot that only the top-level handler can produce."
  - "The CLI maps a missing dataset through the existing `tool_exit` (exit 5 NOT_FOUND, QAI-MORPH-0004) exactly like `cmd_morphology_root` and the counting_graph_s2 frequency snapshots. The plan's Task-1 acceptance text said 'as a policy error (`? 2`)', which contradicts the plan's own mandated `tool_exit` mapping (UnavailableDataset → NOT_FOUND) and the existing snapshot precedent; the mapping and the established surface contract won, and the discrepancy is recorded here rather than encoded as a wrong snapshot."
  - "`LexiconApiError` introduces a new QAI-LEX-* namespace (1 storage, 2 invalid input, 3 unsupported capability, 4 active-dataset-unavailable) and its UnavailableDataset maps to HTTP 404, mirroring the CLI's NOT_FOUND, so the no-dataset path can never be a 200 envelope (T-03-21)."
  - "`LexiconBackend::word_family` returns `Vec<FamilyMemberView>` exactly as the plan pinned it; attribution travels per member (`FamilyMemberView.dataset`), which the contract test asserts, so the pinned signature and threat T-03-19 are both satisfied without inventing a second response type."
  - "The published `FrequencyReport` serializes its CountingRules block under the field name `rules`; the block (datasets + multi_analysis_handling + profile + version + exclusions) and the checksum are all in the response and asserted. No rename to `counting_rules` — that would break the 03-03 CLI/--json contract for zero gain."
  - "The plan's probe boundaries are respected: `quran_cli::parse_multi_analysis_mode` becomes pub and is reused by `RootFrequencyArgs::new`/`LemmaFrequencyArgs::new` so an API count and a CLI count declare the same convention (same accepted spellings, same error wording)."
  - "`docs/08-api/quran-v1-openapi.json` was deliberately NOT edited (outside the plan's files_modified, and the tree is shared with a concurrent session); the missing route documentation is logged in deferred-items.md and the WINDOWS.md ledger instead."
  - "OD-11 and OD-12 stay BLOCKED and are not silently passed: every family relation and every count this plan serves is proven on the synthetic `synthetic_test_only` lexicon, and no surface asserts linguistic correctness."

patterns-established:
  - "A lexicon surface is a trait seam (`LexiconBackend`) + a live service over `SqliteDatabase` + a fake in server contract tests — the same shape search already uses, so the HTTP layer never names storage"
  - "Capability unavailability is expressed once at the service boundary (typed error) and translated per surface: exit 5 for the operator, 404 + Diagnostic body for HTTP"
  - "New routes are added to the existing router before its layer stack, inheriting the 1 MiB body limit, 30 s timeout and concurrency 128 without restating them"

requirements-completed: [REQ-quran-linguistics]

# Coverage metadata (#1602)
coverage:
  - id: D1
    description: "`qai quran family <kind> <id>` and `qai quran lemma <lemma>` are first-class TOP-LEVEL CLI commands (not nested under `morphology`) and dispatch to cmd_family / the lemma read path; a non-empty kind/id guard is a usage error before any read."
    requirement: REQ-quran-linguistics
    verification:
      - kind: integration
        ref: "cargo test -p cli --test quran -- quran_family_snapshots"
        status: pass
      - kind: integration
        ref: "cargo test -p cli --test quran"
        status: pass
    human_judgment: false
  - id: D2
    description: "SC3 at the CLI: with no active dataset `qai quran family` prints the typed `UnavailableDataset` (QAI-MORPH-0004) and exits 5 instead of an empty relation list; the MV-018 canonical-unchanged setup line is asserted in the same segment."
    requirement: REQ-quran-linguistics
    verification:
      - kind: integration
        ref: "cargo test -p cli --test quran -- quran_family_snapshots"
        status: pass
      - kind: other
        ref: "cargo run -q -p xtask -- arch-check"
        status: pass
    human_judgment: false
  - id: D3
    description: "SC3/SC4 at the versioned HTTP API: POST /api/v1/quran/family returns typed relations with a mandatory explanation and dataset attribution, and POST /api/v1/quran/count/{root,lemma}-frequency return the CountingRules block (datasets, multi_analysis_handling, profile, checksum); an unavailable dataset is a typed error status (404 QAI-LEX-0004) and empty selectors/unknown modes are coded 400s — no lexicon route can answer an empty 200 envelope. AppState.lexicon is wired in both cli serve and the server test state."
    requirement: REQ-quran-linguistics
    verification:
      - kind: integration
        ref: "cargo test -p server"
        status: pass
      - kind: other
        ref: "cargo build -p cli"
        status: pass
      - kind: manual_procedural
        ref: "live `qai serve` smoke: POST family/root-frequency/lemma-frequency against a migrated temp db returned 404 QAI-LEX-0004 (no dataset) and 400 QAI-LEX-0002 (empty selector / unknown mode)"
        status: pass
    human_judgment: false
  - id: D4
    description: "Linguistic correctness of the family relations served here and ratification of the counting convention."
    requirement: REQ-quran-linguistics
    verification: []
    human_judgment: true
    rationale: "This plan wires surfaces only; the relations and counts it serves are the code-asserted synthetic ones from 03-03/03-04. ADR-0210/0211/0215 and OD-11/OD-12 are BLOCKED: no qualified Arabic linguist has ratified the tagset, root conventions, alignment, relation semantics or counting convention, and no licensed morphology dataset is active. Until those close, no surface result may be treated as scholarly ground truth."

# Metrics
duration: 1h 50m
completed: 2026-09-28
status: complete
---

# Phase 3 Plan 05: Word-Family + Frequency Surfaces at the CLI and the Versioned HTTP API Summary

**`qai quran family <kind> <id>` (plus the D-10 `lemma` alias) and `POST /api/v1/quran/family` + `/count/{root,lemma}-frequency` now serve the 03-03/03-04 lexicon services through one shared trait seam — typed, explained, dataset-attributed results on both surfaces, and a typed `UnavailableDataset` (CLI exit 5 / HTTP 404 `QAI-LEX-0004`) instead of an empty list or an empty 200 envelope whenever no dataset is active.**

## Performance

- **Duration:** 1h 50m wall clock (inflated — see Issues Encountered: shared-tree cargo lock/CPU contention with a self-launched full-suite run)
- **Started:** 2026-09-28T02:39Z (plan ledger base `2d5878a`)
- **Completed:** 2026-09-28 (SUMMARY written)
- **Tasks:** 2/2
- **Files modified:** 10 code files (3 created, 7 modified) + 2 planning artifacts + WINDOWS.md

## Accomplishments

- **G-02 closed at the CLI:** `quran.word_family` was service-only; `cmd_family` now exists on the exact `cmd_morphology_root` shape and `QuranAction::Family { kind, id }` is a **top-level** command, so the rendered surface is precisely D-10's `qai quran family <kind> <id>` (not `qai quran morphology family …`).
- **G-13 closed:** the D-10 `qai quran lemma <lemma>` top-level alias exists and dispatches to the existing lemma read path.
- **Fail-closed, not empty:** a missing dataset surfaces the typed `UnavailableDataset` through the existing `tool_exit` mapping — `QAI-MORPH-0004`, exit 5 — with a snapshot pinning the exact operator text; the same behaviour as `cmd_morphology_root` and the 03-03 frequency snapshots.
- **G-10 closed at the API:** `POST /api/v1/quran/count/root-frequency` and `/count/lemma-frequency` return the full `FrequencyReport` including the `CountingRules` block (active `<slug>@<version>` in `datasets`, the requested `multi_analysis_handling`, the profile, the exclusions) and the reproducibility checksum.
- **SC3 at the API:** `POST /api/v1/quran/family` returns typed relations with the mandatory explanation and dataset attribution on every member, with `meta.reproducibility.tool = "quran.word_family"`.
- **No empty 200 for an unavailable capability (T-03-21):** `LexiconApiError::UnavailableDataset` → 404 with a full `Diagnostic` body (`QAI-LEX-0004`), asserted for all three routes; `InvalidInput`/`Unsupported` → 400 (`QAI-LEX-0002`/`-0003`); `Storage` → 500. Verified against a **live** `qai serve` on a migrated temp database, not just the fake backend.
- **Shared layers inherited (T-03-20):** the three routes are added to the existing `router()` before its layer stack, so they sit behind the 1 MiB body limit, the 30 s timeout and concurrency 128 without duplicating them.
- **Input binding hardened (T-03-18):** selectors are validated non-empty before any storage read, on both surfaces; the storage lookups remain fully bound parameters (`family_relations_for` binds kind/id) with no concatenated SQL.
- **CLI and API cannot drift:** `LexiconApiService` wraps exactly `word_family`, `root_frequency` and `lemma_frequency` — the functions the CLI calls — and the `--mode` parser is shared.

## Task Commits

Each task was committed atomically:

1. **Task 1: `qai quran family` + `qai quran lemma` CLI surface (G-02, G-13)** - `cbd848e` (feat)
2. **Task 2: Versioned HTTP lexicon routes — family + root/lemma frequency (G-02, G-10)** - `adf477f` (feat)

**Plan metadata:** (this commit) `docs(03-05): complete [plan-name] plan`

_Note: neither task was TDD-typed, so each produced one commit._

## Files Created/Modified

- `crates/application/src/quran_cli.rs` — `cmd_family` (non-empty guard, `word_family`, dataset id + members JSON, `tool_exit` mapping); `parse_multi_analysis_mode` made `pub` and documented as the shared CLI/HTTP selector parser.
- `crates/application/src/quran_lexicon_api.rs` (new) — `LexiconBackend` trait, `LexiconApiService` over `SqliteDatabase`, validated `FamilyArgs`/`RootFrequencyArgs`/`LemmaFrequencyArgs`, `DEFAULT_PROFILE`, `LexiconApiError` + `Diagnostic` (`QAI-LEX-*`) + `From<MorphologyToolError>`/`From<CountingError>`.
- `crates/application/src/lib.rs` — `pub mod quran_lexicon_api;`.
- `crates/cli/src/quran.rs` — top-level `QuranAction::Family { kind, id }` and `QuranAction::Lemma { lemma }` variants + dispatch.
- `crates/cli/src/lib.rs` — `LexiconApiService::open` in the `serve` path and `AppState { …, lexicon }`.
- `crates/cli/tests/quran.rs` — `quran_family_snapshots` host-backed test over the two new segments.
- `crates/cli/tests/quran/family_s1.trycmd` (new) — `db migrate` + queued import.
- `crates/cli/tests/quran/family_s2.trycmd` (new) — activate, `forms rebuild` with the pinned `MV-018 canonical-unchanged: pass (sha256:235795…)` line, then the family/lemma assertions (typed unavailability exit 5; empty-selector usage error).
- `crates/server/src/api.rs` — `AppState.lexicon`; `FamilyBody`/`RootFrequencyBody`/`LemmaFrequencyBody`; `lexicon_error_status`/`lexicon_error_response`/`lexicon_meta`/`lexicon_response`; the three POST handlers; the three routes.
- `crates/server/tests/api.rs` — `FakeLexicon` + `lexicon_frequency_report` helper, `test_state` wiring, and 4 contract tests (attributed relations; no-empty-envelope + selector rejection; CountingRules block + unknown mode; typed unavailability for both count routes).
- `.planning/phases/03-quran-search-linguistics/deferred-items.md` — the three 03-05 findings (pre-existing audit-tamper failure with its base-commit proof, the un-documented OpenAPI routes, and the CLI snapshot scope note).

## Decisions Made

- **Top-level vs nested (D-10):** `Family`/`Lemma` are top-level `QuranAction` variants. The snapshot for `qai quran family token token:1:1:1` can only be produced by the top-level handler, so a regression to nesting fails the suite.
- **`? 2` vs `? 5` (plan-text correction):** the plan's Task-1 acceptance text expected the no-dataset family case "as a policy error (`? 2`)", but the plan's own action mandates mapping through `tool_exit`, which maps `UnavailableDataset` → `exit::NOT_FOUND` (5) — as `cmd_morphology_root` and the existing `counting_graph_s2` frequency snapshots already do. The mapping was followed (exit 5) rather than encoding a snapshot that contradicts the implemented contract.
- **New `QAI-LEX-*` namespace:** the HTTP surface needs codes of its own; numbering is append-only (1 storage, 2 invalid input, 3 unsupported, 4 unavailable) and `UnavailableDataset` maps to 404 to mirror the CLI's NOT_FOUND.
- **Pinned trait signature kept:** `word_family → Vec<FamilyMemberView>`; per-member `dataset` supplies attribution (asserted), instead of inventing a wrapper type the plan did not ask for.
- **No `counting_rules` rename:** the `CountingRules` block is serialized as `rules` by the published `FrequencyReport`; the block and its fields are asserted as-is.
- **OpenAPI untouched:** outside `files_modified` in a shared tree → recorded as a deferred follow-up (`todo` in WINDOWS.md) instead of an uncommitted edit.
- **Owner gates:** OD-11 (dataset/license) and OD-12 (linguist) remain BLOCKED; nothing this plan serves is asserted as linguistically correct.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] The named trycmd artifact cannot exist as specified; the host-backed harness requires segments**
- **Found during:** Task 1 (CLI surface)
- **Issue:** `files_modified` and Task-1 acceptance name `crates/cli/tests/quran/family.trycmd` "with the standard setup block (`qai db migrate`, import, activate, `forms rebuild` …)". Since the 01-04 enqueue-only refactor, `qai quran import` only *queues* a job that the `qai serve` host owns; `ServeGuard::run_segments` synchronizes on the host-owned terminal state **between files**, so `activate` cannot run in the same file as the import. Every real CLI flow in this directory is therefore split (`read_flow_s1..s4`, `search_s1/s2`, `counting_graph_s1/s2`, `verify_s1/s2`, …).
- **Fix:** Landed the family surface as `family_s1.trycmd` (migrate + queued import) and `family_s2.trycmd` (activate + `forms rebuild` with the pinned MV-018 line + the family/lemma assertions). This is the identical correction 03-02 recorded for `search.trycmd` → `search_s2.trycmd` and 03-03 for `counting_graph.trycmd` → `counting_graph_s2.trycmd`.
- **Files modified:** `crates/cli/tests/quran/family_s1.trycmd`, `crates/cli/tests/quran/family_s2.trycmd`
- **Verification:** `cargo test -p cli --test quran` → 14/14 pass (the new segments included).
- **Committed in:** `cbd848e` (Task 1 commit)

**2. [Rule 3 - Blocking] A new trycmd file never executes without a segment-list entry**
- **Found during:** Task 1 (CLI surface)
- **Issue:** `crates/cli/tests/quran.rs` drives every `.trycmd` file through `ServeGuard::run_segments(&[…])`; the only glob is `tests/quran/*.toml`, which matches nothing. A new trycmd file with no entry in a `#[test]` body is silently never run — so it could not "exist and pass" in any meaningful sense.
- **Fix:** Added `quran_family_snapshots` (a `ServeGuard` + `run_segments(&["family_s1", "family_s2"])`) alongside the other flow tests.
- **Files modified:** `crates/cli/tests/quran.rs` (not in the plan's `files_modified`)
- **Verification:** `cargo test -p cli --test quran -- quran_family_snapshots` → 1 passed, all 7 snapshot cases ok; full suite 14/14.
- **Committed in:** `cbd848e` (Task 1 commit)

**3. [Rule 2 - Missing Critical] Non-empty kind/id guard in `cmd_family`**
- **Found during:** Task 1 (CLI surface)
- **Issue:** `cmd_morphology_root` performs no validation, and neither would a literal copy; the plan's route threat model (T-03-18) requires non-empty validation of the member selector, and `qai quran family "" ""` would otherwise hit storage with empty bindings.
- **Fix:** `cmd_family` returns a typed usage error (exit 2) before opening the database, and the API's `FamilyArgs::new`/`RootFrequencyArgs::new`/`LemmaFrequencyArgs::new` reject empty selectors/count targets the same way (asserted in the server contract test).
- **Files modified:** `crates/application/src/quran_cli.rs`, `crates/application/src/quran_lexicon_api.rs`
- **Verification:** `family_s2.trycmd` pins `? 2` + `error: family needs a non-empty <kind> and <id>`; `lexicon_family_route_never_returns_an_empty_envelope` pins HTTP 400 `QAI-LEX-0002`.
- **Committed in:** `cbd848e` (CLI guard) / `adf477f` (API guard)

**4. [Rule 3 - Blocking] `LexiconApiError` needed a `Diagnostic` implementation to reuse the error-body shape**
- **Found during:** Task 2 (HTTP routes)
- **Issue:** the plan asked for "an error enum with a `Diagnostic`/`QAI-*` mapping" and to "reuse `search_response`/`search_error_response`-style mapping"; the existing body builder needs `code()`/`summary()`/`remedy()`/`next_command()`.
- **Fix:** implemented `storage::error::Diagnostic` for `LexiconApiError` (code prefix `QAI-LEX`) plus `From<MorphologyToolError>`/`From<CountingError>`, so the handlers build the same `ErrorBody` as the search path without naming either low-level error enum.
- **Files modified:** `crates/application/src/quran_lexicon_api.rs`, `crates/server/src/api.rs`
- **Verification:** `cargo test -p server` → 22/22 pass (4 new); live-server smoke returns the full diagnostic body.
- **Committed in:** `adf477f` (Task 2 commit)

---

**Total deviations:** 4 auto-fixed (3 blocking, 1 missing critical), plus 1 plan-text correction (the `? 2` → `? 5` exit-code expectation) documented under Decisions.
**Impact on plan:** No scope creep: no new dependencies (`T-03-SC` stays accepted), no package installs, no service logic rewritten, and every `files_modified` entry was touched except that the trycmd artifact became two segment files and `crates/cli/tests/quran.rs` gained the runner entry the harness requires.

## Issues Encountered

- **`cargo test -p cli --test doctor_json` fails on `audit_verify_rejects_corrupt_chain_without_modifying_database` — pre-existing, proven.** `report["tampered_sequences"]` is `Null` where the test expects the tampered sequence. Proven unrelated by reproducing the identical failure on a read-only `git archive` snapshot of the plan base `2d5878a` with none of this plan's changes present. Logged to `deferred-items.md` and the `WINDOWS.md` ledger; not fixed (outside scope).
- **`cargo test -p application --test quran_identity` fails its 2 `NotStaged` cases** — the known pre-existing consequence of 01-04's enqueue-only import, as flagged in this plan's execution context. Untouched.
- **Shared-tree cargo contention made the wall clock misleading.** A `cargo test --workspace --no-fail-fast` and a `cargo test -p application` run launched in the background held the `target/` lock while I verified the plan's own gates, so several invocations timed out at 25–40 min and the recorded duration (1h 50m) overstates the work. The plan's gates were all run to completion once the lock freed: `cargo test -p cli --test quran` 14/14, `cargo test -p server` 22/22, `cargo build -p cli` ok, `cargo test -p application --lib` 45/45, and the three lexicon suites this plan builds on (`counting` 9, `family_goldens` 1, `morphology_import` 12). The full application integration matrix (`--no-fail-fast`) was left unfinished — it was still compiling its 27 test binaries when its own 60-minute budget expired — so it is not claimed as evidence here.
- **`cargo fmt -p application --check` remains red for two committed 03-01 test files** (`alpha_smoke.rs`, `canonical_display_identity.rs`) — pre-existing drift already recorded by 03-03/03-04. Every file this plan touched is rustfmt-clean (formatting was applied to the three files that needed it; nothing else was reformatted).
- **`gsd_run` is not available as a shell function in a fresh executor shell**; the tools were invoked directly as `node ~/.config/opencode/gsd-core/bin/gsd-tools.cjs …` (the shim is not present under the repo root).

## User Setup Required

None — no external service configuration required. No new dependencies and no package installs (`T-03-SC` remains accepted).

## Known Stubs

None. No placeholder, hardcoded-empty or TODO value flows to any output: the no-dataset paths are typed errors on both surfaces, the count reports always carry a rules block and checksum, and the OpenAPI documentation gap is tracked as a deferred follow-up rather than silently left.

## Threat Flags

None beyond the plan's own register. The three new routes add no new trust boundary of a class the plan did not already cover: `T-03-18` (validated selectors, bound storage parameters), `T-03-19` (per-member dataset attribution, asserted), `T-03-20` (registered on the layer-wrapped router) and `T-03-21` (typed error status, never an empty 200) are each mitigated and contract-tested.

## Owner Gates (BLOCKED — not silently passed)

- **OD-11 (morphology dataset selection & licensing, ADR-0203)** — BLOCKED. All family relations and counts served by these surfaces are proven on the synthetic `synthetic_test_only` lexicon only. Closing step: capture the QAC license to `licenses/qac/` per `licenses/README.md`; ratify ADR-0203 Option A when `redistribution_allowed: true` with all fields, else keep Option B; record in `docs/05-followups/owner-decisions.md`.
- **OD-12 (normalization catalog + named linguist, ADR-0204/0205/0210/0211/0215)** — BLOCKED. Surfacing a relation or a count does not ratify it: the tagset, root conventions, alignment, relation semantics and the multi-analysis counting convention remain unratified. Closing step: name a qualified Arabic linguist, record sign-off in `docs/reviews/`, flip ADR-0204/0205 to Accepted (and ratify ADR-0210/0211/0215).

## Next Phase Readiness

- SC3 and SC4 are now reachable at both surfaces D-10 names: `qai quran family/lemma`, and `POST /api/v1/quran/family` + `/count/{root,lemma}-frequency`. 03-06/03-07/03-08 can build on the same trait seam (`LexiconBackend`) and the same `QAI-LEX-*` error map.
- The lexicon surfaces are provably fail-closed: no configuration of the current code can return a lexicon result without a dataset, and no route returns an empty 200 for an unavailable capability.
- **Open follow-ups:** the three new routes are missing from `docs/08-api/quran-v1-openapi.json` (`WINDOWS.md` `todo`); a CLI-level attributed family snapshot would need an approval-seeding helper (`WINDOWS.md` `deviation`); the pre-existing audit-tamper failure and the 03-01 rustfmt drift remain open for their owners.
- Verification snapshot: `cargo test -p cli --test quran` (14 pass), `cargo test -p server` (22 pass), `cargo build -p cli` (ok), `cargo test -p application --test counting --test family_goldens --test morphology_import` (9 + 1 + 12 pass), `cargo test -p application --lib` (45 pass), `cargo clippy -p application -p server -p cli --all-targets -- -D warnings` (clean), `cargo fmt --check` on the touched crates (clean; only the pre-existing 03-01 drift remains), `xtask arch-check` (OK), `xtask migrate-check` (OK), and a live `qai serve` smoke of the three routes.

---
*Phase: 03-quran-search-linguistics*
*Completed: 2026-09-28*

## Self-Check: PASSED

- All 10 declared/created artifacts exist: `crates/application/src/quran_lexicon_api.rs`, `crates/cli/tests/quran/family_s1.trycmd`, `crates/cli/tests/quran/family_s2.trycmd`, `crates/application/src/quran_cli.rs`, `crates/application/src/lib.rs`, `crates/cli/src/quran.rs`, `crates/cli/src/lib.rs`, `crates/cli/tests/quran.rs`, `crates/server/src/api.rs`, `crates/server/tests/api.rs`.
- Both task commits exist: `cbd848e` (Task 1), `adf477f` (Task 2); `git rev-list --count 2d5878a..HEAD` = 2 (measured, matching `actuals.commits`).
- Verification commands re-run and green: `cargo test -p cli --test quran` (14 passed), `cargo test -p server` (22 passed), `cargo build -p cli` (ok), `cargo test -p application --test counting --test family_goldens --test morphology_import` (9/1/12 passed), `cargo clippy -p application -p server -p cli --all-targets -- -D warnings` (clean), `cargo fmt --check` on the touched crates (only pre-existing 03-01 drift), `xtask arch-check` (OK), `xtask migrate-check` (OK), live `qai serve` route smoke (404/400 with typed codes).
- Out-of-scope failures left alone and documented: `application --test quran_identity` (2 known `NotStaged` cases) and `cli --test doctor_json` (`audit_verify_…`, reproduced identically on the pre-plan base `2d5878a`).
