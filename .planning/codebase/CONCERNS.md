---
last_mapped_commit: 6891d7bc64bc469a10e9a28faf81023fa11cb9ae
last_mapped_at: 2026-11-01
---
# Codebase Concerns

**Analysis Date:** 2026-11-01

## Tech Debt

**STALE TEST REFERENCE — `layer_separation.rs` names a deleted component (fails closed, loudly):**

- Issue: `crates/application/tests/layer_separation.rs:149` iterates `["web/src/components/TranslationPanel.tsx", "web/src/components/WordInspector.tsx"]` and calls `std::fs::read_to_string(...).expect("component exists")`. `web/src/components/WordInspector.tsx` no longer exists (verified: `web/src/components/` contains `AyahColumn.tsx`, `TranslationPanel.tsx`, `CitationCopy.tsx`, `ToolLedger.tsx`, `LayerBadge.tsx`, `layout/`, `ui/`, `command/`, `states.tsx` — no `WordInspector.tsx`; zero `WordInspector` references remain anywhere under `web/src/`). A missing file makes `spa_non_canonical_components_wear_no_canonical_class` panic on the `.expect`, failing the whole layer-separation gate, not just one assertion.
- Files: `crates/application/tests/layer_separation.rs:147-154`, `web/src/components/TranslationPanel.tsx` (surviving member of the pair)
- Impact: `cargo test -p application --test layer_separation` fails until the reference is removed or the component is restored. Because the scan is a CI backstop for ADR-0112/D-04/D-12, a red gate blocks every phase landing. Failure mode is loud (panic message names the file), not silent — no integrity risk, purely a broken-gate blocker.
- Fix approach: Delete `"web/src/components/WordInspector.tsx"` from the array (one-line fix) if the WordInspector removal was intentional, and confirm no other test or view still imports it. If the component was deleted accidentally, restore it — `ReadingView.tsx` and `TranslationPanel.tsx` no longer reference it, so removal looks intentional. Re-run the layer_separation suite after the edit.

**Oversized god-modules (grew since last audit):**

- Issue: Three files keep absorbing unrelated responsibilities and grew significantly: `crates/application/src/quran_cli.rs` (5483 lines — was 1815 at last audit; CLI verb dispatch plus tool plumbing), `crates/storage-sqlite/src/quran.rs` (3031 lines — was 1885; staging, activation, reads, reports, citations, translations), `crates/application/src/quran_search.rs` (2175 lines — query parsing, caching, tools, counting, explanation). `crates/storage-sqlite/src/lib.rs` (1747 lines) remains large.
- Files: `crates/application/src/quran_cli.rs`, `crates/storage-sqlite/src/quran.rs`, `crates/application/src/quran_search.rs`, `crates/storage-sqlite/src/lib.rs`
- Impact: Merge-collision magnet (already demonstrated once — `quran_search.rs` was rewritten mid-session during a parallel-session collision, FU-COORD-01); hard to review; blast radius of any search/storage/CLI change is the whole file.
- Fix approach: Split `quran_cli.rs` by verb family and `quran.rs` into staging/reads/reports submodules without behavior change; split `quran_search.rs` along its existing module seams (query model / cache / tools / explain). Enforce new-code placement in the split modules.

**`server` layering allowlist (accepted but load-bearing):**

- Issue: `xtask/allowlist.toml:48-54` explicitly allows `server` to depend on `storage` + `tools` directly instead of routing through `application`. The comment records OD-14 acceptance with trait-seam requirements at each call site so the Phase-3 routing evaluation (candidate ADR-0300) stays a wiring change. `cargo run -p xtask -- arch-check` enforces the allowlist, so the violation is fenced but not fixed.
- Files: `xtask/allowlist.toml`, `crates/server/src/api.rs`, `crates/server/src/lib.rs`
- Impact: HTTP layer can bypass application-service invariants (approval gates, audit bridging, generation-keyed cache invalidation) by touching the DB handle directly.
- Fix approach: In Phase-3 server hardening, route all Quran reads through `application` services and tighten the allowlist to remove `storage` from `server`. Until then, forbid new `server → storage` call sites in review. Changing the allow set requires a new OD entry per the file header.

