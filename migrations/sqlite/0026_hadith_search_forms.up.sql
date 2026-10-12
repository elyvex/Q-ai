-- Phase 6 (plan 01): Hadith search forms.
-- Forward-only: no .down.sql (Phase-1 convention).
-- Derived search forms following the quran_forms pattern: canonical text is
-- immutable (D-02); search forms are derived into a separate table with a
-- corpus_generation stamp. FTS5 virtual table for matn text, narrator names,
-- and collection name.

CREATE TABLE hadith_search_forms (
  hadith_id          TEXT NOT NULL REFERENCES hadith_records(id) ON DELETE CASCADE,
  form_type          TEXT NOT NULL CHECK (form_type IN ('exact','normalized','skeleton','affix')),
  form_text          TEXT NOT NULL,
  profile_id         TEXT NOT NULL,
  profile_version    TEXT NOT NULL,
  corpus_generation  INTEGER NOT NULL,
  PRIMARY KEY (hadith_id, form_type, profile_id, profile_version)
);

CREATE INDEX ix_hadith_search_forms_hadith ON hadith_search_forms(hadith_id);
CREATE INDEX ix_hadith_search_forms_profile ON hadith_search_forms(profile_id, profile_version);

-- FTS5 virtual table for hadith text search.
-- Standalone (not external-content): its columns do not mirror
-- hadith_search_forms, so `content=` would not be a valid backing table.
-- `hadith_id` is UNINDEXED — stored for the join, not searchable.
-- `matn_text` holds the L3-normalized form (written by the ingest service),
-- NOT canonical text: the query path normalizes with L3 too, so both sides
-- agree. Canonical matn_text lives untouched in `hadith_records` (D-02).
-- The default unicode61 tokenizer is used deliberately — `remove_diacritics`
-- only folds Latin marks and does not strip Arabic combining marks.
CREATE VIRTUAL TABLE hadith_fts USING fts5(
  hadith_id UNINDEXED,
  matn_text,
  narrator_names,
  collection_name
);
