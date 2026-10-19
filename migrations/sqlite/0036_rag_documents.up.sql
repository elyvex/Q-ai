-- Phase 8 (plan 01): Multi-RAG documents.
-- Forward-only: no .down.sql (Phase-1 convention).
-- Standalone FTS5 table (not external content=) rebuilt manually on import.

CREATE TABLE rag_documents (
  id             TEXT PRIMARY KEY,
  project_id     TEXT NOT NULL REFERENCES rag_projects(id) ON DELETE CASCADE,
  source_id      TEXT NOT NULL,
  path           TEXT NOT NULL,
  title          TEXT,
  text           TEXT NOT NULL,
  metadata_json  TEXT NOT NULL DEFAULT '{}',
  created_at     TEXT NOT NULL
);

CREATE INDEX ix_rag_documents_project ON rag_documents(project_id);
CREATE INDEX ix_rag_documents_source ON rag_documents(source_id);

CREATE VIRTUAL TABLE rag_documents_fts USING fts5(doc_id UNINDEXED, text);
