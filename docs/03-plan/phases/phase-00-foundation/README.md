# Phase 0 — Foundations & Provenance

**Phase ID:** P0
**Plan:** [`plan.md`](./plan.md) v1.0.0
**PRD Baseline:** Q-ai PRD v0.3.2
**Depends On:** — (first phase)
**Blocks:** Phase 1 (Canonical Quran Core) and every later phase
**Target Duration:** 5 calendar weeks (see §9 — capacity caveat)
**Status:** 🔴 Not Started

---

## 1. What This Phase Is

Phase 0 builds the **chassis**: types, storage, provenance, jobs, config, secrets, audit, CLI,
and CI. **No religious corpora are touched.** Not one Quran byte is imported.

> **One-sentence goal:** a trustworthy, observable, testable skeleton with
> immutable-canonical and provenance semantics already enforced by types and the database —
> *before* any Quran byte is imported.

### Why it must come first

Every integrity guarantee in PRD §46/§56/§93 is enforced by mechanisms built here. Three of
them are **unenforceable if retrofitted**:

| Mechanism | Deliverable | Why it cannot be added later |
|---|---|---|
| `ApprovalToken` + `CanonicalWriter` | D0.11 | If canonical rows are ever writable through a normal repository, the guarantee is gone — code will already depend on the loose path. |
| `Untrusted<T>` taint wrapper | D0.15 | PRD §74 / §92:36 require explicit unwrapping at every boundary; adding it later means auditing every call site. |
| Outbox + generation stamping | D0.18 | "Commit implies durable outbox row" must be a transaction-level invariant, not a retry loop bolted on in Phase 7. |

---

## 2. Document Set

| File | Purpose | Update cadence |
|---|---|---|
| `readme.md` | This file. Phase entry point, orientation, conventions. | On scope change |
| `plan.md` | The authoritative Phase 0 plan (deliverables, schema, ADRs, risks). | On scope change (versioned) |
| `tasks.md` | Live work board: 67 tasks + 3 critical-path decisions, with dependencies and status. | Daily |
| `acceptance.md` | The 26 exit-gate criteria, verification method, and evidence log. | On AC verification |
| `done.md` | Append-only completion ledger (tasks, ACs, ADRs, decisions). | On every completion |

**Rule:** `plan.md` states intent, `tasks.md` states current state, `done.md` states history.
Never edit `done.md` entries after the fact — append a correction instead.

---

## 3. Objective (8 Capabilities)

Deliver a production-shaped Rust workspace that can:

1. Represent **all five trust layers** (§6) as first-class, non-mergeable data.
2. Persist, migrate, and version data in SQLite behind a storage abstraction that can later
   become PostgreSQL (§32.1) **without domain changes**.
3. Record **provenance and audit** for every mutation, with source-version pinning (§39, §82).
4. Run **cancellable, resumable, idempotent** background jobs (§34, §41, §54).
5. Load and validate **configuration with correct precedence**; handle **secrets** without
   ever leaking them (§25.13).
6. Expose a `qai` CLI skeleton and a `qai doctor` that **never mutates data** (§50).
7. Define the **source manifest schema** and source lifecycle state machine (§22).
8. Enforce a **security baseline** and **deny-by-default** posture from day one (§37, §84).

---

## 4. Scope Fence

### In scope

Workspace/toolchain/lints · domain model & newtype IDs · typed error model · layered config ·
secret handling · storage abstraction + SQLite · migration framework · logging/metrics/tracing ·
durable job system · source catalog + manifest + state machine · provenance model (A–E) ·
hash-chained audit · `qai` CLI skeleton · `qai doctor` core · security guard libraries ·
test harness + CI + coverage gates · ADR process + docs · outbox/generation/tombstone primitives.

### Out of scope — do not build these in Phase 0

