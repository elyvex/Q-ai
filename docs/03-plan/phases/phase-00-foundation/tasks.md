# Phase 0 — Task Board

**Phase:** P0 — Foundations & Provenance
**Source:** `plan.md` §7 (Work Breakdown Structure)
**Total tasks:** 67 sprint tasks + 3 swimlane decisions
**Status legend:** ☐ Not Started · ◐ In Progress · ⊘ Blocked · ☑ Done (→ log in `done.md`)
**Roles:** **BE** backend/Rust · **INF** infra/CI · **SEC** security · **DOC** docs
**Est** = engineer-days (ed)

> Completion protocol: a task is ☑ only when it satisfies the Definition of Done
> (`acceptance.md` §3) — not when the code compiles. On completion, append an entry to
> `done.md` with the date, owner, PR, and evidence link.

---

## 1. Swimlane X — Cross-Phase Decisions (start Week 1, run in parallel)

These are **not** Phase 0 deliverables and are **not** on Phase 0's critical path. They are
tracked here because each has an external lead time that engineering cannot compress later, and
because "not on the critical path" is exactly why they get silently dropped.

| ID | Decision | ADR | Needed by | Owner | Decision-open date | Status |
|---|---|---|---|---|---|---|
| P0-X01 | Quran text dataset selection & licensing — open the ADR **this week**; start vendor/license correspondence in parallel with T01–T05 | ADR-0101 | End of Phase 0 | _unassigned_ | _TBD_ | ☐ |
| P0-X02 | Morphology dataset selection & licensing — open as a **draft** during Phase 0; do not wait for Phase 1 to finish | ADR-0203 | Start of Phase 2 | _unassigned_ | _TBD_ | ☐ |
| P0-X03 | Normalization rule catalog + **linguist engagement** — begin sourcing/booking immediately; target 0.4 FTE from Sprint 2.0 | ADR-0204 | Start of Phase 2 | _unassigned_ | _TBD_ | ☐ |

**Action required at the first standup:** assign an owner and a decision-open date to each row
above, independent of whether that owner has other Phase 0 work. Do not close Sprint 0.1
retrospective with any cell reading `_unassigned_`.

---

## 2. Sprint 0.1 — Skeleton & Contracts (Week 1) — 14.5 ed

| ID | Task | Deliv. | Depends | Est | Role | Status |
|---|---|---|---|---|---|---|
| P0-T01 | Create workspace, toolchain pin, lint config, `[workspace.dependencies]` | D0.1 | — | 1.5 | INF | ☑ |
| P0-T02 | `xtask` with `arch-check`, `ci`, `migrate-check`, `gen-schema` | D0.1 | T01 | 2.0 | INF | ☑ |
| P0-T03 | `deny.toml` license/advisory policy + CI job | D0.1 | T01 | 0.5 | INF | ☑ |
| P0-T04 | Placeholder crates for all PRD §87 crates with `//! Phase N` doc comments | D0.1 | T01 | 0.5 | BE | ☑ |
| P0-T05 | `domain`: typed IDs, `SemVer`, `Timestamp`, `Language`, `Confidence` | D0.2 | T01 | 1.5 | BE | ☑ |
| P0-T06 | `domain`: `DataLayer`, `TrustLevel`, `VerificationStatus`, `SideEffectClass` | D0.2 | T05 | 1.0 | BE | ☑ |
| P0-T07 | `domain`: `ContentHash`, `canonical_json_bytes`, hashing spec + prop tests | D0.2 | T05 | 1.5 | BE | ☑ |
| P0-T08 | `domain`: `LicenseRecord`, `DerivationVersions`, `SubjectRef` URN grammar | D0.2 | T05 | 1.0 | BE | ☑ |
| P0-T09 | `Diagnostic` trait, error-code registry, uniqueness test | D0.3 | T05 | 1.5 | BE | ☑ |
| P0-T10 | ADR-0001 / 0002 / 0006 / 0012 written and reviewed | ADR | T01 | 1.5 | DOC | ☑ |
| P0-T11 | CI pipeline (check / arch / test-3os / deny / schemas / coverage) | D0.16 | T02 | 2.0 | INF | ☐ |

