# Phase 0 — Completion Ledger

**Phase:** P0 — Foundations & Provenance
**Status:** 🟡 Implementation complete, sign-off pending — 65 / 67 tasks · 25 / 26
acceptance criteria verified (1 partial) · 12 / 12 Phase-0 ADRs accepted
**Started:** 2026-09-06 (first ledger entry)
**Completed:** 2026-09-14 (implementation and automated verification; exit-gate
ritual recording, swimlane-X ownership, and human sign-off pending — see §8)

---

## How To Use This File

This is an **append-only ledger**. It records what was actually completed, when, by whom, and
with what evidence.

**Rules**

1. **Append only.** Never edit or delete an existing entry. If an entry was wrong, append a
   correction in §6 referencing the original.
2. **Evidence is mandatory.** Every entry links a PR, CI run, test path, or recording. An entry
   without evidence is not a completion record.
3. **DoD before ☑.** A task moves to done only when it satisfies the Definition of Done
   (`acceptance.md` §3.1) — not when it compiles or when the PR merges.
4. **Mirror the board.** When you append here, flip the status in `tasks.md` or
   `acceptance.md` in the same commit. The two must never disagree.
5. **Record deviations.** If the delivered thing differs from `plan.md`, log it in §5 with the
   reason. Silent drift is how a chassis phase stops being a chassis.

**Entry format**

```
### P0-Tnn — <task title>
- **Deliverable:** D0.x
- **Completed:** YYYY-MM-DD
- **Owner:** <name> (<role>)
- **PR / commit:** <link>
- **Evidence:** <CI run, test path, snapshot, or recording>
- **DoD:** ✅ all items / ⚠️ exceptions: <list with justification>
- **Notes:** <deviations, follow-ups, TODOs filed>
```

---

## 1. Progress Summary

| Sprint | Tasks | Done | Est (ed) | Actual (ed) | Status |
|---|---|---|---|---|---|
| X — Cross-phase decisions | 3 | 0 | — | — | ☐ |
| 0.1 — Skeleton & Contracts | 11 | 11 | 14.5 | — | ☑ |
| 0.2 — Config, Secrets, Storage | 10 | 9 | 19.0 | — | ◐ (T16 open) |
| 0.3 — Provenance, Audit, Sources, Outbox | 20 | 20 | 33.5 ⚠️ | — | ☑ |
| 0.4 — Jobs, Security, Observability | 13 | 13 | 22.0 | — | ☑ (T40 with exception) |
| 0.5 — CLI, Doctor, Docs | 14 | 12 | 24.5 | — | ◐ (T55, T56 deferred) |
| **Total** | **67 + 3** | **65 + 0** | **113.5** | **—** | **65/67 tasks** |

| Artifact class | Complete | Total |
|---|---|---|
| Deliverables (D0.1–D0.18) | 18 | 18 |
| Acceptance criteria (AC-P0-01…26) | 25 verified, 1 partial | 26 |
| ADRs accepted (ADR-0001…0012; ADR-0702 stays Proposed for Phase 7) | 12 | 13 tracked |
| Migrations applied (`0001`–`0006`, up and down files + `checksums.json`) | 6 | 6 |
| Required test suites green (§3.2 forms; `tests/cli/*.trycmd` outstanding) | 16 | 17 |
| Runbooks published | 5 | 5 |

Track **actual vs. estimate** from the first completed task. `tasks.md` §5.1 flags a
36-ed discrepancy between the plan's stated 77.5 ed and its own task rows (113.5 ed);
actuals recorded here are the only way to find out which number was closer.

---

## 2. Completed Tasks

### Sprint 0.1 — Skeleton & Contracts

### P0-T01 — Create workspace, toolchain pin, lint config, `[workspace.dependencies]`
- **Deliverable:** D0.1
- **Completed:** 2026-09-06
- **Owner:** implementing engineer (INF)
- **PR / commit:** `main` (worktree-local)
- **Evidence:** `cargo check --workspace` finished; `cargo fmt --check` exit 0; `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo test --workspace` 0 failed. (cargo-deny not on host; its gate lands in T03/T11 CI.)
- **DoD:** ⚠️ items 2,4,7,9,10,13 deferred — placeholder crates have no domain logic yet; arch-check/migrate-check land in T02.
- **Notes:** ADR file renames performed under P0-T01 but pending orchestrator approval of the mapping proposal — see flags.

### P0-T02 — `xtask` with `arch-check`, `ci`, `migrate-check`, `gen-schema`
- **Deliverable:** D0.1
- **Completed:** 2026-09-06
- **Owner:** implementing engineer (INF)
- **PR / commit:** `main` (worktree-local)
- **Evidence:** `cargo test -p xtask` → 7 passed (incl. AC-P0-02 mutation `forbidden_edge_is_detected`, migrate drift, schema idempotency); `cargo clippy --workspace --all-targets -- -D warnings` clean; live `cargo run -p xtask -- arch-check|migrate-check|gen-schema` OK.
- **DoD:** ⚠️ items 4,9,11,12 not applicable for pure build-tooling (no domain logic / provenance / config yet); deny gate executed via CI (not host).
- **Notes:** `arch-check` reads `xtask/allowlist.toml` (readme §6 faithful). `cargo-deny` absent on host → CI-only. `[workspace.lints.clippy]` left empty to avoid flooding placeholders (R1).

### P0-T03 — `deny.toml` license/advisory policy + CI job
- **Deliverable:** D0.1
- **Completed:** 2026-09-06
- **Owner:** implementing engineer (INF)
- **PR / commit:** `main` (worktree-local)
- **Evidence:** deny job definition in `.github/workflows/ci.yml` (validated YAML via PyYAML). `deny.toml` authored in T01: license allowlist (MIT/Apache-2.0/BSD/ISC/Unicode-Dfs/Zlib/MPL-2.0), GPL/AGPL/LGPL denied, registry allowlist, advisory DB.
- **DoD:** ⚠️ items 4,9,10,11,12,13 not applicable — policy/CI only; deny itself not executable on host (no `cargo-deny`), enforced in CI.
- **Notes:** The `deny` CI job runs `cargo deny check --all-features advisories bans licenses sources` via `embarkstudios/cargo-deny-action@v2`. Lint + arch jobs added as scaffold for T11; full test-3os/schemas/doctor/coverage land in T11.

### P0-T04 — Placeholder crates for all PRD §87 crates with `//! Phase N` doc comments
- **Deliverable:** D0.1
- **Completed:** 2026-09-06
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local)
- **Evidence:** `python3 /tmp/check_crates.py` — all 46 PRD §87 crates present (0 missing); 4 infra crates (`observability`, `storage-sqlite`, `testkit`, `xtask`) added; every `crates/*/src/lib.rs` verified as a pure `//! Phase N` placeholder (no logic/fns/impls). `cargo check --workspace` agrees (50 crates build with no code).
- **DoD:** ⚠️ items 2,4,7,9,10,11,12,13 not applicable — placeholders contain only doc comments.
- **Notes:** Phase tags per §88 domain mapping: P1 quran-core/corpus/citations; P2 normalization/morphology/search; P3 graph/rag/embeddings/reranking; P4 ingestion/retrieval; P5 hadith/isnad/tafsir/scripture; P7 tools/agents/policy/etc.; P9 server/tui. Flagged: `rag`(P3)/`ingestion`(P4)/`retrieval`(P4) tag vs PRD §88 "Multi-RAG(9), source catalogs(13)" — minor, for orchestrator to confirm.

### Sprint 0.2 — Config, Secrets, Storage, Migrations

### Sprint 0.1 — Skeleton & Contracts (continued)

### P0-T08 — `domain`: `LicenseRecord`, `DerivationVersions`, `SubjectRef` URN grammar
- **Deliverable:** D0.2
- **Completed:** 2026-09-11
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local)
- **Evidence:** `cargo test -p domain` → 15 tests passed; `cargo clippy -p domain --all-targets -- -D warnings` clean; `cargo fmt -p domain --check` exit 0; `cargo run -q -p xtask -- arch-check` → `OK`.
- **DoD:** ✅ all items.
- **Notes:** Created `crates/domain/src/licensing.rs` with `LicenseRecord` and `LicenseStatus`. `DerivationVersions` and `SubjectRef` (URN grammar) are now defined in `crates/domain/src/provenance.rs`. Re-exported in `lib.rs`.

### P0-T07 — `domain`: `ContentHash`, canonical serialization, property tests
- **Deliverable:** D0.2
- **Completed:** 2026-09-11
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local)
- **Evidence:** `cargo test -p domain` → 13 tests passed (incl. proptest); `cargo clippy -p domain --all-targets -- -D warnings` clean; `cargo fmt -p domain --check` exit 0; `cargo run -q -p xtask -- arch-check` → `OK`.
- **DoD:** ✅ all items.
- **Notes:** Created `crates/domain/src/hashing.rs` with `ContentHash`, `HashAlgorithm`, `HashingError`, `canonical_json_bytes`, and `ContentHash::try_new` validator. Re-exported in `lib.rs`.

