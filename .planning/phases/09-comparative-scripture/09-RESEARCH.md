# Phase 9: Comparative Scripture - Research

**Researched:** 2026-10-18
**Domain:** Comparative scripture (Torah, Tanakh, New Testament) alongside the Quran, with honest labeling
**Confidence:** HIGH (in-repo patterns) / MEDIUM (trait API + migration design)

## Summary

Phase 9 turns the `scripture` placeholder crate into a working comparative-scripture subsystem. It follows the established Phase 2 edition identity pattern (`QuranEdition`), the Phase 6 source-aware record pattern (`HadithRecord`/`TafsirRecord`), the Phase 6 `verse_links` owner-gate pattern, and the Phase 8 typed-tool/RAG pattern. Storage adds five forward-only migrations (0039-0043) for editions, books, verses, translations, and parallel-passage links. Application adds `scripture_reader`, `scripture_search`, `scripture_tools`; tool-registry adds a `ScriptureBackend` trait mirroring `TafsirBackend`/`HadithBackend`; CLI/server/TUI gain `scripture` commands.

Cross-domain research reuses Phase 8 RAG metadata filters for imported books and fans out to domain search services (Quran/hadith/tafsir/scripture) through a new application-level aggregator gated by a `KnowledgeDomain` inclusion set. Computational parallel-passage matches are persisted as `Uncertain` suggestions with confidence and an algorithm/version stamp, exactly like `verse_links`; only owner-confirmed links are authoritative. No suggestion is ever presented as certainty, and disagreement is preserved rather than collapsed into consensus.

**Primary recommendation:** Mirror the hadith/tafsir subsystem (edition + records + search forms + owner-gated links) rather than inventing a new shape; reuse `verse_links`'s confidence/label/algorithms by extending its `source_type` CHECK to include `'scripture'` and adding a dedicated `scripture_parallel_links` table only if scripture-to-scripture parallels are needed (deferred).

## User Constraints

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
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

### Deferred Ideas (OUT OF SCOPE)
- Full scripture text ingestion (data acquisition) — schema + tools only
- Comparative theology/interpretation — not in scope
</user_constraints>

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| REQ-comparative-scripture | Browse scripture editions with chapter/verse addressing and translation comparison; run parallel-passage research with computational suggestions labeled; include/exclude knowledge domains per query | Edition/verse/translation model (0039-0042); owner-gated parallel links (0043 + verse_links pattern); `KnowledgeDomain` filter + cross-domain aggregator; `ScriptureBackend` typed tools |

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Scripture edition identity (hashes, lifecycle) | storage-sqlite (scripture_editions) | application (scripture_reader) | Immutable hashes + status CHECK, mirroring quran_editions |
| Verse addressing (book,chapter,verse) + stable IDs | scripture crate (domain types) | storage-sqlite (scripture_verses) | Pure domain types; no I/O in crate (I2-style) |
| Canonical text storage | storage-sqlite (scripture_verses.text) | application (scripture_reader) | Immutable; search forms derived separately |
| Translations | storage-sqlite (scripture_translations) | application (scripture_reader) | Separate records per verse, never mutate canonical text |
| Parallel-passage suggestions | storage-sqlite (verse_links/source_type='scripture') | application (verse_link service) | Reuse owner-gate CHECK + confidence/label |
| Search (FTS5) | application (scripture_search) | storage-sqlite (scripture_fts) | L5-normalized forms, mirroring hadith/tafsir |
| Cross-domain research | application (research aggregator) | tool-registry (ScriptureBackend) | Fans out to domain services; tags domain+source+confidence |
| Domain inclusion/exclusion | application (KnowledgeDomain filter) | tool-registry | Default all; explicit subset |
| Typed tools (scripture.*) | tool-registry (ScriptureBackend) | application (scripture_tools) | Follows TafsirBackend/HadithBackend pattern |
| CLI/server/TUI surfaces | cli/server/tui | application | Construct services from db path, mirroring tafsir.rs |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| domain | path | Shared IDs, hashes, timestamps, languages | Workspace standard |
| scripture (new impl) | path | Pure domain types (edition, verse, translation, ref) | Follows quran-core/hadith-core/tafsir pattern |
| sqlx | 0.8 (workspace) | SQLite access | Workspace standard |
| serde / serde_json | 1 (workspace) | Serialization | Workspace standard |
| async-trait | 0.1 (workspace) | Backend traits | Workspace standard |
| thiserror | 2 (workspace) | Error types | Workspace standard |
| quran-normalization | path | L5 normalization for FTS | Reuse Phase 3/6 pattern |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| quran-core | path | QuranRef / QuranEdition reference for parallel targets | Reuse for verse candidate addressing |
| application::verse_link | path | Owner-gate pattern + confidence bands | Reuse `LinkLabel`, `ConfidenceBand`, `VerseLink` semantics |
| rag | path | Phase 8 RAG for imported books | Domain filter `domain=books` |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Separate scripture_parallel_links table | Extend verse_links source_type CHECK | Extending is smaller migration and reuses read path; separate table only if scripture-to-scripture parallels needed (deferred) |
| Per-verse translation editions | Per-verse translation records | Per-verse records simpler and match tafsir dimension pattern; edition-level only if translator-level hashing needed |
| New FTS tokenizer | unicode61 default | Keep default tokenizer to match hadith/tafsir (Arabic combining marks preserved) |

