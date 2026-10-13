# Phase 6: Hadith & Tafsir — Research

**Researched:** 2026-10-12
**Domain:** Structured hadith/tafsir records with attributed grading, verse linking, and scholarly disagreement in a Rust workspace
**Confidence:** HIGH for in-repo patterns (read directly this session); MEDIUM for data-model design (synthesized from existing patterns, not yet implemented)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Data Model**
- Hadith records follow the existing Quran corpus pattern: source-aware structured records with collection, edition, and source numbering preserved verbatim
- Matn/isnad separation where the source supports it; single-field where it doesn't
- Grading records are per-scholar (never consensus-merged), each attributed to its scholar with source citation
- Shia collections use the same schema with traditional-source metadata distinguishing them

**Grading Attribution**
- Each grading is a separate record: scholar name, grading value, source citation
- Multiple scholarly gradings for the same hadith are stored as independent records
- UI displays them side-by-side (success criterion 4: never silently merged)

**Verse Linking**
- Computational matching links hadith/tafsir to Quran locations, labeled as uncertain (success criterion 5)
- Owner gate confirms or rejects computational links before they become authoritative
- Data model: link record with source verse, target reference, confidence, and confirmation status

**Disagreement Display**
- Side-by-side layout in UI: each scholar's grading shown independently
- Data model: independent grading records, no consensus field
- Research surface shows all gradings with scholar attribution

**Computational Identity**
- All computational matches carry a confidence score and "uncertain" label
- No computational match is presented as certainty without owner confirmation
- Confidence thresholds: high (>0.9), medium (0.7-0.9), low (<0.7) — all labeled uncertain

**Tafsir Indexing**
- Indexed by work/author/dimensions following the existing catalog pattern
- Each tafsir record: work name, author, dimensions (volume, page, chapter), linked verses
- Comparative scripture (Torah, Tanakh, NT) uses edition-aware model from Phase 9

**Search**
- Exact + normalized search reusing Phase 3 normalization infrastructure
- Hadith search: matn text, narrator names, collection names
- Tafsir search: verse reference, author, content

**Shia Collections**
- Same schema as Sunni collections; traditional-source metadata field distinguishes them
- No separate code path — collection type is metadata, not structural

### the agent's Discretion
- No explicit discretion areas defined in CONTEXT.md. The agent applies judgment within the locked decisions above.

### Deferred Ideas (OUT OF SCOPE)
- Comparative scripture (Torah, Tanakh, NT) — Phase 9 scope
- Full narrator graph — Phase 7 scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| REQ-hadith-corpus | Source-aware structured hadith records with metadata, Shia collections, attributed grading, hadith tools (PRD §14). Acceptance: identifies collection+edition, preserves source numbering, separates isnad/matn where supported, attributes gradings, represents disagreement, resolves citations exactly, never presents computational identity matching as certainty (PRD §46) | Architecture Patterns 1-3 (source-aware records, grading attribution, verse linking); Data Model section; Pitfalls 1-5 |
| REQ-tafsir-comparative | Tafsir indexed by work/author/dimensions; comparative scripture (Torah, Tanakh/Hebrew Bible, NT) with comparative tools (PRD §16, §17) | Architecture Pattern 4 (tafsir indexing); Data Model section (comparative scripture deferred to Phase 9) |
</phase_requirements>

## Project Constraints (from AGENTS.md)

- **Primary Language:** Rust
- **Architecture:** Canonical Quran Engine + Knowledge Graph + Multi-RAG + Agentic Tool Runtime
- **Key Principles:** Exact text before generated interpretation; every factual claim traceable; canonical text separated from translations/annotations; no fabrication; distinguish quotation/source summary/AI analysis; deny-by-default permissions; no false scholarly consensus
- **Naming Conventions:** Tasks `TASK-nnn-slug.md`; Phases `phase-NN-slug/`; ADRs `ADR-nnnn-title.md`; Deliverables `Dn.n`; Acceptance Criteria `AC-Pn-nn`
- **Planning Rules:** Never modify master plan without justification; every task has TASK-ID; update status after implementation; record ADRs; never mark complete until acceptance criteria satisfied

## Summary

Phase 6 extends the Q-ai platform from Quran-only to **structured hadith and tafsir research**. The phase must deliver: (1) source-aware hadith records with collection/edition numbering preserved verbatim, (2) per-scholar grading records that never merge into false consensus, (3) tafsir indexed by work/author/dimensions, (4) verse linking with computational confidence + owner confirmation, and (5) cross-surface parity (CLI/API/TUI) reusing the Phase 5 typed-tool pattern.

The codebase already contains **four placeholder crates** reserved for this phase: `hadith-core`, `hadith-ingestion`, `tafsir`, and `isnad-graph` — all empty stubs [VERIFIED: crates/hadith-core/src/lib.rs:1-4, crates/tafsir/src/lib.rs:1-4, crates/hadith-ingestion/src/lib.rs:1-4, crates/isnad-graph/src/lib.rs:1-4]. The `SourceContentType` enum already has `Tafsir` and `HadithCollection` variants [VERIFIED: crates/sources/src/lib.rs:14-19]. This means the phase is a **greenfield implementation on top of mature infrastructure** — the source catalog, normalization, tool registry, citation, and provenance patterns are all built and tested.

The single highest-leverage finding: **every pattern Phase 6 needs already exists in the codebase**. The source catalog (`sources` → `source_versions` → `source_files`) provides the ingestion pipeline. The normalization engine (N01-N22, L0-L8) provides Arabic text search. The graph assertion store (`graph_assertions`) provides the attribution + evidence + review pattern for grading records. The tool registry (`QuranBackend` trait + `ToolRegistry`) provides the typed-tool surface. The citation resolver (`CitationSource` trait + `QuotationVerdict`) provides exact citation resolution. The planner's job is to **compose these patterns**, not invent new ones.