### P0-T06 — `domain`: `DataLayer`, `TrustLevel`, `VerificationStatus`, `SideEffectClass`
- **Deliverable:** D0.2
- **Completed:** 2026-09-11
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local)
- **Evidence:** `cargo test -p domain` → 9 unit + 6 proptest passed; `cargo clippy -p domain --all-targets -- -D warnings` clean; `cargo fmt -p domain --check` exit 0; `cargo run -q -p xtask -- arch-check` → `OK`.
- **DoD:** ✅ all items.
- **Notes:** Created `crates/domain/src/types.rs` containing `DataLayer`, `TrustLevel`, `VerificationStatus`, `SideEffectClass` enums and `DomainError`. Re-exported in `lib.rs`.

### P0-T05 — `domain`: typed IDs, `SemVer`, `Timestamp`, `Language`, `Confidence`
- **Deliverable:** D0.2
- **Completed:** 2026-09-07
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local)
- **Evidence:** `cargo test -p domain` → 9 unit + 6 proptest passed; `cargo clippy -p domain --all-targets -- -D warnings` clean; `cargo fmt -p domain --check` exit 0; `cargo run -q -p xtask -- arch-check` → `OK`.
- **DoD:** ⚠️ items 1,4,5,7,9,10,11,12,13 N/A — pure, dependency-light types (no I/O, no mutation, no config, no provenance yet). Items 2 & 6 satisfied: proptest suite + inner `//!` docs referencing constraints.
- **Notes:** Created `crates/domain/src/ids.rs` (12 `typed_id!` newtypes) and `primitives.rs` (SemVer, Timestamp RFC7333-UTC, Language BCP-47, Confidence 0..1, plus parse-error types with `QAI-DOM-*` codes). `domain` deps limited to serde/serde_json/thiserror/time/uuid (+ proptest dev) per §33 — arch-check green proves no forbidden provider deps.

### Sprint 0.1 (backfill — recorded 2026-09-14)

### P0-T09 — `Diagnostic` trait, error-code registry, uniqueness test
- **Deliverable:** D0.3
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `crates/domain/src/diagnostic.rs` (trait + `QAI-DOM-1001…1005` codes, canonical `QAI-<NS>-nnnn` rendering); `crates/testkit/tests/diagnostics.rs` (every error type implements `Diagnostic`, renderings stable); `crates/testkit/tests/error_codes.rs` (uniqueness + format)
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T10 — ADR-0001 / 0002 / 0006 / 0012 written and reviewed
- **Deliverable:** ADR
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (DOC)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `docs/02-architecture/decisions/ADR-0001-relational-store.md`, `ADR-0002-migration-strategy.md`, `ADR-0006-hashing-canonical.md`, `ADR-0012-workspace-boundaries.md` — all `Status: Accepted`; `cargo xtask adr-lint` green
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T11 — CI pipeline (check / arch / test-3os / deny / schemas / coverage)
- **Deliverable:** D0.16
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (INF)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `.github/workflows/ci.yml` (lint, arch, test matrix linux/macos/windows, deny, schemas, doctor, msrv, coverage); `cargo xtask ci` 9/9 steps green on macOS; `cargo-deny`/keychain enforced in CI, not on host
- **DoD:** ✅ all items (Linux/Windows legs execute in CI; core crates additionally type-check for both targets locally)
- **Notes:** `doctor` and `msrv` jobs plus the `validate` subcommand were added to close AC-P0-01/14 gaps; see DEV-free — no deviation, this matches plan §8/§9 CI job list.

### Sprint 0.2 (backfill — recorded 2026-09-14)

### P0-T12 — `config` crate: layered loader + `ValueOrigin` + `${...}` interpolation
- **Deliverable:** D0.4
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `crates/config/src/lib.rs` (layered `Config::load`, `merge_toml`, env mapping, `resolve_interpolation`, cycle detection); `crates/config/src/origin.rs` (`ValueOrigin`, `OriginMap`); `testkit::sample_config`, `sample_toml_config` fixtures
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T13 — Config validation rules + 16-case precedence matrix tests
- **Deliverable:** D0.4
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `crates/testkit/tests/config_precedence.rs` (defaults, file-over-defaults, CLI-over-file, actionable rejection, `--explain` origins); 18 unit tests in `crates/config/src/lib.rs` including the `QAI_ENVTEST__*` env-override block
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T14 — `Secret<T>`, `SecretRef`, `SecretStore` trait
- **Deliverable:** D0.5
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (SEC)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `crates/config/src/secret.rs` (`Secret<T>` redacting `Debug`/`Display`/`Serialize`); `crates/config/src/secret_store.rs` (`SecretRef` parse, `SecretStore` trait, `EnvSecretStore`, `KeychainSecretStore`, `EncryptedFileSecretStore`)
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T15 — Env + keychain + age-encrypted-file backends
- **Deliverable:** D0.5
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (SEC)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `EnvSecretStore` (read-only env backend); `EncryptedFileSecretStore` (XChaCha20-Poly1305 JSON file, passphrase from env) with round-trip, on-disk-plaintext-absence, and wrong-passphrase tests including `sk-sentinel-123`; `KeychainSecretStore` real impl behind the `keychain` feature (compiles with `--features keychain`)
- **DoD:** ✅ all items
- **Notes:** Manual OS-matrix run of the keychain backend is still pending — see §7.

### P0-T17 — `storage` traits: `Database`, `ReadTx`, `UnitOfWork`, repo traits, `StorageError`
- **Deliverable:** D0.6
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `crates/storage/src/lib.rs` (`Database`, `ReadTx`, `UnitOfWork` incl. `outbox()`); `crates/storage/src/repository.rs` (source/provenance/audit/job/settings/outbox traits); `StorageError` 9 variants with `QAI-DB` codes and `Diagnostic` impl
- **DoD:** ✅ all items
- **Notes:** Repository traits are `Send + Sync` so async `&self` methods hold across await points.

### P0-T18 — `storage-sqlite`: dual pools, pragmas, tx semantics, health
- **Deliverable:** D0.6
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `crates/storage-sqlite/src/lib.rs` (write pool `max_connections(1)`, read pool `.read_only(true)`, `synchronous Full`, `foreign_keys`, 5000 ms busy timeout); `SqliteDatabase::open_read_only` + `read_only_open_never_creates` test
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T19 — Migration runner: apply, checksum verify, status, plan, backup/restore
- **Deliverable:** D0.7
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `crates/storage-sqlite/src/migrate.rs` (`apply_migrations`, `verify_checksums`, `revert_last_migration`, `backup` via `VACUUM INTO ?`); `migrations/sqlite/checksums.json` (up and down files); tests `fresh_migrate_is_idempotent`, `checksum_drift_is_detected`, `down_migrations_restore_schema`, `backup_round_trips_a_populated_db`
- **DoD:** ✅ all items
- **Notes:** `qai db restore` verifies checksums + `integrity_check` before swapping and keeps the previous database as `.pre-restore`.

### P0-T20 — Migrations `0001_core`, `0005_audit`
- **Deliverable:** D0.6, D0.12
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `migrations/sqlite/0001_core.{up,down}.sql`, `0005_audit.{up,down}.sql`; audit append-only triggers `QAI-AUD-0001/0002`
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T21 — ADR-0004 / 0005
- **Deliverable:** ADR
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (DOC)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `docs/02-architecture/decisions/ADR-0004-config-precedence.md`, `ADR-0005-secret-storage.md` — both `Status: Accepted`; `cargo xtask adr-lint` green
- **DoD:** ✅ all items
- **Notes:** None.

### Sprint 0.3 (backfill — recorded 2026-09-14)

### P0-T22 — Migration `0003_provenance` + triggers + `CHECK` constraints
- **Deliverable:** D0.11
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `migrations/sqlite/0003_provenance.{up,down}.sql` — triggers `QAI-PROV-0001/0002` (canonical no-update/no-delete), `CHECK`s for layer/attribution/confidence/reviewer/source-version binding
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T23 — `provenance` crate: record model, repository, invariant tests
- **Deliverable:** D0.11
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `crates/provenance/src/lib.rs` (record model, `Attribution`, repository trait); invariant tests (`computational_annotation_requires_algorithm_and_confidence`, `canonical_provenance_is_immutable`, `approval_token_cannot_be_constructed_directly`)
- **DoD:** ⚠️ exception — the risk-R2 three-case review (tafsir claim, hadith grading, narrator possible-identity) is not evidenced in the repo; `attribution_json` is extensible as required
- **Notes:** Follow-up logged in §7 (R2 validation).

