# Task Done Rollup

> Completed tasks across all phases. Newest first.

## Phase 2 — P2-T35 single-step index rollback, 2026-09-24

- `rollback_index_single_step` flips the serving pointer back to the newest
  `superseded` run below the serving generation with run states swapping in one
  transaction; the newer generation stays on disk (no ping-pong — never moves to
  a newer generation). Fail-closed `QAI-IDX-0008` (no previous) / `0009`
  (evicted dir); pointer manifest re-read from the retained `gen-<N>/manifest.json`.
  New `qai quran index rollback` verb (conflict exit 6 when nothing to undo).
- Evidence: `application --test index_lifecycle` 12/12 (4 new rollback tests:
  restore-and-serve, no-ping-pong, no-previous, evicted); `cli` suites green;
  `clippy -D warnings` clean; live CLI proof (`generation 2 → 1`, second
  rollback refused). Board 59/114 (52%).
- Remaining: T37 process-kill evidence, T38 full-corpus benchmark, and the
  owner/linguist/licensed-data gates listed in the phase board.

## Phase 2 — implementation reconciliation, 2026-09-24

- Reconciled the live task board with landed code: **58 / 114 tasks ☑ (51%)**;
  partial/synthetic/owner-gated work is explicitly ◐ rather than overstated.
- Newly closed task groups: T36/T54 (index/search hardening), T57/T58/T60/T61/T63/T66/T69/T70/T71
  (morphology import/validation/diff), T73 (edition-relative FTS lexicon projection), T75/T76/T77/T83/T86/T93
  (morphology/family core), and T94/T98/T99/T100/T103/T109 (counting/discovery core).
- Evidence: `cargo test -p quran-search --test trigram_index --test regex_dos --test fts5_backend`
  (12/12), `cargo test -p application --test morphology_import --test counting --test index_lifecycle --test search_goldens --test search_latency`
  (26/26), `cargo test -p application --test index_build --test index_lifecycle` (16/16),
  `cargo test -p cli --test quran quran_counting_graph_snapshots` (11 trycmd cases),
  `cargo test -p storage-sqlite --test quran` (10/10), and
  `cargo test -p quran-morphology -p quran-graph` (86/86). Synthetic fixtures
  do not close licensed-data, linguist, ADR-acceptance, API-parity, doctor/evaluation,
  or full-corpus exit gates; those remain ◐/☐ in the phase board.

## Cross-phase — Phase-4 board reconciliation + Phase-0 residual fixes, 2026-09-24

- **Phase-4 graph (code ahead of paperwork).** `crates/quran-graph` (2,883 lines)
  and `crates/quran-morphology` (2,534 lines) had landed while the Phase-4 board
  still read 0/30. Verified and reconciled honestly: TASK-401/403/404/409/413/414/416/420/424/427
  moved ☐ → ◐ — board total **0 ☑ / 10 ◐ / 3 ⊘** (pure crate/file-backed CLI groundwork
  over `MemGraphStore`; SQLite persistence, durable application lifecycle, HTTP/tools,
  doctor and visualization remain absent), M5 (TASK-405/406/407) stays ⊘ — the blocker is
  the missing licensed dataset + linguist goldens + graph root-family projection, not a
  placeholder crate. No task marked ☑; AC-P4-* all remain unverified.
- **P4-X03 / P4-X05 decision drafts landed.** ADR-0217 (graph query safety limits:
  budget defaults, validation ranges, "truncated ≠ empty" exhaustion semantics) and
  ADR-0218 (Graph JSON v1 mandatory/lossless; GraphML as a declared-lossy derived
  view) are written as **Proposed** — they record what `QueryBudgets` and
  `export.rs` already implement, so the owner ratifies a concrete proposal rather
  than an empty slot. Ratification is owner work; no status was flipped by an agent.
- **Conformance hardened to match ADR-0217** (TASK-409, still ◐): two new
  port-level tests in `crates/quran-graph/tests/conformance.rs` —
  out-of-range budgets are rejected at *every* port entry point (never clamped,
  never silently empty) and authorization is applied during pattern expansion.
  Suite 6 → 8 tests; `cargo test -p quran-graph` 36/36, clippy clean.
