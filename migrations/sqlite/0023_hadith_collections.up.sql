-- Phase 6 (plan 01): Hadith collections.
-- Forward-only: no .down.sql (Phase-1 convention).
-- Follows the quran_editions pattern (Phase 2): source-aware records with
-- immutable hashes, license, and lifecycle status. The `tradition` field is
-- metadata only — no separate code path per CONTEXT.md locked decision.

CREATE TABLE hadith_collections (
  id                     TEXT PRIMARY KEY,
  slug                   TEXT NOT NULL,
  version                TEXT NOT NULL,
  name                   TEXT NOT NULL,
  tradition              TEXT NOT NULL CHECK (tradition IN ('sunni','shia')),
  source_url             TEXT,
  license_json           TEXT NOT NULL,
  text_hash              TEXT NOT NULL,
  structure_hash         TEXT NOT NULL,
  source_version_id      TEXT NOT NULL REFERENCES source_versions(id),
  status                 TEXT NOT NULL CHECK (status IN
                             ('Staged','Approved','Active','Deprecated','Quarantined')),
  imported_at            TEXT NOT NULL,
  UNIQUE (slug, version)
);

CREATE INDEX ix_hadith_collections_status ON hadith_collections(status);
