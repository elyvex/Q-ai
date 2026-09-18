<!-- Sync Impact Report (remove before commit):
Version change: 1.1.0 → 1.2.0 (MINOR)
- Modified principles: none renamed or redefined.
- Added sections:
  - VIII. Multi-Edition Quran Governance (NON-NEGOTIABLE) — codifies the
    2026-09-18 owner direction: primary default (Uthmani script + Hafs
    'an 'Asim) as script-plus-transmission only, one logical corpus with
    many identified editions/readings, script/qira'ah/riwayah/edition/
    version/source/licence separation, verbatim upstream slugs, staged
    adapter ingestion, per-family licensing, first-class translations,
    per-edition integrity, typed comparison, edition-relative morphology,
    unknowns staying unknown, and non-blocking engineering.
- Removed sections: none.
- Materially expanded guidance: Technology Stack Scale/scope
  (multi-edition / multilingual corpus) and Development Workflow
  (owner-decisions log + upstream-sources references).
- Follow-up TODOs: none — dataset identity, per-item licences, editorial
  reviewer, reference corpus, morphology provider, and exit rituals remain
  tracked in ADRs and phase ledgers, not in the constitution.
-->
# Q-ai Constitution

## Core Principles

### I. Canonical Text Integrity (NON-NEGOTIABLE)

Quran Arabic text, surah order, verse identifiers, canonical token order,
edition identity, and recitation identity MUST NEVER be generated or
corrected by an LLM. Canonical tables are insert-only; every canonical
change requires a new source version, checksum validation, structural
validation (QV rules), a difference report, human approval
(`ApprovalToken` / approval-gated activation), and an audit event
(PRD §2.1, §7.3). This is enforced by real code, not convention:
`quran-core` owns the immutable edition/structure/token types and reference
grammar; `quran-corpus` runs validators QV-001…028 with adversarial fixtures
that must reject using the specific rule ID; migrations `0007`–`0012` create
insert-only canonical tables guarded by triggers; activation and rollback go
through `CanonicalWriter` + `ApprovalToken` and flip the active edition in a
single atomic pointer change. Canonical reading and exact verse lookup MUST
NOT require an LLM, vector store, or external model. The bundled
`test-edition-min` corpus is **synthetic test data, not Quran text**; a
successful fixture import is never editorial approval of a real corpus.

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
(PRD §15.3). Derivation MUST record every upstream version (source version,
parser, normalizer, chunker, embedding model, graph builder, schema)
(PRD §76). Import MUST separate ingestion from visibility: staging →
validation → explicit human approval → atomic activation → audit, with no
self-activating import.

### III. Traceability and Reproducibility

Every factual claim MUST be traceable to source ID, edition, exact
location, quoted passage, content hash, and ingestion version, with
claim-level citations validated before presentation (PRD §21, §35.3). No
model may fabricate a verse, hadith, chain, grading, or citation. Quran
quotations MUST come from canonical retrieval, never from model memory, and
MUST be edition-pinned through a restricted-constructor quotation type
(`QuranQuotation`); the citation resolver MUST open the exact source
location, not a document homepage. Every tool result MUST include the tool
result contract (`tool_name`, `tool_version`, query, normalization rules,
edition id/version, results, references, confidence, warnings, timing,
reproducibility data), and every research query/plan/report MUST produce a
reproducibility checksum (PRD §12–12.1). Re-running a deterministic query
with the same checksum MUST reproduce identical output; non-deterministic or
model-generated parts MUST be labeled separately.

### IV. Scholarly Honesty (NON-NEGOTIABLE)

Disputed claims MUST be labeled disputed; contradictory tafsir, grading,
and scholarly views MUST be presented side-by-side with attribution,
never merged into one system-endorsed conclusion or AI-synthesized
resolution (PRD §2.6–2.8, §2.17–2.19, §21.4). Hadith gradings MUST record
grader, methodology, exact term, source, and date — never a single
universal `authentic` Boolean (PRD §14.4). Translations and glosses MUST be
attributed and MUST NOT be presented as the original; lineage
(translation/summary/abridgment) MUST be transitive and the actually-quoted
lineage member cited (PRD §2.5, §22.6). Search results MUST state which
normalization rules were applied and MUST NEVER modify the displayed
canonical text (PRD §2.9, §8). Q-ai assists research; it MUST NOT claim
religious authority, issue binding rulings, or present its own synthesis as
scholarly consensus without evidence.

### V. Test-First and Quality Gates (NON-NEGOTIABLE)

