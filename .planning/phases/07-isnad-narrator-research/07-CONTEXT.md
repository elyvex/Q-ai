# Phase 7: Isnad & Narrator Research - Context

**Gathered:** 2026-10-17
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can trace narration chains and judge narrator identity with explicit uncertainty. Narrator records carry name variants and rijal sources; chain paths between narrators are traversable with uncertainty marked on each link; narrator-identity proposals go through a recorded approve/reject workflow.

</domain>

<decisions>
## Implementation Decisions

### Narrator Identity Model
- Narrator is a node in the isnad graph with a stable ID (`narrator:{slug}`)
- Name variants stored as a list on the narrator record (Arabic script variants, kunya, laqab, nisba)
- Rijal sources are attributed records per narrator (scholar/source citation), following the per-scholar grading pattern — never merged into consensus
- Computational identity matches carry confidence + "uncertain" label; never presented as certainty without owner confirmation

### Isnad Chain Representation
- Chain modeled as graph edges: narrator→narrator transmission edges and narrator→hadith transmission edges
- Each link carries provenance layer (B/C/D), confidence, and uncertainty label
- Matn/isnad text from the hadith record is preserved verbatim; chain extraction (computational) is labeled uncertain
- Reuse Phase 4 graph edge vocabulary and assertion authority patterns

### Identity Review Workflow
- Narrator-identity proposals are assertions (graph_assertions pattern) with Pending/Accepted/Rejected/Superseded
- Reviewer identity + decided_at recorded with CHECK constraint (mirrors verse_links owner gate)
- Disputed stays effective (never tombstoned) to represent live scholarly disagreement
- Supersession chain supported (supersedes_id)

### Chain Path Traversal
- Reuse Phase 4 traversal (up_to_k_paths, min_hops, bounded_subgraph) with QueryBudgets
- Uncertainty marked on each link in path results
- Computational extraction confidence thresholds: high (>0.9), medium (0.7-0.9), low (<0.7) — all labeled uncertain until confirmed
- Budget exhaustion explicit (truncated + incomplete_reason)

### the agent's Discretion
- Exact column names and migration numbering
- CLI/API/TUI surface shape (follow Phase 5/6 typed-tool pattern)
- Test organization

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `quran-graph` crate: GraphStore port, traversal, assertions, budgets, export
- `hadith-core`: HadithRecord with isnad_text, GradingRecord per-scholar pattern
- `isnad-graph` crate: placeholder, to be implemented
- Phase 6 verse_links: confidence + owner gate pattern (migration 0030)
- Phase 4 quran_graph_store application adapter

### Established Patterns
- Migrations: append-only checksummed .sql in migrations/sqlite (0023-0030)
- Typed tools: ToolResult with research_checksum, canonical_references, analysis_sources
- Storage: HadithBackend trait in tool-registry; application services wrap readers
- Cross-surface parity: CLI/TUI/API expose same tools

### Integration Points
- application crate: new isnad_reader/isnad_search/isnad_tools modules
- tool-registry: new IsnadBackend trait + tools
- server/cli/tui: expose isnad tools

</code_context>

<specifics>
## Specific Ideas

- Reuse Phase 4 graph traversal and assertion authority for narrator identity
- Reuse Phase 6 per-scholar grading pattern for rijal sources
- Reuse Phase 6 verse_links owner-gate CHECK for identity review
- Follow Phase 5 typed-tool pattern for isnad tools

</specifics>

<deferred>
## Deferred Ideas

- Full narrator database ingestion (data acquisition) — Phase 7 scope is schema + tools + review workflow
- Comparative isnad across collections — Phase 9 scope

</deferred>
