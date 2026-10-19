-- Phase 9 (plan 03): extend verse_links source_type to include 'scripture'.
-- Forward-only: no .down.sql (Phase-1 convention).
-- SQLite cannot ALTER a CHECK constraint, so rebuild the table.

CREATE TABLE verse_links_new (
  id                TEXT PRIMARY KEY,
  source_type       TEXT NOT NULL CHECK (source_type IN ('hadith','tafsir','scripture')),
  source_id         TEXT NOT NULL,
  target_surah      INTEGER NOT NULL CHECK (target_surah > 0),
  target_ayah       INTEGER NOT NULL CHECK (target_ayah > 0),
  confidence        REAL NOT NULL CHECK (confidence >= 0.0 AND confidence <= 1.0),
  label             TEXT NOT NULL DEFAULT 'uncertain'
                    CHECK (label IN ('uncertain','confirmed','rejected')),
  algorithm         TEXT NOT NULL,
  algorithm_version TEXT NOT NULL,
  reviewer          TEXT,
  decided_at        TEXT,
  created_at        TEXT NOT NULL,
  CHECK (label != 'confirmed' OR
         (reviewer IS NOT NULL AND reviewer != '' AND decided_at IS NOT NULL))
);

INSERT INTO verse_links_new
  (id, source_type, source_id, target_surah, target_ayah, confidence,
   label, algorithm, algorithm_version, reviewer, decided_at, created_at)
SELECT id, source_type, source_id, target_surah, target_ayah, confidence,
       label, algorithm, algorithm_version, reviewer, decided_at, created_at
FROM verse_links;

DROP TABLE verse_links;

ALTER TABLE verse_links_new RENAME TO verse_links;

CREATE INDEX ix_verse_links_source ON verse_links(source_type, source_id);
CREATE INDEX ix_verse_links_target ON verse_links(target_surah, target_ayah);
