# ADR-0103 — Verse-Numbering Scheme Handling & Alternate Numbering

- Status: Accepted
- Phase: 1 — Canonical Quran Core
- Date: 2026-09-14
- Related decisions: ADR-0101, ADR-0102, ADR-0106
- Requirements: PRD §7.2, §7.4

## Context

Editions number verses differently (Hafs 6236; Kufi and others differ, notably
in basmala counting and a few surah boundaries). Merging numberings produces a
synthetic text that matches no printed muṣḥaf (violates I3).

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| One numbering per edition, stored with it | Never merges readings; each edition is self-consistent | Cross-edition comparison needs explicit mapping |
| Normalized "canonical" numbering across editions | Simpler cross-edition queries | Creates a synthetic scheme matching nothing; hides real differences |
| Alternate-numbering columns on every ayah | Explicit mappings | Bloats the canonical tables; mappings are scholarly data, not canonical |

## Decision

Each edition declares its `verse_numbering_scheme` (`hafs` | `kufi` |
`custom:<name>`); ayah numbers are valid only *within* that edition, and
`edition_id` is part of every canonical key. Cross-edition alignment is a
later-phase mapping layer, never a merged table.

## Accuracy / Religious-source implications

Numbering is part of the reading. Storing Hafs numbers on Kufi text (or vice
versa) misattributes verses; the per-edition scheme plus `edition_id` in every
key makes that unrepresentable.

## Licensing implications

None beyond the dataset license (ADR-0101).

## Security / Operational / Migration implications

No security surface. Operationally, imports validate `1..=declared` per surah
(QV-005) against the edition's own counts. Changing an edition's scheme later
requires a new edition version.

## Reversal cost

Low before citations exist; after that, citations pin `slug@version`, so a
scheme change is a new edition, not an edit.

## Consequences

- `NumberingScheme` in `quran-core`; scheme stored per edition and per
  translation alignment.
- QV-003/005 validate counts against the edition's own declarations.