**Installation:** No new external dependencies. Add `scripture` crate implementation + wire into `application`/`tool-registry` Cargo deps as needed (path deps already declared).

**Version verification:** No new crates; all versions come from workspace. [VERIFIED: Cargo.toml workspace]

## Package Legitimacy Audit

No new external packages are introduced by this phase. All dependencies are existing workspace path crates. Audit: N/A — no installs.

## Migration Numbering (0039+)

Forward-only (no `.down.sql`), following Phase-1 convention. Last existing migration is `0038_rag_sources.up.sql`.

| # | File | Purpose |
|---|------|---------|
| 0039 | `scripture_editions.up.sql` | Edition identity: slug/version/name, tradition metadata, language, license_json, text_hash/structure_hash/manifest_hash, source_version_id, status, immutable-hash trigger |
| 0040 | `scripture_books.up.sql` | Book catalog per edition: book_slug, book_name, book_order, chapter_count |
| 0041 | `scripture_verses.up.sql` | Verses: edition_id, book, chapter, verse, text, text_hash, status; PK stable id `scripture:{edition}:{book}:{chapter}:{verse}`; UNIQUE(edition_id, book, chapter, verse) |
| 0042 | `scripture_translations.up.sql` | Translations: verse_id, language, translator, text, text_hash, status |
| 0043 | `scripture_search_forms.up.sql` | Derived search forms + FTS5 virtual table (scripture_fts), mirroring hadith/tafsir FTS pattern |
| 0044 | `verse_links_scripture_source.up.sql` | Extend `verse_links.source_type` CHECK to include `'scripture'` (table rebuild or CHECK relaxation) so scripture passages can be parallel-passage sources |

Note: migration 0044 may require a table rebuild because SQLite CHECK constraints are immutable; follow the existing rebuild pattern. If rebuild risk is unacceptable, create a dedicated `scripture_parallel_links` table mirroring `verse_links` columns (source_scripture_ref, target_surah, target_ayah, confidence, label, algorithm, algorithm_version, reviewer, decided_at, created_at) with the same owner-gate CHECK. [ASSUMED: exact rebuild approach; planner decides]

## Existing Patterns to Reuse

### QuranEdition (Phase 2)
- `crates/quran-core/src/edition.rs` — `QuranEdition` struct: id, slug, version, name, script, riwayah, qiraah, publisher, source_url, license, language, numbering scheme, basmala policy, unicode normalization, text_hash/structure_hash/token_order_hash/manifest_hash, version, source_version_id, imported_at, verified_at/by/method, status, statistics. Immutable-hash trigger `trg_edition_immutable_hashes`.
- Reuse: same identity/hash/lifecycle shape for `ScriptureEdition`, dropping Quran-specific fields (script, riwayah, qiraah, basmala_policy, numbering_scheme, token_order_hash) and adding `tradition` (Jewish/Christian) + `religion` metadata. Status enum reuse: `Staged/Approved/Active/Deprecated/Quarantined`.

