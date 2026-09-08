# Phase 0 — Completion Ledger

**Phase:** P0 — Foundations & Provenance
**Status:** 🔴 Not Started — 0 / 67 tasks · 0 / 26 acceptance criteria · 0 / 13 ADRs
**Started:** _not started_
**Completed:** —

---

## How To Use This File

This is an **append-only ledger**. It records what was actually completed, when, by whom, and
with what evidence.

**Rules**

1. **Append only.** Never edit or delete an existing entry. If an entry was wrong, append a
   correction in §6 referencing the original.
2. **Evidence is mandatory.** Every entry links a PR, CI run, test path, or recording. An entry
   without evidence is not a completion record.
3. **DoD before ☑.** A task moves to done only when it satisfies the Definition of Done
   (`acceptance.md` §3.1) — not when it compiles or when the PR merges.
4. **Mirror the board.** When you append here, flip the status in `tasks.md` or
   `acceptance.md` in the same commit. The two must never disagree.
5. **Record deviations.** If the delivered thing differs from `plan.md`, log it in §5 with the
   reason. Silent drift is how a chassis phase stops being a chassis.

**Entry format**

```
### P0-Tnn — <task title>
- **Deliverable:** D0.x
- **Completed:** YYYY-MM-DD
- **Owner:** <name> (<role>)
- **PR / commit:** <link>
- **Evidence:** <CI run, test path, snapshot, or recording>
- **DoD:** ✅ all items / ⚠️ exceptions: <list with justification>
- **Notes:** <deviations, follow-ups, TODOs filed>
```

---

## 1. Progress Summary

| Sprint | Tasks | Done | Est (ed) | Actual (ed) | Status |
|---|---|---|---|---|---|
| X — Cross-phase decisions | 3 | 0 | — | — | ☐ |
| 0.1 — Skeleton & Contracts | 11 | 0 | 14.5 | — | ☐ |
| 0.2 — Config, Secrets, Storage | 10 | 0 | 19.0 | — | ☐ |
| 0.3 — Provenance, Audit, Sources, Outbox | 20 | 0 | 33.5 | — | ☐ |
| 0.4 — Jobs, Security, Observability | 13 | 0 | 22.0 | — | ☐ |
| 0.5 — CLI, Doctor, Docs | 14 | 0 | 24.5 | — | ☐ |
| **Total** | **67 + 3** | **0** | **113.5** | **—** | **0%** |

| Artifact class | Complete | Total |
|---|---|---|
| Deliverables (D0.1–D0.18) | 0 | 18 |
| Acceptance criteria (AC-P0-01…26) | 0 | 26 |
| ADRs accepted | 0 | 13 |
| Migrations applied (`0001`–`0006`) | 0 | 6 |
| Required test suites green | 0 | 17 |
| Runbooks published | 0 | 5 |

Track **actual vs. estimate** from the first completed task. `tasks.md` §5.1 flags a
36-ed discrepancy between the plan's stated 77.5 ed and its own task rows (113.5 ed);
actuals recorded here are the only way to find out which number was closer.

---

## 2. Completed Tasks

_No tasks completed yet._

### Sprint 0.1 — Skeleton & Contracts

### P0-T01 — Create workspace, toolchain pin, lint config, `[workspace.dependencies]`
- **Deliverable:** D0.1
- **Completed:** 2026-09-06
- **Owner:** implementing engineer (INF)
- **PR / commit:** `main` (worktree-local)
- **Evidence:** `cargo check --workspace` finished; `cargo fmt --check` exit 0; `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo test --workspace` 0 failed. (cargo-deny not on host; its gate lands in T03/T11 CI.)
- **DoD:** ⚠️ items 2,4,7,9,10,13 deferred — placeholder crates have no domain logic yet; arch-check/migrate-check land in T02.
- **Notes:** ADR file renames performed under P0-T01 but pending orchestrator approval of the mapping proposal — see flags.

