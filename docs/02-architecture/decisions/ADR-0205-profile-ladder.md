# ADR-0205 — Normalization Profile Ladder, Versioning, Immutability

- Status: **Draft — pending linguist review (do not mark Accepted)**
- Phase: 2 — Quran Search and Linguistics
- Date: 2026-09-23
- Owner: _unassigned_ (P2-X02 swimlane)
- Related decisions: ADR-0204 (rules), ADR-0213 (generations)
- Requirements: plan §3.3; AC-P2-02, AC-P2-10
- Implementation: `crates/quran-normalization/src/profile.rs`
  (`ProfileRegistry`, append-only enforcement); seed migration `0013`

## Context

Users select profiles, not rules; indexes are built per profile; tool
reproducibility blocks name the profile. If a profile's rule list could
change in place, existing indexes and citations would silently disagree with
the catalog.

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| Append-only versioned ladder L0–L8 (current) | Any rule-list change is a new version + rebuild + drift report; registry rejects same-key rewrites (`QAI-NORM-0003`) | Version proliferation over time |
| Mutable profiles | Less bookkeeping | Silent index/citation drift |

## Decision (draft)

Ladder L0.exact → L1.ws → L2.marks → L3.diacritics → L4.hamza →
L5.codepoints → L6.skeleton → L7.affix(heuristic) → L8.fuzzy
(experimental, off by default, query-time only, shares the L5 rule list).
Cumulative rule lists (each rung extends the previous, §3.3); `indexed`
flags mark built fields; L7 results render the heuristic-matched label; L8
never builds an index field. Adhoc rule sets are allowed but never combined
with a profile, with `adhoc:<sha12>` trace labels.

Monotonicity contract (AC-P2-10): `results(Lₙ) ⊆ results(Lₙ₊₁)` across
indexed profiles — property-tested on the fixture (`search_goldens.rs`
exact⊆L3; full-ladder sweep on licensed corpus).

## Open inputs (blocking acceptance)

- Linguist sign-off on rung boundaries (especially L4/L5 fold grouping and
  L7 heuristic scope).

## Accuracy and Religious-Source Implications

Rung boundaries decide what counts as "the same word" in search. Wrong
grouping merges distinct readings. Mitigation: monotonicity suite +
per-rung golden dossiers + linguist review.

## Licensing Implications

None.

## Security Implications

None beyond registry immutability enforcement (fail-closed re-registration).

## Operational Implications

Profile bump → new version → full rebuild → drift report → `QAI-IDX-0101`
warnings until rebuilt. L8 stays off unless explicitly requested.

## Migration Strategy

Seed rows append-only (`0013` + trigger-guarded `QAI-NORM-0003`).

## Reversal Cost

Low (ladder is additive); removing a rung needs index rebuilds.

## Acceptance Criteria

- AC-P2-02 (with ADR-0204); monotonicity holds for every golden query.
- `normalize --explain` / preview endpoint agree rule-by-rule (AC-P2-38/39).
