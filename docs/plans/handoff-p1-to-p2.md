# Handoff — Phase 1 → Phase 2

> **From:** Phase 1 — Canonical Quran Core (in progress, ~77% of task rows ☑)
> **To:** Phase 2 — Quran Search, Normalization & Linguistics
> **Author:** Phase-1 agent (owner to ratify)
> **Date:** 2026-09-15
> **Plan source of truth:** `docs/03-plan/phases/phase-01-core/plan.md`;
> deviations are recorded in `docs/03-plan/phases/phase-01-core/done.md` §5 and
> carried here in §6.

> **Verification state.** At handoff the full gate is green:
> `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`
> (135 suites), `cargo xtask arch-check`, and `cargo xtask migrate-check` all pass;
> `cargo fmt --all -- --check` is clean for the Phase-1 tree (the only unformatted files
> are Phase-2 files under active edit). A concurrent Phase-2 writer was active in the
> same tree, so **re-run the gate** before relying on it; `migrate-check` currently
> reports 14 migrations (Phase-1 delivered `0007`–`0012`, Phase-2 added `0013`–`0014`).

Phase 2 must **not re-invent** the following assets. Extend them.

## 1. Assets Phase 2 inherits

| Asset | Location | Phase 2 usage |
|---|---|---|
| Numbering newtypes + enums | `crates/quran-core/src/{numbers,enums}.rs` | Never re-derive `surah`/`ayah` from strings; use `SurahNumber`/`AyahNumber` |
| Edition / surah / ayah / segment / token structs | `crates/quran-core/src/{edition,structure}.rs` | Search hits carry the same edition identity |
| `QuranQuotation` + constructor guard | `crates/quran-core/src/quotation.rs` | The only allowed carrier of quoted canonical text (I6) |
| `AyahView` / `AttributedTranslation` / `ContextBoundary` | `crates/quran-core/src/view.rs` | Read surfaces; reuse instead of new DTOs |
| Reference grammar (parse/serialize) | `crates/quran-core/src/reference/{parser,serializer}.rs` | Parse user queries into references; never regex an ayah |
| Frozen hashing recipes | `crates/quran-corpus/src/hashing.rs` | `text_hash` / `structure_hash` / `token_order_hash`; do not fork |
| Tokenizer + offsets | `crates/quran-corpus/src/tokenize.rs` | Char-level surface tokens with grapheme-mapped offsets |
| Unicode auditor + normalization surface | `crates/quran-corpus/src/unicode.rs` | Phase-2 rule catalog builds on this |
| Validator registry + QV-001…028 | `crates/quran-corpus/src/validation.rs`, `crates/sources/src/registry.rs` | Add Phase-2 validators to the same registry |
| Importer (13 checkpoints) + differ | `crates/quran-corpus/src/{import,differ}.rs` | Ingest new editions; do not write canonical rows another way |
| `QuranRepository` traits + SQLite impl | `crates/storage/src/quran.rs`, `crates/storage-sqlite/src/quran.rs` | Extend for index/search tables |
| `QuranReader` + generation-keyed cache | `crates/application/src/quran_reader.rs` | Index freshness keys on `corpus_generation` |
| Application services (import/activate/rollback/translations) | `crates/application/src/quran.rs` | Same approval-gated canonical write path |
| Tool contract + registry | `crates/tools/src/lib.rs`, `crates/tool-registry/src/lib.rs` | New tools conform to `ToolResult`/`ReproducibilityData` |
| Citation resolver + deep links | `crates/citations/src/lib.rs`, `crates/quran-core/src/reference/serializer.rs` | Citation identity is frozen (ADR-0111) |
| API v1 + OpenAPI | `crates/server/src/api.rs`, `docs/08-api/quran-v1-openapi.json` | Add endpoints under the same envelope; update the spec |
| `doctor` + check registry | `crates/cli/src/doctor.rs`, `crates/application/src/quran_doctor.rs` | Add search/normalization checks; keep it read-only |
| `quran-normalization` (Phase-2 M1a/M1b, in progress) | `crates/quran-normalization/src/*` | The rule catalog (`N01`–`N22`), `SpanMap` (I10), pipeline, L0–L8 profiles |

## 2. Interfaces to build against

- **Storage:** implement repository traits; never bypass `UnitOfWork`. A
  projection-relevant write enqueues its outbox row through `UnitOfWork::outbox()`
  in the same transaction (`crates/storage/src/workflows.rs`).
