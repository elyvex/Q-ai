# Phase 3: Quran Search & Linguistics - Pattern Map

**Mapped:** 2026-09-26
**Files analyzed:** 23 (14 modify, 9 create)
**Analogs found:** 20 / 23 (3 partial/no clean analog — see §No Analog Found)
**Gap source:** `.planning/phases/03-quran-search-linguistics/03-RESEARCH.md` §Gap Register (G-01…G-13) + Evidence Matrix SC1–SC5. All paths below are git-tracked source (verified `git ls-files`).

> **Reading rule for the planner:** every excerpt is real, cited `path:line`. When a plan action names a file, copy the cited analog pattern — do not re-invent. The legacy `phase-02-rag` board is evidence, not a mandate (D-04).

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality | Gap(s) |
|-------------------|------|-----------|----------------|---------------|--------|
| `crates/application/src/quran_counting.rs` | service | aggregation (read-only SQL) | same file: `frequency` + `rules_for` | exact | G-01, G-07 |
| `crates/application/src/quran_morphology.rs` | service | request-response / import-activate | same file: `activate_morphology`, `build_same_root_relations`, `affix_search` | exact | G-02, G-04, G-05, G-09 |
| `crates/storage/src/quran.rs` | repository (trait) | CRUD / aggregation | same file: `analyses_for_root`, `count_tokens_matching_form` | exact | G-01, G-07 |
| `crates/storage-sqlite/src/quran.rs` | repository (impl) | CRUD / aggregation | same file: `analyses_for_root`, `count_tokens_matching_form` | exact | G-01, G-07 |
| `crates/application/src/quran_cli.rs` | controller (CLI handler) | request-response | same file: `cmd_count_frequency`, `cmd_morphology_root`, `cmd_morphology_import` | exact | G-01, G-02, G-04 |
| `crates/cli/src/quran.rs` | controller (CLI routing) | request-response | same file: `CountAction`, `MorphologyAction`, dispatch `L845-932` | exact | G-02, G-04, G-13 |
| `crates/server/src/api.rs` | controller (HTTP route) | request-response | same file: `search_normalized_handler`, `router` | exact | G-02, G-10 |
| `crates/tool-registry/src/lib.rs` | registry / service | request-response | same file: `tool_names`, `get_ayah` | role-match | G-11 |
| `crates/application/src/quran_tools.rs` | service / adapter | request-response | same file: `ReaderToolBackend`, `QuranBackend` impl | role-match | G-11 |
| `crates/quran-morphology/src/family.rs` | model / utility | transform | same file: `FamilyRelation`, `explain_relation` | exact | G-05 |
| `crates/quran-normalization/src/rules/mod.rs` | config / doc | — | same file (doc comment only) | exact | G-12 |
| `migrations/sqlite/00NN_license_evidence.up.sql` (new) | migration | — | `0017_quran_lexicon.up.sql` | role-match | G-04 |
| `fixtures/quran/search/concatenated.jsonl` (new) | fixture (golden) | batch / transform | `fixtures/quran/search/queries.jsonl` | exact | G-03 |
| `fixtures/quran/lexicon/families/curated.jsonl` (new) | fixture (golden) | batch | `fixtures/quran/lexicon/sample-a.json` | role-match | G-05 |
| `fixtures/quran/morphology/license-matrix.json` (new) | fixture / config | — | `licenses/README.md` capture schema | partial | G-04 |
| `crates/application/tests/counting.rs` | test (integration) | CRUD | same file | exact | G-01, G-07 |
| `crates/application/tests/search_goldens.rs` | test (golden) | batch | same file | exact | G-03 |
| `crates/application/tests/morphology_import.rs` | test (integration) | import-activate | same file | exact | G-04, G-05 |
| `crates/application/tests/family_goldens.rs` (new) | test (golden) | batch | `crates/application/tests/search_goldens.rs` | role-match | G-05 |
| `crates/application/tests/doctor_indexes.rs` | test (integration) | request-response | same file | exact | G-08 |
| `crates/application/tests/search_latency.rs` | test (bench harness) | batch | same file | exact | G-06 |
| `crates/cli/tests/quran/family.trycmd` (new) | test (CLI snapshot) | request-response | `crates/cli/tests/quran/counting_graph.trycmd` | role-match | G-02 |
| `crates/cli/tests/quran/counting_graph.trycmd` | test (CLI snapshot) | request-response | same file | exact | G-01 |

---

## Pattern Assignments

### `crates/application/src/quran_counting.rs` (service, aggregation)

**Analog:** itself — extend, do not restructure.

**Imports + module shape** (`crates/application/src/quran_counting.rs:1-22`): module doc + `pub const CODE_PREFIX: &str = "QAI-CNT";`. All errors are `CountingError` with append-only `QAI-CNT-N` codes.

