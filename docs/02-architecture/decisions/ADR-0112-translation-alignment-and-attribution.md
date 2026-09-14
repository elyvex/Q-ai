# ADR-0112 — Translation Alignment and Attribution Model (Draft)

- Status: **Draft** (accepted with translation import, P1-T36/T37)
- Phase: 1 — Canonical Quran Core
- Date: 2026-09-14
- Related decisions: ADR-0103
- Requirements: PRD §7.2, §13.1, principle 5

## Context

Translations must never be presentable as the original (principle 5), and must
align to a specific Arabic edition's numbering (ADR-0103) rather than floating free.

## Decision (proposed)

- `translation_editions` requires a non-empty `translator`
  (`CHECK(length(trim(translator)) > 0)`, already in migration `0010`) plus
  `aligned_edition_id` and `numbering_scheme`; QV-027 enforces alignment.
- `AyahView.canonical` is `QuranQuotation` (Arabic only); translations ride
  alongside as `AttributedTranslation`, which has no constructor without
  translator + edition reference. There is no type-level path from a
  translation into the canonical slot.
- Word glosses are a separate, attributed dataset aligned by
  `(edition, surah, ayah, position)`.

## Consequences

- Type guards in `quran-core` (M9, P1-T37); DB CHECK already covers storage.
- Accepted when translation import lands.
