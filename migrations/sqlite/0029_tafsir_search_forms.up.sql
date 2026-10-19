-- Phase 6 (plan 02): Tafsir search forms.
-- Forward-only: no .down.sql (Phase-1 convention).
-- Derived search forms following the quran_forms pattern: canonical text is
-- immutable (T-06-05/T-06-06); search forms are derived into a separate table
-- with a corpus_generation stamp. FTS5 virtual table for tafsir content,
-- author, and verse reference (the work's chapter organization).

CREATE TABLE tafsir_search_forms (
  tafsir_id          TEXT NOT NULL REFERENCES tafsir_records(id) ON DELETE CASCADE,
  form_type          TEXT NOT NULL CHECK (form_type IN ('exact','normalized','skeleton','affix')),
  form_text          TEXT NOT NULL,
  profile_id         TEXT NOT NULL,
  profile_version    TEXT NOT NULL,
  corpus_generation  INTEGER NOT NULL,
  PRIMARY KEY (tafsir_id, form_type, profile_id, profile_version)
);

CREATE INDEX ix_tafsir_search_forms_tafsir ON tafsir_search_forms(tafsir_id);
CREATE INDEX ix_tafsir_search_forms_profile ON tafsir_search_forms(profile_id, profile_version);

-- FTS5 virtual table for tafsir text search.
-- Standalone (not external-content): its columns do not mirror
-- tafsir_search_forms, so `content=` would not be a valid backing table.
-- `tafsir_id` is UNINDEXED — stored for the join, not searchable.
-- `content_text` holds the L5-normalized form (written by the ingest
-- service), NOT canonical text: the query path normalizes with L5 too, so
-- both sides agree. Canonical text lives untouched in `tafsir_records`
-- (T-06-05/T-06-06). `verse_reference` holds the record's chapter (the
-- work's own verse organization). The default unicode61 tokenizer is used
-- deliberately — `remove_diacritics` only folds Latin marks and does not
-- strip Arabic combining marks.
CREATE VIRTUAL TABLE tafsir_fts USING fts5(
  tafsir_id UNINDEXED,
  content_text,
  author,
  verse_reference
);