**Mandatory rules block — the contract every numeric output must carry (I15)** (`crates/application/src/quran_counting.rs:35-58`):
```rust
pub struct CountingRules {
    pub profile: String,                                  // e.g. "L3.diacritics"
    pub profile_version: String,
    pub datasets: Vec<String>,                            // ["stored-forms"] until a lexicon activates
    pub multi_analysis_handling: MultiAnalysisHandling,   // SingleSource | AllAnalyses | OneVotePerToken
    pub window: Option<String>,                           // "token:N" for co-occurrence
    pub exclusions: Vec<String>,
}
impl CountingRules { pub fn canonical_json(&self) -> String { serde_json::to_string(self).expect("CountingRules serializes") } }
```

**G-01 target — existing `frequency` shape to mirror for root/lemma** (`crates/application/src/quran_counting.rs:224-259`): normalize target → resolve active edition → one `db.write()` UoW → exact `COUNT(*)` + per-surah breakdown → `rollback()` → assemble `FrequencyReport { target, rules, count, by_surah, checksum }`. The `checksum` is `format!("sha256:{}", quran_corpus::sha256_hex(canonical.as_bytes()))` (`:204-206`).

**G-01 current stubs to replace** (`crates/application/src/quran_counting.rs:716-731`):
```rust
pub async fn root_frequency(_db: &SqliteDatabase, _root: &str) -> Result<FrequencyReport, CountingError> {
    Err(CountingError::UnavailableDataset { capability: "root frequency".to_string() })
}
pub async fn lemma_frequency(_db: &SqliteDatabase, _lemma: &str) -> Result<FrequencyReport, CountingError> {
    Err(CountingError::UnavailableDataset { capability: "lemma frequency".to_string() })
}
```
Replace body with lexicon joins via `require_active_dataset` + storage methods (below), keeping `UnavailableDataset` as the no-dataset fallback.

**G-07 — `rules_for` currently hard-wires `SingleSource`** (`crates/application/src/quran_counting.rs:186-202`):
```rust
Ok(CountingRules {
    profile: profile.to_string(),
    profile_version: version.to_string(),
    datasets: vec!["stored-forms".to_string()],
    multi_analysis_handling: MultiAnalysisHandling::SingleSource,
    window, exclusions: Vec::new(),
})
```
Add a `multi_analysis_handling` (and `datasets`) parameter or a second constructor so `AllAnalyses`/`OneVotePerToken` are selectable and visibly change both the count and the rules block. Keep `canonical_json` field order fixed (checksum input).

**Repository call pattern to reuse** (`frequency`): typed accessor via `uow.quran().count_tokens_matching_form(...)` / `..._by_surah(...)` (`:233-247`).

**Test expected-code discipline** (`crates/application/src/quran_counting.rs:746-760`): the in-module `tests::codes_carry_cnt_prefix` asserts every `CountingError` carries the `QAI-CNT` prefix — extend it if a new variant is added.

---

### `crates/application/src/quran_morphology.rs` (service, import-activate)

**Analog:** itself.

**G-04 — `MorphologyImportParams` already carries license fields; the CLI hardcodes them** (`crates/application/src/quran_morphology.rs:147-170`): `license_status: String`, `license_json: String`, `attribution: String`. Add license-evidence params here if the gate needs more than the dataset row.

**G-04 — activation gate: insert the license/attribution rejection here** (`crates/application/src/quran_morphology.rs:550-615`). Current ordered gates:
```rust
// 1. batch.state == "staged"                 (:567-574)
// 2. approval.decision == Some("approved")   (:577-586)
// 3. approval.subject_urn == dataset_urn(slug, version)   (:587-593)
// 4. fatal + error findings == 0             (:594-601)
// 5. zero unmatched alignment rows           (:602-615)
```
Add gate **6** before promotion: consult the machine-readable license matrix; reject when `license_status` ∈ {`Unspecified`, `metadata_only`, `pending_license_review`} or when required capture fields are absent. Reuse the error idiom `uow.rollback()…; return Err(MorphologyJobError::…)` — do not add a new error enum unless necessary.

**G-05 — family builders live here; `build_same_root_relations` is the template** (`crates/application/src/quran_morphology.rs:1746-1805`): one `db.write()` UoW → `list_roots(dataset_id)` → per-root `analyses_for_root` → build `FamilyRelationRow` with mandatory non-empty `explanation`, `status: "proposed"`, `evidence_json`, `corpus_generation` → `insert_family_relations` → `commit()`. Note the constructor check `quran_morphology::FamilyMember::new(&from, "token", &explanation)?` (`:1776`). Add sibling builders for `same_form`/`same_lemma`/`same_stem`/`derived`/`inflectional`/`affix` on this shape; relation strings must match the `0017` CHECK domain.

**G-02 — `word_family` read path (already exists; only surface is missing)** (`crates/application/src/quran_morphology.rs:1808-1830`): returns `(dataset_id, Vec<FamilyMemberView>)`, gated by `require_active_dataset(db, "word family")` (`:893-903`). The CLI/API gap is a *surface* gap, not a service gap — wire it, do not rewrite it.

