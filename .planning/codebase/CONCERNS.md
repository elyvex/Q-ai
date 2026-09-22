---
last_mapped_commit: 68b7ea6c1aacd1476d154550b2a6e9574614b6af
last_mapped_at: 2026-09-22
---
# Codebase Concerns

**Analysis Date:** 2026-09-22

## Tech Debt

**Flaky job scheduler — `plus()` discards subsecond delays:**

- Issue: `crates/jobs/src/queue.rs:76` builds `available_at` via `time::Duration::seconds(d.as_secs() as i64)`, truncating any sub-second component to whole seconds. Zero-delay reschedules land on the same RFC3339 second as `now`, so `claim_next_at` ordering becomes a same-second tie. `docs/05-followups/open-questions.md` records `cargo test -p jobs` intermittently returning `Idle` instead of `Succeeded` in `worker::tests::retries_then_succeeds` (`crates/jobs/src/worker.rs:397`); isolated/serial runs pass. Timestamp *comparison* was fixed to parse instants (`rfc3339_le` at `crates/jobs/src/queue.rs:81`), but the *generation* side still loses precision.
- Files: `crates/jobs/src/queue.rs`, `crates/jobs/src/worker.rs`
- Impact: Non-deterministic scheduling at sub-second granularity; parallel `cargo test -p jobs` flakes; production SQLite scheduling and wall-clock behavior still unvalidated (production path does not use `InMemoryJobQueue`).
- Fix approach: Preserve sub-second precision in `plus()` (format with nanoseconds — RFC3339 already supports it — or store `unix_timestamp_nanos` alongside the string). Add a regression test that reschedules with 0/50/200 ms delays and asserts strict `available_at` ordering. Validate the SQLite-backed queue path separately, plus lease-recovery retry policy.

**OTLP span-field redaction gap:**

- Issue: The global tracing redaction layer (`RedactingWriter`, Rule A on field names in `crates/domain/src/redaction.rs`, wired in `crates/observability/src/lib.rs`) scrubs stderr output only. The OTLP exporter pipeline (`crates/observability/src/otlp.rs:28-50`) installs a separate `tracing_opentelemetry` layer + batch exporter that bypasses stderr, so span fields matching secret keys reach the OTLP backend unscrubbed.
- Files: `crates/observability/src/otlp.rs`, `crates/observability/src/lib.rs`, `crates/observability/src/telemetry.rs`, `crates/domain/src/redaction.rs`
- Impact: Opt-in telemetry export can leak secret-shaped span attributes to a remote collector — the one path where data leaves the local-first boundary.
- Fix approach: Add span-field scrubbing at the exporter boundary when `redact_secrets=true` (a filtering span processor or OTLP-specific `FormatFields` hook); keep the content-field denylist (`is_forbidden_field` in `crates/observability/src/telemetry.rs:16`) gating export payloads as the interim mitigation. Cover with unit tests using scrub fixtures. Recorded as Phase 3+ follow-up in `docs/05-followups/open-questions.md`.

**FTS5 vs Tantivy drift — enum promises an engine that does not exist:**

- Issue: `crates/quran-search/src/model.rs:23-24` defines a `FtsBackend::Tantivy` variant and `crates/quran-search/src/index.rs`, `crates/quran-search/src/lib.rs:4` describe "FTS5 now, Tantivy later", but no Tantivy adapter exists, no `tantivy` dependency is declared in any `Cargo.toml`, and ADR-0201's Tantivy decision is still `Proposed`. Specs repeatedly forbid assuming Tantivy (`specs/030-search-service-wiring/spec.md`, `specs/006-quran-corpus/spec.md`, `specs/007-quran-normalization/spec.md`). Any code path that constructs or matches `FtsBackend::Tantivy` is a dead/phantom branch.
- Files: `crates/quran-search/src/model.rs`, `crates/quran-search/src/index.rs`, `crates/quran-search/src/lib.rs`, `crates/quran-search/src/fts5.rs`
- Impact: Contributors can write backend-agnostic code against an engine that will fail at runtime; spec/code vocabulary drift.
- Fix approach: Either gate `FtsBackend::Tantivy` behind an explicit `unimplemented!` error with code `QAI-IDX-*` and a test asserting it fails closed, or remove the variant until the ADR is accepted. Do not add the dependency until ADR-0201 is ratified.

