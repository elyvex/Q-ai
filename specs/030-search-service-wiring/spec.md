# Feature Specification: search service wiring (CLI + server API + SSE)

**Feature Branch**: `030-search-service-wiring`

**Created**: 2026-09-18

**Status**: Draft

**Input**: User description: "Expose the implemented application search services (`search_exact`, `search_normalized`, `search_phrase`, `search_concatenated`, `search_regex` in `crates/application/src/quran_search.rs` returning `SearchOutput`/`SearchHit`) through `qai quran search` CLI flags and server API routes (+SSE): query params, normalization profile passthrough, pagination, PRD section-12 result envelope, SSE streaming. Forward specification — behavior to build. Must not duplicate engine coverage (spec 008), CLI tree coverage (spec 010), or server read-API coverage (spec 011); covers ONLY the missing wiring between them."

**Constitution compliance**: `.specify/memory/constitution.md`, Principle III (every tool result carries the section-12 result contract + reproducibility checksum; quotations come from canonical retrieval via the restricted constructor, never model memory), Principle VII (wiring routes through `application` services preserving `server` → `application` layering; SQLite FTS5 is the implemented backend — Tantivy remains an unaccepted proposal per ADR-0201 and MUST NOT be assumed; no new crates, no engine changes).

**Out of scope (covered elsewhere, MUST NOT be re-specified here)**: search engine semantics, ranking, span verification, and FTS5 backend behavior (spec 008); CLI exit-code scheme, lifecycle verbs, and trycmd harness conventions (spec 010); server health/readiness, read-API routes, and debug reader (spec 011); normalization rule catalog and profile definitions (ADR-0204); index build/verify lifecycle (existing `quran index` verbs).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Search from the terminal (Priority: P1)

A researcher runs `qai quran search` with a query and mode flags and gets ranked or canonically ordered hits as JSON or human-readable text, with the normalization profile, edition, pagination, and truncation explicitly reported.

**Why this priority**: The CLI is the developer-stage primary interface. The search services exist but are unreachable from the terminal today (`crates/cli/src/quran.rs` has `index` management only — no search verb), so this is the highest-value wiring slice.

**Independent Test**: Can be fully tested by running `qai quran search --query <text> --mode <mode> [--profile ...] [--limit N] [--offset M] [--json]` against a built index and delivers printable, paginated hits without touching the server.

**Acceptance Scenarios**:

1. **Given** a built serving generation, **When** the user runs `qai quran search --query <text> --tool search_normalized --profile L3.diacritics --limit 20`, **Then** the command exits 0 and returns up to 20 hits each carrying a pinned canonical reference, validated quotation, canonical span, mandatory normalization trace, and the serving generation.
2. **Given** a query with more matches than the limit, **When** the search runs, **Then** the output reports the exact total match count and a truncated flag, and `--offset` pages deterministically through the same ordered set.
3. **Given** `--explain`, **When** the search runs on an indexed profile, **Then** hits are served in backend relevance order with per-hit score breakdowns; without `--explain` they are served in canonical order without scores.
4. **Given** an unbuilt index (no serving generation), **When** the search runs, **Then** the command fails with a typed not-found/state error and public exit code (never exit 0, never a stack trace).

---

### User Story 2 - Search over HTTP with the section-12 envelope (Priority: P1)

A client issues `GET /api/v1/quran/search/...` with query-string params and receives hits inside the standard API envelope carrying the PRD section-12 result contract (tool name/version, query, normalization rules, edition id/version, results, canonical references, warnings, timing, reproducibility data).

**Why this priority**: Machine-readable search is the counterpart to the CLI wiring and unblocks the future Web GUI, agents, and external research applications on the same reader semantics.

**Independent Test**: Can be fully tested by starting the loopback server and issuing HTTP GET search requests with `curl`, asserting envelope shape, ETag presence, and error-status mapping independently of the CLI.

**Acceptance Scenarios**:

1. **Given** a running loopback server with a built index, **When** the client requests `GET /api/v1/quran/search/normalized?q=<text>&profile=<id>&limit=20&offset=0`, **Then** the response is 200 with an `Envelope{data, meta}` body where each hit carries reference, quotation, span, trace, and generation, and `meta` carries edition identity, normalization rules, and reproducibility fields.
2. **Given** identical repeated requests, **When** served, **Then** responses carry an ETag and reproducibility checksum so re-running a deterministic query reproduces identical output.
3. **Given** an invalid query (unknown profile, `profile` + `rules` together, limit over ceiling, malformed filter), **When** requested, **Then** the server returns a Diagnostic error body with the mapped HTTP status (never 200 with an empty result masquerading as success).

---

### User Story 3 - Normalization profile passthrough without engine changes (Priority: P2)

A researcher selects the search semantics explicitly per request — registry profile (latest or `@version`-pinned) or explicit adhoc rule list, never both — and the wiring layer forwards that selection unchanged to the application service, which reports the profile that actually served the query.

**Why this priority**: Explicit profile selection is the mechanism that keeps exact, normalized, heuristic, and adhoc searches distinct (constitution IV adjacency, PRD §8). The wiring must be a faithful passthrough, not a second normalization implementation.

**Independent Test**: Can be fully tested by issuing the same query with different `--profile`/`?profile=` values (and `--rules`/`?rules=`) and observing distinct serving traces and rule sets in the output, independently of pagination or transport tests.

**Acceptance Scenarios**:

1. **Given** `--profile L3.diacritics@<version>` (or `?profile=`), **When** the search runs, **Then** the pinned profile version serves the query and the output `rule_set` names exactly that profile.
2. **Given** `--rules N01,N03` (or `?rules=`), **When** the search runs without `--profile`, **Then** the adhoc rule set serves the query with per-candidate verification semantics and the output trace reflects the adhoc rules.
3. **Given** both `--profile` and `--rules`, **When** the search is invoked, **Then** the invocation is rejected as a usage error (CLI exit 2 / HTTP 400) before any index access.

---

### User Story 4 - Stream large result sets over SSE (Priority: P2)

A client that cannot or does not want to page through thousands of hits opens an SSE endpoint and receives hits as a stream of envelope-framed events followed by a terminal summary event (totals, truncation, warnings, checksum).

**Why this priority**: Count paths are exact and unbounded result sets can be large; streaming gives clients a bounded-memory alternative to offset pagination while preserving the same per-hit contract.

**Independent Test**: Can be fully tested by opening the SSE route with an `Accept: text/event-stream` client, collecting events, and asserting each hit event matches the non-streaming hit schema plus a final summary event — independently of the paged routes.

**Acceptance Scenarios**:

1. **Given** a query with more matches than one page, **When** the client opens the SSE search route, **Then** the server emits one event per hit (same hit schema as the paged route) followed by a terminal event carrying exact total, truncated flag, warnings, and reproducibility checksum.
2. **Given** a client disconnect mid-stream, **When** the connection drops, **Then** the server stops work promptly and never logs an error above warning level for the cancellation.

---

### User Story 5 - Bounded regex search through both surfaces (Priority: P3)

An advanced user runs a DFA-bounded regex search over an explicitly selected indexed text field from the CLI and the API, with timeout and per-principal rate-limit budgets enforced and the regex execution report (pattern, field, terms matched/examined) returned in the output.

**Why this priority**: Regex is the highest-risk search form (resource exhaustion); wiring it last keeps the common paths shippable while the dangerous path gets explicit budgets on both surfaces.

**Independent Test**: Can be fully tested by running a regex query via CLI flags and via HTTP params and asserting identical `regex_report` content and identical budget rejections, independently of the other search tools.

**Acceptance Scenarios**:

1. **Given** `--tool search_regex --field text_bare --pattern <expr>` (or HTTP equivalents), **When** the search runs, **Then** the output includes the regex report and hits verified against the same automaton semantics as the engine.
2. **Given** a pathological or over-budget pattern, **When** submitted, **Then** the request is rejected or bounded with a typed error (CLI rate-limit/policy exit, HTTP 429/400) and never executed with backtracking.

---

### Edge Cases