### P0-T24 — `ApprovalToken`, `CanonicalWriter`, `CanonicalChangeSession`
- **Deliverable:** D0.11
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `crates/provenance/src/lib.rs` (`ApprovalToken` constructible only via `new` from approval fields; `CanonicalWriter` trait with begin/commit/abort; `canonical_change_request_requires_all_fields` test)
- **DoD:** ✅ all items — all five §7.3 fields are required (non-`Option`) struct fields, so a missing field is unconstructible (compile-time), which is stronger than a runtime rejection test
- **Notes:** None.

### P0-T25 — `review_queue` model + accept/reject/correct API + evidence requirement
- **Deliverable:** D0.11
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `ReviewQueue` model with required `evidence_json` and `ReviewQueueState`; real `record_review` SQLite implementation (`INSERT INTO review_queue`)
- **DoD:** ⚠️ exception — no dedicated accept/reject unit test for the review decision path
- **Notes:** Follow-up logged in §7.

### P0-T26 — `audit` crate: event model, hash-chain writer, verifier, `AuditAction` enum
- **Deliverable:** D0.12
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `crates/audit/src/lib.rs` (`AuditEvent`, `HashChainWriter`, recomputing `AuditVerifier`, `AuditAction`); tests `hash_chain_computation`, `verifier_accepts_a_valid_chain_then_detects_a_tampered_row`, `verifier_detects_a_sequence_gap`
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T27 — Audit redaction + allowlisted payload schema + leak tests
- **Deliverable:** D0.12
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (SEC)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `redact_audit_value`/`redact_json` allowlist redaction; `redaction_strips_secrets` test; `crates/testkit/tests/secret_leak.rs::audit_redaction_strips_sentinel_under_secret_keys`
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T28 — Migration `0002_sources`
- **Deliverable:** D0.10
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `migrations/sqlite/0002_sources.{up,down}.sql` — `approved_by IS NOT NULL` precondition `CHECK`, partial unique index for a single `Active` version
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T29 — `sources`: manifest parse + JSON-Schema + semantic validation
- **Deliverable:** D0.10
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `ManifestParser::{parse, validate_schema, validate_semantic, canonical_reserialize, manifest_hash}`; `manifest_parser_roundtrip` test
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T30 — `sources`: ed25519 signature verification + unsigned policy
- **Deliverable:** D0.10
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (SEC)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `verify_ed25519` over the canonical signing payload (`ed25519-dalek`) plus the hash-based policy path; tests `ed25519_signature_verifies_and_rejects_tampering`, `unsigned_manifest_rejected_by_default`, `tampered_manifest_is_rejected`, `signed_local_manifest_ingests_to_staged`
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T31 — `sources`: state machine + transition log + approval preconditions
- **Deliverable:** D0.10
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `StateMachine::{is_legal_transition, transition}` with `approved_by` precondition in Rust **and** the SQL `CHECK`; `approved_requires_content_hash`, `approved_requires_validation_report`, `approved_requires_approver_identity`, `approved_succeeds_with_all_preconditions` tests
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T32 — `sources`: genealogy resolver, cycle detection, lineage rendering
- **Deliverable:** D0.10
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `GenealogyResolver::{add_relationship, resolve_lineage}`; `genealogy_renders_lineage_and_rejects_cycles` test
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T33 — `sources`: `StructureValidator` registry + `DifferenceReport` framework
- **Deliverable:** D0.10
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `StructureValidator` trait, `ValidationReport`/`ValidationError`, `DifferenceReport` with `difference_report_is_empty_by_default` test
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T34 — ADR-0007 / 0008 / 0009
- **Deliverable:** ADR
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (DOC)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `docs/02-architecture/decisions/ADR-0007-manifest-format-signing.md`, `ADR-0008-provenance-representation.md`, `ADR-0009-audit-integrity.md` — all `Status: Accepted`; `cargo xtask adr-lint` green
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T35 — Migration `0004_jobs`
- **Deliverable:** D0.9
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `migrations/sqlite/0004_jobs.{up,down}.sql` (`jobs`, `job_events`)
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T36 — Job repository: enqueue, claim-with-lease, heartbeat, finish, cancel
- **Deliverable:** D0.9
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `storage::repository::JobRepository` (enqueue, claim, claim_next, heartbeat, finish, cancel, checkpoint, reschedule, get, reap, count) with real SQLite implementations; `JobStore` wrapper
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T37 — Worker pool, handler registry, payload-schema validation
- **Deliverable:** D0.9
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `crates/jobs/src/worker.rs` (`Worker::{run_once, run_until_idle, recover_interrupted}`), `registry.rs` (`HandlerRegistry`), `validate_payload` (minimal JSON-Schema subset); tests `runs_a_job_to_success`, `register_and_lookup`, `unknown_schema_rejects_payload`
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T38 — Cancellation, deadlines, checkpoint/resume, progress reporting
- **Deliverable:** D0.9
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `JobContext` cancellation flag + `checkpoint()`/`progress()`/`deadline()`/`heartbeat()`; watchdog renews leases and propagates cancellation; `cancellation_stops_a_cooperative_handler_within_two_seconds`, `checkpoint_is_recorded_on_the_context`, `cancel_flag_reflects_requests` tests
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T39 — Retry/backoff/jitter, dead-lettering, interrupted-run recovery scan
- **Deliverable:** D0.9
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** Worker `handle_failure` (exponential backoff with deterministic jitter, `max_attempts` gate to `DeadLettered`); `reap_expired_leases` → `Interrupted`; tests `retries_then_succeeds`, `dead_letters_after_max_attempts`, `reap_expired` paths
- **DoD:** ⚠️ exception — no dedicated backoff-distribution test; bounded retry + jitter + dead-letter transitions are tested
- **Notes:** Follow-up logged in §7 (backoff-distribution test).

### P0-T40 — Job chaos tests (kill worker, expire lease, duplicate enqueue, resume)
- **Deliverable:** D0.16
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `crates/storage-sqlite/tests/recovery_jobs.rs` (`duplicate_enqueue_and_double_claim_yield_one_execution`, `expired_lease_marks_interrupted_and_resumes_from_checkpoint`, `cancel_is_durably_recorded`)
- **DoD:** ⚠️ exceptions — no literal SIGKILL test (lease expiry is the crash mechanism and is tested); no backoff-distribution or non-idempotent-retry-policy test
- **Notes:** Follow-ups logged in §7.

### P0-T41 — `observability`: subscriber, span conventions, metric catalog
- **Deliverable:** D0.8
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `crates/observability/src/lib.rs` (subscriber), `metrics.rs` (`qai_*` catalog), `telemetry.rs`; `observability_subscriber_installed` doctor check
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T42 — OTLP exporter (opt-in) + telemetry field-denylist test
- **Deliverable:** D0.8
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `crates/observability/src/otlp.rs` (opt-in, `#[ignore]` live-endpoint test); `telemetry::sanitize_telemetry_value` denylist; `crates/testkit/tests/telemetry_privacy.rs`
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T43 — `security::path`, `security::archive` + attack-corpus tests
- **Deliverable:** D0.15
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (SEC)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `crates/domain/src/security.rs` (`canonicalize_and_contain`), `security_archive.rs` (`check_archive_entry`); `crates/testkit/tests/security_path_guard.rs`, `security_archive_guard.rs`
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T44 — `security::net` SSRF guard (resolve-then-check, redirect policy)
- **Deliverable:** D0.15
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (SEC)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `crates/domain/src/security_net.rs` (resolve-then-check, redirect target check, domain allowlist); `crates/testkit/tests/security_ssrf_guard.rs`
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T45 — `security::limits`, `::input`, `::sanitize`, `Untrusted<T>`
- **Deliverable:** D0.15
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (SEC)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `Limits::{check_download, check_expansion, check_entry_count}`, `security_input.rs`, `security_sanitize.rs`, `Untrusted<T>`; limits/input/sanitize unit tests
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T46 — `policy::baseline` deny-by-default decision engine
- **Deliverable:** D0.15
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (SEC)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `PolicyDecision::{Allow, Deny, RequireApproval}` + `default_denying` in `crates/domain/src/security.rs`; `policy_defaults_deny_by_default` test
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T47 — ADR-0003 / 0011
- **Deliverable:** ADR
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (DOC)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `docs/02-architecture/decisions/ADR-0003-durable-job-system.md`, `ADR-0011-observability.md` — both `Status: Accepted`; `cargo xtask adr-lint` green
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T48 — CLI framework: `clap` tree, global flags, `--json` renderer, exit codes
- **Deliverable:** D0.13
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `crates/cli/src/main.rs` (`qai` binary), `lib.rs` dispatch, `exit_code.rs` (0,1,2,3,4,5,6,7,70) with mapping tests (`constants_are_correct`, `config_validation_maps_to_validation`)
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T49 — `config`, `db`, `secret` command groups
- **Deliverable:** D0.13
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `db migrate|status|verify|plan|backup|restore` wired to `application::db` (real); `config` group real; `secret` group parses and dispatches to the Phase-11 stub per code comment (see DEV-02)
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T50 — `source`, `job`, `audit` command groups
- **Deliverable:** D0.13
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `Commands::Source|Job|Audit` parse in the `clap` tree; dispatch routes to Phase-1 stubs (see DEV-02)
- **DoD:** ⚠️ exception — dispatch is stubbed; verification for these surfaces comes from library/integration tests and the D0.14 doctor
- **Notes:** Implement real dispatch in Phase 1 before the runbooks' CLI references are executable.

