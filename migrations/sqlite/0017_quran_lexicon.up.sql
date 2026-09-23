-- Phase 2 (M4a, P2-T57): Quran lexicon tables (D2.6).
-- Forward-only: no .down.sql (Phase-1 convention).
--
-- Multi-analysis discipline (I11, ADR-0209) is enforced by ABSENCE: no
-- `is_correct` / `is_primary` / `selected` column exists on any table here
-- (schema-tested). Competing analyses coexist with dataset attribution;
-- cross-dataset root links are review-queue suggestions, never merges
-- (ADR-0210). Every linguistic row carries Layer B/D provenance (never
-- Layer A); Layer D rows carry algorithm/version/confidence and cannot
-- reach `human_verified` without a reviewer (CHECK-enforced, AC-P2-21).
-- Canonical tables are untouched (derived data only; MV-018 guards builds).
-- Morpheme segments ride `quran_token_analyses.segments_json`; the
-- standalone `quran_morphemes` table carries prefix/stem/suffix rows for
-- graph projection (Phase-4 TASK-405 consumes both).

CREATE TABLE quran_datasets (
  id                TEXT PRIMARY KEY,
  slug              TEXT NOT NULL,
  version           TEXT NOT NULL,
  title             TEXT NOT NULL DEFAULT '',
  license_status    TEXT NOT NULL,
  license_json      TEXT NOT NULL DEFAULT '{}',
  attribution       TEXT NOT NULL,
  root_convention   TEXT NOT NULL DEFAULT '',
  tagset_version    TEXT NOT NULL DEFAULT '',
  state             TEXT NOT NULL CHECK (state IN ('staged','active','superseded')),
  created_at        TEXT NOT NULL,
  UNIQUE (slug, version)
);

CREATE TABLE quran_roots (
  id                TEXT PRIMARY KEY,
  dataset_id        TEXT NOT NULL REFERENCES quran_datasets(id),
  root              TEXT NOT NULL,
  root_normalized   TEXT NOT NULL,
  provenance_layer  TEXT NOT NULL CHECK (provenance_layer IN ('B','D')),
  algorithm         TEXT,
  algorithm_version TEXT,
  confidence        REAL,
  reviewer          TEXT,
  status            TEXT NOT NULL CHECK (status IN ('imported','human_verified'))
                    CHECK (status != 'human_verified' OR (reviewer IS NOT NULL AND reviewer != '')),
  corpus_generation INTEGER NOT NULL DEFAULT 0,
  created_at        TEXT NOT NULL,
  UNIQUE (dataset_id, root),
  CHECK (provenance_layer != 'D' OR (algorithm IS NOT NULL AND algorithm_version IS NOT NULL AND confidence IS NOT NULL))
);

CREATE TABLE quran_lemmas (
  id                TEXT PRIMARY KEY,
  dataset_id        TEXT NOT NULL REFERENCES quran_datasets(id),
  lemma             TEXT NOT NULL,
  root_id           TEXT REFERENCES quran_roots(id),
  pos_unified       TEXT NOT NULL DEFAULT '',
  pos_native        TEXT NOT NULL DEFAULT '',
  provenance_layer  TEXT NOT NULL CHECK (provenance_layer IN ('B','D')),
  algorithm         TEXT,
  algorithm_version TEXT,
  confidence        REAL,
  reviewer          TEXT,
  status            TEXT NOT NULL CHECK (status IN ('imported','human_verified'))
                    CHECK (status != 'human_verified' OR (reviewer IS NOT NULL AND reviewer != '')),
  corpus_generation INTEGER NOT NULL DEFAULT 0,
  created_at        TEXT NOT NULL,
  UNIQUE (dataset_id, lemma),
  CHECK (provenance_layer != 'D' OR (algorithm IS NOT NULL AND algorithm_version IS NOT NULL AND confidence IS NOT NULL))
);

CREATE TABLE quran_token_analyses (
  id                TEXT PRIMARY KEY,
  dataset_id        TEXT NOT NULL REFERENCES quran_datasets(id),
  edition_id        TEXT NOT NULL,
  surah             INTEGER NOT NULL,
  ayah              INTEGER NOT NULL,
  token_position    INTEGER NOT NULL,
  analysis_index    INTEGER NOT NULL DEFAULT 0,
  surface           TEXT NOT NULL,
  lemma_id          TEXT REFERENCES quran_lemmas(id),
  root_id           TEXT REFERENCES quran_roots(id),
  stem              TEXT NOT NULL DEFAULT '',
  pos_unified       TEXT NOT NULL DEFAULT '',
  pos_native        TEXT NOT NULL DEFAULT '',
  features_json     TEXT NOT NULL DEFAULT '{}',
  segments_json     TEXT NOT NULL DEFAULT '[]',
  provenance_layer  TEXT NOT NULL CHECK (provenance_layer IN ('B','D')),
  algorithm         TEXT,
  algorithm_version TEXT,
  confidence        REAL,
  reviewer          TEXT,
  status            TEXT NOT NULL CHECK (status IN ('imported','human_verified'))
                    CHECK (status != 'human_verified' OR (reviewer IS NOT NULL AND reviewer != '')),
  corpus_generation INTEGER NOT NULL DEFAULT 0,
  created_at        TEXT NOT NULL,
  UNIQUE (dataset_id, edition_id, surah, ayah, token_position, analysis_index),
  CHECK (provenance_layer != 'D' OR (algorithm IS NOT NULL AND algorithm_version IS NOT NULL AND confidence IS NOT NULL))
);

