-- Phase 4 (M1, TASK-416): Quran knowledge-graph store (D4.1).
-- Forward-only: no .down.sql (Phase-1 convention).
--
-- Adjacency + assertion authority are SEPARATE stores: `graph_edges` holds
-- traversable structure with attribution pointers; `graph_assertions` holds
-- the immutable/versioned scholarly records with evidence links, review
-- decisions, and supersession (M4, TASK-421). Interpretive edges always
-- resolve to attribution + evidence; rebuilds preserve annotations and
-- review history. Snapshots pin the dependency set each build read
-- (TASK-415); publication flips one pointer row per projection in a single
-- transaction (TASK-417, ADR-0213 pattern).

CREATE TABLE graph_projections (
  id                TEXT PRIMARY KEY,
  projection_id     TEXT NOT NULL,
  builder_version   TEXT NOT NULL,
  edition_id        TEXT NOT NULL,
  corpus_generation INTEGER NOT NULL DEFAULT 0,
  dataset_versions_json TEXT NOT NULL DEFAULT '{}',
  dependency_snapshot_json TEXT NOT NULL DEFAULT '{}',
  status            TEXT NOT NULL CHECK (status IN ('building','active','superseded')),
  manifest_json     TEXT NOT NULL DEFAULT '{}',
  created_at        TEXT NOT NULL,
  UNIQUE (projection_id, id)
);

CREATE TABLE graph_build_progress (
  build_id          TEXT PRIMARY KEY,
  projection_row_id TEXT NOT NULL REFERENCES graph_projections(id),
  stage             TEXT NOT NULL DEFAULT 'reserve',
  cursor_json       TEXT NOT NULL DEFAULT '{}',
  updated_at        TEXT NOT NULL
);

CREATE TABLE graph_nodes (
  id                TEXT PRIMARY KEY,
  projection_row_id TEXT NOT NULL REFERENCES graph_projections(id),
  node_kind         TEXT NOT NULL CHECK (node_kind IN (
                      'edition','surah','ayah','token','division',
                      'root','lemma','concept','entity','annotation')),
  stable_id         TEXT NOT NULL,
  attrs_json        TEXT NOT NULL DEFAULT '{}',
  created_at        TEXT NOT NULL,
  UNIQUE (projection_row_id, stable_id)
);

CREATE TABLE graph_edges (
  id                TEXT PRIMARY KEY,
  projection_row_id TEXT NOT NULL REFERENCES graph_projections(id),
  src_stable_id     TEXT NOT NULL,
  edge              TEXT NOT NULL,
  dst_stable_id     TEXT NOT NULL,
  assertion_id      TEXT REFERENCES graph_assertions(id),
  budgets_json      TEXT NOT NULL DEFAULT '{}',
  created_at        TEXT NOT NULL,
  UNIQUE (projection_row_id, src_stable_id, edge, dst_stable_id)
);

-- Authoritative assertion store (TASK-421): immutable/versioned records.
-- Structural edges need no assertion (`assertion_id` NULL with input-version
-- provenance in `evidence_json`); every non-structural edge MUST reference
-- an assertion carrying attribution + evidence (M4 exit gate).
CREATE TABLE graph_assertions (
  id                TEXT PRIMARY KEY,
  projection_row_id TEXT NOT NULL REFERENCES graph_projections(id),
  assertion_kind    TEXT NOT NULL CHECK (assertion_kind IN (
                      'annotation','concept_link','entity_link','family_link','import')),
  claim_json        TEXT NOT NULL,
  evidence_json     TEXT NOT NULL,
  source_location   TEXT NOT NULL DEFAULT '',
  reviewer          TEXT,
  decision          TEXT NOT NULL CHECK (decision IN ('pending','accepted','rejected','superseded')),
  decided_at        TEXT,
  supersedes_id     TEXT REFERENCES graph_assertions(id),
  provenance_layer  TEXT NOT NULL CHECK (provenance_layer IN ('B','D')),
  algorithm         TEXT,
  algorithm_version TEXT,
  confidence        REAL,
  created_at        TEXT NOT NULL,
  CHECK (decision = 'pending' OR (reviewer IS NOT NULL AND reviewer != '' AND decided_at IS NOT NULL)),
  CHECK (provenance_layer != 'D' OR (algorithm IS NOT NULL AND algorithm_version IS NOT NULL AND confidence IS NOT NULL))
);

CREATE INDEX ix_gnodes_proj ON graph_nodes(projection_row_id);
CREATE INDEX ix_gnodes_kind ON graph_nodes(projection_row_id, node_kind);
CREATE INDEX ix_gedges_proj ON graph_edges(projection_row_id);
CREATE INDEX ix_gedges_src ON graph_edges(projection_row_id, src_stable_id);
CREATE INDEX ix_gedges_dst ON graph_edges(projection_row_id, dst_stable_id);
CREATE INDEX ix_gedges_edge ON graph_edges(projection_row_id, edge);
CREATE INDEX ix_gassert_proj ON graph_assertions(projection_row_id, decision);
