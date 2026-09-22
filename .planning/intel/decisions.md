# Decisions (synthesized from 33 ADRs)

Classification confidence: high for all entries. Status values follow the
classification `locked` flag (locked:true → locked; locked:false → proposed).
Decision statements are taken from each ADR's Decision section.

## ADR-0000: Project Architecture — layered Cargo workspace, local-first single binary
- source: docs/02-architecture/decisions/ADR-0000-project-architecture.md
- status: locked (Accepted)
- decision: Adopt a layered Cargo workspace (domain / contract / implementation / composition-root / interfaces) with an enforced dependency graph and a local-first single `qai` binary; canonical immutability is a type-level property (CanonicalWriter requires ApprovalToken from a persisted human ApprovalRecord).
- scope: Cargo workspace, domain layer, storage abstraction, SQLite, provenance, audit, qai binary

## ADR-0001: Relational Store — SQLite + sqlx, PostgreSQL-portable SQL
- source: docs/02-architecture/decisions/ADR-0001-relational-store.md
- status: locked (Accepted)
- decision: Use SQLite as the default authoritative relational database through sqlx-backed storage adapters; design schemas and repository contracts for PostgreSQL portability and implement PostgreSQL only as a separate adapter when server deployment requires it. Full-text indexes, vector indexes, and materialized graph structures are rebuildable projections, not authorities.
- scope: SQLite, sqlx, PostgreSQL, relational store, canonical text

## ADR-0002: Migration Strategy — append-only checksummed SQL, forward-only canonical
- source: docs/02-architecture/decisions/ADR-0002-migration-strategy.md
- status: locked (Accepted)
- decision: Implement a custom migration runner in `storage-sqlite` using standard `.sql` files with SHA-256 checksums recorded in `_qai_migrations` (abort on any applied-file mismatch); canonical tables are forward-only, down-migrations disabled by default, corrections applied as new forward migrations.
- scope: migration runner, storage-sqlite, SQLite, PostgreSQL, canonical tables

## ADR-0003: Durable Job System — DB-backed leased queue, no external broker
- source: docs/02-architecture/decisions/ADR-0003-durable-job-system.md
- status: locked (Accepted)
- decision: Use a DB-backed leased SQLite queue via the `jobs` crate (lease claiming, idempotency keys, checkpoints, cancellation flags, Queued → Leased → Running → … state machine); no external broker in local mode; jobs changing projection-relevant state must write an outbox row in the same transaction.
- scope: DB-backed leased queue, jobs crate, SQLite, job lifecycle, outbox

## ADR-0004: Configuration & Precedence Model — CLI > Env > File > Defaults
- source: docs/02-architecture/decisions/ADR-0004-config-precedence.md
- status: locked (Accepted)
- decision: Use a layered model with strict precedence CLI > Env > File > Defaults backed by a `config` crate; every field records its ValueOrigin; validation runs at load and on every change; `bind = "0.0.0.0"` with disabled TLS hard-fails unless auth-outside-localhost is satisfied.
- scope: configuration, precedence model, config crate, ValueOrigin, qai config

## ADR-0005: Secret Storage per OS — env / keychain / encrypted file
- source: docs/02-architecture/decisions/ADR-0005-secret-storage.md
- status: locked (Accepted)
- decision: Support three secret backends selected at runtime (`env` default, OS `keychain`, age/XChaCha20-Poly1305 `encrypted_file`); config/SQLite store only `SecretRef`s, never secret values; `Secret<T>` redacts on Debug/Display/Serialize and zeroizes on drop; a global redaction layer covers logs, spans, audit records, and CLI output.
- scope: secret storage, API keys, env backend, OS keychain, encrypted file, SecretRef

## ADR-0006: Hashing & Canonical Serialization — SHA-256, canonical JSON, NFC
- source: docs/02-architecture/decisions/ADR-0006-hashing-canonical.md
- status: locked (Accepted)
- decision: Adopt SHA-256 as the primary hash algorithm wrapped in an explicit `ContentHash` type tagged with the algorithm; all canonical JSON uses sorted keys, minimal whitespace, UTF-8 without BOM, LF newlines, and NFC normalization of text strings prior to serialization.
- scope: content hashing, SHA-256, canonical JSON, Unicode NFC, ContentHash