CREATE TABLE quran_morphemes (
  id                TEXT PRIMARY KEY,
  analysis_id       TEXT NOT NULL REFERENCES quran_token_analyses(id),
  kind              TEXT NOT NULL CHECK (kind IN ('prefix','stem','suffix')),
  surface           TEXT NOT NULL,
  features_json     TEXT NOT NULL DEFAULT '{}',
  created_at        TEXT NOT NULL
);

-- Word-family relations (D2.8, ADR-0210/I13): typed, explained, attributed.
-- `explanation` is mandatory (T86: no member without a human-readable
-- string); computational suggestions stay `proposed` until the review queue
-- promotes them with reviewer + timestamp + evidence (AC-P2-27).
CREATE TABLE word_family_relations (
  id                TEXT PRIMARY KEY,
  relation          TEXT NOT NULL CHECK (relation IN (
                        'same_form','same_lemma','same_stem','same_root',
                        'derived','inflectional','affix','computational_suggestion')),
  from_kind         TEXT NOT NULL CHECK (from_kind IN ('root','lemma','token')),
  from_id           TEXT NOT NULL,
  to_kind           TEXT NOT NULL CHECK (to_kind IN ('root','lemma','token')),
  to_id             TEXT NOT NULL,
  explanation       TEXT NOT NULL CHECK (length(explanation) > 0),
  dataset_id        TEXT REFERENCES quran_datasets(id),
  provenance_layer  TEXT NOT NULL CHECK (provenance_layer IN ('B','D')),
  algorithm         TEXT,
  algorithm_version TEXT,
  confidence        REAL,
  reviewer          TEXT,
  status            TEXT NOT NULL CHECK (status IN ('proposed','scholar_verified'))
                    CHECK (status != 'scholar_verified' OR (reviewer IS NOT NULL AND reviewer != '')),
  evidence_json     TEXT NOT NULL DEFAULT '{}',
  corpus_generation INTEGER NOT NULL DEFAULT 0,
  created_at        TEXT NOT NULL,
  UNIQUE (relation, from_kind, from_id, to_kind, to_id),
  CHECK (relation != 'computational_suggestion' OR provenance_layer = 'D')
);

-- Morphology review queue (D2.8): suggestion → ScholarVerified with
-- evidence (T88). Named `morphology_review_queue` to avoid colliding with
-- the provenance `review_queue` (migration 0003), which tracks generic
-- human-review workflow; this table carries morphology suggestion payloads.
CREATE TABLE morphology_review_queue (
  id                TEXT PRIMARY KEY,
  kind              TEXT NOT NULL CHECK (kind IN ('root_unification','family_relation','analysis_correction')),
  subject_json      TEXT NOT NULL,
  evidence_json     TEXT NOT NULL,
  status            TEXT NOT NULL CHECK (status IN ('pending','approved','rejected')),
  reviewer          TEXT,
  decided_at        TEXT,
  created_at        TEXT NOT NULL,
  CHECK (status = 'pending' OR (reviewer IS NOT NULL AND reviewer != '' AND decided_at IS NOT NULL))
);

CREATE INDEX ix_roots_dataset ON quran_roots(dataset_id);
CREATE INDEX ix_lemmas_dataset ON quran_lemmas(dataset_id);
CREATE INDEX ix_analyses_token ON quran_token_analyses(dataset_id, edition_id, surah, ayah, token_position);
CREATE INDEX ix_analyses_root ON quran_token_analyses(root_id);
CREATE INDEX ix_analyses_lemma ON quran_token_analyses(lemma_id);
CREATE INDEX ix_morphemes_analysis ON quran_morphemes(analysis_id);
CREATE INDEX ix_family_from ON word_family_relations(from_kind, from_id);
CREATE INDEX ix_family_to ON word_family_relations(to_kind, to_id);
CREATE INDEX ix_morph_review_status ON morphology_review_queue(status);