**FTS5 vs Tantivy drift — enum promises an engine that does not exist:**

- Issue: `crates/quran-search/src/model.rs:24-25` still defines a `Tantivy` variant ("deferred; see DEV-05") and the crate docs say "FTS5 now, Tantivy later", but no Tantivy adapter exists and no `tantivy` dependency is declared in any `Cargo.toml`. Specs repeatedly forbid assuming Tantivy. Any code path that constructs or matches the `Tantivy` variant is a dead/phantom branch.
- Files: `crates/quran-search/src/model.rs`, `crates/quran-search/src/index.rs`, `crates/quran-search/src/lib.rs`, `crates/quran-search/src/fts5.rs`
- Impact: Contributors can write backend-agnostic code against an engine that will fail at runtime; spec/code vocabulary drift.
- Fix approach: Either gate the `Tantivy` variant behind an explicit `unimplemented!` error with a `QAI-IDX-*` code and a test asserting it fails closed, or remove the variant until the ADR is accepted. Do not add the dependency until ratified.

**Migration numbering / reversibility drift (chain now at 0019+):**

- Issue: Phase-0 migrations `0001`–`0006` ship `.down.sql` files; migrations `0007_quran_editions` through at least `0019_quran_graph` in `migrations/sqlite/` are up-only with no down migration. Forward-only is intentional per invariant I7, but the asymmetry confuses rollback expectations, and phase ledgers undercount the chain (`docs/03-plan/phases/` tracks only up to phase-04 while `.planning/phases/` runs 01 through 09+ — two planning trees describing the same workspace).
- Files: `migrations/sqlite/0007_quran_editions.up.sql` through `migrations/sqlite/0019_quran_graph.up.sql`, `migrations/sqlite/checksums.json`, `crates/storage-sqlite/src/migrate.rs`
- Impact: A bad canonical-table migration cannot be rolled back; checksum verification (`verify_checksums`) aborts forward progress on drift; stale phase ledgers misreport schema state.
- Fix approach: Document forward-only as policy in `migrations/sqlite/` and either remove the Phase-0 `.down.sql` files or mark them legacy/non-executed. Reconcile the two planning trees (or declare one canonical). Keep `cargo run -p xtask -- migrate-check` green on every migration PR; never edit an applied migration.

**Concurrent-writer hazard (SQLite multi-process):**

- Issue: `crates/storage-sqlite/src/lib.rs` correctly uses WAL + `synchronous=FULL` + busy timeout with a single-connection write pool (`max_connections(1)`). But SQLite still serializes writers at the database level: a second `qai` process (CLI import while the server holds the write pool, or two CLIs) blocks to the busy-timeout and then fails with `StorageUnavailable`. No cross-process lock, queue, or single-writer sidecar exists. Concurrency tests cover tasks against one pool, not two processes.
- Files: `crates/storage-sqlite/src/lib.rs`, `crates/config/src/lib.rs`
- Impact: `SQLITE_BUSY` failures under concurrent CLI+server use; generation allocation races across processes.
- Fix approach: Document single-writer discipline in the import/runbook docs; surface a clear "database is locked by another writer, retry" diagnostic instead of bare `StorageUnavailable`; long-term consider a writer queue or file-lock guard. Keep the FU-COORD-01 precedent: implementation owner edits, second session verifies read-only.

**Spec/code paperwork drift (two planning trees):**

- Issue: Workspace state is tracked in two places that disagree on coverage: `docs/03-plan/phases/` (phase-00 through phase-04) and `.planning/phases/` (01-foundations through 09-comparative-scripture plus 03.5-residual-closure). `docs/03-plan/current-plan.md` is 0 bytes yet `AGENTS.md` instructs agents to read it first. Prior audits also found spec checklists unchecked while implementation is landed (code ahead of paperwork) and vice versa.
- Files: `docs/03-plan/phases/`, `.planning/phases/`, `docs/03-plan/current-plan.md`, `AGENTS.md`
- Impact: Auditors and new sessions cannot tell which tree is authoritative; acceptance-evidence mapping rots; every new session pays rediscovery cost.
- Fix approach: Declare one tree canonical (or a merge direction) and either fill `current-plan.md` with an active-phase pointer or delete it and remove the references. Owning session flips spec checkboxes with landing commit or records a deviation note. No code changes.

