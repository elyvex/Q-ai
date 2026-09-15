# Quran Import Runbook (D1.14)

> **Audience:** operators importing or upgrading a Quran edition.
> **Related:** `docs/07-technical/quran-corpus-architecture.md`,
> `docs/plans/handoff-p1-to-p2.md`, ADR-0101 (dataset + license).

## 0. Preconditions

- A licensed dataset with a declared script, riwayah, and numbering scheme
  (ADR-0101 sign-off; until then use the synthetic `test-edition-min` fixture).
- A manifest that validates:
  `cargo run -p xtask -- validate <manifest> docs/schemas/quran-edition-source.v1.schema.json`.
- A migrated database (`qai db migrate`).

## 1. Validate before you import

```bash
# Schema-shape check first (no writes):
cargo run -p xtask -- validate fixtures/quran/test-edition-min/manifest.json \
  docs/schemas/quran-edition-source.v1.schema.json

# Full pipeline dry-run (runs every QV rule, writes nothing):
qai quran import <manifest> --dry-run
```

A dry-run reports `Fatal`/`Error`/`Warning` counts. **Any `Fatal` blocks the
import.** `QV-015` (reference-corpus comparison) is a recorded skip until
ADR-0114 is configured — a skip is never a silent pass.

## 2. Import

```bash
qai quran import <manifest>
# → imported <slug>@<version> to Staged
```

The importer writes **only** `quran_stg_*` staging tables. It holds no approval
token and cannot activate anything (invariant I5).

### What happens (13 checkpoints)

`Claimed → ManifestHashed → AdapterSelected → Parsed → UnicodeAudited →
Validated → Tokenized → HashesComputed → Staged → RoundtripVerified →
ReferenceCompared → …`

Every checkpoint is durable: if the process dies, re-running the import resumes
from the last completed checkpoint instead of starting over. It never
double-writes.

## 3. Inspect the staged run

```bash
qai quran validate <slug>@<version>          # staged edition: fatal/error/warn summary
qai quran validate <slug>@<version> --report /tmp/report.json
qai quran diff <slug> --from <old> --to <new>   # once both versions are known
qai quran hashes <slug>@<version>            # re-verify stored hashes
```

`qai quran validate` on a **staged** edition re-runs validation and prints the
counts. On an edition that is already Active it reports
`no staged edition …` (exit 5) — that is expected, not a failure.

## 4. Approve and activate

Activation requires a **recorded human approval** for the exact edition URN
(`quran-edition:<slug>@<version>`). In the interim CLI, `--yes` records the
approval and activates in one step:

```bash
qai quran activate <slug>@<version> --yes
# → activated <slug>@<version> (generation N)
```

Activation flips the active pointer and bumps `corpus_generation` in a single
transaction (I7); canonical rows are never rewritten. Gen-1 is `generation 1`,
and every subsequent activation/rollback increments it.

## 5. Verify

```bash
qai quran edition active                      # the new pointer
qai quran edition list                        # old is now [deprecated]
qai doctor --quran                            # 19 corpus checks, read-only
qai doctor --quran --deep                     # full-corpus hash + token round-trip

# `--json` is a single merged document that validates against the schema:
qai doctor --quran --json > /tmp/doctor.json
cargo run -p xtask -- validate /tmp/doctor.json docs/schemas/doctor.v1.schema.json
```

`doctor --quran` must be read-only (Phase-0 AC-P0-14). A `Fail` check exits 3;
`Warn`/`Skipped` do not.

## 6. Troubleshooting

| Symptom | Likely cause | Action |
|---|---|---|
| `storage unavailable` | data dir/objects path not both set | pass `--data-dir`, or set `QAI_DATA_DIR` (both paths derive from it) |
| `unique constraint violation` | re-importing the same version | each import mints a fresh version row; bump the manifest `version` |
| `FOREIGN KEY constraint failed` | missing provenance/source row | re-import after `qai db migrate`; report if it persists |
| `Fatal` QV finding | malformed dataset | fix the dataset; never edit staging rows to force a pass |
| `no staged edition` | edition already Active/Deprecated | use `rollback` instead of `activate` |
| token round-trip fails | adapter altered text | fix the adapter (see adapter guide); do not patch hashes |

## 7. Never

- Never `UPDATE`/`DELETE` canonical rows — triggers abort it (by design).
- Never import an unlicensed dataset into the canonical catalog (ADR-0101).
- Never re-import the same `slug@version` expecting an overwrite.
- Never treat a `Skipped` check as a pass.
