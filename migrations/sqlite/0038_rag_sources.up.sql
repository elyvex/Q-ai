-- Phase 8 (plan 01): Multi-RAG sources with status.
-- Forward-only: no .down.sql (Phase-1 convention).

CREATE TABLE rag_sources (
  id             TEXT PRIMARY KEY,
  project_id     TEXT NOT NULL REFERENCES rag_projects(id) ON DELETE CASCADE,
  path           TEXT NOT NULL,
  glob           TEXT NOT NULL DEFAULT '**/*',
  status         TEXT NOT NULL CHECK (status IN ('active','deactivated','tombstoned')) DEFAULT 'active',
  tombstoned_at  TEXT,
  audit_seq      INTEGER,
  created_at     TEXT NOT NULL,
  CHECK (status != 'tombstoned' OR tombstoned_at IS NOT NULL)
);

CREATE INDEX ix_rag_sources_project ON rag_sources(project_id);
CREATE INDEX ix_rag_sources_status ON rag_sources(status);