- **Export contract pinned to match ADR-0218** (TASK-427, still ◐): two new
  `export.rs` tests assert that an interpretive edge never leaves the exporter
  without its assertion + evidence record (source location, reviewer, decision,
  timestamp), and that a restricted assertion is absent from the *serialized
  bytes* — not merely dropped from the edge list. `cargo test -p quran-graph`
  38/38, clippy clean.
- **P0-T39/T40 scheduling follow-up closed on the code side.** `plus` keeps
  sub-second precision (`d.as_secs()` + `d.subsec_nanos()`); regression
  `queue::tests::plus_preserves_subsecond_delays`. SQLite scheduling validated by
  `recovery_jobs::rescheduled_job_is_not_claimable_until_due`. Real-time
  lease-recovery behaviour and the intermittent parallel `retries_then_succeeds`
  flake remain under watch.
- **Stale schema-version assertion fixed** (`storage-sqlite` `sqlite_database_health`
  hard-coded 16 while the live tree has 19 migrations — the test had been failing
  since the lexicon/morphology/graph migrations landed). It now derives the
  expected version from the migration tree, so appending a migration cannot break
  it again.
- **OTLP span-field scrubbing implemented** (deferred since P0-T16): new
  `observability::otlp::ScrubbingProcessor` strips denylisted + secret-named span
  attributes at the exporter boundary; `telemetry::is_scrubbed_span_field` is the
  single policy predicate. Tests green with `--features otlp`.
- **Docs hygiene.** Filled the 0-byte navigation files (`current-plan.md`,
  `master-plan.md`, `roadmap.md`, `milestones.md`, `phase-status.md`, `progress.md`,
  `milestone-status.md`, `docs/03-plan/backlog/*`, `unresolved-issues.md`), gave
  the server placeholder files explicit "not an accepted phase" content, and
  created `docs/04-tasks/{active,completed}/README.md` as an index over the phase
  boards. Closed FU-DOC-01, FU-SPEC-01 (stale — only T024 gate sweep remains), and
  FU-TEST-01 (DEBUG leftovers already gone; only a comment was rewritten).
- **P0-T56 remains ◐.** Docker daemon socket still absent (`docker info` fails
  2026-09-24) — container runtime verification and the clean-machine exit-gate
  ritual stay open; recorded in `docs/05-followups/open-questions.md`.
- Gates this session: `cargo test -p quran-graph` 38/38, `cargo test -p
  quran-morphology` 52/52, `cargo test -p jobs --lib` 22/22, `cargo test -p
  storage-sqlite` (all targets) green, `cargo test -p observability
  --features otlp` 14/14, `cargo test -p testkit --test secret_leak` 9/9; clippy
  `-D warnings` clean on all touched crates; fmt clean; `cargo xtask arch-check`
  OK; `cargo xtask migrate-check` OK (19 migrations); `cargo xtask adr-lint` OK;
  **`cargo test --workspace --no-fail-fast` — zero failures workspace-wide.**
- **One workspace-test finding, triaged as a harness issue, not a regression:**
  `application::quran_reader::lookup_performance_smoke` failed at 6.009 ms
  average (5.0 ms budget) while a second cargo build and another live session
  were competing for CPU. It then passed in isolation *and* in a full
  `cargo test --workspace --no-fail-fast` (zero failures workspace-wide),
  confirming the load hypothesis. Filed as FU-TEST-02 with a concrete fix
  (median/p95 or best-of-N instead of a single mean) — no reader behaviour may
  be relaxed.

## Phase 2 — P2-T13–T18/T20/T22 normalization engine close-out, 2026-09-24

- Review-pass flip of the M1a/M1b foundation (implementation landed in earlier sessions;
  tasks stayed ☐ pending verification): `RuleId` N01–N24 + trait (T13), `SpanMap`
  (T14), 5-property suite (T15), deterministic rules N01–N17 (T16), heuristic N18–N22
  with `RuleKind` tagging (T17), pipeline + append-only L0–L8 registry (T18),
  `NormalizationTrace` empty-label guard (T20), idempotency/associativity/fuzz (T22).
- Evidence: `cargo test -p quran-normalization` 89/89 (64 lib + 7 deterministic +
  3 golden + 6 pipeline + 2 vectors + 7 spanmap); `clippy --all-targets -D warnings`
  clean (repaired 3 lints in test/example code, no assertions changed; re-ran green);
  `fmt --check` clean. Fixture `pairs.jsonl` holds header + 2,000 pairs.
