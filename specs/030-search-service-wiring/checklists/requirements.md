# Requirements Checklist: search service wiring (CLI + server API + SSE)

**Purpose**: Requirements-quality review of `spec.md` for branch `030-search-service-wiring` — wiring-only scope, forward (unimplemented) behavior, no duplication of specs 008/010/011.
**Created**: 2026-09-18
**Feature**: `specs/030-search-service-wiring/spec.md`

**Note**: This custom checklist is generated based on feature context and requirements.
**Review Ownership**: This checklist is a reviewer-owned requirements-quality review artifact. Mark an item `[x]` only when the reviewer determines the requirements-quality criterion is satisfied.
**Marker Semantics**: `[x]` means the criterion has been reviewed and satisfied for requirements quality. It does not mean implementation work is complete.

## Specification Structure & Clarity

- [x] CHK001 Header follows the template exactly (Branch `030-search-service-wiring`, Created 2026-09-18, Status Draft, Input) — verified present in `spec.md`.
- [x] CHK002 All five user stories carry priority, Why, Independent Test, and Given/When/Then acceptance scenarios — verified: US1 P1 CLI, US2 P1 HTTP envelope, US3 P2 profile passthrough, US4 P2 SSE, US5 P3 regex.
- [x] CHK003 Each story is independently testable and shippable as a slice (CLI alone, HTTP alone, passthrough, SSE, regex) — verified per-story Independent Test statements.
- [x] CHK004 Scope boundary is explicit: engine (008), CLI tree (010), server reads (011) excluded; wiring-only FRs reference but do not restate covered behavior — verified Out-of-scope block.
- [x] CHK005 Forward framing is consistent: acceptance is written as behavior to build, with no claims that routes/flags already exist — verified (spec states services exist but are unreachable from CLI/server today).
- [x] CHK006 Zero NEEDS CLARIFICATION markers and none are required: services, params, ceilings (limit 1000, timeout 10000), and contract fields are all grounded in code truth — verified, no markers present.

## Requirements Completeness

- [x] CHK007 All five application services are wired on both surfaces (FR-001/FR-002 CLI tools, FR-013 server routes) — verified against `search_exact/normalized/phrase/concatenated/regex` signatures.
- [x] CHK008 Normalization passthrough covers registry pinned/latest and adhoc rules with never-both exclusivity on both surfaces (FR-004, FR-014, US3) — verified against `NormalizedProfile::Registry/Adhoc`.
- [x] CHK009 Pagination, filters, explain/highlight, and ceilings are specified for both surfaces (FR-008/FR-009/FR-010/FR-014) — verified against `SearchParams`, `SearchOpts::normalized()`, and `Filter` variants.
- [x] CHK010 Section-12 result contract + quotation fields + reproducibility checksum required on every response (FR-015/FR-018, US2) — verified against PRD §12/§12.1 and constitution III.
- [x] CHK011 SSE streaming (hit events + terminal summary + cancellation) is specified without duplicating paged-route semantics (FR-017, US4) — verified.
- [x] CHK012 Error mapping uses existing public schemes on both surfaces (CLI exit codes, HTTP Diagnostic statuses) with no new error taxonomy (FR-012/FR-016) — verified against specs 010/011 conventions.
- [x] CHK013 Risk paths are fenced: regex DFA-only budgets + rate limits (FR-021, US5), empty-query never-match-all (FR-022), edition-mismatch surfacing, staleness advisories (FR-019), cache passthrough with no second layer (FR-020), loopback-only unchanged — verified.
- [x] CHK014 Constitution VII constraints recorded: `server` → `application` layering, FTS5-shipped/Tantivy-unaccepted, no new crates, no engine changes — verified in compliance line, FR-001, and Assumptions.

## Acceptance Criteria & Measurability

- [x] CHK015 Each SC item is measurable and tech-agnostic (invoke/diff/validate/repeat rather than implementation detail) — verified SC-001 through SC-006.
- [x] CHK016 CLI↔HTTP parity is asserted by output diffing fixed queries (SC-002), not by shared-code inspection — verified.
- [x] CHK017 Pagination exactness is asserted by page-concatenation equality (SC-004), covering the exact-total/never-estimated contract — verified.
- [x] CHK018 Misuse matrix covers exclusivity, unknown profiles, non-indexed regex fields, and ceiling clamping on both surfaces (SC-005) — verified.
- [x] CHK019 No-regression gate on engine suites + golden reference set is explicit (SC-006) so wiring cannot silently change ranking/spans — verified.
- [x] CHK020 Edge cases enumerate empty normalization, edition mismatch, stale generation, ceiling clamping, unsatisfiable filters, corrupt cache rows, and non-loopback bind — verified seven bullets in Edge Cases.

## Notes

- Evidence grounding (code truth, 2026-09-18): `crates/application/src/quran_search.rs` head + `pub fn`/`pub async fn` grep (services `search_exact` L799, `search_normalized` L894, `search_phrase` L1214, `search_concatenated` L1481, `search_regex` L1967; `NormalizedProfile` L883; `PhraseMode` L1088; `RateLimiter` L1903); `quran_search_cache.rs` full read (128 MiB cap, generation-keyed, miss-on-mismatch); `quran-search/src/model.rs` (`SearchOpts` ceilings, `Filter`, `FtsBackend::Fts5` shipped); `hit.rs` (`SearchHit` validating constructor, trace/span/quotation mandatory); CLI `quran.rs` (no search verb — `index` management only); server `api.rs` routes (no search routes; envelope + loopback conventions reused).
- ADR-0201 noted as `Proposed` (Tantivy) vs shipped FTS5; spec assumes FTS5 and forbids assuming Tantivy per constitution VII.
- PRD §11 (tool catalog) and §12/§12.1 (result contract + checksum) are the contract sources; spec adds no new contract fields.
- `checklists/requirements.md` lifecycle: reviewer-owned; `/speckit-implement` reads checkbox state as a gate and must not modify markers.
- `.specify/feature.json` untouched; nothing committed — per task instructions.