### verse_links (Phase 6)
- `migrations/sqlite/0030_verse_links.up.sql` — confidence 0..1, label in (uncertain, confirmed, rejected), owner-gate CHECK (confirmed requires reviewer+decided_at), algorithm + algorithm_version.
- `crates/application/src/verse_link.rs` — `VerseLink`, `LinkLabel`, `ConfidenceBand`, `VerseLinker` trait, `VerseLinkService`, `score_overlap`, `suggested()`, `is_authoritative()`.
- Reuse: parallel-passage suggestions are `VerseLink`-shaped with `source_type = Scripture` (extend enum) and target = Quran verse (surah, ayah). Confidence bands display-only; label starts `Uncertain`.

### Hadith source records (Phase 6)
- `migrations/sqlite/0023_hadith_collections.up.sql`, `0024_hadith_records.up.sql`, `0026_hadith_search_forms.up.sql`.
- Pattern: collection/work + record with canonical text + text_hash + source_version_id + status; derived search forms + FTS5 virtual table; status-filtered reads.
- Reuse: `ScriptureEdition` ≈ collection, `ScriptureVerse` ≈ record, `scripture_search_forms` + `scripture_fts` ≈ search forms.

### Typed tools (Phase 5/6/8)
- `crates/tool-registry/src/lib.rs` — `QuranBackend`, `HadithBackend`, `TafsirBackend`, `RagBackend` traits with `BackendMeta`, default-unsupported methods, `ToolResult` + `research_checksum`.
- Reuse: add `ScriptureBackend` trait with `backend_get_scripture(reference)`, `backend_search_scripture(params)`, `backend_list_editions()`, `backend_get_translation(verse_id, language)`, `backend_parallel_passages(quran_ref)` methods. Application implements `ScriptureToolBackend`.

### RAG domain filters (Phase 8)
- `crates/application/src/rag_search.rs` — `query(project, query_text, filters: BTreeMap, limit)` applies metadata filters; `RagHit` carries doc_id/score/source/rank.
- Reuse: imported scripture books carry `metadata.domain = "scripture"` (or "books"); RAG `query` filters by domain. Cross-domain aggregator unions domain-tagged results.

## Recommended Architecture

### scripture crate (pure domain, no I/O)
Follow quran-core/hadith-core/tafsir invariants (no storage/sqlx/tokio/llm).

- `crates/scripture/src/edition.rs` — `ScriptureEdition`, `ScriptureEditionStatistics`, `Tradition` (Jewish/Christian metadata enum), `EditionStatus` reuse from domain or local enum.
- `crates/scripture/src/book.rs` — `ScriptureBook { id, edition_id, slug, name, order, chapter_count }`.
- `crates/scripture/src/verse.rs` — `ScriptureVerse { id: ScriptureVerseId, edition_id, book, chapter, verse, text, text_hash, status, ... }` with stable id `scripture:{edition}:{book}:{chapter}:{verse}`.
- `crates/scripture/src/translation.rs` — `ScriptureTranslation { id, verse_id, language, translator, text, text_hash, status }`.
- `crates/scripture/src/reference.rs` — parser/serializer for `(book, chapter, verse)` addressing and edition embedding; `ScriptureRef`.
- `crates/scripture/src/view.rs` — `ScriptureView` (verse + translations) for tool envelopes.
- Stable id helper: `ScriptureVerseId::from_parts(edition, book, chapter, verse)`.

