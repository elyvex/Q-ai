-- Phase 9 (plan 01): Scripture editions.
-- Forward-only: no .down.sql (Phase-1 convention).

CREATE TABLE scripture_editions (
  id                     TEXT PRIMARY KEY,
  slug                   TEXT NOT NULL,
  version                TEXT NOT NULL,
  name                   TEXT NOT NULL,
  tradition              TEXT NOT NULL,
  language               TEXT NOT NULL,
  source_url             TEXT,
  license_json           TEXT NOT NULL DEFAULT '{}',
  text_hash              TEXT NOT NULL,
  structure_hash         TEXT NOT NULL,
  manifest_hash          TEXT NOT NULL,
  source_version_id      TEXT NOT NULL REFERENCES source_versions(id),
  status                 TEXT NOT NULL CHECK (status IN
                             ('Staged','Approved','Active','Deprecated','Quarantined')),
  imported_at            TEXT NOT NULL,
  UNIQUE (slug, version)
);

CREATE INDEX ix_scripture_editions_status ON scripture_editions(status);

CREATE TRIGGER trg_scripture_edition_immutable_hashes
BEFORE UPDATE OF text_hash, structure_hash, manifest_hash, slug, version, tradition
ON scripture_editions
BEGIN
  SELECT RAISE(ABORT, 'QAI-SCR-0002: edition identity and hashes are immutable');
END;
