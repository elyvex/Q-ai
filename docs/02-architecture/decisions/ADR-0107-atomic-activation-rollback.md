# ADR-0107 — Atomic Activation & Rollback (Staging + Pointer Flip + Generation)

- Status: Accepted
- Phase: 1 — Canonical Quran Core
- Date: 2026-09-14
- Related decisions: ADR-0106, ADR-0108
- Requirements: PRD §7.3, §85; invariants I5, I7

## Context

A partially imported edition must never become readable, and a crash
mid-import must never expose a half-corpus (§85). The importer and the
activator must be different principals with different capabilities.

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| Physically separate staging + single-transaction pointer flip | Crash-safe by construction; readers never see staging | Staging doubles write volume during import |
| In-place import with a status flag | Less storage | A crash leaves readable partial data; readers must filter |
| Importer-held activation capability | Fewer moving parts | Violates I5/I7; a bug can publish unreviewed text |

## Decision

- Staging mirrors (`quran_stg_*`) are physically separate, trigger-free, and
  cascade from `quran_import_runs`; only validation and the activation
  transaction read them.
- The importer holds no `ApprovalToken` and has no code path to canonical
  tables; it ends at `Staged` plus a validation report and a difference report.
- Activation is one transaction: move staging rows to canonical tables,
  deprecate the old `Active`, flip the `quran_active_edition` singleton
  pointer, bump `corpus_generation`, consume staging. Rollback flips the
  pointer to a prior version and bumps the generation the same way.
- Every cache and derived index keys on `corpus_generation`; staleness is
  detectable, never assumed.

## Accuracy / Religious-source implications

Publication of canonical text is a human-gated act (`application::quran`
verifies a granted approval covering the exact edition URN). No automation can
publish silently.

## Licensing implications

Activation records the license snapshot already carried on the staging rows.

## Security implications

Deny-by-default: activation requires (granted approval) + (staged rows) +
(audit write) in one transaction; anything missing aborts everything.

## Operational / Migration implications

`corpus_generation` starts at 1 and increments on every activation/rollback.
`doctor --quran` verifies pointer consistency.

## Reversal cost

The mechanism is load-bearing for every phase; replacing it requires reworking
import, activation, caching, and all derived indexes.

## Consequences

- `SqliteQuranRepository::{activate_edition, rollback_edition}` + approval-gated
  application services + 13-prefix crash matrix (AC-P1-10) + cancellation test
  (AC-P1-11).
