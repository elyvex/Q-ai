-- Phase 8 (plan 01): Multi-RAG embeddings (BLOB).
-- Forward-only: no .down.sql (Phase-1 convention).

CREATE TABLE rag_embeddings (
  doc_id      TEXT PRIMARY KEY REFERENCES rag_documents(id) ON DELETE CASCADE,
  project_id  TEXT NOT NULL,
  embedding   BLOB NOT NULL,
  dim         INTEGER NOT NULL,
  created_at  TEXT NOT NULL
);

CREATE INDEX ix_rag_embeddings_project ON rag_embeddings(project_id);
