-- Phase 9 (plan 01): Scripture verses.
-- Forward-only: no .down.sql (Phase-1 convention).

CREATE TABLE scripture_verses (
  id             TEXT PRIMARY KEY,
  edition_id     TEXT NOT NULL REFERENCES scripture_editions(id),
  book           TEXT NOT NULL,
  chapter        INTEGER NOT NULL CHECK (chapter > 0),
  verse          INTEGER NOT NULL CHECK (verse > 0),
  text           TEXT NOT NULL,
  text_hash      TEXT NOT NULL,
  status         TEXT NOT NULL CHECK (status IN
                     ('Staged','Approved','Active','Deprecated','Quarantined')),
  created_at     TEXT NOT NULL,
  UNIQUE (edition_id, book, chapter, verse)
);

CREATE INDEX ix_scripture_verses_edition_book_chapter ON scripture_verses(edition_id, book, chapter);