| Excluded | Lands in |
|---|---|
| Any Quran / hadith / tafsir / scripture text or schema | Phase 1 / 5 / 8 |
| Full-text index, vector store, embeddings, graph store | Phase 2 / 3 / 7 |
| LLM / model provider adapters | Phase 9 |
| Tool runtime, sandbox, agents | Phase 9 / 10 |
| Web GUI, TUI screens | Phase 4 / 11 (`serve` returns health only) |
| Internet source **download** (schema + state machine *are* in scope) | Phase 10 |
| Auth / RBAC beyond local single-principal | Phase 11 |

### Non-negotiable design constraints adopted now

1. **Canonical immutability is a type-level property**, not a convention — canonical rows are
   written only through `CanonicalWriter` requiring an `ApprovalToken`.
2. **No domain crate may depend on** `api`, `server`, `tui`, `cli`, or any concrete provider (§33).
3. **Every derived artifact records the version of everything it was derived from** (§76).
4. **Deny-by-default**: permissions, network, filesystem, and side effects default to empty sets.
5. **Nothing that can modify data runs inside `doctor`** (§50).

---

## 5. Deliverable Index

| ID | Deliverable | Primary crates | ACs |
|---|---|---|---|
| D0.1 | Workspace, toolchain & build tooling (`xtask`, `deny.toml`, Docker) | `xtask` | 01, 02 |
| D0.2 | Core domain model, typed IDs, `ContentHash`, canonical JSON | `domain` | 17 |
| D0.3 | Typed error model (`Diagnostic`, code registry) | `domain` | 17 |
| D0.4 | Layered configuration (CLI > Env > File > Defaults) | `config` | 04, 16 |
| D0.5 | Secret management (env / keychain / age file) + redaction | `config`, `security` | 05 |
| D0.6 | Storage abstraction + SQLite implementation (dual pools) | `storage`, `storage-sqlite` | 03 |
| D0.7 | Migration framework (append-only, checksummed) + backup/restore | `storage-sqlite` | 03, 21 |
| D0.8 | Observability (spans, metric catalog, opt-in OTLP) | `observability` | 18 |
| D0.9 | Durable job system (leases, cancel, checkpoints, backoff) | `jobs` | 11, 12 |
| D0.10 | Source catalog, manifest schema v1, state machine, genealogy | `sources` | 08, 09, 10 |
| D0.11 | Provenance model + `ApprovalToken` / `CanonicalWriter` + review queue | `provenance` | 06, 07 |
| D0.12 | Append-only hash-chained audit log | `audit` | 13 |
| D0.13 | `qai` CLI skeleton, global flags, `--json`, exit codes | `cli` | 17 |
| D0.14 | `qai doctor` core checks + `--repair-preview` | `cli` | 14, 26 |
| D0.15 | Security baseline (path/archive/SSRF/limits/taint/policy guards) | `security` | 15 |
| D0.16 | Test harness, `testkit`, CI pipeline, coverage gates | `testkit` | 01, 22 |
| D0.17 | Docs, ADRs, runbooks, `.env.example`, Dockerfile | `docs/`, `adr/` | 19, 20 |
| D0.18 | Outbox, corpus-generation stamping, tombstone primitives | `storage`, `domain` | 23–26 |

---

## 6. Crate Graph & Dependency Rule

```text
                     ┌────────────┐
                     │   config   │  (no deps on domain)
                     └─────┬──────┘
                           │
┌──────────┐      ┌────────▼────────┐      ┌────────────┐
│  domain  │◄─────│   application   │─────►│   audit    │
└────┬─────┘      └───┬────────┬────┘      └─────┬──────┘
     │                │        │                 │
     │          ┌─────▼──┐  ┌──▼─────┐     ┌─────▼──────┐
     │          │ jobs   │  │sources │     │ provenance │
     │          └─────┬──┘  └──┬─────┘     └─────┬──────┘
     │                │        │                 │
     │            ┌───▼────────▼─────────────────▼───┐
     └───────────►│            storage               │
                  │  (traits) + storage-sqlite       │
                  └──────────────┬───────────────────┘
                                 │
                        ┌────────▼────────┐
                        │  observability  │
                        └────────┬────────┘
                                 │
                    ┌────────────▼────────────┐
                    │   cli   │  server(stub) │
                    └─────────────────────────┘
```

