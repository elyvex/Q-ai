CREATE TABLE audit_events (
  id                TEXT PRIMARY KEY,
  sequence          INTEGER NOT NULL UNIQUE,
  occurred_at       TEXT NOT NULL,
  actor_kind        TEXT NOT NULL CHECK (actor_kind IN ('principal','system','job','agent')),
  actor_id          TEXT,
  action            TEXT NOT NULL,
  subject_urn       TEXT NOT NULL,
  outcome           TEXT NOT NULL CHECK (outcome IN ('allowed','denied','failed')),
  reason            TEXT,
  before_json       TEXT,
  after_json        TEXT,
  request_id        TEXT,
  prev_chain_hash   TEXT NOT NULL,
  chain_hash        TEXT NOT NULL
);

CREATE INDEX ix_audit_time    ON audit_events(occurred_at);
CREATE INDEX ix_audit_action  ON audit_events(action, occurred_at);
CREATE INDEX ix_audit_subject ON audit_events(subject_urn);

CREATE TRIGGER trg_audit_no_update BEFORE UPDATE ON audit_events
BEGIN SELECT RAISE(ABORT, 'QAI-AUD-0001: audit log is append-only'); END;

CREATE TRIGGER trg_audit_no_delete BEFORE DELETE ON audit_events
BEGIN SELECT RAISE(ABORT, 'QAI-AUD-0002: audit log is append-only'); END;
