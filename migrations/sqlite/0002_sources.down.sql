-- Revert 0002_sources (children before parents).
DROP TABLE IF EXISTS source_state_transitions;
DROP TABLE IF EXISTS source_genealogy;
DROP TABLE IF EXISTS source_files;
DROP TABLE IF EXISTS source_versions;
DROP TABLE IF EXISTS approvals;
DROP TABLE IF EXISTS sources;