**Migration numbering / reversibility drift:**

- Issue: (a) Phase-0 migrations `0001`–`0006` ship `.down.sql` files; Phase-1/2 migrations `0007_quran_editions` through `0016_quran_search_cache` in `migrations/sqlite/` are up-only with no down migration. (b) `docs/03-plan/phases/phase-01-core/STATUS.md` notes the workspace is at 16 migrations while Phase-1 documents claim "6/6 (`0007`–`0012`)" — Phase-2 additions `0013`–`0016` are invisible to Phase-1 ledgers. (c) One spec checklist notes trigger-code drift (`0003` not `0001`) as "code truth" (`specs/007-quran-normalization/checklists/requirements.md:34`).
- Files: `migrations/sqlite/0007_quran_editions.up.sql` through `migrations/sqlite/0016_quran_search_cache.up.sql`, `migrations/sqlite/checksums.json`, `crates/storage-sqlite/src/migrate.rs`
- Impact: Forward-only is intentional per invariant I7, but the asymmetry (some downs exist, later ones do not) confuses rollback expectations; stale phase ledgers misreport schema state; checksum drift detection (`verify_checksums` at `crates/storage-sqlite/src/migrate.rs:230`) only helps if `checksums.json` is regenerated on every migration edit.
- Fix approach: Document forward-only as policy in `migrations/sqlite/` (README or header comment) and either remove the Phase-0 `.down.sql` files or mark them legacy/non-executed. Refresh `STATUS.md` migration counts when Phase-2 lands migrations. Keep `cargo run -p xtask -- migrate-check` green on every migration PR.

**Concurrent-writer hazard (SQLite multi-process):**

- Issue: `crates/storage-sqlite/src/lib.rs:60-100` correctly uses WAL + `synchronous=FULL` + 5 s busy timeout with a single-connection write pool (`max_connections(1)`). But SQLite still serializes writers at the database level: a second `qai` process (e.g. CLI import running while the server holds the write pool, or two CLI invocations) blocks to the busy-timeout and then fails with `StorageUnavailable`. There is no cross-process lock, queue, or single-writer-sidecar. The `generation_monotonicity` test (`crates/storage-sqlite/tests/generation_monotonicity.rs`) covers 50 concurrent *tasks* against one pool, not two processes. `STATUS.md` explicitly warns a concurrent Phase-2 writer was active in the same tree during the snapshot.
- Files: `crates/storage-sqlite/src/lib.rs`, `crates/storage-sqlite/tests/generation_monotonicity.rs`, `crates/config/src/lib.rs:65`
- Impact: `SQLITE_BUSY` failures under concurrent CLI+server use; generation allocation races across processes; developer-tree collisions (already observed once — see FU-COORD-01).
- Fix approach: Document single-writer discipline in the import/runbook docs; surface a clear "database is locked by another writer, retry" diagnostic instead of bare `StorageUnavailable`; long-term consider a writer queue or file-lock guard. Keep the FU-COORD-01 precedent: implementation owner edits, second session verifies read-only.

**Oversized god-modules:**