**Primary recommendation:** Sequence the phase as: (1) data model + migrations (hadith records, grading records, tafsir records, verse links); (2) domain crates (`hadith-core`, `tafsir`) with types following the `quran-core` pattern; (3) ingestion pipeline reusing the source catalog workflow; (4) search services reusing `quran-normalization` + FTS5; (5) typed tools extending `QuranBackend`; (6) verse linking with confidence + owner gate; (7) cross-surface parity via the Phase 5 `Envelope` pattern.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Hadith record storage (collection, edition, matn, isnad) | Database / Storage (`storage-sqlite`) | — | Follows `quran_editions` + `translation_passages` pattern; immutable text hashes |
| Grading attribution (per-scholar records) | Database / Storage | Application service | Follows `graph_assertions` pattern: independent records with attribution + evidence + review status |
| Tafsir indexing (work/author/dimensions) | Database / Storage | Application service | Follows `translation_editions` + `sources` pattern |
| Verse linking (computational match + owner confirm) | Application service | Database (link records) | Confidence scoring + approval gate; follows `graph_assertions` decision pattern |
| Hadith/tafsir search (exact + normalized) | Application service | `quran-normalization` + FTS5 | Reuses Phase 3 normalization pipeline; derived search forms |
| Typed tools (hadith.get, hadith.search, tafsir.get, tafsir.search) | API / Backend (`application` + `tool-registry`) | CLI, TUI, Web (thin callers) | Extends `QuranBackend` trait; follows Phase 5 `ToolResult` contract |
| Citation resolution (hadith/tafsir citations) | API / Backend (`citations`) | — | Extends `CitationSource` trait; hard-failure on mismatch |
| Disagreement display (side-by-side gradings) | Browser / SPA + TUI | API (data) | Presentation layer; data model guarantees independent records |
| Source catalog ingestion (discover→stage→validate→activate) | Application service (`sources`) | — | Already built; reuse `SourceState` workflow |

## Standard Stack

### Core — Rust (existing workspace crates)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `quran-normalization` | workspace | N01-N22 normalization rules, L0-L8 profiles, `NormalizationPipeline` | Phase 3 infrastructure; reusable for hadith/tafsir Arabic text [VERIFIED: crates/quran-normalization/src/rules/mod.rs:35-56] |
| `quran-search` | workspace | FTS5-backed exact/normalized/phrase search | Phase 3 search engine; tokenizer parity [VERIFIED: crates/quran-search/] |
| `tool-registry` | workspace | `QuranBackend` trait, `ToolRegistry`, typed tool params | Phase 5 tool surface; extensible for hadith/tafsir tools [VERIFIED: crates/tool-registry/src/lib.rs:75-150] |
| `tools` | workspace | `ToolResult<T>`, `ReproducibilityData`, `research_checksum` | Phase 5 result contract; required on every tool result [VERIFIED: crates/tools/src/lib.rs:56-89] |
| `citations` | workspace | `CitationSource` trait, `QuotationVerdict`, hard-failure semantics | Phase 5 citation resolution; extensible for hadith/tafsir citations [VERIFIED: crates/citations/src/lib.rs:33-77] |
| `domain` | workspace | `typed_id!` macro, `ContentHash`, `SemVer`, `LicenseRecord` | Phase 1 domain types; compile-time ID separation [VERIFIED: crates/domain/src/ids.rs:18-81] |
| `provenance` | workspace | `ProvenanceRecord`, `Attribution`, `SourceLocation`, `ApprovalToken` | Phase 4 provenance; attribution + evidence pattern [VERIFIED: crates/provenance/src/lib.rs] |
| `sources` | workspace | `Source`, `SourceVersion`, `SourceState`, `SourceContentType` | Phase 2 source catalog; ingestion workflow [VERIFIED: crates/sources/src/lib.rs] |
| `storage-sqlite` | workspace | SQLite persistence, `UnitOfWork`, repository traits | Storage backend; all migrations [VERIFIED: crates/storage-sqlite/] |

### New Crates (Phase 6)

| Crate | Purpose | Pattern Source |
|-------|---------|----------------|
| `hadith-core` | Hadith domain types: `HadithRecord`, `HadithCollection`, `GradingRecord`, `MatnRecord`, `IsnadRecord` | Follows `quran-core` pattern [VERIFIED: crates/quran-core/src/edition.rs] |
| `tafsir` | Tafsir domain types: `TafsirRecord`, `TafsirWork`, `TafsirDimensions` | Follows `quran-core` + `sources` pattern |
| `hadith-ingestion` | Hadith import/validation/activation pipeline | Follows `sources` + `quran-corpus` staging pattern |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `serde` | workspace | Serialization of domain types | All records |
| `sqlx` | workspace | SQLite access | All persistence |
| `uuid` | workspace | Typed ID generation | All new ID types |
| `sha2` | workspace | Content hashing | Immutable record hashes |
| `thiserror` | workspace | Typed errors | All fallible operations |
| `async-trait` | workspace | Async trait definitions | Repository + backend traits |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Reusing `graph_assertions` for grading records | New `hadith_gradings` table | `graph_assertions` is projection-scoped and tied to graph builds; grading records are standalone scholarly assertions. New table is simpler and avoids coupling to graph rebuilds. |
| Separate Shia/Sunni schemas | Same schema + metadata | CONTEXT.md decision: "No separate code path — collection type is metadata, not structural" |
| New normalization rules for hadith | Reuse N01-N22 | Hadith text is Arabic; same normalization rules apply. No new rules needed for v1. |

**Installation:** No new external dependencies needed. All required crates already exist in the workspace.

## Architecture Patterns