### application services
- `application::scripture_reader` — `ScriptureReader` trait + `SqliteScriptureReader`: edition CRUD, book CRUD, verse CRUD, translation CRUD, status filters. Mirrors `TafsirReaderService`.
- `application::scripture_search` — `ScriptureSearchService`: FTS5 search over `scripture_fts`, L5 normalization, status filter. Mirrors `TafsirSearchService`.
- `application::scripture_tools` — `ScriptureToolBackend` implementing `ScriptureBackend`.
- `application::research_aggregator` (new) or extend — `CrossDomainResearchService` taking `domains: BTreeSet<KnowledgeDomain>` (default all five) and fanning out to quran/hadith/tafsir/scripture search + RAG (books). Returns merged results tagged `{domain, source, confidence}`. Applies domain filter before retrieval (no wasted queries for excluded domains).

### tool-registry
- Add `ScriptureBackend` trait (mirroring `TafsirBackend`) with default-unsupported methods.
- Add `scripture_backend: Option<Arc<dyn ScriptureBackend>>` to `ToolRegistry`, plus `with_scripture`/`set_scripture_backend`/accessor.
- Register tool names: `scripture.get`, `scripture.search`, `scripture.translations`, `scripture.parallel_passages`, `scripture.editions`.
- Add `is_mutating_scripture_tool` for any owner-gate mutations (confirm/reject parallel link).

### verse_links extension
- Extend `LinkSourceType` enum with `Scripture` variant (as_str "scripture") for parallel-passage suggestions (Quran target).
- Migration 0044 extends `verse_links.source_type` CHECK. Application `source_text()` gains a `Scripture` arm loading `scripture_verses.text`.

### CLI/server/TUI
- `crates/cli/src/scripture.rs` — `qai scripture editions`, `get <edition> <book> <chapter> <verse>`, `search <query>`, `translations <verse_id>`, `parallel <quran_ref>`, `link confirm/reject <id>`. Mirrors `tafsir.rs`.
- Server routes + TUI panels follow existing hadith/tafsir surfaces.

### System Architecture Diagram

```
User query (CLI/server/TUI/agent)
        |
        v
  [CrossDomainResearchService] --domains--> {Quran,Hadith,Tafsir,Scripture,Books}
        |                                          |
        | (domain filter)                          v
        |                               [per-domain search services]
        |                               quran_search / hadith_search /
        |                               tafsir_search / scripture_search /
        |                               rag_search (books, domain filter)
        v
  [merged results tagged domain+source+confidence]
        |
        v
  [ScriptureBackend typed tools]  -->  ToolResult + research_checksum

Parallel-passage flow:
  scripture verse text  --score_overlap-->  Quran ayah candidates
        |                                           |
        v                                           v
  [VerseLink::suggested (Uncertain, confidence)] --owner gate--> confirmed/rejected
```

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Edition identity/hashes | Custom edition struct | Mirror `QuranEdition` | Proven immutable-hash + lifecycle pattern |
| Owner-gated suggestions | New gate logic | Reuse `verse_links` pattern + `LinkLabel`/`ConfidenceBand` | DB CHECK + service decide() already tested |
| FTS normalization | New normalizer | `quran-normalization` L5 profile | Consistent with hadith/tafsir |
| Hybrid retrieval for books | Custom vector store | Phase 8 RAG (SQLite BLOB) | Already implemented, filterable, tombstone-aware |
| Typed tool envelopes | Ad-hoc JSON | `ToolResult` + `research_checksum` | Cross-surface parity + reproducibility |

**Key insight:** The cheapest correct implementation is a copy of the hadith/tafsir subsystem with `(book, chapter, verse)` addressing and a tradition metadata field.

## Common Pitfalls

### Pitfall 1: Presenting computational parallels as certainty
**What goes wrong:** Parallel-passage suggestions shown as "this verse parallels X" without uncertainty.
**Why it happens:** Confidence > 0.9 mistaken for proof.
**How to avoid:** Label always `Uncertain` until owner confirms; confidence bands display-only; read path filters `label='confirmed'` by default (mirrors `get_links_for_verse(include_uncertain=false)`).
**Warning signs:** Any UI/API exposing `is_authoritative()` false as authoritative.