- Issue: Three files exceed 1600 lines and concentrate unrelated responsibilities: `crates/application/src/quran_search.rs` (2150 lines — query parsing, caching, tools, counting, explanation), `crates/storage-sqlite/src/quran.rs` (1885 lines — staging, activation, reads, reports, citations, translations), `crates/application/src/quran_cli.rs` (1815 lines). `quran_search.rs` was already rewritten mid-session (2146 → 2052 lines) during a parallel-session collision (FU-COORD-01), proving churn risk.
- Files: `crates/application/src/quran_search.rs`, `crates/storage-sqlite/src/quran.rs`, `crates/application/src/quran_cli.rs`, `crates/storage-sqlite/src/lib.rs` (1626 lines)
- Impact: Merge-collision magnet; hard to review; blast radius of any search/storage change is the whole file.
- Fix approach: Split `quran_search.rs` along its existing module seams (query model / cache / tools / explain) without behavior change; split `storage-sqlite/src/quran.rs` into staging/reads/reports submodules. Enforce new-code placement in the split modules.

**`server` layering allowlist (accepted but load-bearing):**

- Issue: `xtask/allowlist.toml:39-44` explicitly allows `server` to depend on `storage` + `tools` directly instead of routing through `application`. The comment admits this is a "known layering" issue deferred to Phase 3 (OD-14, unanswered). `cargo run -p xtask -- arch-check` enforces the allowlist, so the violation is fenced but not fixed.
- Files: `xtask/allowlist.toml`, `crates/server/src/api.rs`, `crates/server/src/lib.rs`
- Impact: HTTP layer can bypass application-service invariants (approval gates, audit bridging, generation-keyed cache invalidation) by touching the DB handle directly.
- Fix approach: In Phase-3 server hardening, route all Quran reads through `application` services and tighten the allowlist to remove `storage` from `server`. Until then, forbid new `server → storage` call sites in review.

**Spec/code paperwork drift:**

- Issue: `specs/001-redaction-hardening/tasks.md` shows 28/28 checkboxes unchecked while the implementation (`domain::redaction`, `RedactingWriter`, `InitOptions`, CLI redact-then-print) is committed and its 9 sentinel tests pass (`cargo test -p testkit --test secret_leak` 9/9). Code is ahead of its spec paperwork (FU-SPEC-01). The reverse also occurs: `specs/006-quran-corpus/spec.md` disclaims "plan-text drift" re Tantivy.
- Files: `specs/001-redaction-hardening/tasks.md`, `crates/testkit/tests/secret_leak.rs`, `crates/domain/src/redaction.rs`
- Impact: Auditors cannot trust checkbox state; acceptance-evidence mapping rots.
- Fix approach: Owning session flips each checkbox with landing commit or records a deviation note; then marks P0-T16 evidence complete in `docs/03-plan/phases/phase-00-foundation/tasks.md` and `done.md`. No code changes.

**Docs scaffolding rot:**

- Issue: `docs/03-plan/current-plan.md` is 0 bytes yet `AGENTS.md` ("Before starting work" step 3) and `README.md` both instruct agents to read it first (FU-DOC-01). `docs/04-tasks/{active,completed}/` required by `AGENTS.md` §2 and `.agent/instructions.md` §3 does not exist — work is tracked in `docs/03-plan/phases/*/tasks.md` instead (FU-DOC-02).
- Files: `docs/03-plan/current-plan.md`, `AGENTS.md`, `README.md`, `.agent/instructions.md`
- Impact: Every new session pays rediscovery cost; mixed task-tracking conventions will rot.
- Fix approach: Fill `current-plan.md` with a one-paragraph active-phase pointer or delete it and remove both references; either create `docs/04-tasks/` or update the workflow docs to name phase `tasks.md` files as the system of record. Decide once.

## Known Bugs

**Flaky `jobs::worker::retries_then_succeeds` under parallelism:**

- Symptoms: `cargo test -p jobs` intermittently reports `Idle` instead of `Succeeded` after zero-delay rescheduling; isolated (`--test-threads=1`) and serial reruns pass. 25 consecutive parallel jobs-suite runs passed after the timestamp-comparison fix, but the root cause (`plus()` truncation, above) is unfixed so the flake can recur.
- Files: `crates/jobs/src/worker.rs:397`, `crates/jobs/src/queue.rs:76-90`
- Trigger: Parallel test execution with zero/sub-second reschedule delays (`cargo test -p jobs` default threading).
- Workaround: Run `cargo test -p jobs -- --test-threads=1` for a green signal; do not close P1-T60 on parallel-only evidence.

