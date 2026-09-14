# ADR-0106 — Canonical Text Storage Layout (Row-per-Ayah in SQLite)

- Status: Accepted
- Phase: 1 — Canonical Quran Core
- Date: 2026-09-14
- Related decisions: ADR-0001, ADR-0107, ADR-0108
- Requirements: PRD §7.1, §7.3, §32.1

## Context

The corpus needs query flexibility (ayah lookup, ranges, divisions, token
spans) and hash simplicity, in SQLite today with PostgreSQL portability later
(Phase 12).

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| Row-per-ayah + row-per-token + separators table | Query flexibility; covering indexes; per-row hashes | More rows than a blob |
| Blob + index (whole edition as one object) | Hash simplicity | No structural queries; every lookup parses |
| FTS5-backed storage | Built-in search | Search is Phase 2 and must never become the source of truth |

## Decision

Row-per-ayah (`quran_ayahs`), row-per-token (`quran_tokens`), exact separators
(`quran_token_separators`), divisions (`quran_divisions`), all keyed by
`(edition_id, …)` with covering indexes on global ayah/token order. Canonical
tables are insert-only via triggers (`QAI-QUR-0001…0005`); staging mirrors
(`quran_stg_*`) carry no triggers and cascade from `quran_import_runs`.

Due to the repository's own contiguity gate, the migrations landed as
`0007`–`0012` instead of the plan's `0010`–`0015` (DEV-02); contents follow
plan §6 exactly. No `.down.sql`: canonical tables are forward-only.

## Accuracy / Religious-source implications

Row granularity makes byte-equality and offset checks per-ayah testable, and
the triggers make silent alteration a database error rather than a code-review
promise.

## Licensing implications

None.

## Security implications

Triggers are the last line of defence behind the `ApprovalToken` write path;
staging tables are readable only by validation and the activation transaction.

## Operational / Migration implications

Portability rule holds: no SQLite-only SQL in shared code paths. Recomputed
hashes (`doctor --quran --deep`) scan these tables directly.

## Reversal cost

Very high after import: every downstream index, citation, and hash assumes
these rows. Layout change = new phase-level ADR + re-import.

## Consequences

- Migrations `0007`–`0012` + `QuranRepository` + SQLite implementation.
- AC-P1-08 automated-green (raw-SQL trigger tests).
