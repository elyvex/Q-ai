# Coding Rules

## Language

- Rust is the primary language. All new code is Rust.
- Follow the phase plans under `docs/03-plan/phases/` for crate boundaries.

## Conventions

- No domain code depends on `api`, `server`, `tui`, `cli`, or any concrete provider.
- Canonical rows are written only through a `CanonicalWriter` that requires an `ApprovalToken`.
- Every derived artifact records the version of everything it was derived from.
- Deny-by-default: permissions, network, filesystem, and side effects default to empty sets.
- Nothing that can modify data runs inside `doctor`.
- Secrets are never logged, never embedded in code, and never written to plain config.

## Security

- Imported internet content is untrusted until validated.
- Local data remains local unless the user explicitly enables a remote provider.
- Agents and tools use deny-by-default permissions.
- No model may fabricate a verse, hadith, chain, grading, or citation.