- Remaining in Sprint 2.1: T21 stays ◐ — harness runs all 2,000 pairs green but the
  fixture is `reviewed_by: pending-linguist` (P2-T11/P2-X02). T39/T56 blocked on owner
  ADR acceptance (0201/0208/0213, 0207/0212/0214 all Proposed). Board now 34/114 (30%).
  Working tree also carries an unrelated in-progress morphology workstream (untouched).

## Phase 2 — P2-T51/T52 search API + CLI surfaces, 2026-09-23

- `POST /api/v1/quran/search/{exact,normalized,phrase,concatenated,regex}` serve the
  read-only M3 services in the stable envelope (Diagnostic bodies, `Content-Language: ar`);
  `Accept: text/event-stream` streams `hit` events plus a terminal `totals` event with the
  exact total and reproducibility block. Regex rate-limiting is per `x-principal`.
- `qai quran search <text>` covers all five tools with profile/rules, phrase, cross-ayah,
  filter, paging, `--explain`, `--highlight`, and `--json` support; tool flags are exclusive.
- Evidence: server api suite 17/17 (4 new), CLI quran suite 4/4 (new `search.trycmd`,
  13 blocks), application `search_tools` 14/14 + `search_cache` 6/6 unchanged;
  workspace fmt/clippy `-D warnings`, arch-check, migrate-check green. Full-workspace
  `cargo test` exceeds the 15-minute command timeout (pre-existing suite size; targeted
  crates all green).
- Remaining: T53–T56 (goldens, DoS suite, latency gates, ADRs), entire M4–M7,
  morphology/root endpoints (no datasets yet), true incremental SSE streaming.

## Phase 0 — FU-10/DEV-02 catalog CLI dispatch, 2026-09-18

- Real read-only dispatch for `source list/show`, `job list/show`, `audit list`,
  `secret list` (refs only), and static shell `completions`; guarded mutations
  refuse with exit 2. Fixed nested-JSON secret redaction in job payload columns.
- Evidence: 2 new application catalog tests + 2 new CLI catalog tests; full
  `cli` + `application` suites green; workspace clippy/fmt/check, arch-check,
  and migrate-check green.
- Remaining: T56 runtime (daemon unavailable), production scheduling validation,
  and human/external gates. No commit made.

## Phase 0 — P0-T50 audit verification subtask, 2026-09-17

- `audit verify` now checks persisted hashes, links, and gaps read-only, with human/JSON output and failing exit codes for corruption/errors.
- CLI regression verifies a real imported chain, then corrupts its hash and checks failure, no database mutation and no sensitive output. Application regression covers payload/link tampering and gaps.
- Full CLI suite (20 tests), focused application tests, workspace fmt/clippy/check, architecture and migration checks passed. Other CLI stubs and full exit gates remain open; T56 runtime deferred on missing Docker daemon.

## Phase 0 — P0-T39/T40 timestamp ordering, 2026-09-17

- In-memory claims and expired-lease recovery compare parsed RFC3339 instants; malformed timestamps are ineligible. Private fixed-time entry points exercise the same implementations without sleeps.
- Mixed precision, equality, future timestamps, invalid timestamps, and state transitions covered by two regression tests.
- Verified: 25 consecutive `cargo test -q -p jobs` runs (21 passing each), focused exact tests, jobs formatting, clippy all-targets with denied warnings, and check.
- Remaining: subsecond-delay truncation, SQLite scheduling validation, and full Phase-0 gates. No commit made.

## Phase 0 — P0-T57 deterministic fixtures, 2026-09-17

- Completed FU-06: controllable `FixtureClock`, typed `fixture_id`, and `sample_job_record_at` in `crates/testkit/src/lib.rs`; existing random fixture API unchanged.
- Three regression tests; full testkit suite 40 passing. Testkit fmt, clippy all-targets with denied warnings, and type check passed.
- Jobs scheduling follow-up remains open; these helpers do not change production clocks.

## Phase 0 — targeted retry hardening, 2026-09-17

