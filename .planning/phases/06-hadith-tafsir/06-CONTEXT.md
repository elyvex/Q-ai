# Phase 6: Hadith & Tafsir - Context

**Gathered:** 2026-10-12
**Status:** Ready for planning
**Mode:** Auto-generated (discuss skipped — user chose to continue without interactive discussion)

<domain>
## Phase Boundary

Users can research structured hadith and tafsir linked to exact Quran locations with attributed gradings.

</domain>

<decisions>
## Implementation Decisions

### Data Model
- Hadith records follow the existing Quran corpus pattern: source-aware structured records with collection, edition, and source numbering preserved verbatim
- Matn/isnad separation where the source supports it; single-field where it doesn't
- Grading records are per-scholar (never consensus-merged), each attributed to its scholar with source citation
- Shia collections use the same schema with traditional-source metadata distinguishing them

### Grading Attribution
- Each grading is a separate record: scholar name, grading value, source citation
- Multiple scholarly gradings for the same hadith are stored as independent records
- UI displays them side-by-side (success criterion 4: never silently merged)

### Verse Linking
- Computational matching links hadith/tafsir to Quran locations, labeled as uncertain (success criterion 5)
- Owner gate confirms or rejects computational links before they become authoritative
- Data model: link record with source verse, target reference, confidence, and confirmation status

### Disagreement Display
- Side-by-side layout in UI: each scholar's grading shown independently
- Data model: independent grading records, no consensus field
- Research surface shows all gradings with scholar attribution

### Computational Identity
- All computational matches carry a confidence score and "uncertain" label
- No computational match is presented as certainty without owner confirmation
- Confidence thresholds: high (>0.9), medium (0.7-0.9), low (<0.7) — all labeled uncertain

### Tafsir Indexing
- Indexed by work/author/dimensions following the existing catalog pattern
- Each tafsir record: work name, author, dimensions (volume, page, chapter), linked verses
- Comparative scripture (Torah, Tanakh, NT) uses edition-aware model from Phase 9

### Search
- Exact + normalized search reusing Phase 3 normalization infrastructure
- Hadith search: matn text, narrator names, collection names
- Tafsir search: verse reference, author, content

### Shia Collections
- Same schema as Sunni collections; traditional-source metadata field distinguishes them
- No separate code path — collection type is metadata, not structural

</decisions>

<code_context>
## Existing Code Insights

- Phase 3 provides normalization infrastructure (N01-N22 rules) reusable for hadith text
- Phase 5 provides the typed tool registry, result contract, and cross-surface parity pattern
- Phase 4 provides graph traversal and annotation authority patterns
- Phase 2 provides the source-aware record pattern with edition/identity/license
- `qai` CLI tree, HTTP API, and TUI surfaces exist and are extensible

</code_context>

<specifics>
## Specific Ideas

- Reuse Phase 3 normalization for hadith text search
- Follow Phase 5 typed-tool pattern for hadith/tafsir tools
- Follow Phase 4 annotation authority pattern for grading attribution
- Follow Phase 2 source-record pattern for collection/edition numbering

</specifics>

<deferred>
## Deferred Ideas

- Comparative scripture (Torah, Tanakh, NT) — Phase 9 scope
- Full narrator graph — Phase 7 scope

</deferred>
