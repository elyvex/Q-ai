# Research: Search-Service Wiring

**Feature**: `030-search-service-wiring` | **Date**: 2026-09-18

The spec carries zero `[NEEDS CLARIFICATION]` markers and names exact code symbols; this phase verifies each grounding claim against the tree and records the resulting decisions. All unknowns resolved — no open research items.

## Decision 1 — CLI shape: `Search` variant on `QuranAction` + `cmd_search` in `application::quran_cli`

- **Decision**: Add a `Search { ... }` variant to `QuranAction` (`crates/cli/src/quran.rs`, alongside `Get`/`Context`/`Normalize`), a dispatch arm in `handle_quran_async` (same file), and a `cmd_search(db_path, ...) -> CommandOutput` free function in `crates/application/src/quran_cli.rs`. Argument parsing and human/`--json` formatting live in `cli`; all database/index access goes through `application`.
- **Rationale**: Verified in code — `quran.rs` documents the rule in its module header ("Argument parsing and output formatting live here; every database access goes through `application::quran_cli`"), `QuranAction` currently has read/edition/import/index/normalize verbs but no search verb, and `handle_quran_async` dispatches each variant to a `cmd_*` function returning `CommandOutput { exit, human, json }`. Following the pattern keeps `arch-check` green.
- **Alternatives considered**: One subcommand per tool (`search exact`, `search regex`, …) — rejected: five near-identical verbs for one service family; a single `--tool` selector (FR-002) keeps the surface minimal. Putting execution in `cli` directly — rejected: violates the crate's own documented layering and `arch-check`.

## Decision 2 — Tool selection via `--tool`, default `search_normalized`

- **Decision**: `--tool {search_exact,search_normalized,search_phrase,search_concatenated,search_regex}`, default `search_normalized`. The flag value maps 1:1 to the five `application::quran_search` service functions; HTTP uses five distinct GET paths instead of a selector (REST convention, matches existing `/ayahs/{reference}` style).
- **Rationale**: Service names are already the stable public vocabulary (tool-registry spec 023, PRD §11). CLI flag values and HTTP path segments use the same suffixes (`exact`, `normalized`, `phrase`, `concatenated`, `regex`) so the misuse matrix and docs stay uniform.
- **Alternatives considered**: Separate CLI subcommands per tool — rejected per Decision 1. A single HTTP route with `?tool=` — rejected: diverges from the existing per-resource route style in `api.rs` (`editions`, `surahs`, `ayahs`, `context`, `divisions`, `tokens`, `resolve`, `citations`).

## Decision 3 — Profile exclusivity enforced twice: clap `conflicts_with` + `NormalizedProfile` by construction

- **Decision**: `--profile` / `--rules` (CLI) and `profile` / `rules` (HTTP) are declared mutually exclusive at the parse layer (clap `conflicts_with`, HTTP 400 on both-present), and the parsed selection is built into `application::quran_search::NormalizedProfile::Registry(id, version?) | Adhoc(ids)` — a type for which "both" is unrepresentable (`quran_search.rs:883-888`: "the plan's 'never both' rule holds by construction, not by runtime check"). Version pinning reuses the existing `profile[@version]` syntax from the `Normalize` verb (`quran.rs:160-165`).
- **Rationale**: Verified — `Normalize` already established the `profile`/`rules` UX and the `L3.diacritics[@version]` / `N01,N03` value shapes; reusing them makes search flags learnable. Double enforcement (parse + type) matches the codebase's defense-in-depth style.
- **Alternatives considered**: Runtime-only check in `cmd_search` — rejected: weaker, and contradicts the existing by-construction design. New value syntax — rejected: gratuitous inconsistency with `quran normalize`.

## Decision 4 — HTTP: five GET routes + one SSE route under `/api/v1/quran/search/`, reusing envelope helpers

- **Decision**: `GET /api/v1/quran/search/{exact,normalized,phrase,concatenated,regex}` returning `Envelope<SearchEnvelope>` via the existing `json_response` + `etag_for` + `Meta` helpers (`api.rs:67-93, 202-266`), plus `GET /api/v1/quran/search/stream` returning `axum::response::Sse`. All routes registered in `router()` (`api.rs:799-833`) behind the same loopback guard that `router()` already enforces (spec edge case: non-loopback `--bind` refusal applies unchanged — verified the guard lives at serve/bind time, `api.rs:846`, so new routes inherit it with no code).
- **Rationale**: Verified — `api.rs` module header states the conventions (`ETag` from text-hash+generation, `Content-Language`, `Diagnostic` error bodies); `Meta` already carries edition, generation, canonical reference, deep link, timing, reproducibility JSON, and warnings — everything section-12 needs except tool identity/query/rules, which the `SearchEnvelope` data payload supplies. axum 0.8 ships `Sse` with event framing, matching the spec's hit-events-plus-terminal-summary shape.
- **Alternatives considered**: WebSocket or chunked JSON — rejected: SSE is unidirectional server→client, matches "stream of envelope-framed events," and needs no new dependency. POST search routes — rejected: searches are safe/idempotent reads; GET keeps ETag/conditional-request semantics working.