**Doctor-suite timeout under parallel test threads:**

- Symptoms: An initial parallel `cargo test -p application` run timed out during doctor tests (`crates/application/tests/quran_doctor.rs`), while isolated and serial reruns passed (110 tests green with `--test-threads=1`). Recorded in FU-P1-02.
- Files: `crates/application/tests/quran_doctor.rs`
- Trigger: Full parallel application suite including the 10,000-lookup fixture soak.
- Workaround: Serial invocation for doctor/soeak tests; investigate reproducibility rather than claiming the serial pass as a fix. P1-T60 remains open.

**DEBUG leftovers in `secret_leak.rs` (working-tree, not landed):**

- Symptoms: Three `println!("DEBUG …")` lines in `sentinel_key_value_pairs_scrubbed_from_free_text` plus a missing trailing newline at EOF; `cargo fmt --all -- --check` flags the EOF newline.
- Files: `crates/testkit/tests/secret_leak.rs`
- Trigger: Committing the working tree as-is lands noise on `main`.
- Workaround: Delete the three DEBUG lines, run `cargo fmt -p testkit`, re-run the suite (FU-TEST-01).

**Container verification unverified (not a code bug — an evidence gap):**

- Symptoms: `Dockerfile`, `docker-compose.yml`, `.dockerignore` exist and compose validation passed, but the Docker daemon socket was unavailable at check time (rechecked 2026-09-18), so image build, shared-library compatibility, named-volume ownership, and non-root (UID 65532) execution were never exercised.
- Files: `Dockerfile`, `docker-compose.yml`, `.dockerignore`
- Trigger: Any claim that "container deployment works" before `docker compose build`, `docker compose run --rm app db migrate`, `db verify`, and in-namespace `/healthz`, `/readyz`, `/api/v1/meta` checks pass.
- Workaround: None — run the recorded sequence once a daemon is available before closing the container task. Note the stub intentionally uses `network_mode: none` with no published ports (loopback-only server); that is design, not a bug.

## Security Considerations

**OTLP export bypasses the redaction layer:**

- Risk: Secret-shaped span attributes exfiltrate to a remote OTLP collector.
- Files: `crates/observability/src/otlp.rs`, `crates/observability/src/telemetry.rs`, `crates/domain/src/redaction.rs`
- Current mitigation: Telemetry off by default (opt-in only, PRD §39); `is_forbidden_field` denylist gates export payloads (AC-P0-18); `redact_secrets=true` default in `InitOptions` (`crates/observability/src/lib.rs:44`).
- Recommendations: Exporter-boundary scrubber (see Tech Debt); attribute-key allowlist (deny-by-default); never put Arabic text, user strings, or query payloads in span attributes; hash file paths; scrub-fixture unit tests. Pending owner ratification (`docs/05-followups/open-questions.md` § Telemetry & Redaction).

**Secret backends — env-only in practice:**

- Risk: `crates/config/src/secret_store.rs` defines `EnvSecretStore`, `EncryptedFileSecretStore` (XChaCha20-Poly1305 JSON), and `KeychainSecretStore` behind one trait, but Phase-0 exit notes "secret backends beyond env (keychain, age-encrypted file)" as remaining work. Operators fall back to plaintext env vars (`QAI_SECRET_<KEY>`) with shell-history / process-table exposure.
- Files: `crates/config/src/secret_store.rs`, `crates/config/src/secret.rs`, `crates/config/src/lib.rs`
- Current mitigation: `Secret<T>` wrapper with debug-redaction; `secret://<backend>/<path>` reference parsing that fails closed.
- Recommendations: Finish and document the encrypted-file backend first (cross-platform), then keychain; add a `qai doctor` check warning when secrets resolve from env in server mode.

**No model fabrication is architecturally enforced, but only as far as the constructor discipline holds:**