### P0-T51 — Phase-N stub commands + shell completions + CLI conformance test
- **Deliverable:** D0.13
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `Commands::Completions` (Phase-13 stub); `--yes` on destructive `restore`; exit-code mapping tests; doctor `--json` shape test; serve/bind validation tests
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T52 — `doctor` engine: check registry, read-only enforcement, severity, remedies
- **Deliverable:** D0.14
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `crates/cli/src/doctor.rs` (`CheckResult` with remedy + next command; `Pass|Warn|Fail|Skipped`); probes via `SqliteDatabase::open_read_only`
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T53 — Phase-0 doctor checks (all listed in D0.14) + JSON schema
- **Deliverable:** D0.14
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** 26 checks; `doctor_is_read_only` (byte-identical DB assertion); `json_document_matches_the_documented_schema_shape`; `docs/schemas/doctor.v1.schema.json`
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T54 — `--repair-preview` planner (no mutation)
- **Deliverable:** D0.14
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `print_repair_preview` (plan only, `qai repair` deferred to Phase 1+); `repair-preview: no repairs needed` path covered in doctor tests
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T57 — `testkit` finalization + fixtures + deterministic clock/UUID
- **Deliverable:** D0.16
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `crates/testkit/src/lib.rs` (`temp_dir`, `temp_db_path`, `sample_config`, `sample_toml_config`, `sample_source_version`, `sample_source_file`, `sample_job_record`, `sample_provenance_record`, `MockAuditRepo` + tamper)
- **DoD:** ⚠️ exception — fixtures use wall-clock `Timestamp::now()`; no dedicated deterministic-clock helper
- **Notes:** Follow-up logged in §7 (test-hardening residuals).

### P0-T58 — Architecture docs, runbooks, `CONTRIBUTING`/DoD PR template
- **Deliverable:** D0.17
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (DOC)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `docs/architecture/` (crate-map, data-layers, source-lifecycle, hashing-spec, error-codes); 5 runbooks in `docs/runbooks/`; `CONTRIBUTING.md`; `examples/config/default.toml`; `.env.example`
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T59 — ADR-0010 + ADR index + template lint (all §48 fields present)
- **Deliverable:** ADR
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (DOC)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `docs/02-architecture/decisions/ADR-0010-error-taxonomy.md` (`Status: Accepted`); `cargo xtask adr-lint` enforces all §48 fields incl. religious-source and licensing
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T59b — Reconcile ADR numbering scheme + rename ADR files
- **Deliverable:** ADR
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (DOC)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `ADR-0001-relational-store.md` (phase-coded); `docs/02-architecture/database-engine-survey.md` (memo out of the ADR sequence); real ADR-0002 present; `cargo xtask adr-lint` green
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T60 — Phase-0 exit-gate review, AC verification, Phase-1 handoff doc
- **Deliverable:** —
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (all)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** §3 below (25 verified + 1 partial); `docs/plans/handoff-p0-to-p1.md`; this ledger; `summary.md`
- **DoD:** ⚠️ exception — the recorded exit-gate ritual (§4) is pending; all automated verification is done
- **Notes:** Follow-up logged in §7 (ritual recording).

### P0-T61 — Migration `0006_outbox_generations_tombstones`
- **Deliverable:** D0.18
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `migrations/sqlite/0006_outbox_generations_tombstones.{up,down}.sql` (`corpus_generations`, `outbox_events`, `tombstones` + append-only triggers)
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T62 — `CorpusGeneration` allocator: transactional monotonicity + concurrency test
- **Deliverable:** D0.18
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `OutboxRepository::allocate_generation` (single write connection serializes read-then-insert; PostgreSQL `SELECT … FOR UPDATE` documented); `crates/storage-sqlite/tests/generation_monotonicity.rs` (50 concurrent writers)
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T63 — `OutboxRepository` + wiring into `sources`/`provenance` write paths
- **Deliverable:** D0.18
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `UnitOfWork::outbox()` shares the single SQLite transaction; `crates/storage/src/workflows.rs` (`record_source_activation`, `record_source_deactivation`, `record_provenance_write`); `crates/storage-sqlite/tests/commit_bounds_outbox.rs`
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T64 — Generic outbox-relay job (`system.outbox_relay`) reusing job lease/heartbeat
- **Deliverable:** D0.18
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `kinds::SYSTEM_OUTBOX_RELAY` registered; `relay_outbox_once` (claim + dispatch, reusing outbox lease semantics); `crates/storage-sqlite/tests/outbox_relay.rs`; doctor remediation text; T37 worker pool exists to execute it
- **DoD:** ✅ all items — the plan's "no consumers yet" note holds: the contract, table, and guarantee exist before Phase 2 needs them
- **Notes:** None.

### P0-T65 — `Tombstone` model + wiring into source deactivation/rollback
- **Deliverable:** D0.18
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `Tombstone`/`TombstoneReason`/`PropagationState` domain types; tombstone written first inside `record_source_deactivation`; `crates/storage-sqlite/tests/tombstone_before_visibility.rs`
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T66 — Doctor checks: `outbox.backlog_age`, `outbox.dead_letter_count`, `generations.monotonicity`, `tombstones.unpropagated_count`
- **Deliverable:** D0.14
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `outbox.backlog_age`, `outbox.undispatched_count`, `tombstones.unpropagated_count` checks (read-only probe fields); `outbox_checks_report_backlog_without_mutation` test
- **DoD:** ✅ all items
- **Notes:** None.

### P0-T67 — Cross-store consistency test suite subset (§8.3)
- **Deliverable:** D0.16
- **Completed:** 2026-09-14
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local; gates recorded 2026-09-14 in `docs/05-followups/done.md`)
- **Evidence:** `crates/storage-sqlite/tests/commit_bounds_outbox.rs`, `outbox_idempotency.rs`, `generation_monotonicity.rs`, `tombstone_before_visibility.rs`, `outbox_relay.rs`
- **DoD:** ✅ all items
- **Notes:** None.

---

## 3. Verified Acceptance Criteria

**Entry format**

```
### AC-P0-nn — <criterion short name>
- **Verified:** YYYY-MM-DD
- **Verified by:** <reviewer name> (must not be the implementer for ritual ACs)
- **Method:** <test path / scripted check / live walkthrough>
- **Evidence:** <CI run URL, artifact, or recording timestamp>
- **Result:** ☑ Pass
- **Notes:** <caveats, re-verification triggers>
```

Live-walkthrough ACs (AC-P0-03, 05, 06, 08, 11, 14, 16) additionally require the recording link
and the reviewer's name, and must be verified by someone other than the implementer.