- P0-T39/T40: restored failure-path idempotency enforcement and post-jitter delay cap in `crates/jobs/src/worker.rs`, with two regression tests.
- Verified: fully qualified exact backoff test (1 test); `cargo test -p jobs -- --test-threads=1` (19 tests); `cargo fmt -p jobs -- --check`; `cargo clippy -p jobs --all-targets -- -D warnings`; `cargo check -p jobs`.
- Initial parallel jobs run: 18 passed, `retries_then_succeeds` returned Idle; isolated and serial reruns passed. Follow-up remains open. No workspace or phase-exit verification claimed; T57/T56/CLI work not completed.

## Phase 1 — Canonical Quran Core (in progress)

### Session: 2026-09-17 — P1-T58 fixture soak / P1-T60 gate preparation

- P1-T58 remains partial: added `quran_doctor.rs::fixture_soak_ten_thousand_lookups_preserves_corpus_integrity`; existing harness imports, validates, and activates the synthetic fixture, then the test performs 10,000 seeded lookups, verifies exact text and token offsets, and requires post-soak hashes and round-trip checks to pass. A licensed standard-edition soak is still pending.
- P1-T60 preparation: `cargo test -p application -- --test-threads=1` (110 tests), workspace fmt/clippy/check, architecture check, and migration check passed. Initial parallel application run timed out; serial rerun passed. No full-workspace test, coverage, dependency-audit, or reviewer ritual claim.
- No dataset, reference corpus, font-license choice, or editorial approval was invented; T26 remains unchanged and partial.

### Session: 2026-09-15 — M9a/M9b tools, citations, API, CLI, doctor

**Tools, citations, API (`crates/tools`, `tool-registry`, `citations`, `server`)**
- P1-T43/T44/T45 — `ToolResult`/`ReproducibilityData` contract + `ToolRegistry`
  (`quran.get_ayah`, `quran.get_context`) with typed no-fabrication errors; fixed a
  flaky equality test on nondeterministic `execution_time_ms`.
- P1-T46/T47 — citation resolver (`resolve`/`resolve_stored`/`StoredCitation`) +
  deep links; `GET /api/v1/quran/…` + `/debug/read/{edition}/{surah}`.
- P1-T39/T40 — axum API v1 with envelope/ETag/`Content-Language`/Diagnostic error
  body; `docs/08-api/quran-v1-openapi.json` + route-coverage test.

**CLI + doctor (`crates/cli`, `application::quran_cli`)**
- P1-T48/T49 — read verbs (`get/context/surah/division/resolve`) and lifecycle verbs
  (`import/validate/activate/rollback/diff/edition/translation`), every read command
  `--json`, destructive verbs gated on `--yes`, Phase-0 exit-code table.
- P1-T50 — `cli/tests/quran/read_flow.trycmd` (trycmd): migrate → import → activate →
  RTL read + provenance → `surah`/`context`/`division`/`resolve` → attributed translations
  → import v2 → `validate` → `diff` → `rollback` → `hashes` → error exits 5/6.
- P1-T51/T52 — `doctor --quran` 19 checks + `--deep`, read-only DB open; `--quran --json`
  now emits one merged `{"checks":[...]}` document (45 checks) that passes
  `cargo xtask validate` against `docs/schemas/doctor.v1.schema.json`
  (regression-guarded by `cli/tests/doctor_json.rs`; previously two concatenated docs).
- Fixed three end-to-end defects: `QAI_DATA_DIR` ignored by config resolution,
  translation import violating the `provenance_records` FK, and doubled trailing
  newline in CLI human output.

**Hygiene**
- Cleared `-D warnings` clippy regressions exposed by the toolchain: `unreachable_else`
  (`quran_import.rs`), collapsible `if` (server `api.rs` + test), items-after-test-module
  (`server/api.rs`, `cli/doctor.rs`); `application/src/db.rs` schema-version assertions
  now derive the version from `migrations/sqlite/` instead of hard-coding it.

**Docs (D1.14)**
- P1-T59 — five published documents: `docs/07-technical/quran-corpus-architecture.md`,
  `quran-adapter-authoring.md`, `quran-citation-spec.md`; and
  `docs/10-operations/quran-import-runbook.md`, `quran-rollback-runbook.md`.

### Session: 2026-09-14 — M7 importer + M8 reader

