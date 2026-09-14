# Phase 0 — Implementation Summary

This document summarizes what Phase 0 (Foundations and
Provenance) delivered, how it was verified, what differs
from the plan, and what remains open. It is a narrative
companion to the append-only record in `./done.md`. For
the authoritative task states, see `./tasks.md`; for the
exit-gate criteria, see `./acceptance.md`.

**Status as of September 14, 2026:** implementation is
complete (65 of 67 tasks; 25 of 26 acceptance criteria
verified by automated tests, 1 partially verified).
Human sign-off, the exit-gate ritual recording, and the
three swimlane-X decision owners are still pending, so
formal phase acceptance stays open. Details live in
`./done.md` §8.

## What was built

Phase 0 delivers the chassis described in `./plan.md`:
a dependency-light domain model, layered configuration
with secret handling, a SQLite storage backend behind a
storage abstraction, provenance and hash-chained audit,
a durable job system with a worker pool, the source
catalog with manifest verification and lifecycle rules,
a read-only `doctor`, security guard libraries, and the
outbox, generation, and tombstone primitives. No
religious corpora were touched.

The per-deliverable record:

- **D0.1** — Workspace, pinned toolchain, lint gates,
  and `xtask` with `arch-check`, `ci` (9 steps),
  `migrate-check`, `gen-schema`, `coverage-gate`,
  `adr-lint`, and a JSON-schema `validate` command.
- **D0.2** — Domain types: newtype IDs, `SemVer`,
  `Timestamp`, `Language`, `Confidence`,
  `DataLayer`, trust and verification levels,
  `ContentHash` with canonical JSON, licensing,
  and `CorpusGeneration` scope types.
- **D0.3** — `Diagnostic` trait, namespaced
  `QAI-<NS>-nnnn` codes, and uniqueness tests.
- **D0.4** — Layered config loader with CLI over
  environment over file over defaults, per-value
  `ValueOrigin`, `${...}` interpolation, and
  validation with actionable errors.
- **D0.5** — `Secret<T>` redaction, `SecretRef`,
  `SecretStore` trait, and env, OS-keychain, and
  XChaCha20 encrypted-file backends.
- **D0.6** — Storage traits plus a real SQLite
  backend: serialized single-connection write pool,
  read-only pool, pragmas, health, and all five
  repository implementations.
- **D0.7** — Append-only checksummed migrations
  (`0001`–`0006`, up and down files), checksum
  manifest, status and plan commands, and
  `VACUUM INTO` backup with verified restore.
- **D0.8** — Tracing subscriber, span conventions,
  metric catalog, telemetry denylist, and an opt-in
  OTLP exporter behind the `otlp` feature.
- **D0.9** — Durable jobs: leased queue, handler
  registry, payload validation, worker pool with
  cancellation, checkpoints, retry with backoff and
  jitter, dead-lettering, and lease-expiry recovery.
- **D0.10** — Source catalog, manifest parsing and
  validation, hash and ed25519 signature checks with
  unsigned-remote rejection, the lifecycle state
  machine with approval preconditions, genealogy
  with cycle rejection, and local ingest to
  `Staged`.
- **D0.11** — Provenance records, `ApprovalToken`
  obtainable only from a persisted human approval,
  `CanonicalWriter` sessions, canonical-row
  immutability triggers, and the review queue.
- **D0.12** — Append-only hash-chained audit log
  with writer, recomputing verifier, and redaction.
- **D0.13** — `qai` binary with config, database,
  secret, source, job, and audit command groups,
  global flags, `--json` output, and exit codes.
- **D0.14** — `qai doctor` with 26 checks, each
  emitting a remedy and a next command, running
  read-only against the database, plus a
  `--repair-preview` planner that never mutates.
- **D0.15** — Path, archive, SSRF, input, and
  sanitization guards with an attack corpus, plus
  deny-by-default policy decisions. Guards fail
  closed when they cannot be evaluated.