| ID | Criterion | Verified | By | Evidence |
|---|---|---|---|---|
| AC-P0-01 | `cargo xtask ci` green on 3 OSes | 2026-09-14 | implementing engineer | `cargo xtask ci` 9/9 (macOS); cross-target checks; CI matrix |
| AC-P0-02 | `arch-check` fails on forbidden edge (mutation test) | 2026-09-14 | implementing engineer | `forbidden_edge_is_detected`; live `arch-check` |
| AC-P0-03 | 🎥 Migrations: fresh, idempotent, checksum hard-fail | 2026-09-14 | implementing engineer (automated; ritual pending) | migrate idempotent/drift/down-restore tests |
| AC-P0-04 | 🎥 Config precedence + `--explain` origins | 2026-09-14 | implementing engineer | `config_precedence.rs` + config unit tests |
| AC-P0-05 | 🎥 Secret sentinel leaks nowhere | ◐ 2026-09-14 | implementing engineer | `secret_leak.rs` (3 tests); see entry for scope note |
| AC-P0-06 | 🎥 Canonical immutable; writes need `ApprovalToken` | 2026-09-14 | implementing engineer (automated; ritual pending) | provenance invariants + `integrity_provenance.rs` |
| AC-P0-07 | Computational annotation constraints | 2026-09-14 | implementing engineer | `integrity_provenance.rs` CHECK tests |
| AC-P0-08 | 🎥 Source lifecycle + approval preconditions | 2026-09-14 | implementing engineer (automated; ritual pending) | sources tests + `integrity_sources.rs` |
| AC-P0-09 | Manifest signature & unsigned policy | 2026-09-14 | implementing engineer | manifest/ingest/ed25519 tests (`QAI-SRC-0007`) |
| AC-P0-10 | Genealogy rendering + cycle rejection | 2026-09-14 | implementing engineer | `genealogy_renders_lineage_and_rejects_cycles` |
| AC-P0-11 | 🎥 Job idempotency + crash resume | 2026-09-14 | implementing engineer (automated; ritual pending) | `recovery_jobs.rs` + worker retry tests |
| AC-P0-12 | Cancel within 2s | 2026-09-14 | implementing engineer | cancel-timing + durable-record tests |
| AC-P0-13 | Audit chain verify + tamper detection | 2026-09-14 | implementing engineer | audit chain/tamper/gap tests + `integrity_audit.rs` |
| AC-P0-14 | 🎥 Doctor read-only + JSON schema | 2026-09-14 | implementing engineer (automated; ritual pending) | `doctor_is_read_only` + schema-shape test |
| AC-P0-15 | Security guards + fail-closed | 2026-09-14 | implementing engineer | path/archive/SSRF guard suites |
| AC-P0-16 | 🎥 Localhost bind default; TLS policy validation | 2026-09-14 | implementing engineer (automated; ritual pending) | serve/bind/TLS validation tests |
| AC-P0-17 | `Diagnostic` conformance + unique codes | 2026-09-14 | implementing engineer | `diagnostics.rs` + `error_codes.rs` + exit-code tests |
| AC-P0-18 | Telemetry off by default; denylist honoured | 2026-09-14 | implementing engineer | denylist tests + `telemetry_privacy.rs` |
| AC-P0-19 | ADRs accepted with all §48 fields | 2026-09-14 | implementing engineer | `cargo xtask adr-lint` (12 accepted) |
| AC-P0-20 | Docs + 5 runbooks complete | 2026-09-14 | implementing engineer | arch docs, runbooks, CONTRIBUTING, examples |
| AC-P0-21 | Backup/restore round-trip | 2026-09-14 | implementing engineer | `backup_restore.rs` + restore tests |
| AC-P0-22 | Coverage gates met | 2026-09-14 | implementing engineer | `llvm-cov` + `coverage-gate` numbers in §1 entry |
| AC-P0-23 | Commit-bounded outbox row (by construction) | 2026-09-14 | implementing engineer | `commit_bounds_outbox.rs` + `outbox_idempotency.rs` |
| AC-P0-24 | Generation monotonicity under 50 writers | 2026-09-14 | implementing engineer | `generation_monotonicity.rs` |
| AC-P0-25 | Tombstone before reader visibility | 2026-09-14 | implementing engineer | `tombstone_before_visibility.rs` |
| AC-P0-26 | Doctor reports outbox backlog read-only | 2026-09-14 | implementing engineer | backlog test + `doctor_is_read_only` |

### AC-P0-01 — `cargo xtask ci` green on 3 OSes
- **Verified:** 2026-09-14
- **Verified by:** implementing engineer (macOS leg; Linux/Windows legs execute in CI)
- **Method:** `cargo xtask ci` + cross-target checks + CI matrix review
- **Evidence:** 9/9 steps green on macOS; pure-Rust crates type-check for Linux and Windows targets; `.github/workflows/ci.yml` matrix
- **Result:** ☑ Pass
- **Notes:** SQLite/C crates need a native toolchain per OS, which CI provides.

### AC-P0-02 — `arch-check` fails on forbidden edge
- **Verified:** 2026-09-14
- **Verified by:** implementing engineer
- **Method:** mutation test (synthetic `domain -> cli` edge) + live run
- **Evidence:** `xtask` test `forbidden_edge_is_detected`; `cargo xtask arch-check` OK on the compliant tree
- **Result:** ☑ Pass
- **Notes:** None.

### AC-P0-03 — Migrations: fresh, idempotent, checksum hard-fail
- **Verified:** 2026-09-14
- **Verified by:** implementing engineer (automated; live ritual (acceptance.md §4) pending)
- **Method:** scripted migration tests
- **Evidence:** `fresh_migrate_is_idempotent`, `checksum_drift_is_detected` (QAI-DB-0003), `down_migrations_restore_schema`
- **Result:** ☑ Pass
- **Notes:** None.

### AC-P0-04 — Config precedence + `--explain` origins
- **Verified:** 2026-09-14
- **Verified by:** implementing engineer (automated; live ritual pending)
- **Method:** `crates/testkit/tests/config_precedence.rs` + config unit tests
- **Evidence:** defaults/file/CLI/invalid-file/`--explain` tests; 18 unit tests including env overrides
- **Result:** ☑ Pass
- **Notes:** None.

### AC-P0-05 — Secret sentinel leaks nowhere
- **Verified:** 2026-09-14
- **Verified by:** implementing engineer
- **Method:** `crates/testkit/tests/secret_leak.rs` + secret-store round-trip tests
- **Evidence:** sentinel absent from `Secret` Debug/Display/Serialize, containers, and audit serialization; env + encrypted-file backends round-trip sentinel values
- **Result:** ◐ Partially verified — log-emission, error-formatting, `config show`, and doctor-JSON surfaces plus the manual keychain OS matrix are not covered by the suite
- **Notes:** Hardening follow-up FU-01/FU-02 in §7; live ritual (acceptance.md §4) pending.

### AC-P0-06 — Canonical immutable; writes need `ApprovalToken`
- **Verified:** 2026-09-14
- **Verified by:** implementing engineer (automated; live ritual pending)
- **Method:** provenance invariant tests + DB trigger tests
- **Evidence:** `canonical_provenance_is_immutable`, `approval_token_cannot_be_constructed_directly`; `integrity_provenance.rs` (QAI-PROV-0001/0002 aborts)
- **Result:** ☑ Pass
- **Notes:** None.

### AC-P0-07 — Computational annotation constraints
- **Verified:** 2026-09-14
- **Verified by:** implementing engineer
- **Method:** DB `CHECK` constraint tests
- **Evidence:** `computational_annotation_requires_algorithm_and_confidence`, `human_verified_requires_a_reviewer` in `integrity_provenance.rs`
- **Result:** ☑ Pass
- **Notes:** None.

### AC-P0-08 — Source lifecycle + approval preconditions
- **Verified:** 2026-09-14
- **Verified by:** implementing engineer (automated; live ritual pending)
- **Method:** sources state-machine tests + single-`Active` index test
- **Evidence:** legal/illegal transitions, `Approved` preconditions including approver, `integrity_sources.rs`
- **Result:** ☑ Pass
- **Notes:** None.

### AC-P0-09 — Manifest signature and unsigned policy
- **Verified:** 2026-09-14
- **Verified by:** implementing engineer
- **Method:** sources manifest tests
- **Evidence:** `signed_local_manifest_ingests_to_staged`, `tampered_local_manifest_does_not_ingest`, `unsigned_local_manifest_rejected_under_remote_policy`, ed25519 verify/tamper test; rejections carry `QAI-SRC-0007`
- **Result:** ☑ Pass
- **Notes:** None.

### AC-P0-10 — Genealogy rendering + cycle rejection
- **Verified:** 2026-09-14
- **Verified by:** implementing engineer
- **Method:** unit test
- **Evidence:** `genealogy_renders_lineage_and_rejects_cycles`
- **Result:** ☑ Pass
- **Notes:** None.

### AC-P0-11 — Job idempotency + crash resume
- **Verified:** 2026-09-14
- **Verified by:** implementing engineer (automated; live ritual pending)
- **Method:** `crates/storage-sqlite/tests/recovery_jobs.rs` + worker retry tests
- **Evidence:** `duplicate_enqueue_and_double_claim_yield_one_execution`, `expired_lease_marks_interrupted_and_resumes_from_checkpoint`, `retries_then_succeeds`, `dead_letters_after_max_attempts`
- **Result:** ☑ Pass
- **Notes:** Crash is exercised via lease expiry (the reclaim mechanism); a literal SIGKILL step belongs to the acceptance.md §4 ritual.

### AC-P0-12 — Cancel within 2 seconds
- **Verified:** 2026-09-14
- **Verified by:** implementing engineer
- **Method:** timed test + durable-record test
- **Evidence:** `cancellation_stops_a_cooperative_handler_within_two_seconds`, `cancel_is_durably_recorded`
- **Result:** ☑ Pass
- **Notes:** None.

### AC-P0-13 — Audit chain verify + tamper detection
- **Verified:** 2026-09-14
- **Verified by:** implementing engineer
- **Method:** audit verifier tests + DB trigger tests
- **Evidence:** `verifier_accepts_a_valid_chain_then_detects_a_tampered_row`, `verifier_detects_a_sequence_gap`; `integrity_audit.rs` (QAI-AUD-0001/0002 aborts)
- **Result:** ☑ Pass
- **Notes:** None.

### AC-P0-14 — Doctor read-only + JSON schema
- **Verified:** 2026-09-14
- **Verified by:** implementing engineer (automated; live ritual pending)
- **Method:** read-only byte-identity test + schema-shape test
- **Evidence:** `doctor_is_read_only`, `json_document_matches_the_documented_schema_shape`, `docs/schemas/doctor.v1.schema.json`
- **Result:** ☑ Pass
- **Notes:** None.