## ADR-0007: Source Manifest Format & Signing — ed25519 detached over canonical JSON
- source: docs/02-architecture/decisions/ADR-0007-manifest-format-signing.md
- status: locked (Accepted)
- decision: Use ed25519 detached signatures computed over the canonical JSON bytes (ADR-0006) of the manifest with the signature block removed; `signature = null` allowed only when `security.allow_unsigned_manifests = true`, and unsigned manifests can never become Active.
- scope: source manifest, ed25519 signing, canonical JSON, signature verification

## ADR-0008: Provenance Representation — single universal record + typed attribution
- source: docs/02-architecture/decisions/ADR-0008-provenance-representation.md
- status: locked (Accepted)
- decision: Use a single universal `provenance_records` table with a closed `layer` enum (canonical_source, publisher_metadata, scholarly_annotation, computational_annotation, user_or_ai_note), closed attribution kinds with typed JSON, and type-level plus DB CHECK invariants (computational ⇒ confidence present; scholarly ⇒ named scholar; canonical ⇒ source_version present and insert-only).
- scope: provenance_records, attribution, layer enum, DerivationVersions, ApprovalToken

## ADR-0009: Audit Integrity — hash chain + append-only triggers
- source: docs/02-architecture/decisions/ADR-0009-audit-integrity.md
- status: locked (Accepted)
- decision: Implement an append-only `audit_events` table enforced by BEFORE UPDATE/DELETE triggers (QAI-AUD-0001/0002) plus a per-row SHA-256 hash chain over canonical JSON; gapless sequence; audit writer runs all values through the global redaction layer; `qai audit verify` recomputes the chain.
- scope: audit_events, hash chain, append-only triggers, audit log

## ADR-0010: Error Taxonomy & CLI Exit Codes
- source: docs/02-architecture/decisions/ADR-0010-error-taxonomy.md
- status: locked (Accepted)
- decision: Use namespaced semantic error codes (QAI-CFG, QAI-SEC, QAI-DB, QAI-JOB, QAI-SRC, QAI-PROV, QAI-AUD, QAI-CLI, QAI-QUR, QAI-NORM, QAI-IDX) carried in `Diagnostic.code` with summary, cause chain, remedy, and retryability, plus a separate shell-axis CLI exit-code table (0..70); every emitted code is unique and registered (unit-tested).
- scope: error codes, CLI exit codes, Diagnostic, error taxonomy

## ADR-0011: Observability Stack — tracing + metrics + optional OTLP (opt-in)
- source: docs/02-architecture/decisions/ADR-0011-observability.md
- status: locked (Accepted)
- decision: Deploy `tracing` + `tracing-subscriber` plus the `metrics` crate with a named metric catalog and fixed span conventions; the OTLP exporter sits behind `telemetry.enabled = false` (opt-in) with a compile-time-tested field denylist (query text, prompt text, document content, research questions, model responses) whose violations fail the build.
- scope: observability, tracing, metrics, OTLP, telemetry

## ADR-0012: Workspace/Crate Boundaries & Dependency Enforcement
- source: docs/02-architecture/decisions/ADR-0012-workspace-boundaries.md
- status: locked (Accepted)
- decision: Enforce the crate layer graph via a custom `xtask arch-check` command comparing actual Cargo.toml dependencies against a declarative allowlist (`xtask/allowlist.toml`); any forbidden edge emits a structured Diagnostic and fails CI immediately.
- scope: Cargo workspace, crate boundaries, dependency allowlist, xtask arch-check, layer graph

## ADR-0101: Quran Edition Model — primary default, multiple readings, licensing
- source: docs/02-architecture/decisions/ADR-0101-initial-quran-dataset.md
- status: proposed (Draft — pending human sign-off, do not mark Accepted)
- decision: The multi-edition, multi-riwayah, multilingual model is the accepted direction and is implemented incrementally (quran-core::catalog); the concrete dataset choice is still pending — this ADR remains Draft until a human records the chosen option, dataset identity, licence evidence, and editorial reviewer. Implementing the model does not make this ADR Accepted.
- scope: Quran edition model, Uthmani script, Hafs an Asim, qiraat, translations, licensing, upstream sources

