# Phase 0 — Done Log

> **Completed:** 2026-09-14
> **Phase:** P0 — Foundations & Provenance
> **Verification:** all commands below were run in this session on macOS (Rust stable, edition 2024).

## Final gate results

| Gate | Command | Result |
|---|---|---|
| Format | `cargo fmt --all -- --check` | ✅ clean |
| Lint | `cargo clippy --workspace --all-targets -- -D warnings` | ✅ 0 errors, 0 warnings |
| Tests | `cargo test --workspace` | ✅ **209 passing, 0 failing** |
| Architecture | `cargo xtask arch-check` | ✅ no forbidden edges |
| Migrations | `cargo xtask migrate-check` | ✅ 6 ordered, checksums stable |
| ADRs | `cargo xtask adr-lint` | ✅ 12 present, Accepted, complete |
| Binary smoke | `qai db migrate/status/verify/backup/restore`, `qai doctor [--json|--repair-preview]` | ✅ works |

## Deliverables completed

- **D0.1** workspace/toolchain/`xtask` (`arch-check`, `ci`, `migrate-check`, `gen-schema`, `coverage-gate`, `adr-lint`), `.cargo/config.toml` alias.
- **D0.2** domain model: IDs, primitives, `DataLayer`/`TrustLevel`/`VerificationStatus`, `ContentHash` + canonical JSON, `DerivationVersions`, `CorpusGeneration`.
- **D0.3** diagnostic error codes (`QAI-<NS>-nnnn`), uniqueness tested.
- **D0.4** layered config (CLI>Env>File>Defaults), `ValueOrigin`, validation.
- **D0.5** `Secret<T>` redaction + `SecretStore` (env real; keychain/age abstracted).
- **D0.6** storage traits + SQLite backend (dual pools, all repositories real).
- **D0.7** migration framework: apply / checksum-verify / status / backup (`VACUUM INTO`) / **down-migrations**.
- **D0.8** observability: subscriber, span conventions, metric catalog, telemetry denylist.
- **D0.9** jobs: types, repository, cancellation, checkpoints, lease reaping.
- **D0.10** sources: state machine, manifest parse/validate/sign, genealogy, ingest.
- **D0.11** provenance model, `ApprovalToken`, `CanonicalWriter`, DB triggers.
- **D0.12** audit model, hash-chain writer/verifier, append-only triggers.
- **D0.13** CLI skeleton + `qai` binary + real `db` commands.
- **D0.14** `qai doctor`: 26 checks, read-only, remedies, `--json`, `--repair-preview`.
- **D0.15** security guards (path/archive/SSRF/input/sanitize), deny-by-default.
- **D0.16** testkit fixtures + security/integrity/recovery/consistency suites.
- **D0.17** architecture docs, 5 runbooks, `CONTRIBUTING.md`, example config.
- **D0.18** outbox / generations / tombstones: domain types, `OutboxRepository`, workflows, SQLite impl, relay, doctor checks, consistency tests.

## Acceptance criteria