- Risk: A new surface that builds `QuranQuotation` (or renders ayah text) without the visibility-restricted constructor in `crates/quran-core` could present generated text as quotation.
- Files: `crates/quran-core/src/` (`QuranQuotation`), `crates/citations/src/lib.rs`, `crates/server/src/api.rs`, `crates/cli/src/lib.rs`
- Current mitigation: Restricted constructor (invariant I6); citation `QuotationVerdict` with `Mismatch`/`LocationNotFound` variants; `arch-check` layering gate; golden `fixtures/quran/golden/ayah_texts.jsonl` (331 cases) byte-tied to the fixture.
- Recommendations: Keep routing new read surfaces through `application::QuranReader`; never fabricate scripture in fixtures (P1-T56 rule); require citation-verdict coverage on any new quote-rendering endpoint.

## Performance Bottlenecks

**Fixture soak is synthetic — real-corpus performance unknown:**

- Problem: `fixture_soak_ten_thousand_lookups_preserves_corpus_integrity` (`crates/application/tests/quran_doctor.rs`) proves 10k seeded lookups preserve integrity on `test-edition-min`, but says nothing about standard-edition latency, deep-scan duration, or editorial correctness (FU-P1-01).
- Files: `crates/application/tests/quran_doctor.rs`, `crates/storage-sqlite/src/quran.rs`
- Cause: Approved dataset blocked on owner decisions OD-01/OD-02/OD-03 (dataset, reviewer, reference corpus all `_unassigned_`).
- Improvement path: Once ADR-0101 supplies an approved edition, adapt the harness to accept it and record corpus identity, hashes, lookup results, and deep-scan duration. P1-T58 stays partial until then.

**FTS5 index build / large-result search unprofiled:**

- Problem: No benchmarks exist for index-build time, query latency on large result sets, or the `quran_search` concatenation-window path over the full corpus. The largest application file (`crates/application/src/quran_search.rs`, 2150 lines) does per-hit `to_string()` rendering for diagnostics/remedies and holds `HashMap`-based intermediate state on the hot path.
- Files: `crates/application/src/quran_search.rs`, `crates/quran-search/src/fts5.rs` (747 lines), `crates/quran-search/src/tokenizer.rs`, `migrations/sqlite/0015_quran_indexes.up.sql`, `migrations/sqlite/0016_quran_search_cache.up.sql`
- Cause: Optimization deferred; corpus small enough that nothing hurts yet.
- Improvement path: Add a criterion-style bench for index build + representative queries once the approved edition lands; cache diagnostic strings or defer rendering until the error path; verify `0015`/`0016` indexes cover the actual query shapes (`EXPLAIN QUERY PLAN` in review).

**Generation-keyed reader cache invalidation cost:**

- Problem: `QuranReader`'s generation-keyed cache (`crates/application/src/quran_reader.rs`, 994 lines; cache logic in `crates/application/src/quran_search_cache.rs`) avoids stale text after activation, but every activation bumps the generation and cold-starts dependent caches (search cache table `0016_quran_search_cache` included).
- Files: `crates/application/src/quran_reader.rs`, `crates/application/src/quran_search_cache.rs`
- Cause: Correctness-first design (no stale text ever) with coarse invalidation.
- Improvement path: Measure before optimizing; if activation-adjacent latency matters, move to per-edition generation keys. Current behavior is correct — do not weaken it for speed.

## Fragile Areas

**`crates/application/src/quran_search.rs` (2150 lines):**

- Files: `crates/application/src/quran_search.rs`, `crates/application/tests/search_tools.rs` (918 lines)
- Why fragile: Five responsibilities in one file; already rewritten once mid-session by a colliding writer; tight-hull span assertions (`0..6` vs `== 7`) show how brittle window expectations are.
- Safe modification: Single-owner edits only (FU-COORD-01 precedent); run `search_tools` 14/14 + application lib suites + `quran-search` suites + `cargo fmt` before handing off; prefer adding a focused submodule over extending the file.
- Test coverage: Good for shipped paths (`search_tools` 14/14, application lib 28/28) but Tantivy branches (if any match on the enum) are untested by construction.

