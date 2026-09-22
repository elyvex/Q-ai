# Context (topic-keyed notes; no DOC-type documents in this ingest set)

All entries below are drawn from the single PRD unless otherwise noted. No
standalone DOC files were classified, so this file records cross-cutting
topics observed across the PRD and ADR set for downstream consumers.

## Topic: implementation phase map
- source: docs/01-requirements/requirements.md (PRD §43, §88)
- note: Authoritative implementation order is Foundations and provenance → Canonical Quran corpus → Quran search/linguistics → Quran graph → Rich Quran experience → Hadith/tafsir → Isnad/narrator research → Multi-RAG → Comparative scripture → Agentic research → Tool creation/workflows → Server/management → Production hardening (Phases 0–12).

## Topic: open decisions awaiting ADRs or sign-off
- source: docs/01-requirements/requirements.md (PRD §48, §91, §25.15); docs/02-architecture/decisions/ADR-0101-initial-quran-dataset.md, ADR-0114-reference-corpus-comparison.md, ADR-0201-full-text-engine.md, ADR-0202-graph-store.md, ADR-0203-quran-morphology-dataset.md, ADR-0301-rag-strategy.md, ADR-0701-vector-store.md, ADR-0702-cross-store-consistency.md
- note: Still open: initial Quran dataset/license (ADR-0101 Draft, pending human sign-off), morphology dataset (ADR-0203 Draft), reference-comparison typing (ADR-0114 Draft), full-text engine (ADR-0201 Proposed), graph store (ADR-0202 Proposed), RAG topology/embeddings/chunking/reranking (ADR-0301 undecided), vector store (ADR-0701 Proposed), cross-store consistency coordination (ADR-0702 Proposed, foundational subset required from Phases 0–3). Decided by locked ADRs: primary database (ADR-0001, SQLite), job queue (ADR-0003, DB-backed), manifest signing (ADR-0007, ed25519), citation verification (ADR-0111, verify_quotation verdicts).

## Topic: non-goals boundary
- source: docs/01-requirements/requirements.md (PRD §4, §25.14)
- note: Initial releases will not build a foundation model from scratch, declare one sectarian interpretation universally authoritative, or implement every vector database natively.

## Topic: living-PRD and terminology
- source: docs/01-requirements/requirements.md (PRD §94, §96, §95, §97)
- note: The PRD declares itself the authoritative product specification with update rules (§94); terminology fixes meanings for canonical text, edition, Quran tool, model, RAG, graph, agent, agency, tool, tool creation, workflow, research claim, and computational suggestion (§96); change history is recorded in §95/§97.

## Topic: project status and diagnostics
- source: docs/01-requirements/requirements.md (PRD §49, §50)
- note: PRD §49 holds per-area status checklists (foundations, generic RAG, Quran, hadith/tafsir, RAG/agents, sources); PRD §50 requires a `qai doctor` diagnostic command for corpus diagnostics.
