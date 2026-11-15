---
phase: 06-hadith-tafsir
verified: 2026-11-15T15:17:03Z
status: passed
score: 19/20 must-haves verified
covered_files:
  - ".planning/phases/06-hadith-tafsir/06-01-PLAN.md"
  - ".planning/phases/06-hadith-tafsir/06-01-SUMMARY.md"
  - ".planning/phases/06-hadith-tafsir/06-02-PLAN.md"
  - ".planning/phases/06-hadith-tafsir/06-02-SUMMARY.md"
  - ".planning/phases/06-hadith-tafsir/06-03-PLAN.md"
  - ".planning/phases/06-hadith-tafsir/06-03-SUMMARY.md"
  - ".planning/phases/06-hadith-tafsir/06-04-PLAN.md"
  - ".planning/phases/06-hadith-tafsir/06-04-SUMMARY.md"
  - ".planning/phases/06-hadith-tafsir/06-05-PLAN.md"
  - ".planning/phases/06-hadith-tafsir/06-05-SUMMARY.md"
  - "crates/application/src/disagreement.rs"
  - "crates/application/src/hadith_reader.rs"
  - "crates/application/src/hadith_search.rs"
  - "crates/application/src/hadith_tools.rs"
  - "crates/application/src/lib.rs"
  - "crates/application/src/quran_tools.rs"
  - "crates/application/src/tafsir_reader.rs"
  - "crates/application/src/tafsir_search.rs"
  - "crates/application/src/tafsir_tools.rs"
  - "crates/application/src/verse_link.rs"
  - "crates/cli/src/hadith.rs"
  - "crates/cli/src/lib.rs"
  - "crates/cli/src/links.rs"
  - "crates/cli/src/tafsir.rs"
  - "crates/cli/tests/parity_cross_surface.rs"
  - "crates/domain/src/ids.rs"
  - "crates/hadith-core/src/collection.rs"
  - "crates/hadith-core/src/grading.rs"
  - "crates/hadith-core/src/lib.rs"
  - "crates/hadith-core/src/record.rs"
  - "crates/hadith-ingestion/src/lib.rs"
  - "crates/hadith-ingestion/src/parser.rs"
  - "crates/hadith-ingestion/src/validate.rs"
  - "crates/server/src/api.rs"
  - "crates/server/tests/api.rs"
  - "crates/server/tests/citation_open.rs"
  - "crates/server/tests/graph.rs"
  - "crates/tafsir/src/lib.rs"
  - "crates/tafsir/src/record.rs"
  - "crates/tafsir/src/work.rs"
  - "crates/tool-registry/src/lib.rs"
  - "migrations/sqlite/0023_hadith_collections.up.sql"
  - "migrations/sqlite/0024_hadith_records.up.sql"
  - "migrations/sqlite/0025_hadith_gradings.up.sql"
  - "migrations/sqlite/0026_hadith_search_forms.up.sql"
  - "migrations/sqlite/0027_tafsir_works.up.sql"
  - "migrations/sqlite/0028_tafsir_records.up.sql"
  - "migrations/sqlite/0029_tafsir_search_forms.up.sql"
  - "migrations/sqlite/0030_verse_links.up.sql"
  - "xtask/allowlist.toml"
covered_digest: "v3:sha256:eb59d73bb6b152601db83502a4f3fce03c5891d073da4f6233133f1f152732c8"
behavior_unverified: 1
overrides_applied: 0
re_verification:
  previous_status: passed
  previous_score: 19/20
  gaps_closed: []
  gaps_remaining: []
  regressions: []
