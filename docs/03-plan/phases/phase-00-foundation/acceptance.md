# Phase 0 — Acceptance Criteria & Exit Gate

**Phase:** P0 — Foundations & Provenance
**Source:** `plan.md` §9 (Acceptance Criteria), §8 (Testing Strategy), §11 (Definition of Done)
**Criteria:** 26 (AC-P0-01 … AC-P0-26)
**Gate owner:** task P0-T60
**Status:** 🔴 0 / 26 verified

**Status legend:** ☐ Not verified · ◐ Partially verified · ✗ Failed · ☑ Verified

> **Verification rule:** an AC is ☑ only when an **automated test or scripted check** proves it
> and the evidence artifact (CI run URL, test path, or recording) is recorded in the Evidence
> column and appended to `done.md`. "It works on my machine" is not verification. Seven ACs
> additionally require a live walkthrough — see §4.

---

## 1. Acceptance Criteria

### 1.1 Build, architecture & migrations

| ID | Criterion | Verification | Blocks | Evidence | Status |
|---|---|---|---|---|---|
| **AC-P0-01** | `cargo xtask ci` passes on Linux, macOS, and Windows from a clean clone | CI matrix green | D0.1, D0.16 | | ☐ |
| **AC-P0-02** | `cargo xtask arch-check` **fails** when a forbidden dependency edge is introduced | **Mutation test:** add a `domain -> cli` dependency; CI must fail | D0.1 | | ☐ |
| **AC-P0-03** | 🎥 `qai db migrate` on an empty dir produces a valid schema; re-run is a no-op; editing an applied migration file causes a hard, coded failure | Scripted test (`tests/db/migrations.rs`) | D0.7 | | ☐ |
| **AC-P0-21** | `qai db backup` / `qai db restore` round-trip a populated database with byte-identical audit-chain verification afterwards | Scripted test | D0.7 | | ☐ |
| **AC-P0-22** | Coverage gates met (§2 below) | CI coverage report | D0.16 | | ☐ |

> AC-P0-02 is a *negative* criterion — it is only satisfied by demonstrating the guard fires.
> A green `arch-check` on compliant code proves nothing.

### 1.2 Configuration & secrets

| ID | Criterion | Verification | Blocks | Evidence | Status |
|---|---|---|---|---|---|
| **AC-P0-04** | 🎥 Precedence **CLI > Env > File > Defaults** holds for all typed field kinds, and `qai config show --explain` prints the origin of every value | Precedence matrix test (16 cases) + snapshot | D0.4 | | ☐ |
| **AC-P0-05** | 🎥 A sentinel secret set through **each** backend never appears in logs, errors, CLI output, doctor JSON, or audit rows | `tests/security/secret_leak.rs` — sentinel must appear in **zero output bytes** | D0.5 | | ☐ |
| **AC-P0-16** | 🎥 `qai serve` binds `127.0.0.1` by default; a non-loopback bind with `tls = "disabled"` **fails config validation** with an actionable error | Config + integration test | D0.4, D0.1 | | ☐ |

> AC-P0-05 is a **permanent CI gate reused by every future phase**, not a one-time Phase 0
> check. It must exercise env, keychain, and age-encrypted-file backends.

### 1.3 Canonical integrity & provenance — the non-negotiable core

| ID | Criterion | Verification | Blocks | Evidence | Status |
|---|---|---|---|---|---|
| **AC-P0-06** | 🎥 Canonical provenance rows cannot be updated or deleted, and canonical writes are **impossible** without an `ApprovalToken` derived from a persisted human approval | `tests/integrity/canonical_guard.rs` + trigger tests | D0.11 | | ☐ |
| **AC-P0-07** | A computational annotation cannot be stored without algorithm + version + confidence, and cannot reach `human_verified` without a reviewer | DB `CHECK` constraint tests | D0.11 | | ☐ |
| **AC-P0-13** | Audit chain verifies end-to-end; `qai audit verify` **detects a manually tampered row** | `tests/integrity/audit_chain.rs` | D0.12 | | ☐ |

> These three encode PRD §7.3, §10.6, §82, and §92:43–46. AC-P0-06 additionally requires that a
> `CanonicalChangeRequest` missing **any** §7.3 field (new source version, checksum result,
> structural validation result, difference report, approver identity) is rejected — one test
> per missing field.
>
> **If AC-P0-06 or AC-P0-13 cannot be met, Phase 0 does not exit.** Neither is retrofittable:
> once code paths exist that write canonical rows without a token, the guarantee is gone.

### 1.4 Source lifecycle