## ADR-0102: Quran Addressing Scheme & Reference Grammar
- source: docs/02-architecture/decisions/ADR-0102-addressing-and-reference-grammar.md
- status: locked (Accepted)
- decision: Freeze the Quran reference grammar (optional `quran:` prefix, `slug[@semver]` edition, ayah/range/token/division locators) implemented as a hand-written allocation-light parser/serializer in `quran-core::reference`; `parse(serialize(r)) == r`; `canonical_form` emits the pinned storage form `quran:slug@version:locator`; malformed input returns coded errors QAI-QUR-0100…0112 and never panics.
- scope: Quran reference grammar, addressing scheme, quran-core::reference, canonical form

## ADR-0103: Verse-Numbering Scheme Handling & Alternate Numbering
- source: docs/02-architecture/decisions/ADR-0103-verse-numbering-scheme.md
- status: locked (Accepted)
- decision: Each edition declares its own `verse_numbering_scheme` (`hafs` | `kufi` | `custom:<name>`); ayah numbers are valid only within that edition and `edition_id` is part of every canonical key; cross-edition alignment is a later-phase mapping layer, never a merged table.
- scope: verse-numbering scheme, editions, ayah numbers, canonical keys, cross-edition alignment

## ADR-0104: Unicode Policy — normalization, blocks, forbidden points, graphemes
- source: docs/02-architecture/decisions/ADR-0104-unicode-policy.md
- status: locked (Accepted)
- decision: Stored form is NFC (declared per edition; QV-007 Fatal on mismatch); forbid C0/C1 controls, BOM, bidi controls, ZWJ/ZWNJ, private-use areas, and noncharacters (QV-008 Fatal); restrict to expected Arabic blocks plus plain space (QV-009 Error); count in grapheme clusters with byte offsets carried alongside for slicing.
- scope: Unicode normalization, NFC, forbidden code points, Arabic blocks, grapheme clusters

## ADR-0105: Canonical Tokenization Rule — whitespace-preserving surface tokens
- source: docs/02-architecture/decisions/ADR-0105-canonical-tokenization.md
- status: locked (Accepted)
- decision: Split per character and map offsets back to grapheme clusters: Unicode whitespace runs are separators recorded exactly; Quranic annotation signs U+06D6…U+06ED form their own tokens except a mark following a word character attaches to that word; positions are dense 1..=k in row order; adapter-supplied tokens are verified against recomputed separators/offsets (QV-010…012), never trusted.
- scope: canonical tokenization, surface tokens, separators, token offsets, quran_corpus

## ADR-0106: Canonical Text Storage Layout — row-per-ayah in SQLite
- source: docs/02-architecture/decisions/ADR-0106-canonical-storage-layout.md
- status: locked (Accepted)
- decision: Store canonical text as row-per-ayah (`quran_ayahs`), row-per-token (`quran_tokens`), exact separators (`quran_token_separators`), and divisions (`quran_divisions`), all keyed by `(edition_id, …)` with covering indexes on global ayah/token order; canonical tables are insert-only via triggers (QAI-QUR-0001…0005); trigger-free staging mirrors cascade from `quran_import_runs`; no `.down.sql` for canonical tables.
- scope: canonical Quran text, SQLite storage layout, quran_ayahs, quran_tokens, insert-only triggers

## ADR-0107: Atomic Activation & Rollback — staging + pointer flip + generation
- source: docs/02-architecture/decisions/ADR-0107-atomic-activation-rollback.md
- status: locked (Accepted)
- decision: Activate editions atomically in one transaction (move staging rows to canonical tables, deprecate the old Active, flip the `quran_active_edition` singleton pointer, bump `corpus_generation`, consume staging); rollback flips the pointer to a prior version and bumps the generation the same way; every cache and derived index keys on `corpus_generation`. The importer holds no ApprovalToken and has no code path to canonical tables.
- scope: staging, activation, rollback, corpus_generation, canonical tables

