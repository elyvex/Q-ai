-- Phase 1 (plan 0012): structural divisions (juz/hizb/rub/manzil/ruku/page/sajdah).
-- Forward-only: no .down.sql.

CREATE TABLE quran_divisions (
  edition_id     TEXT NOT NULL REFERENCES quran_editions(id),
  kind           TEXT NOT NULL CHECK (kind IN ('juz','hizb','rub','manzil','ruku','page','sajdah')),
  number         INTEGER NOT NULL,
  start_surah    INTEGER NOT NULL,
  start_ayah     INTEGER NOT NULL,
  end_surah      INTEGER NOT NULL,
  end_ayah       INTEGER NOT NULL,
  start_global   INTEGER NOT NULL,
  end_global     INTEGER NOT NULL,
  label          TEXT,
  provenance_id  TEXT NOT NULL REFERENCES provenance_records(id),
  PRIMARY KEY (edition_id, kind, number),
  CHECK (end_global >= start_global)
);

CREATE INDEX ix_div_range ON quran_divisions(edition_id, kind, start_global, end_global);
