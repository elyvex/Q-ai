# ADR-0012 — Workspace/Crate Boundaries & Dependency Enforcement

- Status: Accepted
- Phase: 0 — Foundations
- Date: 2026-09-11
- Related decisions: ADR-0001, ADR-0003, ADR-0004, ADR-0005
- Requirements: PRD §33, §87, §88

## Context

Q-ai is structured as a Cargo workspace with ~50 crates spanning domain, infrastructure, application, and interface layers.
To maintain architectural integrity, prevent cyclic dependencies, and enforce the separation of concerns mandated by the PRD (especially the "domain depends only on serde/thiserror/time/uuid" rule), we require automated enforcement of crate dependency boundaries.

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| **cargo-deps / cargo-tree manual review** | No tooling needed | Manual, error-prone, not CI-gateable |
| **cargo-deny only (licenses/advisories)** | Already in CI | Does not enforce custom architectural rules (e.g., domain -> no sqlx) |
| **Custom `xtask arch-check`** | Tailored to Q-ai's exact layer graph; runs in CI; outputs clear `Diagnostic` errors | Requires maintenance of allowlist as crates are added |

## Decision

We implement a custom **`xtask arch-check`** command (D0.1, P0-T02) that:
1. Reads a declarative allowlist (`xtask/allowlist.toml`) mapping each crate to its permitted direct dependencies.
2. Parses `Cargo.toml` files across the workspace to extract actual `dependencies`, `dev-dependencies`, and `build-dependencies`.
3. Compares actual vs. allowed dependencies.
4. Emits a structured `Diagnostic` (using the domain error codes) for any forbidden edge, including the file/line, the offending dependency, and a remedy.
5. Fails CI immediately if any forbidden edges exist.

### Layer Graph (Enforced by `arch-check`)

```
domain          -> (serde, thiserror, time, uuid) ONLY
application     -> domain, storage(traits), provenance, audit, jobs, sources, config
storage         -> domain (traits + errors only)
storage-sqlite  -> storage, domain, sqlx
provenance      -> domain, storage
audit           -> domain, storage
jobs            -> domain, storage, observability
sources         -> domain, storage, provenance
cli             -> application, config, observability
server          -> application, config, observability
observability   -> (tracing, metrics) ONLY
```

## Accuracy and Religious-Source Implications

- Prevents canonical text logic from leaking into infrastructure crates (e.g., `domain` importing `sqlx` would expose direct SQL manipulation of Quranic text).
- Guarantees provenance, audit, and source-catalog integrity boundaries are maintained at compile time.

## Licensing Implications

- `xtask` is internal tooling (MIT/Apache-2.0). The allowlist is committed to source control and auditable.

## Security Implications

- The dependency graph enforces the deny-by-default principle at the crate level.
- Infrastructure crates (network, crypto, FFI) cannot be reached from `domain` without explicit architectural review.

## Operational Implications

- Adding a new crate requires updating `xtask/allowlist.toml` and justifying its layer.
- `cargo xtask ci` runs `arch-check` as a mandatory gate.

## Migration Strategy

Not applicable — this is a build-time guard that tightens as the workspace grows.

## Reversal Cost

Low. The allowlist can be relaxed by editing `xtask/allowlist.toml` if architectural boundaries are formally changed.

## Acceptance Criteria

- `cargo xtask arch-check` passes on a compliant workspace.
- `cargo xtask arch-check` fails with a clear `Diagnostic` when a forbidden edge is introduced (mutation test in `tests/arch/forbidden_edge_is_detected`).
- `xtask/allowlist.toml` documents every crate in the workspace.