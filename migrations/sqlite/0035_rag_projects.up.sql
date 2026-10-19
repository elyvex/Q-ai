-- Phase 8 (plan 01): Multi-RAG projects.
-- Forward-only: no .down.sql (Phase-1 convention).

CREATE TABLE rag_projects (
  id                 TEXT PRIMARY KEY,
  name               TEXT NOT NULL UNIQUE,
  config_toml        TEXT NOT NULL,
  embedding_provider  TEXT NOT NULL,
  embedding_dim      INTEGER NOT NULL,
  reranker           TEXT NOT NULL,
  router             TEXT NOT NULL,
  created_at         TEXT NOT NULL,
  updated_at         TEXT NOT NULL
);

CREATE INDEX ix_rag_projects_name ON rag_projects(name);
