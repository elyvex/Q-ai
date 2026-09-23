# Q-ai

## What This Is

Q-ai is a local-first research platform for studying the Quran, Quranic vocabulary/morphology/roots, hadith collections, Islamic literature, and comparative scripture (Torah, Tanakh, New Testament). It combines a canonical Quran engine with structural text storage, a knowledge graph, multi-RAG retrieval, and a controlled agentic tool runtime, exposed through a Web GUI, TUI, CLI, and API.

## Core Value

Trustworthy Quran research: exact canonical text before generated interpretation, and every factual claim traceable to its source.

## Business Context

- **Customer**: Researchers, students, and developers studying Quranic and Islamic texts (local-first, single user by default)
- **Revenue model**: None (research platform, local-first + optional self-hosted server)
- **Success metric**: PRD quality gates and definition-of-done pass per phase
- **Strategy notes**: `docs/01-requirements/requirements.md` is the authoritative product specification (living-PRD rules, §94)

## Target Runtime

Local-first + server (local-first default with optional server deploy).

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Foundations: workspace, config, SQLite, migrations, jobs, provenance, audit, CLI
- [ ] Canonical Quran engine: validated import, addressing, tokens, exact lookup, integrity tests
- [ ] Quran search and linguistics: normalization, roots/lemmas, morphology, frequency tools
- [ ] Quran knowledge graph: schema, traversal, annotations, visualization
- [ ] Rich Quran experience: Web GUI reading/research views, TUI, CLI tree
- [ ] Hadith and tafsir: structured records, grading attribution, verse links
- [ ] Isnad and narrator research: identity review, chain paths, uncertainty
- [ ] Multi-RAG and smart routing: full-text, vector, hybrid, reranking, debugging
- [ ] Comparative scripture: edition-aware model, parallel-passage research
- [ ] Agentic research, tool creation, and workflows: permissions, approvals, sandbox, MCP
- [ ] Server and management: versioned API, streaming, auth, source/agent/tool management
- [ ] Production hardening: PostgreSQL/Qdrant adapters, TLS, Docker, backups, recovery

### Out of Scope

- Building a foundation model from scratch — not a model lab (PRD §4, §25.14)
- Declaring one sectarian interpretation universally authoritative — platform represents disagreement, never resolves it by fiat (PRD §4, §25.14)
- Implementing every vector database natively — backend-neutral ports with selected adapters only (PRD §25.14)
- Unreviewed AI changes to canonical corpora — canonical writes require persisted human approval (ADR-0000, PRD §44.2)
- Arbitrary shell execution / unrestricted generated tools — deny-by-default permissions (PRD §44.2)
- Claims of authoritative religious rulings — no false scholarly-consensus claims (PRD §2, §44.2)

## Context

- Rust workspace (`qai` binary); SQLite default with PostgreSQL-portable contracts; DB-backed job queue, no external broker locally.
- Authoritative build order is PRD §43/§88 Phases 0–12 (foundations → canonical Quran → search/linguistics → graph → rich experience → hadith/tafsir → isnad → multi-RAG → comparative scripture → agentic research → tool creation/workflows → server/management → hardening). Roadmap phases 1–12 track this order; PRD §43 Phase 9 + Phase 10 are delivered together (Phase 10 of roadmap) since safe tool creation is exercised by the same agent runtime and acceptance gates (§90/§93 overlap).
- Open decisions pending ADRs/sign-off: initial Quran dataset + license (ADR-0101 Draft), morphology dataset (ADR-0203 Draft), reference-comparison typing (ADR-0114 Draft), full-text engine (ADR-0201 Proposed), graph store (ADR-0202 Proposed), RAG topology/embeddings/chunking/reranking (ADR-0301 undecided), vector store (ADR-0701 Proposed), cross-store consistency coordination (ADR-0702 Proposed, foundational subset from Phases 0–3). Settled by locked ADRs: primary database (ADR-0001), job queue (ADR-0003), manifest signing (ADR-0007), citation verification (ADR-0111).
- Living PRD with fixed terminology (§94, §96); per-area status checklists (§49); `qai doctor` corpus diagnostics (§50).
- Ingest: 36 classified documents (33 ADRs, 2 SPECs, 1 PRD), 0 blockers, 0 warnings, 8 auto-resolved INFO (see `.planning/INGEST-CONFLICTS.md`).

## Constraints

- **[Schema]**: All content hashes use `sha256:<hex>`; `canonical_json_bytes` is deterministic (sorted keys, minimal whitespace, UTF-8 no BOM, LF, NFC). Hashes from Phase 0 stay valid; changing format/algorithm requires global re-hash migration — `docs/architecture/hashing-spec.md` (ADR-0006).
- **[API contract]**: Quran citation identity, verdicts, resolution algorithm, and CLI/API surface are frozen per the citation spec; quoted canonical text travels only inside `QuranQuotation`; mismatch is a hard failure on answer paths — `docs/07-technical/quran-citation-spec.md` (ADR-0111).
- **[Runtime]**: Local-first default with optional server deploy — local data stays local unless the user enables a remote provider.
- **[Stack]**: Rust; SQLite default (PostgreSQL-portable SQL); `qai` single binary; layered Cargo workspace with enforced dependency graph (`xtask arch-check`).
- **[Security]**: Deny-by-default agent/tool permissions with human approval; secrets via env/keychain/encrypted file, never stored as values; global redaction layer.