**G-09 — `affix_search` silent empty to replace with a typed error** (`crates/application/src/quran_morphology.rs:1679-1695`):
```rust
if active.is_some() {
    // Dataset backend: morpheme-exact scan over active analyses.
    return Ok(Vec::new());   // ← silent empty (G-09): return a typed UnavailableDataset instead
}
```
The L7 heuristic backend (`:1696-1727`) is the labeled fallback; keep it.

**Error idiom for tool services** (`crates/application/src/quran_morphology.rs:829-890`): `MorphologyToolError` with `Diagnostic` mapping; `UnavailableDataset` → `QAI-MORPH-0004` (asserted in tests).

**Import report MV-018 field** (`:174-191`): `mv018_unchanged: bool` — every import path reports it (SC5).

---

### `crates/storage/src/quran.rs` (repository trait, CRUD)

**Analog:** itself.

**G-01 — add lexicon aggregation methods beside the existing analysis readers.** Trait method style (`crates/storage/src/quran.rs:1246-1262`):
```rust
async fn analyses_for_root(&self, _dataset_id: &str, _root_normalized: &str) -> Result<Vec<TokenAnalysisRow>, StorageError> { Err(StorageError::StorageUnavailable) }
async fn analyses_for_lemma(&self, _dataset_id: &str, _lemma: &str) -> Result<Vec<TokenAnalysisRow>, StorageError> { Err(StorageError::StorageUnavailable) }
```
Existing form-count methods to mirror (`crates/storage/src/quran.rs:939-969`): `count_tokens_matching_form`, `count_tokens_matching_form_by_surah`, `list_all_token_forms`. Add e.g. `count_analyses_for_root_by_surah` / `count_analyses_for_lemma_by_surah` (or a single grouped-count) so the service never loads all rows to count.

**Default-impl convention:** every new trait method gets a `StorageError::StorageUnavailable` body (the trait provides defaults so non-SQLite stores compile).

---

### `crates/storage-sqlite/src/quran.rs` (repository impl, CRUD)

**Analog:** itself.

**G-01/G-07 — SQL join template for root/lemma aggregation** (`crates/storage-sqlite/src/quran.rs:2458-2504`):
```rust
async fn analyses_for_root(&self, dataset_id: &str, root_normalized: &str) -> Result<Vec<TokenAnalysisRow>, StorageError> {
    let mut tx = self.tx.lock().await;
    let rows = sqlx::query(
        "SELECT a.id, a.dataset_id, a.edition_id, a.surah, a.ayah, a.token_position,
                a.analysis_index, a.surface, a.lemma_id, a.root_id, a.stem, a.pos_unified,
                a.pos_native, a.features_json, a.segments_json, a.provenance_layer,
                a.algorithm, a.algorithm_version, a.confidence, a.reviewer, a.status,
                a.corpus_generation, a.created_at
         FROM quran_token_analyses a JOIN quran_roots r ON a.root_id = r.id
         WHERE a.dataset_id = ? AND r.root_normalized = ?
         ORDER BY a.surah, a.ayah, a.token_position, a.analysis_index",
    )
    .bind(dataset_id).bind(root_normalized)
    .fetch_all(&mut **tx).await.map_err(map_sqlx_error)?;
    Ok(rows.iter().map(decode_analysis).collect())
}
```
For counting, replace the SELECT with `SELECT a.surah, COUNT(*) … GROUP BY a.surah` (or `COUNT(DISTINCT (surah,ayah,token_position))` for `OneVotePerToken`). Lemma variant joins `quran_lemmas l ON a.lemma_id = l.id` (`:2482-2504`). Always bind typed params — never string-concatenate (Security §V5).

**Insert/read family template** (`:2535-2593`): `insert_family_relations` iterates rows inside one `tx`; `family_relations_for` matches either side `(from_kind,from_id) OR (to_kind,to_id)`.

---

### `crates/application/src/quran_cli.rs` (controller, request-response)

**Analog:** itself.

**G-01/G-02 — thin handler template** (`crates/application/src/quran_cli.rs:2907-2920`):
```rust
pub async fn cmd_morphology_root(db_path: &str, root: &str) -> CommandOutput {
    use super::quran_morphology::root_search;
    let db = match open_db(db_path).await { Ok(db) => db, Err(error) => return CommandOutput::err(exit::INTERNAL, error.to_string()) };
    match root_search(&db, root).await {
        Ok((dataset, occurrences)) => json_or_err(
            format!("root {root}: {} occurrence(s) in {dataset}", occurrences.len()),
            &serde_json::json!({"dataset": dataset, "occurrences": occurrences}),
        ),
        Err(error) => CommandOutput::err(tool_exit(&error), error.to_string()),
    }
}
```
Add `cmd_family(db_path, kind, id)` on this exact shape, using `tool_exit` for `MorphologyToolError` (`:2590-2599`).

