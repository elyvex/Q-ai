-- Phase 9 (plan 01): Scripture books.
-- Forward-only: no .down.sql (Phase-1 convention).

CREATE TABLE scripture_books (
  id             TEXT PRIMARY KEY,
  edition_id     TEXT NOT NULL REFERENCES scripture_editions(id),
  slug           TEXT NOT NULL,
  name           TEXT NOT NULL,
  book_order     INTEGER NOT NULL,
  chapter_count  INTEGER NOT NULL,
  UNIQUE (edition_id, slug)
);

CREATE INDEX ix_scripture_books_edition ON scripture_books(edition_id);