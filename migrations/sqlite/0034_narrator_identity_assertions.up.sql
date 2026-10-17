-- Phase 7 (plan 01): Narrator identity assertions.
-- Forward-only: no .down.sql (Phase-1 convention).
-- Disputed stays effective (never tombstoned); rejected/superseded retained for audit.

CREATE TABLE narrator_identity_assertions (
  id                  TEXT PRIMARY KEY,
  assertion_kind      TEXT NOT NULL CHECK (assertion_kind IN ('narrator_identity')),
  claim_json          TEXT NOT NULL,
  evidence_json       TEXT NOT NULL,
  source_location     TEXT NOT NULL DEFAULT '',
  reviewer            TEXT,
  decision            TEXT NOT NULL CHECK (decision IN ('pending','accepted','rejected','superseded','disputed')),
  decided_at          TEXT,
  supersedes_id       TEXT REFERENCES narrator_identity_assertions(id),
  provenance_layer    TEXT NOT NULL CHECK (provenance_layer IN ('B','C','D')),
  algorithm           TEXT,
  algorithm_version   TEXT,
  confidence          REAL,
  created_at          TEXT NOT NULL,
  CHECK (decision = 'pending' OR
         (reviewer IS NOT NULL AND reviewer != '' AND decided_at IS NOT NULL)),
  CHECK (provenance_layer != 'D' OR
         (algorithm IS NOT NULL AND algorithm_version IS NOT NULL AND confidence IS NOT NULL))
);

CREATE INDEX ix_nia_decision ON narrator_identity_assertions(decision);