**Sprint exit:** `cargo xtask ci` green on a clean checkout on Linux, macOS, and Windows.
`DerivationVersions` (T08) must already carry `dependency_snapshot_id` as an `Option`
placeholder — this exists now so Phase 1/2 do not rework the type.

---

## 3. Sprint 0.2 — Config, Secrets, Storage, Migrations (Week 2) — 19.0 ed

| ID | Task | Deliv. | Depends | Est | Role | Status |
|---|---|---|---|---|---|---|
| P0-T12 | `config` crate: layered loader + `ValueOrigin` + `${...}` interpolation | D0.4 | T09 | 2.5 | BE | ☐ |
| P0-T13 | Config validation rules + 16-case precedence matrix tests | D0.4 | T12 | 1.5 | BE | ☐ |
| P0-T14 | `Secret<T>`, `SecretRef`, `SecretStore` trait | D0.5 | T09 | 1.0 | SEC | ☐ |
| P0-T15 | Env + keychain + age-encrypted-file backends | D0.5 | T14 | 2.5 | SEC | ☐ |
| P0-T16 | Global redaction tracing layer + secret-leak sentinel suite | D0.5, D0.16 | T14 | 2.0 | SEC | ☐ |
| P0-T17 | `storage` traits: `Database`, `ReadTx`, `UnitOfWork`, repo traits, `StorageError` | D0.6 | T09 | 2.0 | BE | ☐ |
| P0-T18 | `storage-sqlite`: dual pools, pragmas, tx semantics, health | D0.6 | T17 | 2.5 | BE | ☐ |
| P0-T19 | Migration runner: apply, checksum verify, status, plan, backup/restore | D0.7 | T18 | 2.5 | BE | ☐ |
| P0-T20 | Migrations `0001_core`, `0005_audit` | D0.6, D0.12 | T19 | 1.5 | BE | ☐ |
| P0-T21 | ADR-0004 / 0005 | ADR | T12, T15 | 1.0 | DOC | ☐ |

**Non-negotiables in this sprint**
- **T16 is a permanent CI gate**, reused by every future phase. It must put a sentinel value
  into *every* backend, then assert the sentinel appears in **zero output bytes** across log
  emission, error formatting, `config show`, `doctor --json`, and audit serialization.
- **T19 backup/restore must use the SQLite online backup API or `VACUUM INTO`** — never a raw
  `cp`/`fs::copy` of the live file. Copying while the WAL is active can produce a torn snapshot
  (ADR-0001 §7 rules this out).
- **T18**: write pool `max_connections = 1` (serialized); read pool N connections with
  `query_only = ON`.
- `StorageError` variants are fixed in T17 (`Conflict`, `NotFound`, `ImmutableSourceVersion`,
  `ConstraintViolation`, `StorageBusy`, `MigrationRequired`, `MigrationChecksumMismatch`,
  `IdempotencyKeyReplay`, `StorageUnavailable`) so later phases cannot invent divergent names.

---

## 4. Sprint 0.3 — Provenance, Audit, Sources, Outbox (Week 3) — 33.5 ed ⚠️ overloaded

⚠️ **This sprint is 33.5 ed — nearly double Sprint 0.2.** It also contains two tasks with
forward dependencies (see §5.2). Re-plan before starting.

### 4.1 Provenance & audit

| ID | Task | Deliv. | Depends | Est | Role | Status |
|---|---|---|---|---|---|---|
| P0-T22 | Migration `0003_provenance` + triggers + `CHECK` constraints | D0.11 | T20 | 1.5 | BE | ☐ |
| P0-T23 | `provenance` crate: record model, repository, invariant tests | D0.11 | T22 | 2.5 | BE | ☐ |
| P0-T24 | `ApprovalToken`, `CanonicalWriter`, `CanonicalChangeSession` | D0.11 | T23 | 2.0 | BE | ☐ |
| P0-T25 | `review_queue` model + accept/reject/correct API + evidence requirement | D0.11 | T23 | 1.5 | BE | ☐ |
| P0-T26 | `audit` crate: event model, hash-chain writer, verifier, `AuditAction` enum | D0.12 | T20 | 2.5 | BE | ☐ |
| P0-T27 | Audit redaction + allowlisted payload schema + leak tests | D0.12 | T26, T16 | 1.0 | SEC | ☐ |

