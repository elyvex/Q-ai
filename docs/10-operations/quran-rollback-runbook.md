# Quran Rollback & Recovery Runbook (D1.14)

> **Audience:** operators who need to reverse an activation or recover the
> canonical pointer after a bad import.
> **Related:** `docs/10-operations/quran-import-runbook.md`,
> `docs/10-operations/backup.md`, `docs/10-operations/recovery.md`, ADR-0107.

## 1. Model: pointer flip, never a rewrite

Activation and rollback **only move the active pointer and bump
`corpus_generation`** in one transaction (invariant I7). Canonical rows are
insert-only (I1) and are never mutated or deleted. Consequences:

- A rollback is cheap and safe: no data is lost.
- Old versions remain readable by pinned reference
  (`quran:{slug}@{version}:…`).
- The generation bump invalidates the reader cache and any later-phase index, so
  no stale text is served after the pointer moves (AC-P1-19).

## 2. Roll back to a prior version

```bash
qai quran edition list                     # find the target version
qai quran rollback <slug> --to <version> --yes
# → rolled back <slug> to <version> (generation N+1)
qai quran edition active                   # confirm the pointer
```

Requirements (enforced by the activation service):

- A recorded human approval for the target edition URN.
- The target version must exist.
- The write happens in a single transaction together with a hash-chained audit
  event; if either fails, nothing moves.

After a rollback, re-verify:

```bash
qai quran hashes <slug>@<version>          # stored hashes still match
qai doctor --quran                         # edition_active + checksum checks pass
qai quran get 1:1                          # smoke read on the restored pointer
```

## 3. Deprecate (do not delete)

If a version is wrong and should not be activatable again, deprecate it:

```bash
qai quran deprecate <slug>@<version> --yes
```

Deprecation sets `status = Deprecated` and keeps the rows. It is **not** a
delete; tombstones are used where a projection needs to observe removal.

## 4. Recover from a bad staging run

A failed import never touched canonical rows. To recover:

```bash
# Just re-run the import; checkpoints resume and staging is rebuilt idempotently.
qai quran import <manifest>
```

Cancelling an in-flight import removes its `quran_stg_*` rows and records the
cancellation; it does not affect the active edition (AC-P1-11).

## 5. Recover the database itself

If the database file is lost or corrupt:

1. Stop writers (no CLI/server using the data dir).
2. Restore the most recent backup:
   `qai db restore <backup.db> --yes` (verifies integrity, then swaps).
3. `qai db migrate` to apply any migrations newer than the backup.
4. `qai doctor --quran` — reconcile the active pointer and generation.
5. If the restored generation is stale relative to outbox/projections
   (Phase 2+), trigger a re-derive; the generation key is authoritative.

See `docs/10-operations/recovery.md` for the general database recovery flow.

## 6. Decision matrix

| Situation | Action | Touches canonical rows? |
|---|---|---|
| New version is wrong | `rollback --to <previous>` | No (pointer flip) |
| Version must never reactivate | `deprecate <slug>@<version>` | No (status) |
| Import failed / half-staged | re-run `import` (resume) | No (staging only) |
| Import cancelled | nothing (staging cleaned) | No |
| DB file corrupt | `db restore` + `db migrate` | restore from backup |
| Text found corrupt *after* activation | rollback + fix dataset + re-import new version | No (new version) |

## 7. Never

- Never hand-edit canonical or staging tables to "fix" text — the hashes and
  triggers exist to stop exactly this.
- Never rollback without an approval record on the target URN.
- Never assume a rollback restores an old *generation number*; it bumps forward.
  Consumers must key on the generation value, not its magnitude.
- Never skip the `doctor --quran` verification after a rollback.