**CLOSED since last audit — `plus()` subsecond truncation (fixed, keep as regression watch):**

- Resolution verified 2026-11-01: `crates/jobs/src/queue.rs:132-137` now adds `time::Duration::nanoseconds(d.subsec_nanos() as i64)` alongside whole seconds, and a regression test `plus_preserves_subsecond_delays` (`crates/jobs/src/queue.rs:443`) asserts a 500 ms delay lands ~0.5 s out and a 60 s delay stays intact. Timestamp comparison parses instants (`rfc3339_le`).
- Residual: production SQLite scheduling path (wall clock, lease recovery, crash chaos) is still a different system from the tested in-memory queue — see Test Coverage Gaps.

**CLOSED since last audit — OTLP span-field redaction gap (fixed, keep as coverage watch):**

- Resolution verified 2026-11-01: `crates/observability/src/otlp.rs` now defines `scrub_span_attributes` plus a `ScrubbingProcessor<P: SpanProcessor>` that strips scrubbed attribute keys (content-denylist + secret-name rule) from spans and their events at the exporter boundary, precisely because the stderr `RedactingWriter` layer cannot see OTLP traffic. Attribute keys are dropped, not replaced.
- Residual: confirm scrub-fixture unit tests cover the exporter boundary (the only ignored OTLP test historically needed a live endpoint) — see Test Coverage Gaps.

## Known Bugs

**`layer_separation::spa_non_canonical_components_wear_no_canonical_class` panics on missing file:**

- Symptoms: `.expect("component exists")` on `web/src/components/WordInspector.tsx` panics because the file was deleted with no surviving reference. The whole `layer_separation` test binary reports failure on this one test; all other predicates (canonical-slot props on `AyahColumn.tsx`, layer tokens in `web/src/tokens/layers.css`, no-HTML-injection / no-remote-URL / no-bundled-font scans over `web/src`, TUI purity scan over `crates/tui/src`) are unaffected in code but masked by the red gate.
- Files: `crates/application/tests/layer_separation.rs:147-154`
- Trigger: Any `cargo test -p application --test layer_separation` run (deterministic, not flaky).
- Workaround: None — fix the reference (see Tech Debt). Do not "fix" by creating an empty `WordInspector.tsx`; either restore the component or drop the array entry.

**TUI search/lexicon backends are `unimplemented!()` stubs:**

- Symptoms: `crates/tui/src/app.rs:34-60+` defines `UnavailableSearch` (all five `SearchBackend` methods) and `UnavailableLexicon` whose bodies are `unimplemented!()`. Any TUI path that actually issues a search or lexicon call panics instead of rendering an error state.
- Files: `crates/tui/src/app.rs`
- Trigger: Driving the TUI cockpit into search/reader paths backed by these stubs.
- Workaround: Wire the real `application` search/lexicon services (D-12 direction) or render an explicit "unavailable" state; never ship a user-reachable `unimplemented!()`.

**Doctor-suite timeout under parallel test threads (unresolved):**

- Symptoms: A parallel `cargo test -p application` run timed out during doctor tests (`crates/application/tests/quran_doctor.rs`), while isolated and serial reruns passed. Recorded in FU-P1-02.
- Files: `crates/application/tests/quran_doctor.rs`
- Trigger: Full parallel application suite including the 10,000-lookup fixture soak.
- Workaround: Serial invocation for doctor/soak tests (`-- --test-threads=1`); investigate reproducibility rather than claiming the serial pass as a fix.

**Container verification unverified (evidence gap, not a code bug):**