| ID | Criterion | Verification | Blocks | Evidence | Status |
|---|---|---|---|---|---|
| **AC-P0-08** | 🎥 Source lifecycle rejects **every** illegal transition; `Approved` is impossible without hash, known license, validation report, and approver; at most one `Active` version per source | `tests/sources/state_machine.rs` | D0.10 | | ☐ |
| **AC-P0-09** | A signed local manifest imports to `Staged`; a tampered manifest is rejected with `QAI-SRC-…`; an unsigned **remote** manifest is rejected under default policy | `tests/sources/manifest.rs` | D0.10 | | ☐ |
| **AC-P0-10** | Source genealogy renders a 3-level lineage sentence and **rejects cycles** | Unit + snapshot test | D0.10 | | ☐ |

> AC-P0-08 enforces §22.3: no source may become active solely because an LLM recommended it.
> The `approved_by IS NOT NULL` precondition is checked in **both** SQL `CHECK` and Rust — a bug
> in one layer must not produce an illegal row.
>
> AC-P0-10 target rendering: *"English translation (2024) of an Arabic summary (1990) of the
> original manuscript (1200)."*

### 1.5 Jobs & recovery

| ID | Criterion | Verification | Blocks | Evidence | Status |
|---|---|---|---|---|---|
| **AC-P0-11** | 🎥 Duplicate enqueue with the same idempotency key yields **one** execution; `SIGKILL`-ing a worker marks the run `Interrupted` and it resumes from checkpoint **without repeating completed stages** | `tests/recovery/jobs.rs` chaos suite | D0.9 | | ☐ |
| **AC-P0-12** | `qai job cancel` stops a long-running job **within 2 seconds** and records cancellation | Timed test | D0.9 | | ☐ |

### 1.6 Diagnostics, security & observability

| ID | Criterion | Verification | Blocks | Evidence | Status |
|---|---|---|---|---|---|
| **AC-P0-14** | 🎥 `qai doctor` runs against a **read-only** database file and never issues a write; all Phase-0 checks emit a remedy and next command; `--json` validates against the published schema | Read-only + schema test | D0.14 | | ☐ |
| **AC-P0-15** | Security guards reject the full attack corpus (path, archive, SSRF, size, depth) and **fail closed** when a guard cannot be evaluated | `tests/security/{path,archive,ssrf}_guard.rs` | D0.15 | | ☐ |
| **AC-P0-17** | Every error type implements `Diagnostic`; every error code is **unique**; human and JSON renderings are snapshot-stable | Conformance test | D0.3, D0.13 | | ☐ |
| **AC-P0-18** | Telemetry is **off by default**; enabling it never exports denylisted content fields | `tests/observability/telemetry_privacy.rs` | D0.8 | | ☐ |

> AC-P0-15 corpus minimums: **40+** traversal/symlink payloads for path; zip-slip, bomb,
> symlink-entry, and nested-depth for archive; loopback, private, link-local, DNS-rebind, and
> redirect-to-private for SSRF. Fail-closed is a distinct assertion from reject-known-bad.
>
> AC-P0-18 denylist: query text, prompt text, document content, research questions, model
> responses (PRD §39).

### 1.7 Cross-store consistency primitives (D0.18)

| ID | Criterion | Verification | Blocks | Evidence | Status |
|---|---|---|---|---|---|
| **AC-P0-23** | Every commit to `sources` or `provenance_records` classified as projection-relevant produces **exactly one** durable outbox row **in the same transaction**; a fault-injection test proves a crash between the two is **impossible by construction, not by retry** | `tests/consistency/commit_bounds_outbox.rs` | D0.18 | | ☐ |
| **AC-P0-24** | `corpus_generations.number` **never regresses** under 50 concurrent writers targeting the same scope | `tests/consistency/generation_monotonicity.rs` | D0.18 | | ☐ |
| **AC-P0-25** | Deactivating/rolling back a source writes a tombstone **before** the state-machine transition is visible to readers | `tests/consistency/tombstone_before_visibility.rs` | D0.18 | | ☐ |
| **AC-P0-26** | `qai doctor` reports outbox backlog age and undispatched-event count **without mutating data** | Read-only + JSON test | D0.14, D0.18 | | ☐ |

> AC-P0-23's phrasing is deliberate. A test that enqueues, crashes, retries, and eventually
> converges **does not satisfy it**. The outbox insert must share the SQLite transaction with the
> authoritative change so that a simulated crash between the two rolls both back.
> Also required: `tests/consistency/outbox_idempotency.rs` — duplicate enqueue with the same
> `(operation, idempotency_key)` yields exactly one event.

### 1.8 Documentation & decisions