### System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                         User Surfaces                               │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐           │
│  │ Web GUI  │  │   TUI    │  │   CLI    │  │ HTTP API │           │
│  │  (SPA)   │  │(ratatui) │  │  (qai)   │  │ (/api/v1)│           │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘           │
│       │              │              │              │                 │
│       └──────────────┴──────────────┴──────────────┘                 │
│                              │                                       │
│                    ┌─────────▼─────────┐                             │
│                    │  Tool Registry    │                             │
│                    │  (QuranBackend    │                             │
│                    │   + HadithBackend │                             │
│                    │   + TafsirBackend)│                             │
│                    └─────────┬─────────┘                             │
│                              │                                       │
│              ┌───────────────┼───────────────┐                      │
│              │               │               │                      │
│       ┌──────▼──────┐ ┌─────▼─────┐ ┌───────▼──────┐               │
│       │ Hadith      │ │  Tafsir   │ │  Citation    │               │
│       │ Service     │ │  Service  │ │  Resolver    │               │
│       └──────┬──────┘ └─────┬─────┘ └───────┬──────┘               │
│              │               │               │                      │
│       ┌──────▼──────┐ ┌─────▼─────┐ ┌───────▼──────┐               │
│       │ Normalization│ │  Search   │ │  Verse Link  │               │
│       │ (N01-N22)   │ │  (FTS5)   │ │  + Owner Gate│               │
│       └──────┬──────┘ └─────┬─────┘ └───────┬──────┘               │
│              │               │               │                      │
│              └───────────────┼───────────────┘                      │
│                              │                                       │
│                    ┌─────────▼─────────┐                             │
│                    │  Storage Layer    │                             │
│                    │  (SQLite + FTS5)  │                             │
│                    └───────────────────┘                             │
└─────────────────────────────────────────────────────────────────────┘
```

### Recommended Project Structure

```
crates/
├── hadith-core/src/
│   ├── lib.rs              # HadithRecord, GradingRecord, collection types
│   ├── collection.rs       # HadithCollection, collection metadata
│   ├── grading.rs          # GradingRecord, GradingValue, scholar attribution
│   └── record.rs           # HadithRecord with matn/isnad separation
├── tafsir/src/
│   ├── lib.rs              # TafsirRecord, TafsirWork, TafsirDimensions
│   └── work.rs             # Work/author/dimension types
├── hadith-ingestion/src/
│   ├── lib.rs              # Import pipeline: discover→stage→validate→activate
│   ├── parser.rs           # Source format parsers
│   └── validate.rs         # Hadith-specific validation rules
├── application/src/
│   ├── hadith_tools.rs     # hadith.get, hadith.search, hadith.grading tools
│   ├── hadith_search.rs    # Search services reusing normalization
│   ├── hadith_reader.rs    # Read service for hadith records
│   ├── tafsir_tools.rs     # tafsir.get, tafsir.search tools
│   ├── tafsir_reader.rs    # Read service for tafsir records
│   ├── verse_link.rs       # Computational matching + owner gate
│   └── disagreement.rs      # Side-by-side grading assembly
├── server/src/
│   └── api.rs              # /api/v1/hadith/*, /api/v1/tafsir/* routes
└── cli/src/
    └── hadith.rs           # qai hadith *, qai tafsir * commands
```

### Pattern 1: Source-Aware Record (Phase 2 Pattern)
**What:** Every content record is tied to a `SourceVersion` with edition identity, immutable hashes, and lifecycle state.
**When to use:** For all hadith collections, tafsir works, and grading sources.
**Example:**
```rust
// Source: Phase 2 pattern [VERIFIED: migrations/sqlite/0007_quran_editions.up.sql]
// Hadith collection follows the same pattern as quran_editions
CREATE TABLE hadith_collections (
  id                     TEXT PRIMARY KEY,
  slug                   TEXT NOT NULL,
  version                TEXT NOT NULL,
  name                   TEXT NOT NULL,
  tradition              TEXT NOT NULL CHECK (tradition IN ('sunni','shia')),
  source_url             TEXT,
  license_json           TEXT NOT NULL,
  text_hash              TEXT NOT NULL,
  structure_hash         TEXT NOT NULL,
  source_version_id      TEXT NOT NULL REFERENCES source_versions(id),
  status                 TEXT NOT NULL CHECK (status IN
                            ('Staged','Approved','Active','Deprecated','Quarantined')),
  imported_at            TEXT NOT NULL,
  UNIQUE (slug, version)
);
```

### Pattern 2: Attribution + Evidence + Review (Phase 4 Pattern)
**What:** Scholarly assertions (gradings, annotations) are stored as independent records with attribution, evidence, and review decision — never merged.
**When to use:** For hadith grading records, verse link confirmation.
**Example:**
```rust
// Source: Phase 4 graph_assertions pattern [VERIFIED: migrations/sqlite/0019_quran_graph.up.sql]
// Grading records follow the same assertion pattern
CREATE TABLE hadith_gradings (
  id                TEXT PRIMARY KEY,
  hadith_id         TEXT NOT NULL REFERENCES hadith_records(id) ON DELETE RESTRICT,
  scholar_name      TEXT NOT NULL CHECK (length(trim(scholar_name)) > 0),
  grading_value     TEXT NOT NULL,
  source_citation   TEXT NOT NULL,
  evidence_json     TEXT NOT NULL,
  reviewer          TEXT,
  decision          TEXT NOT NULL CHECK (decision IN ('pending','accepted','rejected','superseded')),
  decided_at        TEXT,
  supersedes_id     TEXT REFERENCES hadith_gradings(id),
  provenance_layer  TEXT NOT NULL CHECK (provenance_layer IN ('B','C','D')),
  confidence        REAL,
  created_at        TEXT NOT NULL,
  CHECK (decision = 'pending' OR (reviewer IS NOT NULL AND reviewer != '' AND decided_at IS NOT NULL))
);
```

### Pattern 3: Computational Link with Confidence (Phase 4 + CONTEXT.md)
**What:** Computational matches carry confidence scores and "uncertain" labels; owner confirmation gates authoritative status.
**When to use:** For hadith/tafsir → Quran verse links.
**Example:**
```rust
// Verse link record with confidence + confirmation status
CREATE TABLE verse_links (
  id                TEXT PRIMARY KEY,
  source_type       TEXT NOT NULL CHECK (source_type IN ('hadith','tafsir')),
  source_id         TEXT NOT NULL,
  target_surah      INTEGER NOT NULL,
  target_ayah       INTEGER NOT NULL,
  confidence        REAL NOT NULL CHECK (confidence >= 0.0 AND confidence <= 1.0),
  label             TEXT NOT NULL DEFAULT 'uncertain' CHECK (label IN ('uncertain','confirmed','rejected')),
  algorithm         TEXT NOT NULL,
  algorithm_version TEXT NOT NULL,
  reviewer          TEXT,
  decided_at        TEXT,
  created_at        TEXT NOT NULL,
  CHECK (label != 'confirmed' OR (reviewer IS NOT NULL AND decided_at IS NOT NULL))
);
```

### Pattern 4: Typed Tool Extension (Phase 5 Pattern)
**What:** New tools extend the existing backend trait, returning `ToolResult<T>` with `research_checksum`.
**When to use:** For hadith.get, hadith.search, hadith.grading, tafsir.get, tafsir.search.
**Example:**
```rust
// Source: Phase 5 tool-registry pattern [VERIFIED: crates/tool-registry/src/lib.rs:75-150]
// Extend QuranBackend or add HadithBackend/TafsirBackend traits
#[async_trait]
pub trait HadithBackend: Send + Sync {
    async fn backend_get_hadith(
        &self,
        reference: &HadithRef,
    ) -> Result<(HadithView, BackendMeta), ToolError>;
    
