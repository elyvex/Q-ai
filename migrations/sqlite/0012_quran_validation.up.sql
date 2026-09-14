-- Phase 1 (plan 0015): validation reports, difference reports, citations.
-- Forward-only: no .down.sql.

CREATE TABLE validation_reports (
  id                TEXT PRIMARY KEY,
  subject_urn       TEXT NOT NULL,
  validator         TEXT NOT NULL,
  validator_version TEXT NOT NULL,
  outcome           TEXT NOT NULL CHECK (outcome IN ('pass','pass_with_warnings','fail')),
  fatal_count       INTEGER NOT NULL,
  error_count       INTEGER NOT NULL,
  warning_count     INTEGER NOT NULL,
  findings_json     TEXT NOT NULL,
  created_at        TEXT NOT NULL
);

CREATE TABLE difference_reports (
  id             TEXT PRIMARY KEY,
  subject_urn    TEXT NOT NULL,
  from_version   TEXT NOT NULL,
  to_version     TEXT NOT NULL,
  differ         TEXT NOT NULL,
  differ_version TEXT NOT NULL,
  summary_json   TEXT NOT NULL,
  details_json   TEXT NOT NULL,
  created_at     TEXT NOT NULL
);

CREATE TABLE citations (
  id                    TEXT PRIMARY KEY,
  kind                  TEXT NOT NULL,
  canonical_reference   TEXT NOT NULL,
  source_id             TEXT REFERENCES sources(id),
  source_version_id     TEXT REFERENCES source_versions(id),
  edition_ref           TEXT,
  location_json         TEXT NOT NULL,
  quoted_text_hash      TEXT,
  ingestion_version     TEXT NOT NULL,
  resolved_at           TEXT NOT NULL,
  verdict               TEXT NOT NULL
);

CREATE INDEX ix_citations_ref ON citations(canonical_reference);
