-- Phase 4 (plan 04-01, tracer): Quran graph schema fixes (D-05/D-07).
-- Forward-only: never edit 0019.
--
-- 1. Multi-assertion adjacency (RESEARCH Pitfall 1, AC-P4-05): one
--    (src, edge, dst) triple may carry several attributed assertions, so the
--    stable edge identity includes `assertion_id` (model.rs). The triple
--    UNIQUE becomes two NULL-safe partial unique indexes: structural rows
--    (`assertion_id` NULL) stay triple-unique; asserted rows key on the full
--    (projection, src, edge, dst, assertion) quintuple.
-- 2. Disputed review state (RESEARCH Pitfall 3, D-07): the `decision` CHECK
--    gains 'disputed' (effective for traversal/export, never tombstoned).
-- 3. Authority scoping (RESEARCH Pitfall 4, AC-P4-03): assertion authority no
--    longer hangs solely off the build-row foreign key. `projection_id`,
--    `edition_id`, and `dataset_scope` scope columns are backfilled from the
--    parent build row (`dataset_scope ''` marks legacy/unscoped rows); the
--    build-row FK is retained for staging linkage.
-- 4. Structural provenance round-trip: `graph_edges` gains `attrs_json` so
--    structural input-version provenance survives the projection (refs-only
--    payloads; never canonical text).

PRAGMA foreign_keys = OFF;

-- --- graph_edges rebuild: partial multi-assertion key + attrs_json ---
CREATE TABLE graph_edges_new (
  id                TEXT PRIMARY KEY,
  projection_row_id TEXT NOT NULL REFERENCES graph_projections(id),
  src_stable_id     TEXT NOT NULL,
  edge              TEXT NOT NULL,
  dst_stable_id     TEXT NOT NULL,
  assertion_id      TEXT REFERENCES graph_assertions(id),
  budgets_json      TEXT NOT NULL DEFAULT '{}',
  attrs_json        TEXT NOT NULL DEFAULT '{}',
  created_at        TEXT NOT NULL
);

CREATE UNIQUE INDEX ux_gedges_structural ON graph_edges_new(
  projection_row_id, src_stable_id, edge, dst_stable_id
) WHERE assertion_id IS NULL;

CREATE UNIQUE INDEX ux_gedges_asserted ON graph_edges_new(
  projection_row_id, src_stable_id, edge, dst_stable_id, assertion_id
) WHERE assertion_id IS NOT NULL;

INSERT INTO graph_edges_new (
  id, projection_row_id, src_stable_id, edge, dst_stable_id,
  assertion_id, budgets_json, attrs_json, created_at
)
SELECT
  id, projection_row_id, src_stable_id, edge, dst_stable_id,
  assertion_id, budgets_json, '{}', created_at
FROM graph_edges;

DROP TABLE graph_edges;

ALTER TABLE graph_edges_new RENAME TO graph_edges;

CREATE INDEX ix_gedges_proj ON graph_edges(projection_row_id);
CREATE INDEX ix_gedges_src ON graph_edges(projection_row_id, src_stable_id);
CREATE INDEX ix_gedges_dst ON graph_edges(projection_row_id, dst_stable_id);
CREATE INDEX ix_gedges_edge ON graph_edges(projection_row_id, edge);

-- --- graph_assertions rebuild: disputed state + authority scope columns ---
CREATE TABLE graph_assertions_new (
  id                TEXT PRIMARY KEY,
  projection_row_id TEXT NOT NULL REFERENCES graph_projections(id),
  projection_id     TEXT NOT NULL DEFAULT '',
  edition_id        TEXT NOT NULL DEFAULT '',
  dataset_scope     TEXT NOT NULL DEFAULT '',
  assertion_kind    TEXT NOT NULL CHECK (assertion_kind IN (
                      'annotation','concept_link','entity_link','family_link','import')),
  claim_json        TEXT NOT NULL,
  evidence_json     TEXT NOT NULL,
  source_location   TEXT NOT NULL DEFAULT '',
  reviewer          TEXT,
  decision          TEXT NOT NULL CHECK (decision IN (
                      'pending','accepted','rejected','superseded','disputed')),
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

-- LEFT JOIN preserves every authority row even if its build row is gone;
-- unscoped rows keep '' markers rather than invented scope.
INSERT INTO graph_assertions_new (
  id, projection_row_id, projection_id, edition_id, dataset_scope,
  assertion_kind, claim_json, evidence_json, source_location,
  reviewer, decision, decided_at, supersedes_id,
  provenance_layer, algorithm, algorithm_version, confidence, created_at
)
SELECT
  a.id, a.projection_row_id,
  COALESCE(p.projection_id, ''), COALESCE(p.edition_id, ''), '',
  a.assertion_kind, a.claim_json, a.evidence_json, a.source_location,
  a.reviewer, a.decision, a.decided_at, a.supersedes_id,
  a.provenance_layer, a.algorithm, a.algorithm_version, a.confidence, a.created_at
FROM graph_assertions a LEFT JOIN graph_projections p ON p.id = a.projection_row_id;

DROP TABLE graph_assertions;

ALTER TABLE graph_assertions_new RENAME TO graph_assertions;

CREATE INDEX ix_gassert_proj ON graph_assertions(projection_row_id, decision);
CREATE INDEX ix_gassert_scope ON graph_assertions(projection_id, edition_id, dataset_scope);

PRAGMA foreign_keys = ON;
