-- Phase 2 (M2, P2-T25): derived token/ayah forms + skeletons (D2.2).
-- Forward-only: no .down.sql (Phase-1 convention).
--
-- Derived data only: every row references its canonical source row and
-- carries (rule_set_id, rule_set_version, corpus_generation, provenance_id)
-- at Layer D (plan §3.5). Canonical tables are untouched (I8); MV-018
-- re-verification runs in the build job, not in SQL.
--
-- Span maps are NOT stored: they are pure functions of (canonical text,
-- profile) recomputed at query time through the shared pipeline, so stored
-- strings can never disagree with their maps (R6 by construction).
-- ADR-0208 will decide the FTS-side posting representation, not this.

CREATE TABLE quran_token_forms (
  edition_id       TEXT NOT NULL,
  surah            INTEGER NOT NULL,
  ayah             INTEGER NOT NULL,
  position         INTEGER NOT NULL,
  simple           TEXT NOT NULL,  -- L2.marks
  bare             TEXT NOT NULL,  -- L3.diacritics
  hamza_folded     TEXT NOT NULL,  -- L4.hamza
  folded           TEXT NOT NULL,  -- L5.codepoints
  affix_stripped   TEXT NOT NULL,  -- L7.affix (token-level only)
  transliteration  TEXT,           -- reserved (N23, Phase 4); NULL in Phase 2
  phonetic         TEXT,           -- reserved (N24, experimental); NULL in Phase 2
  rule_set_id      TEXT NOT NULL,  -- e.g. 'quran-normalization'
  rule_set_version TEXT NOT NULL,  -- profile ladder version, e.g. '1.0.0'
  corpus_generation INTEGER NOT NULL,
  provenance_id    TEXT NOT NULL REFERENCES provenance_records(id),
  PRIMARY KEY (edition_id, surah, ayah, position),
  FOREIGN KEY (edition_id, surah, ayah, position)
    REFERENCES quran_tokens(edition_id, surah, ayah, position)
);

CREATE INDEX ix_token_forms_bare ON quran_token_forms(edition_id, bare);
CREATE INDEX ix_token_forms_folded ON quran_token_forms(edition_id, folded);

CREATE TABLE quran_ayah_forms (
  edition_id       TEXT NOT NULL,
  surah            INTEGER NOT NULL,
  ayah             INTEGER NOT NULL,
  simple           TEXT NOT NULL,  -- L2.marks
  bare             TEXT NOT NULL,  -- L3.diacritics
  hamza_folded     TEXT NOT NULL,  -- L4.hamza
  folded           TEXT NOT NULL,  -- L5.codepoints
  transliteration  TEXT,           -- reserved (N23, Phase 4); NULL in Phase 2
  rule_set_id      TEXT NOT NULL,
  rule_set_version TEXT NOT NULL,
  corpus_generation INTEGER NOT NULL,
  provenance_id    TEXT NOT NULL REFERENCES provenance_records(id),
  PRIMARY KEY (edition_id, surah, ayah),
  FOREIGN KEY (edition_id, surah, ayah)
    REFERENCES quran_ayahs(edition_id, surah, ayah)
);

CREATE INDEX ix_ayah_forms_bare ON quran_ayah_forms(edition_id, bare);
CREATE INDEX ix_ayah_forms_folded ON quran_ayah_forms(edition_id, folded);

-- Skeleton store: one row per ayah plus sliding 3-ayah windows (stride 1,
-- surah-scoped; windows never cross a surah boundary). Window rows are
-- labeled so cross-ayah matches render as spanning verses, never as one.
CREATE TABLE quran_skeletons (
  edition_id       TEXT NOT NULL,
  surah            INTEGER NOT NULL,
  ayah_start       INTEGER NOT NULL,
  ayah_end         INTEGER NOT NULL CHECK (ayah_end >= ayah_start AND ayah_end - ayah_start <= 2),
  skeleton         TEXT NOT NULL,  -- L6.skeleton
  rule_set_id      TEXT NOT NULL,
  rule_set_version TEXT NOT NULL,
  corpus_generation INTEGER NOT NULL,
  provenance_id    TEXT NOT NULL REFERENCES provenance_records(id),
  PRIMARY KEY (edition_id, surah, ayah_start, ayah_end)
);

CREATE INDEX ix_skeleton_lookup ON quran_skeletons(edition_id, surah, ayah_start);