    async fn backend_search_hadith(
        &self,
        _params: &HadithSearchToolParams,
    ) -> Result<ToolResult<serde_json::Value>, ToolError> {
        Err(unsupported("hadith.search"))
    }
    
    async fn backend_get_gradings(
        &self,
        hadith_id: &str,
    ) -> Result<Vec<GradingRecord>, ToolError>;
}
```

### Anti-Patterns to Avoid

- **Consensus-merged gradings:** Storing a single "consensus" grading value violates the core principle "no false scholarly consensus" (PRD §2). Always store per-scholar independent records.
- **Computational certainty:** Presenting a computational match (confidence > 0) as certainty without owner confirmation violates "never presents computational identity matching as certainty" (REQ-hadith-corpus acceptance).
- **Canonical text mutation:** Storing normalized hadith text in the same column as canonical text violates I8 (normalization never mutates canonical text). Always derive search forms into separate columns/tables.
- **Sectarian code paths:** Separate handling for Shia vs Sunni collections violates the locked decision "No separate code path — collection type is metadata, not structural."
- **Silent disagreement resolution:** Picking one grading as "correct" in the data layer violates the side-by-side display requirement. The UI must show all gradings; the data layer must not resolve them.

## Data Model Design

### Hadith Records

```rust
// Hadith record following the source-aware pattern
pub struct HadithRecord {
    pub id: HadithId,                    // New typed ID
    pub collection_id: HadithCollectionId,
    pub source_number: String,           // Preserved verbatim from source
    pub matn_text: String,               // Canonical matn text (immutable)
    pub matn_hash: ContentHash,          // Hash of canonical matn
    pub isnad_text: Option<String>,      // None when source doesn't separate
    pub narrator_names: Vec<String>,     // Extracted for search
    pub source_version_id: SourceVersionId,
    pub status: RecordStatus,
    pub metadata_json: serde_json::Value, // Collection-specific metadata
}

pub struct HadithCollection {
    pub id: HadithCollectionId,
    pub slug: String,
    pub version: String,
    pub name: String,
    pub tradition: Tradition,            // Sunni | Shia (metadata, not structural)
    pub source_version_id: SourceVersionId,
    pub status: CollectionStatus,
}

pub struct GradingRecord {
    pub id: GradingId,
    pub hadith_id: HadithId,
    pub scholar_name: String,           // Non-empty, attributed
    pub grading_value: String,           // e.g., "sahih", "hasan", "da'if"
    pub source_citation: String,         // Where this grading was published
    pub evidence_json: serde_json::Value, // Supporting evidence
    pub decision: ReviewDecision,        // pending | accepted | rejected | superseded
    pub reviewer: Option<String>,
    pub decided_at: Option<Timestamp>,
    pub provenance_layer: ProvenanceLayer, // B (publisher) | C (scholarly) | D (computational)
    pub confidence: Option<f64>,         // For computational suggestions
}
```

### Tafsir Records

```rust
pub struct TafsirRecord {
    pub id: TafsirId,
    pub work_id: TafsirWorkId,
    pub dimensions: TafsirDimensions,    // volume, page, chapter
    pub text: String,                    // Canonical tafsir text
    pub text_hash: ContentHash,
    pub verse_links: Vec<VerseLinkRef>,  // Linked Quran verses
    pub source_version_id: SourceVersionId,
    pub status: RecordStatus,
}

pub struct TafsirWork {
    pub id: TafsirWorkId,
    pub slug: String,
    pub name: String,
    pub author: String,                  // Non-empty, attributed
    pub dimensions_schema: DimensionsSchema, // How to interpret volume/page/chapter
    pub source_version_id: SourceVersionId,
    pub status: WorkStatus,
}

