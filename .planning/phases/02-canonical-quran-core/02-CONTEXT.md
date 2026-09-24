# Phase 2: Canonical Quran Core - Context

**Gathered:** 2026-09-24
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver and prove one validated Quran edition that is importable, addressable, and provably immutable: operator import through staging → validation → atomic activation with rollback; byte-exact canonical Arabic lookup for any `surah:ayah` with a pinned edition reference; corpus-integrity checks (counts, addressing, Unicode, checksums, round-trip, reference comparison) passing; canonical tables rejecting all non-approved writes with the importer having no code path to them; and every quotation verifying via `verify_quotation` with mismatch as a hard failure.

This phase is **evidence-driven hardening over an existing brownfield canonical core**, not greenfield construction. It does not rebuild working importer/validator/reader/activation code, and it does not expand into later-phase capabilities (normalization/search, graph, GUI/TUI, hadith/tafsir, RAG, agents, server). Its boundary is the 5 Phase 2 success criteria in `.planning/ROADMAP.md` plus `REQ-quran-corpus`, `REQ-data-separation-layers`, and `REQ-ingestion-validation-eval`.

</domain>

<decisions>
## Implementation Decisions

### Phase Posture (Brownfield Evidence)
- **D-01:** Treat Phase 2 as an evidence-driven brownfield hardening phase: map each of the five Phase 2 success criteria to existing code and at least one repeatable check, preserve working implementations, and create implementation work only for missing behavior or weak evidence. (Mirrors Phase 1 D-01.)
- **D-02:** Authority — `.planning/ROADMAP.md` + `.planning/REQUIREMENTS.md` are the phase contract; the legacy doc tree (`docs/03-plan/phases/phase-02-rag/` board, `specs/005·006·012·015·016·019`, ADRs 0101–0114) is authoritative **evidence/reference**. Note the legacy numbering drift: `docs/03-plan/phases/phase-02-rag/` covers search/linguistics, while this phase is the canonical core (legacy "Phase 1"). Do not renumber directories by hand. — **Reversibility:** reversible — planning-and-evidence choice only.
- **D-03:** Fixture-complete with recorded owner gates. Phase 2 may complete on the synthetic fixture; the human/editor gates `OD-01` (dataset + license), `OD-02` (named reviewer + `verified_by`), and `OD-03` (reference corpus) are recorded as **explicit blocked/deferred gates that block real canonical activation** — never a silent pass. — **Reversibility:** reversible — a recorded gate can be closed later.
- **D-04:** Translation-edition work (layer separation) is **in scope**: model and validate translation editions as a distinct non-canonical layer per ADR-0112 and `REQ-data-separation-layers` — `translator NOT NULL`, `aligned_edition_id`, per-translation `slug`/`version`/`license`/`text_hash`, and **no type-level path from a translation into a canonical slot**; exercised on synthetic data. — **Reversibility:** costly — the canonical/translation separation is a published contract (ADR-0112) that citations and later surfaces depend on.

### Canonical Dataset & Bundle Policy
- **D-05:** Adopt ADR-0101 **Option B** — ship no real canonical text; engineering runs against the synthetic fixture and an operator imports an approved edition via `qai quran import`. This is the current documented fallback (`README.md` §8). — **Reversibility:** costly — moving to Option A later adds a bundled-edition import + first-run path; the import/validation contract itself is unchanged.
- **D-06:** Leave the concrete canonical edition identity **owner-gated**: do not invent an upstream slug, publisher, release, license, or hash. Plan the data shape and import path generically and record `upstream_edition_slug` + license evidence as pending owner gate `OD-01`. — **Reversibility:** one-way — once a real edition is activated its bytes, hashes (ADR-0108, frozen recipe), and citation identity (ADR-0111) are frozen; changing the dataset requires a new edition version + difference report + human approval.
- **D-07:** Represent the primary/default edition as an **explicit primary/default flag** (Uthmani script + Ḥafṣ ʿan ʿĀṣim) feeding the existing `quran_active_edition` singleton pointer, per the ADR-0101/ADR-0112 model — "primary default" is not "only edition". — **Reversibility:** costly — flags/pointers appear in storage and the display/selection contract.
- **D-08:** Add a **richer synthetic fixture** — more surahs/ayahs, basmala variants, multi-token ayahs, division boundaries — to exercise the integrity checks more deeply; keep the existing `test-edition-min` and the 16 adversarial corpora. — **Reversibility:** reversible — additional fixtures only.