## ADR-0108: Corpus Hashing Scheme
- source: docs/02-architecture/decisions/ADR-0108-corpus-hashing-scheme.md
- status: locked (Accepted)
- decision: Use SHA-256 length-prefixed (`len u64 LE || bytes`) domain-separated hashing with tags `qai-text-hash-v1` (per-ayah text in surah/ayah order), a canonical structure string, and per-token global-order hashing; per-ayah/per-token plain SHA-256 digests; storage form `sha256:<hex>` with the algorithm tag carried on `ContentHash`.
- scope: corpus hashing, SHA-256, text_hash, structure_hash, token_order_hash, quran_corpus::hashing, doctor, verify_quotation

## ADR-0109: Edition Difference Algorithm
- source: docs/02-architecture/decisions/ADR-0109-edition-difference-algorithm.md
- status: locked (Accepted)
- decision: Align ayahs by `(surah, ayah)` identity; classify each as added / removed / changed / unchanged; report new-text character ranges for changed ayahs; fold edition-level metadata into a `metadata_changed` boolean; persist as `difference_reports` at every import against the current active edition (or all-added on first import).
- scope: edition difference algorithm, ayah alignment, difference_reports, quran_corpus::differ

## ADR-0110: Basmala Representation Policy
- source: docs/02-architecture/decisions/ADR-0110-basmala-representation.md
- status: locked (Accepted)
- decision: Basmala handling is edition metadata: `BasmalaPolicy::{CountedAsFirstAyah, UnnumberedHeader, Absent, PerSurah}` declared per edition with per-surah values; QV-020 enforces consistency when the policy is edition-wide; text-level consistency is verified against the reference corpus (QV-015).
- scope: basmala, BasmalaPolicy, ayah numbering, edition metadata

## ADR-0111: Citation Identity and Deep-Link Format
- source: docs/02-architecture/decisions/ADR-0111-citation-identity-and-deep-links.md
- status: locked (Accepted; resolver + deep links shipped, P1-T46/T47)
- decision: Canonical reference `quran:{slug}@{version}:{surah}:{ayah}`; deep link `/read/{slug}@{version}/{surah}:{ayah}[?highlight=token:{n}]`; storage URN `qai://quran/{slug}@{version}/{surah}:{ayah}`; every quotation carries edition id + version + hash enforced by the `QuranQuotation` constructor; `verify_quotation` returns ExactMatch | MatchAfterWhitespaceNormalization | MatchAfterDeclaredNormalization{rules} | Mismatch | LocationNotFound | EditionNotFound | AccessDenied, and mismatch is a hard failure on answer paths.
- scope: citation identity, deep links, citation URN, QuranQuotation, verify_quotation

## ADR-0112: Translation Alignment and Attribution Model
- source: docs/02-architecture/decisions/ADR-0112-translation-alignment-and-attribution.md
- status: locked (Accepted; translation import + type guards shipped, P1-T36/T37)
- decision: `translation_editions` requires a non-empty `translator` plus `aligned_edition_id` and `numbering_scheme` (QV-027 enforces alignment); `AyahView.canonical` is `QuranQuotation` (Arabic only) with translations alongside as `AttributedTranslation` (no constructor without translator + edition reference), so there is no type-level path from a translation into the canonical slot; word glosses are a separate attributed dataset aligned by (edition, surah, ayah, position).
- scope: translation alignment, attribution, translation_editions, AyahView, word glosses

## ADR-0113: Canonical Lookup Caching & Invalidation
- source: docs/02-architecture/decisions/ADR-0113-canonical-lookup-caching.md
- status: locked (Accepted; implemented in M8 with the cache-consistency test)
- decision: Use an `lru`-backed cache keyed by `(edition_id, version, corpus_generation, ref, options_hash)` with wholesale invalidation on `corpus_generation` change; correctness over hit rate — a cache-consistency test (activate a new version, assert no stale text) gates the reader.
- scope: canonical lookups, LRU cache, corpus_generation, invalidation

## ADR-0114: Typed Corpus Comparison — integrity vs. edition/readings differences
- source: docs/02-architecture/decisions/ADR-0114-reference-corpus-comparison.md
- status: proposed (Draft — pending human sign-off, do not mark Accepted)
- decision: Every comparison declares operand kinds and a compatibility class (integrity / version / readings / translation / reference / checksum-manifest verification); only compatible representations are compared by default and incompatible comparisons require an explicit opt-in naming the kind, never labeled "corruption"; reported differences carry a closed classification (same, normalization_only, orthographic_difference, script_difference, riwayah_difference, edition_difference, tokenization_difference, …).
- scope: corpus comparison, integrity comparison, edition differences, reference corpus, QV-015

