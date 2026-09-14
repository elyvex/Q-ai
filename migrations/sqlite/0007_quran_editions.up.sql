-- Phase 1 (plan 0010): Quran editions + the single active-edition pointer.
-- Forward-only: no .down.sql. Deactivation replaces deletion (§76).

CREATE TABLE quran_editions (
  id                     TEXT PRIMARY KEY,
  slug                   TEXT NOT NULL,
  version                TEXT NOT NULL,
  name                   TEXT NOT NULL,
  script                 TEXT NOT NULL,
  riwayah                TEXT,
  qiraah                 TEXT,
  publisher              TEXT,
  source_url             TEXT,
  language               TEXT NOT NULL,
  verse_numbering_scheme TEXT NOT NULL,
  basmala_policy         TEXT NOT NULL CHECK (basmala_policy IN
                            ('counted_as_first_ayah','unnumbered_header','absent','per_surah')),
  unicode_normalization  TEXT NOT NULL CHECK (unicode_normalization IN ('nfc','nfd','nfkc','nfkd')),
  license_json           TEXT NOT NULL,
  text_hash              TEXT NOT NULL,
  structure_hash         TEXT NOT NULL,
  token_order_hash       TEXT NOT NULL,
  manifest_hash          TEXT NOT NULL,
  source_version_id      TEXT NOT NULL REFERENCES source_versions(id),
  statistics_json        TEXT NOT NULL,
  status                 TEXT NOT NULL CHECK (status IN
                            ('Staged','Approved','Active','Deprecated','Quarantined')),
  imported_at            TEXT NOT NULL,
  verified_at            TEXT,
  verified_by            TEXT,
  verification_method    TEXT,
  activated_at           TEXT,
  deprecated_at          TEXT,
  UNIQUE (slug, version)
);

-- Exactly one active Arabic edition pointer; row id fixed at 1 (§7.3, §85).
CREATE TABLE quran_active_edition (
  singleton          INTEGER PRIMARY KEY CHECK (singleton = 1),
  edition_id         TEXT NOT NULL REFERENCES quran_editions(id),
  corpus_generation  INTEGER NOT NULL,
  activated_at       TEXT NOT NULL,
  activated_by       TEXT NOT NULL REFERENCES principals(id),
  approval_id        TEXT NOT NULL REFERENCES approvals(id)
);

CREATE TRIGGER trg_edition_immutable_hashes
BEFORE UPDATE OF text_hash, structure_hash, token_order_hash, slug, version
ON quran_editions
BEGIN
  SELECT RAISE(ABORT, 'QAI-QUR-0002: edition identity and hashes are immutable');
END;