deferred:
  - truth: "Cross-surface parity three-leg rows for hadith.get / hadith.search / hadith.gradings / tafsir.get / tafsir.search"
    addressed_in: "Phase 11/13 (test-coverage backlog)"
    evidence: "crates/cli/tests/parity_cross_surface.rs fixtures_cover_all_tool_names still asserts these five names are uncovered ('unexpected uncovered tools (hadith/tafsir/scripture parity lands later)'). Dispatch arms, research_checksum shape, and the shared application service layer all exist; test-coverage only, owner-accepted under autonomous mode."
  - truth: "Comparative scripture (Torah, Tanakh, New Testament) linked at the data layer"
    addressed_in: "Phase 9"
    evidence: "Delivered since the prior report: crates/scripture/{book,edition,translation,verse,reference}.rs plus application::scripture_{reader,search,tools}; dispatch arms scripture.get/search/translations/editions/parallel_passages registered in quran_tools.rs:962-981; verse_links gained a scripture source in 09-03."
behavior_unverified_items:
  - truth: "Hadith and tafsir tools return byte-identical payloads and identical research_checksum across CLI, HTTP, and TUI"
    test: "Add five hadith.* / tafsir.* fixture rows to crates/cli/tests/parity_cross_surface.rs and drive them through all three legs"
    expected: "Identical normalized payload (minus execution_time_ms) and identical research_checksum for hadith.get, hadith.search, hadith.gradings, tafsir.get, tafsir.search"
    why_human: "No automated parity rows exist for these five tools; the parity gate explicitly whitelists them as uncovered. Presence + wiring verified (dispatch arms in quran_tools.rs, one shared application service layer), runtime three-leg equivalence unexercised. Test-coverage only — not a product behavior gap; owner-accepted."
---

# Phase 6: Hadith & Tafsir — Verification Report

**Phase Goal:** Users can research structured hadith and tafsir linked to exact Quran locations with attributed gradings.
**Verified:** 2026-11-15
**Status:** passed
**Re-verification:** Yes — fingerprint refresh at HEAD `67b47f6` (covered files changed by phases 7–13). Every claim below was re-checked against current source; the previous report's statements were not carried over unverified.

## Goal Achievement

### Observable Truths