- What happens when the query normalizes to an empty term list (e.g. whitespace/marks only)? The wiring MUST return zero hits with `total_matches: 0` and `truncated: false`, never match-everything.
- How does the system handle a requested edition that is not the indexed one? The wiring MUST surface the typed edition-mismatch error (CLI non-zero exit / HTTP mapped status), never silently serve another edition's hits.
- How does the system handle a stale index (serving generation drifted from inputs)? Hits MUST still be served with the mandatory staleness advisory warning propagated through both surfaces.
- How does the system handle `limit` above the 1000 ceiling or `timeout_ms` above 10000? Values MUST be clamped to their ceilings per engine contract, and the effective values MUST be visible in the output/meta.
- How does the system handle metadata filters (`surah`, `juz` range, `page`, `revelation_place`, `global` range)? Filters MUST be AND-combined and an unsatisfiable filter set MUST yield zero hits, never an error.
- How does the system handle unparseable or generation-mismatched cache rows? They MUST be treated as misses (recompute from the index), never as errors, on both surfaces.
- How does the system handle non-loopback `--bind` for `serve` when search routes are enabled? The existing loopback-only refusal MUST apply unchanged to the new routes.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: CLI MUST add a `qai quran search` verb that reaches all five application services (`search_exact`, `search_normalized`, `search_phrase`, `search_concatenated`, `search_regex`) with no engine-semantics changes.
- **FR-002**: CLI MUST accept `--tool {search_exact,search_normalized,search_phrase,search_concatenated,search_regex}` selecting the service, defaulting to `search_normalized`.
- **FR-003**: CLI MUST accept `--query <text>` (required) plus `--edition <slug@version>` (optional, defaults to the indexed edition).
- **FR-004**: CLI MUST pass normalization selection through unchanged: `--profile <id[@version]>` or `--rules <N01,N02,...>`, never both; both together MUST be a usage error (exit 2).
- **FR-005**: CLI MUST accept phrase/concatenated options `--phrase-mode {ordered_exact,ordered_near,unordered_near}` and `--slop N`, forwarded to `search_phrase`.
- **FR-006**: CLI MUST accept exact-search options `--field {text_exact,text_ws}` and `--match-mode {whole_token,substring,ayah_prefix}`, forwarded to `search_exact`.
- **FR-007**: CLI MUST accept regex options `--regex-field <indexed field>` and `--pattern <expr>` for `--tool search_regex`, restricted to indexed text fields.
- **FR-008**: CLI MUST accept metadata filters (`--surah`, `--juz`, `--page`, `--revelation-place`, `--global-range`) AND-combined into the service filter list.
- **FR-009**: CLI MUST accept `--limit N` (default 100, ceiling 1000) and `--offset M` (default 0) for pagination, with the ceiling enforced.
- **FR-010**: CLI MUST accept `--explain` (relevance order + score breakdowns) and `--highlight` (span wrapped in `<b>` display markers) flags forwarded to the service.
- **FR-011**: CLI MUST support `--json` output carrying the full section-12 result contract, and human-readable output that never alters the canonical text (normalization shown only as labelled trace/rules, never edited into the quotation).
- **FR-012**: CLI MUST map service errors to the public exit-code scheme (usage 2, not-found 5, conflict/state 6, policy/rate-limit 4, internal 70) with a Diagnostic body, never a stack trace.
- **FR-013**: Server MUST add `GET /api/v1/quran/search/exact`, `/search/normalized`, `/search/phrase`, `/search/concatenated`, and `/search/regex` routes behind the existing loopback-only guard.
- **FR-014**: Server search routes MUST accept query-string params mirroring the CLI surface: `q` (required), `edition`, `profile` | `rules` (never both), `phrase_mode`, `slop`, `field`/`match_mode` (exact), `regex_field`+`pattern` (regex), filters, `limit` (default 100, ceiling 1000), `offset`, `explain`, `highlight`, `timeout_ms` (default 2000, ceiling 10000).
- **FR-015**: Server search responses MUST use the standard `Envelope{data, meta}` shape with ETag and `Content-Language`; `meta` MUST carry edition id/version, normalization rules, generation, and reproducibility checksum fields.
- **FR-016**: Server search failures MUST return Diagnostic error bodies with the existing mapped HTTP statuses (400 usage, 404 not indexed/found, 409 state, 429 rate-limited, 5xx internal).
- **FR-017**: Server MUST add a `GET /api/v1/quran/search/stream` SSE route accepting the normalized-search param set, emitting one event per hit in the paged hit schema followed by a terminal summary event (exact total, truncated, warnings, checksum).
- **FR-018**: Every search response on both surfaces MUST include the section-12 result contract fields: tool name/version, query, normalization rules, edition id/version, results, canonical references, warnings, timing, and reproducibility data; quotations MUST include surah number+name, ayah number, Arabic canonical text, edition identifier, and stable deep link.
- **FR-019**: Wiring MUST propagate the output-level advisories unchanged: zero-result hints, generation-drift staleness warnings, and the regex execution report.
- **FR-020**: Wiring MUST NOT bypass the generation-keyed result cache nor invent a second caching layer; cache misses/mismatches MUST recompute from the index.
- **FR-021**: Regex wiring MUST enforce the DFA-only bounded execution path with timeout and per-principal rate limiting on both surfaces; backend query syntax MUST never be exposed to callers.
- **FR-022**: Empty normalized queries MUST return zero hits (`total_matches: 0`, `truncated: false`), never match-everything, on both surfaces.

