-- Revert 0001_core: drop core tables (children before parents).
-- schema_migrations is dropped last; the runner removes its row first.
DROP TABLE IF EXISTS blobs;
DROP TABLE IF EXISTS settings;
DROP TABLE IF EXISTS workspaces;
DROP TABLE IF EXISTS principals;
DROP TABLE IF EXISTS schema_migrations;