### Pitfall 2: False scholarly consensus
**What goes wrong:** Merging domain results into a single "consensus" answer, hiding disagreement.
**How to avoid:** Preserve per-domain results and disagreement; tag domain+source+confidence; never collapse. [CITED: docs/01-requirements/requirements.md — no false consensus]

### Pitfall 3: Domain filter defaulting
**What goes wrong:** Defaulting to a subset (e.g., only Quran) silently.
**How to avoid:** Default `KnowledgeDomain::all()` (all five); explicit opt-out only; reflect selection in result metadata.

### Pitfall 4: Immutable text mutation
**What goes wrong:** Search index rebuilds mutating canonical `text`.
**How to avoid:** Canonical text in `scripture_verses.text` never mutated; derived forms in `scripture_search_forms` + `scripture_fts` (T-06-05/T-06-06 pattern).

### Pitfall 5: Stable verse ID drift
**What goes wrong:** ID format changes or includes mutable fields.
**How to avoid:** ID = `scripture:{edition}:{book}:{chapter}:{verse}` using immutable edition slug/version? Use edition id (stable) not slug to avoid rename drift; document format. [ASSUMED: exact id uses edition id vs slug; planner confirms]

### Pitfall 6: Migration CHECK immutability
**What goes wrong:** `ALTER TABLE ... ADD CHECK` unsupported in SQLite.
**How to avoid:** Table rebuild for `verse_links.source_type` extension, or dedicated `scripture_parallel_links` table. Test migration on a populated DB.

### Pitfall 7: Tradition as code path
**What goes wrong:** Branching logic per religion (Jewish vs Christian).
**How to avoid:** Tradition is metadata only; addressing and storage are religion-agnostic (follows hadith `tradition` metadata decision).

## Code Examples

### ScriptureRef addressing
```rust
// crates/scripture/src/reference.rs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptureRef {
    pub edition: String,   // edition slug or id
    pub book: String,      // book slug, e.g. "genesis"
    pub chapter: u32,
    pub verse: u32,
}
// stable id
pub fn verse_id(edition: &str, book: &str, chapter: u32, verse: u32) -> String {
    format!("scripture:{edition}:{book}:{chapter}:{verse}")
}
```

### Parallel-passage suggestion (reuse verse_links)
```rust
// Extend LinkSourceType
pub enum LinkSourceType { Hadith, Tafsir, Scripture }
// source_text() arm:
LinkSourceType::Scripture => {
    sqlx::query_scalar("SELECT text FROM scripture_verses WHERE id = ?")
        .bind(source_id).fetch_optional(...).await?
}
// Suggestion always uncertain:
let link = VerseLink::suggested(
    LinkSourceType::Scripture, scripture_verse_id, surah, ayah,
    confidence, "phrase-overlap-l5".into(), "1.0.0".into(), now,
);
```

### KnowledgeDomain filter
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KnowledgeDomain { Quran, Hadith, Tafsir, Scripture, Books }
impl KnowledgeDomain { pub fn all() -> BTreeSet<Self> { ... } }
pub struct CrossDomainQuery { pub query: String, pub domains: BTreeSet<KnowledgeDomain> } // default all
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| None (placeholder crate) | Edition-aware scripture subsystem | Phase 9 | Enables comparative research |
| verse_links (hadith/tafsir only) | Extended to scripture sources | Phase 9 | Parallel-passage suggestions |

**Deprecated/outdated:** None.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Stable verse ID uses edition id vs slug | Code Examples | Medium — id drift if edition renamed |
| A2 | Migration 0044 rebuild approach for CHECK extension | Migration Numbering | Medium — SQLite rebuild complexity |
| A3 | `KnowledgeDomain` lives in application or scripture crate | Architecture | Low — placement only |
| A4 | REQ-comparative-scripture requirement id name | Phase Requirements | Low — naming only |
| A5 | Cross-domain aggregator is application-level, not a new RAG project | Architecture | Medium — affects domain filter wiring |

## Open Questions

