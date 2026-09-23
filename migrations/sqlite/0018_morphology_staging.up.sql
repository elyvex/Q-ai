-- Phase 2 (M4a, P2-T57): morphology import staging (D2.6).
-- Forward-only: no .down.sql (Phase-1 convention).
--
-- The importer (P2-T66) holds NO approval capability and cannot activate:
-- it writes staging rows + alignment + per-rule findings here, reaching
-- `Staged` with zero `Fatal` findings. Activation (P2-T67) is a separate
-- approval-gated command that promotes staged rows into `0017` tables and
-- enqueues an FTS rebuild — never mutating lexicon fields in place.
-- Alignment never modifies `quran_tokens` (AC-P2-16): `DirectKey` requires a
-- 100% key match; anything else needs a hashed `AlignmentTable` row here
-- with per-surah unmatched reporting gating approval.

CREATE TABLE morphology_staging_batches (
  id                TEXT PRIMARY KEY,
  dataset_slug      TEXT NOT NULL,
  dataset_version   TEXT NOT NULL,
  adapter           TEXT NOT NULL,
  source_manifest_hash TEXT NOT NULL DEFAULT '',
  state             TEXT NOT NULL CHECK (state IN (
                      'reading','aligning','validating','building','staged',
                      'failed','cancelled','activated')),
  checkpoint        TEXT NOT NULL DEFAULT 'reading',
  created_at        TEXT NOT NULL,
  finished_at       TEXT
);

CREATE TABLE morphology_staging_rows (
  id                TEXT PRIMARY KEY,
  batch_id          TEXT NOT NULL REFERENCES morphology_staging_batches(id),
  reference         TEXT NOT NULL,
  surah             INTEGER NOT NULL,
  ayah              INTEGER NOT NULL,
  token_position    INTEGER NOT NULL,
  payload_json      TEXT NOT NULL,
  native_tags_json  TEXT NOT NULL DEFAULT '{}',
  created_at        TEXT NOT NULL,
  UNIQUE (batch_id, reference)
);

CREATE TABLE morphology_alignment (
  id                TEXT PRIMARY KEY,
  batch_id          TEXT NOT NULL REFERENCES morphology_staging_batches(id),
  direct_key        TEXT NOT NULL,
  edition_id        TEXT NOT NULL,
  surah             INTEGER NOT NULL,
  ayah              INTEGER NOT NULL,
  token_position    INTEGER NOT NULL,
  alignment_kind    TEXT NOT NULL CHECK (alignment_kind IN ('direct_key','table_mapped','unmatched')),
  alignment_hash    TEXT NOT NULL DEFAULT '',
  created_at        TEXT NOT NULL,
  UNIQUE (batch_id, direct_key)
);

CREATE TABLE morphology_findings (
  id                TEXT PRIMARY KEY,
  batch_id          TEXT NOT NULL REFERENCES morphology_staging_batches(id),
  rule_id           TEXT NOT NULL,
  severity          TEXT NOT NULL CHECK (severity IN ('fatal','error','warn')),
  reference         TEXT NOT NULL DEFAULT '',
  detail            TEXT NOT NULL,
  created_at        TEXT NOT NULL
);

CREATE INDEX ix_staging_batch ON morphology_staging_rows(batch_id);
CREATE INDEX ix_alignment_batch ON morphology_alignment(batch_id);
CREATE INDEX ix_alignment_unmatched ON morphology_alignment(batch_id, alignment_kind);
CREATE INDEX ix_findings_batch ON morphology_findings(batch_id, severity);