**Counting handlers** use `json_or_err(...)` + `counting_exit(&error)` (`:2391-2399`, `:2402-2418`): `json_or_err(format!("…"), &report)`. Add `cmd_count_root_frequency` / `cmd_count_lemma_frequency` here.

**G-04 — license plumbing currently hardcoded** (`crates/application/src/quran_cli.rs:2630-2642`):
```rust
license_status: "Unspecified".to_string(),
license_json: "{}".to_string(),
```
Thread CLI-provided license evidence through `cmd_morphology_import` params into `MorphologyImportParams`, so the activation gate has something to reject on.

**Exit-code maps** (`:20-39`): `OK=0, GENERIC=1, USAGE=2, VALIDATION=3, POLICY=4, NOT_FOUND=5, CONFLICT=6, CANCELLED=7, INTERNAL=70`; `LOCAL_PRINCIPAL` at `:69`.

---

### `crates/cli/src/quran.rs` (controller, CLI routing)

**Analog:** itself.

**G-02 — add a top-level/`MorphologyAction` family variant** following `RootAction`/`MorphologyAction` (`crates/cli/src/quran.rs:419-431`, `:526-620`): `#[derive(Subcommand)] pub enum …` with doc-commented fields and `#[arg(...)]` defaults. The top-level `QuranAction` variants are at `:14-236` (`Search`, `Root`, `Count`, `Morphology`, `Graph`).

**Dispatch pattern to extend** (`crates/cli/src/quran.rs:880-932`): `QuranAction::Morphology { action } => match action { … MorphologyAction::Root { root } => application::quran_cli::cmd_morphology_root(db_path, &root).await, … }`. Add `MorphologyAction::Family { kind, id }` (or a top-level `Family`) → `cmd_family`.

**Counting dispatch** (`:845-879`): `CountAction::Frequency { target, profile } => cmd_count_frequency(...)`. Add `CountAction::RootFrequency`/`LemmaFrequency`.

**G-13 — naming divergence:** D-10 names `qai quran search/root/lemma/morphology/family/freq`; today root/lemma search live under `morphology`, and there is no `family`/`freq`. Either add thin aliases or record the divergence explicitly (Discretion area).

---

### `crates/server/src/api.rs` (controller, HTTP route)

**Analog:** itself.

**G-02/G-10 — route table to extend** (`crates/server/src/api.rs:1289-1337`): routes are registered `.route("/api/v1/quran/…", get|post(handler))`. Search/normalization block:
```rust
.route("/api/v1/quran/normalization/preview", post(normalization_preview_handler))
.route("/api/v1/quran/normalization/profiles", get(normalization_profiles_handler))
.route("/api/v1/quran/search/exact", post(search_exact_handler))
.route("/api/v1/quran/search/normalized", post(search_normalized_handler))
…
```
Add `/api/v1/quran/family` (and count/morphology routes if Q1 resolves to include them). Layers (trace, body limit 1 MiB, 30 s timeout, concurrency 128) already wrap all routes.

**Handler template** (`crates/server/src/api.rs:1043-1069`): parse body → validate non-empty → build args → call `state.search.…` → `search_response(&headers, "quran.<tool>", output, started)` on Ok, `search_error_response(error)` on Err. New SC3/SC4 routes need a backend on `AppState` (`:149-158`: `tools`, `api: Arc<dyn QuranApiBackend>`, `search: Arc<dyn SearchBackend>`) — either extend `SearchBackend` or add a new trait + a `SearchApiService`-style impl (`crates/application/src/quran_search_api.rs:249-327`).

**Error → HTTP status mapping** (`:828-861` for search; `:212-228` `tool_status`): reuse, do not hand-roll.

**Wiring** (`crates/cli/src/lib.rs:315-324`): `AppState { tools, api, search }` is constructed in the CLI `serve` arm — a new backend field must be wired there too.

---

### `crates/tool-registry/src/lib.rs` (registry, request-response)

**Analog:** itself.

**G-11 — `tool_names` and the two registered tools** (`crates/tool-registry/src/lib.rs:152-155`):
```rust
pub fn tool_names(&self) -> Vec<&'static str> {
    vec!["quran.get_ayah", "quran.get_context"]
}
```
**Tool-result envelope pattern** (`:161-203`): build a `ToolResult { tool_name, tool_version, query, normalization_rules, edition_id, edition_version, canonical_references, analysis_sources, results, confidence, warnings, execution_time_ms, reproducibility }`. New search/root/lemma/morphology/family tools must populate `normalization_rules` (from the `NormalizationTrace`, I9) and `analysis_sources` (attribution), and use `reproducibility(...)` (`crates/tools`).

**Backend trait** (`:49-63`): `QuranBackend` with `backend_get_ayah`/`backend_get_context`. Adding search/lexicon tools means either extending this trait or adding a second backend trait — the registry holds one `Arc<dyn QuranBackend>` (`:142-144`).

