# ADR-0005 — Secret Storage per OS (Env / Keychain / Encrypted File)

- Status: Accepted
- Phase: 0 — Foundations
- Date: 2026-09-11
- Related decisions: ADR-0004, ADR-0001
- Requirements: PRD §§25.13, 37, 84

## Context

Q-ai needs API keys (LLM providers, search services) and other secrets. Local
single-user installs want OS-native protection; headless/CI installs want
env-var injection; privacy-conscious users want an age-encrypted file. No
secret value may ever be stored in SQLite, logged, printed, or appear in error
messages (the permanent CI "secret never escapes" suite).

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| **Env backend** | Universal; works in CI and containers | Secrets may appear in process listings / env dumps |
| **OS keychain** | Native security (Keychain / Credential Manager / Secret Service) | OS-specific behaviour; CI-unfriendly; gated tests |
| **age-encrypted file** | Offline-friendly; portable; auditable | Requires a passphrase; file management burden |
| HashiCorp Vault / cloud KMS | Enterprise-grade | Network dependency; licensing; overkill for local-first |

## Decision

Support **three backends**, selected at runtime via `secrets.backend`:

1. **`env`** (default): reads from environment variables. Used in CI and containers.
2. **`keychain`**: OS keychain on macOS (Keychain / Security Framework), Windows
   (Credential Manager), Linux (Secret Service via `secret-service` crate).
3. **`encrypted_file`**: age/XChaCha20-Poly1305 encryption to
   `${app.data_dir}/secrets.age`; passphrase from env or interactive prompt.

Design rules:

- `SecretRef` (`secret://keychain/openai/default`) is stored in config/SQLite.
  **Secret values are never stored in SQLite.**
- `Secret<T>` wrapper: `Debug`/`Display`/`Serialize` all emit `***`; `zeroize`
  on drop.
- A **global redaction layer** in the tracing subscriber applies regex +
  known-secret-value matching to logs, spans, audit records, and CLI output.
- `qai secret set` reads from stdin/prompt — never from argv (argv leaks in
  process listings).

## Accuracy and Religious-Source Implications

None directly. Provider credentials for Islamic-content APIs (search, embedding,
LLM) fall under this ADR; compromising them could inject false content into
research results, so leak-prevention is a Phase 0 permanent CI gate.

## Licensing Implications

age crate (MIT/Apache-2.0). Keychain bindings are OS-provided. No licensing concerns.

## Security Implications

- Critical: if any backend fails to redact a secret, the entire system's
  confidentiality is compromised — hence the permanent `secret_leak.rs` CI gate
  that puts a sentinel into every backend and asserts it appears in **zero**
  output bytes across logs, errors, CLI, doctor JSON, and audit rows.
- Audit events reference `SecretRef` only (never values).
- Ref counts list exposes references only, never values.

## Operational Implications

- Backend migration is a one-line config change + `qai secret set` for each ref.
- In CI, `QAI_SECRETS_BACKEND=env` must be set for tests; keychain tests are
  `#[ignore]`-gated.

## Migration Strategy

Phase 0 stores only `SecretRef` in SQLite. If a future phase must persist values
(e.g. offline sync), it must introduce a new encrypted store with a new table
behind a feature flag — never modify the Phase 0 secret schema.

## Reversal Cost

Low for env→keychain (both read the same `SecretRef`). The encrypted-file
backend may require re-importing all refs on rotation.

## Acceptance Criteria

- A sentinel value set through every backend never appears in logs, errors, CLI
  output, doctor JSON, or audit rows (permanent CI gate).
- `qai secret list` returns references only; values require `qai secret get`.
- `qai secret set` rejects values passed as CLI arguments.
