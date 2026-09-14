# Progress Status

**Updated:** 2026-09-14
**Active phase:** Phase 0 — Foundations & Provenance

## Summary

The Phase-0 chassis now builds, tests, and runs end to end. The core integrity
primitives (immutable canonical types, provenance/audit models, source state machine,
config precedence, security guards) exist, and the SQLite backend has real repository
implementations plus a checksummed migration runner and `VACUUM INTO` backups.

## Gate status

| Gate | Command | Status |
|---|---|---|
| Format | `cargo fmt --all -- --check` | ✅ clean |
| Lint | `cargo clippy --workspace --all-targets -- -D warnings` | ✅ clean |
| Tests | `cargo test --workspace` | ✅ 139 passing |
| Architecture | `cargo run -p xtask -- arch-check` | ✅ OK |
| Migrations | `cargo run -p xtask -- migrate-check` | ✅ OK |
| Binary smoke | `qai db migrate/status/verify/backup`, `qai doctor` | ✅ works |

## Crate status

| Crate | Status | Notes |
|---|---|---|
| `domain` | ✅ | IDs, types, hashing, licensing, security guards (path/archive/net/input/sanitize) |
| `config` | ✅ | Layered loader, `ValueOrigin`, `Secret<T>`, validation |
| `storage` | ✅ | Traits + `StorageError`; repo traits now `Send + Sync` |
| `storage-sqlite` | ✅ | Real repos, migration runner, checksums, `VACUUM INTO` backup |
| `provenance` | ✅ | Record model, `ApprovalToken`, `CanonicalWriter` |
| `audit` | ✅ | Event model, hash-chain writer, verifier, redaction |
| `sources` | ✅ | Manifest parser, state machine, genealogy resolver |
| `jobs` | 🔶 | Types + `JobStore`; worker pool/chaos tests pending |
| `observability` | ✅ | Subscriber, span conventions, metric catalog |
| `cli` | ✅ | Full command tree + `qai` binary + 26-check doctor |
| `server` | ✅ | Health server (`/healthz`, `/readyz`) |
| `application` | ✅ | Composition root + `run()` bootstrap |
| `testkit` | ✅ | Fixtures + security/config integration suites |

## Remaining before Phase-0 exit

- AC-P0-11/12: real crash/lease chaos suite and cancellation timing.
- D0.18: outbox/generations/tombstones domain types, allocators, and consistency suites.
- Secret backends beyond env (keychain, age-encrypted file).
- Coverage gates and the recorded exit-gate ritual (AC-P0-03/05/06/08/11/14/16).
- Final AC verification and Phase-1 handoff document.