| ID | Criterion | Verification | Blocks | Evidence | Status |
|---|---|---|---|---|---|
| **AC-P0-19** | ADR-0001 … ADR-0012 exist, are status `Accepted`, and each contains **all §48 fields** — including **religious-source** and **licensing** implications | ADR lint | D0.17 | | ☐ |
| **AC-P0-20** | Docs complete: crate map, data-layer spec, source lifecycle, hashing spec, error codes, **5 runbooks**, `.env.example`, example configs | Doc review checklist | D0.17 | | ☐ |

> AC-P0-19 requires the ADR numbering collision to be resolved first (task P0-T59b) and the lint
> re-run. For Phase 0 ADRs the religious-source section is usually "none directly, but constrains
> Phase 1 canonical integrity" — **the lint must fail if that reasoning is omitted rather than
> written down.**
>
> AC-P0-20's five runbooks: backup & restore · interrupted-job recovery · audit-chain break ·
> migration checksum mismatch · quarantine handling.

---

## 2. Coverage Gates (AC-P0-22)

| Crates | Line coverage | Status |
|---|---|---|
| `domain`, `provenance`, `audit`, `sources`, `security` | **≥ 85%** | ☐ |
| `config`, `jobs`, `storage-sqlite` | **≥ 75%** | ☐ |
| `cli`, `server` | Smoke + snapshot coverage; **no numeric gate** | ☐ |

Measured by `cargo llvm-cov` in the `coverage` CI job. Coverage is a floor, not a target —
it does not substitute for the named suites in §3.2.

---

## 3. Definition of Done

### 3.1 Per-deliverable checklist (PRD §58, §93)

Every Phase-0 deliverable must satisfy **all** of the following. This is the PR template.

- [ ] Implemented behind an explicit interface; no cross-layer coupling (`arch-check` green)
- [ ] Unit tests + property tests where the state space warrants
- [ ] Integration tests against **real** SQLite
- [ ] Typed errors implementing `Diagnostic` with remedy + next command
- [ ] Observable: tracing spans + metrics registered in the catalog
- [ ] Documented in `docs/architecture/` and referenced from the crate's `//!` docs
- [ ] Configuration validated at load **and** on change
- [ ] Cancellation and timeouts implemented where the operation is long-running
- [ ] Secrets redacted everywhere (leak suite green)
- [ ] Provenance and audit events recorded for **every** mutation
- [ ] Schema versions recorded; migration reversible or explicitly forward-only **with rationale**
- [ ] Access/policy checks present (deny-by-default) even in single-user mode
- [ ] Failure recovery tested (crash, retry, interrupted job)
- [ ] `cargo fmt`, `cargo clippy -D warnings`, `cargo test`, `cargo deny` all green

### 3.2 Required test suites

All must exist and be green. Missing a suite fails the gate even if the mapped AC appears
satisfied by other means.

| Suite | Proves | AC |
|---|---|---|
| `tests/security/secret_leak.rs` | Sentinel secret in zero bytes of logs, errors, CLI, doctor JSON, audit rows | 05 |
| `tests/security/path_guard.rs` | 40+ traversal/symlink payloads rejected | 15 |
| `tests/security/archive_guard.rs` | Zip-slip, bomb, symlink-entry, nested-depth rejected | 15 |
| `tests/security/ssrf_guard.rs` | Loopback/private/link-local/DNS-rebind/redirect-to-private rejected | 15 |
| `tests/integrity/canonical_guard.rs` | Canonical rows immutable; writes require `ApprovalToken`; incomplete §7.3 request rejected | 06 |
| `tests/integrity/audit_chain.rs` | Chain verifies; tampered row detected; update/delete aborts | 13 |
| `tests/sources/state_machine.rs` | Illegal transitions rejected; `Approved` preconditions; single `Active` | 08 |
| `tests/sources/manifest.rs` | Schema validation, signature verify/reject, unsigned policy, genealogy cycles | 09, 10 |
| `tests/recovery/jobs.rs` | Lease expiry, crash → `Interrupted`, checkpoint resume, no double side effect | 11 |
| `tests/config/precedence.rs` | Full CLI>Env>File>Defaults matrix; invalid configs rejected actionably | 04 |
| `tests/cli/*.trycmd` | Human + `--json` snapshots for every Phase-0 command | 17 |
| `tests/db/migrations.rs` | Fresh migrate, idempotent re-run, checksum mismatch fails, down-migrations restore | 03 |
| `tests/observability/telemetry_privacy.rs` | Denylisted fields never exported | 18 |
| `tests/consistency/commit_bounds_outbox.rs` | Commit-without-outbox-row impossible by construction | 23 |
| `tests/consistency/outbox_idempotency.rs` | Duplicate `(operation, idempotency_key)` → exactly one event | 23 |
| `tests/consistency/generation_monotonicity.rs` | No regression under 50 concurrent writers | 24 |
| `tests/consistency/tombstone_before_visibility.rs` | Tombstone written before transition is reader-visible | 25 |