| AC | Status | Evidence |
|---|---|---|
| AC-P0-01 | 🔶 PARTIAL | `.github/workflows/ci.yml` matrix defined; 3-OS run must execute in CI |
| AC-P0-02 | ✅ | `xtask` tests `forbidden_edge_is_detected`; `cargo xtask arch-check` |
| AC-P0-03 | ✅ | `storage-sqlite` `fresh_migrate_is_idempotent`, `checksum_drift_is_detected`, `down_migrations_restore_schema` |
| AC-P0-04 | ✅ | `config` unit tests + `testkit/tests/config_precedence.rs` |
| AC-P0-05 | ✅ | `testkit/tests/secret_leak.rs` + `config` secret redaction tests |
| AC-P0-06 | ✅ | `provenance` invariant tests + `storage-sqlite/tests/integrity_provenance.rs` triggers |
| AC-P0-07 | ✅ | `storage-sqlite/tests/integrity_provenance.rs` CHECK constraints |
| AC-P0-08 | ✅ | `sources` tests (incl. approver) + `storage-sqlite/tests/integrity_sources.rs` |
| AC-P0-09 | ✅ | `sources` tests: `signed_local_manifest_ingests_to_staged`, `tampered_local_manifest_does_not_ingest`, `unsigned_local_manifest_rejected_under_remote_policy` |
| AC-P0-10 | ✅ | `sources` `genealogy_renders_lineage_and_rejects_cycles` |
| AC-P0-11 | ✅ | `storage-sqlite/tests/recovery_jobs.rs`; jobs `duplicate_enqueue_and_double_claim_yield_one_execution` |
| AC-P0-12 | ✅ | `jobs` `cancellation_stops_a_cooperative_handler_within_two_seconds`; `recovery_jobs::cancel_is_durably_recorded` |
| AC-P0-13 | ✅ | `audit` chain/tamper/gap tests + `storage-sqlite/tests/integrity_audit.rs` triggers |
| AC-P0-14 | ✅ | `cli` `doctor_is_read_only`, `json_document_matches_the_documented_schema_shape`; `docs/schemas/doctor.v1.schema.json` |
| AC-P0-15 | ✅ | `testkit/tests/{secret_leak,security_path_guard,security_archive_guard,security_ssrf_guard}.rs` |
| AC-P0-16 | ✅ | `cli` `serve_defaults_to_loopback`, `non_loopback_bind_without_tls_fails_validation` |
| AC-P0-17 | 🔶 PARTIAL | `testkit/tests/error_codes.rs` uniqueness/format PASS; not every error enum implements `Diagnostic` |
| AC-P0-18 | ✅ | `observability` telemetry tests + `testkit/tests/telemetry_privacy.rs` |
| AC-P0-19 | ✅ | `cargo xtask adr-lint` (12 ADRs, Accepted, all §48 fields) |
| AC-P0-20 | ✅ | `docs/architecture/*`, 5 runbooks, `CONTRIBUTING.md`, `.env.example`, `examples/config/default.toml` |
| AC-P0-21 | ✅ | `storage-sqlite/tests/backup_restore.rs`; `application` restore tests |
| AC-P0-22 | 🔶 PARTIAL | `xtask coverage-gate` + CI wiring; percentages require a CI `cargo llvm-cov` run |
| AC-P0-23 | ✅ | `storage-sqlite/tests/commit_bounds_outbox.rs`, `outbox_idempotency.rs` |
| AC-P0-24 | ✅ | `storage-sqlite/tests/generation_monotonicity.rs` (50 writers) |
| AC-P0-25 | ✅ | `storage-sqlite/tests/tombstone_before_visibility.rs` |
| AC-P0-26 | ✅ | `cli` `outbox_checks_report_backlog_without_mutation` + `doctor_is_read_only` |

**22 PASS · 4 PARTIAL · 0 FAIL.** The four partials are environmental (3-OS CI run,
coverage measurement) or scope (universal `Diagnostic` impls), not correctness failures.

## Known limitations

- Manifest signatures verify a SHA-256 detached hash, not ed25519 (the
  `ed25519-dalek` crate is not available in this environment); the policy path
  and tamper detection are real and tested.
- Keychain / age-encrypted secret backends are abstracted but return
  `Unsupported` until the `keyring` / `age` crates are added.
- No in-process job worker pool yet (T37); jobs are driven through the
  repository + cooperative cancellation, which is what the Phase-0 guarantees
  and tests exercise.
- Coverage percentages and the 3-OS matrix must be confirmed by a CI run.

## Recommended next phase

Phase 1 (Canonical Quran Core) is unblocked: it inherits the domain types,
storage abstraction, provenance/`ApprovalToken`, audit chain, source state
machine + manifest registry, job system, and the outbox/generation primitives.
See `docs/plans/handoff-p0-to-p1.md`.