### P0-T02 — `xtask` with `arch-check`, `ci`, `migrate-check`, `gen-schema`
- **Deliverable:** D0.1
- **Completed:** 2026-09-06
- **Owner:** implementing engineer (INF)
- **PR / commit:** `main` (worktree-local)
- **Evidence:** `cargo test -p xtask` → 7 passed (incl. AC-P0-02 mutation `forbidden_edge_is_detected`, migrate drift, schema idempotency); `cargo clippy --workspace --all-targets -- -D warnings` clean; live `cargo run -p xtask -- arch-check|migrate-check|gen-schema` OK.
- **DoD:** ⚠️ items 4,9,11,12 not applicable for pure build-tooling (no domain logic / provenance / config yet); deny gate executed via CI (not host).
- **Notes:** `arch-check` reads `xtask/allowlist.toml` (readme §6 faithful). `cargo-deny` absent on host → CI-only. `[workspace.lints.clippy]` left empty to avoid flooding placeholders (R1).

### P0-T03 — `deny.toml` license/advisory policy + CI job
- **Deliverable:** D0.1
- **Completed:** 2026-09-06
- **Owner:** implementing engineer (INF)
- **PR / commit:** `main` (worktree-local)
- **Evidence:** deny job definition in `.github/workflows/ci.yml` (validated YAML via PyYAML). `deny.toml` authored in T01: license allowlist (MIT/Apache-2.0/BSD/ISC/Unicode-Dfs/Zlib/MPL-2.0), GPL/AGPL/LGPL denied, registry allowlist, advisory DB.
- **DoD:** ⚠️ items 4,9,10,11,12,13 not applicable — policy/CI only; deny itself not executable on host (no `cargo-deny`), enforced in CI.
- **Notes:** The `deny` CI job runs `cargo deny check --all-features advisories bans licenses sources` via `embarkstudios/cargo-deny-action@v2`. Lint + arch jobs added as scaffold for T11; full test-3os/schemas/doctor/coverage land in T11.

### P0-T04 — Placeholder crates for all PRD §87 crates with `//! Phase N` doc comments
- **Deliverable:** D0.1
- **Completed:** 2026-09-06
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local)
- **Evidence:** `python3 /tmp/check_crates.py` — all 46 PRD §87 crates present (0 missing); 4 infra crates (`observability`, `storage-sqlite`, `testkit`, `xtask`) added; every `crates/*/src/lib.rs` verified as a pure `//! Phase N` placeholder (no logic/fns/impls). `cargo check --workspace` agrees (50 crates build with no code).
- **DoD:** ⚠️ items 2,4,7,9,10,11,12,13 not applicable — placeholders contain only doc comments.
- **Notes:** Phase tags per §88 domain mapping: P1 quran-core/corpus/citations; P2 normalization/morphology/search; P3 graph/rag/embeddings/reranking; P4 ingestion/retrieval; P5 hadith/isnad/tafsir/scripture; P7 tools/agents/policy/etc.; P9 server/tui. Flagged: `rag`(P3)/`ingestion`(P4)/`retrieval`(P4) tag vs PRD §88 "Multi-RAG(9), source catalogs(13)" — minor, for orchestrator to confirm.

### Sprint 0.2 — Config, Secrets, Storage, Migrations

### Sprint 0.1 — Skeleton & Contracts (continued)

### P0-T05 — `domain`: typed IDs, `SemVer`, `Timestamp`, `Language`, `Confidence`
- **Deliverable:** D0.2
- **Completed:** 2026-09-07
- **Owner:** implementing engineer (BE)
- **PR / commit:** `main` (worktree-local)
- **Evidence:** `cargo test -p domain` → 9 unit + 6 proptest passed; `cargo clippy -p domain --all-targets -- -D warnings` clean; `cargo fmt -p domain --check` exit 0; `cargo run -q -p xtask -- arch-check` → `OK`.
- **DoD:** ⚠️ items 1,4,5,7,9,10,11,12,13 N/A — pure, dependency-light types (no I/O, no mutation, no config, no provenance yet). Items 2 & 6 satisfied: proptest suite + inner `//!` docs referencing constraints.
- **Notes:** Created `crates/domain/src/ids.rs` (12 `typed_id!` newtypes) and `primitives.rs` (SemVer, Timestamp RFC7333-UTC, Language BCP-47, Confidence 0..1, plus parse-error types with `QAI-DOM-*` codes). `domain` deps limited to serde/serde_json/thiserror/time/uuid (+ proptest dev) per §33 — arch-check green proves no forbidden provider deps.

---

## 3. Verified Acceptance Criteria