1. **Should scripture parallels target Quran only or scripture-to-scripture too?**
   - What we know: Context emphasizes Quran alongside scripture.
   - What's unclear: scripture-to-scripture parallels.
   - Recommendation: Quran-target only in Phase 9; defer scripture-to-scripture.

2. **Should `scripture` crate depend on quran-core for QuranRef?**
   - Recommendation: application layer bridges; keep scripture crate pure (no quran-core dep) to preserve I2-style purity.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | Build | ✓ | (workspace) | — |
| SQLite (via sqlx) | Storage | ✓ | (workspace) | — |
| Cargo test | Validation | ✓ | — | — |

**Missing dependencies with no fallback:** None.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (Rust) |
| Config file | Cargo.toml (workspace) |
| Quick run command | `cargo test -p scripture -p application` |
| Full suite command | `cargo test --workspace` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| REQ-comparative-scripture | Edition serde roundtrip | unit | `cargo test -p scripture edition` | ❌ Wave 0 |
| REQ-comparative-scripture | Verse stable id format | unit | `cargo test -p scripture verse_id` | ❌ Wave 0 |
| REQ-comparative-scripture | Translation separate records | unit | `cargo test -p scripture translation` | ❌ Wave 0 |
| REQ-comparative-scripture | Parallel suggestion starts Uncertain | unit | `cargo test -p application scripture_parallel` | ❌ Wave 0 |
| REQ-comparative-scripture | Owner gate confirms/rejects | integration | `cargo test -p application verse_link_scripture` | ❌ Wave 0 |
| REQ-comparative-scripture | Domain filter excludes domain | unit | `cargo test -p application cross_domain_filter` | ❌ Wave 0 |
| REQ-comparative-scripture | Cross-surface parity | integration | `cargo test -p cli parity_scripture` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p scripture -p application`
- **Per wave merge:** `cargo test --workspace`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `crates/scripture/tests/` — edition/verse/translation/ref unit tests
- [ ] `crates/application/tests/scripture_*.rs` — reader/search/parallel-link service tests
- [ ] Migration test harness entries for 0039-0044
- [ ] CLI parity test additions

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | yes | Owner gate: confirm/reject requires non-empty reviewer + decided_at (DB CHECK) |
| V5 Input Validation | yes | ScriptureRef parse validation; FTS query escaping (phrase-quoting); edition/book slug validation |
| V6 Cryptography | yes | Content hashes (SHA-256 via domain::ContentHash); never hand-roll |

### Known Threat Patterns for Rust/SQLite stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| SQL injection via FTS query | Tampering | Phrase-quoting + escaping (mirrors rag_search.search_fts) |
| False consensus / overclaim | Repudiation | Honest labeling: Uncertain until owner confirms; domain tagging; no collapse |
| Canonical text mutation | Tampering | Immutable text + derived search forms (T-06-05/T-06-06) |
| Unreviewed suggestion served as authoritative | Elevation | Read path filters label='confirmed' by default |

## Sources

### Primary (HIGH confidence)
- `crates/quran-core/src/edition.rs` — QuranEdition pattern
- `crates/application/src/verse_link.rs` — owner-gate pattern
- `migrations/sqlite/0030_verse_links.up.sql` — confidence + owner-gate CHECK
- `migrations/sqlite/0023/0024/0027/0028/0029` — hadith/tafsir record + search forms pattern
- `crates/tool-registry/src/lib.rs` — backend trait pattern
- `crates/application/src/tafsir_tools.rs`, `rag_tools.rs` — typed tool backend pattern
- `crates/application/src/rag_search.rs` — domain filter pattern

### Secondary (MEDIUM confidence)
- `.planning/phases/08-multi-rag-platform/08-RESEARCH.md` — phase research format

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - all existing workspace deps
- Architecture: HIGH - mirrors proven hadith/tafsir/RAG patterns
- Pitfalls: MEDIUM - honest-labeling discipline depends on execution

**Research date:** 2026-10-18
**Valid until:** 2026-11-17 (30 days)
