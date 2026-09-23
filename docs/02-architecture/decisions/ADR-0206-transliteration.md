# ADR-0206 — Transliteration Standard (Reserved; Decision Deferred to Phase 4)

- Status: Reserved (no decision taken in Phase 2)
- Phase: 4 (transliteration implementation); reserved in Phase 2 (P2-X05)
- Date: 2026-09-23
- Owner: _unassigned_

## Context

Normalization rules N23 (`transliterate`) and N24 (`phonetic_key`) are
reserved identifiers with no implementation. Transliteration standard choice
(Arabic→Latin scheme comparison) affects future search, export, and graph
labels, but no Phase-2 deliverable needs it.

## Decision

None in Phase 2. N23/N24 resolve to `UnknownRule` through the pipeline;
`--show-rule` documents them as reserved. The standard comparison
(P2-X05) runs before Phase 4; implementation lands there.

## Reversal Cost

None — nothing built on it yet.