> **T23 must include risk-R2 validation:** review the provenance model against three concrete
> future cases — a tafsir claim, a hadith grading, and a narrator possible-identity — before
> the model is frozen. Keep `attribution_json` extensible.
>
> **T24 is the core integrity mechanism.** `ApprovalToken` must be obtainable *only* from a
> persisted `ApprovalRecord` created by a human principal holding `canonical:approve`. It must
> not be constructible by agents, tools, or importers. `CanonicalChangeRequest` must require
> all five §7.3 fields (new source version, checksum result, structural validation result,
> difference report, approver identity); a unit test asserts each missing field is rejected.

### 4.2 Source catalog

| ID | Task | Deliv. | Depends | Est | Role | Status |
|---|---|---|---|---|---|---|
| P0-T28 | Migration `0002_sources` | D0.10 | T20 | 1.0 | BE | ☐ |
| P0-T29 | `sources`: manifest parse + JSON-Schema + semantic validation | D0.10 | T28, T07 | 2.5 | BE | ☐ |
| P0-T30 | `sources`: ed25519 signature verification + unsigned policy | D0.10 | T29 | 1.5 | SEC | ☐ |
| P0-T31 | `sources`: state machine + transition log + approval preconditions | D0.10 | T28, T24 | 2.5 | BE | ☐ |
| P0-T32 | `sources`: genealogy resolver, cycle detection, lineage rendering | D0.10 | T28 | 1.5 | BE | ☐ |
| P0-T33 | `sources`: `StructureValidator` registry + `DifferenceReport` framework | D0.10 | T31 | 1.5 | BE | ☐ |
| P0-T34 | ADR-0007 / 0008 / 0009 | ADR | T30, T23, T26 | 1.5 | DOC | ☐ |

> **T31 encodes the rules Phase 1 depends on:** `Approved` requires license status ≠ `Unknown`,
> a verified `ContentHash`, a completed structural validation report, **and** a human
> `ApprovalRecord` — because §22.3 forbids any internet source becoming active solely because
> an LLM recommended it. `Active` is set only by an atomic pointer flip, with exactly one
> `Active` version per `(source_id, role)` enforced by a partial unique index.
>
> **T30 default policy:** unsigned manifests rejected for remote sources
> (`allow_unsigned_manifests = false`), permitted for local files.

### 4.3 Outbox, generations, tombstones (D0.18)

Adopted from ADR-0702 §§2–4, 9 — the **relational primitives only**. Multi-store
publish/reconcile/doctor-repair remains Phase 7. These sit in this sprint because they share
transactions with provenance and sources.

| ID | Task | Deliv. | Depends | Est | Role | Status |
|---|---|---|---|---|---|---|
| P0-T61 | Migration `0006_outbox_generations_tombstones` | D0.18 | T20 | 1.0 | BE | ☐ |
| P0-T62 | `CorpusGeneration` allocator: transactional monotonicity + concurrency test | D0.18 | T61 | 1.5 | BE | ☐ |
| P0-T63 | `OutboxRepository` + wiring into `sources`/`provenance` write paths | D0.18 | T61, T23, T31 | 2.0 | BE | ☐ |
| P0-T64 | Generic outbox-relay job (`system.outbox_relay`) reusing job lease/heartbeat | D0.18 | T61, **T37** | 1.0 | BE | ⊘ |
| P0-T65 | `Tombstone` model + wiring into source deactivation/rollback | D0.18 | T61, T31 | 1.5 | BE | ☐ |
| P0-T66 | Doctor checks: `outbox.backlog_age`, `outbox.dead_letter_count`, `generations.monotonicity`, `tombstones.unpropagated_count` | D0.14 | T61, T63, T65, **T52** | 1.5 | BE | ⊘ |
| P0-T67 | Cross-store consistency test suite subset (§8.3) | D0.16 | T63, T62 | 2.0 | BE | ☐ |

