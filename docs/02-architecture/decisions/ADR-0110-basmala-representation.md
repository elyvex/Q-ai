# ADR-0110 — Basmala Representation Policy

- Status: Accepted
- Phase: 1 — Canonical Quran Core
- Date: 2026-09-14
- Related decisions: ADR-0101, ADR-0103
- Requirements: PRD §7.4, §13.1

## Context

Whether the basmala counts as an ayah differs by surah and edition
(Al-Fatihah 1:1 vs At-Tawbah's absence vs An-Naml 27:30's inline basmala).
Guessing corrupts ayah counts and citations.

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| Edition-metadata-driven policy (chosen) | Never guessed; per-surah overrides possible | Requires the dataset to declare it |
| Fixed rule (always ayah 1 except Tawbah) | Simple | Wrong for editions that differ |

## Decision

Basmala handling is edition metadata: `BasmalaPolicy::{CountedAsFirstAyah,
UnnumberedHeader, Absent, PerSurah}`, declared per edition with per-surah
values. QV-020 enforces consistency when the edition policy is edition-wide.
Text-level consistency (does the stored text match the policy) is verified
against the reference corpus (QV-015).

## Accuracy / Religious-source implications

Ayah numbering is downstream of this policy; a wrong policy misnumbers every
ayah in the surah. Hence edition-driven, validated, never inferred.

## Licensing / Security / Operational / Migration implications

None beyond validation. Policy changes require a new edition version.

## Reversal cost

Medium after import (ayah counts and golden fixtures assume it).

## Consequences

- `BasmalaPolicy` in `quran-core`; per-surah column; QV-020.
- Golden edge cases cover Fatihah 1:1, Tawbah 9:1, Naml 27:30.
