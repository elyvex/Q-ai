# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | ✓         |

## Reporting a Vulnerability

Please report security vulnerabilities by opening a private security advisory
on the repository. Do not open public issues for security reports.

## Security Controls

This document describes the concrete security controls already implemented in
the codebase. Each control names the file that implements it.

### 1. Secret Redaction

**Control:** A global redaction layer ensures secrets never appear in logs,
error messages, or output.

**Implementation:** `crates/domain/src/redaction.rs`

**Details:**
- `Secret<T>` redacts and zeroizes on drop
- `SecretRef` references secrets without exposing them
- Sentinel suite asserts secrets never appear in output bytes
- CI gate: `cargo test -p testkit --test secret_leak`

### 2. OTLP Exporter Boundary

**Control:** The OTLP telemetry exporter scrubs sensitive fields before
export.

**Implementation:** `crates/observability/src/otlp.rs`

**Details:**
- Compile-time-tested telemetry denylist
- Query text, prompt text, document content, research questions, and model
  responses are never exportable
- Telemetry is opt-in (`telemetry.enabled = false` by default)

### 3. Deny-by-Default Permissions

**Control:** Agent and tool permissions are deny-by-default with human
approval required for canonical writes.

**Implementation:** `crates/provenance/src/lib.rs`, `crates/domain/src/ids.rs`

**Details:**
- `ApprovalToken` is only mintable from a persisted `ApprovalRecord`
- `CanonicalWriter` requires an `ApprovalToken`
- Agents and tools cannot construct an `ApprovalToken`
- The importer holds no approval capability

### 4. Approval-Gated Canonical Writer

**Control:** Canonical text can only be modified through an approval-gated
write path.

**Implementation:** `crates/quran-corpus/src/import.rs`, `crates/application/src/quran.rs`

**Details:**
- Canonical writes require a persisted human `ApprovalRecord`
- The activation transaction is atomic (staging → canonical, pointer flip,
  `corpus_generation` bump)
- Rollback is via pointer flip
- The importer holds no `ApprovalToken`

### 5. Agent-Readable-Archive / Symlink / Path Guards

**Control:** File system operations are guarded against path traversal and
symlink attacks.

**Implementation:** `crates/security/src/`

**Details:**
- Path canonicalization before access
- Symlink resolution and validation
- Archive extraction guards against path traversal
- Fail-closed: if a guard cannot be enforced, the operation is denied

### 6. Input Size Limits

**Control:** Input size limits prevent denial-of-service through oversized
inputs.

**Implementation:** `crates/security/src/`

**Details:**
- `Untrusted<T>` wrapper for untrusted input
- Size limits on search queries, regex patterns, and API payloads
- Regex: 1 MiB pattern limit, 4 MiB result limit, 512-char pattern length,
  3-second timeout
- Rate limits per principal

### 7. Error Code Taxonomy

**Control:** Namespaced error codes (`QAI-*`) provide typed, actionable
errors without leaking internal details.

**Implementation:** `crates/domain/src/diagnostic.rs`

**Details:**
- Every error code is registered and unit-tested
- Error codes are namespaced by domain (e.g., `QAI-QUR-*`, `QAI-CNT-*`,
  `QAI-IDX-*`)
- CLI exit codes are separate from error codes

### 8. Audit Integrity

**Control:** Append-only audit events with hash-chain verification.

**Implementation:** `crates/provenance/src/lib.rs`

**Details:**
- `audit_events` table with triggers `QAI-AUD-0001/0002`
- Per-row SHA-256 hash chain
- `qai audit verify` command for verification
- Tamper detection: any modification breaks the chain

### 9. Manifest Signing

**Control:** Source manifests are signed with ed25519 detached signatures.

**Implementation:** `crates/sources/src/`

**Details:**
- Unsigned manifests can never become `Active`
- Signatures are verified before activation
- Signature verification is fail-closed

### 10. Configuration Security

**Control:** Configuration precedence is CLI > Env > File > Defaults, with
validation errors that name the remedy.

**Implementation:** `crates/config/src/`

**Details:**
- `bind 0.0.0.0` without TLS + auth hard-fails
- Secrets via env/keychain/encrypted file, never stored as values
- `SecretRef`s only in config/DB

## CI Security Gates

The following CI jobs enforce security controls:

| Job | Command | What It Checks |
|-----|---------|----------------|
| Deny | `cargo-deny check --all-features advisories bans licenses sources` | License compliance, advisory bans |
| Lint | `cargo clippy --workspace --all-targets -- -D warnings` | Lint denies |
| Arch | `cargo run -p xtask -- arch-check` | No forbidden crate dependencies |
| Migrate | `cargo run -p xtask -- migrate-check` | Migration checksums stable |
| Test | `cargo test --workspace` | All tests pass |

## Scope

This security policy covers the Q-ai codebase. It does not cover:
- Infrastructure security (deployment, networking, OS)
- Third-party dependency security (handled by `cargo-deny`)
- User data security (user's responsibility to secure their local data)
