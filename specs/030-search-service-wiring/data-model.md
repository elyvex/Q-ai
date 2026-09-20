# Data Model: Search-Service Wiring

**Feature**: `030-search-service-wiring` | **Date**: 2026-09-18

No new persisted state. All entities below are transport/wiring-level assemblies over existing types (`SearchParams`, `SearchOutput`, `SearchHit`, `Filter`, `Warning`, `RegexReport`, `ToolResult`, `ReproducibilityData`, `Envelope`/`Meta`). Field types cite the verified code definitions.

## Entity 1 — SearchRequest (wiring-level, transport-neutral)

Assembled by both surfaces from flags / query-string, then fanned out to the five service signatures. Owns validation; never performs normalization itself.

| Field | Type | Rules |
|---|---|---|
| `tool` | enum `search_exact \| search_normalized \| search_phrase \| search_concatenated \| search_regex` | Required; CLI default `search_normalized`; HTTP is implicit in the path |
| `text` | `String` | Required, non-empty after trim; normalizes-to-empty → zero-hit short-circuit (`total_matches: 0`, `truncated: false`), never match-everything |
| `edition` | `slug@version` string, optional | Defaults to indexed edition; non-indexed value → `EditionNotIndexed` error on both surfaces |
| `profile_or_rules` | `NormalizedProfile::Registry(ProfileId, Option<SemVer>) \| Adhoc(Vec<RuleId>)` | Exactly one side; both-present → CLI exit 2 / HTTP 400 before any index access |
| `mode` | `MatchMode` (`WholeToken \| Substring \| AyahPrefix`) | Exact tool only; default `WholeToken` |
| `exact_field` | `ExactField` (`TextExact \| TextWs`) | Exact tool only; default `TextExact` |
| `phrase_mode` | `PhraseMode` | Phrase tool only |
| `slop` | `u32` | Phrase tool only |
| `allow_cross_ayah` | `bool` | Concatenated tool only |
| `max_ayah_span` | `u32`, `>= 1` | Concatenated tool only; `< 1` → `QueryRejected` (verified service guard) |
| `regex_field` | `String` | Regex tool only; must be a member of `INDEXED_FIELDS`, else `QueryRejected` |
| `pattern` | `String` | Regex tool only; DFA-compiled up front, malformed → typed index error |
| `principal` | `String` | Regex tool only; rate-limit identity surfaced, policy engine stays out of scope (tool-registry gate, Phase 7) |
| `timeout_ms` | `u64` | Default 2000, clamped to `1..=10_000` (verified `SearchOpts` clamp) |
| `filters` | `Vec<Filter>` | `Surah(Vec<u16>)`, `JuzRange(u16,u16)`, `Page(Vec<u32>)`, `RevelationPlace(String)`, `GlobalRange(u64,u64)` (verified `model.rs:98-112`); AND-combined; unsatisfiable → zero hits, never error |
| `limit` | `u32` | Default 100, hard ceiling 1000 (verified `model.rs:123,157`) |
| `offset` | `u32` | Default 0; pages deterministically over the same ordered set |
| `explain` | `bool` | `false` → canonical order, no scores; `true` → relevance order + per-hit BM25 breakdown (indexed modes) |
| `highlight` | `bool` | Wraps hit span in `<b>` display markers on a rendered copy only |

## Entity 2 — SearchEnvelope (section-12 result contract over `SearchOutput`)

HTTP `Envelope.data` payload and CLI `--json` top-level object. Hit objects are the serialized `SearchHit` (verified `hit.rs:97-137` parts: edition ref, surah number + Arabic/translit names, ayah, canonical Arabic text, text hash, page/juz, canonical span, matched tokens, score + breakdown, mandatory normalization trace, segmentation, highlight, `spans_ayah_boundary`, warnings) plus envelope-level fields:

| Field | Source |
|---|---|
| `tool_name`, `tool_version` | Service identity (`search_normalized@…`) |
| `query` | Exact input text + parsed parameters (secret-redacted by callers per `ToolResult` convention) |
| `normalization_rules` | Serving `rule_set` + per-hit trace |
| `edition_id`, `edition_version` | Serving edition (matches indexed edition or error) |
| `hits` | `SearchOutput.hits` in requested order |
| `total_matches` | Exact count from the separate count path, never estimated |
| `truncated` | `limit` cut the list |
| `generation` | Serving corpus generation read from |
| `warnings` | Output advisories: zero-result hints, `stale_index` drift notes (`Warning{code, message}`, verified `hit.rs:55-70`) |
| `regex_report` | `RegexReport{pattern, field, terms_matched, terms_examined}`; `None` for non-regex tools |
| `canonical_references` | One fully-qualified reference per hit (+ deep links inside hits) |
| `execution_time_ms` | Wall-clock, measured by the wiring layer |
| `reproducibility` | `ReproducibilityData` (checksum, query_hash, tool_plan `name@version`, edition/source versions, rule set, generation, deterministic flag) |

HTTP wraps this in `Envelope{api_version, data, meta}` where `meta` carries edition descriptor, `corpus_generation`, timing, `reproducibility` JSON, warnings, plus ETag (`etag_for`) and `Content-Language` headers.

## Entity 3 — SSE hit stream (`GET /api/v1/quran/search/stream`)

Ordered event sequence for one normalized-search query:

1. Zero or more `hit` events, each carrying one paged-schema hit object (identical shape to Entity 2 hits).
2. Exactly one terminal `summary` event: `{total_matches, truncated, warnings, reproducibility_checksum}`.
3. Client disconnect stops work promptly; no above-warning logging for the cancellation.

## Entity 4 — Error mapping (no new error types)

| `SearchError` variant | Diagnostic code | CLI exit | HTTP status |
|---|---|---|---|
| Usage/parse failures (both-present profile+rules, unknown profile, malformed filter, bad pattern shape) | `QAI-NORM-*` / validation | 2 (`USAGE`) / 3 (`VALIDATION`) | 400 |
| `NoServingIndex` (index never built) | `QAI-IDX-3` | 5 (`NOT_FOUND`) | 404 |
| `EditionNotIndexed` | `QAI-IDX-2` | 6 (`CONFLICT`/state) | 409 |
| `RateLimited` | `QAI-IDX-7` | 4 (`POLICY`) | 429 |
| `Storage` / unexpected `Index` failures | `QAI-IDX-*` | 70 (`INTERNAL`) | 5xx |
| Async runtime startup failure | n/a | 70 (`INTERNAL`) | n/a |

Bodies: CLI `CommandOutput::err` (`error: {message}` human + `{"error":…}` JSON, never stack traces); HTTP `ErrorBody` Diagnostic shape via the existing conversion.

## Validation rules (cross-entity)

- `profile` + `rules` together → reject before index access (both surfaces).
- `limit > 1000` → clamp, effective value visible in output/meta; `timeout_ms` clamped `1..=10_000`.
- Requested edition ≠ indexed edition → typed mismatch error, never cross-edition hits.
- Stale generation → serve hits with mandatory `stale_index` advisory propagated through both surfaces.
- Cache rows unparseable or generation-mismatched → miss + recompute, never error.
- Non-loopback `--bind` → existing refusal applies to all six new routes unchanged.
