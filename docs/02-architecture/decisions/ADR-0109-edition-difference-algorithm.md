# ADR-0109 — Edition Difference Algorithm

- Status: Accepted
- Phase: 1 — Canonical Quran Core
- Date: 2026-09-14
- Related decisions: ADR-0107, ADR-0108
- Requirements: PRD §7.3 (I7), §85

## Context

Corrections create a new version with a difference report and human approval
(I7). The reviewer needs to see exactly what changed at ayah granularity with
character-level ranges — a whole-text diff is unreviewable and a hash-only
comparison is unactionable.

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| Ayah-aligned diff + char-level ranges (`similar`) | Reviewable; stable alignment by identity | New `similar` dependency |
| Hand-rolled Myers | No dependency | Reimplementation risk in review-critical code |
| Hash-only comparison | Trivial | Tells the reviewer nothing |

## Decision

Align ayahs by `(surah, ayah)` identity; classify each as added / removed /
changed / unchanged; for changed ayahs report new-text character ranges via
`similar`. Edition-level metadata folds into a `metadata_changed` boolean.
Persisted as `difference_reports` (`summary_json` + `details_json`) at every
import, against the current active edition (or all-added on first import).

## Accuracy / Religious-source implications

The difference report is the review artifact behind every correction. Ranges
must be exact: the reviewer approves bytes, not summaries.

## Licensing / Security / Operational implications

None beyond storage of the reports. The differ is pure and deterministic.

## Migration / Reversal implications

Report format is versioned (`quran_edition` 1.0.0). Changing granularity later
requires a differ version bump, not a corpus change.

## Reversal cost

Low: the differ is downstream of storage.

## Consequences

- `quran_corpus::differ` + importer `Diffed` checkpoint.
- Activation of a corrected version requires the report plus approval (I7).
