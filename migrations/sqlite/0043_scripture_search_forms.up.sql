-- Phase 9 (plan 09-02): scripture search forms + FTS5.
-- Forward-only: no .down.sql.
CREATE TABLE scripture_search_forms (
  verse_id          TEXT NOT NULL REFERENCES scripture_verses(id) ON DELETE CASCADE,
  form_type         TEXT NOT NULL CHECK (form_type IN ('exact','normalized','skeleton')),
  form_text         TEXT NOT NULL,
  profile_id        TEXT NOT NULL,
  profile_version   TEXT NOT NULL,
  corpus_generation INTEGER NOT NULL,
  PRIMARY KEY (verse_id, form_type, profile_id, profile_version)
);

CREATE INDEX ix_scripture_search_forms_verse ON scripture_search_forms(verse_id);
CREATE INDEX ix_scripture_search_forms_profile ON scripture_search_forms(profile_id, profile_version);

CREATE VIRTUAL TABLE scripture_fts USING fts5(
  verse_id UNINDEXED,
  form_text
);
