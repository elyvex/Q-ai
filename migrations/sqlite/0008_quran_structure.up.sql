-- Phase 1 (plan 0011): canonical structure — surahs, ayahs, tokens,
-- separators, segments. Insert-only via triggers (I1).
-- Forward-only: no .down.sql. Physical removal is an audited offline
-- maintenance command, never normal operation.

CREATE TABLE quran_surahs (
  edition_id            TEXT NOT NULL REFERENCES quran_editions(id) ON DELETE RESTRICT,
  number                INTEGER NOT NULL CHECK (number BETWEEN 1 AND 200),
  name_arabic           TEXT NOT NULL,
  name_transliteration  TEXT,
  name_translations_json TEXT NOT NULL DEFAULT '{}',
  ayah_count            INTEGER NOT NULL CHECK (ayah_count > 0),
  revelation_place      TEXT CHECK (revelation_place IN ('makki','madani')),
  revelation_order      INTEGER,
  basmala               TEXT NOT NULL,
  ruku_count            INTEGER,
  metadata_provenance_id TEXT REFERENCES provenance_records(id),
  PRIMARY KEY (edition_id, number)
);

CREATE TABLE quran_ayahs (
  edition_id        TEXT NOT NULL,
  surah             INTEGER NOT NULL,
  ayah              INTEGER NOT NULL CHECK (ayah > 0),
  text              TEXT NOT NULL CHECK (length(trim(text)) > 0),
  text_hash         TEXT NOT NULL,
  char_count        INTEGER NOT NULL,
  token_count       INTEGER NOT NULL,
  global_ayah_index INTEGER NOT NULL,
  juz               INTEGER, hizb INTEGER, rub INTEGER, manzil INTEGER,
  ruku              INTEGER, page INTEGER,
  sajdah            TEXT CHECK (sajdah IS NULL OR sajdah IN ('recommended','obligatory')),
  provenance_id     TEXT NOT NULL REFERENCES provenance_records(id),
  PRIMARY KEY (edition_id, surah, ayah),
  FOREIGN KEY (edition_id, surah) REFERENCES quran_surahs(edition_id, number),
  UNIQUE (edition_id, global_ayah_index)
);

CREATE INDEX ix_ayah_global ON quran_ayahs(edition_id, global_ayah_index);
CREATE INDEX ix_ayah_juz    ON quran_ayahs(edition_id, juz, global_ayah_index);
CREATE INDEX ix_ayah_page   ON quran_ayahs(edition_id, page, global_ayah_index);

CREATE TABLE quran_tokens (
  edition_id         TEXT NOT NULL,
  surah              INTEGER NOT NULL,
  ayah               INTEGER NOT NULL,
  position           INTEGER NOT NULL CHECK (position > 0),
  surface            TEXT NOT NULL,
  surface_hash       TEXT NOT NULL,
  char_start         INTEGER NOT NULL,
  char_end           INTEGER NOT NULL,
  byte_start         INTEGER NOT NULL,
  byte_end           INTEGER NOT NULL,
  is_pause_mark      INTEGER NOT NULL DEFAULT 0 CHECK (is_pause_mark IN (0,1)),
  global_token_index INTEGER NOT NULL,
  PRIMARY KEY (edition_id, surah, ayah, position),
  FOREIGN KEY (edition_id, surah, ayah) REFERENCES quran_ayahs(edition_id, surah, ayah),
  UNIQUE (edition_id, global_token_index),
  CHECK (byte_end > byte_start AND char_end > char_start)
);

CREATE INDEX ix_token_global  ON quran_tokens(edition_id, global_token_index);
CREATE INDEX ix_token_surface ON quran_tokens(edition_id, surface);

-- Lossless reconstruction: exact separator between token n and n+1 (and leading/trailing).
CREATE TABLE quran_token_separators (
  edition_id TEXT NOT NULL,
  surah      INTEGER NOT NULL,
  ayah       INTEGER NOT NULL,
  after_position INTEGER NOT NULL,
  separator  TEXT NOT NULL,
  PRIMARY KEY (edition_id, surah, ayah, after_position)
);

CREATE TABLE quran_segments (
  edition_id    TEXT NOT NULL,
  surah         INTEGER NOT NULL,
  ayah          INTEGER NOT NULL,
  seg_index     INTEGER NOT NULL,
  kind          TEXT NOT NULL,
  token_start   INTEGER NOT NULL,
  token_end     INTEGER NOT NULL,
  provenance_id TEXT NOT NULL REFERENCES provenance_records(id),
  PRIMARY KEY (edition_id, surah, ayah, seg_index),
  CHECK (token_end >= token_start)
);

-- I1: canonical rows are insert-only.
CREATE TRIGGER trg_ayah_no_update BEFORE UPDATE ON quran_ayahs
BEGIN SELECT RAISE(ABORT, 'QAI-QUR-0001: canonical ayah text is immutable'); END;
CREATE TRIGGER trg_ayah_no_delete BEFORE DELETE ON quran_ayahs
BEGIN SELECT RAISE(ABORT, 'QAI-QUR-0003: canonical ayah rows cannot be deleted'); END;
CREATE TRIGGER trg_token_no_update BEFORE UPDATE ON quran_tokens
BEGIN SELECT RAISE(ABORT, 'QAI-QUR-0004: canonical tokens are immutable'); END;
CREATE TRIGGER trg_token_no_delete BEFORE DELETE ON quran_tokens
BEGIN SELECT RAISE(ABORT, 'QAI-QUR-0005: canonical tokens cannot be deleted'); END;