**Test convention** (`:267-388`): `FakeBackend` + `registry_lists_both_tools` asserts the exact `tool_names` list — update it when registering new tools.

---

### `crates/application/src/quran_tools.rs` (service/adapter, request-response)

**Analog:** itself.

**G-11 — backend impl surface** (`crates/application/src/quran_tools.rs:70-128`): `ReaderToolBackend { reader }` + `impl QuranBackend for ReaderToolBackend`; `ReaderToolBackend::registry(reader) -> ToolRegistry` (`:82-84`). Search/root/lemma/morphology/family backends must be constructed and handed to `ToolRegistry::new` exactly like this (wired in `crates/cli/src/lib.rs:312-314`).

**Meta assembly pattern** (`:33-68` `backend_meta`): reads the active edition + generation through a `db.write()` UoW with `rollback()`, maps errors to `ToolError::Backend { code, detail }`. Reuse for attribution fields on new tools.

**Error bridge** (`:24-31`): `to_tool_error(ReaderError) -> ToolError` — new backends need the equivalent bridge from their own error enum.

---

### `crates/quran-morphology/src/family.rs` (model/utility, transform)

**Analog:** itself.

**Typed relation taxonomy** (`crates/quran-morphology/src/family.rs:19-50`): closed `FamilyRelation` enum → `relation_name()` returns the exact snake_case strings the `0017` CHECK accepts (`same_form`, `same_lemma`, `same_stem`, `same_root`, `derived`, `inflectional`, `affix`, `computational_suggestion`). New builders must call `relation_name` rather than hard-code strings.

**Mandatory explanation + kind validation** (`:66-94`): `FamilyMember::new` rejects empty id/explanation and unknown kinds. Builders must go through it (as `build_same_root_relations` does).

**Explanation generator to reuse** (`:101-144` `explain_relation`): computes shared/differing surface/lemma/root/stem/pos features — use it for `same_*` explanations so each relation states exactly what it claims.

**Suggestion discipline** (`:146-245`): `SUGGESTION_LABEL`, `SUGGESTION_CONFIDENCE_FLOOR = 0.50`, `suggest_computational` (opt-in), `ReviewPromotion::new` (reviewer + timestamp + evidence).

---

### `crates/quran-normalization/src/rules/mod.rs` (doc/config)

**G-12 — doc drift only.** The module doc claims "Rules N18–N24 … are not implemented here" while `heuristic_rules()` implements N18–N22 and `by_id` returns `None` for N23/N24 only. Correct the doc comment; no behavior change. Verify `cargo fmt --check` + `clippy` clean.

---

### `migrations/sqlite/00NN_license_evidence.up.sql` (new migration)

**Analog:** `migrations/sqlite/0017_quran_lexicon.up.sql`.

**House style** (`migrations/sqlite/0017_quran_lexicon.up.sql:1-29`): header comment cites phase/task/ADR; forward-only, **no `.down.sql`** (Phase-1 convention); `CHECK (… IN (…))` constraints; `corpus_generation INTEGER NOT NULL DEFAULT 0`; `created_at TEXT NOT NULL`; `UNIQUE(...)`. The `quran_datasets` table already carries `license_status TEXT NOT NULL`, `license_json TEXT NOT NULL DEFAULT '{}'`, `attribution TEXT NOT NULL` (`:21-23`) — a license-matrix table can be additive (per-artifact evidence rows) or the gate can read the dataset row + a matrix file; pick one and record the decision.

**Allocation rule:** new numbers allocate from the actual directory (`0021` is newest today — `0021_canonical_write_fence`); never renumber an applied migration. `cargo xtask migrate-check` verifies checksums. If no schema is needed, **do not add a migration** — a machine-readable JSON matrix under `licenses/` may suffice; record which.

---

### `fixtures/quran/search/concatenated.jsonl` (new fixture, golden)

**Analog:** `fixtures/quran/search/queries.jsonl`.

**Header contract** (`fixtures/quran/search/queries.jsonl:1`):
```json
{"header": true, "version": 1, "generated_by": "scripts/gen_search_goldens.py", "generated_at": "2026-09-23", "corpus": "fixtures/quran/test-edition-min", "reviewed_by": "pending-linguist", "reviewed_at": null, "note": "Mechanics oracle on synthetic text …"}
```
**Row contract** (exact/normalized/phrase/regex rows; `:2`): `{"tool": "...", "input": "...", "expected_references": ["1:1"], "expected_total": 2, "must_not_contain": ["1:2"]}`. For concatenated add segmentation fields (e.g. expected `query_part`→`canonical_token` tiling), the `بسمالله`→1:1 segmentation case, cross-ayah cases, and Persian-codepoint cases (G-03 / acceptance.md 120-case set). Keep `reviewed_by: pending-linguist`.

**Oracle generator precedent:** `scripts/gen_search_goldens.py` (`:1-70`) computes expected refs independently in Python (pure string matching, no Rust stack). Author the concatenated oracle the same way — independent, deterministic.

