-- Phase 1 (plan 0014): staging mirrors for import staging.
--
-- Each `quran_stg_*` table mirrors its canonical counterpart with an extra
-- `import_run_id` column, NO immutability triggers, and ON DELETE CASCADE from
-- `quran_import_runs`, so cancellation cleanup is one statement. Nothing but
-- validation and the activation transaction reads staging (I5).
-- Forward-only: no .down.sql.

CREATE TABLE quran_import_runs (
  run_id          TEXT PRIMARY KEY,
  job_id          TEXT REFERENCES jobs(id) ON DELETE SET NULL,
  edition_slug    TEXT NOT NULL,
  edition_version TEXT NOT NULL,
  adapter         TEXT NOT NULL,
  state           TEXT NOT NULL CHECK (state IN ('Running','Staged','Cancelled','Failed')),
  created_at      TEXT NOT NULL
);

CREATE TABLE quran_stg_editions (
  import_run_id          TEXT NOT NULL REFERENCES quran_import_runs(run_id) ON DELETE CASCADE,
  id                     TEXT NOT NULL,
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
  basmala_policy         TEXT NOT NULL,
  unicode_normalization  TEXT NOT NULL,
  license_json           TEXT NOT NULL,
  text_hash              TEXT NOT NULL,
  structure_hash         TEXT NOT NULL,
  token_order_hash       TEXT NOT NULL,
  manifest_hash          TEXT NOT NULL,
  source_version_id      TEXT NOT NULL,
  statistics_json        TEXT NOT NULL,
  status                 TEXT NOT NULL,
  imported_at            TEXT NOT NULL,
  PRIMARY KEY (import_run_id, id)
);

CREATE TABLE quran_stg_surahs (
  import_run_id           TEXT NOT NULL REFERENCES quran_import_runs(run_id) ON DELETE CASCADE,
  edition_id              TEXT NOT NULL,
  number                  INTEGER NOT NULL,
  name_arabic             TEXT NOT NULL,
  name_transliteration    TEXT,
  name_translations_json  TEXT NOT NULL DEFAULT '{}',
  ayah_count              INTEGER NOT NULL,
  revelation_place        TEXT,
  revelation_order        INTEGER,
  basmala                 TEXT NOT NULL,
  ruku_count              INTEGER,
  metadata_provenance_id  TEXT,
  PRIMARY KEY (import_run_id, edition_id, number)
);

CREATE TABLE quran_stg_ayahs (
  import_run_id     TEXT NOT NULL REFERENCES quran_import_runs(run_id) ON DELETE CASCADE,
  edition_id        TEXT NOT NULL,
  surah             INTEGER NOT NULL,
  ayah              INTEGER NOT NULL,
  text              TEXT NOT NULL,
  text_hash         TEXT NOT NULL,
  char_count        INTEGER NOT NULL,
  token_count       INTEGER NOT NULL,
  global_ayah_index INTEGER NOT NULL,
  juz               INTEGER, hizb INTEGER, rub INTEGER, manzil INTEGER,
  ruku              INTEGER, page INTEGER,
  sajdah            TEXT,
  provenance_id     TEXT NOT NULL,
  PRIMARY KEY (import_run_id, edition_id, surah, ayah)
);

CREATE TABLE quran_stg_tokens (
  import_run_id      TEXT NOT NULL REFERENCES quran_import_runs(run_id) ON DELETE CASCADE,
  edition_id         TEXT NOT NULL,
  surah              INTEGER NOT NULL,
  ayah               INTEGER NOT NULL,
  position           INTEGER NOT NULL,
  surface            TEXT NOT NULL,
  surface_hash       TEXT NOT NULL,
  char_start         INTEGER NOT NULL,
  char_end           INTEGER NOT NULL,
  byte_start         INTEGER NOT NULL,
  byte_end           INTEGER NOT NULL,
  is_pause_mark      INTEGER NOT NULL DEFAULT 0,
  global_token_index INTEGER NOT NULL,
  PRIMARY KEY (import_run_id, edition_id, surah, ayah, position)
);

CREATE TABLE quran_stg_token_separators (
  import_run_id  TEXT NOT NULL REFERENCES quran_import_runs(run_id) ON DELETE CASCADE,
  edition_id     TEXT NOT NULL,
  surah          INTEGER NOT NULL,
  ayah           INTEGER NOT NULL,
  after_position INTEGER NOT NULL,
  separator      TEXT NOT NULL,
  PRIMARY KEY (import_run_id, edition_id, surah, ayah, after_position)
);

CREATE TABLE quran_stg_segments (
  import_run_id  TEXT NOT NULL REFERENCES quran_import_runs(run_id) ON DELETE CASCADE,
  edition_id     TEXT NOT NULL,
  surah          INTEGER NOT NULL,
  ayah           INTEGER NOT NULL,
  seg_index      INTEGER NOT NULL,
  kind           TEXT NOT NULL,
  token_start    INTEGER NOT NULL,
  token_end      INTEGER NOT NULL,
  provenance_id  TEXT NOT NULL,
  PRIMARY KEY (import_run_id, edition_id, surah, ayah, seg_index)
);

CREATE TABLE quran_stg_divisions (
  import_run_id  TEXT NOT NULL REFERENCES quran_import_runs(run_id) ON DELETE CASCADE,
  edition_id     TEXT NOT NULL,
  kind           TEXT NOT NULL,
  number         INTEGER NOT NULL,
  start_surah    INTEGER NOT NULL,
  start_ayah     INTEGER NOT NULL,
  end_surah      INTEGER NOT NULL,
  end_ayah       INTEGER NOT NULL,
  start_global   INTEGER NOT NULL,
  end_global     INTEGER NOT NULL,
  label          TEXT,
  provenance_id  TEXT NOT NULL,
  PRIMARY KEY (import_run_id, edition_id, kind, number)
);
