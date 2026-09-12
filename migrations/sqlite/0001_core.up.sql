PRAGMA foreign_keys = ON;

CREATE TABLE schema_migrations (
  version      INTEGER PRIMARY KEY,
  name         TEXT    NOT NULL,
  checksum     TEXT    NOT NULL,
  applied_at   TEXT    NOT NULL,
  applied_by   TEXT    NOT NULL,
  duration_ms  INTEGER NOT NULL
);

CREATE TABLE principals (
  id           TEXT PRIMARY KEY,
  kind         TEXT NOT NULL CHECK (kind IN ('local_user','service','system')),
  display_name TEXT NOT NULL,
  created_at   TEXT NOT NULL,
  disabled_at  TEXT
);

CREATE TABLE workspaces (
  id          TEXT PRIMARY KEY,
  name        TEXT NOT NULL UNIQUE,
  owner_id    TEXT NOT NULL REFERENCES principals(id),
  created_at  TEXT NOT NULL,
  archived_at TEXT
);

CREATE TABLE settings (
  key         TEXT PRIMARY KEY,
  value_json  TEXT NOT NULL,
  origin      TEXT NOT NULL,
  updated_at  TEXT NOT NULL,
  updated_by  TEXT REFERENCES principals(id)
);

CREATE TABLE blobs (
  id            TEXT PRIMARY KEY,
  hash          TEXT NOT NULL,
  bytes         INTEGER NOT NULL CHECK (bytes >= 0),
  media_type    TEXT,
  relative_path TEXT NOT NULL UNIQUE,
  created_at    TEXT NOT NULL,
  UNIQUE (hash, bytes)
);