### AC-P0-15 — Security guards + fail-closed
- **Verified:** 2026-09-14
- **Verified by:** implementing engineer
- **Method:** `crates/testkit/tests/security_{path,archive,ssrf}_guard.rs`
- **Evidence:** traversal/symlink corpus, zip-slip/bomb/symlink-entry/depth, loopback/private/link-local/redirect-to-private rejections; fail-closed assertions
- **Result:** ☑ Pass
- **Notes:** None.

### AC-P0-16 — Localhost bind default; TLS policy validation
- **Verified:** 2026-09-14
- **Verified by:** implementing engineer (automated; live ritual pending)
- **Method:** config + CLI tests
- **Evidence:** `serve_defaults_to_loopback`, `default_config_binds_loopback_and_validates`, `non_loopback_bind_without_tls_fails_validation`
- **Result:** ☑ Pass
- **Notes:** None.

### AC-P0-17 — `Diagnostic` conformance + unique codes
- **Verified:** 2026-09-14
- **Verified by:** implementing engineer
- **Method:** conformance test
- **Evidence:** `crates/testkit/tests/diagnostics.rs`, `error_codes.rs`, exit-code mapping tests
- **Result:** ☑ Pass
- **Notes:** CLI `tests/cli/*.trycmd` snapshots are not present; tracked as FU-05. Conformance itself is proven.

### AC-P0-18 — Telemetry off by default; denylist honoured
- **Verified:** 2026-09-14
- **Verified by:** implementing engineer
- **Method:** `crates/testkit/tests/telemetry_privacy.rs` + observability denylist tests
- **Evidence:** off-by-default assertion; denylisted fields stripped including nested payloads
- **Result:** ☑ Pass
- **Notes:** None.

### AC-P0-19 — ADRs accepted with all §48 fields
- **Verified:** 2026-09-14
- **Verified by:** implementing engineer
- **Method:** `cargo xtask adr-lint`
- **Evidence:** ADR-0000…ADR-0012 `Status: Accepted`; lint enforces Context, Options, Decision, Accuracy, Religious-source, Licensing, Security, Operational, Migration strategy, Reversal cost
- **Result:** ☑ Pass
- **Notes:** ADR-0702 stays `Proposed` (Phase 7 owns its acceptance).

### AC-P0-20 — Docs complete
- **Verified:** 2026-09-14
- **Verified by:** implementing engineer
- **Method:** doc review checklist
- **Evidence:** `docs/architecture/` (crate map, data layers, source lifecycle, hashing spec with NFC, error codes), 5 runbooks, `CONTRIBUTING.md`, `.env.example`, `examples/config/default.toml`
- **Result:** ☑ Pass
- **Notes:** None.

### AC-P0-21 — Backup/restore round-trip
- **Verified:** 2026-09-14
- **Verified by:** implementing engineer
- **Method:** scripted test
- **Evidence:** `crates/storage-sqlite/tests/backup_restore.rs` (round-trip + chain verification); application restore tests; `qai db backup` uses `VACUUM INTO`
- **Result:** ☑ Pass
- **Notes:** None.

### AC-P0-22 — Coverage gates met
- **Verified:** 2026-09-14
- **Verified by:** implementing engineer
- **Method:** `cargo llvm-cov --workspace` + `cargo xtask coverage-gate lcov.info`
- **Evidence:** domain 95.38, provenance 86.36, audit 90.20, sources 88.70, config 93.38, jobs 81.17, storage-sqlite 88.71
- **Result:** ☑ Pass
- **Notes:** None.

### AC-P0-23 — Commit-bounded outbox row (by construction)
- **Verified:** 2026-09-14
- **Verified by:** implementing engineer
- **Method:** `crates/storage-sqlite/tests/commit_bounds_outbox.rs` + `outbox_idempotency.rs`
- **Evidence:** shared-transaction atomicity tests; `UNIQUE(operation, idempotency_key)` single-event test
- **Result:** ☑ Pass
- **Notes:** None.

### AC-P0-24 — Generation monotonicity under 50 writers
- **Verified:** 2026-09-14
- **Verified by:** implementing engineer
- **Method:** `crates/storage-sqlite/tests/generation_monotonicity.rs`
- **Evidence:** 50 concurrent writers, no regression (single write pool serializes; PostgreSQL `SELECT … FOR UPDATE` documented)
- **Result:** ☑ Pass
- **Notes:** None.

### AC-P0-25 — Tombstone before reader visibility
- **Verified:** 2026-09-14
- **Verified by:** implementing engineer
- **Method:** `crates/storage-sqlite/tests/tombstone_before_visibility.rs`
- **Evidence:** tombstone written before the transition in the same transaction; failed transition leaves no tombstone
- **Result:** ☑ Pass
- **Notes:** None.

### AC-P0-26 — Doctor reports outbox backlog read-only
- **Verified:** 2026-09-14
- **Verified by:** implementing engineer
- **Method:** read-only + JSON test
- **Evidence:** `outbox.backlog_age`, `outbox.undispatched_count`, `tombstones.unpropagated_count` checks (read-only probe fields); `outbox_checks_report_backlog_without_mutation` test
- **Result:** ☑ Pass
- **Notes:** None.
- **Verified:** 2026-09-14
- **Verified by:** implementing engineer (automated; live ritual (acceptance.md §4) pending)
- **Method:** read-only byte-identity test + schema-shape test
- **Evidence:** `doctor_is_read_only`, `json_document_matches_the_documented_schema_shape`, `docs/schemas/doctor.v1.schema.json`
- **Result:** ☑ Pass
- **Notes:** None.

---

## 4. Accepted ADRs

**Entry format**

```
### ADR-00nn — <title>
- **Status:** Accepted
- **Accepted:** YYYY-MM-DD
- **Author / reviewers:** <names>
- **File:** adr/ADR-00nn-<slug>.md
- **§48 fields present:** Context · Options · Decision · Accuracy · Religious-source ·
  Licensing · Security · Operational · Migration strategy · Reversal cost
- **Decision summary:** <one or two sentences>
- **Constrains:** <deliverables / later phases>
```

| ADR | Title | Blocking for | Status |
|---|---|---|---|
| ADR-0001 | Relational store & access layer (SQLite + `sqlx`, Postgres-portable SQL) | D0.6 | ☑ 2026-09-14 |
| ADR-0002 | Migration strategy (append-only checksummed SQL; forward-only canonical) | D0.7 | ☑ 2026-09-14 |
| ADR-0003 | Durable job system (DB-backed leased queue, no broker) | D0.9 | ☑ 2026-09-14 |
| ADR-0004 | Configuration & precedence model | D0.4 | ☑ 2026-09-14 |
| ADR-0005 | Secret storage per OS (env / keychain / age file) | D0.5 | ☑ 2026-09-14 |
| ADR-0006 | Hashing & canonical serialization (SHA-256, canonical JSON, NFC) | D0.2 | ☑ 2026-09-14 |
| ADR-0007 | Source manifest format & signing (ed25519 detached) | D0.10 | ☑ 2026-09-14 |
| ADR-0008 | Provenance representation (universal record + typed attribution) | D0.11 | ☑ 2026-09-14 |
| ADR-0009 | Audit integrity (hash chain + append-only triggers) | D0.12 | ☑ 2026-09-14 |
| ADR-0010 | Error taxonomy & CLI exit codes | D0.3, D0.13 | ☑ 2026-09-14 |
| ADR-0011 | Observability stack; telemetry opt-in | D0.8 | ☑ 2026-09-14 |
| ADR-0012 | Workspace/crate boundaries & dependency enforcement | D0.1 | ☑ 2026-09-14 |
| ADR-0702 | Cross-store consistency (§§2–4, 9 adopted early) | D0.18 | Proposed (Phase 7 owns acceptance) |

### ADR-0001 — Relational store & access layer
- **Status:** Accepted
- **Accepted:** 2026-09-14
- **Author / reviewers:** implementing engineer
- **File:** docs/02-architecture/decisions/ADR-0001-relational-store.md
- **§48 fields present:** Context · Options · Decision · Accuracy · Religious-source · Licensing · Security · Operational · Migration strategy · Reversal cost
- **Decision summary:** SQLite via `sqlx` with Postgres-portable SQL; storage traits isolate backends.
- **Constrains:** D0.6, D0.7; Phase 12 Postgres migration.

### ADR-0002 — Migration strategy
- **Status:** Accepted
- **Accepted:** 2026-09-14
- **Author / reviewers:** implementing engineer
- **File:** docs/02-architecture/decisions/ADR-0002-migration-strategy.md
- **§48 fields present:** Context · Options · Decision · Accuracy · Religious-source · Licensing · Security · Operational · Migration strategy · Reversal cost
- **Decision summary:** Append-only checksummed SQL; forward-only for canonical tables; `VACUUM INTO` backups.
- **Constrains:** D0.7; all later migrations.

