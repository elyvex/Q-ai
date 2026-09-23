# ADR-0208 — Offset-Mapping Representation (`SpanMap`) and Storage Format

- Status: Proposed (implementation complete; acceptance pending owner review)
- Phase: 2 — Quran Search and Linguistics
- Date: 2026-09-23
- Related decisions: ADR-0204 (rule catalog), ADR-0205 (profile ladder), ADR-0201 (FTS)
- Requirements: PRD invariants I10; plan §3.4; AC-P2-04, AC-P2-11
- Implementation: `crates/quran-normalization/src/span.rs`; suites
  `tests/spanmap_properties.rs`, `tests/deterministic_rules.rs`,
  `tests/golden_harness.rs`

## Context

Every search hit must map back to exact canonical character ranges for
highlighting, citation, and re-verification (I10). Normalization deletes,
folds, and expands characters, so the map must survive arbitrary rule
chains and compose across pipeline stages without losing provenance.

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| Total per-character function `derived → canonical` (chosen) | Composition is plain function composition (associative by construction); every derived char always has an image; deletions simply absent from the image | Linear storage in derived length; non-contiguous hulls for heavy deletions |
| Segment list `(derived_range, canonical_range, rule)` as primary | Compact for clean folds | Composition over segment lists is fiddly; associativity must be proved, not inherited |
| Byte offsets | Direct slicing | Breaks on multibyte text; char indices compose cleanly, bytes derive on demand |

## Decision

`SpanMap` stores the total per-character function `forward: Vec<u32>`
(derived char → canonical char) plus per-char provenance rules.
`to_canonical` returns the tight hull (re-slicing + re-normalizing contains
the match for deterministic profiles); `to_derived` returns `None` for fully
deleted regions; `compose` is function composition; `segments()` exposes the
grouped segment view for storage and debugging. All offsets are Unicode
scalar (`char`) indices — never bytes, never grapheme clusters; `byte_range_in`
derives byte ranges on demand for renderers. Grapheme-boundary safety is
enforced by callers (verified in `spanmap_properties.rs` property 2).

Storage: span maps are recomputed from canonical text through the shared
pipeline, never persisted (migration `0014` header; R6 by construction).
Search hits carry validated `CanonicalSpan` ranges, not full maps.

Five plan §3.4 properties hold across every synthetic-ayah × every profile
(`spanmap_properties.rs`); composition associativity additionally fuzzed over
random chains. Full-mushaf sweep reuses these assertions once a licensed
corpus lands (P2-X01).

Known limitation (recorded, not hidden): hull re-normalization (property 5)
is asserted for deterministic profiles L0–L6 only. L7's leading-edge
single-pass heuristics (N18–N21) are input-shape-dependent — e.g. N18 strips
a leading ال from a sliced hull that survived the full-ayah pass — so L7
hits verify via full-ayah `scan_match`, never via hull re-normalization.
Every L7 trace carries the mandatory heuristic flag.

## Accuracy and Religious-Source Implications

A wrong span misattributes text to the mushaf. Mitigations: hull
containment property-tested; the citation resolver independently re-verifies
every hit (AC-P2-12); highlighting renders from validated spans only and
fails closed (`None`, never guessed markers).

## Licensing Implications

None — pure logic, no data.

## Security Implications

None beyond totality: maps are total by construction (`SpanMap::build`
rejects dangling images); all rules are fuzz-tested total over Unicode
(no panics, T22).

## Operational Implications

Recomputing maps per query costs linear time in text length; no stored-map
invalidation problem exists by design. `qai quran normalize --explain`
renders the per-rule maps for inspection.

## Migration Strategy

Representation change requires a new ADR + major profile version bump (maps
are recomputed, so no data migration — only the property suite re-runs).

## Reversal Cost

Low. The map is recomputed everywhere; reverting the representation touches
only `span.rs` and its callers, with the property suite as the gate.

## Acceptance Criteria

- All 5 properties green across every ayah × every profile (AC-P2-04).
- Every hit maps to exact canonical ranges; slicing + re-normalizing
  reproduces the match for deterministic profiles (AC-P2-11).
- No `SearchHit` without a trace; highlighting fails closed (AC-P2-06).
