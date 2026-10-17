# Phase 8: Multi-RAG Platform - Research

**Researched:** 2026-10-17
**Domain:** Multi-project RAG (config, hybrid FTS5+vector retrieval, reranking, routing transparency, tombstoning)
**Confidence:** HIGH (in-repo patterns) / MEDIUM (trait API design)

## Summary

Phase 8 turns three placeholder crates (`rag`, `embeddings`, `reranking`) into a working multi-project RAG platform on top of the existing `retrieval` port and SQLite storage. The work reuses the established application-service pattern (hadith/tafsir search services), the FTS5 normalization pattern from `quran-normalization`, the typed-tool contract from `tool-registry`, and the migration/checksum discipline from `storage-sqlite`.

The recommended shape: `embeddings` defines `EmbeddingProvider` with a deterministic local hash-based default; `reranking` defines `Reranker` with a score-based local default; `rag` orchestrates project lifecycle (TOML config + relative paths), document import, hybrid retrieval (FTS5 + vector, RRF merge, metadata filters), routing transparency (`route_decision` + reproducibility checksum), and tombstoning. Storage uses four new SQLite tables (migrations 0035-0038) with BLOB embeddings — no external vector DB. Application layer adds `rag_reader`, `rag_search`, `rag_tools`; tool-registry adds `RagBackend`; CLI/server/TUI gain `rag` commands.

**Primary recommendation:** Build SQLite-native hybrid retrieval (FTS5 + BLOB vectors) rather than reusing `MemvidRetrieval`, so results are reproducible, filterable, and tombstone-aware; keep `MemvidRetrieval` as an optional backend behind its existing feature flag.

## User Constraints

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Declarative TOML config per project directory (config.toml) defining corpus sources, embedding provider, reranker, router, and metadata filters
- Project is portable: config + index; no absolute paths; relative paths resolved against project dir
- CLI: `qai rag create <name>`, `qai rag import <project> <docs>`, `qai rag query <project> ...`
- Reuse Phase 5 typed-tool pattern for rag.* tools
- Full-text via SQLite FTS5 (reuse Phase 3 normalization) + vector via embeddings crate
- Embeddings crate: trait `EmbeddingProvider` with a local default (deterministic hash-based embedding as the no-network baseline) and pluggable providers
- Reranking crate: trait `Reranker` with a local default (score-based) and pluggable providers
- Merge FTS + vector results with reciprocal rank fusion (RRF); apply metadata filters before/after merge
- Vector storage: SQLite table with BLOB embedding column (portable, no external vector DB) — PG/Qdrant deferred to Phase 12
- Every query result carries a `route_decision` record: router name, tools invoked, sources consulted, scores per candidate, final ranking, reproducibility checksum
- Debug view: `qai rag debug <project> <query>` prints the full decision trace
- Reproducibility: same config + same corpus + same query → same result set (deterministic ordering, stable tie-break by id)
- Sources carry status (active/deactivated/tombstoned); query filters exclude non-active at retrieval time (immediate)
- Deactivation propagates to all derived index entries; no stale results

### the agent's Discretion
- Exact config schema fields, embedding dimension default, RRF k constant
- Test organization, CLI flag names

