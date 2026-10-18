# Phase 8 (Multi-RAG Platform) — Deferrals

| ID | Deferred scope | Rationale | Resumes in |
|----|----------------|-----------|------------|
| D-08-01 | PG/Qdrant vector adapters | SQLite BLOB vectors chosen as portable interim; external adapters need Phase 12 infra | Phase 12 |
| D-08-02 | Remote embedding providers (OpenAI etc.) | Local deterministic default only; remote needs Phase 11/12 provider wiring | Phase 11/12 |
| D-08-03 | Multi-tenant RAG | Not in scope; single-user local-first default | Later |
| D-08-04 | Re-activation restoring embeddings | Requires re-import to regenerate embeddings | After D-08-01 |
| D-08-05 | Projection selector on path/subgraph/pattern/inspect | Phase 4 deferral, unchanged | Phase 4 follow-up |
