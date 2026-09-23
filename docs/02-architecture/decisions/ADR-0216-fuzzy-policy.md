# ADR-0216 — Fuzzy-Search Policy (Experimental, Off by Default)

- Status: **Draft**
- Phase: 2 — Quran Search and Linguistics
- Date: 2026-09-23
- Related decisions: ADR-0205 (ladder, L8), ADR-0212 (budgets)
- Requirements: plan §3.3 (L8); AC-P2-43 (latency table row)
- Implementation: pending (L8 shares the L5 rule list; Levenshtein matching
  is query-time only and NOT implemented in Phase 2)

## Context

Fuzzy spelling search (edit-distance matching) helps typo-tolerant lookup
but risks presenting near-misses as canonical text and has unbounded cost
without careful bounds.

## Decision (draft)

L8.fuzzy is experimental, off by default, and query-time only: it builds NO
index field, requires explicit opt-in per request, and carries the heuristic
label in every trace. Any future implementation inherits I16-style budgets
(edit-distance cap, candidate cap, timeout) and reports them like
`RegexReport`. No Phase-2 tool exposes L8 without explicit opt-in.

## Accuracy and Religious-Source Implications

Near-miss results must never render as canonical quotations without the
heuristic/fuzzy label; citation verification applies unchanged.

## Licensing Implications

None.

## Security Implications

Bounds mandatory before any implementation (DoS surface like regex).

## Operational Implications

Off by default; no index cost; latency gated separately when implemented.

## Migration Strategy

None (no stored state).

## Reversal Cost

None — unimplemented.

## Acceptance Criteria

- L8 remains unindexed and opt-in; any implementation meets AC-P2-13-class
  bounds and AC-P2-12 citation verification.