**`crates/storage-sqlite/src/quran.rs` (1885 lines) + migration chain `0007`–`0016`:**

- Files: `crates/storage-sqlite/src/quran.rs`, `crates/storage-sqlite/src/lib.rs`, `migrations/sqlite/0007_quran_editions.up.sql` … `migrations/sqlite/0016_quran_search_cache.up.sql`
- Why fragile: Canonical tables are insert-only via triggers — a bad migration cannot be rolled back (no down files) and checksum verification aborts forward progress on drift. Touching triggers or activation logic risks bricking existing databases.
- Safe modification: New migrations only, never edit an applied one (checksum gate will fail the build); test activation/rollback paths in `crates/storage-sqlite/tests/quran.rs` and `integrity_provenance.rs`; run `cargo run -p xtask -- migrate-check`.
- Test coverage: Strong (`quran.rs` tests, integrity/provenance tests, generation monotonicity) but single-process only.

**Import pipeline `crates/quran-corpus/src/import.rs` (1379 lines, 13 checkpoints):**

- Files: `crates/quran-corpus/src/import.rs`, `crates/quran-corpus/src/validation.rs` (688 lines), `crates/application/tests/quran_import.rs` (819 lines)
- Why fragile: Restart-is-resume + cancellation-cleanup + reference comparison (`compare_reference`, QV-015 fail-closed) interact; QV-001…028 validators each carry adversarial fixtures that must keep rejecting with the specific rule id.
- Safe modification: Change one validator at a time; keep the 16 adversarial fixtures green; never loosen `compared()` fail-closed behavior (`requested_reference_cannot_be_silently_skipped` guards it).
- Test coverage: Strong (adversarial, fixtures, import suites) — but all on synthetic data until OD-01/OD-03 resolve.

**Parallel-agent collision zone:**

- Files: `crates/application/src/quran_search.rs`, `docs/03-plan/phases/phase-*/tasks.md`, `docs/03-plan/phases/phase-*/done.md`
- Why fragile: Demonstrated 2026-09-16: 7+ live sessions, one file rewritten mid-read, staged set churned twice, one `edit` failed safely on stale content (FU-COORD-01).
- Safe modification: File-level ownership in the execution plan before starting; non-owners verify read-only (`cargo test`, no source writes) and sync only task-ledger lines the owner is not touching.
- Test coverage: N/A (process, not code).

## Scaling Limits

**Single-writer SQLite:**

- Current capacity: One write pool connection (`max_connections(1)`), WAL, 5 s busy timeout — adequate for local-first single-user research use.
- Limit: Concurrent writers (CLI + server, two CLIs) hit `SQLITE_BUSY` → `StorageUnavailable` after 5 s. No queue, no retry-with-backoff at the application layer for this case.
- Scaling path: Document single-writer discipline; friendly locked-DB diagnostic; if multi-user server deployment ever becomes a goal, evaluate a writer queue or a server-owned database handle with the CLI going through the API.

**Graph traversal depth (forward-looking):**

- Current capacity: No graph workload exists yet (`crates/graph/src/lib.rs`, `crates/quran-graph/src/lib.rs` are Phase-3 placeholders; `crates/quran-morphology/src/lib.rs` is a Phase-2 placeholder).
- Limit: ADR-0202's default (SQLite adjacency + bounded recursive CTEs / batched frontier traversal) will degrade on deep transitive workloads (word-root families, isnad chains) — which is why the CozoDB spike (TASK-411) exists as a conditional.
- Scaling path: Keep SQLite-CTE default; authorize the CozoDB spike only if a Phase-3-exit benchmark on a realistic word/root graph misses agreed latency targets (targets + dataset + ratifier still pending per `open-questions.md` 2026-09-22).

**Corpus size assumption:**

