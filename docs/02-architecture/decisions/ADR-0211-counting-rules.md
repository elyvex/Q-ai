# ADR-0211 — Counting Rules, Multi-Analysis Semantics, Numeric-Report Policy

- Status: **Draft — pending linguist inputs (do not mark Accepted)**
- Phase: 2 — Quran Search and Linguistics
- Date: 2026-09-23
- Owner: _unassigned_ (P2-X04 swimlane)
- Related decisions: ADR-0209 (multi-analysis), ADR-0205 (ladder)
- Requirements: plan §8.1; AC-P2-28, AC-P2-29, AC-P2-30
- Implementation: pending Sprint 2.6 counting tools; `CountingRules` type
  with mandatory-field enforcement (P2-T94)

## Context

"How many times does X occur?" has no single answer: it depends on the
normalization profile, the dataset(s) consulted, and how competing analyses
are handled. Publishing bare numbers invites numerology (reading
significance into counts); hiding the rules makes research irreproducible.

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| Mandatory `CountingRules` block on every numeric output (chosen direction) | Reproducible; rule-relativity explicit; determinism testable (same rules ⇒ same number) | Verbose output |
| Bare counts with global defaults | Concise | Irreproducible; numerology-prone |

## Decision (draft)

- Every numeric output carries a complete `CountingRules` block (profile +
  version, datasets + versions, `multi_analysis_handling` mode, window
  definitions, exclusions). Two runs with identical rules produce identical
  numbers (determinism suite); changing `multi_analysis_handling` visibly
  changes the reported count AND the rules block.
- Counts aggregate from exact SQL (`COUNT(*)` over lexicon joins), never from
  FTS term frequencies (tokenizer versions can drift).
- `quran.numeric_report` contains no interpretive commentary;
  `interval_analysis` and `missing_expected_form` emit fixed disclaimers
  verbatim (snapshot-tested). `hapax_search` states its profile prominently
  (counts are rule-relative, not absolute).

## Open inputs (blocking acceptance)

- Linguist judgment on ambiguous counts (P2-X04): what counts as "one
  occurrence" for clitics, repeated refrains, cross-ayah windows.
- Numeric-report policy wording review.

## Accuracy and Religious-Source Implications

SUBSTANTIVE: numbers about revelation text carry rhetorical weight.
Mandatory rules blocks + no-interpretation policy + verbatim disclaimers are
the anti-numerology mechanism (AC-P2-29).

## Licensing Implications

None (rules reference dataset versions by string; no redistribution).

## Security Implications

None beyond determinism (no hidden inputs to counts).

## Operational Implications

Counting-rule changes are versioned inputs; drift detection covers them.

## Migration Strategy

No migration (rules are request parameters, persisted inside reports).

## Reversal Cost

Low. Policy is output-shaping; data layer unaffected.

## Acceptance Criteria

- AC-P2-28/29/30: rules-complete determinism; verbatim disclaimers;
  rule-relative hapax.