pub struct TafsirDimensions {
    pub volume: Option<u32>,
    pub page: Option<u32>,
    pub chapter: Option<String>,         // e.g., surah name or number
}
```

### Verse Link Records

```rust
pub struct VerseLink {
    pub id: VerseLinkId,
    pub source_type: LinkSourceType,     // Hadith | Tafsir
    pub source_id: String,
    pub target_surah: u16,
    pub target_ayah: u32,
    pub confidence: f64,                 // 0.0 - 1.0
    pub label: LinkLabel,                // uncertain | confirmed | rejected
    pub algorithm: String,
    pub algorithm_version: String,
    pub reviewer: Option<String>,
    pub decided_at: Option<Timestamp>,
}
```

### Migration Numbering

Existing migrations go up to `0022_quran_graph_fix.up.sql` [VERIFIED: migrations/sqlite/ directory listing]. Phase 6 migrations should start at `0023`:

| Migration | Purpose |
|-----------|---------|
| `0023_hadith_collections.up.sql` | Hadith collection records |
| `0024_hadith_records.up.sql` | Hadith records with matn/isnad |
| `0025_hadith_gradings.up.sql` | Per-scholar grading records |
| `0026_tafsir_works.up.sql` | Tafsir work metadata |
| `0027_tafsir_records.up.sql` | Tafsir content records |
| `0028_verse_links.up.sql` | Verse link records with confidence |

## Search Implementation

### Reusing Phase 3 Normalization

The Phase 3 normalization infrastructure is directly reusable for hadith/tafsir text:

1. **Normalization rules N01-N22** [VERIFIED: crates/quran-normalization/src/rules/mod.rs:35-56]: Hadith text is Arabic; the same rules (whitespace collapse, tatweel strip, harakat strip, hamza normalization, etc.) apply. No new rules needed.

2. **Normalization profiles L0-L8** [VERIFIED: migrations/sqlite/0013_quran_normalization.up.sql]: The ladder from exact (L0) to fuzzy (L8) is text-agnostic. Hadith search can use the same profiles.

3. **`NormalizationPipeline`** [VERIFIED: crates/quran-normalization/src/pipeline.rs]: The pipeline API takes text + profile, returns normalized text + trace. Hadith search services call the same pipeline.

4. **FTS5 derived forms** [VERIFIED: crates/application/src/quran_forms.rs]: The pattern of deriving search forms into separate tables (never mutating canonical text) is established. Hadith search forms go into `hadith_search_forms` table.

### Hadith Search Strategy

```rust
// Derived search forms (following quran_forms pattern)
CREATE TABLE hadith_search_forms (
  hadith_id    TEXT NOT NULL REFERENCES hadith_records(id) ON DELETE CASCADE,
  form_type    TEXT NOT NULL CHECK (form_type IN ('exact','normalized','skeleton','affix')),
  form_text    TEXT NOT NULL,
  profile_id   TEXT NOT NULL,
  profile_version TEXT NOT NULL,
  corpus_generation INTEGER NOT NULL,
  PRIMARY KEY (hadith_id, form_type, profile_id, profile_version)
);

-- FTS5 virtual table for matn text
CREATE VIRTUAL TABLE hadith_fts USING fts5(
  matn_text,
  narrator_names,
  collection_name,
  content='hadith_search_forms',
  content_rowid='rowid'
);
```

### Tafsir Search Strategy

Tafsir search indexes by verse reference, author, and content:

```rust
CREATE TABLE tafsir_search_forms (
  tafsir_id    TEXT NOT NULL REFERENCES tafsir_records(id) ON DELETE CASCADE,
  form_type    TEXT NOT NULL,
  form_text    TEXT NOT NULL,
  profile_id   TEXT NOT NULL,
  corpus_generation INTEGER NOT NULL,
  PRIMARY KEY (tafsir_id, form_type, profile_id)
);

CREATE VIRTUAL TABLE tafsir_fts USING fts5(
  content_text,
  author,
  verse_reference,
  content='tafsir_search_forms'
);
```

## Verse Linking

### Computational Matching

The verse linking system uses computational matching to suggest connections between hadith/tafsir and Quran verses:

1. **Text similarity:** Extract Arabic phrases from hadith/tafsir, normalize using N01-N22, match against normalized Quran text using FTS5.
2. **Named entity recognition:** If a hadith mentions a surah/ayah by name, create a candidate link.
3. **Confidence scoring:** Each match gets a confidence score based on match quality, phrase length, and context.

### Owner Gate

Following the Phase 4 `graph_assertions` pattern [VERIFIED: migrations/sqlite/0019_quran_graph.up.sql]:

1. Computational matches enter with `label = 'uncertain'`, `decision = 'pending'`.
2. Owner reviews and sets `label = 'confirmed'` or `'rejected'`.
3. Only `confirmed` links are displayed as authoritative.
4. All links display their confidence score and "uncertain" label until confirmed.

### Confidence Thresholds

Per CONTEXT.md locked decision:
- High: > 0.9
- Medium: 0.7 - 0.9
- Low: < 0.7
- **All labeled uncertain** regardless of confidence

## Disagreement Display

### Data Model

The data model guarantees independent grading records:

```
hadith_records (1) ←── (N) hadith_gradings
```

Each `hadith_gradings` row is one scholar's grading. There is **no consensus field**. The relationship is one-to-many.

### Display Logic

```rust
// Assembly of side-by-side gradings for display
pub struct DisagreementView {
    pub hadith_id: HadithId,
    pub gradings: Vec<GradingView>,  // All gradings, sorted by scholar name
}

