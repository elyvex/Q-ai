-- Phase 6 (plan 03): Verse links with confidence + owner gate.
-- Forward-only: no .down.sql (Phase-1 convention).
-- Computational matches link hadith/tafsir records to Quran locations.
-- Every computational match carries a confidence score and starts with
-- label 'uncertain' (D-07). Only owner-confirmed links are authoritative:
-- the read path filters on label = 'confirmed'. The owner-gate CHECK
-- mirrors the graph_assertions pattern (Phase 4, migration 0019).

CREATE TABLE verse_links (
  id                TEXT PRIMARY KEY,
  source_type       TEXT NOT NULL CHECK (source_type IN ('hadith','tafsir')),
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

CREATE INDEX ix_verse_links_source ON verse_links(source_type, source_id);
CREATE INDEX ix_verse_links_target ON verse_links(target_surah, target_ayah);