**Importer (`quran-corpus::import`, `application::quran`)**
- P1-T25 — 13-checkpoint `quran.import` job (deterministic restart-is-resume,
  cancellation with staging cleanup, `stop_after` dry-run/chaos support).
- P1-T27 — char-level edition differ (`similar`), persisted per import.
- P1-T28 — approval-gated activation/rollback services (granted approval +
  subject match + same-tx audit); importer holds no token (I5/I7).
- P1-T29 — 13-prefix crash matrix (active untouched) + cancellation test.
- P1-T26 — round-trip verifier (QV-014/QV-024 fail-closed); QV-015 recorded skip.
- First real hash-chained audit writer app-wide (`application::audit_bridge`).
- AC-P1-02/03/10/11 automated-green (rituals pending).

**Reader (`application::quran_reader`)**
- P1-T32/T33/T34 — `QuranReader` trait + service: ayah/range/surah/division/token
  expansion, structure-bounded context with caps, typed errors.
- P1-T35 — generation-keyed `lru` cache + no-stale-text test (AC-P1-19 partial).
- P1-T37 — `AyahView`/`AttributedTranslation` principle-5 types (translations and
  gloss import stay in M9).
- P1-T41 — lookup performance smoke (2000 warm reads, < 5 ms avg budget).
- P1-T42 — ADR-0113 Accepted; ADR-0112 stays Draft for M9.

### Verified
- `cargo test --workspace`: 387 passing, 0 failing.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `cargo fmt --all`: clean.
- `cargo run -p xtask -- arch-check` / `migrate-check`: OK.



### Session: 2026-09-14 — execution plan + M0/M1

**Plan**
- `docs/03-plan/phases/phase-01-core/execution-plan.md` written (M0–M10, files/deps/
  allowlist/tests per increment, blocker fallbacks B1–B5, risk mitigations).
- Owner decisions flagged in `done.md` §7 (OWN-01 dataset, OWN-02 reference corpus,
  OWN-03 estimate gap, OWN-04 axum provisional, OWN-05 Phase-0 exit discrepancy).

**quran-core (new crate)**
- P1-T06 — newtypes, enums, slug grammar, grapheme-count helper.
- P1-T07 — edition/surah/ayah/segment/token structs (+ per-row `provenance`, matching
  the authoritative DDL/QV-025).
- P1-T10 — `QuranQuotation` + I6/principle-5 constructor guards.
- P1-T08/T09 — hand-written reference parser + serializer; 331-case golden set
  (`fixtures/quran/golden/references.jsonl`); round-trip + never-panics proptests.
  AC-P1-12/13 automated-green (exit ritual pending).
- `cargo test -p quran-core`: 29 unit + 6 integration passing. Pinned deps respect I2
  (`domain, serde, thiserror, unicode-segmentation` only).

**quran-corpus (new crate)**
- P1-T15 — `qai.quran.edition` v1 types + JSON Schema doc.
- P1-T16/T17 — `EditionAdapter` trait with JSON and CSV adapters; CSV reproduces the
  JSON manifest's ayahs exactly.
- P1-T05 — synthetic `test-edition-min` + 16 adversarial fixtures (all schema-valid;
  targeted at specific QV ids in M5).
- P1-T21 — Unicode auditor (normalization form, forbidden points, expected blocks).
- P1-T22 — whitespace-preserving tokenizer + exact separators + grapheme/byte
  offsets, with a losslessness proptest.
- P1-T23 — frozen `text_hash` / `structure_hash` / `token_order_hash` recipes.
- P1-T18 — `quran_edition_v1` registered in a new (additive) `sources::ValidatorRegistry`;
  `QuranEditionValidator` bridges registry calls to the detailed validator.
- P1-T19/T20 — all QV-001…QV-028 implemented (content rules in `validate_edition`;
  pipeline-state rules split honestly: QV-013/014 helpers now, QV-015 skip-if-unconfigured,
  QV-024/025/026 with the importer, QV-028 vacuous in v1).
- P1-T30 — 16/16 adversarial fixtures rejected with their specific rule ids.
  AC-P1-04 automated-green (exit ritual pending).
