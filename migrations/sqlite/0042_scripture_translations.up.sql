-- Phase 9 (plan 09-02): scripture translations.
-- Forward-only: no .down.sql.
CREATE TABLE scripture_translations (
  id          TEXT PRIMARY KEY,
  verse_id    TEXT NOT NULL REFERENCES scripture_verses(id) ON DELETE CASCADE,
  language    TEXT NOT NULL,
  translator  TEXT,
  text        TEXT NOT NULL,
  text_hash   TEXT NOT NULL,
  status      TEXT CHECK (status IN ('Staged','Approved','Active','Deprecated','Quarantined')),
  created_at  TEXT NOT NULL
);

CREATE INDEX ix_scripture_translations_verse ON scripture_translations(verse_id, language);