pub struct GradingView {
    pub scholar_name: String,
    pub grading_value: String,
    pub source_citation: String,
    pub confidence: Option<f64>,
    pub label: String,  // "uncertain" for computational, "confirmed" for owner-reviewed
}
```

The UI renders `DisagreementView` as a side-by-side layout. No grading is hidden, merged, or resolved.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Arabic text normalization | Custom hadith normalizer | `quran-normalization` N01-N22 | Rules are language-specific, order-significant, span-mapping; already built and tested [VERIFIED: crates/quran-normalization/src/rules/mod.rs] |
| Full-text search | Custom search index | FTS5 + `quran-search` pattern | FTS5 handles Arabic text, ranking, tokenization; Phase 3 proved the pattern [VERIFIED: crates/quran-search/] |
| Content hashing | Custom hash function | `domain::ContentHash` + `canonical_json_bytes` | Frozen recipe (ADR-0006); changing it invalidates every stored hash [VERIFIED: crates/domain/src/hashing.rs] |
| Typed IDs | String IDs | `domain::typed_id!` macro | Compile-time separation; prevents ID confusion [VERIFIED: crates/domain/src/ids.rs:18-81] |
| Citation resolution | Custom matcher | `citations` crate `CitationSource` trait | Hard-failure semantics already implemented [VERIFIED: crates/citations/src/lib.rs] |
| Reproducibility checksum | Custom checksum | `tools::research_checksum` | Domain-separated, payload-covering, deterministic [VERIFIED: crates/tools/src/lib.rs:152-168] |
| Approval workflow | Custom approval logic | `provenance::ApprovalToken` + `approvals` table | Phase 1-2 pattern; state machine + actor tracking [VERIFIED: migrations/sqlite/0002_sources.up.sql] |

**Key insight:** Every infrastructure component Phase 6 needs already exists. The phase is composition, not invention.

## Common Pitfalls

### Pitfall 1: Consensus-Merged Gradings
**What goes wrong:** Storing a single "consensus" grading value (e.g., "majority says sahih") in the hadith record.
**Why it happens:** Tempting to simplify the data model by resolving disagreement at ingest time.
**How to avoid:** Never add a `consensus_grading` field. Store only per-scholar `hadith_gradings` records. The UI assembles the side-by-side view at read time.
**Warning signs:** A `consensus` or `majority` column in the schema; grading logic that "picks a winner."

### Pitfall 2: Computational Certainty Leak
**What goes wrong:** A computational verse link with confidence 0.95 is displayed as a confirmed link without owner review.
**Why it happens:** Confidence score is treated as authority.
**How to avoid:** The `label` field defaults to `'uncertain'` and can only be changed to `'confirmed'` by an owner action (reviewer + decided_at). Display the label, not just the confidence.
**Warning signs:** Links displayed without an "uncertain" badge; no review workflow for computational matches.

### Pitfall 3: Normalized Text in Canonical Column
**What goes wrong:** Storing normalized hadith text in the `matn_text` column, destroying the canonical text.
**Why it happens:** Convenience — one column serves both display and search.
**How to avoid:** `matn_text` is canonical-only. Derived search forms go into `hadith_search_forms` with a `corpus_generation` stamp. The canonical text is never mutated (I8).
**Warning signs:** `matn_text` column being written by search index rebuilds; no separate search forms table.

### Pitfall 4: Sectarian Code Paths
**What goes wrong:** Separate structs/tables for Shia vs Sunni collections (`shia_hadith_records`, `sunni_hadith_records`).
**Why it happens:** Different metadata requirements lead to separate schemas.
**How to avoid:** Single `hadith_records` table with a `tradition` metadata field on the collection. The record structure is identical; only metadata differs.
**Warning signs:** `tradition` column in `hadith_records` (should be in `hadith_collections`); match arms on tradition type in business logic.

### Pitfall 5: Missing Source Numbering
**What goes wrong:** Hadith records use internal sequential IDs instead of preserving the source's own numbering (e.g., "Bukhari 1234").
**Why it happens:** Convenience — internal IDs are easier to work with.
**How to avoid:** `source_number` field is NOT NULL, preserved verbatim from the source. The internal `id` is for database relationships only.
**Warning signs:** `source_number` is optional or empty; citations reference internal IDs instead of source numbers.

### Pitfall 6: Approval Gate Bypass
**What goes wrong:** Computational verse links become authoritative without owner confirmation.
**Why it happens:** The approval gate is implemented but not enforced in the read path.
**How to avoid:** The read path filters on `label = 'confirmed'` for authoritative display. Computational links are always labeled and gated.
**Warning signs:** Verse links displayed without checking the `label` field; no audit trail for link confirmation.

### Pitfall 7: Cross-Surface Parity Breaks
**What goes wrong:** Hadith tools return different results via CLI vs API vs TUI.
**Why it happens:** Each surface implements its own query logic.
**How to avoid:** All surfaces call the same `application` service layer. The `ToolResult` envelope with `research_checksum` guarantees identical payloads. Follow the Phase 5 parity gate pattern.
**Warning signs:** Divergent result payloads across surfaces; missing `research_checksum` on some tools.

## Code Examples

### Example 1: Hadith Record Insert (following quran_editions pattern)
```rust
// Source: Phase 2 pattern [VERIFIED: migrations/sqlite/0007_quran_editions.up.sql]
// Hadith collection insert with source version binding
let collection = HadithCollection {
    id: HadithCollectionId::new(),
    slug: "bukhari-arabic".to_string(),
    version: "1.0.0".to_string(),
    name: "Sahih al-Bukhari (Arabic)".to_string(),
    tradition: Tradition::Sunni,
    source_version_id: source_version_id.clone(),
    status: CollectionStatus::Staged,
};
// Insert with immutable hash verification
let text_hash = ContentHash::try_new(HashAlgorithm::Sha256, &hash_of_matn)?;
```

### Example 2: Grading Record with Attribution (following graph_assertions pattern)
```rust
// Source: Phase 4 pattern [VERIFIED: migrations/sqlite/0019_quran_graph.up.sql]
// Per-scholar grading — never merged
let grading = GradingRecord {
    id: GradingId::new(),
    hadith_id: hadith_id.clone(),
    scholar_name: "Al-Albani".to_string(),
    grading_value: "sahih".to_string(),
    source_citation: "Silsilat al-Ahadith al-Sahihah, hadith 1234".to_string(),
    evidence_json: serde_json::json!({ "chain_analysis": "..." }),
    decision: ReviewDecision::Pending,
    reviewer: None,
    decided_at: None,
    provenance_layer: ProvenanceLayer::C,  // Scholarly
    confidence: None,
};
```

### Example 3: Verse Link with Confidence (following graph_assertions pattern)
```rust
// Computational verse link — always uncertain until owner confirms
let link = VerseLink {
    id: VerseLinkId::new(),
    source_type: LinkSourceType::Hadith,
    source_id: hadith_id.to_string(),
    target_surah: 2,
    target_ayah: 255,
    confidence: 0.92,
    label: LinkLabel::Uncertain,  // Always starts uncertain
    algorithm: "phrase-match-v1".to_string(),
    algorithm_version: "1.0.0".to_string(),
    reviewer: None,
    decided_at: None,
};
// Owner confirms:
// UPDATE verse_links SET label = 'confirmed', reviewer = 'owner', decided_at = '...' WHERE id = ?;
```

### Example 4: Tool Result with Research Checksum (Phase 5 pattern)
```rust
// Source: Phase 5 pattern [VERIFIED: crates/tools/src/lib.rs:56-89]
// Every hadith/tafsir tool returns ToolResult with research_checksum
let result = ToolResult {
    tool_name: "hadith.get".to_string(),
    tool_version: SemVer::new(1, 0, 0),
    query: serde_json::json!({ "reference": "bukhari:1234" }),
    normalization_rules: vec![],
    edition_id: None,
    edition_version: None,
    results: hadith_json,
    canonical_references: vec!["bukhari:1234".to_string()],
    analysis_sources: vec![AnalysisSource {
        kind: "hadith".to_string(),
        reference: "bukhari:1234".to_string(),
    }],
    confidence: None,
    warnings: vec![],
    execution_time_ms: 12.5,
    reproducibility: reproducibility(...),
    research_checksum: research_checksum(
        "hadith.get",
        SemVer::new(1, 0, 0),
        &query,
        &canonical_inputs,
        &hadith_json,
    )?,
};
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Quran-only search | Hadith/tafsir search via same normalization | Phase 6 | Reuses N01-N22; no new rules needed |
| Single grading field | Per-scholar independent records | Phase 6 | Enables side-by-side disagreement display |
| Computational certainty | Confidence + owner gate | Phase 6 | Prevents false certainty claims |
| Separate Shia/Sunni schemas | Unified schema + metadata | Phase 6 | No sectarian code paths |