> **T63 is the invariant, not a convenience.** Every write that changes projection-relevant
> authoritative state (source activation, provenance write, canonical-change commit) **must**
> insert an outbox row **in the same transaction**. Wire `OutboxRepository` into `UnitOfWork`
> so committing the change durably implies committing its outbox row. T67 proves this by
> fault injection: a crash between the two writes must roll both back — the guarantee is
> "impossible by construction", not "fixed by retry".
>
> **T64 has no consumers yet** (no FTS/vector/graph exist). That is expected and correct. What
> matters is that the contract, table, and commit-implies-outbox guarantee exist before Phase 2
> needs them.
>
> **T65:** deactivation/rollback must write a tombstone row **and** an outbox event — not just
> flip `state`. Retrieval consumers don't exist yet, but the rule "current policy blocks
> retrieval immediately even while physical cleanup is pending" (ADR-0702 §9) needs the table
> to exist so nothing in Phase 1+ bolts it on later.
>
> **T62:** allocation happens inside the same write transaction as the authoritative change.
> SQLite's single write pool already serializes this — document the equivalent PostgreSQL
> `SELECT … FOR UPDATE` clause now so Phase 1/2 don't relearn it.

---

## 5. Sequencing & Estimate Corrections — Resolve Before Sprint 0.1

Three inconsistencies exist between `plan.md`'s WBS tables and its stated totals. They are
recorded here rather than silently propagated.

### 5.1 Estimate total does not reconcile

| Sprint | Sum of task estimates |
|---|---|
| 0.1 | 14.5 ed |
| 0.2 | 19.0 ed |
| 0.3 | 33.5 ed (23.0 core + 10.5 D0.18) |
| 0.4 | 22.0 ed |
| 0.5 | 24.5 ed |
| **Total** | **113.5 ed** |

`plan.md` §7 states "Total ≈ 77.5 ed (68 + 9.5 from D0.18) ⇒ ~5 weeks with 3 engineers".
The task rows sum to **113.5 ed**, and the D0.18 block sums to **10.5 ed**, not 9.5.

At 3 engineers, 113.5 ed is **~7.5 weeks** before review overhead and slack — not 5 weeks.
**Owner decision required** (see `readme.md` §9.1): accept ~7.5 weeks, cut scope explicitly,
or add a fourth engineer for Sprints 0.3–0.5. Do not resolve this by compressing estimates.
D0.11 (T22–T25), D0.12 (T26–T27), and D0.15 (T43–T46) are retrofit-impossible and must not
absorb the cut; risk R1 nominates D0.17 documentation depth as the first cut.

### 5.2 Two tasks depend on later sprints

| Task | Sprint | Depends on | That task's sprint | Resolution |
|---|---|---|---|---|
| P0-T64 outbox-relay job | 0.3 | T37 worker pool | **0.4** | Move T64 → Sprint 0.4, after T37 |
| P0-T66 outbox doctor checks | 0.3 | T52 doctor engine | **0.5** | Move T66 → Sprint 0.5, after T52 |

Both are marked ⊘ **Blocked** above until re-planned. This also relieves Sprint 0.3 by 2.5 ed
(33.5 → 31.0), which does not solve its overload but helps.

### 5.3 Sprint 0.3 is overloaded

31.0–33.5 ed in one week against a 3-engineer team (~15 ed/week) is a ~2× overrun, and this is
the sprint containing the retrofit-impossible integrity work (T24 `ApprovalToken`, T26 audit
chain, T31 state machine). Recommended split: keep provenance + audit + sources in 0.3; move
the entire D0.18 block (T61–T67) to a new Sprint 0.3b or into 0.4.

---

## 6. Sprint 0.4 — Jobs, Security Guards, Observability (Week 4) — 22.0 ed