### Deferred Ideas (OUT OF SCOPE)
- PostgreSQL/Qdrant vector adapters — Phase 12
- Remote embedding providers (OpenAI etc.) — Phase 11/12
- Multi-tenant RAG — not in scope
</user_constraints>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| RAG project config (TOML) | CLI / application | storage-sqlite | Declarative file on disk; DB stores derived state |
| Document import + indexing | application (rag_reader) | storage-sqlite | Orchestrates FTS + vector writes |
| Hybrid retrieval (FTS+vector) | application (rag_search) | storage-sqlite | Merge + rerank + filter in one service |
| Embedding generation | embeddings crate | — | Trait + local default; pluggable |
| Reranking | reranking crate | — | Trait + local default; pluggable |
| Routing transparency | application (rag_search) | tool-registry | route_decision attached to ToolResult |
| Tombstoning/deactivation | storage-sqlite (rag_sources) | application | Status column; query-time filter |
| Typed tools (rag.*) | tool-registry (RagBackend) | application (rag_tools) | Follows hadith/tafsir backend pattern |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| toml | 0.8 (workspace) | Project config parsing | Already in workspace deps; serde derive |
| serde / serde_json | 1 (workspace) | Config + result serialization | Workspace standard |
| sqlx | 0.8 (workspace) | SQLite access | Workspace standard |
| sha2 | 0.10 (workspace) | Reproducibility checksums | Already used by tools crate |
| uuid | 1 (workspace) | Project/document ids | Workspace standard |
| quran-normalization | path | FTS5 normalization | Reuse L5 profile for text normalization |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| retrieval | path | MemvidRetrieval (optional) | Only if `--features memvid` backend selected |
| async-trait | 0.1 (workspace) | Trait objects for backends | Backend trait pattern |
| thiserror | 2 (workspace) | Error types | Crate error enums |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| SQLite BLOB vectors | Qdrant/PG | Deferred to Phase 12; BLOB keeps portable |
| Local hash embedding | ONNX embedder | Deferred; hash baseline is deterministic, no network |
| RRF merge | Learned ranker | Local score-based reranker default; pluggable |

**Installation:** No new external crates. `rag`, `embeddings`, `reranking` gain deps on existing workspace crates (serde, thiserror, sqlx via storage-sqlite, toml, sha2, uuid, async-trait, quran-normalization). `rag` may depend on `embeddings` and `reranking` (path).

**Version verification:** `toml = "0.8"` confirmed in workspace `[dependencies]` [VERIFIED: Cargo.toml workspace.dependencies]. `sha2 = "0.10"`, `uuid = "1"`, `sqlx = "0.8"` confirmed same source [VERIFIED: Cargo.toml workspace.dependencies].

## Package Legitimacy Audit

No new external packages. All dependencies are already in the workspace allowlist. No `npm view`/registry checks needed.

| Package | Registry | Verdict | Disposition |
|---------|----------|---------|-------------|
| toml | crates.io (workspace) | OK | Approved (already allowlisted) |
| sha2 | crates.io (workspace) | OK | Approved |
| sqlx | crates.io (workspace) | OK | Approved |

## Architecture Patterns

### Recommended Project Structure
```
crates/embeddings/src/
  lib.rs              # EmbeddingProvider trait + LocalHashEmbedding
crates/reranking/src/
  lib.rs              # Reranker trait + ScoreReranker
crates/rag/src/
  lib.rs              # RagProject, RagConfig, RagSource, RagDocument, RagHit, RouteDecision
  config.rs           # TOML config parsing + validation
  project.rs          # Project lifecycle (create/import)
  hybrid.rs           # FTS + vector retrieval, RRF, filters
  tombstone.rs        # Source status propagation
crates/application/src/
  rag_reader.rs       # RagReaderService
  rag_search.rs       # RagSearchService (hybrid + route_decision)
  rag_tools.rs        # RagToolBackend (RagBackend impl)
migrations/sqlite/
  0035_rag_projects.up.sql
  0036_rag_documents.up.sql
  0037_rag_embeddings.up.sql
  0038_rag_sources.up.sql
crates/cli/src/rag.rs
```

### Pattern 1: EmbeddingProvider trait
**What:** Trait with `embed(&self, text) -> Vec<f32>` and a deterministic local hash-based default.
**When to use:** All embedding generation; pluggable via config.
**Example:**
```rust
// Source: retrieval crate pattern (trait port)
pub trait EmbeddingProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn dimension(&self) -> usize;
    fn embed(&self, text: &str) -> Vec<f32>;
}
```

### Pattern 2: Reranker trait
**What:** Trait with `rerank(&self, query, candidates) -> Vec<RankedHit>`.
**When to use:** Post-merge reranking.
**Example:**
```rust
pub trait Reranker: Send + Sync {
    fn name(&self) -> &'static str;
    fn rerank(&self, query: &str, hits: &[RagHit]) -> Vec<RagHit>;
}
```

