# ADR-0004 — Configuration & Precedence Model

- Status: Accepted
- Phase: 0 — Foundations
- Date: 2026-09-11
- Related decisions: ADR-0001, ADR-0003, ADR-0005
- Requirements: PRD §25.13

## Context

Q-ai runs in desktop, server, and CI environments with different configuration
needs. A single source of truth is needed that is deterministic, debuggable,
and safe to ship with defaults that meet the security baseline (§37, §84).

Configuration must never leak secrets through logs, error messages, or
`config show` output.

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| **Layered (CLI > Env > File > Defaults)** | Familiar; explicit precedence; easy to debug with `--explain` | Env var syntax can be verbose for deep nesting |
| Pure TOML file | Simple | No CLI/Env integration; hard to vary per environment |
| Pure CLI flags | One-line reproducibility | Impractical for dozens of settings |
| Env-only with `QAI__` prefix | Container-friendly | Hard to read; no defaults layer |

## Decision

Use a **layered model with strict precedence `CLI > Env > File > Defaults`**,
backed by a `config` crate that:

1. Loads a TOML file (`examples/config/default.toml` is the canonical default).
2. Maps environment variables via `QAI__SECTION__KEY=value` (double underscore
   = nesting).
3. Applies CLI flags (clap `ValueEnum`-derived overlays).
4. Every field records its `ValueOrigin` (`Default`, `File(path, line)`, `Env(var)`,
   `Cli(flag)`) so `qai config show --explain` prints provenance.
5. Validates at load time **and** on every change (hot reload is Phase 11; the
   `validate()` contract is fixed now).
6. Rejects `bind = "0.0.0.0"` with `tls = "disabled"` unless
   `require_auth_outside_localhost` can be satisfied (hard fail with remedy).

## Accuracy and Religious-Source Implications

None directly. Configuration drives security policy (`policy::baseline`) and
the deny-by-default posture; misconfiguration here can cause canonical data to
be exposed or processed unsafely, so `config validate` is a `qai doctor` check.

## Licensing Implications

TOML parsing uses the `toml` crate (MIT/Apache-2.0). No licensing impact.

## Security Implications

- `Secret<T>` fields serialize as `"***"` in every representation, including
  `config show`.
- Config files containing secret refs must be `0600` (a `qai doctor` check).
- `${...}` interpolation cycles are detected and rejected with `QAI-CFG-000x`.

## Operational Implications

- `qai config set <key> <value>` updates the file and re-validates atomically.
- Defaults are versioned with the binary; changing a default version-bumps the
  schema version stored in `settings` table (`origin = 'default'`).

## Migration Strategy

Phase 0 stores config in a `settings` table (`key`, `value_json`, `origin`,
`updated_at`). Future server mode adds env-file layering; the precedence order
does not change.

## Reversal Cost

Low. The layered model is additive; removing any layer is a no-op since the
next layer down already provides a fallback.

## Acceptance Criteria

- 16-case precedence matrix (CLI > Env > File > Defaults) holds for all typed fields.
- `qai config show --explain` prints the origin of every value.
- `qai config validate --file <path>` rejects invalid configs with actionable diagnostics.
- A sentinel secret never appears in any `config show` output.
