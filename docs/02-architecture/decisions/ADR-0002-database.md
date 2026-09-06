# Part A — Database engine recommendation

Short answer: **one embedded relational spine (SQLite) as the single source of truth, plus purpose-built embedded engines mounted behind traits for full-text, vectors, and graph — all swappable to server-grade backends without touching domain code.**

## A.1 Recommended default profile (`local`, zero-config)

| Layer | Default (zero-config, embedded) | Why |
|---|---|---|
| Relational / canonical / metadata / jobs / audit | **SQLite** (WAL, `synchronous=FULL`, `foreign_keys=ON`) | Single file, no daemon, transactional, mature, perfect for immutable canonical rows + triggers; already assumed by PRD §32.1 |
| Full-text (Arabic-aware, BM25, phrase, proximity) | **Tantivy** (embedded, pure Rust, a directory on disk) | Custom tokenizer required for your normalization forms; BM25 + positional phrase queries; no server; PRD §32.2 already suggests it |
| Vector | **`sqlite-vec`** (default) → **LanceDB** (when corpora grow) | `sqlite-vec` keeps everything in the *same file* = truly zero-config; LanceDB gives real ANN (IVF/HNSW) still file-based |
| Graph | **SQLite adjacency tables + recursive CTEs** | PRD §32.4 explicitly forbids requiring a graph DB for local MVP; typed edges + provenance columns are trivially relational |
| Analytics (frequency, distribution, co-occurrence, collocation) | **SQLite aggregates**, optional **DuckDB** attach for heavy scans | Optional accelerator; never authoritative |
| Cache / offset maps / skeleton blobs | **`redb`** or plain files (optional) | Pure-Rust embedded KV; only if SQLite proves slow |

Everything lives under `~/.local/share/qai/` — one directory, no services, `qai serve` works offline.

## A.2 Recommended server profile (Phase 11/12, opt-in)

| Layer | Server backend |
|---|---|
| Relational | **PostgreSQL** |
| Full-text | Tantivy (still embedded per node) or **OpenSearch** |
| Vector | **Qdrant** (or `pgvector` if you want fewer services) |
| Graph | **Postgres recursive CTE** → **Apache AGE** → **Neo4j** only if graph queries become the bottleneck |

## A.3 The "open hand" — the abstraction contract

Make these four traits the *only* way anything touches storage. This is what actually keeps your hand open:

```rust
pub trait Database      { /* relational, tx, UnitOfWork */ }   // sqlite | postgres
pub trait FullTextIndex { /* index, search, delete, generation */ } // tantivy | fts5 | opensearch
pub trait VectorStore   { /* upsert, query, delete, dims */ }  // sqlite-vec | lance | qdrant | pgvector
pub trait GraphStore    { /* nodes, edges, neighbors, paths, bounded pattern */ } // sqlite-cte | age | kuzu | neo4j
```

Rules that make polyglot persistence safe (this is the part most projects get wrong):

1. **SQLite is the source of truth.** FTS, vector, and graph stores are *derived, rebuildable projections*. `qai index rebuild --all` must restore them from the relational store + source versions.
2. **Every derived store records `corpus_generation` + `rule_set_version` + `dataset_version`** in an index manifest. `qai doctor` compares them and reports drift (PRD §50's `✗ Morphology index differs from source version` is exactly this).
3. **No cross-store transactions.** Write relational first, then enqueue an idempotent reindex job. Reconciliation job detects orphans/tombstones (PRD §76).
4. **Canonical lookup never touches FTS/vector/graph** (PRD §40). Enforced by an architecture test.
5. **Deletion is tombstone + verified propagation**, never a raw delete in one store.

## A.4 Honest trade-offs

| Choice | Give up | Get |
|---|---|---|
| SQLite spine | Single-writer throughput; no network access | Zero config, ACID, trivial backup (`qai db backup`), triggers enforce immutability |
| Tantivy | A separate directory to manage; custom tokenizer work | Real BM25 + phrase + proximity over your normalized Arabic fields, offline |
| `sqlite-vec` default | Brute-force-ish at large N (fine to ~10⁵–10⁶ vectors) | Literally zero extra config; upgrade path to LanceDB/Qdrant is a trait swap |
| Relational graph | No Cypher; you write bounded CTEs | No new dependency; provenance columns per edge are natural; meets PRD §32.4 |

**Candidates to consciously reject for the default:** Neo4j/Kuzu (extra runtime or C++ dep for MVP), Milvus/Weaviate (server-only), sled (unmaintained-ish), embedded Postgres (defeats "light").

## A.5 ADRs to write

- **ADR-0001** Relational store: SQLite + `sqlx`, Postgres-portable SQL *(Phase 0)*
- **ADR-0201** Full-text engine: Tantivy + custom Arabic tokenizer *(Phase 2)*
- **ADR-0202** Graph store abstraction: relational adjacency + bounded CTE first *(Phase 3)*
- **ADR-0701** Vector store: `sqlite-vec` default, LanceDB/Qdrant adapters *(Phase 7)*
- **ADR-0702** Cross-store consistency, generation stamping, reconciliation *(Phase 7)*

