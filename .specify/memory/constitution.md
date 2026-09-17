<!-- Sync Impact Report (remove before commit):
Version change: 1.0.1 → 1.0.2 (PATCH)
- Modified principles: none renamed; Principle VII clarified to state the
  existing structural rules explicitly (domain has no I/O/async;
  deterministic canonical path has zero model/vector dependencies;
  doctor never mutates); Principle V gate list unchanged.
- Tech Stack corrections: edition 2024 (was "2021+"), toolchain pinned to
  1.97.1 per rust-toolchain.toml, migration path corrected to
  migrations/sqlite/ with append-only + checksummed discipline and
  forward-only canonical tables.
- Workflow clarification: human-only gates (dataset licensing, editorial
  sign-off, exit rituals) cannot be performed or simulated by agents.
- Scale typo: "114 surah" → "114 surahs".
- Removed sections: none. Added sections: none.
- Verification notes (2026-09-17): claims re-checked against the live tree —
  Cargo.toml workspace resolver = "2", edition = "2024",
  unsafe_code = "forbid", rust-toolchain channel 1.97.1, migrations
  0001–0016 contiguous under migrations/sqlite/, placeholder crates remain
  by design (prior report line "all placeholders resolved" was inaccurate
  and is superseded here).
- Follow-up TODOs: none — FTS5-vs-Tantivy, synthetic-fixture, and phase-board
  drift items belong in ADRs/specs/ledgers, not in the constitution.
-->
# Q-ai Constitution

## Core Principles

### I. Canonical Text Integrity (NON-NEGOTIABLE)

Quran Arabic text, surah order, verse identifiers, canonical token order,
edition identity, and recitation identity MUST NEVER be generated or
corrected by an LLM. Canonical tables are insert-only; every canonical
change requires a new source version, checksum validation, structural
validation (QV rules), a difference report, human approval
(`ApprovalToken` / approval-gated activation), and an audit event.
Rationale: exact text before generated interpretation (PRD §2.1, §7.3).

### II. Layered Trust and Provenance (NON-NEGOTIABLE)

Data MUST be separated into Layer A (canonical source text), Layer B
(publisher/dataset metadata), Layer C (scholarly annotation), Layer D
(computational annotation with algorithm, version, confidence, timestamp,
input version, verification status), and Layer E (user/AI notes)
(PRD §6). AI-generated material MUST NEVER be visually or structurally
confused with canonical text or verified scholarship. Every
non-structural graph edge MUST carry source, author-or-algorithm,
version, confidence, verification status, and timestamp; computational
suggestions MUST NOT become verified edges without explicit human review
(PRD §10.3, §10.6). Narrator identity MUST NEVER be silently merged
(PRD §15.3).

### III. Traceability and Reproducibility

Every factual claim MUST be traceable to source ID, edition, exact
location, quoted passage, content hash, and ingestion version, with
claim-level citations validated before presentation (PRD §21). No model
may fabricate a verse, hadith, chain, grading, or citation. Every tool
result MUST include the tool result contract (`tool_name`,
`tool_version`, query, normalization rules, edition id/version, results,
references, confidence, warnings, timing, reproducibility data), and
every research query/plan/report MUST produce a reproducibility checksum
(PRD §12–12.1). Quran quotations MUST come from canonical retrieval,
never from model memory.

### IV. Scholarly Honesty (NON-NEGOTIABLE)

Disputed claims MUST be labeled disputed; contradictory tafsir, grading,
and scholarly views MUST be presented side-by-side with attribution,
never merged into one system-endorsed conclusion or AI-synthesized
resolution (PRD §2.6–2.8, §2.17–2.19, §21.4). Hadith gradings MUST record
grader, methodology, exact term, source, and date — never a single
universal `authentic` Boolean (PRD §14.4). Translations MUST NOT be
presented as the original; lineage (translation/summary/abridgment)
MUST be transitive and the actually-quoted lineage member cited
(PRD §2.5, §22.6). Q-ai assists research; it MUST NOT claim religious
authority, issue binding rulings, or present conclusions as scholarly
consensus without evidence.

### V. Test-First and Quality Gates (NON-NEGOTIABLE)

Red-Green-Refactor is mandatory for behavior changes: write or update
tests, confirm they fail, then implement. Every change MUST pass before
merge: `cargo fmt --all -- --check`, `cargo clippy --workspace
--all-targets -- -D warnings`, `cargo test --workspace`,
`cargo run -p xtask -- arch-check`, `cargo run -p xtask -- migrate-check`,
plus the `qai db migrate/status/verify/backup` and `qai doctor` smoke
path. Canonical/import paths require adversarial fixtures with specific
rule IDs, property tests (roundtrip, never-panic, invariant) where
applicable, and integration suites via `crates/testkit`. No phase exits
with red gates or unverified acceptance criteria.