- **Canonical writes:** only through the importer → `Staged` → approval → activation
  path. `ApprovalToken` exists only from a persisted human `ApprovalRecord`; the
  importer itself holds no token (I5/I7).
- **Reader:** go through `QuranReader` so caching and generation semantics are shared;
  do not read canonical tables directly from a tool or endpoint.
- **Tools:** every tool returns `ToolResult` with a deterministic reproducibility
  checksum and typed errors; requesting a non-existent reference must be a typed
  error, never a synthesized result.
- **API:** reuse the response envelope (`meta` + `data`) and the Phase-0 `Diagnostic`
  error body; extend `docs/08-api/quran-v1-openapi.json` and keep the route-coverage
  test green.
- **CLI:** verbs after nouns, `--json` on every read command, destructive verbs confirm
  unless `--yes`, plus the Phase-0 exit-code table.
- **Normalization:** the L0–L8 profile ladder and `NormalizationPipeline` live in
  `crates/application/src/quran_normalize.rs`; search indexing must pin a profile
  version and record it in provenance.

## 3. Invariants Phase 2 is bound by

| ID | Invariant | Enforcement |
|---|---|---|
| I1 | Canonical tables are insert-only | DB triggers + `crates/storage-sqlite/tests/quran.rs` |
| I2 | No LLM/embeddings/retrieval/vector-store dependency in `quran-core` / `quran-corpus` | `xtask arch-check` + `xtask/allowlist.toml` |
| I3 | `edition_id` is part of every canonical key | DDL PKs |
| I5 | Import cannot activate; a human approval is required | `application::quran` activation service |
| I6 | Quotation only via `QuranQuotation` | Constructor visibility |
| I7 | Activation is a single transaction + generation bump | `rollback_edition`/`activate` services |
| I10 | Normalization spans are composable and reversible | `quran-normalization` `SpanMap` |

Also: the reader cache is keyed on `(edition_id, version, corpus_generation, ref,
options_hash)` — a generation change must invalidate wholesale (AC-P1-19). `doctor`
must remain strictly read-only (Phase-0 AC-P0-14).

## 4. Constraints carried forward

- `domain`/`quran-core` dependency sets are enforced by `cargo xtask arch-check`
  against `xtask/allowlist.toml` (name-keyed; register new crates there, fail-closed).
- Migrations are append-only and checksummed (`migrations/sqlite/checksums.json`);
  `xtask migrate-check` requires versions contiguous from 1. Phase 2 continues from
  `0013`.
- Canonical rows are insert-only (DB triggers); deactivation uses tombstones.
- Secrets never persist in SQLite — store `SecretRef`s only.
- Timestamps are UTC RFC3339; hashes are lowercase `sha256:<hex>`.
- Divisions are numbered **globally per kind** (`(kind, number)` PK) — see DEV-03.

## 5. Known limitations / deferred items

| Item | Owner | Notes |
|---|---|---|
| ADR-0101 dataset + license + named editorial reviewer | Swimlane X (owner) | Engineering runs on the synthetic `test-edition-min` fixture; ADR-0101 stays Draft (OWN-01) |
| ADR-0114 reference corpus + sign-off procedure | Swimlane X (owner) | QV-015 is skip-when-unconfigured, never a silent pass (OWN-02) |
| ADR-0111 / ADR-0112 | — (accepted 2026-09-15) | Accepted on shipped-code conditions (resolver + citation endpoint, P1-T46/T47; translation import + type guards, P1-T36/T37); see done.md P1-T42/P1-T53 |
| Estimate gap 82 ed vs 131.0 ed summed | phase owner | Proceeded incrementally; no silent compression (OWN-03) |
| axum + tower-http | phase owner | Adopted provisionally for API v1; ADR still to ratify (OWN-04) |
| Phase-0 exit discrepancy | phase owner | `status.md` lists outstanding Phase-0 items while the build prompt declared Phase 0 complete (OWN-05) |
| XML adapter shape | Phase 2+ | JSON + CSV only; the `Adapter` trait supports adding XML unchanged (DEV-01) |
| `doctor --quran --deep` < 30 s on a **standard** edition | Phase 1 exit | 19 checks, read-only open, and schema-valid single JSON document are verified; the fixture `--deep` runs in 0.07 s but a full 6236-ayah edition needs a real dataset (ADR-0101) to time |
| `cli` snapshot coverage for `diff`/`rollback`/`context`/`surah` | done | `read_flow.trycmd` now covers every AC-P1-16 verb plus the edition lifecycle (`validate` → activate gen 2 → `diff` → rollback gen 3 → `hashes`) (P1-T50) |
| Phase-0 `application/src/db.rs` schema-version tests | done | Now derive the expected version from `migrations/sqlite/` instead of hard-coding it |
| `server` → `storage` / `server` → `tools` layering edge | Phase 3 / server hardening | `ReaderBackend` uses the `storage::Database` trait and `tools::ToolError` directly; allowlisted to keep `arch-check` green (OWN-06). Route through `application` re-exports and tighten the allowlist |
| Flaky `jobs::worker::tests::retries_then_succeeds` | Phase 0 / jobs | Passes in isolation (3/3) but intermittently returns `Idle` instead of `Succeeded` under the fully-parallel workspace run; a queue-timing race, not a Phase-1 regression. Blocks the "15 suites green" gate intermittently, so de-flake before the exit ritual |