- Current capacity: Everything validated on `test-edition-min` (synthetic fixture, 331-case golden set).
- Limit: Full Uthmani corpus + translations + word-gloss + morphology tables will stress import time, index build, FTS5 size, and the 10k-lookup soak — none measured.
- Scaling path: Approved-edition soak (FU-P1-01) with recorded deep-scan duration; `0015`/`0016` index verification; search benches.

## Dependencies at Risk

**Proposed-but-unaccepted graph backends:**

- Risk: CozoDB spike (MPL-2.0 copyleft scope needs review for static linking), SQLite graph extension (alpha, >1k-node bar), Kuzu archived read-only Oct 2025. None are dependencies today — the risk is a future phase assuming one without ratification.
- Impact: License surprise (CozoDB) or alpha instability (sqlite-graph) if pulled in casually.
- Migration plan: No migration needed (nothing to migrate from); keep the decision gate: benchmark first, ratify in `open-questions.md` with date + ratifier, then add the dependency.

**Unpinned external data catalogs:**

- Risk: `quran-api` (preferred catalog), `quran-database` (schema reference), `quranchecksum` (integrity reference) are upstream roles recorded in `docs/02-architecture/upstream-sources.md` and ADR-0101, but repository licence ≠ data licence and per-edition terms stay `unknown`. Depending on upstream shape without pinned slugs/hashes invites silent corpus drift.
- Impact: Importing an unlicensed or drifted edition as canonical would violate the project's core integrity guarantees.
- Migration plan: Pin exact `upstream_edition_slug` + publisher/release + license evidence + numbering/normalization sign-off (OD-01) before any network import; `compare_reference` (QV-015) stays fail-closed until the reference corpus + procedure + signer land (OD-03).

**CI-only gates with no local enforcement reminder:**

- Risk: `cargo-deny` (advisories/bans/licenses/sources), 3-OS test matrix, `arch-check` + `migrate-check` run in `.github/workflows/ci.yml` but are easy to skip locally. A contributor adding a dependency (e.g. Tantivy, an OTLP vendor crate) may only discover the deny/allowlist failure after pushing.
- Impact: Slow feedback; license-banned crate could briefly land on a branch.
- Migration plan: Document the pre-push gate sequence (`fmt`, `clippy -D warnings`, `arch-check`, `migrate-check`, workspace tests) in the contributor path; consider a pre-commit hook or `xtask` composite command.

## Missing Critical Features

**Owner-gated Phase-1 closure items (14 open decisions):**

- Problem: OD-01 (dataset/license/policy), OD-02 (named reviewer + `verified_by`), OD-03 (reference corpus + QV-015 sign-off), OD-04 (debug font license), OD-05 (schedule: 82 ed stated vs 131.0 ed summed), OD-06 (ratify axum + tower-http), OD-07 (coverage floors), OD-08/OD-09 (ritual reviewer + sign-off rows), OD-10 (Swimlane X ownership), OD-11/OD-12 (Phase-2 morphology/normalization inputs), OD-13 (Phase-0 exit reconciliation), OD-14 (server layering). All 🔴 per `docs/05-followups/decisions-needed.md`; orchestrator recommendations + owner addendum A0–A10 (2026-09-22) become effective 2026-09-25 barring written objection but still need recorded ratification.
- Blocks: AC-P1-01, AC-P1-18, 19 partially-verified acceptance criteria (ritual half), P1-X01…X05, P1-T01/T02/T03/T26/T54/T55/T56/T58/T60.

**Secret backends beyond env:**

- Problem: Keychain + age-encrypted-file backends unfinished (Phase-0 remaining work, `docs/06-progress/status.md`).
- Blocks: Production-grade server deployment; safe handling of API keys beyond local dev.

**Coverage gates and recorded exit ritual:**