Enforced in CI by `cargo-deny` + `cargo xtask arch-check`:

```text
domain          -> (serde, thiserror, time, uuid) only. No I/O, no async runtime.
application     -> domain, storage(traits), provenance, audit, jobs, sources, config
storage         -> domain            (traits + errors only)
storage-sqlite  -> storage, domain, sqlx
provenance      -> domain, storage
audit           -> domain, storage
jobs            -> domain, storage, observability
sources         -> domain, storage, provenance
cli             -> application, config, observability
server          -> application, config, observability
observability   -> (tracing, metrics) only
```

Crates created in Phase 0: `domain`, `application`, `config`, `observability`, `storage`,
`storage-sqlite`, `jobs`, `provenance`, `audit`, `sources`, `cli`, `server`, `xtask`, `testkit`.
All other PRD §87 crates exist as **empty placeholders** with a `//! Phase N` doc comment so the
workspace graph is visible from day one. Placeholders must stay empty (see risk R1).

---

## 7. Migrations Delivered

| File | Contents |
|---|---|
| `0001_core.up.sql` | `schema_migrations`, `principals`, `workspaces`, `settings`, `blobs` |
| `0002_sources.up.sql` | `sources`, `source_versions`, `source_files`, `source_genealogy`, `source_state_transitions`, `approvals` |
| `0003_provenance.up.sql` | `provenance_records` (+ immutability triggers), `review_queue` |
| `0004_jobs.up.sql` | `jobs`, `job_events` |
| `0005_audit.up.sql` | `audit_events` (+ append-only triggers) |
| `0006_outbox_generations_tombstones.up.sql` | `corpus_generations`, `outbox_events`, `tombstones` (+ append-only triggers) |

Migrations are **append-only and checksummed**. Editing an applied migration is a hard,
coded failure (`QAI-DB-0003`). `.down.sql` required for all non-canonical tables.

---

## 8. Quick Start

```bash
# Full gate — must be green before any PR merges
cargo xtask ci          # fmt, clippy -D warnings, test, deny, arch-check, migrate-check

# Individual gates
cargo xtask arch-check      # dependency-direction enforcement
cargo xtask migrate-check   # migrations additive + checksums unchanged
cargo xtask gen-schema      # emit JSON Schemas -> docs/schemas/ (must be committed)

# Runtime
qai db migrate
qai doctor --json
qai config show --explain
qai serve               # health-only in Phase 0; binds 127.0.0.1
```

**Error code namespaces** (registry: `docs/architecture/error-codes.md`):
`QAI-CFG` · `QAI-SEC` · `QAI-DB` · `QAI-JOB` · `QAI-SRC` · `QAI-PROV` · `QAI-AUD` · `QAI-CLI`
Reserved for later phases: `QAI-QUR` (P1) · `QAI-NORM` (P2) · `QAI-IDX` (P2).

**CLI exit codes:** `0` ok · `1` generic · `2` usage · `3` validation failed ·
`4` denied by policy · `5` not found · `6` conflict/state · `7` cancelled · `70` internal.
Codes are **public API** — see `CONTRIBUTING.md`.

---

## 9. Scheduling Reality Check — Read Before Committing To Dates

Two things in `plan.md` need an owner decision before the phase is scheduled.

### 9.1 Capacity

The plan states "≈ 77.5 ed ⇒ ~5 weeks with 3 engineers". **Summing the individual task
estimates in the WBS gives 113.5 ed**, which is ~7.5 weeks at 3 engineers before review
overhead and slack. See `tasks.md` §5 for the per-sprint arithmetic.

Pick one before Sprint 0.1 starts:
- **(a)** Accept ~7.5 weeks and revise the target duration; or
- **(b)** Hold 5 weeks and cut scope explicitly — risk R1's mitigation nominates D0.17
  documentation depth as the first cut, then D0.18 doctor checks (P0-T66); or
- **(c)** Add a fourth engineer for Sprints 0.3–0.5 (the two heaviest, 33.5 ed and 24.5 ed).

Do **not** resolve this by silently compressing estimates. D0.11, D0.12, and D0.15 are the
retrofit-impossible deliverables and must not absorb the cut.