- Symptoms: `Dockerfile`, `docker-compose.yml`, `.dockerignore` exist and compose validation passed, but the Docker daemon socket was unavailable at check time, so image build, shared-library compatibility, named-volume ownership, and non-root (UID 65532) execution were never exercised.
- Files: `Dockerfile`, `docker-compose.yml`, `.dockerignore`
- Trigger: Any claim that "container deployment works" before `docker compose build`, `docker compose run --rm app db migrate`, `db verify`, and in-namespace `/healthz`, `/readyz`, `/api/v1/meta` checks pass.
- Workaround: None — run the recorded sequence once a daemon is available. The stub intentionally uses `network_mode: none` with no published ports (loopback-only server); that is design, not a bug.

## Security Considerations

**Deny-by-default tool execution (verified boundary — keep closed):**

- Risk: Mutating tools reached over HTTP could write durable RAG or narrator-identity state from an unauthenticated browser surface.
- Files: `crates/server/src/api.rs:173,243-245,531-550` (deny with `QAI-RAG-0326` for `rag.create_project|create|import|source_status|source_activate|source_deactivate|source_tombstone`, `QAI-ISNAD-0326` for `isnad.identity_propose|accept|reject|dispute|supersede`, `QAI-QUR-0326` mapped to 403), `crates/config/src/lib.rs:123,211` (`policy.tool_execution_default` defaults to `"deny"`), `crates/cli/src/lib.rs:478` (CLI opts in via config), `web/src/lib/tools.ts:522+` (`DENIED` catalogue with per-family codes and reasons), `crates/tool-registry/src/lib.rs:2265` (registry-side gating note)
- Current mitigation: Deny-by-default on the HTTP route with typed 403 codes; opt-in only via `policy.tool_execution_default = "allow"`. The web UI renders every denied tool as an `aria-disabled` row with its policy code, no form and no run control (`web/src/lib/tools.test.ts:97-121`, `web/src/views/ToolsView.test.tsx`, `web/src/views/RagDebugView.tsx:29`, `web/src/views/IsnadView.test.tsx:114-128`). The trusted mutation surface is the local CLI. Verified in sync across server, config, registry, and web catalogue on 2026-11-01.
- Recommendations: Never add a new mutating tool to the HTTP-callable set without a `QAI-*-0326` deny entry, a `DENIED` catalogue row, and a disabled-row test. Keep the CLI as the only mutation path. Audit any future `tool_execution_default` config-plumbing change (`crates/config/src/lib.rs:564-565,780-781,904-906`) for fail-open defaults.

**OTLP export exfiltration (exporter-boundary scrubber landed):**

- Risk: Secret-shaped span attributes exfiltrate to a remote OTLP collector — the one path where data leaves the local-first boundary.
- Files: `crates/observability/src/otlp.rs` (`ScrubbingProcessor`, `scrub_span_attributes`), `crates/observability/src/telemetry.rs` (`is_forbidden_field` denylist), `crates/domain/src/redaction.rs` (Rule A)
- Current mitigation: Telemetry off by default (opt-in only, PRD §39); `is_forbidden_field` denylist gates export payloads; `redact_secrets=true` default in `InitOptions`; exporter-boundary `ScrubbingProcessor` drops scrubbed keys from spans and events.
- Recommendations: Attribute-key allowlist (deny-by-default) as the next hardening step; never put Arabic text, user strings, or query payloads in span attributes; hash file paths; keep scrub-fixture unit tests covering the exporter boundary. Pending owner ratification (`docs/05-followups/open-questions.md` § Telemetry & Redaction).

**Secret backends — env-only in practice:**

- Risk: `crates/config/src/secret_store.rs` defines `EnvSecretStore`, `EncryptedFileSecretStore` (XChaCha20-Poly1305 JSON), and `KeychainSecretStore` behind one trait, but operators fall back to plaintext env vars (`QAI_SECRET_<KEY>`) with shell-history / process-table exposure.
- Files: `crates/config/src/secret_store.rs`, `crates/config/src/secret.rs`, `crates/config/src/lib.rs`
- Current mitigation: `Secret<T>` wrapper with debug-redaction; `secret://<backend>/<path>` reference parsing that fails closed; workspace-wide `unsafe_code = "forbid"` (`Cargo.toml:64`).
- Recommendations: Finish and document the encrypted-file backend first (cross-platform), then keychain; add a `qai doctor` check warning when secrets resolve from env in server mode.

