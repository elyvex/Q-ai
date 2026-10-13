-- Phase 6 (plan 02): Tafsir records.
-- Forward-only: no .down.sql (Phase-1 convention).
-- Each record carries its work, author (via the work row), and dimensions
-- (volume, page, chapter). `text` is canonical-only (T-06-05/T-06-06) — never
-- mutated by search index rebuilds; derived search forms live in
-- `tafsir_search_forms` (migration 0029).

CREATE TABLE tafsir_records (
  id                     TEXT PRIMARY KEY,
  work_id                TEXT NOT NULL REFERENCES tafsir_works(id),
  volume                 INTEGER,
  page                   INTEGER,
  chapter                TEXT,
  text                   TEXT NOT NULL,
  text_hash              TEXT NOT NULL,
  source_version_id      TEXT NOT NULL REFERENCES source_versions(id),
  status                 TEXT NOT NULL CHECK (status IN
                             ('Staged','Approved','Active','Deprecated','Quarantined')),
  created_at             TEXT NOT NULL
);

CREATE INDEX ix_tafsir_records_work_dims ON tafsir_records(work_id, volume, page);
CREATE INDEX ix_tafsir_records_status ON tafsir_records(status);