| ID | Task | Deliv. | Depends | Est | Role | Status |
|---|---|---|---|---|---|---|
| P0-T35 | Migration `0004_jobs` | D0.9 | T20 | 0.5 | BE | ☐ |
| P0-T36 | Job repository: enqueue, claim-with-lease, heartbeat, finish, cancel | D0.9 | T35 | 2.5 | BE | ☐ |
| P0-T37 | Worker pool, handler registry, payload-schema validation | D0.9 | T36 | 2.0 | BE | ☐ |
| P0-T38 | Cancellation, deadlines, checkpoint/resume, progress reporting | D0.9 | T37 | 2.0 | BE | ☐ |
| P0-T39 | Retry/backoff/jitter, dead-lettering, interrupted-run recovery scan | D0.9 | T37 | 1.5 | BE | ☐ |
| P0-T40 | Job chaos tests (kill worker, expire lease, duplicate enqueue, resume) | D0.16 | T39 | 2.0 | BE | ☐ |
| P0-T41 | `observability`: subscriber, span conventions, metric catalog | D0.8 | T12 | 2.0 | BE | ☐ |
| P0-T42 | OTLP exporter (opt-in) + telemetry field-denylist test | D0.8 | T41 | 1.5 | BE | ☐ |
| P0-T43 | `security::path`, `security::archive` + attack-corpus tests | D0.15 | T09 | 2.0 | SEC | ☐ |
| P0-T44 | `security::net` SSRF guard (resolve-then-check, redirect policy) | D0.15 | T09 | 2.0 | SEC | ☐ |
| P0-T45 | `security::limits`, `::input`, `::sanitize`, `Untrusted<T>` | D0.15 | T09 | 2.0 | SEC | ☐ |
| P0-T46 | `policy::baseline` deny-by-default decision engine | D0.15 | T09 | 1.0 | SEC | ☐ |
| P0-T47 | ADR-0003 / 0011 | ADR | T37, T41 | 1.0 | DOC | ☐ |

**Job guarantees each task must land with a test** (`plan.md` D0.9):

| Guarantee | Mechanism | Test | Task |
|---|---|---|---|
| At-least-once with idempotency | `UNIQUE(kind, idempotency_key)` + handler flag | duplicate-enqueue | T36, T40 |
| Crash recovery | expired-lease reclaim; `Running` → `Interrupted` on startup scan | kill-worker | T39, T40 |
| Cancellation propagates | `cancel_requested` polled + `CancellationToken` | cancel-mid-run | T38 |
| Resume from checkpoint | `checkpoint` JSON replayed to handler | resume | T38, T40 |
| No partial activation | handlers write to staging, flip pointer in one tx | atomicity | T38 |
| Bounded retries + jitter | `attempts`, exponential backoff, `DeadLettered` | backoff distribution | T39 |
| Non-idempotent ops never auto-retry | handler flag gate | policy | T39 |

Phase 0 job kinds (real, useful): `system.integrity_scan`, `system.vacuum`,
`source.validate_manifest`, `source.compute_hashes`, `audit.verify_chain`,
`system.noop_test`, `system.outbox_relay` (T64).

**Fail-closed rule (T43–T46):** if a guard cannot be enforced — cannot canonicalize a path,
cannot resolve DNS to check ranges — the operation is **denied**, not allowed.

**T45 note (risk R10):** `Untrusted<T>` is introduced now specifically because §74 and §92:36
are unenforceable if added later. Ship it with the `xtask`/clippy check that greps for
`.into_inner()` outside allowlisted sanitizer modules.

**T42 privacy gate:** compile-time-tested denylist — query text, prompt text, document
content, research questions, model responses must never be exportable.

---

## 7. Sprint 0.5 — CLI, Doctor, Hardening, Docs (Week 5) — 24.5 ed