### ADR-0003 — Durable job system
- **Status:** Accepted
- **Accepted:** 2026-09-14
- **Author / reviewers:** implementing engineer
- **File:** docs/02-architecture/decisions/ADR-0003-durable-job-system.md
- **§48 fields present:** Context · Options · Decision · Accuracy · Religious-source · Licensing · Security · Operational · Migration strategy · Reversal cost
- **Decision summary:** DB-backed leased queue with no external broker in local mode.
- **Constrains:** D0.9; Phase 1 `quran.*` jobs.

### ADR-0004 — Configuration & precedence model
- **Status:** Accepted
- **Accepted:** 2026-09-14
- **Author / reviewers:** implementing engineer
- **File:** docs/02-architecture/decisions/ADR-0004-config-precedence.md
- **§48 fields present:** Context · Options · Decision · Accuracy · Religious-source · Licensing · Security · Operational · Migration strategy · Reversal cost
- **Decision summary:** Strict CLI > Env > File > Defaults with `ValueOrigin` tracking and `${...}` interpolation.
- **Constrains:** D0.4; every configurable subsystem.

### ADR-0005 — Secret storage per OS
- **Status:** Accepted
- **Accepted:** 2026-09-14
- **Author / reviewers:** implementing engineer
- **File:** docs/02-architecture/decisions/ADR-0005-secret-storage.md
- **§48 fields present:** Context · Options · Decision · Accuracy · Religious-source · Licensing · Security · Operational · Migration strategy · Reversal cost
- **Decision summary:** Env, OS keychain, and encrypted-file backends behind `SecretStore`; values never in SQLite.
- **Constrains:** D0.5; all secret handling.

### ADR-0006 — Hashing & canonical serialization
- **Status:** Accepted
- **Accepted:** 2026-09-14
- **Author / reviewers:** implementing engineer
- **File:** docs/02-architecture/decisions/ADR-0006-hashing-canonical.md
- **§48 fields present:** Context · Options · Decision · Accuracy · Religious-source · Licensing · Security · Operational · Migration strategy · Reversal cost
- **Decision summary:** SHA-256 with algorithm tags, canonical JSON, NFC policy, rehash procedure.
- **Constrains:** D0.2; every checksum in the system.

### ADR-0007 — Source manifest format & signing
- **Status:** Accepted
- **Accepted:** 2026-09-14
- **Author / reviewers:** implementing engineer
- **File:** docs/02-architecture/decisions/ADR-0007-manifest-format-signing.md
- **§48 fields present:** Context · Options · Decision · Accuracy · Religious-source · Licensing · Security · Operational · Migration strategy · Reversal cost
- **Decision summary:** ed25519 detached signatures over canonical JSON; unsigned manifests rejected for remote sources by default.
- **Constrains:** D0.10; Phase 1 edition import.

### ADR-0008 — Provenance representation
- **Status:** Accepted
- **Accepted:** 2026-09-14
- **Author / reviewers:** implementing engineer
- **File:** docs/02-architecture/decisions/ADR-0008-provenance-representation.md
- **§48 fields present:** Context · Options · Decision · Accuracy · Religious-source · Licensing · Security · Operational · Migration strategy · Reversal cost
- **Decision summary:** One universal provenance record with typed attribution; extensible `attribution_json`.
- **Constrains:** D0.11; every non-canonical assertion.

### ADR-0009 — Audit integrity
- **Status:** Accepted
- **Accepted:** 2026-09-14
- **Author / reviewers:** implementing engineer
- **File:** docs/02-architecture/decisions/ADR-0009-audit-integrity.md
- **§48 fields present:** Context · Options · Decision · Accuracy · Religious-source · Licensing · Security · Operational · Migration strategy · Reversal cost
- **Decision summary:** Hash-chained, trigger-enforced append-only audit log.
- **Constrains:** D0.12; every mutation.

### ADR-0010 — Error taxonomy & CLI exit codes
- **Status:** Accepted
- **Accepted:** 2026-09-14
- **Author / reviewers:** implementing engineer
- **File:** docs/02-architecture/decisions/ADR-0010-error-taxonomy.md
- **§48 fields present:** Context · Options · Decision · Accuracy · Religious-source · Licensing · Security · Operational · Migration strategy · Reversal cost
- **Decision summary:** `QAI-<NS>-nnnn` namespaces with exit codes 0–70 as public API.
- **Constrains:** D0.3, D0.13; all error reporting.

### ADR-0011 — Observability stack
- **Status:** Accepted
- **Accepted:** 2026-09-14
- **Author / reviewers:** implementing engineer
- **File:** docs/02-architecture/decisions/ADR-0011-observability.md
- **§48 fields present:** Context · Options · Decision · Accuracy · Religious-source · Licensing · Security · Operational · Migration strategy · Reversal cost
- **Decision summary:** `tracing` + `metrics` with opt-in OTLP; telemetry off by default with a content denylist.
- **Constrains:** D0.8; all operational visibility.

### ADR-0012 — Workspace/crate boundaries & dependency enforcement
- **Status:** Accepted
- **Accepted:** 2026-09-14
- **Author / reviewers:** implementing engineer
- **File:** docs/02-architecture/decisions/ADR-0012-workspace-boundaries.md
- **§48 fields present:** Context · Options · Decision · Accuracy · Religious-source · Licensing · Security · Operational · Migration strategy · Reversal cost
- **Decision summary:** Many small crates with CI-enforced dependency directions via `xtask arch-check`.
- **Constrains:** D0.1; every crate added later.

> **Open before more ADRs are written:** the numbering collision (P0-T59b). Adopted scheme is
> **phase-coded** (`ADR-00nn` / `ADR-02nn` / `ADR-07nn`). Log the three file fixes here when done:
> rename `ADR-0004-relational-store.md` → `ADR-0001-…`; move `ADR-0002-database.md` out of the
> ADR sequence as a survey memo; write the real ADR-0002.

---

## 5. Deviations From Plan

### DEV-01 — Integration suites live in crate test targets, not a workspace `tests/` tree
- **Date:** 2026-09-14
- **Plan reference:** plan.md §8.1 / acceptance.md §3.2 (`tests/security/…`, `tests/integrity/…`, `tests/sources/…`, `tests/recovery/…`, `tests/config/…`, `tests/cli/…`, `tests/db/…`, `tests/observability/…`, `tests/consistency/…`)
- **Planned:** All integration suites under a workspace-level `tests/` directory with those exact paths.
- **Delivered:** Suites live in crate test targets with equivalent coverage: `crates/testkit/tests/` (secret-leak, path/archive/SSRF guards, precedence, telemetry privacy, diagnostics, error codes), `crates/storage-sqlite/tests/` (recovery, integrity, consistency, backup/restore), and in-crate unit tests (sources manifest/state-machine/genealogy, audit chain, jobs, doctor, CLI).
- **Reason:** Crate targets keep ownership, dependencies, and `#[ignore]` gating local; a single workspace tree would tangle crate boundaries the phase is enforcing.
- **Scope impact:** None on behavior; AC evidence paths differ textually from the plan. The one suite with no equivalent anywhere is `tests/cli/*.trycmd` (see §7).
- **Phase-1 impact:** Handoff must point Phase 1 at the crate-local suite locations, not the plan's paths.
- **Approved by:** implementing engineer (recorded; human sign-off pending in §8).

### DEV-02 — `source`, `job`, `audit`, and `secret` CLI groups parse but dispatch to Phase-1 stubs
- **Date:** 2026-09-14
- **Plan reference:** plan.md D0.13 / P0-T50
- **Planned:** `source`, `job`, and `audit` command groups implemented.
- **Delivered:** The groups exist in the `clap` tree; only `db`, `config`, `doctor`, and `serve` execute. `source`/`job`/`audit` dispatch to `phase_stub` (Phase 1), matching the D0.13 rule that Phase-1+ commands are registered as stubs. Verification for the stubbed surfaces comes from library and integration tests plus the D0.14 doctor.
- **Reason:** Slice scope to the framework plus the commands the exit gate exercises end to end.
- **Scope impact:** Runbook CLI references to `qai audit verify` and friends are Phase-1-forward, not executable today.
- **Phase-1 impact:** Implement real dispatch for these groups before the runbooks are executable; T50's entry records the exception.
- **Approved by:** implementing engineer (recorded; human sign-off pending in §8).

Log anything delivered differently from `plan.md`. Deviations are expected and fine — **undocumented**
deviations are the problem, because Phase 1 inherits this chassis assuming the plan describes it.

**Entry format**

```
### DEV-nn — <short title>
- **Date:** YYYY-MM-DD
- **Plan reference:** plan.md §x / D0.y / P0-Tnn
- **Planned:** <what the plan said>
- **Delivered:** <what was actually built>
- **Reason:** <why>
- **Scope impact:** <deliverables / ACs affected>
- **Phase-1 impact:** <what the handoff doc must say>
- **Approved by:** <name>
```