### VI. Local-First Security, Deny-by-Default

Local data remains local unless the user explicitly enables a remote
provider. Agents and tools use deny-by-default permissions; internet
sources are untrusted until validated and MUST NEVER become active on
LLM recommendation alone (PRD §2.10, §2.16, §22.3). Enforce the
`domain` security guards (path containment, archive limits, SSRF /
resolved-IP checks, input caps, HTML sanitization), `Secret<T>` handling
(no `Debug` leaks), TLS/domain-allowlist/size-limit/staged-index/
rollback update safety, and ACL-aware retrieval. `unsafe_code = "forbid"`
workspace-wide with zero allowlisted exceptions.

### VII. Simplicity and Architecture Discipline

Start simple (YAGNI); no organizational-only crates. Respect the
workspace dependency layering enforced by `cargo xtask arch-check`
(`server` → `application` → domain/storage/tools, never sideways);
route new cross-crate access through `application` and record any
temporary allowlist as a follow-up with an owner. `domain` has no I/O
or async runtime; the deterministic canonical Quran path has zero
model/vector/embeddings dependencies; `doctor` never mutates data.
Record significant decisions as ADRs under `docs/02-architecture/decisions/`
(`ADR-nnnn-title.md`). Prefer SQLite-adjacent, zero-extra-dependency
solutions (e.g. adjacency tables + bounded CTEs per ADR-0202) unless a
spike with benchmarks justifies an accelerator. Search normalization
MUST NEVER modify displayed canonical text.

## Technology Stack & Architectural Constraints

**Language/Version**: Rust (workspace `resolver = "2"`, edition 2024;
toolchain pinned to **1.97.1** in `rust-toolchain.toml`). **Primary dependencies**: tokio, axum +
tower-http (API v1), sqlx/SQLite (+ FTS5), clap (CLI), tracing +
OpenTelemetry, unicode-segmentation / unicode-normalization,
regex-automata (DFA-only), csv, similar, lru. **Storage**: SQLite with
checksummed, contiguous, append-only migrations (`migrations/sqlite/`,
`VACUUM INTO` backups); canonical tables insert-only via triggers and
forward-only (deactivation, never deletion). **Testing**: `cargo
test --workspace`, trycmd CLI snapshots, insta snapshots, proptest,
`testkit` fixtures. **Target platforms**: local-first CLI (`qai`), TUI
(ratatui/crossterm), server (`/healthz`, `/readyz`, API v1), library
crates per surface. **Constraints**: `<30s` `doctor --quran --deep` on a
standard edition (pending real dataset per ADR-0101); bounded regex and
graph queries (resource-limited, N-hop caps); RTL-correct display with
Web GUI authoritative for rich reading. **Scale/scope**: 114 surahs /
6236-ayah-class canonical corpus, multi-collection hadith/tafsir/
scripture graphs. Migrations MUST remain contiguous and append-only;
`arch-check` violations MUST be fixed or ADR-justified, never silently
allowlisted. Placeholder crates stay empty until their phase starts.

## Development Workflow & Quality Gates

Spec-Kit flow is constitution → `/speckit-specify` (WHAT/WHY, no tech
stack) → `/speckit-clarify` (max 3 NEEDS CLARIFICATION, optional) →
`/speckit-plan` (research.md → data-model.md, contracts/, quickstart.md)
→ `/speckit-checklist` (optional) → `/speckit-analyze` (optional) →
`/speckit-tasks` (story-ordered, independently testable) →
`/speckit-implement`. Every implementation task MUST have a TASK-ID
(`TASK-nnn-slug.md`); never modify the master plan without
justification; never mark complete until acceptance criteria
(`AC-Pn-nn`) are satisfied. Completion ritual per task: implement →
tests → lint/checks → update task doc → phase progress →
`docs/06-progress/task-done-rollup.md` → follow-ups in
`docs/05-followups/` → `CHANGELOG.md` when appropriate. Human-only gates
(dataset licensing, editorial/reviewer sign-off, exit rituals) cannot be
performed or simulated by agents; mark them pending with a named owner
request. All PRs MUST
verify constitution compliance; complexity MUST be justified in the plan.

## Governance

This constitution supersedes all other practices on conflict.
Amendments require a documented Sync Impact Report, a semantic version
bump (MAJOR: incompatible governance/principle removal or
redefinition; MINOR: new principle/section or materially expanded
guidance; PATCH: clarifications/wording), and an ADR or amendment note
with a migration plan where behavior changes. Use
`.specify/templates/constitution-template.md` resolution at amendment
time; write only `.specify/memory/constitution.md`.

**Version**: 1.0.2 | **Ratified**: 2026-09-15 | **Last Amended**: 2026-09-17