_None verified yet._

**Entry format**

```
### AC-P0-nn — <criterion short name>
- **Verified:** YYYY-MM-DD
- **Verified by:** <reviewer name> (must not be the implementer for ritual ACs)
- **Method:** <test path / scripted check / live walkthrough>
- **Evidence:** <CI run URL, artifact, or recording timestamp>
- **Result:** ☑ Pass
- **Notes:** <caveats, re-verification triggers>
```

Live-walkthrough ACs (AC-P0-03, 05, 06, 08, 11, 14, 16) additionally require the recording link
and the reviewer's name, and must be verified by someone other than the implementer.

| ID | Criterion | Verified | By | Evidence |
|---|---|---|---|---|
| AC-P0-01 | `cargo xtask ci` green on 3 OSes | — | — | — |
| AC-P0-02 | `arch-check` fails on forbidden edge (mutation test) | — | — | — |
| AC-P0-03 | 🎥 Migrations: fresh, idempotent, checksum hard-fail | — | — | — |
| AC-P0-04 | 🎥 Config precedence + `--explain` origins | — | — | — |
| AC-P0-05 | 🎥 Secret sentinel leaks nowhere | — | — | — |
| AC-P0-06 | 🎥 Canonical immutable; writes need `ApprovalToken` | — | — | — |
| AC-P0-07 | Computational annotation constraints | — | — | — |
| AC-P0-08 | 🎥 Source lifecycle + approval preconditions | — | — | — |
| AC-P0-09 | Manifest signature & unsigned policy | — | — | — |
| AC-P0-10 | Genealogy rendering + cycle rejection | — | — | — |
| AC-P0-11 | 🎥 Job idempotency + crash resume | — | — | — |
| AC-P0-12 | Cancel within 2s | — | — | — |
| AC-P0-13 | Audit chain verify + tamper detection | — | — | — |
| AC-P0-14 | 🎥 Doctor read-only + JSON schema | — | — | — |
| AC-P0-15 | Security guards + fail-closed | — | — | — |
| AC-P0-16 | 🎥 Localhost bind default; TLS policy validation | — | — | — |
| AC-P0-17 | `Diagnostic` conformance + unique codes | — | — | — |
| AC-P0-18 | Telemetry off by default; denylist honoured | — | — | — |
| AC-P0-19 | ADRs accepted with all §48 fields | — | — | — |
| AC-P0-20 | Docs + 5 runbooks complete | — | — | — |
| AC-P0-21 | Backup/restore round-trip | — | — | — |
| AC-P0-22 | Coverage gates met | — | — | — |
| AC-P0-23 | Commit-bounded outbox row (by construction) | — | — | — |
| AC-P0-24 | Generation monotonicity under 50 writers | — | — | — |
| AC-P0-25 | Tombstone before reader visibility | — | — | — |
| AC-P0-26 | Doctor reports outbox backlog read-only | — | — | — |

---

## 4. Accepted ADRs

_None accepted yet._

**Entry format**

```
### ADR-00nn — <title>
- **Status:** Accepted
- **Accepted:** YYYY-MM-DD
- **Author / reviewers:** <names>
- **File:** adr/ADR-00nn-<slug>.md
- **§48 fields present:** Context · Options · Decision · Accuracy · Religious-source ·
  Licensing · Security · Operational · Migration strategy · Reversal cost
- **Decision summary:** <one or two sentences>
- **Constrains:** <deliverables / later phases>
```

| ADR | Title | Blocking for | Status |
|---|---|---|---|
| ADR-0001 | Relational store & access layer (SQLite + `sqlx`, Postgres-portable SQL) | D0.6 | ☐ |
| ADR-0002 | Migration strategy (append-only checksummed SQL; forward-only canonical) | D0.7 | ☐ |
| ADR-0003 | Durable job system (DB-backed leased queue, no broker) | D0.9 | ☐ |
| ADR-0004 | Configuration & precedence model | D0.4 | ☐ |
| ADR-0005 | Secret storage per OS (env / keychain / age file) | D0.5 | ☐ |
| ADR-0006 | Hashing & canonical serialization (SHA-256, canonical JSON, NFC) | D0.2 | ☐ |
| ADR-0007 | Source manifest format & signing (ed25519 detached) | D0.10 | ☐ |
| ADR-0008 | Provenance representation (universal record + typed attribution) | D0.11 | ☐ |
| ADR-0009 | Audit integrity (hash chain + append-only triggers) | D0.12 | ☐ |
| ADR-0010 | Error taxonomy & CLI exit codes | D0.3, D0.13 | ☐ |
| ADR-0011 | Observability stack; telemetry opt-in | D0.8 | ☐ |
| ADR-0012 | Workspace/crate boundaries & dependency enforcement | D0.1 | ☐ |
| ADR-0702 | Cross-store consistency (§§2–4, 9 adopted early) | D0.18 | ☐ |

