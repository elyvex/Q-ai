# ADR-0209 — Multi-Analysis Representation; No Authoritative Flag

- Status: **Draft — pending dataset + import implementation (do not mark Accepted)**
- Phase: 2 — Quran Search and Linguistics
- Date: 2026-09-23
- Owner: _unassigned_
- Related decisions: ADR-0203 (dataset), ADR-0210 (roots), ADR-0215 (tagset)
- Requirements: PRD invariant I11; AC-P2-17, AC-P2-18, AC-P2-19
- Implementation: pending Sprint 2.4 (migrations `0017`/`0018`, P2-T63/T75);
  integrity suite `tests/integrity/no_authoritative_analysis.rs` (planned)

## Context

Tokens often carry competing morphological analyses (different datasets,
different readings). Storing a single "correct" analysis (an `is_correct` /
`is_primary` / `selected` column) would silently elect a winner and present
one scholarly position as the text's own.

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| All analyses with attribution, no winner column (chosen direction) | Competing positions coexist; suppression counted under every `AnalysisPolicy`; compare returns verdicts, never resolutions | Consumers must handle multiplicity |
| Single authoritative analysis | Simple reads | Fabricated consensus; breaks I11 |

## Decision (draft)

- Schema contains NO `is_correct`/`is_primary`/`selected` column on analyses
  (schema test enforces the absence).
- `quran.morphology` returns all analyses with dataset attribution;
  `analysis_sources` always reports `analyses_returned` AND
  `analyses_suppressed` (suppression never silent under any policy).
- `quran.morphology_compare` returns per-field verdicts
  (`Identical | CompatibleVariant | Conflicting | OnlyInOne`) with NO
  resolution/synthesis field (contract test + code review).
- Every linguistic row carries Layer B/D provenance (never Layer A); Layer D
  rows carry algorithm + version + confidence and cannot reach
  `human_verified` without a reviewer id (DB CHECK tests).

## Open inputs (blocking acceptance)

- Dataset with one-or-several analyses per token (ADR-0203).
- 18 adversarial fixtures → specific MV rule ids (AC-P2-15).

## Accuracy and Religious-Source Implications

SUBSTANTIVE: I11 is the core anti-fabrication rule for morphology. The
schema-level absence (not just API-level) is the enforcement.

## Licensing Implications

Per-analysis dataset attribution strings (ADR-0203).

## Security Implications

None beyond the review-queue promotion path (suggestions never
auto-verified).

## Operational Implications

`AnalysisPolicy` + suppression reporting on every morphology read; doctor
`quran.morphology.provenance` check.

## Migration Strategy

Lexicon/staging migrations `0017`/`0018`; the no-authoritative-column rule
is a schema invariant from day one (no backfill problem by construction).

## Reversal Cost

Prohibitive by design — reversing I11 needs a new ADR explaining why
fabricated consensus is acceptable (it is not expected to be written).

## Acceptance Criteria

- AC-P2-15/17/18/19/20/21: adversarial MV ids; no winner column; verdicts
  without resolution; suppression counted; Layer B/D provenance with
  reviewer-gated verification.
