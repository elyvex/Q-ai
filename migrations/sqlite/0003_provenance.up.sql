CREATE TABLE provenance_records (
  id                  TEXT PRIMARY KEY,
  layer               TEXT NOT NULL CHECK (layer IN (
                        'canonical_source','publisher_metadata','scholarly_annotation',
                        'computational_annotation','user_or_ai_note')),
  subject_urn         TEXT NOT NULL,
  attribution_kind    TEXT NOT NULL CHECK (attribution_kind IN ('dataset','scholar','computational','user')),
  attribution_json    TEXT NOT NULL,
  source_version_id   TEXT REFERENCES source_versions(id),
  location_json       TEXT,
  quoted_text_hash    TEXT,
  trust_level         TEXT NOT NULL,
  verification_status TEXT NOT NULL CHECK (verification_status IN
                        ('unverified','needs_review','human_verified','rejected','superseded')),
  confidence          REAL CHECK (confidence IS NULL OR (confidence >= 0.0 AND confidence <= 1.0)),
  versions_json       TEXT NOT NULL,
  created_at          TEXT NOT NULL,
  created_by          TEXT REFERENCES principals(id),
  reviewed_by         TEXT REFERENCES principals(id),
  reviewed_at         TEXT,
  review_note         TEXT,
  superseded_by       TEXT REFERENCES provenance_records(id),
  CHECK (layer <> 'computational_annotation'
         OR (attribution_kind = 'computational' AND confidence IS NOT NULL)),
  CHECK (verification_status <> 'human_verified' OR reviewed_by IS NOT NULL),
  CHECK (layer <> 'canonical_source' OR source_version_id IS NOT NULL)
);

CREATE INDEX ix_prov_subject ON provenance_records(subject_urn);
CREATE INDEX ix_prov_layer   ON provenance_records(layer, verification_status);
CREATE INDEX ix_prov_srcver  ON provenance_records(source_version_id);

CREATE TRIGGER trg_prov_canonical_no_update
BEFORE UPDATE ON provenance_records
WHEN OLD.layer = 'canonical_source'
BEGIN
  SELECT RAISE(ABORT, 'QAI-PROV-0001: canonical provenance is immutable');
END;

CREATE TRIGGER trg_prov_canonical_no_delete
BEFORE DELETE ON provenance_records
WHEN OLD.layer = 'canonical_source'
BEGIN
  SELECT RAISE(ABORT, 'QAI-PROV-0002: canonical provenance cannot be deleted');
END;

CREATE TABLE review_queue (
  id             TEXT PRIMARY KEY,
  provenance_id  TEXT NOT NULL REFERENCES provenance_records(id) ON DELETE CASCADE,
  queue          TEXT NOT NULL,
  priority       INTEGER NOT NULL DEFAULT 0,
  evidence_json  TEXT NOT NULL,
  state          TEXT NOT NULL CHECK (state IN ('pending','accepted','rejected','corrected')),
  decided_by     TEXT REFERENCES principals(id),
  decided_at     TEXT,
  decision_note  TEXT,
  created_at     TEXT NOT NULL
);