---

### `fixtures/quran/lexicon/families/curated.jsonl` (new fixture)

**Analog:** `fixtures/quran/lexicon/sample-a.json`.

**Synthetic lexicon row shape** (`fixtures/quran/lexicon/sample-a.json:1-22`): array of `{sura_no, aya_no, tok_idx, analysis_no, surface_form, lemma_str, root_str, stem_str, tag_native, tag_unified, layer, algo, algo_ver, conf, state, reviewer, segmented, morphs, synthetic_test_only: true}`. The `synthetic_test_only: true` label is mandatory (Pitfall 4). `intermediate-schema.json` in the same dir is the schema reference; `sample-b.csv` is the CSV adapter variant. A 120-family curated set stays `reviewed_by: pending-linguist` until OD-12 closes.

---

### `fixtures/quran/morphology/license-matrix.json` (new; no clean analog — see §No Analog Found)

**Best reference:** `licenses/README.md` `capture.json` schema (`licenses/README.md:24-48`): `artifact`, `source_url`, `capture_date`, `capturer`, `spdx_id`, `license_sha256`, `redistribution_allowed`, `modification_allowed`, `attribution_required`, `notes`. `source_url`/`capture_date`/`capturer` are mandatory; `redistribution_allowed: true` with all fields is the bundling gate. The matrix should be a superset keyed by artifact and consumed by the G-04 activation gate. No such machine-readable file exists today — planner must define the schema (Discretion area) and cite `licenses/README.md`.

---

### `crates/application/tests/counting.rs` (test, integration)

**Analog:** itself.

**DB harness** (`crates/application/tests/counting.rs:37-116` `ready_db`): tempdir + `apply_migrations("…/migrations/sqlite")` + `SqliteDatabase::new(…, 4, true)` + raw seed inserts (principal/source/source_version/approval) + `run_import` + `activate_edition` + `rebuild_forms`. Copy this for lexicon counts (extend to `run_morphology_import` + `activate_morphology`).

**G-01/G-07 test shape to add** (`:131-152`, `:243-256`): assert `count > 0`, `rules.profile`, breakdown sums to total, `checksum.starts_with("sha256:")`, determinism (same rules ⇒ same count/checksum), rule-relativity (`other.rules.profile != first.rules.profile`), and the typed `QAI-CNT-0005` `UnavailableDataset` when no dataset is active. Add a mode-matrix test where `AllAnalyses` vs `OneVotePerToken` changes the count **and** the `CountingRules.multi_analysis_handling` field (AC-P2-28).

**Imports block** (`:10-22`): already imports `root_frequency`, `lemma_frequency`, `unusual_usage`.

---

### `crates/application/tests/search_goldens.rs` (test, golden)

**Analog:** itself.

**G-03 — extend the golden loader/runner.** Header parse + row deserialize (`crates/application/tests/search_goldens.rs:155-179`):
```rust
#[derive(serde::Deserialize)] struct GoldenRow { tool: String, input: String, expected_references: Vec<String>, expected_total: u64, must_not_contain: Vec<String> }
// load_goldens(): read header, assert header["reviewed_by"] == "pending-linguist", parse rows
```
Tool dispatch match (`:204-289`) currently handles `search_exact|search_normalized|search_phrase|search_regex`; add a `search_concatenated` arm calling `search_concatenated(&db, &data_dir, &base_params(...), allow_cross_ayah, max_ayah_span)` and assert segmentation tiling. The fixture path constant is `FIXTURE` (`:30-31`) — point a second loader at `concatenated.jsonl` (do not widen the 400-count assertion at `:177`; add a separate count assertion for the new set).

**Setup harness** (`:45-140` `searchable_db`): migrates + imports + activates + `rebuild_forms` + `rebuild_index` (`data_dir = dir.path().join("index")`). Reuse verbatim.

---

### `crates/application/tests/morphology_import.rs` (test, integration)

**Analog:** itself.

**Aligned-document builder** (`crates/application/tests/morphology_import.rs:139-173`): builds an array-shape doc from real fixture tokens, **two competing analyses per token** (multi-analysis read path), `synthetic_test_only: true`. Reuse for license-gate + family-builder tests.

**Import params (license passthrough)** (`:175-189`): `license_status: "PublicDomain"`, `license_json: "{}"` — the G-04 test flips this to `"Unspecified"` and asserts activation is rejected (new `license_gate` test).

**Activation gate test template** (`:288-355`): missing approval → `MorphologyJobError::Approval`; wrong subject → `Approval`; correct approval → active with promoted rows; then read tools return attributed analyses. Add a sibling `license_gate_rejects` test using this exact shape.

**Family test template** (`:445-471`): import → activate → `build_same_root_relations(&db, "@")` → assert `built > 0` → `word_family(&db, "token", "token:1:1:1")` → all `explanation` non-empty, relation typed. Extend for the new builders + a non-merge assertion.