## ADR-0201: Full-Text Engine — Tantivy + custom Arabic tokenizer
- source: docs/02-architecture/decisions/ADR-0201-full-text-engine.md
- status: proposed (Proposed)
- decision: Use Tantivy behind the `FullTextIndex` abstraction with a custom Arabic tokenization adapter consuming Q-ai's versioned normalization and canonical-alignment model; Tantivy is a derived retrieval engine, not a canonical text store and not the owner of Arabic normalization rules; responsibility is split across quran-normalization, quran-morphology, quran-search, and the Tantivy adapter.
- scope: Tantivy, FullTextIndex, Arabic tokenization, normalization, Quran search

## ADR-0202: Graph Store Abstraction — relational adjacency + bounded CTE first
- source: docs/02-architecture/decisions/ADR-0202-graph-store.md
- status: proposed (Proposed)
- decision: Define a backend-neutral `GraphStore` port implemented initially with relational adjacency tables and bounded recursive CTEs, with CozoDB and SQLite graph extensions planned as optional derived projections behind the same port; SQLite remains the single source of truth; no unrestricted backend query language is exposed through the public graph API.
- scope: GraphStore, knowledge graph, relational adjacency, SQLite, Quran Graph

## ADR-0203: Quran Morphology Dataset, Multi-Edition Alignment, and Attribution
- source: docs/02-architecture/decisions/ADR-0203-quran-morphology-dataset.md
- status: proposed (Draft — pending human sign-off, do not mark Accepted)
- decision: Morphology is edition-relative (Quran → Quran Edition → Ayah → Token → 0..n morphological analyses in Layer D); every morphology record carries its `quran_edition_id` and alignment is performed against the exact source edition + version + token order, never against "the Quran" in the abstract; each record carries explicit source and confidence.
- scope: Quran morphology dataset, Quran editions, tokenization, morphological analysis, alignment

## ADR-0301: RAG Strategy
- source: docs/02-architecture/decisions/ADR-0301-rag-strategy.md
- status: proposed (Proposed; Phase 3 — not yet decided)
- decision: absent (no decision taken; Phase 3 must select retrieval topology, embedding model, chunking policy, and reranking strategy). Fixed constraints on any choice: retrieval is a consumer of the outbox (never polls authoritative tables); every result carries `source_version_id` and `corpus_generation`; tombstoned/deactivated subjects are excluded immediately.
- scope: RAG, retrieval, canonical corpus, outbox, indexes

## ADR-0701: Vector Store — sqlite-vec default, LanceDB/Qdrant adapters
- source: docs/02-architecture/decisions/ADR-0701-vector-store.md
- status: proposed (Proposed)
- decision: Use `sqlite-vec` as the default local vector backend with optional LanceDB and Qdrant adapters behind a backend-neutral `VectorStore` port (LanceDB for larger local deployments, Qdrant for server/explicit remote); adapter selection is configuration-driven with no automatic migration to remote; ANN capabilities are discovered from the pinned backend version.
- scope: vector store, sqlite-vec, LanceDB, Qdrant, VectorStore port, semantic retrieval

## ADR-0702: Cross-Store Consistency, Generation Stamping, and Reconciliation
- source: docs/02-architecture/decisions/ADR-0702-cross-store-consistency.md
- status: proposed (Proposed; foundational subset required from Phases 0–3)
- decision: Use one authoritative relational state, a transactional outbox, at-least-once idempotent projection jobs, versioned dependency snapshots and projection manifests, atomic relational publication of complete release manifests, continuous drift detection with explicit repair, and tombstones with verified propagation; no cross-store transactions. Phased application: Phase 0 establishes outbox/revisions/jobs/tombstones; Phase 2 applies generation manifests and safe full-text publication; Phase 3 the same contract for graph projections; Phase 7 coordinates multi-store releases and vector reconciliation.
- scope: cross-store consistency, relational store, projection manifests, transactional outbox, reconciliation
