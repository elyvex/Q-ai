-- Phase 6 (plan 02): Tafsir works.
-- Forward-only: no .down.sql (Phase-1 convention).
-- Follows the quran_editions pattern (Phase 2): source-aware records with
-- immutable hashes and lifecycle status. Per D-06 (locked): tafsir is indexed
-- by work/author/dimensions following the existing catalog pattern.
-- `dimensions_schema_json` records how to interpret volume/page/chapter for
-- this work (e.g. which chapter scheme it follows).

CREATE TABLE tafsir_works (
  id                     TEXT PRIMARY KEY,
  slug                   TEXT NOT NULL,
  version                TEXT NOT NULL,
  name                   TEXT NOT NULL,
  author                 TEXT NOT NULL CHECK (length(trim(author)) > 0),
  dimensions_schema_json TEXT NOT NULL DEFAULT '{}',
  source_version_id      TEXT NOT NULL REFERENCES source_versions(id),
  status                 TEXT NOT NULL CHECK (status IN
                             ('Staged','Approved','Active','Deprecated','Quarantined')),
  created_at             TEXT NOT NULL,
  UNIQUE (slug, version)
);

CREATE INDEX ix_tafsir_works_status ON tafsir_works(status);