**Typed-unavailable test** (`:357-401`): before activation every tool returns `MorphologyToolError::UnavailableDataset` with code `QAI-MORPH-0004`.

**`ready_db` seed helper:** `migrated_db` (`:45-94`) + `ready_db` (`:96-137`) seed the morph approval `appr-morph` covering `morphology-dataset:test-morph@0.1.0`.

---

### `crates/application/tests/family_goldens.rs` (new test, golden)

**Analog:** `crates/application/tests/search_goldens.rs` (structure) + `morphology_import.rs` (setup).

Load `fixtures/quran/lexicon/families/curated.jsonl` with the header/row/`pending-linguist` pattern from `search_goldens.rs:155-179`; set up via `searchable_db`-style import+activate from `morphology_import.rs:139-189`; build all family relation types; assert each curated family's members + typed relations + non-empty explanations. Keep `reviewed_by: pending-linguist` until OD-12.

---

### `crates/application/tests/doctor_indexes.rs` (test, integration)

**Analog:** itself.

**G-08 — 19-check assertions + harness** (`crates/application/tests/doctor_indexes.rs:150-321`): `index_checks_are_nineteen_with_stable_ids`, `index_checks_go_green_on_built_index`, `doctor_indexes_is_read_only`, `missing_generations_fail_loudly`, `index_json_matches_doctor_checks_shape`. `status_of(checks, id)` helper (`:137-145`). Extend: run after a full rebuild→activate→search soak on an **activated dataset** and assert the morphology/lexicon/trigram checks green + `QAI-IDX-0101` on injected drift. `INDEX_CHECK_IDS` is at `crates/application/src/quran_doctor_indexes.rs:25-45` (19 ids).

---

### `crates/application/tests/search_latency.rs` (test, bench harness)

**Analog:** itself.

**G-06 — budget harness** (`crates/application/tests/search_latency.rs:1-33`): `const FIXTURE_BOUND: Duration = Duration::from_millis(2000)` with a doc note that the plan §17.1 p50/p99 targets gate full-corpus runs, not the 14-ayah fixture. Add a machine-readable budget artifact (ADR-0207 `p99 ≤ 150 ms` concatenated) and a threshold check; record full-corpus execution as OD-11-dependent.

---

### `crates/cli/tests/quran/family.trycmd` (new CLI snapshot)

**Analog:** `crates/cli/tests/quran/counting_graph.trycmd`.

**trycmd setup + command block** (`crates/cli/tests/quran/counting_graph.trycmd:1-33`): header comment, `$ qai db migrate`, `$ qai quran import …`, `$ qai quran activate … --yes`, `$ qai quran forms rebuild …` (expects the `MV-018 canonical-unchanged: pass (sha256:…)` line). With no active dataset, `qai quran family …` must print the typed unavailability as a usage/policy error (mirror `counting_graph.trycmd:37-42` `? 2` / `error: unknown counting profile`). Add an activated-family snapshot once a synthetic dataset path is exercised.

---

### `crates/cli/tests/quran/counting_graph.trycmd` (modify)

**Analog:** itself. Add `qai quran count root-frequency …` / `lemma-frequency …` blocks asserting the typed `UnavailableDataset` error (no dataset) and, where possible, an activated count with the `CountingRules` line.

---

## Shared Patterns

### Explainability payload is type-mandatory (I9/I10/D-11)
**Source:** `crates/quran-search/src/hit.rs:97-137` (`SearchHitParts`) + `:139-147` (`SearchHit`, private fields, validating constructor).
**Apply to:** every new result type (family views, count reports, tool results).
```rust
// Fields are private; the only constructor validates and requires:
//   quotation: QuranQuotation, canonical_span: CanonicalSpan,
//   explanation: NormalizationTrace (I9 — mandatory, never optional),
//   segmentation: Vec<Segmentation> (concatenated only).
```
Never introduce a search/lexicon result that bypasses the canonical quotation + span + trace.

### One normalizing path, query and index (R6)
**Source:** `crates/application/src/quran_search.rs:1050-1065` (`empty_trace_for`, `latest_version`) + `:1083-1087` (`persian_fold_differs`); proof gate `crates/quran-search/tests/search_parity.rs`.
**Apply to:** any new search mode and every derived build.
```rust
let registry = db_registry(db).await?;
let profile = quran_normalization::ProfileId::L3; // L3.diacritics default
let pipeline = quran_normalization::NormalizationPipeline::for_profile(&registry, profile, latest_version(&registry, profile)?)?;
let trace = empty_trace_for(&pipeline);
let normalized_query = normalize_token(&pipeline, &params.text);
```

### Typed capability-unavailable, never guessed data (D-05 fallback)
**Source:** `crates/application/src/quran_counting.rs:718-731` (`UnavailableDataset`), `crates/application/src/quran_morphology.rs:893-903` (`require_active_dataset`), error→code maps at `quran_counting.rs:89-99` / `quran_morphology.rs:859-890`.
**Apply to:** every lexicon-gated tool (root/lemma/family/affix/pattern) and every numeric lexicon count.

