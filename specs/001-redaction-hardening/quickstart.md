# Quickstart: 001-redaction-hardening validation

**Spec**: [spec.md](spec.md) | **Contracts**: [contracts/redaction-api.md](contracts/redaction-api.md)

Proves the feature end to end. No implementation code here — only commands
and expected outcomes (used by `/speckit-tasks` and manual verification).

## Prerequisites

- Rust toolchain per `rust-toolchain.toml`; clean tree (`git status`).
- Baseline gates green before starting (record versions):
  `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --
  -D warnings`, `cargo test --workspace`.

## 1. Run the sentinel suite (primary validation, SC-001/SC-002)

```bash
cargo test -p testkit --test secret_leak
cargo test -p domain redaction
cargo test -p domain diagnostic
cargo test -p audit
cargo test -p observability
```

Expected: all green, including the six new sentinel tests; the three
pre-existing leak tests pass unchanged.

## 2. Manual sentinel drill (US1, log emission)

```bash
RUST_LOG=info cargo run -p qai -- config show 2>stderr.log
grep -c 'SENTINEL' stderr.log || echo "zero sentinel bytes"
```

Then plant a real-shaped drill: temporarily set a secret-bearing env value
through a secret-named tracing field in a debug build, re-run, and confirm
the marker `***REDACTED***` appears where the value would have.

## 3. Error-rendering drill (US2)

Trigger a representative failure (e.g. point the db at an unreadable path
with a credential-shaped segment), capture both human and `--json` error
output, and assert zero sentinel bytes with surrounding structure intact
(parse the JSON rendering to confirm validity).

## 4. CLI inspection drill (US3)

```bash
cargo run -p qai -- config show > config.txt
cargo run -p qai -- config show --json | python3 -m json.tool > /dev/null
cargo run -p qai -- doctor --json > doctor.json
cargo run -p xtask -- schema-check   # doctor doc still validates
```

Expected: outputs contain origins/keys but no secret bytes; doctor JSON
validates against `doctor.v1.schema.json`.

## 5. Architecture + migration gates (SC-003, constitution VII)

```bash
cargo run -p xtask -- arch-check     # passes WITH the allowlist amendment
git diff xtask/allowlist.toml        # exactly one justified line for observability→domain
cargo run -p xtask -- migrate-check  # untouched, still green
```

## 6. Full gate sweep

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
qai db status && qai doctor
```

## Review guidance (code-review concern, research D3)

`redact_text` cannot catch a raw exposed secret interpolated with no
`key=` shape. During review, flag any `.expose()` (or `Deref` of
`Secret<T>`) inside logging/diagnostic formatting — secrets must cross
those boundaries only as `Secret<T>` (typed redaction) or under
secret-named structured fields (Rule A).