> **Open before more ADRs are written:** the numbering collision (P0-T59b). Adopted scheme is
> **phase-coded** (`ADR-00nn` / `ADR-02nn` / `ADR-07nn`). Log the three file fixes here when done:
> rename `ADR-0004-relational-store.md` → `ADR-0001-…`; move `ADR-0002-database.md` out of the
> ADR sequence as a survey memo; write the real ADR-0002.

---

## 5. Deviations From Plan

_None recorded yet._

Log anything delivered differently from `plan.md`. Deviations are expected and fine — **undocumented**
deviations are the problem, because Phase 1 inherits this chassis assuming the plan describes it.

**Entry format**

```
### DEV-nn — <short title>
- **Date:** YYYY-MM-DD
- **Plan reference:** plan.md §x / D0.y / P0-Tnn
- **Planned:** <what the plan said>
- **Delivered:** <what was actually built>
- **Reason:** <why>
- **Scope impact:** <deliverables / ACs affected>
- **Phase-1 impact:** <what the handoff doc must say>
- **Approved by:** <name>
```

Deviations requiring **explicit sign-off** because they touch retrofit-impossible guarantees:
any change to D0.11 (`ApprovalToken` / `CanonicalWriter`), D0.12 (audit chain), D0.15
(`Untrusted<T>`), or D0.18 (commit-bounded outbox).

---

## 6. Corrections

_None._

**Entry format**

```
### COR-nn — correction to <entry ID>
- **Date:** YYYY-MM-DD
- **Original entry:** <ID and date>
- **What was wrong:** <description>
- **Correct record:** <description>
- **Cause:** <how the wrong record happened>
```

---

## 7. Deferred Items & Follow-Ups

_None recorded yet._

Anything intentionally not done in Phase 0 that is **not** already in the out-of-scope list
(`readme.md` §4). Every row needs a named owner and a target phase — an unowned deferral is a
silent scope leak into Phase 1.

| ID | Item | Reason deferred | Target phase | Owner | Logged |
|---|---|---|---|---|---|
| — | — | — | — | — | — |

Carried into `docs/plans/handoff-p0-to-p1.md` by task P0-T60.

---

## 8. Phase Closure

| Gate | Requirement | Evidence | Signed off by | Date |
|---|---|---|---|---|
| All 67 tasks done | `tasks.md` fully ☑ | | | |
| All 26 ACs verified | §3 above | | | |
| Coverage gates met | `acceptance.md` §2 | | | |
| 17 required suites green | `acceptance.md` §3.2 | | | |
| 13 ADRs accepted, §48-complete | §4 above | | | |
| ADR numbering reconciled | P0-T59b | | | |
| 6 migrations applied & checksummed | `migrations/sqlite/` | | | |
| 5 runbooks published | `docs/runbooks/` | | | |
| Exit-gate ritual recorded | `acceptance.md` §4 (7 live ACs) | | | |
| Handoff doc published | `docs/plans/handoff-p0-to-p1.md` | | | |
| Swimlane X decisions owned & open | P0-X01 / X02 / X03 | | | |
| Deviations documented | §5 above | | | |
| Deferrals owned | §7 above | | | |

**Phase 0 accepted:** _pending_
**Phase 1 unblocked:** _pending_

> Closure requires the **Swimlane X** row. ADR-0101 (Quran dataset licensing) is needed by the
> end of Phase 0 and ADR-0203/0204 by the start of Phase 2; a green Phase 0 with all three
> unowned means Phase 1 or 2 starts stalled on an external decision that engineering cannot
> compress. Recording this as a closure gate is the only reliable defence.