### Generation-keyed cache + derived rebuild (I14)
**Source:** `crates/application/src/quran_search_cache.rs:34-138` (key `v1:{generation}:{sha256(canonical_json)}`, lookup verifies stored generation, store/invalidate), `crates/quran-search/src/model.rs:226-244` (`IndexManifest`).
**Apply to:** any new derived artifact — bump generation via rebuild; do not add an uncached/generation-blind path.

### Canonical-unchanged post-check (I8/SC5)
**Source:** `crates/application/src/quran_forms.rs:222-255` (`verify_canonical_unchanged`/`_in`); import reports `mv018_unchanged` (`quran_morphology.rs:189-190`); doctor check `quran.canonical_unchanged` (`quran_doctor_indexes.rs:507-526`).
**Apply to:** every derived build and morphology activation — assert after, never mutate canonical bytes.

### Diagnostic errors with stable `QAI-*` codes (ADR-0010)
**Source:** `crates/application/src/quran_counting.rs:22,89-121`; `crates/quran-search/src/error.rs:90-106`.
**Apply to:** all new service errors — append-only numbering within the prefix (`QAI-CNT-*`, `QAI-MORPH-*`, `QAI-IDX-*`), plus `summary`/`remedy`/`next_command`.

### SQL binding discipline (Security V5)
**Source:** `crates/storage-sqlite/src/quran.rs:2458-2593` (typed `.bind(...)`, never concatenation).
**Apply to:** all new lexicon count/family queries.

### Purity fence (ADR-0012 / AC-P2-36)
**Source:** `xtask/allowlist.toml:82-99` — `quran-normalization`/`quran-search`/`quran-morphology` must never gain `llm`/`embeddings`/`retrieval`/vector-store deps.
**Apply to:** every Phase-3 change in those crates; `cargo run -q -p xtask -- arch-check` is the gate.

---

## No Analog Found

Files with no direct precedent in the codebase; the planner should define the shape (Discretion area) and cite the closest reference doc.

| File | Role | Data Flow | Reason / Closest Reference |
|------|------|-----------|----------------------------|
| `fixtures/quran/morphology/license-matrix.json` | fixture/config | — | No machine-readable license matrix exists; only the human-readable capture schema in `licenses/README.md:24-48`. Define the schema; keep it keyed per artifact and consumable by the G-04 gate. |
| `crates/application/tests/family_goldens.rs` | test (golden) | batch | No family golden test exists (P2-T92 ☐). Compose the golden-loader pattern from `search_goldens.rs:155-179` with the morphology setup from `morphology_import.rs:139-189`. |
| `crates/application/src/quran_tools.rs` new search/lexicon tool backends | service/adapter | request-response | Only `backend_get_ayah`/`backend_get_context` exist; no search/root/lemma/morphology/family tool backend precedent. Follow the `ReaderToolBackend` + `ToolResult` envelope shape (`quran_tools.rs:70-128`, `tool-registry/src/lib.rs:161-203`). |

---

## Owner Gates (must appear as explicit BLOCKED in every relevant plan)

| ID | Gate | Closing step |
|----|------|--------------|
| OD-11 | Morphology dataset selection & licensing (ADR-0203 A vs B) | Capture QAC license to `licenses/qac/` per `licenses/README.md`; ratify Option A or keep Option B import path. |
| OD-12 | Normalization catalog + named linguist (ADR-0204/0205) | Name a qualified Arabic linguist; record sign-off in `docs/reviews/`; flip ADRs to Accepted. Goldens stay `reviewed_by: pending-linguist` until then. |
| D-08 | Linguist sign-off on tagset/root conventions/alignment (ADR-0210/0215) | Same engagement as OD-12. |

**Default posture (per D-05/D-07/A5):** satisfy SC3/SC4 **behaviorally** on the synthetic lexicon with prominent `synthetic_test_only` labels; keep ADR-0203/0210/0215 + goldens **not Accepted**; the license/attribution gate must still **fail closed** in code.

---

## Metadata

**Analog search scope:** `crates/{application,cli,server,storage,storage-sqlite,quran-morphology,quran-normalization,quran-search,tool-registry}/`, `migrations/sqlite/`, `fixtures/quran/`, `licenses/`, `scripts/`.
**Files scanned:** 23 target files + 20 analogs (via `graft file_api` skeletons + targeted `Read` line ranges).
**Tracked-source check:** `git ls-files` returned 15/15 for the checked analog paths (no `.gsd/capabilities` mirrors used).
**Pattern extraction date:** 2026-09-26
**Source of record:** `03-CONTEXT.md` (D-01…D-15), `03-RESEARCH.md` (Evidence Matrix SC1–SC5, Gap Register G-01…G-13, Validation Architecture, Security Domain).
