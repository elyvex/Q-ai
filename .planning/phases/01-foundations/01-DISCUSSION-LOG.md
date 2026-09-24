# Phase 1: Foundations - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-09-24
**Phase:** 1-Foundations
**Areas discussed:** Brownfield gap closure, Operator bootstrap path, Audit transaction coverage, Job worker lifecycle

---

## Brownfield gap closure

### Determination of remaining work

| Option | Description | Selected |
|--------|-------------|----------|
| Evidence-driven gaps | Map each criterion to current code and repeatable checks; plan only missing behavior or weak evidence. | ✓ |
| Full foundation audit | Re-audit every foundation crate and acceptance path deeply. | |
| Fresh install gate | Treat current code as provisional and require fresh-workspace acceptance for every criterion. | |

**User's choice:** Evidence-driven gaps  
**Notes:** This is a brownfield repository with substantial working foundation code.

### Governing completion contract

| Option | Description | Selected |
|--------|-------------|----------|
| Roadmap + locked ADRs | Roadmap criteria and locked ADRs govern; other documents are evidence only. | ✓ |
| All existing requirements | Use the union of all current plans, task docs, criteria, and ADRs. | |
| Behavior first | Validate observable behavior; defer documentation inconsistencies. | |

**User's choice:** Roadmap + locked ADRs

### Completion evidence threshold

| Option | Description | Selected |
|--------|-------------|----------|
| Code + repeatable check | Accept code inspection plus an automated or repeatable CLI-level check. | ✓ |
| Dedicated acceptance suite | Require dedicated acceptance coverage for every criterion. | |
| Full E2E evidence | Require standard gates and fresh-workspace E2E demonstration per criterion. | |

**User's choice:** Code + repeatable check

### Later-phase code boundary

| Option | Description | Selected |
|--------|-------------|----------|
| Reuse, don't expand | Reuse later code as evidence; later capabilities retain their phase ownership. | ✓ |
| Foundation subset only | Limit Phase 1 to the minimum foundation subset. | |
| Close adjacent gaps | Complete adjacent later-phase code when it reduces integration risk. | |

**User's choice:** Reuse, don't expand

---

## Operator bootstrap path

### Pending migrations

| Option | Description | Selected |
|--------|-------------|----------|
| Explicit migrations | Ordinary commands report `qai db migrate`; they do not mutate schema automatically. | ✓ |
| Automatic migrations | Automatically apply pending migrations before stateful commands. | |
| New DB auto-migrate | Auto-migrate only a brand-new database. | |

**User's choice:** Explicit migrations

### First-run path

| Option | Description | Selected |
|--------|-------------|----------|
| Doctor-guided setup | Use read-only `qai doctor` as the first-run readiness and remediation entry point. | ✓ |
| Guided init command | Add a state-changing `qai init` command. | |
| Documented manual steps | Document manual setup without a dedicated flow. | |

**User's choice:** Doctor-guided setup

### Default configuration detail

| Option | Description | Selected |
|--------|-------------|----------|
| Origins + JSON | Show redacted effective config with origins by default and machine-readable JSON. | ✓ |
| Summary first | Show keys and status by default; origins require verbose mode. | |
| Origins first | Make origins the only detailed default, with a separate compact summary. | |

**User's choice:** Origins + JSON

### Stable command-tree contract

| Option | Description | Selected |
|--------|-------------|----------|
| Stable foundation groups | Keep `config`, `db`, `doctor`, `job`, `audit`, `secret`, and `source` stable. | ✓ |
| Reorganize full tree | Reorganize the command tree now and retain aliases. | |
| Foundation namespace | Place foundation operations under `qai foundation`. | |

**User's choice:** Stable foundation groups

---

## Audit transaction coverage

### Audited operation scope

| Option | Description | Selected |
|--------|-------------|----------|
| Durable domain changes | Audit authoritative domain mutations; exclude reads and rebuildable derived state. | ✓ |
| Every operation | Audit every command, read, cache rebuild, and write. | |
| High-risk actions | Audit only approvals and security-sensitive actions. | |

**User's choice:** Durable domain changes

### Transaction atomicity

| Option | Description | Selected |
|--------|-------------|----------|
| All-or-nothing | Domain state, provenance, audit, and required outbox records commit or roll back together. | ✓ |
| Async audit | Commit state and provenance, then enqueue audit asynchronously. | |
| Risk-based atomicity | Require atomic audit only for high-risk actions. | |

**User's choice:** All-or-nothing

### Job lifecycle audit coverage

| Option | Description | Selected |
|--------|-------------|----------|
| Lifecycle + results | Audit enqueue, lease, completion, failure, cancellation, and handler domain results. | ✓ |
| Requests + terminal | Audit enqueue/cancel requests and terminal outcomes only. | |
| Handlers only | Leave lifecycle events to logs and metrics. | |

**User's choice:** Lifecycle + results

### Audit integrity verification

| Option | Description | Selected |
|--------|-------------|----------|
| Continuous integrity check | Make `qai audit verify` a required readiness/foundation check. | ✓ |
| Automatic verification | Verify automatically at startup and after every append. | |
| Explicit command only | Provide the command but omit it from routine readiness. | |

**User's choice:** Continuous integrity check

---

## Job worker lifecycle

### Worker execution location

| Option | Description | Selected |
|--------|-------------|----------|
| Long-lived hosts only | Run workers in `qai serve`; one-shot commands only enqueue. | ✓ |
| One-shot drain | Commands start a bounded worker and drain eligible jobs before exit. | |
| Explicit worker command | Add `qai job worker`; all other entry points only enqueue. | |

**User's choice:** Long-lived hosts only

### Crash and lease recovery

| Option | Description | Selected |
|--------|-------------|----------|
| Durable handler checkpoints | Persist named checkpoints and resume safely from the last committed point. | ✓ |
| Restart whole job | Restart from the beginning and rely on idempotency. | |
| Selective checkpoints | Require checkpoints only for jobs classified as long-running. | |

**User's choice:** Durable handler checkpoints

### Transient failure retries

| Option | Description | Selected |
|--------|-------------|----------|
| Bounded backoff | Exponential backoff with jitter, per-kind caps, inspectable exhaustion. | ✓ |
| Immediate fixed retries | Retry immediately up to a fixed count. | |
| Manual retry only | Require explicit operator retry after any failure. | |

**User's choice:** Bounded backoff

### Running-job cancellation

| Option | Description | Selected |
|--------|-------------|----------|
| Cooperative checkpoints | Persist the request, stop at checkpoints, finalize safely, and report the observed outcome. | ✓ |
| Queued-only cancellation | Cancel queued jobs; running jobs finish. | |
| Immediate termination | Terminate tasks immediately and roll back transactions. | |

**User's choice:** Cooperative checkpoints

---

## the agent's Discretion

- Exact checkpoint payload schemas, retry constants, and remediation wording, provided they conform to existing project conventions and preserve the decisions above.

## Deferred Ideas

- Later-phase Quran/search/server feature completion.
- Production PostgreSQL/Qdrant, TLS, and deployment hardening.
- Dataset/license selection for Phase 2.
