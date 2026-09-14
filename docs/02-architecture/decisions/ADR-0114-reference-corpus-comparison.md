# ADR-0114 — Reference-Corpus Comparison Procedure (Draft)

- Status: **Draft — pending human sign-off (do not mark Accepted)**
- Phase: 1 — Canonical Quran Core
- Date: 2026-09-14
- Related decisions: ADR-0101, ADR-0108
- Requirements: PRD §35.1 (QV-015)
- Owner: _unassigned_ (swimlane P1-X03)

## Context

QV-015 compares the imported text against an *independent* reference corpus:
zero ayah-text differences, or differences explicitly acknowledged. The corpus,
the comparison procedure, and the sign-off are editorial matters.

## Decision (proposed)

- The comparator diffs every ayah text against the reference; any difference
  fails QV-015 unless acknowledged in the manifest with reviewer identity.
- **Until a reference corpus is configured, QV-015 is recorded as skipped
  (Info finding), never silently passed.** The importer implements this today.
- Sign-off requires a named reviewer distinct from the implementer, recorded
  alongside `verified_by`.

## Consequences

- This ADR stays Draft until the owner names the corpus, the procedure, and
  the signer (needed before P1-T26's comparator runs against real data).
- AC-P1-01's fallback covers the unconfigured case; the exit ritual shows the
  skip explicitly.
