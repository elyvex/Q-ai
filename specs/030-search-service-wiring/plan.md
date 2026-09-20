# Implementation Plan: Search-Service Wiring (CLI + Server API + SSE)

**Branch**: `030-search-service-wiring` | **Date**: 2026-09-18 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/030-search-service-wiring/spec.md`

## Summary

Expose the five implemented application search services (`search_exact`, `search_normalized`, `search_phrase`, `search_concatenated`, `search_regex` in `crates/application/src/quran_search.rs`) through a new `qai quran search` CLI verb and six new loopback HTTP routes (five paged GET + one SSE stream), reusing the existing `SearchParams`/`SearchOutput`/`SearchHit` types, the generation-keyed result cache, the `Envelope{data,meta}` + `Diagnostic` server conventions, and the CLI `CommandOutput` + exit-code scheme. No engine changes, no new crates, no migrations, no new tables.

## Technical Context

**Language/Version**: Rust 1.97.1, edition 2024, workspace resolver 2 (`rust-toolchain.toml` pinned)

**Primary Dependencies**: clap 4 (CLI arg parsing), axum 0.8 + tower-http (server routes + `Sse` type), tokio (async runtime), serde/serde_json (envelopes), existing workspace crates `application`, `quran-search`, `quran-normalization`, `storage`, `tools` (section-12 contract), `citations` (deep links) — no new dependencies

**Storage**: Existing SQLite database + on-disk FTS5 index generations; reads only via `open_serving`/`db_registry`. No new tables, no new migrations.

**Testing**: `cargo test --workspace`; new trycmd CLI snapshot cases for `qai quran search` (JSON + human + misuse matrix); new server route tests in `crates/server/tests/` style (envelope shape, ETag, error-status mapping, SSE event framing); CLI↔HTTP parity test diffing identical queries; engine suites + golden reference set re-run unchanged as no-regression gate.

**Target Platform**: Local-first dev machines (Linux/macOS); loopback-only HTTP server (`serve --bind` refusal unchanged and applying to all new routes).

**Project Type**: CLI + web-service wiring inside the existing Cargo workspace (thin transport adapters over `application` services).

**Performance Goals**: Default per-query timeout 2000ms, ceiling 10000ms (engine contract, `SearchOpts` clamp); result cap default 100, ceiling 1000; SSE streaming as the bounded-memory path for large result sets instead of giant pages.

**Constraints**: Loopback-only binding (fail-closed, unchanged); DFA-only bounded regex execution with timeout + per-principal rate limiting, backend query syntax never exposed; `--profile` + `--rules` mutually exclusive (enforced at clap level via `conflicts_with` AND by construction via the `NormalizedProfile` enum); limit clamped to 1000, timeout clamped to 1..10000; every response carries the section-12 contract; human CLI output must never alter canonical text (normalization visible only as labelled trace/rules); empty normalized queries return zero hits, never match-everything.

**Scale/Scope**: 114-surah / 6236-ayah-class corpus; 5 lexical services; 1 new CLI verb; 6 new HTTP routes (5 paged + 1 SSE); 0 new crates; 0 migrations.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. Canonical Text Integrity**: PASS — wiring performs reads only through existing services; no canonical writes, no new tables, no LLM/model involvement. Quotations travel inside `SearchHit`'s validating constructor (`QuranQuotation`), unchanged.
- **II. Layered Trust and Provenance**: PASS — no new annotations or edges; the serving `rule_set` trace and per-hit `explanation` are propagated verbatim, never edited.
- **III. Traceability and Reproducibility**: PASS — every response carries tool name/version, query, normalization rules, edition id/version, references, warnings, timing, and `ReproducibilityData` checksum; HTTP adds ETag + conditional-request short-circuit via existing `etag_for`/`json_response` helpers.
- **IV. Scholarly Honesty**: PASS — no synthesis, no merging; normalization shown as labelled trace/rules; human output never edits canonical text (highlight uses `<b>` display markers on a rendered copy only).
- **V. Test-First and Quality Gates**: PASS — Red-Green-Refactor; full gate set (`fmt`, `check`, `clippy -D warnings`, `cargo test --workspace`, `arch-check`, `migrate-check`) plus engine no-regression gate (SC-006).
- **VI. Local-First Security, Deny-by-Default**: PASS — new routes sit behind the existing loopback-only guard; regex surfaces `principal` + `RateLimiter` budgets; no new network surface, no new secret handling, `unsafe_code = "forbid"` untouched.
- **VII. Simplicity and Architecture Discipline**: PASS — no new crates (violations unjustified → none introduced); `server` → `application` → storage layering preserved (CLI parses, `application::quran_cli::cmd_search` executes, same as `cmd_get` pattern); SQLite FTS5 assumed, Tantivy not; no migrations.
- **VIII. Multi-Edition Governance**: PASS — edition selector defaults to indexed edition; `EditionNotIndexed` mismatch surfaced typed on both surfaces, never silent cross-edition serving.

*Post-design re-check (Phase 1): no new violations introduced — design adds transport adapters only; all gates re-affirmed.*

## Project Structure

### Documentation (this feature)

```text
specs/030-search-service-wiring/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
├── contracts/           # Phase 1 output (/speckit-plan command)
│   ├── cli-search.md    # CLI verb contract (flags, exits, JSON shape)
│   └── http-search.md   # HTTP routes contract (params, envelope, SSE, errors)
└── tasks.md             # Phase 2 output (/speckit-tasks command - NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
crates/
├── cli/src/quran.rs                 # + Search variant on QuranAction; + dispatch arm
├── application/src/quran_cli.rs     # + cmd_search (SearchRequest assembly → service → CommandOutput)
├── application/src/quran_search.rs  # UNCHANGED (services called, not modified)
├── server/src/api.rs                # + 6 routes, query structs, SSE handler, error mapping
└── server/tests/                    # + search route tests (envelope, ETag, errors, SSE)

tests/ (trycmd snapshots)
└── crates/cli/tests/                # + quran search cases (JSON/human/misuse)
```

**Structure Decision**: Thin-adapter wiring inside existing crates per the established `cli → application → storage/services` layering (`arch-check` enforced). CLI parsing lives in `crates/cli/src/quran.rs`, execution in `application::quran_cli` free functions (same split as `cmd_get`/`cmd_context`). HTTP transport lives in `crates/server/src/api.rs` reusing `Envelope`/`Meta`/`ErrorBody`/`json_response`/`etag_for`. No new crates, no engine edits.

## Complexity Tracking

> No constitution violations; nothing to justify. This section intentionally left empty.