**Deprecated/outdated:**
- Consensus-merged grading values: Violates "no false scholarly consensus" (PRD §2)
- Computational matches without confidence labels: Violates "never presents computational identity matching as certainty" (REQ-hadith-corpus)

## Assumptions Log

> List all claims tagged `[ASSUMED]` in this research. The planner and discuss-phase use this section to identify decisions that need user confirmation before execution.

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Hadith text is sufficiently Arabic for N01-N22 normalization | Search Implementation | If hadith text includes significant non-Arabic content, additional normalization rules may be needed |
| A2 | FTS5 handles Arabic text adequately for hadith search | Search Implementation | If FTS5 Arabic tokenization is insufficient, a custom tokenizer may be needed |
| A3 | Source formats for hadith collections provide matn/isnad separation | Data Model | If sources don't separate matn/isnad, the `isnad_text` optional field handles it |
| A4 | Confidence thresholds (0.9, 0.7) are appropriate for verse linking | Verse Linking | Thresholds may need tuning after owner review of initial computational matches |
| A5 | `hadith-core`, `tafsir`, `hadith-ingestion` crate names are acceptable | Architecture | Crate names may conflict with future phases or naming conventions |

**If this table is empty:** All claims in this research were verified or cited — no user confirmation needed.

## Open Questions

1. **Hadith source format**
   - What we know: Sources must be identified with collection+edition, source numbering preserved verbatim.
   - What's unclear: What specific source formats (JSON, XML, text) will be imported? What fields do they provide?
   - Recommendation: The ingestion pipeline should support pluggable parsers. Start with a common format (JSON) and extend as needed.

2. **Grading value vocabulary**
   - What we know: Each grading has a `grading_value` string field.
   - What's unclear: Is there a controlled vocabulary (e.g., "sahih", "hasan", "da'if", "mawdu'") or free text?
   - Recommendation: Use free text with a suggested vocabulary in documentation. Constraining to an enum may exclude valid scholarly gradings.

3. **Verse linking algorithm**
   - What we know: Computational matching with confidence scores; algorithm + version recorded.
   - What's unclear: What specific algorithm(s) will be used for matching? Phrase overlap? Embedding similarity? Named entity recognition?
   - Recommendation: Start with phrase-overlap matching (deterministic, explainable). Add embedding-based matching in a later phase.

4. **Tafsir dimensions schema**
   - What we know: Tafsir records have dimensions (volume, page, chapter).
   - What's unclear: How do different tafsir works define their dimensions? Is there a standard schema?
   - Recommendation: Use a flexible `dimensions_json` field with a suggested schema per work. Validate at ingest time.

5. **Shia collection metadata**
   - What we know: Shia collections use the same schema with traditional-source metadata.
   - What's unclear: What specific metadata fields distinguish Shia collections (e.g., specific riwayah traditions)?
   - Recommendation: The `tradition` field (Sunni/Shia) plus `metadata_json` for tradition-specific fields. No structural difference.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust workspace | All | ✓ | Stable | — |
| SQLite + FTS5 | Storage | ✓ | Bundled | — |
| `quran-normalization` | Hadith/tafsir search | ✓ | workspace | — |
| `tool-registry` | Typed tools | ✓ | workspace | — |
| `citations` | Citation resolution | ✓ | workspace | — |
| `provenance` | Attribution + approval | ✓ | workspace | — |
| `sources` | Ingestion pipeline | ✓ | workspace | — |
| `domain` | Typed IDs + hashing | ✓ | workspace | — |

**Missing dependencies with no fallback:** None.

