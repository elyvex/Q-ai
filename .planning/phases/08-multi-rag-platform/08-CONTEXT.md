# Phase 8: Multi-RAG Platform - Context

**Gathered:** 2026-10-17
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can build, configure, and debug multiple independent RAG systems over any corpus. Portable RAG projects from declarative config; hybrid (FTS + vector) retrieval with reranking and metadata filters; transparent routing/tool/source choices; reproducible result sets; tombstoned/deactivated sources disappear immediately.

</domain>

<decisions>
## Implementation Decisions

### RAG Project Config
- Declarative TOML config per project directory (config.toml) defining corpus sources, embedding provider, reranker, router, and metadata filters
- Project is portable: config + index; no absolute paths; relative paths resolved against project dir
- CLI: `qai rag create <name>`, `qai rag import <project> <docs>`, `qai rag query <project> ...`
- Reuse Phase 5 typed-tool pattern for rag.* tools

### Hybrid Retrieval + Reranking
- Full-text via SQLite FTS5 (reuse Phase 3 normalization) + vector via embeddings crate
- Embeddings crate: trait `EmbeddingProvider` with a local default (deterministic hash-based embedding as the no-network baseline) and pluggable providers
- Reranking crate: trait `Reranker` with a local default (score-based) and pluggable providers
- Merge FTS + vector results with reciprocal rank fusion (RRF); apply metadata filters before/after merge
- Vector storage: SQLite table with BLOB embedding column (portable, no external vector DB) — PG/Qdrant deferred to Phase 12

### Routing Transparency
- Every query result carries a `route_decision` record: router name, tools invoked, sources consulted, scores per candidate, final ranking, reproducibility checksum
- Debug view: `qai rag debug <project> <query>` prints the full decision trace
- Reproducibility: same config + same corpus + same query → same result set (deterministic ordering, stable tie-break by id)

### Tombstoning / Deactivation
- Sources carry status (active/deactivated/tombstoned); query filters exclude non-active at retrieval time (immediate)
- Deactivation propagates to all derived index entries; no stale results

### the agent's Discretion
- Exact config schema fields, embedding dimension default, RRF k constant
- Test organization, CLI flag names

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `retrieval` crate: MemvidRetrieval backend, Document/Hit types, error handling
- `quran-normalization`: FTS5 + derived search forms pattern
- `hadith-core`/`tafsir`: source-aware record pattern
- Phase 5 typed-tool registry + result contract (research_checksum, canonical_references, analysis_sources)
- Phase 4 graph: QueryBudgets, traversal patterns (reusable for routing)

### Established Patterns
- Migrations 0031-0034 done; new migrations 0035+ for rag_projects, rag_documents, rag_embeddings, rag_sources
- Typed tools: ToolResult with research_checksum
- Cross-surface parity: CLI/TUI/API

### Integration Points
- application: rag_reader, rag_search, rag_tools modules
- tool-registry: RagBackend trait + tools
- cli/server/tui: rag commands

</code_context>

<specifics>
## Specific Ideas

- Reuse retrieval port; implement embeddings + reranking as traits with local defaults
- Follow Phase 6 verse_links owner-gate pattern for source deactivation audit
- Follow Phase 5 result contract for transparency

</specifics>

<deferred>
## Deferred Ideas

- PostgreSQL/Qdrant vector adapters — Phase 12
- Remote embedding providers (OpenAI etc.) — Phase 11/12
- Multi-tenant RAG — not in scope

</deferred>