### 3.3 CI jobs that must be green

```text
check      -> cargo fmt --check; cargo clippy --all-targets -- -D warnings
arch       -> cargo xtask arch-check; cargo xtask migrate-check
test-linux -> cargo test --workspace --all-features
test-macos -> cargo test --workspace
test-windows -> cargo test --workspace
deny       -> cargo deny check advisories bans licenses sources
schemas    -> cargo xtask gen-schema && git diff --exit-code docs/schemas/
doctor     -> qai doctor --json | validate against docs/schemas/doctor.v1.schema.json
coverage   -> cargo llvm-cov; gates per §2
msrv       -> build with pinned MSRV
```

Keychain-backend tests are `#[ignore]`-gated in CI (risk R5) and must be run in a **manual
OS matrix** before the gate closes — record the result as AC-P0-05 evidence.

---

## 4. Exit-Gate Ritual

A **recorded walkthrough** in which a reviewer — not the implementer — performs the following
**live on a clean machine**:

| Order | AC | Demonstration |
|---|---|---|
| 1 | AC-P0-03 | Migrate an empty dir; re-run; edit an applied migration and show the coded hard failure |
| 2 | AC-P0-05 | Set a sentinel secret in each backend; grep all output surfaces for zero hits |
| 3 | AC-P0-06 | Attempt a canonical write without an `ApprovalToken`; attempt to update/delete a canonical row |
| 4 | AC-P0-08 | Drive an illegal source transition; attempt `Approved` with an unknown license; attempt a second `Active` |
| 5 | AC-P0-11 | `SIGKILL` a worker mid-job; show `Interrupted`, then resume from checkpoint without re-running completed stages |
| 6 | AC-P0-14 | Run `doctor` against a read-only DB file; validate `--json` against the schema |
| 7 | AC-P0-16 | Show default `127.0.0.1` bind; set a non-loopback bind with TLS disabled and show validation failure |

Recording is archived and linked from `done.md`. The gate is **not** closed by a green CI run
alone — these seven are the criteria most likely to pass in CI while being wrong in practice.

---

## 5. Sign-Off

| Gate | Requirement | Owner | Date | Status |
|---|---|---|---|---|
| All 26 ACs verified | §1 fully ☑ | | | ☐ |
| Coverage gates met | §2 | | | ☐ |
| DoD satisfied per deliverable | §3.1 × 18 deliverables | | | ☐ |
| All required suites green | §3.2 | | | ☐ |
| ADR numbering reconciled | P0-T59b | | | ☐ |
| Exit-gate ritual recorded | §4 | | | ☐ |
| Handoff doc published | `docs/plans/handoff-p0-to-p1.md` (P0-T60) | | | ☐ |
| Swimlane X decisions owned & open | `tasks.md` §1 — P0-X01/02/03 | | | ☐ |
| **Phase 0 accepted → Phase 1 unblocked** | All rows above ☑ | | | ☐ |

---

## 6. Handoff Assets Phase 1 Must Not Re-Invent

Verified as part of P0-T60. Phase 1 inherits:

| Asset | Location | Phase-1 usage |
|---|---|---|
| `DataLayer`, `TrustLevel`, `VerificationStatus` | `domain` | Quran text = `CanonicalSource`; numbering = `PublisherMetadata` |
| `ContentHash` + canonical JSON | `domain::hashing` | `text_hash`, `manifest_hash` on the Quran edition (§7.2) |
| `SourceId`/`SourceVersionId` + state machine | `sources` | Edition import → `Staged` → `Approved` → `Active` |
| Manifest schema + `StructureValidator` registry | `sources` | Register `quran_edition_v1` |
| `DifferenceReport` framework | `sources::diff` | Edition version diff (§7.3) |
| `ApprovalToken` + `CanonicalWriter` | `provenance` | The **only** path that may write canonical ayah rows |
| Job system with checkpoints/cancel | `jobs` | `quran.import`, `quran.validate`, `quran.reindex` |
| Audit actions + chain | `audit` | Edition activation/rollback events (§82) |
| Doctor check registry | `cli::doctor` | Add `--quran` checks |
| `Untrusted<T>`, path/archive/SSRF guards | `security` | Importing a downloaded edition archive |
| `testkit` fixtures | `crates/testkit` | Quran corpus golden-file harness |
| Error namespace `QAI-QUR-*` | `domain::error` | Reserved and ready |
| `CorpusGeneration`, `OutboxRepository`, `Tombstone` | `storage`, `domain::generation` | First generation for scope `quran:<edition>`; later builders consume outbox events instead of polling |
