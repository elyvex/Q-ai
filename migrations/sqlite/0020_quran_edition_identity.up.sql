-- Phase 2 (plan 0020): edition upstream identity, internal id, and the
-- explicit primary/default designation.
--
-- Append-only and additive (ADR-0002): migrations 0007-0019 are never edited.
-- There is no in-tree ALTER TABLE precedent, so this uses plain
-- `ALTER TABLE ... ADD COLUMN` (SQLite supports it); nullable/defaulted
-- additive columns cannot rewrite existing canonical bytes. No value is
-- invented: an undeclared manifest leaves these NULL / 0.
--
-- This file makes no trigger claim. The immutability trigger for these
-- columns lands in plan 02-04's append-only migration; until then only the
-- activation `INSERT ... SELECT` writes them and no repository mutator exists.

ALTER TABLE quran_editions ADD COLUMN upstream_edition_slug TEXT;
ALTER TABLE quran_editions ADD COLUMN qai_edition_id TEXT;
ALTER TABLE quran_editions ADD COLUMN is_primary INTEGER NOT NULL DEFAULT 0;

ALTER TABLE quran_stg_editions ADD COLUMN upstream_edition_slug TEXT;
ALTER TABLE quran_stg_editions ADD COLUMN qai_edition_id TEXT;
ALTER TABLE quran_stg_editions ADD COLUMN is_primary INTEGER NOT NULL DEFAULT 0;