**No model fabrication is architecturally enforced, but only as far as constructor discipline holds:**

- Risk: A new surface that builds `QuranQuotation` (or renders ayah text) without the visibility-restricted constructor in `crates/quran-core` could present generated text as quotation.
- Files: `crates/quran-core/src/` (`QuranQuotation`), `crates/citations/src/lib.rs`, `crates/server/src/api.rs`, `crates/cli/src/lib.rs`
- Current mitigation: Restricted constructor (invariant I6); citation `QuotationVerdict` with `Mismatch`/`LocationNotFound` variants; `arch-check` layering gate; golden `fixtures/quran/golden/ayah_texts.jsonl` byte-tied to the fixture. The `web/src` vs `crates/` separation (enforced by `layer_separation.rs` scans: no `dangerouslySetInnerHTML`, no remote URLs, no bundled webfonts, canonical-slot prop guard on `AyahColumn.tsx`) keeps dataset text out of HTML injection paths.
- Recommendations: Keep routing new read surfaces through `application::QuranReader`; never fabricate scripture in fixtures; require citation-verdict coverage on any new quote-rendering endpoint.

## Performance Bottlenecks

**Fixture soak is synthetic — real-corpus performance unknown:**

- Problem: `fixture_soak_ten_thousand_lookups_preserves_corpus_integrity` (`crates/application/tests/quran_doctor.rs`) proves 10k seeded lookups preserve integrity on `test-edition-min`, but says nothing about standard-edition latency, deep-scan duration, or editorial correctness.
- Files: `crates/application/tests/quran_doctor.rs`, `crates/storage-sqlite/src/quran.rs`
- Cause: Approved dataset blocked on owner decisions OD-01/OD-02/OD-03 (dataset, reviewer, reference corpus).
- Improvement path: Once an approved edition lands, adapt the harness to accept it and record corpus identity, hashes, lookup results, and deep-scan duration.

**FTS5 index build / large-result search unprofiled:**

- Problem: No benchmarks exist for index-build time, query latency on large result sets, or the `quran_search` concatenation-window path over the full corpus. The hot path does per-hit `to_string()` rendering for diagnostics/remedies and holds `HashMap`-based intermediate state.
- Files: `crates/application/src/quran_search.rs`, `crates/quran-search/src/fts5.rs`, `crates/quran-search/src/tokenizer.rs`, `migrations/sqlite/0015_quran_indexes.up.sql`, `migrations/sqlite/0016_quran_search_cache.up.sql`
- Cause: Optimization deferred; corpus small enough that nothing hurts yet.
- Improvement path: Add a criterion-style bench for index build + representative queries once the approved edition lands; cache diagnostic strings or defer rendering until the error path; verify `0015`/`0016` indexes cover the actual query shapes (`EXPLAIN QUERY PLAN` in review).

**Generation-keyed reader cache invalidation cost:**

- Problem: `QuranReader`'s generation-keyed cache (`crates/application/src/quran_reader.rs`; cache logic in `crates/application/src/quran_search_cache.rs`) avoids stale text after activation, but every activation bumps the generation and cold-starts dependent caches (search cache table `0016_quran_search_cache` included).
- Files: `crates/application/src/quran_reader.rs`, `crates/application/src/quran_search_cache.rs`
- Cause: Correctness-first design (no stale text ever) with coarse invalidation.
- Improvement path: Measure before optimizing; if activation-adjacent latency matters, move to per-edition generation keys. Current behavior is correct — do not weaken it for speed.

## Fragile Areas

**`crates/application/src/quran_cli.rs` (5483 lines and still growing):**

- Files: `crates/application/src/quran_cli.rs`
- Why fragile: CLI verb dispatch plus tool plumbing in a single file; more than tripled since the last audit. Any CLI-surface change touches the same file every other verb lives in.
- Safe modification: Single-owner edits; split by verb family before adding new verbs; run the CLI parity suites + `cargo fmt` before handing off.
- Test coverage: CLI parity legs exist but the file's size means targeted test runs miss neighbors — run the full CLI-related suites.