- **D0.16** — `testkit` fixtures, the permanent
  secret-leak suite, and the CI pipeline with the
  3-OS matrix, deny, schema, doctor, coverage, and
  MSRV jobs.
- **D0.17** — Architecture docs, five runbooks,
  `CONTRIBUTING.md`, example config, and thirteen
  accepted ADRs covering ADR-0000 through
  ADR-0012.
- **D0.18** — Outbox, generation, and tombstone
  tables with a same-transaction guarantee: a
  projection-relevant commit always carries its
  outbox row, generations never regress, and
  deactivation writes the tombstone first.

## Verification results

Every result below was produced by an automated
command. Full evidence lives in `./done.md` and
in `../../05-followups/done.md`.

| Gate | Command | Result |
|---|---|---|
| Tests | `cargo test --workspace` | 267 passing, 0 failing |
| Pre-merge gate | `cargo xtask ci` | 9 of 9 steps pass on macOS |
| Format and lint | `cargo fmt --check`, `cargo clippy -- -D warnings` | Clean |
| Architecture | `cargo xtask arch-check` | No forbidden edges |
| Migrations | `cargo xtask migrate-check` | 6 ordered, checksums stable |
| ADRs | `cargo xtask adr-lint` | 12 accepted, §48-complete |
| Coverage | `cargo llvm-cov` + `coverage-gate` | All thresholds met |
| Cross-target | `cargo check` for Linux and Windows targets | Core crates type-check |
| Binary smoke | `qai db migrate`, `status`, `verify`, `backup`, `restore`, `doctor --json` | Works end to end |

Acceptance criteria: 25 of 26 verified by the
suites named in `./acceptance.md` §3.2, with
`AC-P0-05` partially verified (the secret-leak
suite passes; the strict all-surfaces wording
needs a hardening follow-up — see `./done.md`
§7). Live-walkthrough criteria additionally need
the recorded ritual before sign-off.

## Deviations from the plan

Two deliberate deviations are logged in
`./done.md` §5:

- **DEV-01** — Integration suites live in crate
  test targets (`crates/testkit/tests/`,
  `crates/storage-sqlite/tests/`) instead of a
  workspace-level `tests/` tree. The named suites
  exist with equivalent coverage.
- **DEV-02** — The `source`, `job`, `audit`, and
  `secret` CLI groups parse but dispatch to
  Phase-1 stubs; only `db`, `config`, `doctor`,
  and `serve` execute. Verification for the
  stubbed surfaces comes from library and
  integration tests, and the runbooks' CLI
  references are Phase-1-forward.

## Open items

The following are not done and must not be read
as done:

- **T55, T56** — the `/api/v1/meta` route and the
  Dockerfile/compose stub were deferred (see
  `./done.md` §7).
- **T16 scope** — the sentinel suite passes, but
  log, error, CLI, and doctor output surfaces plus
  a global tracing redaction layer remain as
  hardening work.
- **Manual keychain matrix** — keychain tests are
  `#[ignore]`-gated per plan risk R5 and need a
  manual OS run.
- **Residual guarantee tests** — backoff
  distribution, non-idempotent retry policy, a
  dedicated review accept/reject test, and a
  deterministic clock helper for `testkit`.
- **Exit-gate ritual** — the seven live
  walkthroughs still need recording.
- **Swimlane X** — P0-X01, P0-X02, and P0-X03
  still have no owner or decision-open date.
- **Sign-off** — all `./done.md` §8 owner and
  date cells are empty pending human review.

## Next steps

1. Assign swimlane-X owners and decision-open
   dates so Phase 1 does not stall on external
   lead times.
2. Record the seven exit-gate walkthroughs and
   archive the recording link in `./done.md`.
3. Collect human sign-off on `./done.md` §8,
   then mark Phase 0 accepted.
4. Enter Phase 1 through
   `../phase-01-core/` using the handoff at
   `../../plans/handoff-p0-to-p1.md`, which lists
   every asset Phase 1 inherits and must not
   re-invent.