**Missing dependencies with fallback:** None.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust `#[cfg(test)]` + `cargo test` |
| Config file | `Cargo.toml` (workspace) |
| Quick run command | `cargo test -p hadith-core` |
| Full suite command | `cargo test --workspace` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| REQ-hadith-corpus | Hadith record insert + retrieval | unit | `cargo test -p hadith-core` | ❌ Wave 0 |
| REQ-hadith-corpus | Grading attribution (per-scholar) | unit | `cargo test -p hadith-core` | ❌ Wave 0 |
| REQ-hadith-corpus | Matn/isnad separation | unit | `cargo test -p hadith-core` | ❌ Wave 0 |
| REQ-hadith-corpus | Source numbering preserved | unit | `cargo test -p hadith-core` | ❌ Wave 0 |
| REQ-hadith-corpus | Shia collections same schema | unit | `cargo test -p hadith-core` | ❌ Wave 0 |
| REQ-hadith-corpus | Hadith search (exact + normalized) | integration | `cargo test -p application` | ❌ Wave 0 |
| REQ-hadith-corpus | Verse link confidence + label | unit | `cargo test -p application` | ❌ Wave 0 |
| REQ-hadith-corpus | Owner gate confirmation | unit | `cargo test -p application` | ❌ Wave 0 |
| REQ-hadith-corpus | Cross-surface parity | integration | `cargo test --workspace` | ❌ Wave 0 |
| REQ-tafsir-comparative | Tafsir record insert + retrieval | unit | `cargo test -p tafsir` | ❌ Wave 0 |
| REQ-tafsir-comparative | Tafsir indexed by work/author/dimensions | unit | `cargo test -p tafsir` | ❌ Wave 0 |
| REQ-tafsir-comparative | Tafsir search | integration | `cargo test -p application` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p hadith-core -p tafsir -p application`
- **Per wave merge:** `cargo test --workspace`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `crates/hadith-core/src/lib.rs` — domain types (currently placeholder)
- [ ] `crates/tafsir/src/lib.rs` — domain types (currently placeholder)
- [ ] `crates/hadith-ingestion/src/lib.rs` — ingestion pipeline (currently placeholder)
- [ ] `crates/application/src/hadith_tools.rs` — hadith tool backend
- [ ] `crates/application/src/tafsir_tools.rs` — tafsir tool backend
- [ ] `crates/application/src/hadith_search.rs` — search services
- [ ] `crates/application/src/verse_link.rs` — verse linking + owner gate
- [ ] `migrations/sqlite/0023_hadith_collections.up.sql` — hadith collections table
- [ ] `migrations/sqlite/0024_hadith_records.up.sql` — hadith records table
- [ ] `migrations/sqlite/0025_hadith_gradings.up.sql` — grading records table
- [ ] `migrations/sqlite/0026_tafsir_works.up.sql` — tafsir works table
- [ ] `migrations/sqlite/0027_tafsir_records.up.sql` — tafsir records table
- [ ] `migrations/sqlite/0028_verse_links.up.sql` — verse links table
- [ ] `crates/application/tests/hadith_tests.rs` — integration tests
- [ ] `crates/application/tests/tafsir_tests.rs` — integration tests
- [ ] Framework install: `cargo test --workspace` (no additional install needed)

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | Single-user local mode (Phase 1-6) |
| V3 Session Management | No | Single-user local mode |
| V4 Access Control | No | Single-user local mode; owner gate for computational links |
| V5 Input Validation | Yes | Serde deserialization + domain type validation |
| V6 Cryptography | Yes | SHA-256 content hashing (existing `domain::ContentHash`) |

### Known Threat Patterns for Rust + SQLite stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| SQL injection | Tampering | Parameterized queries via `sqlx` |
| Text normalization bypass | Tampering | Canonical text is immutable; search forms are derived |
| False scholarly consensus | Repudiation | Per-scholar independent grading records |
| Computational certainty | Spoofing | Confidence + label + owner gate on verse links |
| Source misattribution | Spelling | `provenance_records` + `source_citation` required on gradings |
| ID confusion | Tampering | Typed IDs via `domain::typed_id!` macro |

## Sources

### Primary (HIGH confidence)
- `crates/sources/src/lib.rs` — `SourceContentType` enum with `Tafsir` and `HadithCollection` variants [VERIFIED: lines 14-19]
- `crates/quran-core/src/edition.rs` — `QuranEdition` struct with immutable hashes, status lifecycle [VERIFIED: lines 37-96]
- `crates/quran-normalization/src/rules/mod.rs` — N01-N22 normalization rules [VERIFIED: lines 35-56]
- `crates/quran-normalization/src/pipeline.rs` — `NormalizationPipeline` API [VERIFIED: pub struct + methods]
- `crates/tool-registry/src/lib.rs` — `QuranBackend` trait, `ToolRegistry`, `BackendMeta` [VERIFIED: lines 46-150]
- `crates/tools/src/lib.rs` — `ToolResult<T>`, `ReproducibilityData`, `research_checksum` [VERIFIED: lines 27-207]
- `crates/citations/src/lib.rs` — `CitationSource` trait, `QuotationVerdict`, hard-failure semantics [VERIFIED: lines 33-100]
- `crates/domain/src/ids.rs` — `typed_id!` macro, `SourceId`, `EditionId`, etc. [VERIFIED: lines 18-146]
- `crates/domain/src/hashing.rs` — `ContentHash`, `HashAlgorithm`, `canonical_json_bytes` [VERIFIED: pub items]
- `crates/provenance/src/lib.rs` — `ProvenanceRecord`, `Attribution`, `ApprovalToken` [VERIFIED: pub items]
- `migrations/sqlite/0002_sources.up.sql` — Source catalog tables [VERIFIED: full file]
- `migrations/sqlite/0007_quran_editions.up.sql` — Edition pattern with immutable hashes [VERIFIED: full file]
- `migrations/sqlite/0010_quran_translations.up.sql` — Translation editions with attribution [VERIFIED: full file]
- `migrations/sqlite/0013_quran_normalization.up.sql` — Normalization rules + profiles [VERIFIED: full file]
- `migrations/sqlite/0019_quran_graph.up.sql` — Graph assertions with attribution + review [VERIFIED: full file]
- `crates/hadith-core/src/lib.rs` — Placeholder crate [VERIFIED: lines 1-4]
- `crates/tafsir/src/lib.rs` — Placeholder crate [VERIFIED: lines 1-4]
- `crates/hadith-ingestion/src/lib.rs` — Placeholder crate [VERIFIED: lines 1-4]
- `crates/isnad-graph/src/lib.rs` — Placeholder crate [VERIFIED: lines 1-4]
- `.planning/phases/06-hadith-tafsir/06-CONTEXT.md` — Locked decisions [VERIFIED: full file]
- `.planning/REQUIREMENTS.md` — REQ-hadith-corpus, REQ-tafsir-comparative [VERIFIED: lines 35-38]

### Secondary (MEDIUM confidence)
- Phase 5 research document — Typed tool pattern, result contract, cross-surface parity [CITED: .planning/phases/05-rich-quran-experience/05-RESEARCH.md]
- Phase 3 research document — Normalization infrastructure, search architecture [CITED: .planning/phases/03-quran-search-linguistics/03-RESEARCH.md]

### Tertiary (LOW confidence)
- None — all claims verified against source files this session.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all crates exist in workspace; versions verified by reading source
- Architecture: HIGH — patterns verified by reading existing migrations and domain types
- Pitfalls: HIGH — derived from locked decisions and existing pattern violations
- Data model: MEDIUM — synthesized from existing patterns; not yet implemented
- Search: HIGH — Phase 3 normalization infrastructure verified
- Verse linking: MEDIUM — pattern from Phase 4 verified; specific algorithm assumed

**Research date:** 2026-10-12
**Valid until:** 2026-11-12 (stable — workspace infrastructure, no external dependencies)
