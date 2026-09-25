-- Phase 2 (plan 02-04): complete the canonical insert-only write fence.
--
-- Append-only and additive (ADR-0002): migrations 0007-0020 are never edited.
-- Before this file only quran_ayahs (UPDATE/DELETE), quran_tokens
-- (UPDATE/DELETE), and the edition identity/hash columns carried triggers.
-- This migration fences every remaining canonical table, the edition DELETE
-- verb, and the edition identity/default columns added in 0020.
--
-- Codes are sequential from QAI-QUR-0006; existing QAI-QUR-0001..0005 are
-- neither reused nor renumbered (ADR-0010 stable taxonomy).
--
-- Translation rows are published artifacts and are insert-only here: a future
-- status transition would require a new forward migration, never an edit of
-- this one. quran_segments has no writer today and is fenced empty; see
-- docs/07-technical/quran-canonical-core-decisions.md for the scope decision.

-- QAI-QUR-0006/0007 — canonical surahs.
CREATE TRIGGER trg_surah_no_update BEFORE UPDATE ON quran_surahs
BEGIN SELECT RAISE(ABORT, 'QAI-QUR-0006: canonical surahs are immutable'); END;
CREATE TRIGGER trg_surah_no_delete BEFORE DELETE ON quran_surahs
BEGIN SELECT RAISE(ABORT, 'QAI-QUR-0007: canonical surah rows cannot be deleted'); END;

-- QAI-QUR-0008/0009 — canonical token separators.
CREATE TRIGGER trg_separator_no_update BEFORE UPDATE ON quran_token_separators
BEGIN SELECT RAISE(ABORT, 'QAI-QUR-0008: canonical token separators are immutable'); END;
CREATE TRIGGER trg_separator_no_delete BEFORE DELETE ON quran_token_separators
BEGIN SELECT RAISE(ABORT, 'QAI-QUR-0009: canonical token separator rows cannot be deleted'); END;

-- QAI-QUR-0010/0011 — canonical segments (reserved; no writer in Phase 2).
CREATE TRIGGER trg_segment_no_update BEFORE UPDATE ON quran_segments
BEGIN SELECT RAISE(ABORT, 'QAI-QUR-0010: canonical segments are immutable'); END;
CREATE TRIGGER trg_segment_no_delete BEFORE DELETE ON quran_segments
BEGIN SELECT RAISE(ABORT, 'QAI-QUR-0011: canonical segment rows cannot be deleted'); END;

-- QAI-QUR-0012/0013 — canonical divisions.
CREATE TRIGGER trg_division_no_update BEFORE UPDATE ON quran_divisions
BEGIN SELECT RAISE(ABORT, 'QAI-QUR-0012: canonical divisions are immutable'); END;
CREATE TRIGGER trg_division_no_delete BEFORE DELETE ON quran_divisions
BEGIN SELECT RAISE(ABORT, 'QAI-QUR-0013: canonical division rows cannot be deleted'); END;

-- QAI-QUR-0014/0015 — translation editions (published artifacts, insert-only).
CREATE TRIGGER trg_translation_edition_no_update BEFORE UPDATE ON translation_editions
BEGIN SELECT RAISE(ABORT, 'QAI-QUR-0014: translation editions are immutable once published'); END;
CREATE TRIGGER trg_translation_edition_no_delete BEFORE DELETE ON translation_editions
BEGIN SELECT RAISE(ABORT, 'QAI-QUR-0015: translation edition rows cannot be deleted'); END;

-- QAI-QUR-0016/0017 — translation passages (published artifacts, insert-only).
CREATE TRIGGER trg_translation_passage_no_update BEFORE UPDATE ON translation_passages
BEGIN SELECT RAISE(ABORT, 'QAI-QUR-0016: translation passages are immutable once published'); END;
CREATE TRIGGER trg_translation_passage_no_delete BEFORE DELETE ON translation_passages
BEGIN SELECT RAISE(ABORT, 'QAI-QUR-0017: translation passage rows cannot be deleted'); END;

-- QAI-QUR-0018/0019 — canonical word glosses.
CREATE TRIGGER trg_gloss_no_update BEFORE UPDATE ON word_glosses
BEGIN SELECT RAISE(ABORT, 'QAI-QUR-0018: canonical word glosses are immutable'); END;
CREATE TRIGGER trg_gloss_no_delete BEFORE DELETE ON word_glosses
BEGIN SELECT RAISE(ABORT, 'QAI-QUR-0019: canonical word gloss rows cannot be deleted'); END;

-- QAI-QUR-0020 — edition DELETE (only UPDATE was covered by 0007).
CREATE TRIGGER trg_edition_no_delete BEFORE DELETE ON quran_editions
BEGIN SELECT RAISE(ABORT, 'QAI-QUR-0020: canonical edition rows cannot be deleted'); END;

-- QAI-QUR-0021 — the identity/default columns added in 0020 must be immutable
-- once canonical. This is a second trigger (not a DROP/recreate of
-- trg_edition_immutable_hashes), so the 0007 trigger stays byte-identical.
CREATE TRIGGER trg_edition_identity_no_update
BEFORE UPDATE OF upstream_edition_slug, qai_edition_id, is_primary
ON quran_editions
BEGIN
  SELECT RAISE(ABORT, 'QAI-QUR-0021: edition upstream identity and primary designation are immutable');
END;
