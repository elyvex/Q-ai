# ADR-0215 — Unified Morphological Tagset + Dataset Tag Mapping

- Status: **Draft — pending dataset-native tag analysis + linguist inputs**
- Phase: 2 — Quran Search and Linguistics
- Date: 2026-09-23
- Owner: _unassigned_ (P2-X03 swimlane)
- Related decisions: ADR-0203 (dataset), ADR-0210 (roots), ADR-0209 (multi-analysis)
- Requirements: plan §6.1; AC-P2-17
- Implementation: pending Sprint 2.4 tagset mapper (P2-T62); native tags
  preserved verbatim alongside unified tags

## Context

Each morphology dataset ships its own part-of-speech/feature tag inventory
with different granularity and different theoretical commitments. Querying
"all verbs of form X" across datasets needs a unified tagset — but mapping
natively fine-grained tags onto coarser unified tags loses information, and
re-tagging source data rewrites scholarship.

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| Unified tagset + verbatim native tags + explicit mapping (chosen direction) | Cross-dataset queries work; no source information lost; mapping itself reviewable | Mapping table maintenance per dataset |
| Native tags only | Zero mapping risk | No cross-dataset morphology queries |
| Retag everything into one tagset | Uniform | Destroys source distinctions; misattribution |

## Decision (draft)

Every analysis row carries BOTH the dataset-native tag string (verbatim,
immutable) and the unified tag (via the versioned mapping table). The
mapping table is itself a reviewed artifact (linguist sign-off, P2-X03);
unmappable native tags map to an explicit `Unmapped` value with the native
string preserved — never to a guessed nearest tag. `morphology_compare`
verdicts (`Identical | CompatibleVariant | Conflicting | OnlyInOne`) are
computed over stated tags, never synthesized.

## Open inputs (blocking acceptance)

- Dataset-native tag inventory analysis (needs the licensed dataset, P2-X01).
- Linguist-authored unified tagset + mapping (P2-X03).

## Accuracy and Religious-Source Implications

Tagging is grammatical scholarship. Verbatim preservation + explicit
`Unmapped` + no-synthesis compare (AC-P2-18) keep attribution honest.

## Licensing Implications

Tag inventories may be dataset-licensed; mapping tables reference them by
string (attribution per ADR-0203).

## Security Implications

None.

## Operational Implications

New dataset version → re-run mapping coverage report; `Unmapped` rate gates
import approval (coverage + unmatched-token reports, P2-T68).

## Migration Strategy

Tagset lives in lexicon tables (`0017`/`0018`); mapping version bumps ride
dataset versions.

## Reversal Cost

Low. Mapping is additive; native tags always present.

## Acceptance Criteria

- AC-P2-17/18/19: no authoritative column; verdicts without resolution;
  suppression always counted.
