# Phase 9: Comparative Scripture - Context

**Gathered:** 2026-10-18
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can research Torah, Tanakh, and New Testament passages alongside the Quran with honest labeling. Browse scripture editions with chapter/verse addressing and translation comparison; run parallel-passage research with computational suggestions explicitly labeled; include or exclude any knowledge domain (Quran, hadith, tafsir, scripture, books) per query.

</domain>

<decisions>
## Implementation Decisions

### Scripture Data Model
- Edition-aware: scripture editions (Torah, Tanakh, NT) with version identity, immutable hashes, lifecycle status (follow QuranEdition pattern)
- Addressing: (book, chapter, verse) with stable verse IDs `scripture:{edition}:{book}:{chapter}:{verse}`
- Matn/text stored canonically (immutable); translations as separate records
- Tradition/religion metadata (Jewish/Christian) as metadata, not code path

### Parallel-Passage Research
- Computational matching between Quran verses and scripture passages produces suggestion records with confidence + "uncertain" label
- Suggestions never presented as certainty; owner confirms/rejects (verse_links pattern)
- Domain inclusion/exclusion: query can specify which domains (quran, hadith, tafsir, scripture, books) to search; default all

### Cross-Domain Retrieval
- Unified search across domains via the RAG platform (Phase 8) with domain filters
- Each result tagged with domain + source + confidence
- No false scholarly-consensus claims; disagreement preserved

### the agent's Discretion
- Migration numbering (0039+), exact column names
- CLI/API/TUI surface shape

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `scripture` crate: placeholder, to implement
- Phase 8 RAG platform: domain filters, hybrid retrieval, transparency
- Phase 6 verse_links: confidence + owner gate pattern
- Phase 2 QuranEdition: edition identity pattern
- Phase 7 isnad: graph patterns

### Established Patterns
- Migrations 0035-0038 done; new 0039+
- Typed tools: ToolResult with research_checksum
- Cross-surface parity

### Integration Points
- application: scripture_reader, scripture_search, scripture_tools
- tool-registry: ScriptureBackend trait + tools
- cli/server/tui: scripture commands

</code_context>

<specifics>
## Specific Ideas

- Reuse Phase 8 RAG domain filters for cross-domain search
- Reuse verse_links owner-gate for parallel-passage suggestions
- Follow Phase 2 edition identity pattern

</specifics>

<deferred>
## Deferred Ideas

- Full scripture text ingestion (data acquisition) — schema + tools only
- Comparative theology/interpretation — not in scope

</deferred>
