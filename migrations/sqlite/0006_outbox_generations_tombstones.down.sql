-- Revert 0006_outbox_generations_tombstones.
DROP TABLE IF EXISTS tombstones;
DROP TABLE IF EXISTS outbox_events;
DROP TABLE IF EXISTS corpus_generations;