- P1-T12/T24 — Quran migrations `0007`–`0012` with insert-only triggers and checksums.
- P1-T13 — `QuranRepository` plus SQLite staging, activation/rollback, canonical reads,
  reports, citations, and translations.
- P1-T14 — raw-SQL trigger suite plus staging/activation/rollback behavior. AC-P1-08
  automated-green (exit ritual pending).

**Tooling**
- M0 — allowlist entries for `quran-core`, `quran-corpus`, `citations`, `tools`,
  `tool-registry`; extended `storage-sqlite`, `application`. `arch-check` converted to a
  name-keyed map (fail-closed preserved) with a new regression test.

### Verified
- `cargo test --workspace`: 291 passing, 0 failing.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `cargo fmt --all`: clean.
- `cargo run -p xtask -- arch-check`: OK.
- `cargo run -p xtask -- migrate-check`: OK.

## Phase 0 — Foundations & Provenance

### Session: Phase 0 completion push

**Storage (`storage-sqlite`)**
- T18 — Dual SQLite pools (write `max_connections=1`, read `query_only`), WAL,
  `foreign_keys=ON`, `busy_timeout`. `SqliteDatabase::open_read_only` for doctor.
- T19 — Migration runner: `apply_migrations` (append-only, checksummed, idempotent),
  `verify_checksums`, `backup` via `VACUUM INTO` (never `fs::copy`).
- T17/T23/T26/T36 (partial) — Real repository implementations for sources, provenance,
  audit, jobs, settings over a shared `Arc<Mutex<Transaction>>`; commit atomically persists
  every repo's writes. Added `SourceRepository::insert_source`.
- Migration checksums manifest `migrations/sqlite/checksums.json` (T19 CI gate).

**Domain / security (`domain`)**
- T43 — `security_archive`: zip-slip, symlink-entry, bomb, entry-count, nested-depth guards.
- T44 — `security_net`: resolve-then-check SSRF guard + domain allowlist (fail-closed).
- T45 — `security_input` (field length, control chars, JSON depth) and `security_sanitize`
  (scripts, event handlers, dangerous URI schemes).

**CLI (`cli`)**
- T48/T49/T50 — `qai` binary entry point (`src/main.rs`), `db` command group wired to real
  migrations (migrate/status/verify/plan/backup/restore), config-aware loading.
- T52/T53/T54 — Doctor engine with a 26-check registry (config, DB, secrets, jobs, audit,
  sources, filesystem, security, observability, outbox/tombstones), read-only DB probe,
  remedies + next commands on every non-pass, and `--repair-preview` (no mutation).

**Application (`application`)**
- Orchestration/composition root: `run()` bootstrap, `db` module (migrate/status/verify/
  backup/probe) keeping `storage-sqlite` behind the application boundary.

**Testkit (`testkit`) — D0.16**
- T57 — Fixtures (temp dir/db, sample config/TOML/source version/job/provenance) and
  `MockAuditRepo`.
- Integration suites: `secret_leak`, `security_path_guard` (40+ payloads),
  `security_archive_guard`, `security_ssrf_guard`, `config_precedence`.

**Tooling**
- `xtask/allowlist.toml` — reconciled allowlist with actual dependency edges (testkit,
  application composition root, `storage-sqlite` serde rename fix).

**Docs — D0.17**
- T58 — 5 runbooks (`docs/runbooks/`), `CONTRIBUTING.md` with the DoD checklist, error-code
  registry rules, secret-redaction requirements, and append-only migration policy.

### Verified
- `cargo test --workspace`: 139 passing, 0 failing.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `cargo fmt --all -- --check`: clean.
- `cargo run -p xtask -- arch-check`: OK.
- `cargo run -p xtask -- migrate-check`: OK.
- `qai db migrate | status --json | verify | backup` and `qai doctor [--json|--repair-preview]`
  smoke-tested against a live SQLite database.

### Still open (not claimed complete)
- True `SIGKILL` job chaos suite (AC-P0-11) and 2-second cancellation timing (AC-P0-12).
- Full audit chain recomputation verifier (`tests/integrity/audit_chain.rs`) beyond the
  structural chain check.
- Outbox/generations/tombstones domain types + allocators (T61–T67) and consistency suites.
- Keychain/age-encrypted secret backends.
- Coverage gates (AC-P0-22) and the recorded exit-gate ritual.