### Pattern 3: Hybrid retrieval with RRF
**What:** Run FTS5 (via quran-normalization L5) and vector (cosine over BLOB) in parallel, merge with RRF (k=60), apply metadata filters, rerank.
**When to use:** Every rag query.

### Pattern 4: RouteDecision transparency
**What:** Every result carries `route_decision: { router, tools, sources, scores, final_ranking, checksum }`. Checksum = sha256 over deterministic inputs (config + corpus gen + query + result ids).
**When to use:** All rag query/debug outputs.

### Anti-Patterns to Avoid
- **Mutating canonical text:** Never write normalized forms into rag_documents.text; keep FTS in separate rag_documents_fts (or reuse external-content FTS with content=). Follow T-06-05/T-06-06.
- **Stale tombstones:** Never cache results across deactivation; filter at query time AND propagate on deactivation.
- **Non-deterministic ordering:** Always stable tie-break by document id.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| TOML parsing | Custom parser | toml crate | Edge cases, serde derive |
| FTS5 indexing | Custom tokenizer | quran-normalization + SQLite FTS5 | Proven pattern |
| Checksums | Custom hash | sha2 (workspace) | Standard |
| Ids | Custom generator | uuid v4 | Standard |
| Tool envelope | Custom result | tools::ToolResult | Reproducibility contract |

**Key insight:** Reuse the hadith/tafsir search service pattern (Arc<SqliteDatabase> + reader) to keep surface parity.

## Runtime State Inventory

Not applicable (greenfield phase for these crates). No rename/migration of existing data.

## Common Pitfalls

### Pitfall 1: Vector storage in SQLite BLOB
**What goes wrong:** Float endianness / dimension mismatch.
**How to avoid:** Store dimension in rag_projects; validate on read; use little-endian f32 bytes.

### Pitfall 2: Tombstoning propagation
**What goes wrong:** Deactivated source leaves orphan embeddings.
**How to avoid:** On deactivation, delete rag_embeddings rows for that source's documents (or mark status and filter). Immediate query-time filter on rag_sources.status = 'active'.

### Pitfall 3: Reproducibility
**What goes wrong:** Non-deterministic HashMap iteration, float rounding.
**How to avoid:** Use BTreeMap, stable sort by id, fixed RRF k, deterministic embedding.

### Pitfall 4: FTS5 external content
**What goes wrong:** Stale FTS index after document update.
**How to avoid:** Rebuild FTS on import; tombstone removes from FTS via delete.

## Code Examples

### Config schema (TOML)
```toml
[project]
name = "my-rag"
version = "1"

[embedding]
provider = "local-hash"
dimension = 128

[reranker]
provider = "score"

[router]
name = "hybrid-rrf"

[[sources]]
path = "./docs"
glob = "**/*.md"
status = "active"
```