### Reference Corpus & Integrity Strictness
- **D-09:** **Configure an independent reference corpus in this phase.** Candidate: `spqrxi/quranchecksum` (hash-only integrity reference, MIT, usable for compatible datasets only per ADR-0101). Exact identity, scope, procedure, and licensing remain owner-gated (`OD-03` / ADR-0114) — record them as a gate, never fabricate them. — **Reversibility:** costly — the selected corpus and procedure shape QV-015's persistent comparison report.
- **D-10:** Require **all six** integrity check families to pass: counts, addressing, Unicode, checksums, round-trip, and reference comparison (the last exercised once D-09's reference corpus is wired; until then it stays a recorded skip, never a silent pass). — **Reversibility:** reversible.
- **D-11:** Surface integrity evidence **both** ways: an operator-facing/CI surface (`qai quran verify` / `qai doctor` + a repeatable CI check) **and** a committed corpus-integrity report artifact as the evidence of record. — **Reversibility:** reversible.
- **D-12:** ADR-0108 v1 hash recipes (`qai-text-hash-v1`: `text_hash`, `structure_hash`, `token_order_hash`) stay **frozen and unchanged**; **additive**, domain-separated recipes are permitted if the phase needs them. — **Reversibility:** one-way — changing a v1 recipe invalidates every stored hash and requires a global re-hash migration.

### Immutability & Quotation Evidence
- **D-13:** Prove **all three** enforcement layers for "canonical tables reject all non-approved writes": (a) SQL insert-only triggers on canonical tables reject non-approved writes; (b) the type-level `CanonicalWriter` + `ApprovalToken` gate in `crates/provenance`; (c) an importer code-path audit proving the importer has no path to canonical tables. — **Reversibility:** reversible — evidence and tests only.
- **D-14:** Add **adversarial write tests**: direct `INSERT`/`UPDATE`/`DELETE` against canonical tables must fail with the expected trigger error, plus a test asserting the importer path never holds an `ApprovalToken` and cannot activate. — **Reversibility:** reversible.
- **D-15:** Enforce `verify_quotation` on **every current answer path** that emits quoted canonical text (CLI, HTTP, tool results); a mismatch is a hard failure. Audit existing paths and enforce on each. — **Reversibility:** costly — loosening this reverses the published citation-integrity contract (ADR-0111; mismatch is a hard failure on answer paths).
- **D-16:** Confirm the ADR-0107 trust model **as-is**: activation requires an `ApprovalToken` bound to the exact `quran-edition:{slug}@{version}` URN; the importer holds none; rollback is an atomic pointer flip with a `corpus_generation` bump. — **Reversibility:** one-way — ADR-0107 and the citation/edition identity contracts are frozen.

### the agent's Discretion
- The exact required integrity-check set was deferred to the agent ("you decide") and resolved to **all six** (D-10).
- The concrete reference-corpus identity, scope, and license within the D-09 gate (candidate `spqrxi/quranchecksum`) — the researcher should confirm candidates and the owner ratifies.
- Checkpoint payload schemas, backoff constants, fixture naming, and exact operator-facing wording may follow existing project conventions so long as the locked behavior above is preserved (Phase 1 D-16 convention).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Contract
- `.planning/ROADMAP.md` §Phase 2 — goal, five success criteria, dependencies, and phase boundary.
- `.planning/REQUIREMENTS.md` — `REQ-quran-corpus`, `REQ-data-separation-layers`, `REQ-ingestion-validation-eval` (incl. §46 acceptance rule).
- `.planning/PROJECT.md` — locked decisions (25 Accepted ADRs), constraints, out-of-scope, local-first direction.
- `.planning/STATE.md` — current position and known blockers (ADR-0101 sign-off; ADR-0114; ADR-0702 subset).
- `docs/01-requirements/requirements.md` — authoritative living PRD: §7 corpus, §6 layers, §34–§36 ingestion/validation/eval, §46 completeness rule, §86 evaluation.

### Locked Architecture Decisions
- `docs/02-architecture/decisions/ADR-0102-addressing-and-reference-grammar.md` — frozen reference grammar, round-trip, coded errors.
- `docs/02-architecture/decisions/ADR-0103-verse-numbering-scheme.md` — per-edition numbering; ayah numbers valid only within an edition.
- `docs/02-architecture/decisions/ADR-0104-unicode-policy.md` — NFC storage, forbidden code points, grapheme counts with byte offsets.
- `docs/02-architecture/decisions/ADR-0105-canonical-tokenization.md` — whitespace-preserving tokens and annotation signs.
- `docs/02-architecture/decisions/ADR-0106-canonical-storage-layout.md` — row layout, insert-only triggers, trigger-free staging mirrors.
- `docs/02-architecture/decisions/ADR-0107-atomic-activation-rollback.md` — staging → canonical pointer flip + generation bump; importer holds no token.
- `docs/02-architecture/decisions/ADR-0108-corpus-hashing-scheme.md` — frozen `qai-text-hash-v1` recipes and `sha256:<hex>` storage form.
- `docs/02-architecture/decisions/ADR-0109-edition-difference-algorithm.md` — `(surah, ayah)` edition diff persisted as `difference_reports`.
- `docs/02-architecture/decisions/ADR-0110-basmala-representation.md` — per-edition `BasmalaPolicy` + QV-020 consistency.
- `docs/02-architecture/decisions/ADR-0111-citation-identity-and-deep-links.md` — citation identity/URNs; `verify_quotation` verdicts; mismatch hard failure.
- `docs/02-architecture/decisions/ADR-0112-translation-alignment-and-attribution.md` — translation editions, `AyahView.canonical` Arabic-only, no path into canonical slot.
- `docs/02-architecture/decisions/ADR-0113-canonical-lookup-caching.md` — generation-keyed LRU cache and wholesale invalidation.
- `docs/02-architecture/decisions/ADR-0006-hashing-canonical.md`, `ADR-0007-manifest-format-signing.md`, `ADR-0008-provenance-representation.md`, `ADR-0009-audit-integrity.md`, `ADR-0010-error-taxonomy.md`, `ADR-0012-workspace-boundaries.md` — hashing, signed manifests, provenance, audit, error codes, and crate boundaries that this phase must respect.

### Open / Owner-Gated Decisions
- `docs/02-architecture/decisions/ADR-0101-initial-quran-dataset.md` — **Draft**; dataset identity, licensing, bundle policy, and reviewer sign-off gate real activation.
- `docs/02-architecture/decisions/ADR-0114-reference-corpus-comparison.md` — **Draft**; reference-corpus identity, comparison procedure, and QV-015 behavior.
- `docs/05-followups/decisions-needed.md` — `OD-01`, `OD-02`, `OD-03` (all 🔴, human-only); also OD-11/OD-12 note Phase-2 inputs.
- `docs/05-followups/owner-decisions.md` — owner's recorded architectural intent (multi-edition/multi-riwayah/multilingual).
- `docs/02-architecture/upstream-sources.md` — verified facts and revisions for `fawazahmed0/quran-api`, `gaitco/quran-database`, `spqrxi/quranchecksum`.

### Existing Reconciliation & Specs (evidence)
- `specs/005-quran-core/spec.md`, `specs/006-quran-corpus/spec.md`, `specs/012-quran-corpus-foundation/spec.md`, `specs/015-edition-integrity-manifests/spec.md`, `specs/016-typed-corpus-comparison/spec.md`, `specs/019-reference-corpus-config/spec.md` — multi-edition model, integrity manifests, typed comparison, reference-corpus config.
- `docs/03-plan/current-plan.md` and `docs/03-plan/phases/phase-02-rag/` — legacy reconciliation board (numbering is legacy; use as evidence only).
- `docs/07-technical/quran-citation-spec.md` — frozen citation identity, verdicts, and resolution algorithm.
- `docs/architecture/hashing-spec.md` — canonical hashing specification.
- `docs/schemas/quran-edition-source.v1.schema.json` — edition-source manifest schema validated by the importer.
- `docs/06-progress/task-done-rollup.md` — what has already been reconciled/completed.

### Codebase Maps & Fixtures
- `.planning/codebase/ARCHITECTURE.md`, `.planning/codebase/STACK.md`, `.planning/codebase/INTEGRATIONS.md`, `.planning/codebase/CONVENTIONS.md`, `.planning/codebase/TESTING.md` — current architecture, stack, integration points, conventions, and test strategy.
- `fixtures/quran/test-edition-min/`, `fixtures/quran/adversarial/`, `fixtures/quran/golden/` — synthetic and adversarial corpora used as the phase's exercised dataset.

### Prior Phase
- `.planning/phases/01-foundations/01-CONTEXT.md`, `.planning/phases/01-foundations/01-RESEARCH.md`, `.planning/phases/01-foundations/01-0*-PLAN.md` — Phase 1 decisions, research, and the plan/evidence patterns this phase inherits.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/quran-core` — frozen reference grammar (`src/reference/`, ADR-0102), newtypes/enums, `QuranQuotation` (`src/quotation.rs`), views (`src/view.rs`), Unicode text primitives (`src/text.rs`).
- `crates/quran-corpus` — `EditionAdapter` (JSON + CSV) (`src/adapters.rs`), Unicode auditor (`src/unicode.rs`), tokenizer (`src/tokenize.rs`), QV-001…QV-028 validator (`src/validation.rs`), char-level differ (`src/differ.rs`), 13-checkpoint importer (`src/import.rs:run_import`), corpus hashing (`src/hashing.rs`).
- `crates/citations` — `QuotationVerdict` + `resolve`/`verify_quotation` (`src/lib.rs`); `QuranQuotation` is the only type that may carry quoted canonical text.
- `crates/storage` / `crates/storage-sqlite` — `Database`/`ReadTx`/`UnitOfWork` ports; Quran repo (`src/quran.rs`); migration runner (`src/migrate.rs`); canonical vs `quran_stg_*` tables.
- `crates/provenance` — `ProvenanceRecord`, `ApprovalToken`, `CanonicalWriter` gate.
- `crates/application` — composition root: import handler + `activate_edition`/`rollback_edition` (`src/quran.rs`), generation-keyed reader (`src/quran_reader.rs`), audit bridge (`src/audit_bridge.rs`), Quran doctor data (`src/quran_doctor.rs`).
- `crates/cli` — `qai quran …` command mapping (`src/quran.rs`), doctor registry (`src/doctor.rs`), exit-code mapping (`src/exit_code.rs`).
- `fixtures/quran/` — `test-edition-min/`, `test-edition-min-v2.json`, `test-translation-min.json`, and 16 adversarial corpora.

### Established Patterns
- Layered modular monolith with ports-and-adapters; edges reach storage only through `application`; `cargo xtask arch-check` enforces boundaries (`xtask/allowlist.toml`).
- Durable writes use a single shared transaction so repository + provenance + audit + outbox commit atomically.
- Typed `Diagnostic` errors with unique `QAI-<NS>-NNNN` codes mapped to stable CLI exit codes.
- Canonical writes are approval-gated (`CanonicalWriter` + `ApprovalToken`); the importer stages only and ends at `ApprovalRequested`.
- Migrations are append-only and checksum-verified; canonical tables are forward-only (deactivation, not deletion).
- Purity fence: no LLM/embeddings/retrieval/vector dependency on the canonical path.

### Integration Points
- `crates/application/src/quran.rs` — import handler, `activate_edition`, `rollback_edition`, differ report persistence.
- `crates/application/src/audit_bridge.rs` — same-transaction audit/provenance boundary.
- `crates/application/src/quran_cli.rs` — CLI command implementations and exit mapping.
- `crates/application/src/quran_reader.rs` — canonical reads with generation-keyed cache.
- `crates/citations/src/lib.rs` + `crates/application/src/quran_tools.rs` + `crates/server/src/api.rs` — the answer paths where `verify_quotation` must be enforced (D-15).
- `migrations/sqlite/*.up.sql` — canonical insert-only triggers and staging mirrors (D-13/D-14).

</code_context>

<specifics>
## Specific Ideas

- Begin planning with a five-row evidence matrix (criterion → existing implementation → repeatable evidence → gap → planned task if needed), mirroring Phase 1's approach.
- Owner gates (`OD-01`/`OD-02`/`OD-03`) must appear in the plan as explicit blocked/deferred items with the exact command needed to close them — never folded into a silent pass.
- Enforce the layer-separation invariant as a **test**, not a comment: no type-level path from a translation into a canonical slot (ADR-0112), and no importer path to canonical tables (D-13).
- Surface the corpus-integrity report as a committed artifact so the phase's evidence is reproducible offline (D-11).

</specifics>

<deferred>
## Deferred Ideas

- Additional Quran qira'at beyond the initial validated edition(s) — V2-01/ADR-0101 (later phase).
- Normalization/search, graph, GUI/TUI, hadith/tafsir, isnad, multi-RAG, comparative scripture, agents/tools, server, and production hardening — their own roadmap phases.
- OD-11 (morphology dataset/license) and OD-12 (normalization rule catalog + linguist) are Phase-2-adjacent **owner inputs** recorded for Phase 3, not scope of this phase.
- Remote PostgreSQL/Qdrant adapters, TLS, and production management — Phase 12.

</deferred>

---

*Phase: 2-Canonical Quran Core*
*Context gathered: 2026-09-24*