Red-Green-Refactor is mandatory for behavior changes: write or update
tests, confirm they fail, then implement. Every change MUST pass before
merge: `cargo fmt --all -- --check`, `cargo check --workspace
--all-targets`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace`, `cargo run -p xtask -- arch-check`, and
`cargo run -p xtask -- migrate-check`. Canonical and import changes
additionally MUST pass the Quran integrity path: adversarial QV fixtures
with specific rule IDs, the golden reference set, `qai quran forms rebuild`,
`qai quran index rebuild` / `qai quran index verify`, the approval-gated
`qai quran activate` path, and `qai doctor --quran --deep`; the CLI smoke
path (`qai db migrate/status/verify/backup`, `qai doctor`) MUST stay green.
Property tests (roundtrip, never-panic, invariant) are required where the
state space warrants, and integration suites live in `crates/testkit` and
crate-local `tests/`. No phase exits with red gates or unverified acceptance
criteria, and no historical passing test count substitutes for a green run
on the current tree.

### VI. Local-First Security, Deny-by-Default

Local data remains local unless the user explicitly enables a remote
provider. Agents and tools use deny-by-default permissions; internet
sources are untrusted until validated and MUST NEVER become active on
LLM recommendation alone (PRD §2.10, §2.16, §22.3). Enforce the
`domain` security guards (path containment, archive limits, SSRF /
resolved-IP checks, input caps, HTML sanitization, `Untrusted<T>`), the
global redaction layer and `Secret<T>` handling (no `Debug` leak, values
never stored in SQLite, sentinel-leak suite green), TLS/domain-allowlist/
size-limit/staged-index/rollback update safety, and ACL-aware retrieval
applied before results reach a model. The server MUST bind loopback by
default, MUST reject non-loopback binding without TLS, and MUST fail closed
when a guard cannot be evaluated. `unsafe_code = "forbid"` workspace-wide
with zero allowlisted exceptions. `doctor` MUST report local-only fallback
paths and MUST NOT mutate data.

### VII. Simplicity and Architecture Discipline

Start simple (YAGNI); no organizational-only crates. Respect the
workspace dependency layering enforced by `cargo xtask arch-check`
(`server` → `application` → domain/storage/tools, never sideways);
route new cross-crate access through `application` and record any
temporary allowlist as a follow-up with an owner. `domain` has no I/O
or async runtime; the deterministic canonical Quran path has zero
model/vector/embeddings dependencies. Record significant decisions as ADRs
under `docs/02-architecture/decisions/` (`ADR-nnnn-title.md`); the accepted
Phase-0 scheme is phase-coded (`ADR-00nn` / `ADR-02nn` / `ADR-07nn`).
Prefer SQLite-adjacent, zero-extra-dependency solutions (e.g. adjacency
tables + bounded CTEs per ADR-0202) unless a spike with benchmarks justifies
an accelerator. **SQLite FTS5 is the implemented full-text backend**;
Tantivy remains an unaccepted proposal and MUST NOT be assumed. Migrations
stay contiguous, append-only, and checksummed (`migrations/sqlite/`,
currently `0001`–`0016`); canonical tables are forward-only (deactivation,
never deletion). Placeholder crates stay empty until their phase starts.
Search normalization MUST NEVER modify displayed canonical text.

### VIII. Multi-Edition Quran Governance (NON-NEGOTIABLE)

The primary/default Quran representation is Uthmani script + Hafs
'an 'Asim. This states an application default — script plus transmission —
not a dataset: it MUST NEVER be read as an edition slug, publisher,
release, licence, checksum, or source identity. Q-ai models one logical
Quran corpus with multiple explicitly identified editions/readings;
alternate editions MUST be representable without changing the core model,
and different editions MUST NEVER be merged (extends invariant I3).
Script, qira'ah, riwayah, edition, edition version, source, publisher,
licence, attribution, provenance, and checksum are separate fields;
inferences such as `Uthmani ⇒ Hafs` are forbidden unless the source
explicitly establishes them. Upstream edition identifiers MUST be preserved
verbatim as `upstream_edition_slug`; any Q-ai-internal id is an explicit
mapping, never a rename. External data enters only through the staged
adapter pipeline (source → raw artifact → extraction → validation →
normalized dataset → storage), with repository, revision, path, retrieval
date, source hash, transformation, validation result, licence state, and
attribution recorded. A repository being open source MUST NEVER imply its
contained data text is redistributable: every data family carries its own
licence record, and data with unverified terms stays
`metadata_only` / `pending_license_review` — never vendored, never labeled
open. Translations are first-class records, many per language, never
identified by language code alone, never canonical text. Each edition
carries its own integrity manifest (verse/surah/edition scope, explicit
algorithm and normalization); one reading's checksum MUST NEVER certify
another, and translation integrity is hash-only with a root key distinct
from the Quran root. Comparison is typed — integrity, version, readings,
translation, reference — and every difference is classified; a difference
between two legitimate readings is a readings comparison, never a
corruption verdict. Morphology is edition-relative (edition, version,
surah, ayah, token position) and bridged by explicit alignment tables;
disagreeing tokenizations MUST NEVER be force-matched, and no LLM may
generate roots/lemmas stored as dataset-supplied. Unverified fields MUST
stay `unknown` / `pending_owner_verification` /
`source_verification_required`; ADRs MUST NOT be marked Accepted to unblock
development. Pending owner decisions MUST NOT block unrelated engineering:
isolate the decision, record it, and build everything else. Enforced by
`quran-core::catalog` types, the `sources::upstream` registry, and
ADR-0101 / ADR-0114 / ADR-0203.

## Technology Stack & Architectural Constraints

**Language/Version**: Rust (workspace `resolver = "2"`, edition 2024;
toolchain pinned to **1.97.1** in `rust-toolchain.toml`). **PRD baseline**:
Q-ai PRD **v0.3.2**. **Primary dependencies**: tokio, axum + tower-http
(API v1), sqlx/SQLite (incl. FTS5), clap (CLI), tracing + OpenTelemetry,
unicode-segmentation / unicode-normalization, regex-automata (DFA-only),
csv, similar, lru, ed25519-dalek, thiserror/anyhow. **Storage**: SQLite with
checksummed, contiguous, append-only migrations (`migrations/sqlite/`,
`0001`–`0016` plus `checksums.json`, `VACUUM INTO` backups); canonical
tables insert-only via triggers and forward-only. **Quran canonical
crates**: `quran-core` (immutable models, reference grammar, quotations),
`quran-corpus` (adapters, tokenizer, hashing recipes, QV validators,
importer/differ), `citations` (citation identity/resolution),
`quran-normalization` (versioned rules/profiles, offset traces),
`quran-search` (exact/normalized/phrase/concatenated/regex services over
SQLite FTS5). **Interfaces**: CLI (`qai`, incl. the `quran` group: `import`,
`activate`, `get`, `context`, `edition show`, `normalize`, `forms rebuild`,
`index rebuild|verify`), loopback HTTP server (`/healthz`, `/readyz`,
`/api/v1/quran/*` read/citation and normalization routes), library crates
per surface. **Testing**: `cargo test --workspace`, trycmd CLI snapshots,
insta snapshots, proptest, `testkit` fixtures, golden reference sets,
adversarial QV fixtures. **Constraints**: canonical lookup deterministic and
model-free; bounded regex and graph queries (resource-limited, N-hop caps);
RTL-correct display with Web GUI authoritative for rich reading; async,
non-blocking I/O with cancellation and query timeouts. **Scale/scope**: 114
surahs / 6236-ayah-class canonical corpus, multi-collection hadith/tafsir/
scripture graphs. The Quran corpus is multi-edition/multi-riwayah by design
(primary default: Uthmani script + Hafs 'an 'Asim) with per-edition
integrity manifests; translations are multilingual, many per language.
Migrations MUST remain contiguous and append-only;
`arch-check` violations MUST be fixed or ADR-justified, never silently
allowlisted.

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
request. Owner direction and the source-verification backlog live in
`docs/05-followups/owner-decisions.md`; verified upstream facts live in
`docs/02-architecture/upstream-sources.md`. Phase state is tracked in the
phase ledgers, not here: as of 2026-09-18, P0 is implementation-complete
with sign-off and the exit ritual pending, P1 (canonical core) and P2
(normalization/search) are in progress with source decisions and exit
gates open. All PRs MUST verify constitution compliance; complexity MUST
be justified in the plan.

## Governance

This constitution supersedes all other practices on conflict.
Amendments require a documented Sync Impact Report, a semantic version
bump (MAJOR: incompatible governance/principle removal or
redefinition; MINOR: new principle/section or materially expanded
guidance; PATCH: clarifications/wording), and an ADR or amendment note
with a migration plan where behavior changes. Use
`.specify/templates/constitution-template.md` resolution at amendment
time; write only `.specify/memory/constitution.md`.

**Version**: 1.2.0 | **Ratified**: 2026-09-15 | **Last Amended**: 2026-09-18
