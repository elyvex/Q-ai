# Phase 2: Canonical Quran Core - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-09-24
**Phase:** 2-Canonical Quran Core
**Areas discussed:** Phase posture — harden vs build; Canonical dataset & bundle policy; Reference corpus & integrity strictness; Immutability & quotation evidence depth

---

## Phase posture — harden vs build

| Option | Description | Selected |
|--------|-------------|----------|
| Evidence-driven harden | Map each success criterion to existing code + repeatable check; build only missing behavior | ✓ |
| Harden + targeted redesign | Harden existing, redesign anything non-compliant with a locked ADR | |
| Rebuild from scratch | Treat the canonical core as greenfield | |

**User's choice:** Evidence-driven harden
**Notes:** Codebase already implements the importer, canonical/staging tables, activation/rollback, and `verify_quotation`; Phase 2 reconciles and proves them.

| Option | Description | Selected |
|--------|-------------|----------|
| Legacy docs as evidence | docs/03-plan board + specs + ADRs are authoritative evidence; .planning/ROADMAP is the contract | ✓ |
| Plan only from .planning | Ignore the legacy docs reconciliation | |
| You decide | Planner chooses | |

**User's choice:** Legacy docs as evidence

| Option | Description | Selected |
|--------|-------------|----------|
| Fixture-complete, record gates | Complete on synthetic fixture; OD-01/02/03 recorded as explicit blocked gates | ✓ |
| Block completion on sign-off | Do not complete until dataset/reviewer/reference corpus supplied | |
| Out of planning scope | Do not surface owner gates | |

**User's choice:** Fixture-complete, record gates

| Option | Description | Selected |
|--------|-------------|----------|
| Translations IN scope | Model/validate translation editions as a distinct non-canonical layer (ADR-0112) | ✓ |
| Canonical Arabic only | Defer translations to a later phase | |
| You decide | Planner chooses | |

**User's choice:** Translations IN scope

---

## Canonical dataset & bundle policy

| Option | Description | Selected |
|--------|-------------|----------|
| B — fixture + user import | Ship no real text; operator imports an approved edition | ✓ |
| A — bundle a licensed edition | Vendor a licensed edition for zero-config first run | |
| C — defer Phase 2 | Block until a dataset is licensed | |

**User's choice:** B — fixture + user import

| Option | Description | Selected |
|--------|-------------|----------|
| Leave identity owner-gated | No invented slug/publisher/license; OD-01 gate | ✓ |
| Name a candidate now | Pin a candidate upstream slug now | |
| You decide | Planner chooses | |

**User's choice:** Leave identity owner-gated

| Option | Description | Selected |
|--------|-------------|----------|
| Explicit flag + active pointer | Primary/default flag (Uthmani + Ḥafṣ) feeding `quran_active_edition` | ✓ |
| Active pointer only | No separate flag | |
| You decide | Planner chooses | |

**User's choice:** Explicit flag + active pointer

| Option | Description | Selected |
|--------|-------------|----------|
| Keep test-edition-min | Use existing fixture + adversarial corpora | |
| Add a richer synthetic fixture | Larger synthetic edition to exercise integrity checks | ✓ |
| You decide | Planner chooses | |

**User's choice:** Add a richer synthetic fixture

---

## Reference corpus & integrity strictness

| Option | Description | Selected |
|--------|-------------|----------|
| Recorded skip + documented procedure | QV-015 stays a recorded skip; procedure documented | |
| Require QV-015 as a phase exit | Block completion until a reference corpus passes | |
| Configure a reference corpus now | Select and wire an independent reference corpus this phase | ✓ |

**User's choice:** Configure a reference corpus now
**Notes:** Candidate `spqrxi/quranchecksum` (hash-only, MIT per ADR-0101); exact identity/scope/license remain owner-gated (OD-03/ADR-0114).

| Option | Description | Selected |
|--------|-------------|----------|
| All deterministic checks required | All except reference comparison | |
| All six incl. reference comparison | Counts, addressing, Unicode, checksums, round-trip, reference comparison | |
| You decide | Planner chooses | ✓ |

**User's choice:** You decide → resolved to all six (counts, addressing, Unicode, checksums, round-trip, reference comparison)

| Option | Description | Selected |
|--------|-------------|----------|
| CLI + doctor + CI gate | Operator-visible and machine-gated | |
| Committed report artifact | Persisted evidence of record | |
| Both | Both surfaces | ✓ |

**User's choice:** Both

| Option | Description | Selected |
|--------|-------------|----------|
| Frozen — ADR-0108 v1 | No change to the v1 recipe | |
| Allow additive recipes | Additive domain-separated recipes permitted; v1 unchanged | ✓ |

**User's choice:** Allow additive recipes

---

## Immutability & quotation evidence depth

| Option | Description | Selected |
|--------|-------------|----------|
| Triggers + type gate + importer audit | All three enforcement layers | ✓ |
| Triggers + importer audit only | Skip re-proving the type gate | |
| Type gate only | Rely on the Rust type system | |

**User's choice:** Triggers + type gate + importer audit

| Option | Description | Selected |
|--------|-------------|----------|
| Add adversarial write tests | Direct writes to canonical tables must fail; importer holds no token | ✓ |
| Reuse existing coverage only | Rely on existing trigger/migration tests | |
| You decide | Planner chooses | |

**User's choice:** Add adversarial write tests

| Option | Description | Selected |
|--------|-------------|----------|
| Enforce on every current path | Any path emitting quoted text calls verify_quotation; mismatch hard-fails | ✓ |
| Primitive + tests only | Deliver the primitive; wire surfaces later | |
| Audit + enforce where present | Enforce only on existing paths without adding coverage | |

**User's choice:** Enforce on every current path

| Option | Description | Selected |
|--------|-------------|----------|
| Confirm ADR-0107 as-is | ApprovalToken bound to exact URN; importer holds none | ✓ |
| Tighten with runtime check | Add a pre-write token URN/scope re-check | |

**User's choice:** Confirm ADR-0107 as-is

---

## the agent's Discretion

- Required integrity-check set (user deferred) → resolved to all six families.
- Concrete reference-corpus identity/scope/license within the D-09 gate.
- Checkpoint payload schemas, backoff constants, fixture naming, and operator-facing wording per existing conventions.

## Deferred Ideas

- Additional qira'at beyond the initial validated edition(s) (V2-01).
- OD-11 (morphology dataset/license) and OD-12 (normalization catalog + linguist) remain Phase-3 owner inputs.
- Remote PostgreSQL/Qdrant, TLS, production management (Phase 12).