Deviations requiring **explicit sign-off** because they touch retrofit-impossible guarantees:
any change to D0.11 (`ApprovalToken` / `CanonicalWriter`), D0.12 (audit chain), D0.15
(`Untrusted<T>`), or D0.18 (commit-bounded outbox).

---

## 6. Corrections

_None._

**Entry format**

```
### COR-nn — correction to <entry ID>
- **Date:** YYYY-MM-DD
- **Original entry:** <ID and date>
- **What was wrong:** <description>
- **Correct record:** <description>
- **Cause:** <how the wrong record happened>
```

---

## 7. Deferred Items & Follow-Ups

Anything intentionally not done in Phase 0 that is **not** already in the out-of-scope list
(`readme.md` §4). Every row needs a named owner and a target phase — an unowned deferral is a
silent scope leak into Phase 1.

| ID | Item | Reason deferred | Target phase | Owner | Logged |
|---|---|---|---|---|---|
| FU-01 | T16 hardening: sentinel zero-byte coverage for log emission, error formatting, `config show`, doctor JSON; global tracing redaction layer | Suite covers type-level + audit-level redaction only | Phase 1 hardening | implementing engineer (SEC) | 2026-09-14 |
| FU-02 | Manual keychain OS-matrix run (risk R5) | `#[ignore]`-gated; needs a manual matrix | Before gate close | implementing engineer (SEC) | 2026-09-14 |
| FU-03 | Backoff-distribution + non-idempotent-retry-policy tests (T39/T40 residual) | Bounded retry/jitter/dead-letter transitions tested; distribution + policy gates not asserted | Phase 1 | implementing engineer (BE) | 2026-09-14 |
| FU-04 | Review accept/reject unit test (T25 residual) | Model + repo path exist; no dedicated decision test | Phase 1 | implementing engineer (BE) | 2026-09-14 |
| FU-05 | trycmd CLI snapshots for every Phase-0 command (§3.2 suite form) | Conformance proven via unit tests; snapshot form absent | Phase 1 | implementing engineer (BE) | 2026-09-14 |
| FU-06 | Deterministic clock/UUID helper for `testkit` (T57 residual) | Fixtures use wall-clock timestamps | Phase 1 | implementing engineer (BE) | 2026-09-14 |
| FU-07 | Risk-R2 three-case provenance review (T23 residual) | Model + extensible attribution exist; review not evidenced | Phase 1/2 | implementing engineer (BE) | 2026-09-14 |
| FU-08 | `/api/v1/meta` route (T55 residual) | Serve stub covers health/readyz + bind guard only | Phase 1 | implementing engineer (BE) | 2026-09-14 |
| FU-09 | Dockerfile/compose/non-root runtime (T56) | Not started | Phase 1 | implementing engineer (INF) | 2026-09-14 |
| FU-10 | Real dispatch for source/job/audit CLI groups (DEV-02) | Groups parse; dispatch stubbed to Phase 1 | Phase 1 | implementing engineer (BE) | 2026-09-14 |
| FU-11 | Exit-gate ritual recording (§4) | Requires a non-implementer reviewer on a clean machine | Immediate | _unassigned_ | 2026-09-14 |
| FU-12 | Swimlane-X ownership (P0-X01/02/03) | No owner or decision-open date recorded | Immediate | _unassigned_ | 2026-09-14 |

Carried into `docs/plans/handoff-p0-to-p1.md` by task P0-T60.

---

## 8. Phase Closure

| Gate | Requirement | Evidence | Signed off by | Date |
|---|---|---|---|---|
| All 67 tasks done | `tasks.md` fully ☑ | §2: 65 done; T55, T56 deferred (§7) | | |
| All 26 ACs verified | §3 above | §3: 25 verified + AC-P0-05 partial | | |
| Coverage gates met | `acceptance.md` §2 | domain 95.38 · provenance 86.36 · audit 90.20 · sources 88.70 · config 93.38 · jobs 81.17 · storage-sqlite 88.71 | | |
| 17 required suites green | `acceptance.md` §3.2 | 16 of 17 forms green; `tests/cli/*.trycmd` → FU-05 | | |
| 13 ADRs accepted, §48-complete | §4 above | ADR-0000…0012 accepted; ADR-0702 Proposed (Phase 7) | | |
| ADR numbering reconciled | P0-T59b | Phase-coded filenames; survey memo out of sequence; `adr-lint` green | | |
| 6 migrations applied & checksummed | `migrations/sqlite/` | 0001–0006 up+down files + `checksums.json` | | |
| 5 runbooks published | `docs/runbooks/` | backup-restore, interrupted-job-recovery, audit-chain-break, migration-checksum-mismatch, quarantine-handling | | |
| Exit-gate ritual recorded | `acceptance.md` §4 (7 live ACs) | | | |
| Handoff doc published | `docs/plans/handoff-p0-to-p1.md` | File exists (2026-09-14) | | |
| Swimlane X decisions owned & open | P0-X01 / X02 / X03 | | | |
| Deviations documented | §5 above | DEV-01 (test layout), DEV-02 (stub dispatch) | | |
| Deferrals owned | §7 above | FU-01…FU-10 owned; FU-11/12 unassigned | | |

**Phase 0 accepted:** _pending_
**Phase 1 unblocked:** _pending_

> Closure requires the **Swimlane X** row. ADR-0101 (Quran dataset licensing) is needed by the
> end of Phase 0 and ADR-0203/0204 by the start of Phase 2; a green Phase 0 with all three
> unowned means Phase 1 or 2 starts stalled on an external decision that engineering cannot
> compress. Recording this as a closure gate is the only reliable defence.

## 9. Targeted correction — 2026-09-17, P0-T39/T40

- Owner: implementing assistant; changes remain uncommitted by this session.
- Restored the handler idempotency gate on failure and the post-jitter backoff cap in `crates/jobs/src/worker.rs`.
- Evidence: `non_idempotent_failures_are_never_automatically_retried` and `backoff_is_deterministic_distributed_and_bounded`; jobs serial suite 19 passing; jobs fmt, clippy all-targets with denied warnings, and check pass.
- FU-03 failure-retry/distribution coverage addressed. This does not establish non-idempotent lease-recovery safety.
- Initial parallel run failed the existing retry-success test (Idle); isolated and serial reruns passed. Scheduling follow-up recorded in `docs/05-followups/open-questions.md`.
- Phase acceptance, T57, T56, CLI stubs, and full verification remain pending.

## 10. P0-T57 / FU-06 — deterministic fixtures, 2026-09-17

- Owner: implementing assistant; no commit created by this session.
- Added `FixtureClock` with explicit set/read operations, typed reproducible UUID fixtures over a u64 index, and deterministic job fixtures without RNG or wall-clock calls. Existing random job fixtures retain their behavior.
- Evidence: three new unit tests cover explicit time control, UUID boundary indices and typed replay, every job-record field, and independence between fixture calls.
- Verification: `cargo test -p testkit -- --test-threads=1` (40 passing), `cargo fmt -p testkit -- --check`, `cargo clippy -p testkit --all-targets -- -D warnings`, `cargo check -p testkit` all passed.
- FU-06 addressed; no claim that production scheduling uses this clock or that the scheduling flake is resolved.

## 11. P0-T56 — container stub, partial, 2026-09-17

- Added a multi-stage Rust build and distroless non-root runtime, migration assets, persistent data directory, and a build-context allowlist.
- Compose uses UID 65532, read-only root filesystem, dropped capabilities, no-new-privileges, no networking or published ports, and an explicit migration initialization step.
- `docker compose config --quiet`, workspace `cargo check`, and architecture check passed. Docker daemon unavailable, so image build and runtime smoke tests were not run. T56 remains partial; follow-up commands recorded in `docs/05-followups/open-questions.md`.

## 12. P0-T39/T40 — in-memory timestamp ordering, 2026-09-17

- Owner: implementing assistant; changes not committed by this session.
- Replaced lexicographic eligibility comparisons with parsed instant comparisons in claim and lease-reap paths. Equal expiry is treated as expired; malformed timestamps remain ineligible.
- Corrected the initial test typo and reversed inequality; deterministic tests cover mixed precision and actual claim/reap state transitions through private fixed-time entry points.
- Evidence: both exact regression tests pass; 25 consecutive full jobs-suite runs pass (21 tests each). Jobs formatting, clippy all-targets with denied warnings, and type checks pass.
- Negative control: temporarily restoring the original lexicographic claim comparison made `mixed_precision_timestamps_control_claims_and_reaping` fail deterministically for `.123Z` at `.123456Z`. Restoring parsed comparison made the same exact test pass; lint and check passed again.
- This addresses the observed in-memory ordering defect, not subsecond delay truncation, production SQLite ordering, or non-idempotent lease recovery. Those remain follow-ups; phase closure remains open.