## Key Decisions

<decisions>
The following 25 decisions are LOCKED (Accepted ADRs) and constrain all implementation work:

- [LOCKED] ADR-0000 — Layered Cargo workspace, local-first single `qai` binary; canonical immutability is type-level (CanonicalWriter requires ApprovalToken from persisted human ApprovalRecord).
- [LOCKED] ADR-0001 — SQLite default authoritative relational store via sqlx; PostgreSQL-portable contracts; indexes/graph structures are rebuildable projections.
- [LOCKED] ADR-0002 — Custom migration runner, append-only checksummed `.sql` files in `_qai_migrations`; canonical tables forward-only, no down-migrations.
- [LOCKED] ADR-0003 — DB-backed leased SQLite job queue (`jobs` crate); no external broker locally; projection-relevant jobs write outbox rows transactionally.
- [LOCKED] ADR-0004 — Config precedence CLI > Env > File > Defaults (`config` crate, ValueOrigin); `bind 0.0.0.0` without TLS + auth hard-fails.
- [LOCKED] ADR-0005 — Three secret backends (env / OS keychain / age+XChaCha20 encrypted file); `SecretRef`s only in config/DB; `Secret<T>` redacts and zeroizes; global redaction layer.
- [LOCKED] ADR-0006 — SHA-256 `ContentHash`; canonical JSON (sorted keys, minimal whitespace, UTF-8 no BOM, LF, NFC).
- [LOCKED] ADR-0007 — ed25519 detached signatures over canonical manifest bytes; unsigned manifests can never become Active.
- [LOCKED] ADR-0008 — Single universal `provenance_records` table, closed layer enum + typed attribution with DB/type invariants.
- [LOCKED] ADR-0009 — Append-only `audit_events` (triggers QAI-AUD-0001/0002) + per-row SHA-256 hash chain; `qai audit verify`.
- [LOCKED] ADR-0010 — Namespaced error codes (QAI-*) in `Diagnostic.code` + separate CLI exit-code table; every code registered and unit-tested.
- [LOCKED] ADR-0011 — `tracing` + `metrics` with named catalog; OTLP opt-in behind `telemetry.enabled = false`; compile-time-tested telemetry denylist.
- [LOCKED] ADR-0012 — Crate layer graph enforced by `xtask arch-check` against `xtask/allowlist.toml`; violations fail CI.
- [LOCKED] ADR-0102 — Frozen Quran reference grammar (`quran:slug@version:locator`) in `quran-core::reference`; round-trip `parse(serialize(r)) == r`; coded errors QAI-QUR-0100…0112, never panics.
- [LOCKED] ADR-0103 — Per-edition `verse_numbering_scheme`; ayah numbers valid only within edition; cross-edition alignment is a later mapping layer.
- [LOCKED] ADR-0104 — Stored form NFC; forbidden code points (QV-008 Fatal); Arabic-block restriction (QV-009); grapheme-cluster counts with byte offsets.
- [LOCKED] ADR-0105 — Whitespace-preserving surface tokenization; U+06D6…U+06ED annotation signs; dense positions; adapter tokens verified, never trusted.
- [LOCKED] ADR-0106 — Row-per-ayah/token/separator/division SQLite layout keyed by `(edition_id, …)`; insert-only triggers; trigger-free staging mirrors.
- [LOCKED] ADR-0107 — Atomic activation (staging → canonical, pointer flip, `corpus_generation` bump) in one transaction; rollback via pointer flip; importer holds no ApprovalToken.
- [LOCKED] ADR-0108 — SHA-256 length-prefixed domain-separated corpus hashing (`qai-text-hash-v1`); storage form `sha256:<hex>`.
- [LOCKED] ADR-0109 — Edition diff by `(surah, ayah)` identity (added/removed/changed/unchanged + char ranges); persisted as `difference_reports` at every import.
- [LOCKED] ADR-0110 — Basmala as per-edition `BasmalaPolicy` metadata; QV-020 consistency enforcement.
- [LOCKED] ADR-0111 — Citation identity `quran:{slug}@{version}:{surah}:{ayah}`, deep links, URNs; `verify_quotation` verdicts with mismatch as hard failure on answer paths.
- [LOCKED] ADR-0112 — `translation_editions` require translator + aligned edition + numbering scheme; `AyahView.canonical` is Arabic-only `QuranQuotation`; no type-level path from translation into canonical slot.
- [LOCKED] ADR-0113 — `lru` canonical-lookup cache keyed on `(edition, version, corpus_generation, ref, options)`; wholesale invalidation on generation change; cache-consistency test gates the reader.
</decisions>

Proposed (NOT locked — explicit decision points during roadmap execution): ADR-0101 (dataset pending sign-off), ADR-0114, ADR-0201, ADR-0202, ADR-0203, ADR-0301, ADR-0701, ADR-0702.

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Layered workspace + local-first binary (ADR-0000) | Enforceable boundaries; canonical safety at type level | — Pending |
| SQLite default, PG-portable (ADR-0001) | Local-first authority; server path without rewrite | — Pending |
| Canonical Quran safety model (ADR-0102…0113) | Exact text + verifiable citations are the core value | — Pending |
| Deny-by-default tools, human approvals | Untrusted retrieval; no silent canonical modification | — Pending |

---
*Last updated: 2026-09-23 after ingest bootstrap (36 docs, 0 blockers)*
