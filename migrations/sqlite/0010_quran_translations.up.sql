-- Phase 1 (plan 0013): attributed translations (verse-level, footnotes,
-- optional word glosses). Principle 5 is structural: `translator` is NOT NULL
-- and non-empty by CHECK, so a translation can never be stored unattributed.
-- Forward-only: no .down.sql.

CREATE TABLE translation_editions (
  id                 TEXT PRIMARY KEY,
  slug               TEXT NOT NULL,
  version            TEXT NOT NULL,
  name               TEXT NOT NULL,
  translator         TEXT NOT NULL CHECK (length(trim(translator)) > 0),
  language           TEXT NOT NULL,
  aligned_edition_id TEXT NOT NULL REFERENCES quran_editions(id),
  numbering_scheme   TEXT NOT NULL,
  license_json       TEXT NOT NULL,
  trust_level        TEXT NOT NULL,
  source_version_id  TEXT NOT NULL REFERENCES source_versions(id),
  text_hash          TEXT NOT NULL,
  status             TEXT NOT NULL,
  imported_at        TEXT NOT NULL,
  UNIQUE (slug, version)
);

CREATE TABLE translation_passages (
  translation_edition_id TEXT NOT NULL REFERENCES translation_editions(id) ON DELETE RESTRICT,
  surah                  INTEGER NOT NULL,
  ayah                   INTEGER NOT NULL,
  text                   TEXT NOT NULL,
  footnotes_json         TEXT NOT NULL DEFAULT '[]',
  provenance_id          TEXT NOT NULL REFERENCES provenance_records(id),
  PRIMARY KEY (translation_edition_id, surah, ayah)
);

CREATE TABLE word_glosses (
  gloss_dataset_id TEXT NOT NULL REFERENCES sources(id),
  edition_id       TEXT NOT NULL,
  surah            INTEGER NOT NULL,
  ayah             INTEGER NOT NULL,
  position         INTEGER NOT NULL,
  language         TEXT NOT NULL,
  gloss            TEXT NOT NULL,
  provenance_id    TEXT NOT NULL REFERENCES provenance_records(id),
  PRIMARY KEY (gloss_dataset_id, edition_id, surah, ayah, position, language)
);
