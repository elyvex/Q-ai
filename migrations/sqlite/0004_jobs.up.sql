CREATE TABLE jobs (
  id                TEXT PRIMARY KEY,
  kind              TEXT NOT NULL,
  payload_json      TEXT NOT NULL,
  idempotency_key   TEXT,
  state             TEXT NOT NULL CHECK (state IN (
                      'Queued','Leased','Running','Checkpointed','Succeeded',
                      'Failed','Cancelled','Interrupted','DeadLettered')),
  priority          INTEGER NOT NULL DEFAULT 0,
  attempts          INTEGER NOT NULL DEFAULT 0,
  max_attempts      INTEGER NOT NULL DEFAULT 5,
  available_at      TEXT NOT NULL,
  lease_owner       TEXT,
  lease_expires_at  TEXT,
  checkpoint_json   TEXT,
  progress_json     TEXT,
  cancel_requested  INTEGER NOT NULL DEFAULT 0 CHECK (cancel_requested IN (0,1)),
  parent_job_id     TEXT REFERENCES jobs(id),
  versions_json     TEXT NOT NULL DEFAULT '{}',
  error_code        TEXT,
  error_json        TEXT,
  created_at        TEXT NOT NULL,
  started_at        TEXT,
  finished_at       TEXT,
  created_by        TEXT REFERENCES principals(id),
  UNIQUE (kind, idempotency_key)
);

CREATE INDEX ix_jobs_claim ON jobs(state, available_at, priority DESC);
CREATE INDEX ix_jobs_kind  ON jobs(kind, state);

CREATE TABLE job_events (
  id          TEXT PRIMARY KEY,
  job_id      TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
  sequence    INTEGER NOT NULL,
  level       TEXT NOT NULL CHECK (level IN ('trace','debug','info','warn','error')),
  stage       TEXT,
  message     TEXT NOT NULL,
  data_json   TEXT,
  occurred_at TEXT NOT NULL,
  UNIQUE (job_id, sequence)
);