### 9.2 Three cross-phase decisions with external lead times

These are **not on Phase 0's own critical path**, which makes them easy to deprioritize —
and each has a lead time engineering effort cannot compress later.

| Decision | ADR | Needed by | Bottleneck |
|---|---|---|---|
| Quran text dataset selection & licensing | ADR-0101 | End of Phase 0 | Licensing review, source verification, attribution terms |
| Morphology dataset selection & licensing | ADR-0203 | Start of Phase 2 | Same, plus dataset-shape evaluation |
| Normalization rule catalog + linguist engagement | ADR-0204 | Start of Phase 2 | **Sourcing a qualified Arabic linguist**, not the writing |

Engineering delay is recoverable (add people, extend a sprint). A late *decision* here is not
— it stalls an entire phase's start, and qualified linguists book out weeks ahead.

**Action:** in the **first standup**, assign an owner and a "decision-open" date to each,
independent of Phase 0 task assignments. Tracked as swimlane **X** in `tasks.md` §1 — its own
board lane, *not* line items inside Sprints 0.1–0.5.

---

## 10. ADRs Required In This Phase

`ADR-0001` relational store · `ADR-0002` migration strategy · `ADR-0003` durable job system ·
`ADR-0004` config & precedence · `ADR-0005` secret storage · `ADR-0006` hashing & canonical
serialization · `ADR-0007` manifest format & signing · `ADR-0008` provenance representation ·
`ADR-0009` audit integrity · `ADR-0010` error taxonomy & exit codes · `ADR-0011` observability
stack · `ADR-0012` crate boundaries · `ADR-0702` (adopted early, §§2–4 + 9 only).

> ⚠️ **Numbering collision — resolve via P0-T59b before writing more ADRs.**
> Adopted decision: **phase-coded scheme everywhere** (`ADR-00nn` Phase 0, `ADR-02nn` Phase 2,
> `ADR-07nn` Phase 7). Required fixes: rename `ADR-0004-relational-store.md` →
> `ADR-0001-relational-store.md`; move `ADR-0002-database.md` out of the ADR sequence (it is a
> survey memo, not the migration-strategy ADR) and write the real ADR-0002.

Every ADR uses the §48 template including **Accuracy implications**, **Religious-source
implications**, and **Licensing implications**. For Phase 0 ADRs the religious-source section is
usually "none directly, but constrains Phase 1 canonical integrity" — **that reasoning must be
written down, not omitted** (checked by AC-P0-19).

---

## 11. Exit Gate

Phase 0 is complete when **all 26 criteria in `acceptance.md` pass** and the
**exit-gate ritual** is recorded: a reviewer performs `AC-P0-03`, `05`, `06`, `08`, `11`, `14`,
and `16` **live on a clean machine**, on video.

Handoff artifact: `docs/plans/handoff-p0-to-p1.md` (task P0-T60), listing every asset Phase 1
inherits and must not re-invent, plus known limitations and deferred items **with owners**.

---

## 12. Top Risks

| # | Risk | L | I | Mitigation |
|---|---|---|---|---|
| R1 | Over-engineering the chassis; Phase 1 slips | High | High | Hard scope fence §4; placeholders stay empty; timebox sprints; cut D0.17 depth before D0.11/D0.12 |
| R2 | Provenance model too rigid for hadith gradings / narrator uncertainty | Med | High | Review against 3 concrete future cases during P0-T23; keep `attribution_json` extensible |
| R6 | Audit hash-chain gaps under concurrent writers | Med | High | Chain writes serialized through the single write pool, same transaction as the audited change |
| R4 | Canonical-JSON/hashing decision changes later, invalidating hashes | Low | High | Algorithm tag inside `ContentHash`; spec frozen in ADR-0006 with rehash-migration procedure |
| R10 | Team treats `Untrusted<T>` as ceremony and bypasses it | Med | High | `xtask` grep for `.into_inner()` outside allowlisted sanitizers; PR checklist item |

Full register: `plan.md` §10 (R1–R10).