## 6. Deviations Phase 2 must honour

- **DEV-01** — No XML adapter; JSON + CSV prove adapter extensibility.
- **DEV-02** — Phase-1 migrations are `0007`–`0012` (not the plan's `0010`–`0015`);
  `migrate-check` requires contiguity. Phase 2 continues from `0013`.
- **DEV-03** — Division numbers are globally unique per kind (`ruku`/`rub` cumulative);
  per-surah ruku stays available via the ayah `ruku` column.

## 7. Suggested first Phase-2 tasks

1. Read `docs/03-plan/phases/phase-02-rag/execution-plan.md`; it reconciles the
   migration mapping (`0013`–`0018`) and the FTS5-first fallback (DEV-05).
2. Land the normalization catalog/profile seed and bind it to the code ladder with a
   test (`tests/normalization_seed.rs`) so migration seed and code cannot drift.
3. Build the first search index *from the reader*, keyed on `corpus_generation`, and
   prove no stale hits after activation.
4. Add search/normalization checks to the existing `doctor` registry rather than a
   parallel doctor.
5. Extend the API v1 envelope + OpenAPI spec for search; keep the route-coverage test.

## 8. Phase-1 closure state at handoff

- 50 / 65 task rows ☑ (2 ◐ / 13 ☐, excluding 3 sequencing notes); 19 / 21
  acceptance criteria partial (automated-green, ritual pending; ☐ are AC-P1-01
  dataset/license and AC-P1-18 web font); 12 / 14 ADRs Accepted (0101/0114 Draft
  pending owners); 6 / 6 Phase-1 migrations applied and checksummed.
- Phase-1 exit gate (`done.md` §8) is **not** signed: it requires the Swimlane-X
  decisions, 15 green suites, and the 9-step ritual recording. The five D1.14 docs
  are now published (5/5), and the read/CLI/doctor surfaces are automated-green.
- Phase 2 is not blocked by the *engineering* of Phase 1 — only by the owner/editorial
  decisions above, which have external lead time and should be started now.

## 9. Addendum — 2026-09-24 (engineering only; no gate signed)

- **Typed comparison (P1-T26 Tier-2):** `quran-corpus::differ` now exports
  `ComparisonKind`, `DifferenceClass` (ADR-0114 literals), `classify_difference`,
  `diff_ayahs_typed`, `ComparisonOperands`, and `CLASSIFICATION_VOCABULARY`.
  `EditionDiff` carries `comparison_kind` + `normalization_applied`;
  `AyahChange` carries `classification` (all `#[serde(default)]`, legacy
  reports deserialize). Phase 2 comparison surfaces should build on these
  types, not a second vocabulary. QV-015 Tier-1 exact behavior unchanged;
  ADR-0114 still Draft; T26 still ◐ on the Tier-2 corpus + sign-off.
- **Editorial verification recording (P1-T55 plumbing):**
  `QuranRepository::set_edition_verification` (metadata-only write),
  approval-gated `application::quran::record_edition_verification`
  (`SourceApproved` audit, `QAI-QUR-0306` on empty reviewer), and
  `qai quran edition verify --reviewer --method`. `verified_by` /
  `verification_method` are already in the edition `--json`; the reviewer
  name itself remains OD-02. T55 still ☐.
- Owner gates unchanged: P1-X01…X05 `_unassigned_`; P1-T02/T03 ☐;
  P1-T56 ☐ (no fabricated scripture); P1-T58 ◐ (fixture soak re-passes);
  P1-T60 ☐ (ritual + coverage + deny outstanding).