## Decision 5 — ETag and reproducibility from existing pieces

- **Decision**: HTTP ETag reuses `etag_for(&meta)` (derived from edition text-hash + generation); the reproducibility checksum comes from `tools::ReproducibilityData` (verified fields: `checksum`, `query_hash`, `tool_plan: name@version`, `source_versions`, `edition_versions`, `normalization_rule_set`, `corpus_generation`, `deterministic`) serialized into `Meta.reproducibility`. CLI `--json` embeds the same `ToolResult`-shaped contract (verified `ToolResult` fields in `tools/src/lib.rs:56-81`: tool_name/version, query, normalization_rules, edition id/version, results, canonical_references, analysis_sources, confidence, warnings, execution_time_ms, reproducibility).
- **Rationale**: Both halves of the section-12 contract already exist as types; the wiring only assembles them. CLI↔HTTP parity (SC-002) falls out of sharing `SearchOutput` as the single source of hit shape.
- **Alternatives considered**: New checksum scheme for search — rejected: `ReproducibilityData` already covers deterministic inputs + edition versions + rule set.

## Decision 6 — Error mapping extends the existing mappers

- **Decision**: `SearchError` already implements `storage::error::Diagnostic` with stable codes (`QAI-IDX-*`, `QAI-NORM-*`, verified `quran_search.rs:159-198`, including `EditionNotIndexed → QAI-IDX-2`, `NoServingIndex → QAI-IDX-3`, `RateLimited → QAI-IDX-7`). CLI maps via the existing `storage_error_code` helper in `crates/cli/src/lib.rs` (verified present) extended to the search code range → exit codes per FR-012 (usage 2, validation 3, policy 4, not-found 5, conflict 6, internal 70 — verified constants in `exit_code.rs`). Server maps via the existing `Diagnostic → HTTP status` conversion used by current read routes (400/404/409/429/5xx per FR-016).
- **Rationale**: No new error taxonomy; the codes and both mapping sites predate this feature. `NoServingIndex`'s remedy ("Run `qai quran index rebuild` first") already yields the spec's required not-found/state failure without stack traces.
- **Alternatives considered**: Search-specific error enum at the wiring layer — rejected: would duplicate `SearchError` and fork the diagnostic codes.

## Decision 7 — Cache reuse, no new layer

- **Decision**: Both surfaces call `cache_lookup` / `cache_store` / generation-keyed invalidation in `quran_search_cache.rs` (verified signatures, 128 MiB LRU per subagent grounding) around the unchanged service calls; unparseable or generation-mismatched rows are treated as misses (recompute), never errors.
- **Rationale**: Verified the cache API exists with generation keying; FR-020 forbids a second layer. Wholesale invalidation on generation bump means the wiring needs no cache-coherence logic of its own.
- **Alternatives considered**: HTTP-level response caching (tower-http) — rejected: would key on URL while correctness keys on generation; the application cache is the right layer.

## Decision 8 — Per-tool parameter threading (verified signatures)

- **Decision**: Thread each service's extra parameters from flags/query-string (verified signatures against `quran_search.rs`):
  - `search_exact(db, index_root, params, field: ExactField)` — `--field {text_exact,text_ws}` → `ExactField::TextExact/TextWs`; `--match-mode` → `SearchParams.mode` (`WholeToken/Substring/AyahPrefix`, verified enum).
  - `search_normalized(db, index_root, params, profile: NormalizedProfile)` — profile passthrough per Decision 3.
  - `search_phrase(db, index_root, params, profile, mode: PhraseMode, slop: u32)` — `--phrase-mode/--slop`.
  - `search_concatenated(db, index_root, params, allow_cross_ayah: bool, max_ayah_span: u32)` — cross-ayah window flags (SSE `spans_ayah_boundary` hits originate here).
  - `search_regex(db, index_root, params, field, pattern, principal, limiter, timeout_ms)` — `--regex-field/--pattern` restricted to `INDEXED_FIELDS`, DFA compile up front (`compile_dfa`), `timeout_ms.clamp(1, 10_000)` (verified), per-principal `RateLimiter::check`.
- **Rationale**: Signatures read directly from code (`quran_search.rs:799, 894, 1214, 1481, 1967`); ceilings (limit 1000, timeout 10000) verified in `model.rs:123-158` and service defaults. Nothing to invent.
- **Alternatives considered**: None — signatures are fixed; the plan step only designs their transport exposure.

## Open items

None. Every Technical Context entry is grounded in verified code; no `NEEDS CLARIFICATION` remains.