**`crates/storage-sqlite/src/quran.rs` (3031 lines) + migration chain `0007`–`0019+`:**

- Files: `crates/storage-sqlite/src/quran.rs`, `crates/storage-sqlite/src/lib.rs`, `migrations/sqlite/0007_quran_editions.up.sql` … `migrations/sqlite/0019_quran_graph.up.sql`
- Why fragile: Canonical tables are insert-only via triggers — a bad migration cannot be rolled back (no down files) and checksum verification aborts forward progress on drift. Touching triggers or activation logic risks bricking existing databases.
- Safe modification: New migrations only, never edit an applied one (checksum gate will fail the build); test activation/rollback paths in `crates/storage-sqlite/tests/quran.rs` and `integrity_provenance.rs`; run `cargo run -p xtask -- migrate-check`.
- Test coverage: Strong (quran tests, integrity/provenance tests, generation monotonicity) but single-process only.

**`crates/application/src/quran_search.rs` (2175 lines):**

- Files: `crates/application/src/quran_search.rs`, `crates/application/tests/search_tools.rs`
- Why fragile: Five responsibilities in one file; already rewritten once mid-session by a colliding writer; tight-hull span assertions show how brittle window expectations are.
- Safe modification: Single-owner edits only (FU-COORD-01 precedent); run `search_tools` + application lib suites + `quran-search` suites + `cargo fmt` before handing off; prefer adding a focused submodule over extending the file.
- Test coverage: Good for shipped paths but Tantivy branches (if any match on the enum) are untested by construction.

**Import pipeline `crates/quran-corpus/src/import.rs` (13 checkpoints):**

- Files: `crates/quran-corpus/src/import.rs`, `crates/quran-corpus/src/validation.rs`, `crates/application/tests/quran_import.rs`
- Why fragile: Restart-is-resume + cancellation-cleanup + reference comparison (`compare_reference`, fail-closed) interact; each validator carries adversarial fixtures that must keep rejecting with the specific rule id.
- Safe modification: Change one validator at a time; keep the adversarial fixtures green; never loosen `compared()` fail-closed behavior.
- Test coverage: Strong — but all on synthetic data until OD-01/OD-03 resolve.

**`temp-inspire/` scratch (read-only by convention, gitignored):**

- Files: `temp-inspire/` (contains `complete-phase-13-roadmap*` working copies), `.gitignore:61` (`temp-inspire/` — never committed)
- Why fragile: Phase-13 roadmap scratch lives beside the tree and is easy to mistake for source of truth or to import into real code. Nothing in `crates/` or `web/src` reads from it (verified no references), and git ignores it — that is the entire enforcement mechanism, i.e. convention only.
- Safe modification: Treat as read-only inspiration: copy ideas out, never add imports pointing in. Do not remove the `.gitignore` entry. Delete the directory when Phase-13 closes to avoid stale-plan drift.

**Parallel-agent collision zone:**

- Files: `crates/application/src/quran_search.rs`, `.planning/phases/*/`, `docs/03-plan/phases/*/`
- Why fragile: Demonstrated 2026-09-16: 7+ live sessions, one file rewritten mid-read, staged set churned twice, one `edit` failed safely on stale content (FU-COORD-01). The two planning trees multiply the collision surface (two task ledgers to sync).
- Safe modification: File-level ownership in the execution plan before starting; non-owners verify read-only (`cargo test`, no source writes) and sync only task-ledger lines the owner is not touching.
- Test coverage: N/A (process, not code).

## Scaling Limits

**Single-writer SQLite:**

- Current capacity: One write pool connection (`max_connections(1)`), WAL, busy timeout — adequate for local-first single-user research use.
- Limit: Concurrent writers (CLI + server, two CLIs) hit `SQLITE_BUSY` → `StorageUnavailable` after timeout. No queue, no retry-with-backoff at the application layer for this case.
- Scaling path: Document single-writer discipline; friendly locked-DB diagnostic; if multi-user server deployment ever becomes a goal, evaluate a writer queue or a server-owned database handle with the CLI going through the API.

**Graph traversal depth (forward-looking):**