### RouteDecision struct
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteDecision {
    pub router: String,
    pub tools: Vec<String>,
    pub sources: Vec<String>,
    pub scores: Vec<(String, f32)>,
    pub final_ranking: Vec<String>,
    pub checksum: ContentHash,
}
```

## State of the Art

| Old Approach | Current Approach | Impact |
|--------------|------------------|--------|
| Memvid-only lexical | SQLite-native hybrid (FTS5 + BLOB vectors) | Portable, filterable, reproducible |

**Deprecated/outdated:**
- Memvid vector path: not used for Multi-RAG; stays opt-in.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | TOML config field names (discretion) | Code Examples | Low — user confirms |
| A2 | Embedding dimension default 128 | Standard Stack | Low |
| A3 | RRF k = 60 | Architecture Patterns | Low |
| A4 | Migration numbering 0035-0038 | Standard Stack | Medium — must match actual next version (verified 0034 last) [VERIFIED: migrations/sqlite listing shows 0034 last] |

## Open Questions

1. **TOML parsing crate** — recommend `toml` (workspace). [VERIFIED: Cargo.toml has toml = "0.8"]
2. **Embedding dimension default** — recommend 128 for local hash baseline.
3. **Reuse MemvidRetrieval vs SQLite-native** — recommend SQLite-native for portability/transparency; Memvid as optional backend.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | Build | ✓ | 1.97 (edition 2024) | — |
| SQLite (sqlx) | Storage | ✓ | 0.8 | — |
| toml crate | Config | ✓ | 0.8 | — |

**Missing dependencies with no fallback:** None.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (workspace) |
| Config file | Cargo.toml (workspace) |
| Quick run command | `cargo test -p rag -p embeddings -p reranking` |
| Full suite command | `cargo test --workspace` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| REQ-multi-rag | Create project from TOML | unit | `cargo test -p rag project::tests` | ❌ Wave 0 |
| REQ-multi-rag | Import documents | integration | `cargo test -p application rag_reader` | ❌ Wave 0 |
| REQ-multi-rag | Hybrid retrieval + filters | unit | `cargo test -p rag hybrid::tests` | ❌ Wave 0 |
| REQ-multi-rag | Routing transparency + checksum | unit | `cargo test -p rag route::tests` | ❌ Wave 0 |
| REQ-multi-rag | Tombstoning immediate | unit | `cargo test -p rag tombstone::tests` | ❌ Wave 0 |
| REQ-generic-rag-platform | Typed tool rag.* | integration | `cargo test -p application rag_tools` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p rag -p embeddings -p reranking`
- **Per wave merge:** `cargo test --workspace`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `crates/rag/src/lib.rs` tests
- [ ] `crates/embeddings/src/lib.rs` tests
- [ ] `crates/reranking/src/lib.rs` tests
- [ ] `crates/application/src/rag_reader.rs` tests
- [ ] `crates/application/src/rag_search.rs` tests
- [ ] `crates/application/src/rag_tools.rs` tests
- [ ] Migration files 0035-0038

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | single-user local |
| V5 Input Validation | yes | Validate TOML config, paths (no traversal), query non-empty |
| V6 Cryptography | yes | sha2 for checksums (never hand-roll) |

### Known Threat Patterns for Rust/SQLite

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Path traversal in project paths | Tampering | Resolve relative to project dir; reject `..` |
| SQL injection via metadata filters | Tampering | Parameterized queries / allowlisted columns |
| Stale tombstoned data leak | Information disclosure | Query-time status filter + propagation |

## Sources

### Primary (HIGH confidence)
- `crates/retrieval/src/lib.rs` — RetrievalBackend trait, BackendKind [VERIFIED: crates/retrieval/src/lib.rs:38-60]
- `crates/retrieval/src/document.rs` — Document/Hit types [VERIFIED: crates/retrieval/src/document.rs:1-40]
- `migrations/sqlite/` listing — 0034 last [VERIFIED: ls]
- `Cargo.toml` workspace deps — toml, sha2, uuid, sqlx [VERIFIED: Cargo.toml]
- `crates/application/src/tafsir_search.rs` — search service pattern [VERIFIED: crates/application/src/tafsir_search.rs:1-80]
- `crates/tool-registry/src/lib.rs` — TafsirBackend/HadithBackend patterns [VERIFIED: crates/tool-registry/src/lib.rs:279-410]
- `crates/cli/src/lib.rs` — dispatch pattern [VERIFIED: crates/cli/src/lib.rs:580-610]

### Secondary (MEDIUM confidence)
- `crates/quran-search/src/fts5.rs` — FTS5 usage pattern [VERIFIED: crates/quran-search/src/fts5.rs:1-60]

### Tertiary (LOW confidence)
- None.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - all deps already in workspace
- Architecture: HIGH - follows established patterns
- Pitfalls: MEDIUM - vector BLOB details need validation

**Research date:** 2026-10-17
**Valid until:** 2026-11-16