- Problem: Coverage floors (quran-core ≥ 90%, validation/tokenize/hashing ≥ 90%, adapters/differ ≥ 80%, citations ≥ 85%, storage-sqlite ≥ 80% per `acceptance.md` §4) unratified (OD-07); no coverage tooling pinned in CI (`tarpaulin`/`llvm-cov` absent); the §5 live walkthrough + recording archive has no reviewer or location (OD-08).
- Blocks: AC-P0-03/05/06/08/11/14/16, P1-T60, Phase-2 unblocking.

**Vector path (new 2026-09-22):**

- Problem: Owner addendum A7 introduces a vector-search path with no backing task yet; `quran-search` constitutionally forbids embedding/vector dependencies (AC-P2-36, `arch-check` enforced).
- Blocks: Nothing yet — but any vector work must land in a new crate/phase, never inside `quran-search`.

## Test Coverage Gaps

**Production SQLite scheduling path:**

- What's not tested: Wall-clock scheduling, lease-recovery retry policy, and crash/lease chaos (AC-P0-11/12) against the real SQLite job store. Only `InMemoryJobQueue` paths are covered.
- Files: `crates/jobs/src/queue.rs`, `crates/jobs/src/worker.rs`
- Risk: The flake fix validated the in-memory queue; production behavior (WAL locking + wall clock + crash recovery) is a different system.
- Priority: High

**Approved-corpus evidence (all synthetic today):**

- What's not tested: Full-corpus import/validation/activation, golden-text comparison against a real muṣḥaf, 10k-lookup soak timing, and deep-scan duration on the standard edition.
- Files: `crates/application/tests/quran_doctor.rs`, `crates/application/tests/quran_import.rs`, `fixtures/quran/golden/ayah_texts.jsonl`
- Risk: Fixture-green results misread as production readiness; editorial correctness entirely unproven.
- Priority: High (blocked on OD-01/OD-02/OD-03, not on engineering effort)

**OTLP export redaction:**

- What's not tested: No test asserts span fields are scrubbed before OTLP export — the only ignored OTLP test needs a live endpoint (`crates/observability/src/otlp.rs:61-67`).
- Files: `crates/observability/src/otlp.rs`, `crates/observability/src/telemetry.rs`, `crates/testkit/tests/telemetry_privacy.rs`
- Risk: Silent secret exfiltration via the one network egress telemetry has.
- Priority: High

**Placeholder crates (by design, but track the gap):**

- What's not tested: `crates/agency`, `crates/agent-domain`, `crates/agent-runtime`, `crates/api`, `crates/approvals`, `crates/conversations`, `crates/embeddings`, `crates/evaluation`, `crates/graph`, `crates/hadith-core`, `crates/hadith-ingestion`, `crates/ingestion`, `crates/isnad-graph`, `crates/llm`, `crates/mcp`, `crates/memory`, `crates/model-router`, `crates/policy`, `crates/quran-graph`, `crates/quran-morphology`, `crates/rag`, `crates/reranking`, `crates/retrieval`, `crates/scripture`, `crates/tui`, `crates/workflows` are one-line Phase-N placeholders (`//! Phase N placeholder — <name>.`) with no logic and no tests.
- Files: Each `crates/<name>/src/lib.rs`
- Risk: Low today (placeholders reserve names and keep the workspace graph stable), but any logic added to a placeholder without tests + allowlist update + arch-check will look deceptively established.
- Priority: Low (revisit as each phase activates its crates)

**Container runtime:**

- What's not tested: Image build, non-root execution, volume ownership, and endpoint reachability from inside the container network namespace.
- Files: `Dockerfile`, `docker-compose.yml`
- Risk: "Deployable" claim unproven; first real deploy discovers environment issues.
- Priority: Medium

**Coverage instrumentation itself:**

- What's not tested: No enforced measurement — coverage floors exist only as unratified prose in `acceptance.md` §4.
- Files: `docs/03-plan/phases/phase-01-core/acceptance.md`, `.github/workflows/ci.yml`
- Risk: Coverage decays silently while gates claim green (fmt/clippy/tests/arch/migrate all pass without it).
- Priority: Medium

---

*Concerns audit: 2026-09-22*
