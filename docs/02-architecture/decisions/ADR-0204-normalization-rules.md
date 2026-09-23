# ADR-0204 — Arabic Normalization Rule Catalog, Mapping Tables, Rule Ordering

- Status: **Draft — pending linguist review (do not mark Accepted)**
- Phase: 2 — Quran Search and Linguistics
- Date: 2026-09-23
- Owner: _unassigned_ (P2-X02 swimlane)
- Related decisions: ADR-0205 (ladder), ADR-0208 (SpanMap), ADR-0104 (Unicode)
- Requirements: plan §3.2; AC-P2-02; invariants I9, I10
- Implementation: `crates/quran-normalization/src/rules/n01.rs…n22.rs`
  (mapping tables documented per rule); suites `deterministic_rules.rs`,
  `golden_harness.rs` (2,000 mechanical pairs, `reviewed_by:
  pending-linguist`)

## Context

Every fold of Quranic orthography loses information (e.g. stripping harakat
merges distinct readings; Persian-codepoint folding merges keyboard variants;
heuristic affix stripping damages genuine morphemes). An unreviewed fold is
a silent recall/precision bug, so each rule ships with a published mapping
table and a documented loss statement, reviewed by a qualified linguist
BEFORE implementation freezes (T04/T05 are the linguistic freeze).

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| Deterministic folds N01–N17 + labeled heuristics N18–N22 (current) | Exact code-point transforms auditable per table; heuristics visibly labeled in every trace/CLI/API/tool payload | Heuristics approximate morphology (true search is the lexicon path) |
| Aggressive stemming in normalization | Higher recall | Silent linguistic damage; misattribution risk |

## Decision (draft)

N01–N17 deterministic (whitespace, tatweel, harakat, quranic marks,
superscript alef, hamza carriers, wasla, alif maqsura, ta marbuta, Persian
code points, zero-width/bidi, punctuation, digits, pause marks, presentation
forms, NFC, space removal); N18–N22 heuristic affix/repetition rules with
`RuleKind::Heuristic` tagging; N23/N24 reserved for Phase 4 (ADR-0206).
Rule order is the profile ladder order (ADR-0205); reordering requires a new
profile version.

## Open inputs (blocking acceptance)

- Linguist review of each mapping table + loss statement (P2-X02).
- Sign-off on the 2,000-pair golden set (P2-T11; fixtures carry
  `reviewed_by: pending-linguist` until then).

## Accuracy and Religious-Source Implications

SUBSTANTIVE (not boilerplate): wrong folds corrupt search over revelation
text. The freeze discipline (no implementation before linguist review;
golden-value changes need linguist review in the PR) is the mitigation.

## Licensing Implications

None — rule tables are original engineering from Unicode data.

## Security Implications

Totality over Unicode (fuzz-pinned, T22); no panics on hostile input.

## Operational Implications

Rule changes require new profile versions + full rebuild + drift report
(never in-place edits).

## Migration Strategy

Catalog is code + seed rows (`0013`); changing a rule list = new version.

## Reversal Cost

Medium. Rule removal invalidates indexes built on that profile (rebuild).

## Acceptance Criteria

- AC-P2-02: ADR accepted with complete mapping tables; every loss documented.
- 2,000 golden pairs green with linguist sign-off recorded in `docs/reviews/`.
