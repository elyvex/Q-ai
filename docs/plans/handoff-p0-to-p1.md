# Handoff — Phase 0 → Phase 1

> **From:** Phase 0 — Foundations & Provenance (completed 2026-09-14)
> **To:** Phase 1 — Canonical Quran Core
> **Verification:** `cargo test --workspace` = 209 passing; `arch-check`, `migrate-check`,
> and `adr-lint` green; `qai` binary smoke-tested. See `docs/05-followups/done.md`.

Phase 1 must **not re-invent** the following assets. Extend them.

## Assets Phase 1 inherits

| Asset | Location | Phase 1 usage |
|---|---|---|
| `DataLayer`, `TrustLevel`, `VerificationStatus` | `crates/domain/src/types.rs` | Quran text = `CanonicalSource`; numbering = `PublisherMetadata` |
| `ContentHash` + `canonical_json_bytes` | `crates/domain/src/hashing.rs` | `text_hash`, `manifest_hash` on the edition |
| `SourceId`/`SourceVersionId` + state machine | `crates/sources/src/lib.rs` | Import → `Staged` → `Approved` → `Active` |
| Manifest parse/validate/sign + ingest | `sources::ManifestParser` | Register `quran_edition_v1`, ingest editions |
| `ApprovalToken` + `CanonicalWriter` | `crates/provenance/src/lib.rs` | The **only** path that may write canonical ayah rows |
| Job types + repository + cancellation/checkpoint | `crates/jobs/src/lib.rs`, `storage::repository::JobRepository` | `quran.import`, `quran.validate`, `quran.reindex` |
| Audit model + hash chain | `crates/audit/src/lib.rs` | Edition activation/rollback events |
| Doctor check registry | `crates/cli/src/doctor.rs` | Add `--quran` checks |
| `Untrusted<T>`, path/archive/SSRF/input guards | `crates/domain/src/security*.rs` | Importing a downloaded edition archive |
| `testkit` fixtures | `crates/testkit/` | Quran corpus golden-file harness |
| Error namespaces | `QAI-QUR-*` reserved | Ready for Phase 1 |
| `CorpusGeneration`, `OutboxRepository`, `Tombstone` | `crates/domain/src/generation.rs`, `crates/storage/src/{repository,workflows}.rs` | First generation for scope `quran:<edition>`; builders consume outbox events |

## Interfaces to build against

- **Storage**: implement repository traits; do not bypass `UnitOfWork`. A
  projection-relevant write must enqueue its outbox row through
  `UnitOfWork::outbox()` in the same transaction (see `storage::workflows`).
- **Canonical writes**: `CanonicalWriter::begin_canonical_change(token, request)`.
  `ApprovalToken` is obtainable only from a persisted human `ApprovalRecord`.
- **Doctor**: add checks to the registry in `crates/cli/src/doctor.rs`; keep it
  read-only and emit a remedy + next command on every non-pass.

## Constraints carried forward

- `domain` depends only on `serde`/`thiserror`/`time`/`uuid` (enforced by
  `cargo xtask arch-check` against `xtask/allowlist.toml`).
- Migrations are append-only and checksummed (`migrations/sqlite/checksums.json`);
  add new `NNNN_*.up.sql` (+ `.down.sql`) files.
- Canonical rows are insert-only (DB triggers); deactivation uses tombstones.
- Secrets never persist in SQLite — store `SecretRef`s only.
- Timestamps are UTC RFC3339; hashes are lowercase `sha256:<hex>`.

## Known limitations / deferred items

| Item | Owner | Notes |
|---|---|---|
| ed25519 manifest signatures | Phase 1 | Add `ed25519-dalek`; current path verifies a SHA-256 detached hash |
| Keychain / age secret backends | Phase 1 | Add `keyring` / `age`; traits + stubs exist |
| In-process job worker pool (T37) | Phase 0.4 / Phase 1 | Repository + cancellation exist; a pool is not built |
| Universal `Diagnostic` impls | Phase 1 | `StorageError`/`SourceError` implement codes; extend to all enums |
| Coverage confirmation | CI | `xtask coverage-gate` enforces thresholds once `cargo llvm-cov` runs |
| Exit-gate ritual recording | Phase 0 owner | Seven live ACs (03/05/06/08/11/14/16) need a recorded walkthrough |

## Suggested first Phase-1 tasks

1. Add `QAI-QUR-*` error variants and the `quran_edition_v1` `StructureValidator`.
2. Define the canonical ayah schema (Phase 1 migration) with insert-only triggers.
3. Import a Quran edition manifest → `Staged`, validate, approve with an
   `ApprovalRecord`, activate via `CanonicalWriter`, emitting outbox events.
4. Add `quran.import` / `quran.validate` jobs and doctor `--quran` checks.
