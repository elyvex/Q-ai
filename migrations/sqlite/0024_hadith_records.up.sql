-- Phase 6 (plan 01): Hadith records.
-- Forward-only: no .down.sql (Phase-1 convention).
-- Matn/isnad separation where the source supports it; single-field where it
-- doesn't (isnad_text NULL). source_number is NOT NULL and preserved verbatim
-- from the source (D-01). matn_text is canonical-only (D-02) — never mutated
-- by search index rebuilds.

CREATE TABLE hadith_records (
  id                     TEXT PRIMARY KEY,
  collection_id          TEXT NOT NULL REFERENCES hadith_collections(id),
  source_number          TEXT NOT NULL CHECK (length(trim(source_number)) > 0),
  matn_text              TEXT NOT NULL,
  matn_hash              TEXT NOT NULL,
  isnad_text             TEXT,
  narrator_names_json    TEXT NOT NULL DEFAULT '[]',
  source_version_id      TEXT NOT NULL REFERENCES source_versions(id),
  status                 TEXT NOT NULL CHECK (status IN
                             ('Staged','Approved','Active','Deprecated','Quarantined')),
  metadata_json          TEXT NOT NULL DEFAULT '{}',
  created_at             TEXT NOT NULL,
  UNIQUE (collection_id, source_number)
);

CREATE INDEX ix_hadith_records_collection ON hadith_records(collection_id);
CREATE INDEX ix_hadith_records_status ON hadith_records(status);