- Current capacity: Graph crates exist with substantially more than placeholder content in the Phase-4 line (`crates/quran-graph`, `crates/isnad-graph` with migration `0019_quran_graph`); `crates/policy` verified still a one-file placeholder (`crates/policy/src/lib.rs:1-4`, "reserved for Phase 7 … contains no implementation yet").
- Limit: Default SQLite adjacency + bounded recursive CTEs will degrade on deep transitive workloads (word-root families, isnad chains).
- Scaling path: Keep SQLite-CTE default; authorize any alternative backend only on a Phase-exit benchmark against a realistic graph missing agreed latency targets (targets + dataset + ratifier still pending per `open-questions.md`).

**Corpus size assumption:**

- Current capacity: Everything validated on `test-edition-min` (synthetic fixture, golden set).
- Limit: Full Uthmani corpus + translations + word-gloss + morphology tables will stress import time, index build, FTS5 size, and the 10k-lookup soak — none measured.
- Scaling path: Approved-edition soak with recorded deep-scan duration; `0015`/`0016` index verification; search benches.

## Dependencies at Risk

**Proposed-but-unaccepted graph backends:**

- Risk: CozoDB spike (MPL-2.0 copyleft scope needs review for static linking), SQLite graph extension (alpha), Kuzu archived read-only Oct 2025. None are dependencies today — the risk is a future phase assuming one without ratification.
- Impact: License surprise (CozoDB) or alpha instability if pulled in casually.
- Migration plan: No migration needed (nothing to migrate from); keep the decision gate: benchmark first, ratify with date + ratifier, then add the dependency.

**Unpinned external data catalogs:**

- Risk: Upstream Quran data roles recorded in architecture docs and ADR-0101, but repository licence ≠ data licence and per-edition terms stay unresolved. Depending on upstream shape without pinned slugs/hashes invites silent corpus drift.
- Impact: Importing an unlicensed or drifted edition as canonical would violate the project's core integrity guarantees.
- Migration plan: Pin exact `upstream_edition_slug` + publisher/release + license evidence + numbering/normalization sign-off (OD-01) before any network import; reference comparison stays fail-closed until the reference corpus + procedure + signer land (OD-03).

**CI-only gates with no local enforcement reminder:**

- Risk: `cargo-deny` (advisories/bans/licenses/sources), multi-OS test matrix, `arch-check` + `migrate-check` run in CI but are easy to skip locally. A contributor adding a dependency (e.g. Tantivy, an OTLP vendor crate) may only discover the deny/allowlist failure after pushing. `cargo-deny` is not installed locally (recorded in phase summaries as a known local-tool limitation).
- Impact: Slow feedback; license-banned crate could briefly land on a branch.
- Migration plan: Document the pre-push gate sequence (`fmt`, `clippy -D warnings`, `arch-check`, `migrate-check`, workspace tests) in the contributor path; consider a pre-commit hook or `xtask` composite command.

## Missing Critical Features

**Owner-gated closure items (open decisions):**

- Problem: OD-01 (dataset/license/policy), OD-02 (named reviewer + `verified_by`), OD-03 (reference corpus sign-off), OD-04 (debug font license), OD-14 (server layering), plus coverage-floor and ritual-sign-off decisions — recorded in `docs/05-followups/decisions-needed.md` and `docs/05-followups/open-questions.md` (107 lines, still active). Some addenda become effective barring written objection but still need recorded ratification.
- Blocks: Corpus-dependent acceptance criteria, coverage gates, Phase-3 server-hardening evaluation (candidate ADR-0300).

**Secret backends beyond env:**

- Problem: Keychain + age-encrypted-file backends unfinished; operators use env vars.
- Blocks: Production-grade server deployment; safe handling of API keys beyond local dev.

**Coverage gates and recorded exit ritual:**

- Problem: Coverage floors unratified; no coverage tooling pinned in CI (`tarpaulin`/`llvm-cov` absent); live walkthrough + recording archive has no reviewer or location.
- Blocks: Corpus-independent acceptance evidence, phase-exit confidence.

**Vector path discipline:**