### Key Entities

- **SearchRequest (wiring-level)**: Transport-neutral query assembly — raw query text, tool selector, edition selector, profile-or-rules selection, phrase/regex options, AND-combined metadata filters, limit/offset, explain/highlight/timeout flags. Owns validation (profile+rules exclusivity, ceilings), never normalization itself.
- **SearchResponse envelope**: Section-12 result contract over the engine `SearchOutput` — hits (reference, quotation, span, matched tokens, score/explain, trace, segmentation, highlight, boundary flag, warnings), exact total, truncated flag, serving rule set, generation, output warnings, regex report, plus timing and reproducibility checksum added by the wiring layer.
- **SSE hit stream**: Ordered event sequence for one query — N hit events in the paged hit schema followed by exactly one terminal summary event (exact total, truncated, warnings, checksum); cancellation stops emission without error escalation.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A researcher can run any of the five search tools from the terminal and receive contracted results within the same time budget the engine offers (bounded query timeout, default 2s) — measured by invoking each `--tool` against a built index and observing exit 0 with schema-valid output.
- **SC-002**: A client can retrieve the same query via CLI `--json` and via the HTTP route and obtain identical hit sets (same references, spans, ordering, totals) — measured by diffing the two outputs for a fixed query/edition/profile/pagination.
- **SC-003**: 100% of search responses on both surfaces carry the section-12 contract fields (tool identity, query, rules, edition, references, warnings, timing, reproducibility data) — measured by schema validation over a sample covering all five tools.
- **SC-004**: Pagination is exact and reproducible: repeated identical paged requests return identical pages and the concatenated pages equal the full ordered set — measured by fetching all pages for a multi-page query and comparing against the untruncated ordered result.
- **SC-005**: Invalid invocations (profile+rules together, unknown profile, non-indexed regex field, over-ceiling values unclamped) are rejected or clamped consistently on both surfaces — measured by a misuse matrix asserting CLI exit codes and HTTP statuses per case.
- **SC-006**: No wiring change alters engine behavior: the engine test suites and golden reference set pass unchanged before and after the wiring lands.

## Assumptions

- The five application search services and their signatures (`SearchParams`, `SearchOutput`, `SearchHit`, `NormalizedProfile`, `PhraseMode`, `RateLimiter`) are stable and correct; this spec wires them, it does not redesign them.
- SQLite FTS5 remains the serving backend (ADR-0201's Tantivy decision is still `Proposed`, not shipped); wiring MUST stay backend-agnostic through the existing service signatures.
- CLI exit-code mapping, `--json` conventions, and `serve` loopback-only enforcement already exist and are reused unchanged.
- Server `Envelope{data, meta}`, ETag, `Content-Language`, and Diagnostic error-body conventions already exist and are reused unchanged.
- Generation-keyed caching (128 MiB cap, LRU, wholesale invalidation on generation bump) already exists in the application layer; the wiring adds no cache semantics.
- Agent-policy gating for `search_regex` stays at the tool-registry gate (Phase 7); the wiring surfaces the principal/rate-limit parameters but does not implement the policy engine.
- Morphology-dependent tools (`root_search`, `lemma_search`, `word_family`) are out of scope until their datasets land; only the five implemented lexical services are wired.
