-- Phase 7 (plan 01): Narrators.
-- Forward-only: no .down.sql (Phase-1 convention).
-- Stable id narrator:{slug} is the primary key, never a rowid.

CREATE TABLE narrators (
  id                 TEXT PRIMARY KEY,
  slug               TEXT NOT NULL UNIQUE,
  primary_name       TEXT NOT NULL,
  name_variants_json TEXT NOT NULL DEFAULT '[]',
  metadata_json      TEXT NOT NULL DEFAULT '{}',
  created_at         TEXT NOT NULL
);

CREATE INDEX ix_narrators_slug ON narrators(slug);