- Problem: Any vector-search path must land in a new crate/phase, never inside `quran-search` (AC-P2-36, `arch-check` enforced). `crates/embeddings` exists with a leaf allowlist (`workspace.allow = []`), which is the correct home — but the boundary needs active defense in review as RAG phases grow.
- Blocks: Nothing yet — preventative. Reject any PR adding embedding/vector edges to `quran-search`, `quran-normalization`, or `quran-core`.

## Test Coverage Gaps

**Layer-separation gate (broken, not just gapped):**

- What's not tested: Nothing — the gate exists and its predicates carry inline self-checks (`scan_predicates_catch_violations`), but the suite cannot pass because of the stale `WordInspector.tsx` reference. The `web/src` vs `crates/` separation (no canonical text as untrusted input, no HTML injection, no remote URLs/fonts, TUI purity) is therefore currently *unenforced* in CI until the one-line fix lands.
- Files: `crates/application/tests/layer_separation.rs`
- Risk: While the gate is red, a genuine layering violation in `web/src` or `crates/tui/src` hides behind the known failure — or teams start ignoring the gate entirely.
- Priority: High (fix is trivial; blast radius of delay is gate credibility)

**Production SQLite scheduling path:**

- What's not tested: Wall-clock scheduling, lease-recovery retry policy, and crash/lease chaos against the real SQLite job store. Only in-memory queue paths are covered (now including the `plus_preserves_subsecond_delays` regression test).
- Files: `crates/jobs/src/queue.rs`, `crates/jobs/src/worker.rs`
- Risk: The subsecond fix validated the in-memory queue; production behavior (WAL locking + wall clock + crash recovery) is a different system.
- Priority: High

**Approved-corpus evidence (all synthetic today):**

- What's not tested: Full-corpus import/validation/activation, golden-text comparison against a real muṣḥaf, 10k-lookup soak timing, and deep-scan duration on the standard edition.
- Files: `crates/application/tests/quran_doctor.rs`, `crates/application/tests/quran_import.rs`, `fixtures/quran/golden/ayah_texts.jsonl`
- Risk: Fixture-green results misread as production readiness; editorial correctness entirely unproven.
- Priority: High (blocked on OD-01/OD-02/OD-03, not on engineering effort)

**OTLP export redaction (implementation landed, boundary tests to confirm):**

- What's not tested: Confirm a test asserts span fields are scrubbed at the `ScrubbingProcessor` exporter boundary — historically the only OTLP test needed a live endpoint (`crates/observability/src/otlp.rs`).
- Files: `crates/observability/src/otlp.rs`, `crates/observability/src/telemetry.rs`, `crates/testkit/tests/telemetry_privacy.rs`
- Risk: Silent secret exfiltration via the one network egress telemetry has, if the scrubber regresses uncovered.
- Priority: High

**Placeholder crates (by design, but track the gap):**

- What's not tested: `crates/policy` verified still a placeholder on 2026-11-01 (`crates/policy/src/lib.rs:1-4`). Sibling Phase-N crates (`agency`, `agent-domain`, `agent-runtime`, `api`, `approvals`, `conversations`, `evaluation`, `workflows`, and others) reserve names with minimal logic; treat any logic added to a placeholder without tests + allowlist update + arch-check as suspect.
- Files: Each `crates/<name>/src/lib.rs`
- Risk: Low today (placeholders keep the workspace graph stable), but new logic in a placeholder looks deceptively established.
- Priority: Low (revisit as each phase activates its crates)

**Container runtime:**

- What's not tested: Image build, non-root execution, volume ownership, and endpoint reachability from inside the container network namespace.
- Files: `Dockerfile`, `docker-compose.yml`
- Risk: "Deployable" claim unproven; first real deploy discovers environment issues.
- Priority: Medium

**Coverage instrumentation itself:**

- What's not tested: No enforced measurement — coverage floors exist only as unratified prose.
- Files: Phase acceptance docs, `.github/workflows/ci.yml`
- Risk: Coverage decays silently while gates claim green (fmt/clippy/tests/arch/migrate all pass without it).
- Priority: Medium

---

*Concerns audit: 2026-11-01*
