# Contract: QAI-DB Error Codes

**Feature**: `027-storage-repository-contracts` | **Date**: 2026-09-18

Closed code set (`storage/src/error.rs`). Driver errors never cross the boundary — adapters map them (see 004 `map_sqlx_error`).

| Code | Variant | Retryable | Typical remedy / next command |
|---|---|---|---|
| `QAI-DB-0001` | Conflict | yes | Retry or treat idempotent submit as no-op |
| `QAI-DB-0002` | NotFound | no | Verify the id/version; `db status` / rebuild |
| `QAI-DB-0003` | ImmutableSourceVersion | no | Create a new version; never rewrite history |
| `QAI-DB-0004` | ConstraintViolation | no | Fix the violating input/state transition |
| `QAI-DB-0005` | StorageBusy | yes | Bounded retry with backoff |
| `QAI-DB-0006` | MigrationRequired | no | `qai db migrate` |
| `QAI-DB-0007` | MigrationChecksumMismatch | no | `qai db verify`; investigate drift |
| `QAI-DB-0008` | IdempotencyKeyReplay | no | Safe no-op; reuse the original result |
| `QAI-DB-0009` | StorageUnavailable | yes | Check backend availability; retry |

Rendering: `code()` + human `remedy()` + `Diagnostic` (`summary`, `cause_chain`, `next_command`, `is_retryable`) via `render_human` / `render_json` — both carry the identical code; JSON contains code/summary/why/location/remedy/next_command/retryable.
