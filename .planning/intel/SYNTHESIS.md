# Synthesis Summary

Ingest of 36 classified documents (MODE=new, bootstrap; no existing .planning
context). Precedence applied: ADR > SPEC > PRD (no per-doc precedence
overrides present in any classification).

## Doc counts by type

- ADRs: 33 (docs/02-architecture/decisions/ADR-*.md)
- SPECs: 2 (docs/07-technical/quran-citation-spec.md, docs/architecture/hashing-spec.md)
- PRDs: 1 (docs/01-requirements/requirements.md)
- DOCs: 0
- UNKNOWN / low-confidence: 0 (all 36 classified high confidence)

## Decisions locked

25 locked, 8 proposed. Locked sources: ADR-0000, ADR-0001–ADR-0012 (except
none in that range are proposed — all 13 locked), ADR-0102–ADR-0113 (except
none proposed in that range — all 12 locked). Proposed sources: ADR-0101
(Draft, dataset pending sign-off), ADR-0114 (Draft), ADR-0201 (Proposed),
ADR-0202 (Proposed), ADR-0203 (Draft), ADR-0301 (undecided), ADR-0701
(Proposed), ADR-0702 (Proposed). Full entries: `.planning/intel/decisions.md`.

## Requirements extracted

32 requirement entries (REQ-product-vision … REQ-open-decisions-status)
grouped by PRD section from the single PRD. Explicit acceptance criteria were
recorded verbatim-grounded only where the source states complete-only
conditions (PRD §44.3/§45 acceptance workflows, §46 quality gates, §58
definition of done, §90 tool-creation criteria, §93 updated definition of
done); all other entries mark acceptance absent. Full entries:
`.planning/intel/requirements.md`.

## Constraints

2 total — type breakdown: schema × 1 (hashing spec: `sha256:<hex>` + canonical
JSON bytes), api-contract × 1 (Quran citation spec: identifiers, verdicts,
resolution algorithm, CLI/API surface). Full entries:
`.planning/intel/constraints.md`.

## Context topics

5 topic-keyed notes (phase map, open decisions, non-goals, living-PRD and
terminology, status and diagnostics). No DOC-type docs were in the ingest set;
topics are sourced to PRD sections and the open-decision ADRs. File:
`.planning/intel/context.md`.

## Cycle detection

Directed graph built from all classification `cross_refs` (36 nodes); DFS
three-color detection found 8 reference circuits in 3 strongly connected
groups (details in the conflicts report). All are bidirectional "Related
decisions" navigation links; extraction is per-doc with no ref traversal, max
depth far below the 50-level cap, so synthesis proceeded on the full set with
the rationale logged as INFO. No docs were excluded.

## Conflicts

- 0 blockers (unresolved-blockers)
- 0 competing-variants (warnings)
- 8 auto-resolved (info)
- No LOCKED-vs-LOCKED contradiction found: locked ADRs cover disjoint scopes
  and cross-confirm (e.g., ADR-0108 corpus hashing builds on ADR-0006;
  ADR-0111 cites ADR-0102; ADR-0107 generation-stamping underpins ADR-0113).
- No competing PRD acceptance variants (single PRD in set).

Detail: `.planning/INGEST-CONFLICTS.md` (sections map as
BLOCKERS = unresolved-blockers, WARNINGS = competing-variants,
INFO = auto-resolved).

## Per-type intel files (entry point for gsd-roadmapper)

- `.planning/intel/decisions.md` — 33 ADR entries
- `.planning/intel/requirements.md` — 32 requirement entries
- `.planning/intel/constraints.md` — 2 constraint entries
- `.planning/intel/context.md` — 5 topic notes
- `.planning/INGEST-CONFLICTS.md` — conflict report (0/0/8)
