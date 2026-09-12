CREATE TABLE corpus_generations (
  id            TEXT PRIMARY KEY,
  scope         TEXT NOT NULL,
  number        INTEGER NOT NULL,
  reason        TEXT NOT NULL,
  created_at    TEXT NOT NULL,
  UNIQUE (scope, number)
);
CREATE INDEX ix_corpus_generations_scope ON corpus_generations(scope, number DESC);

CREATE TABLE outbox_events (
  id                  TEXT PRIMARY KEY,
  scope               TEXT NOT NULL,
  target_generation   TEXT NOT NULL REFERENCES corpus_generations(id),
  operation           TEXT NOT NULL,
  subject_urn         TEXT NOT NULL,
  idempotency_key     TEXT NOT NULL,
  payload_json        TEXT NOT NULL,
  state               TEXT NOT NULL CHECK (state IN ('Pending','Claimed','Dispatched','Failed')),
  lease_owner         TEXT,
  lease_expires_at    TEXT,
  attempts            INTEGER NOT NULL DEFAULT 0,
  created_at          TEXT NOT NULL,
  dispatched_at       TEXT,
  UNIQUE (operation, idempotency_key)
);
CREATE INDEX ix_outbox_claim ON outbox_events(state, created_at);

CREATE TABLE tombstones (
  id                 TEXT PRIMARY KEY,
  subject_urn        TEXT NOT NULL,
  reason             TEXT NOT NULL,
  effective_at       TEXT NOT NULL,
  created_by         TEXT REFERENCES principals(id),
  propagation_state  TEXT NOT NULL CHECK (propagation_state IN ('Pending','Propagated','PartiallyFailed')),
  UNIQUE (subject_urn, effective_at)
);
CREATE INDEX ix_tombstones_subject ON tombstones(subject_urn);

CREATE TRIGGER trg_corpus_generations_immutable
BEFORE UPDATE ON corpus_generations
BEGIN SELECT RAISE(ABORT, 'QAI-DB-0010: generations are append-only'); END;

CREATE TRIGGER trg_tombstones_immutable
BEFORE UPDATE ON tombstones
BEGIN SELECT RAISE(ABORT, 'QAI-DB-0011: tombstones are append-only'); END;