| #  | Truth (ROADMAP SC → derived must-have) | Status | Evidence at HEAD |
|----|----------------------------------------|--------|------------------|
| 1 | SC1 — Hadith record insert/retrieve preserves collection, edition, and source numbering verbatim | ✓ VERIFIED | `HadithRecord{source_number, matn_text, matn_hash, isnad_text}` (`hadith-core/src/record.rs:113-131`), `HadithCollection{slug,name,version,...}` (`collection.rs:99-121`); edition carried as `edition_slug/edition_version/edition_id` in `BackendMeta` (`hadith_tools.rs:103-113`); migration 0024 `source_number NOT NULL CHECK(length(trim(...)) > 0)`, `UNIQUE(collection_id, source_number)`; test `insert_and_retrieve_record` |
| 2 | SC2 — Matn/isnad separation where source supports it; single-field otherwise | ✓ VERIFIED | `matn_text: String` + `isnad_text: Option<String>`; migration 0024 `isnad_text TEXT` (nullable); tests `insert_and_retrieve_record`, `record_without_isnad_roundtrips` (None case), `record_roundtrips_serde` |
| 3 | SC2 — Each grading is an independent per-scholar record with attribution and source citation, no consensus field | ✓ VERIFIED | migration 0025 `scholar_name NOT NULL CHECK(...)`, `source_citation NOT NULL`, `supersedes_id`, `provenance_layer CHECK IN ('B','C','D')`, no consensus column anywhere; `GradingRecord` (`grading.rs:149-170`); ingestion rejects merged gradings (`validate_rejects_consensus_extra`, `pipeline_rejects_consensus_grading`) |
| 4 | SC1 — Hadith exact + normalized search works without mutating canonical text | ✓ VERIFIED | `hadith_search.rs` writes only to `hadith_search_forms` and `hadith_fts` (lines 198/212/233) — no UPDATE/INSERT against `hadith_records.matn_text`; tests `search_returns_results`, `normalized_search_matches` |
| 5 | Hadith typed tool returns ToolResult with research_checksum + canonical references + gradings | ✓ VERIFIED | `hadith_tools.rs` `ToolResult{tool_name, tool_version, query, normalization_rules, results, canonical_references, analysis_sources, reproducibility, research_checksum}` (lines 138-175); get view carries `gradings[]` with `provenance_layer` (lines 89-103); tests `hadith_get_returns_view_with_gradings`, `hadith_search_returns_tool_result_with_checksum`, `hadith_gradings_returns_all_with_attribution`, `hadith_search_empty_query_is_invalid_input` |
| 6 | Tafsir record insert/retrieve with work, author, dimensions | ✓ VERIFIED | `TafsirWork{slug,version,name,author,dimensions_schema_json}` (`tafsir/src/work.rs:133-150`), `TafsirDimensions{volume,page,chapter}`, `TafsirRecord{work_id,volume,page,chapter,text,...}` (`record.rs:65-84`); migration 0027/0028; tests `insert_and_retrieve_work`, `insert_and_retrieve_record`, `records_for_verse_uses_chapter_organization` |
| 7 | Tafsir works indexed by work/author/dimensions following the catalog pattern | ✓ VERIFIED | migration 0027 `UNIQUE(slug, version)`, `author NOT NULL CHECK(length(trim(author)) > 0)`, `dimensions_schema_json`; `ix_tafsir_works_status`; reader lists/fetches by slug+version |
| 8 | Tafsir exact + normalized search works without mutating canonical text | ✓ VERIFIED | `tafsir_search.rs` writes only `tafsir_search_forms` / `tafsir_fts` (lines 216/230/250); canonical `text` untouched; tests `search_returns_results`, `normalized_search_matches` |
| 9 | Tafsir typed tool returns ToolResult with research_checksum + canonical references | ✓ VERIFIED | `tafsir_tools.rs:163-176` builds `ToolResult` with `canonical_references`, `analysis_sources`, `research_checksum`; tests `tafsir_get_returns_view_with_work_and_dimensions`, `tafsir_search_returns_tool_result_with_checksum`, `tafsir_search_checksum_is_deterministic` |
| 10 | SC5 — Computational verse links carry confidence score and 'uncertain' label | ✓ VERIFIED | migration 0030 `confidence REAL CHECK(0..1)`, `label DEFAULT 'uncertain' CHECK IN (...)`; `VerseLink::suggested` forces `Uncertain`; test `suggested_links_start_uncertain_and_never_authoritative`, `confidence_is_clamped_to_unit_range` |
| 11 | SC5 — Only owner-confirmed links presented as authoritative (read path hides uncertain by default) | ✓ VERIFIED | `get_links_for_verse` uses `AND label = 'confirmed'` when `include_uncertain = false` (`verse_link.rs:680-691`); test `read_path_hides_uncertain_links_by_default` |
| 12 | Owner gate requires reviewer + decided_at for confirmed/rejected | ✓ VERIFIED | DB `CHECK (label != 'confirmed' OR (reviewer IS NOT NULL AND reviewer != '' AND decided_at IS NOT NULL))` (0030); code rejects empty reviewer (`verse_link.rs:596-598`) and only updates `WHERE label = 'uncertain'`; tests `owner_gate_confirms_and_rejects_with_audit_trail`, `database_check_rejects_confirmed_without_reviewer` |
| 13 | SC4 — Scholarly disagreement shown side-by-side, sorted by scholar, never hidden/merged/resolved | ✓ VERIFIED | User-facing path: `HadithToolBackend` returns *every* grading sorted by scholar (`hadith_tools.rs:80`, `:192`) with `scholar_name`, `grading_value`, `source_citation`, `provenance_layer`; no dedup/merge/filter exists on the path. `disagreement.rs` implements the same assembly (total map + `sort_by(scholar_name)`, `:86-104`) — see warning on its lack of a consumer |
| 14 | SC5 — All computational identity matches labeled uncertain, never as certainty | ✓ VERIFIED | `VerseLink::suggested` hard-sets `LinkLabel::Uncertain` regardless of confidence; disagreement view labels layer-D / confidence-bearing gradings `"uncertain"` (`disagreement.rs:89-93`); tests `suggested_links_start_uncertain_and_never_authoritative`, `suggest_links_creates_uncertain_links_with_scores`, `suggest_links_works_for_tafsir_sources` |
| 15 | Labeling/thresholds applied: high (>0.9), medium (0.7–0.9), low (<0.7), all uncertain | ✓ VERIFIED | `ConfidenceBand::of` (`verse_link.rs:178-187`) exact boundaries; boundary tests `0.95→High, 0.9001→High, 0.9→Medium, 0.7→Medium, 0.6999→Low` |
| 16 | CLI commands exist: hadith get/search/gradings, tafsir get/search | ✓ VERIFIED | `HadithAction::{Get,Search,Gradings}` (`cli/src/hadith.rs:12-24`), `TafsirAction::{Get,Search}` (`cli/src/tafsir.rs:12-26`); registered in the top-level `Commands` enum and dispatched (`cli/src/lib.rs:131-149`, `:610-614`); each arm calls the application service (no direct storage access) |
| 17 | HTTP routes exist for /api/v1/hadith/* and /api/v1/tafsir/* | ✓ VERIFIED | Router lines 2243-2248; handlers call `state.tools.{get_hadith,search_hadith,get_gradings,get_tafsir,search_tafsir}` and set `meta.research_checksum` from the tool result; server tests 4+26+3+11 green |
| 18 | SC3 — User can navigate from any verse to linked hadith/tafsir passages | ✓ VERIFIED | HTTP `GET /api/v1/verse/{surah}/{ayah}/links` → `VerseLinker::get_links_for_verse` on `AppState.verse_links` (`api.rs:2248`, `:2441-2453`); CLI `qai links verse <surah> <ayah> [--include-uncertain]` (`cli/src/links.rs`); no longer the 06-04 stub (empty list) |
| 19 | Hadith/tafsir tools byte-identical via CLI, HTTP, TUI with identical research_checksum | ⚠️ PRESENT_BEHAVIOR_UNVERIFIED | Dispatch arms for all five tools exist (`quran_tools.rs:843-882`), checksum shape identical, all surfaces share the application service layer — but the parity gate explicitly whitelists them as uncovered (`parity_cross_surface.rs:436-465`). Owner-accepted, test-coverage only |
| 20 | Ingestion pipeline discover→stage→validate→activate for hadith/tafsir sources | ✓ VERIFIED | `IngestionPipeline::{discover,stage,validate,activate,staged,active}` (`hadith-ingestion/src/lib.rs:46-123`); 10 tests green incl. `pipeline_full_flow`, `pipeline_source_number_preserved`, `activate_nothing_staged_returns_error`; still in-memory (`Mutex<HashMap>`), SQLite persistence deferred per the 06-04 contract |

**Score:** 19/20 truths verified (1 present, behavior-unverified)
**Behavior-unverified:** 1 (truth 19 — cross-surface parity rows)

### Deferred Items

| # | Item | Addressed In | Evidence |
|---|------|-------------|----------|
| 1 | Cross-surface parity three-leg rows for the five hadith/tafsir tools | Phase 11/13 (backlog) | `fixtures_cover_all_tool_names` still whitelists `hadith.get/gradings/search` + `tafsir.get/search` as uncovered — outstanding at HEAD, owner-accepted |
| 2 | Comparative scripture (Torah, Tanakh, NT) at the data layer | Phase 9 — **now delivered** | `crates/scripture/*` + `application::scripture_{reader,search,tools}`; `scripture.get/search/translations/editions/parallel_passages` dispatch arms (`quran_tools.rs:962-981`); verse_links scripture source (09-03) |
| 3 | Full narrator identity graph | Phase 7 — **now delivered** | `application::isnad_reader`, `isnad_graph_store`, `isnad.*` tools, `qai isnad` CLI (phase 7 commits) |

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `crates/hadith-core/src/{record,collection,grading}.rs` | domain types | ✓ VERIFIED | 225/191/250 lines; `ProvenanceLayer{B,C,D}`, no consensus field |
| `crates/tafsir/src/{work,record}.rs` | tafsir domain types | ✓ VERIFIED | 226/164 lines; author non-empty invariant |
| `migrations/sqlite/0023–0030` | hadith/tafsir/verse_links schema | ✓ VERIFIED | 23–37 lines each, all forward-only with CHECK constraints; `migrate-check`: 44 migrations ordered, checksums stable |
| `crates/application/src/hadith_{reader,search,tools}.rs` | reader/search/tool services | ✓ VERIFIED | 634/391/407 lines; 4+2+4 module tests |
| `crates/application/src/tafsir_{reader,search,tools}.rs` | reader/search/tool services | ✓ VERIFIED | 492/412/356 lines; 4+2+4 module tests |
| `crates/application/src/verse_link.rs` | matching + owner gate + read path | ✓ VERIFIED | 1111 lines; 13 module tests |
| `crates/application/src/disagreement.rs` | side-by-side assembly | ⚠️ ORPHANED | 249 lines, correct assembly (map-all + sort by scholar) and `DisagreementSource` impls for `SqliteDatabase` / `HadithReaderService`, but nothing constructs `DisagreementService` — no CLI/HTTP/TUI consumer. SC4 is delivered through `hadith.get` / `hadith.gradings` instead (same invariant: every grading, sorted by scholar). Only a serde roundtrip test |
| `crates/hadith-ingestion/src/*` | parser/validate/pipeline | ✓ VERIFIED | 345/153/108 lines; 10 tests; in-memory only (accepted) |
| `crates/cli/src/{hadith,tafsir,links}.rs` | CLI commands | ✓ VERIFIED | 141/114/67 lines; dispatched from `cli/src/lib.rs` |
| `crates/server/src/api.rs` | hadith/tafsir/verse-links routes | ✓ VERIFIED | routes 2243-2248; real tool calls, checksum propagated |
| `crates/tool-registry/src/lib.rs` | `HadithBackend`/`TafsirBackend` traits + TOOL_NAMES | ✓ VERIFIED | 2338 lines; five phase-6 tool names registered |
| `xtask/allowlist.toml` | hadith-ingestion allowed; no forbidden edges | ✓ VERIFIED | `arch-check: OK — no forbidden dependency edges` |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| HTTP `/api/v1/hadith/{collection}/{source_number}` | application hadith tools | `state.tools.get_hadith` → `HadithToolBackend` → `HadithReaderService` | WIRED | real query, checksum in envelope meta |
| HTTP `/api/v1/hadith/search`, `/gradings` | application hadith services | `search_hadith` / `get_gradings` | WIRED | no direct storage access from the route |
| HTTP `/api/v1/tafsir/{work_slug}`, `/tafsir/search` | application tafsir services | `get_tafsir` / `search_tafsir` | WIRED | checksum propagated |
| HTTP `/api/v1/verse/{surah}/{ayah}/links` | `VerseLinker::get_links_for_verse` | `AppState.verse_links` | WIRED | confirmed-only by default, `include_uncertain` opt-in |
| CLI `qai hadith|tafsir|links` | application service layer | `handle_*` → service constructors | WIRED | DB handle from config `storage.sqlite.path` |
| Tool dispatch (`dispatch_registered_tool`) | hadith/tafsir backends | name→arm table | WIRED | `hadith.get/search/gradings`, `tafsir.get/search` arms (`quran_tools.rs:843-882`) |
| Ingestion pipeline | hadith/tafsir tables | discover→stage→validate→activate | PARTIAL | pipeline is real and tested but in-memory; no SQLite write path (accepted deferral per 06-04 contract) |
| `disagreement.rs` assembly | per-scholar grading records | `DisagreementSource::fetch_gradings` | ORPHANED | implemented + correct, no surface constructs it |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `HadithReaderService::get_record` | `matn_text`, `isnad_text`, `source_number` | `SELECT ... FROM hadith_records` | Yes | ✓ FLOWING |
| `HadithSearchService::search` | result rows | `hadith_fts` / `hadith_search_forms` (populated from real records) | Yes | ✓ FLOWING |
| `TafsirReaderService` / `TafsirSearchService` | `text`, dimensions | `tafsir_records` / `tafsir_search_forms` | Yes | ✓ FLOWING |
| `verse_links_handler` | `links` | `SELECT ... FROM verse_links WHERE target_surah/ayah` (confirmed filter) | Yes | ✓ FLOWING |
| `hadith.get` / `hadith.gradings` | `gradings[]` | `SELECT ... FROM hadith_gradings WHERE hadith_id = ?` | Yes | ✓ FLOWING |
| CLI human output | first-line counts/summary | tool payload (`serde_json::to_value`) | Yes (JSON is full payload; human line is a summary) | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| hadith domain types | `cargo test -p hadith-core` | 12 passed, 0 failed | ✓ PASS |
| tafsir domain types | `cargo test -p tafsir` | 8 passed, 0 failed | ✓ PASS |
| ingestion pipeline | `cargo test -p hadith-ingestion` | 10 passed, 0 failed | ✓ PASS |
| application services (reader/search/tools/verse_link/disagreement) | `cargo test -p application --lib` | 103 passed, 0 failed | ✓ PASS |
| cross-surface parity gate | `cargo test -p cli --test parity_cross_surface` | 5 passed, 0 failed | ✓ PASS (hadith/tafsir rows still whitelisted as uncovered) |
| server API routes | `cargo test -p server` | 4 + 26 + 3 + 11 passed, 0 failed | ✓ PASS |
| workspace regression (recorded, HEAD) | `cargo test --workspace --no-fail-fast` (log `/tmp/qai-cargo-test.log`) | 203 suites ok, 0 failed; only `soak_full` terminated by SIGTERM (operator-stopped 50k soak, re-running separately) | ✓ PASS |
| dependency architecture | `cargo run -p xtask -- arch-check` | `arch-check: OK — no forbidden dependency edges` | ✓ PASS |
| migration ordering/checksums | `cargo run -p xtask -- migrate-check` | `OK — 44 migration(s) ordered; checksums stable` | ✓ PASS |

### Probe Execution

| Probe | Command | Result | Status |
|-------|---------|--------|--------|
| None declared | n/a | n/a | SKIPPED (no probes declared for this phase; `scripts/*/tests/probe-*.sh` not used by phase 6) |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| REQ-hadith-corpus | 06-01, 06-03, 06-04, 06-05 | Source-aware structured hadith records with metadata, Shia collections, attributed grading, hadith tools | ✓ SATISFIED | collection/edition/source_number preserved (truth 1), matn/isnad (2), per-scholar gradings with attribution+citation and no consensus (3), exact+normalized search (4), typed tools with checksum (5, 16, 17), verse navigation (18), uncertainty labels (10–15), ingestion (20). Shia support is `tradition IN ('sunni','shia')` (0023) — metadata only, per locked CONTEXT decision (no code-path split) |
| REQ-tafsir-comparative | 06-02, 06-03, 06-04, 06-05 (+ Phase 9 for scripture) | Tafsir indexed by work/author/dimensions; comparative scripture with comparative tools | ✓ SATISFIED | Phase 6 scope: works/author/dimensions (6, 7), search (8), tools (9, 16, 17), verse navigation (18). Comparative-scripture half now delivered in Phase 9 (`crates/scripture`, `scripture.*` tools, `parallel_passages`), so the requirement is satisfied at HEAD |
| REQ-data-separation-layers | cross-cutting (mapped to Phase 2; Phase 6 must not violate) | Explicit trust layers A/B/C/D/E | ✓ SATISFIED (Phase 6 contribution) | Phase 6 keeps layers explicit: `hadith_gradings.provenance_layer CHECK IN ('B','C','D')` (0025) and `ProvenanceLayer` enum (`grading.rs:105-123`); computational links carry `label` + confidence instead of being promoted to canonical (0030); tool payloads expose `provenance_layer` per grading (`hadith_tools.rs:99`). No Phase 6 path writes layer-D output into canonical text or a canonical table |

No ORPHANED requirements: all three IDs are claimed by at least one plan or accounted for as cross-cutting, and all appear in REQUIREMENTS.md.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `crates/application/src/disagreement.rs` | 68-106 | Module exported (`lib.rs:10`) but no surface constructs `DisagreementService`; `assemble` has no caller | ⚠️ Warning | Dead code alongside the live path. SC4 is still delivered by `hadith.get`/`hadith.gradings` (same map-all + sort-by-scholar invariant), so no user-facing gap — but two implementations of one rule can drift |
| `crates/application/src/disagreement.rs` | 230-248 | Only test is a serde roundtrip; no multi-grading fixture asserts ordering/never-merge | ⚠️ Warning | The ordering invariant is enforced by `sort_by` in code (`:103`) and by `hadith_tools.rs:80,192`, but untested with ≥2 gradings on either path |
| `crates/cli/tests/parity_cross_surface.rs` | 436-465 | Five hadith/tafsir tools explicitly whitelisted as uncovered | ℹ️ Info | Known owner-accepted coverage gap (truth 19), unchanged from prior report |
| — | — | `TBD`/`FIXME`/`XXX`/`HACK`/`TODO`/`PLACEHOLDER`/`unimplemented!`/`todo!` in all phase-6 source + migrations | ✓ None | Clean — no debt markers, no unreferenced stubs |

### Deferred Human Verification (owner-accepted)

- **Cross-surface parity for the five hadith/tafsir tools.** Dispatch arms, identical `research_checksum` construction, and one shared application service layer are all present, but `fixtures_cover_all_tool_names` still asserts these names are uncovered. Recorded as a deferred item (test-coverage only, not a product behavior gap) under autonomous-mode owner acceptance. To close: add five fixture rows to `crates/cli/tests/parity_cross_surface.rs` and drive all three legs.
- **Ingestion persistence.** `IngestionPipeline` stages/activates in memory (`Mutex<HashMap>`); no SQLite write path. Deferred per the 06-04 contract; the truth as written (discover→stage→validate→activate) holds and is tested.

### Gaps Summary

No blocking gaps. At HEAD the phase goal is observably true: hadith records preserve collection/edition/source numbering with matn/isnad separation, gradings are per-scholar with attribution and citation and no consensus field, both corpora are searchable through non-canonical search-form/FTS tables, typed tools return `ToolResult` with `research_checksum` and canonical references over CLI and HTTP (with dispatch arms ready for TUI), verse→hadith/tafsir navigation is reachable from both HTTP and `qai links verse`, computational links are confidence-scored and uncertain-by-default with a reviewer-gated confirmation path, and the hadith ingestion pipeline passes its 10 tests. Phase-6 tests (12 + 8 + 10 + 103 + 5 + 44), the recorded workspace run (203 suites, 0 failures; only the operator-stopped `soak_full`), `arch-check`, and `migrate-check` are all green at HEAD.

Two warnings are recorded and neither blocks the goal: `disagreement.rs` is orphaned dead code (the same side-by-side/sort-by-scholar rule is delivered live through `hadith.get`/`hadith.gradings`), and the multi-grading ordering invariant has no ≥2-grading test on either path. The single behavior-unverified truth remains cross-surface parity coverage for the five hadith/tafsir tools — test-coverage only, owner-accepted, tracked as deferred.

---

_Verified: 2026-11-15T15:17:03Z_
_Verifier: the agent (gsd-verifier)_