| ID | Task | Deliv. | Depends | Est | Role | Status |
|---|---|---|---|---|---|---|
| P0-T48 | CLI framework: `clap` tree, global flags, `--json` renderer, exit codes | D0.13 | T09, T12 | 2.5 | BE | ☐ |
| P0-T49 | `config`, `db`, `secret` command groups | D0.13 | T48 | 2.0 | BE | ☐ |
| P0-T50 | `source`, `job`, `audit` command groups | D0.13 | T48, T31 | 2.5 | BE | ☐ |
| P0-T51 | Phase-N stub commands + shell completions + CLI conformance test | D0.13 | T48 | 1.5 | BE | ☐ |
| P0-T52 | `doctor` engine: check registry, read-only enforcement, severity, remedies | D0.14 | T48 | 2.5 | BE | ☐ |
| P0-T53 | Phase-0 doctor checks (all listed in D0.14) + JSON schema | D0.14 | T52 | 2.5 | BE | ☐ |
| P0-T54 | `--repair-preview` planner (no mutation) | D0.14 | T52 | 1.0 | BE | ☐ |
| P0-T55 | `serve` stub: `/healthz`, `/readyz`, `/api/v1/meta`, localhost bind guard | D0.1 | T48 | 1.5 | BE | ☐ |
| P0-T56 | Dockerfile + compose stub + non-root runtime | D0.1 | T55 | 1.5 | INF | ☐ |
| P0-T57 | `testkit` finalization + fixtures + deterministic clock/UUID | D0.16 | T18 | 2.0 | BE | ☐ |
| P0-T58 | Architecture docs, runbooks, `CONTRIBUTING`/DoD PR template | D0.17 | all | 2.5 | DOC | ☐ |
| P0-T59 | ADR-0010 + ADR index + template lint (all §48 fields present) | ADR | T09 | 1.0 | DOC | ☐ |
| P0-T59b | Reconcile ADR numbering scheme + rename ADR files | ADR | T59 | 0.5 | DOC | ☐ |
| P0-T60 | Phase-0 exit-gate review, AC verification, Phase-1 handoff doc | — | all | 1.5 | all | ☐ |

**T52/T53 hard rules**
- `doctor` opens the database **read-only** via the read-only pool. Asserted by a test that runs
  `doctor` against a file-permission-restricted DB.
- Repairs are **never** automatic. `--repair-preview` prints the plan only; `qai repair <check-id>`
  is Phase 1+, with confirmation and an audit event.
- Every check emits `Pass | Warn | Fail | Skipped` **plus a remedy and a next command**.

**T51 CLI conformance test enforces:** verbs after nouns; `--json` on every read command;
destructive commands confirm unless `--yes`; the documented exit-code table.

**T59b actions** (adopt the phase-coded scheme `ADR-00nn`/`ADR-02nn`/`ADR-07nn`):
1. Rename `ADR-0004-relational-store.md` → `ADR-0001-relational-store.md` (its content already
   says ADR-0001; only the filename is wrong).
2. Retitle `ADR-0002-database.md` → `docs/architecture/database-engine-survey.md` (it is a
   survey/recommendation memo, not an ADR) **and** write the real ADR-0002 "Migration Strategy".
3. Update the Phase 0 ADR table to phase-coded IDs and re-run the ADR lint (AC-P0-19).

**T60 gate:** cannot close until all 26 ACs in `acceptance.md` pass and the recorded
exit-gate ritual (AC-03, 05, 06, 08, 11, 14, 16, live on a clean machine) is archived.

---

## 8. Board Rollup

| Sprint | Tasks | Est (ed) | Done | Status |
|---|---|---|---|---|
| X — Cross-phase decisions | 3 | — | 0 | ☐ Not Started |
| 0.1 — Skeleton & Contracts | 11 | 14.5 | 0 | ☐ Not Started |
| 0.2 — Config, Secrets, Storage | 10 | 19.0 | 0 | ☐ Not Started |
| 0.3 — Provenance, Audit, Sources, Outbox | 20 | 33.5 ⚠️ | 0 | ☐ Not Started |
| 0.4 — Jobs, Security, Observability | 13 | 22.0 | 0 | ☐ Not Started |
| 0.5 — CLI, Doctor, Docs | 14 | 24.5 | 0 | ☐ Not Started |
| **Total** | **67 + 3** | **113.5** | **0** | **0%** |

By role: BE ≈ 76.0 ed · SEC ≈ 15.5 ed · INF ≈ 8.0 ed · DOC ≈ 12.5 ed · shared 1.5 ed.
