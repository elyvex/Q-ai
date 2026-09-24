# Phase 2: Canonical Quran Core - Research

**Researched:** 2026-09-24
**Domain:** Brownfield validation of an existing Rust canonical Quran pipeline (import → staging → validation → approval-gated activation/rollback, byte-exact addressing, corpus integrity, canonical-write fencing, quotation verification)
**Confidence:** HIGH for in-repo code/test findings (opened this session); MEDIUM for the reference-corpus/owner-gate framing; LOW for any future implementation shape.

> **Provenance convention.** `[VERIFIED: path:lines]` = the cited source definition was opened in this session *and* the values are quoted verbatim below the claim. `[VERIFIED: live probe]` = observed by running a real command. `[CITED: url/or doc]` = official documentation or a locked project record. `[ASSUMED]` = a future implementation shape or an unverified value that needs planner/owner confirmation. The user-constraint block is copied verbatim from `02-CONTEXT.md` and is treated as locked input, not as an independently asserted research claim. [VERIFIED: .planning/phases/02-canonical-quran-core/02-CONTEXT.md:15-149]

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
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
- **D-14:** Add **adversarial write tests**: direct `INSERT`/`UPDATE`/`DELETE` against canonical tables must fail with the expected trigger error, plus a test asserting the importer path never holds an `ApprovalToken` and cannot activate. — **Reversibility:** reversible — evidence and tests only.
- **D-15:** Enforce `verify_quotation` on **every current answer path** that emits quoted canonical text (CLI, HTTP, tool results); a mismatch is a hard failure. Audit existing paths and enforce on each. — **Reversibility:** costly — loosening this reverses the published citation-integrity contract (ADR-0111; mismatch is a hard failure on answer paths).
- **D-16:** Confirm the ADR-0107 trust model **as-is**: activation requires an `ApprovalToken` bound to the exact `quran-edition:{slug}@{version}` URN; the importer holds none; rollback is an atomic pointer flip with a `corpus_generation` bump. — **Reversibility:** one-way — ADR-0107 and the citation/edition identity contracts are frozen.

### the agent's Discretion
- The exact required integrity-check set was deferred to the agent ("you decide") and resolved to **all six** (D-10).
- The concrete reference-corpus identity, scope, and license within the D-09 gate (candidate `spqrxi/quranchecksum`) — the researcher should confirm candidates and the owner ratifies.
- Checkpoint payload schemas, backoff constants, fixture naming, and exact operator-facing wording may follow existing project conventions so long as the locked behavior above is preserved (Phase 1 D-16 convention).

### Deferred Ideas (OUT OF SCOPE)
- Additional Quran qira'at beyond the initial validated edition(s) — V2-01/ADR-0101 (later phase).
- Normalization/search, graph, GUI/TUI, hadith/tafsir, isnad, multi-RAG, comparative scripture, agents/tools, server, and production hardening — their own roadmap phases.
- OD-11 (morphology dataset/license) and OD-12 (normalization rule catalog + linguist) are Phase-2-adjacent **owner inputs** recorded for Phase 3, not scope of this phase.
- Remote PostgreSQL/Qdrant adapters, TLS, and production management — Phase 12.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| REQ-quran-corpus | Validated canonical representation: required hierarchy, edition model, immutable text, stable addressing (PRD §7) | A complete edition model + canonical hierarchy already exists (`crates/quran-core`, `migrations/sqlite/0007`–`0012`) with byte-exact lookup and frozen reference grammar; the missing work is an explicit primary/default flag (D-07), richer fixtures (D-08), and operator-visible integrity evidence (D-11). [VERIFIED: crates/quran-core/src/editor.rs:38-88] [VERIFIED: migrations/sqlite/0007_quran_editions.up.sql:3-27] [VERIFIED: crates/application/src/quran_reader.rs:458-482] |
| REQ-data-separation-layers | Explicit trust layers A (canonical) / B (publisher metadata) / C (scholarly) / D (computational) / E (user+AI notes) (PRD §6) | Canonical/staging/staging-mirror layering is physical (`quran_*` vs `quran_stg_*`); translation editions are a separate non-canonical table with a type-level guard (`AyahView.canonical: QuranQuotation`). The gaps are an unpopulated `translation_editions.text_hash` and the absence of a persisted `upstream_edition_slug` / manifest-declared license on the canonical row. [VERIFIED: migrations/sqlite/0011_quran_staging.up.sql:1-7] [VERIFIED: crates/quran-core/src/view.rs:96-106] [VERIFIED: crates/application/src/quran.rs:792-810] |
| REQ-ingestion-validation-eval | Discover→stage ingestion pipeline; Quran/hadith/citation validation; evaluation (PRD §34, §35, §36, §86). Acceptance (§46): a Quran feature is complete only when it cannot modify canonical text accidentally, has corpus-integrity + Unicode/Arabic tests, preserves exact addressing, outputs source versions, labels generated analysis, works without an LLM, has performance limits, and has API/UI/domain tests | `run_import` implements the §34 pipeline as 13 checkpoints with staging-only writes; QV-001…QV-028 validate content; the round-trip and adversarial suites pass; the purity fence (no LLM/embeddings/retrieval in `quran-core`/`quran-corpus`) is enforced by `xtask arch-check`. Gaps: QV-015 reference comparison has no operator configuration path; no committed integrity artifact; `verify_quotation` has no production call site. [VERIFIED: crates/quran-corpus/src/import.rs:49-94] [VERIFIED: crates/quran-corpus/src/validation.rs:165-545] [VERIFIED: crates/application/src/quran.rs:83] |
</phase_requirements>

## Project Constraints (from AGENTS.md)

- **Read the project briefing in order** before starting work: `AGENTS.md`, `docs/00-overview/project-overview.md`, `docs/03-plan/current-plan.md`, the active phase plan, `docs/06-progress/status.md`, `docs/05-followups/open-questions.md`. [VERIFIED: AGENTS.md:44-52]
- **Every implementation task must have a task ID**, and a task is not complete until tests, lint/checks, acceptance criteria, rollup, follow-ups, and changelog are addressed. [VERIFIED: AGENTS.md:66-74]
- **Documentation taxonomy is fixed**: planning under `docs/03-plan/`, technical under `docs/07-technical/`, follow-ups under `docs/05-followups/`, progress under `docs/06-progress/`. [VERIFIED: AGENTS.md:22-38]
- **Keep the architecture layered.** Domain code must not depend on `api`, `server`, `tui`, `cli`, or a concrete provider. [VERIFIED: .agent/coding-rules.md:5-13]
- **Preserve safety invariants:** "Canonical rows are written only through a `CanonicalWriter` that requires an `ApprovalToken`"; "Every derived artifact records the version of everything it was derived from"; deny-by-default; **"Nothing that can modify data runs inside `doctor`"**; secrets never logged/embedded/plain-config. [VERIFIED: .agent/coding-rules.md:7-22]
- **PRD key principles that bind this phase:** exact text before generated interpretation; every factual claim traceable; canonical text separated from translations and annotations; no fabricated verse/hadith/chain/grading/citation; answers distinguish quotation vs summary vs AI analysis; local data stays local; deny-by-default; no false scholarly consensus. [VERIFIED: AGENTS.md:111-124]
- **Do not silently expand the phase.** Phase 2's boundary is the 5 roadmap success criteria plus the 3 named requirements; normalization/search, graph, GUI/TUI, hadith/tafsir, RAG, agents, and server belong to later phases. [VERIFIED: .planning/phases/02-canonical-quran-core/02-CONTEXT.md:7-11] [VERIFIED: .planning/ROADMAP.md:49-59]
- **Preserve unrelated worktree changes.** This session observed uncommitted concurrent edits (`crates/application/src/quran_doctor_indexes.rs` modified, `crates/application/tests/doctor_indexes.rs` untracked); the research commit must stage only `02-RESEARCH.md`. [VERIFIED: live git status, 2026-09-24]

## Summary

Phase 2 is a **brownfield hardening phase over a substantially complete canonical core**, not a build. Every one of the five roadmap success criteria already has a working implementation in the tree, and four of the five have substantial automated test evidence. The legacy board that built this code is `docs/03-plan/phases/phase-01-core/` (its own acceptance ledger records **0 of 21 criteria verified**, all rows `—` or "partial — automated"), so the honest posture is: implementation landed, evidence is fragmented, closure was never formally recorded. [VERIFIED: docs/03-plan/phases/phase-01-core/done.md:796-826] [VERIFIED: docs/03-plan/phases/phase-01-core/acceptance.md:10] [VERIFIED: docs/03-plan/current-plan.md:1-14]

The genuinely missing or weak items cluster in five places. **(1) QV-015 reference comparison has no operator path**: `ImportInput`/`ImportOptions` carry `reference: Option<EditionSource>`, but `run_import_job` hardcodes `ImportOptions::default()`, so the only production path always records the "skipped" Info finding; and `qai doctor --quran` emits a hardcoded `warn` for `quran.reference_corpus` rather than inspecting configuration. [VERIFIED: crates/quran-corpus/src/import.rs:142-160] [VERIFIED: crates/application/src/quran.rs:83] [VERIFIED: crates/application/src/quran_doctor.rs:438-442] **(2) The canonical-write fence is partial**: insert-only triggers cover `quran_ayahs`, `quran_tokens`, and `quran_editions` identity/hash columns only — `quran_surahs`, `quran_token_separators`, `quran_segments`, `quran_divisions`, `translation_editions`, `translation_passages`, and `word_glosses` have no triggers at all; and the type-level `CanonicalWriter`/`ApprovalToken` gate in `crates/provenance` is **declared but never implemented or called** anywhere. [VERIFIED: migrations/sqlite/0008_quran_structure.up.sql:88-96] [VERIFIED: migrations/sqlite/0007_quran_editions.up.sql:47-51] [VERIFIED: crates/provenance/src/lib.rs:137-216] **(3) `verify_quotation` has zero production call sites** — it is exercised only by `crates/application/tests/quran_tools.rs`, and the one HTTP path that verifies anything uses `resolve_stored` (hash re-verification), not `verify_quotation`. [VERIFIED: crates/server/src/api.rs:1531-1534] **(4) There is no committed corpus-integrity report artifact and no `qai quran verify` command**; `qai quran hashes` explicitly does not recompute. [VERIFIED: crates/application/src/quran_cli.rs:1189-1192] **(5) Translation-edition separation is mostly right but `translation_editions.text_hash` is written as the empty string**, and the manifest-declared `license`/`upstream_edition_slug` are parsed into `EditionMeta` but never persisted on the canonical row. [VERIFIED: crates/application/src/quran.rs:805] [VERIFIED: crates/quran-corpus/src/format.rs:48-95]

What must **not** be rebuilt: the 13-checkpoint importer, the QV validator, the frozen reference grammar and hash recipes, the reader/`QuranQuotation` contract, and the activation/rollback transaction. Their evidence is real and current: `crates/application/tests/quran_import.rs` carries a 13-case crash matrix, cancellation cleanup, rejected-activation and rollback tests; `crates/storage-sqlite/tests/quran.rs` proves raw-SQL trigger aborts with the exact `QAI-QUR-000x` codes; `crates/quran-corpus/tests/adversarial.rs` rejects 16 fixtures by specific rule id; `crates/quran-core/tests/reference_grammar.rs` runs 331 golden reference cases plus a never-panics property. [VERIFIED: crates/application/tests/quran_import.rs:441-484] [VERIFIED: crates/storage-sqlite/tests/quran.rs:233-285] [VERIFIED: crates/quran-corpus/tests/adversarial.rs:31-74] [VERIFIED: crates/quran-core/tests/reference_grammar.rs:35-56]

**Primary recommendation:** plan four bounded work packages — (A) fixture + integrity-evidence surface (richer synthetic edition, `qai quran verify`-style operator check, committed integrity report artifact, QV-015 mechanism behind an owner-gated identity), (B) close the canonical-write fence (triggers for the untriggered canonical tables, wire or explicitly retire the `CanonicalWriter`/`ApprovalToken` layer with adversarial negative tests), (C) quotation hard-failure wiring on every answer path, and (D) translation-layer completion (`text_hash`, license, primary/default flag) — while recording OD-01/OD-02/OD-03 as explicit blocked gates. [ASSUMED: recommended work-package decomposition]

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|---|---|---|---|
| Edition manifest parsing, validation rules (QV-001…QV-028), tokenization, hashing | Pure domain / pipeline (`quran-corpus`) | `quran-core` | No I/O, no async runtime, no model/vector dependency; enforced by `xtask arch-check`. [VERIFIED: crates/quran-corpus/src/lib.rs:1-35] [VERIFIED: .planning/codebase/ARCHITECTURE.md:69-76] |
| Staging writes, activation transaction, pointer flip, generation bump | Storage adapter (`storage-sqlite`) | Storage port (`storage`) | Canonical rows are moved by one SQL transaction in the adapter; the port declares only the four gated mutators. [VERIFIED: crates/storage-sqlite/src/quran.rs:631-748] [VERIFIED: crates/storage/src/quran.rs:1921-1939] |
| Approval verification, audit append, service composition | Composition root (`application`) | `audit`, `sources` | `activate_edition` / `rollback_edition` verify a granted approval covering the exact URN and append a hash-chained audit event in the same unit of work. [VERIFIED: crates/application/src/quran.rs:238-343] |
| Byte-exact canonical lookup + generation-keyed cache | Composition root (`application`) | `quran-core` (types) | `QuranReaderService` resolves the edition, reads canonical rows, and constructs `QuranQuotation` with the stored per-ayah hash. [VERIFIED: crates/application/src/quran_reader.rs:426-535] |
| Frozen reference grammar, quotation type, views | Pure domain (`quran-core`) | — | `parse(serialize(r)) == r`; `QuranQuotation` requires edition + hash; `AyahView.canonical` is Arabic-only. [VERIFIED: crates/quran-core/src/reference/mod.rs:27-30] [VERIFIED: crates/quran-core/src/quotation.rs:5-12] |
| Citation resolution and quotation verification | Pure domain (`citations`) | `application` (implements `CitationSource`) | The resolver reads through a trait so the crate never touches storage; the application layer supplies canonical text. [VERIFIED: crates/citations/src/lib.rs:119-132] [VERIFIED: crates/application/src/quran_tools.rs:125-174] |
| Operator-facing integrity/diagnostic surface | Edge (`cli`) | `application` (`quran_doctor`) | Doctor is read-only by project rule; CLI owns output/exit mapping. [VERIFIED: .agent/coding-rules.md:13-16] [VERIFIED: crates/application/src/quran_doctor.rs:85-95] |
| Canonical immutability enforcement | Database / Storage (triggers) | `provenance` (type gate — currently unused) | Triggers are the last line of defence; the type-level gate is declared in `crates/provenance` but has no implementation. [VERIFIED: migrations/sqlite/0008_quran_structure.up.sql:88-96] [VERIFIED: crates/provenance/src/lib.rs:201-216] |

## Phase 2 Success-Criterion Evidence Matrix

The five rows below are the complete Phase 2 contract from `.planning/ROADMAP.md:53-58`. **Reuse** = do not create a replacement task for working behavior; **harden** = create work only at the demonstrated gap.

| # | Success criterion | Existing implementation | Repeatable evidence | Gap | Planned task if needed |
|---|---|---|---|---|---|
| 1 | Operator can import a Quran edition through staging → validation → atomic activation with rollback. [VERIFIED: .planning/ROADMAP.md:54] | `run_import` runs the 13 checkpoints and writes only `quran_stg_*`; `activate_edition` moves staging → canonical, deprecates the prior `Active`, flips the singleton pointer and bumps `corpus_generation` in one transaction; `rollback_edition` performs the same pointer flip to a prior version. [VERIFIED: crates/quran-corpus/src/import.rs:1304-1379] [VERIFIED: crates/storage-sqlite/src/quran.rs:631-817] [VERIFIED: crates/application/src/quran.rs:301-343] [VERIFIED: crates/application/src/quran.rs:430-468] | `cargo test -p application --test quran_import` — `import_runs_end_to_end_to_staged`, `crash_matrix_all_thirteen_checkpoints_leave_active_untouched` (13/13 prefixes leave `get_active()` `None`), `cancel_cleans_staging_and_marks_cancelled`, `validation_failure_aborts_before_staging_with_report`, `hash_mismatch_aborts_at_the_hash_checkpoint`, `worker_runs_the_import_job_to_staged_with_audit`, `rejected_activations_leave_canonical_state_untouched`, `rollback_service_restores_the_prior_version`. `cargo test -p storage-sqlite --test quran` — `activation_moves_rows_flips_pointer_and_bumps_generation`, `rollback_restores_a_prior_version`, `activation_requires_a_staged_run`. CLI end-to-end `cargo test -p cli --test quran` (`read_flow.trycmd`: migrate → import → activate → … → activate v2 → rollback). [VERIFIED: crates/application/tests/quran_import.rs:125-186] [VERIFIED: crates/application/tests/quran_import.rs:441-484] [VERIFIED: crates/storage-sqlite/tests/quran.rs:350-458] [VERIFIED: crates/cli/tests/quran/read_flow.trycmd:1-40] | **No rejection test for rollback.** Only the happy path is covered: `rollback_service_restores_the_prior_version` activates v1→v2→rollback; there is **no** test that a missing/denied/mismatched approval leaves the active pointer unchanged for `rollback_edition` (activation has such a test; rollback does not). Also: (a) no test that a rollback invalidates the reader cache (the cache test covers activation only); (b) `activate_edition` never copies `quran_stg_segments` → `quran_segments` (the move list omits segments), so the canonical `quran_segments` table is permanently empty; (c) activation `DELETE FROM quran_import_runs` cascades away the staging rows, so "retry is resume" is only true *before* activation. [VERIFIED: crates/application/tests/quran_import.rs:769-819] [VERIFIED: crates/application/tests/quran_import.rs:688-767] [VERIFIED: crates/storage-sqlite/src/quran.rs:676-707] [VERIFIED: crates/storage-sqlite/src/quran.rs:743-747] | **QC-01:** add rollback rejection + cache-invalidation + segments-copy coverage. [ASSUMED: proposed task id] |
| 2 | User can look up any `surah:ayah` and receive byte-exact canonical Arabic with pinned edition reference. [VERIFIED: .planning/ROADMAP.md:55] | `build_view` sets `arabic_text: ayah_row.text.clone()` and `text_hash: parse_tagged(&ayah_row.text_hash)`; the quotation reference is `quran:{slug}@{version}:{surah}:{ayah}`; `QuranQuotation::new` refuses empty reference/text/deep-link. The reader resolves editions by `Active`, `Slug` (active version only), or `Pinned { slug, version }`. [VERIFIED: crates/application/src/quran_reader.rs:454-482] [VERIFIED: crates/quran-core/src/quotation.rs:106-121] [VERIFIED: crates/application/src/quran_reader.rs:363-407] | `cargo test -p application --test quran_reader` — `get_ayah_returns_identified_canonical_text`, `get_ayah_with_tokens_attaches_surfaces`, `missing_references_are_typed_errors`, `ranges_surahs_and_divisions_expand`, `context_respects_boundaries_and_caps` + 2 property tests, `cache_serves_no_stale_text_after_activation`, `lookup_performance_smoke`. `cargo test -p quran-core --test reference_grammar` — `golden_valid_cases_parse_and_serialize` / `golden_invalid_cases_return_the_expected_code` over 331 golden rows + `roundtrip_parse_serialize` + `parser_never_panics_and_errors_are_coded`. Live: `qai quran get 1:1` prints the Arabic line then `— quran:test-edition-min@0.1.0:1:1`. [VERIFIED: crates/application/tests/quran_reader.rs:27-70] [VERIFIED: crates/application/tests/quran_reader.rs:256-323] [VERIFIED: crates/quran-core/tests/reference_grammar.rs:41-56] [VERIFIED: fixtures/quran/golden/references.jsonl] [VERIFIED: crates/cli/tests/quran/read_flow.trycmd:31-38] | Weak evidence only at the **human** surface: `qai quran get` prints text + pinned reference but **not** the text hash, and `qai quran hashes` deliberately does not recompute ("Full recomputation lives in `qai doctor --quran --deep`"). There is no single command whose output is a byte-exact lookup **plus** its hash. Also no test asserts byte-exactness against a *golden* hash for the served view (the golden set covers fixture text, and the doctor deep scan covers recomputation). [VERIFIED: crates/application/src/quran_cli.rs:170-181] [VERIFIED: crates/application/src/quran_cli.rs:1189-1192] [VERIFIED: crates/quran-corpus/tests/fixtures.rs:84-103] | **QC-02:** surface hash + pinned reference together on the lookup path (or assert it in a snapshot), reusing `--json` rather than adding a second reader. [ASSUMED: proposed task id] |
| 3 | Corpus integrity checks (counts, addressing, Unicode, checksums, round-trip, reference comparison) pass. [VERIFIED: .planning/ROADMAP.md:56] | All six families have code: counts QV-001…QV-005; addressing QV-016…QV-020 + QV-023; Unicode QV-006…QV-009 + QV-027; checksums QV-013/QV-014; round-trip QV-010/QV-011/QV-012/QV-024; reference comparison QV-015. `validate_edition` runs every rule with no fail-fast. `qai doctor --quran` exposes 19 checks. [VERIFIED: crates/quran-corpus/src/validation.rs:169-545] [VERIFIED: crates/quran-corpus/src/import.rs:907-921] [VERIFIED: crates/application/src/quran_doctor.rs:94-459] | `cargo test -p quran-corpus --test adversarial` (16 fixtures, specific rule id + Fatal/Error severity), `--test fixtures` (reference comparison exact/order-independent/missing/duplicate/empty/incompatible + golden ayah texts + CSV parity), `-p quran-corpus --lib` (validator unit tests), `cargo test -p application --test quran_doctor` (`recomputed_hashes_match_import_time`, `deep_scan_of_the_fixture_has_no_failures`, `fixture_soak_ten_thousand_lookups_preserves_corpus_integrity`), `cargo test -p cli --test doctor_json`. [VERIFIED: crates/quran-corpus/tests/adversarial.rs:31-74] [VERIFIED: crates/quran-corpus/tests/fixtures.rs:21-103] [VERIFIED: crates/application/tests/quran_doctor.rs:78-113] | **The reference-comparison family is exercised only from Rust tests, never from an operator command.** `run_import_job` hardcodes `ImportOptions::default()` → `reference: None` on every production path; `qai quran import` has no `--reference` flag; and `qai doctor --quran`'s `quran.reference_corpus` check is a **hardcoded** `warn("no reference corpus configured; QV-015 skips (ADR-0114 pending)")` that never inspects state. Additionally (a) the typed comparison taxonomy (`diff_ayahs_typed`, `ComparisonKind`, `DifferenceClass`) is implemented and unit-tested in `differ.rs` but **not wired into** `compared()` — the importer only embeds `CLASSIFICATION_VOCABULARY` as a string; (b) there is **no committed corpus-integrity report artifact** anywhere in the tree; (c) no `qai quran verify` command exists. [VERIFIED: crates/application/src/quran.rs:83] [VERIFIED: crates/application/src/quran_cli.rs:570-671] [VERIFIED: crates/application/src/quran_doctor.rs:438-442] [VERIFIED: crates/quran-corpus/src/import.rs:982-1002] [VERIFIED: crates/quran-corpus/src/differ.rs:29-140] | **QC-03:** implement the D-11 operator/CI surface + committed report artifact, and expose a reference-comparison *mechanism* (identity still `OD-03`-gated). **QC-04:** wire the typed comparison into QV-015 or explicitly record why v1 stays byte-only. [ASSUMED: proposed task ids] |
| 4 | Canonical tables reject all non-approved writes; importer has no code path to canonical tables. [VERIFIED: .planning/ROADMAP.md:57] | (a) Triggers: `trg_ayah_no_update`/`trg_ayah_no_delete`/`trg_token_no_update`/`trg_token_no_delete`/`trg_edition_immutable_hashes`. (b) `canonical_write_surface_is_gated_to_three_mutators` is a source-scan test asserting no `fn insert_ayah(`/`fn insert_surah(`/`fn write_canonical(`-style method exists in the `QuranRepository` trait. (c) `run_import` calls only `clear_staging`, `set_import_run_state`, `insert_stg_*`, `insert_validation_report` and reads. [VERIFIED: migrations/sqlite/0008_quran_structure.up.sql:88-96] [VERIFIED: migrations/sqlite/0007_quran_editions.up.sql:47-51] [VERIFIED: crates/storage/src/quran.rs:1902-1940] [VERIFIED: crates/quran-corpus/src/import.rs:452-1103] | `cargo test -p storage-sqlite --test quran` — `canonical_triggers_abort_raw_writes_with_codes` (5 statements → `QAI-QUR-0001…0005`), `canonical_tables_declare_the_trigger_set`, `edition_verification_stamp_is_metadata_only_and_fail_closed`. `cargo test -p storage --lib quran` — `canonical_write_surface_is_gated_to_three_mutators`, `gated_mutators_fail_closed_without_backend`. `cargo test -p application --test quran_import` — `rejected_activations_leave_canonical_state_untouched`. [VERIFIED: crates/storage-sqlite/tests/quran.rs:233-285] [VERIFIED: crates/storage/src/quran.rs:1942-1957] [VERIFIED: crates/application/tests/quran_import.rs:688-767] | **Two of the three D-13 layers are incomplete.** (a) **Partial trigger coverage:** `quran_surahs`, `quran_token_separators`, `quran_segments`, `quran_divisions`, `translation_editions`, `translation_passages`, `word_glosses` have **no triggers**, and `quran_editions` has no DELETE trigger (only UPDATE of identity/hash columns). (b) **`CanonicalWriter` + `ApprovalToken` are declared but unused:** nothing outside `crates/provenance` constructs an `ApprovalToken`; no type implements `CanonicalWriter`; `ApprovalToken::new` is `pub`, so the type is not actually a gate, and the test named `approval_token_cannot_be_constructed_directly` constructs it directly and asserts a tautology. (c) No test asserts the importer "never holds an `ApprovalToken`" (the importer contains no such type at all), and the "no code path" evidence is an audit of calls, not a test. [VERIFIED: migrations/sqlite/0008_quran_structure.up.sql:88-96] [VERIFIED: migrations/sqlite/0009_quran_divisions.up.sql:1-19] [VERIFIED: migrations/sqlite/0010_quran_translations.up.sql:9-42] [VERIFIED: crates/provenance/src/lib.rs:155-165] [VERIFIED: crates/provenance/src/lib.rs:295-303] | **QC-05:** extend triggers to every canonical table and add adversarial `INSERT`/`UPDATE`/`DELETE` tests for each. **QC-06:** either wire the `CanonicalWriter`/`ApprovalToken` gate to the activation path or record its retirement with a rationale (never leave a declared-but-unused safety layer). [ASSUMED: proposed task ids] |
| 5 | Every quotation verifies via `verify_quotation` with mismatch as a hard failure. [VERIFIED: .planning/ROADMAP.md:58] | `CitationResolver::verify_quotation` and `resolve_stored` return the frozen `QuotationVerdict` set; `Mismatch`/`LocationNotFound`/`EditionNotFound` are documented as hard failures; `persist_citation` stores the resolved hash + verdict. [VERIFIED: crates/citations/src/lib.rs:224-299] [VERIFIED: docs/07-technical/quran-citation-spec.md:48-60] [VERIFIED: crates/application/src/quran_tools.rs:176-205] | `cargo test -p citations` — `exact_match_resolves_with_hash_and_link`, `whitespace_variants_and_mismatches`, `missing_edition_location_and_unparseable`, `links_render`. `cargo test -p application --test quran_tools` — `verify_quotation(&citation, "tampered text")` returns `Mismatch`; the same citation verifies `ExactMatch` for the real text. `cargo test -p server --test api` — `listings_divisions_tokens_resolve_citations`. [VERIFIED: crates/citations/src/lib.rs:379-426] [VERIFIED: crates/application/tests/quran_tools.rs:140-160] [VERIFIED: crates/server/tests/api.rs:523-549] | **`verify_quotation` has no production call site.** Grep across `crates/` finds it only in `crates/citations/src/lib.rs` (definition) and `crates/application/tests/quran_tools.rs` (test). The only HTTP verification path is `GET /api/v1/quran/citations/{id}` → `api_citation` → `resolve_stored` (stored-hash re-verification), not `verify_quotation`. There is **no CLI verb** and **no endpoint** that accepts an externally supplied quotation and verifies it, and **no shared helper that turns a non-`ExactMatch` verdict into a hard failure** — the mapping from verdict to error/exit code does not exist. [VERIFIED: crates/server/src/api.rs:584-598] [VERIFIED: crates/server/src/api.rs:1531-1534] [VERIFIED: crates/application/src/quran_tools.rs:109-123] [VERIFIED: crates/cli/src/quran.rs:14-210] | **QC-07:** add the verifying surface (CLI verb + shared verdict→hard-failure mapping) and enforce it on every path that emits a quotation supplied from outside canonical storage. Note the honest nuance: the *read* paths (`qai quran get`, `GET /api/v1/quran/ayahs/{ref}`, `quran.get_ayah`) read canonical rows directly and therefore cannot mismatch — they are the source of truth, not quoters. The contract D-15 protects is `docs/07-technical/quran-citation-spec.md:58-60`. [ASSUMED: recommended enforcement shape] |

## Current Evidence to Reuse

Do not rebuild these working seams:

- **13-checkpoint importer** (`ImportCheckpoint::ALL`, `run_import`, deterministic restart-is-resume, cancellation cleanup, `stop_after` dry-run). [VERIFIED: crates/quran-corpus/src/import.rs:80-94] [VERIFIED: crates/quran-corpus/src/import.rs:1304-1379]
- **QV-001…QV-028 validator** running all rules with no fail-fast and a machine-readable `ValidationReport`. [VERIFIED: crates/quran-corpus/src/validation.rs:5-17] [VERIFIED: crates/quran-corpus/src/validation.rs:165-545]
- **Frozen reference grammar + serializer** with 331 golden cases and a never-panics property. [VERIFIED: crates/quran-core/src/reference/mod.rs:1-36] [VERIFIED: crates/quran-core/tests/reference_grammar.rs:35-56]
- **Frozen hashing recipes** `qai-text-hash-v1` / `structure_hash` / `token_order_hash` (length-prefixed, domain-separated). [VERIFIED: crates/quran-corpus/src/hashing.rs:1-27]
- **`QuranQuotation` + `AyahView` type guards** (quotation requires edition + hash; translation can only ride alongside). [VERIFIED: crates/quran-core/src/quotation.rs:104-136] [VERIFIED: crates/quran-core/src/view.rs:29-45]
- **Atomic activation/rollback transaction** with approval check + same-transaction audit. [VERIFIED: crates/storage-sqlite/src/quran.rs:631-817] [VERIFIED: crates/application/src/quran.rs:238-343]
- **Generation-keyed reader cache** (`(edition.id, version, generation, serialize(reference), options_hash)`). [VERIFIED: crates/application/src/quran_reader.rs:327-361]
- **19-check `qai doctor --quran`** read-only diagnostic with `Pass/Warn/Fail/Skipped` + remedy + next command. [VERIFIED: crates/application/src/quran_doctor.rs:30-77]
- **CI/build gates**: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `xtask arch-check`, `xtask migrate-check`, `xtask gen-schema` diff, `xtask adr-lint`, coverage gate, `cargo-deny`. [VERIFIED: xtask/src/ci.rs:15-104] [VERIFIED: .github/workflows/ci.yml:29-128]

## Implementation Gap Register

Every row is a *demonstrated* gap with a file:line seam; none proposes new architecture.

| Gap ID | Exact current seam | Evidence (opened this session) | Prescriptive direction | Required evidence |
|---|---|---|---|---|
| QC-01 | `crates/application/src/quran.rs::rollback_edition` (no rejection test); `crates/storage-sqlite/src/quran.rs::activate_edition` move list omits `quran_stg_segments`. [VERIFIED: crates/application/src/quran.rs:429-468] [VERIFIED: crates/storage-sqlite/src/quran.rs:676-707] | `rollback_service_restores_the_prior_version` covers only the happy path; `rejected_activations_leave_canonical_state_untouched` covers activation only. [VERIFIED: crates/application/tests/quran_import.rs:769-819] | Add rollback rejection tests (missing/denied/mismatched approval → pointer unchanged), a rollback cache-invalidation test, and either copy `quran_segments` during activation or record why segments are staging-only. | `cargo test -p application --test quran_import`; `cargo test -p storage-sqlite --test quran`. [ASSUMED: proposed test names] |
| QC-02 | `crates/application/src/quran_cli.rs::cmd_get` prints text + reference but no hash; `cmd_hashes` does not recompute. [VERIFIED: crates/application/src/quran_cli.rs:170-181] [VERIFIED: crates/application/src/quran_cli.rs:1189-1192] | The byte-exact guarantee is proven in library tests but no operator command emits lookup + hash together. | Surface the stored `text_hash` (already in `AyahView` JSON) and the pinned reference on the human line, or assert it in the existing trycmd snapshot; do not add a second reader. | `cargo test -p cli --test quran` snapshot update. [ASSUMED] |
| QC-03 | No operator/CI integrity surface; no committed report artifact. [VERIFIED: crates/cli/src/quran.rs:14-210] | Grep for an integrity artifact found none outside ADR-0009/audit tests. `qai doctor --quran` reports 19 checks but nothing persists them. | Add a repeatable integrity command reusing `run_quran_checks` + `validate_staged`, and commit its JSON as the phase's evidence-of-record. Keep it read-only (`.agent/coding-rules.md`). | New CLI trycmd case + committed artifact; `cargo test -p cli --test doctor_json`. [ASSUMED] |
| QC-04 | `crates/quran-corpus/src/import.rs::compared` uses `compare_reference` (raw bytes) and only embeds `CLASSIFICATION_VOCABULARY` as a string; `diff_ayahs_typed` is unused. [VERIFIED: crates/quran-corpus/src/import.rs:925-1045] [VERIFIED: crates/quran-corpus/src/differ.rs:237-310] | `diff_ayahs_typed` / `ComparisonKind` / `DifferenceClass` / `classify_difference` appear only in `differ.rs` and its `lib.rs` re-export. | Either call `diff_ayahs_typed` for the QV-015 report so each difference carries a `DifferenceClass`, or record explicitly that v1 QV-015 is byte-only and the typed layer is reserved for the version/readings comparisons. Do not invent a second classification vocabulary. | `cargo test -p quran-corpus --lib differ` + reference tests. [ASSUMED] |
| QC-05 | Triggers exist only for `quran_ayahs`, `quran_tokens`, `quran_editions` identity columns. [VERIFIED: migrations/sqlite/0008_quran_structure.up.sql:88-96] [VERIFIED: migrations/sqlite/0007_quran_editions.up.sql:47-51] | `grep -rn "CREATE TRIGGER" migrations/sqlite/*.up.sql` returns only migrations 0003, 0005, 0006, 0007, 0008, 0013. `quran_surahs`, `quran_token_separators`, `quran_segments`, `quran_divisions`, `translation_editions`, `translation_passages`, `word_glosses` are absent. | Add insert-only triggers (UPDATE/DELETE aborts with new `QAI-QUR-*` codes) for every canonical table, appended as a **new** migration (migrations are append-only, ADR-0002) — never edit an applied file. | Extend `canonical_triggers_abort_raw_writes_with_codes` per table; `cargo run -q -p xtask -- migrate-check`. [ASSUMED: new migration number] |
| QC-06 | `CanonicalWriter` trait + `ApprovalToken` declared with no implementor and no caller; `ApprovalToken::new` is `pub`. [VERIFIED: crates/provenance/src/lib.rs:137-216] | `grep -rn 'CanonicalWriter\|ApprovalToken' crates/` shows only definition sites, the provenance crate's own vacuous test, and a `testkit` diagnostic assertion. | Decide explicitly: wire the gate to `activate_edition`/`rollback_edition` (a token minted from the persisted granted approval row), **or** record its retirement as an ADR-level change. Leaving `.agent/coding-rules.md`'s stated invariant as dead code is the one outcome to avoid. | A test that a canonical write without the gate cannot compile/execute; or an ADR revision. [ASSUMED] |
| QC-07 | `verify_quotation` has no production call site; no verdict→hard-failure mapping. [VERIFIED: crates/citations/src/lib.rs:224-233] | Only `crates/application/tests/quran_tools.rs:147` calls it. `crates/server/src/api.rs:1531-1534` uses `resolve_stored`. | Add a shared helper that maps `Mismatch`/`LocationNotFound`/`EditionNotFound` to a typed `QAI-QUR-*` diagnostic + CLI exit, and expose one verifying surface (CLI verb and/or HTTP endpoint) so the contract is exercised in production. | New test asserting the hard-failure mapping; a trycmd case for the CLI exit code. [ASSUMED: exact verb name is a planner choice] |
| QC-08 | `translation_editions.text_hash` is written as `String::new()`. [VERIFIED: crates/application/src/quran.rs:805] | `import_translations_persists_attributed_edition_and_passages` asserts slug/version/translator/language/status/alignment but **not** `text_hash`. [VERIFIED: crates/application/tests/quran_translation.rs:36-77] | Compute a translation text hash (its own domain-separated recipe per D-12: additive, never reusing `qai-text-hash-v1`) over passages in `(surah, ayah)` order; store the `sha256:<hex>` form. Additive recipe only. | `cargo test -p application --test quran_translation` asserting a 64-hex `sha256:` value. [ASSUMED: recipe naming] |
| QC-09 | `EditionMeta.{upstream_edition_slug, qai_edition_id, license, verified_by, source}` are parsed but never persisted on the canonical row; `quran_editions` has no such columns, and `cmd_import` hardcodes `license_status: "Unknown"` + a synthetic license JSON. [VERIFIED: crates/quran-corpus/src/format.rs:48-95] [VERIFIED: migrations/sqlite/0007_quran_editions.up.sql:3-34] [VERIFIED: crates/application/src/quran_cli.rs:641-652] | `upstream_edition_slug` appears only in `format.rs`, `upstream_catalog.rs` (catalog path), `quran-core/src/catalog.rs`, and `cmd_catalog`. `grep 'edition.license'` in `quran-corpus/src/import.rs` returns nothing. | Record the upstream identity + declared license on the canonical edition (column or provenance/versions JSON), preserving the value verbatim (ADR-0101: "preserved exactly as `upstream_edition_slug`"). Identity values stay `OD-01`-gated; the *shape* is not. | Import test asserting a declared `upstream_edition_slug` survives to the canonical row. [ASSUMED: storage location] |
| QC-10 | `qai doctor --quran`'s `quran.reference_corpus` check is a hardcoded warn. [VERIFIED: crates/application/src/quran_doctor.rs:438-442] | The message string is unconditional — it never reads configuration or import options. | Derive the check from actual state (was a reference configured for the active edition? was QV-015 skipped or evaluated?) so it can report Pass/Skip/Fail honestly. | `cargo test -p application --test quran_doctor` + `cargo test -p cli --test doctor_json`. [ASSUMED] |
| QC-11 | No primary/default edition flag; `EditionSelector` is `Active`/`Slug`/`Pinned` only. [VERIFIED: crates/quran-core/src/enums.rs:117-129] [VERIFIED: migrations/sqlite/0007_quran_editions.up.sql:36-44] | Grep for `primary`/`is_default`/`default_edition` in `quran-core`, `storage`, and the editions migration returns nothing. | Add an explicit primary/default flag (Uthmani + Ḥafṣ ʿan ʿĀṣim) that feeds the existing `quran_active_edition` singleton — "primary default" is not "only edition" (D-07). | A test that the flag is settable/readable and does not displace the active pointer semantics. [ASSUMED: exact column/flag shape] |
| QC-12 | Coverage gate omits `quran-core`, `quran-corpus`, `citations`. [VERIFIED: xtask/src/coverage.rs:28-36] | `THRESHOLDS` lists `domain`, `provenance`, `audit`, `sources` (85%) and `config`, `jobs`, `storage-sqlite` (75%) — none of the Quran crates. The legacy ledger's §4 claims quran-core ≥90% / validation ≥90% / citations ≥85% "ratified (OD-07)". [VERIFIED: docs/03-plan/phases/phase-01-core/acceptance.md:256-276] | Reconcile: either add the Phase 2 crates to `THRESHOLDS`, or record that OD-07's Quran-specific floors were not implemented and re-scope. Do **not** silently keep an unmet published gate. | `cargo run -q -p xtask -- coverage-gate <lcov.info>`. [ASSUMED] |
| QC-13 | Adversarial fixtures are format-valid for 15 manifests; `hash_mismatch` is a source package. The validator test asserts one rule id per fixture. [VERIFIED: crates/quran-corpus/tests/fixtures.rs:152-182] [VERIFIED: crates/quran-corpus/tests/adversarial.rs:31-74] | `zero_width` asserts only `QV-008` although the ledger documents `QV-008 / QV-011`; `truncated_ayah` asserts `QV-011` (ledger allows `QV-011 / QV-015`). | Keep the fixtures; extend them rather than replacing (D-08). Treat single-id assertions as stronger evidence than the ledger's alternatives, not as drift. | Unchanged suites remain green after fixture additions. [VERIFIED: docs/03-plan/phases/phase-01-core/acceptance.md:146-171] |

## Standard Stack

No new external package is required or recommended for this phase.

### Core

| Library / tool | Version | Purpose | Why standard for this phase |
|---|---:|---------|------------------------------|
| Rust toolchain | `1.97.1` | Language + pinned compiler with rustfmt/clippy. | Pinned by `rust-toolchain.toml`; reuse rather than change. [VERIFIED: rust-toolchain.toml:1-5] |
| SQLx | `0.8.6` | SQLite pools, transactions, migration execution. | Already the sole storage adapter; owns the shared `UnitOfWork`. [VERIFIED: .planning/codebase/STACK.md:40-46] |
| SQLite (bundled via SQLx) | `3.51.0` CLI observed | Local authoritative relational store. | Locked by ADR-0001 (SQLite authority, PG-portable contracts). [VERIFIED: live `sqlite3 --version`] |
| `sha2` | workspace dep | SHA-256 for corpus, per-ayah/per-token, and manifest hashes. | Frozen by ADR-0108. [VERIFIED: crates/quran-corpus/src/hashing.rs:1-27] |
| `unicode-normalization`, `unicode-segmentation` | workspace deps | NFC/NFD detection and grapheme-cluster counting. | Locked by ADR-0104. [VERIFIED: docs/02-architecture/decisions/ADR-0104-unicode-policy.md:24-35] |
| `datatest`-free golden loop + `proptest` | workspace `1.x` | Golden JSONL suites + property tests (round-trip, never-panics). | Existing pattern; do not add a new test framework. [VERIFIED: .planning/codebase/TESTING.md:9-19] |
| `trycmd` | `0.15.x` | Real-binary CLI snapshot + exit-code assertions. | Already drives `qai` through `env!("CARGO_BIN_EXE_qai")` with `QAI_DATA_DIR`. [VERIFIED: crates/cli/tests/quran.rs:11-20] |
| `tempfile` | workspace dep | Isolated data directories for real-SQLite tests. | Existing pattern (`migrated_db()` helpers). [VERIFIED: crates/storage-sqlite/tests/quran.rs:32-95] |
| `lru` | workspace dep | Generation-keyed canonical lookup cache. | Locked by ADR-0113. [VERIFIED: crates/application/src/quran_reader.rs:327-345] |

### Supporting

| Library / tool | Version | Purpose | When to use |
|---|---:|---|---|
| `xtask` (workspace binary) | in-repo | `arch-check`, `migrate-check`, `ci`, `gen-schema`, `adr-lint`, `coverage-gate`, `validate`. | The machine gates for every wave. [VERIFIED: xtask/src/main.rs:28-63] |
| `cargo-llvm-cov` | installed locally | Produce the LCOV report the coverage gate consumes. | Only if QC-12 is taken into scope. [VERIFIED: live probe: `/Users/ali/.cargo/bin/cargo-llvm-cov`] |
| `cargo-deny` | **not installed locally** | Advisory/license/source policy CI gate. | CI enforces it; `xtask ci` warns and skips when absent. [VERIFIED: .github/workflows/ci.yml:14-25] [VERIFIED: xtask/src/ci.rs:29-48] |

**Installation:** none. Build with the existing lockfile: `cargo build -p cli --bin qai`. Do not run `cargo add` or upgrade dependencies in this phase. [ASSUMED: preserving the locked stack is the recommended scope]

**Version verification:** workspace toolchain verified live (`rustc 1.97.1`, `cargo 1.97.1`) and package versions were read from the in-repo resolution records; no registry publish-date probe was performed this session, and no upgrade is proposed. [VERIFIED: rust-toolchain.toml:1-5] [VERIFIED: live `rustc --version` / `cargo --version`, 2026-09-24] [ASSUMED: no upgrade recommendation]

## Package Legitimacy Audit

**No new package is proposed or required for Phase 2.** The package-legitimacy gate is therefore not applicable to an install action in this phase; existing workspace dependencies are reused under the lockfile and are not reclassified as newly approved packages.

| Package family | Registry | Resolved-version evidence | New install | Disposition |
|---|---|---|---|---|
| `sha2`, `unicode-normalization`, `unicode-segmentation`, `lru`, `serde`, `thiserror` | crates.io | Already resolved in the workspace; used by the existing canonical path. [VERIFIED: crates/quran-corpus/src/hashing.rs:28-29] [VERIFIED: crates/quran-core/src/quotation.rs:14-15] | No | Reuse; no new legitimacy decision required. |
| Any new fixture/report/CLI dependency | Any | None proposed. [ASSUMED: no new dependency] | No | Do not add without a separate measured need and package audit. |

**Packages removed due to a SLOP verdict:** none — no new package was researched for installation. **Packages flagged as SUS:** none. [ASSUMED: no new package is in scope]

## Architecture Patterns

### System Architecture Diagram

Data flows one way, from untrusted bytes to a human-gated canonical publication; nothing below the activation arrow can reach the canonical tables.

```text
                          operator / CI
                               |
                               v
                 +--------------------------------+
                 |  qai quran import <manifest>   |
                 |  (or quran.import job)         |
                 +--------------------------------+
                               |
                     manifest bytes (untrusted)
                               v
      +-------------------------------------------------------+
      | quran-corpus::import::run_import  (13 checkpoints)     |
      |  Claimed > ManifestHashed > AdapterSelected > Parsed   |
      |  > UnicodeAudited > Validated > Tokenized              |
      |  > HashesComputed > Staged > RoundtripVerified         |
      |  > ReferenceCompared > Diffed > ApprovalRequested      |
      +-------------------------------------------------------+
             |            |                       |
             |            |                       +--> validation_reports
             |            |                       +--> difference_reports
             |            +--> QV-001..QV-028 (no fail-fast) --> Fatal? grep
             v
      +--------------------------------------------+
      | quran_stg_*  (trigger-free staging mirror)  |
      | cascade ON DELETE from quran_import_runs    |
      +--------------------------------------------+
                               |
                    =========== | ===== APPROVAL GATE (persisted, human) =====
                               v
      +-------------------------------------------------------+
      | application::activate_edition / rollback_edition        |
      |   check_approval(granted AND exact urn)                 |
      |   + one UnitOfWork:                                     |
      |       INSERT canonical rows (SELECT FROM staging)       |
      |       UPDATE prior Active -> Deprecated                 |
      |       DELETE + INSERT quran_active_edition (gen++)      |
      |       append hash-chained audit event                   |
      |       commit  (all-or-nothing)                          |
      +-------------------------------------------------------+
                               |
                               v
      +-------------------------------------------------------+
      | quran_editions / quran_surahs / quran_ayahs /           |
      | quran_tokens / quran_token_separators / quran_divisions |
      | (insert-only triggers: QAI-QUR-0001..0005)              |
      +-------------------------------------------------------+
                               |
                               v
      +-------------------------------------------------------+
      | application::QuranReaderService                         |
      |  resolve_edition -> generation                           |
      |  LRU key = (id, version, generation, ref, options_hash)  |
      |  build_view -> QuranQuotation{arabic_text, text_hash,    |
      |                edition{slug,version,script,riwayah}}     |
      +-------------------------------------------------------+
             |                    |                    |
             v                    v                    v
     qai quran get/context   GET /api/v1/quran/...   quran.get_ayah (tools)
             |                    |                    |
             +--------------------+--------------------+
                                  |
                                  v
                  citations::CitationResolver
                  resolve / verify_quotation / resolve_stored
                  -> ExactMatch | whitespace | declared-norm
                  -> Mismatch | LocationNotFound | EditionNotFound  (HARD FAILURE)
```

The primary decision branches are: a Fatal QV finding → the run is marked `Failed` touching no canonical row; a non-`Staged` run or a non-granted/mismatched approval → activation aborts and the active pointer is unchanged; a generation bump → the reader cache key changes so no stale text can be served; and a non-`ExactMatch` verdict → a hard failure that must never be downgraded. [VERIFIED: crates/quran-corpus/src/import.rs:519-544] [VERIFIED: crates/storage-sqlite/src/quran.rs:645-655] [VERIFIED: crates/application/src/quran_reader.rs:352-361] [VERIFIED: docs/07-technical/quran-citation-spec.md:53-60]

### Recommended Project Structure

```text
crates/
├── quran-core/src/            # reference grammar, quotation, views (unchanged)
├── quran-corpus/src/
│   ├── import.rs              # 13 checkpoints; QV-015 comparison (QC-04)
│   ├── validation.rs          # QV-001..QV-028
│   ├── differ.rs              # typed comparison taxonomy (QC-04)
│   └── hashing.rs             # FROZEN v1 recipes (D-12: additive only)
├── citations/src/lib.rs       # resolver + verify_quotation (QC-07)
├── application/src/
│   ├── quran.rs               # import handler, activate/rollback (QC-01)
│   ├── quran_cli.rs           # operator surfaces (QC-02, QC-03, QC-09)
│   ├── quran_reader.rs        # byte-exact lookup + cache (QC-02)
│   ├── quran_doctor.rs        # integrity checks (QC-10)
│   └── quran_tools.rs         # answer-path backends (QC-07)
├── storage/src/quran.rs       # gated-mutator contract (QC-06)
├── storage-sqlite/src/quran.rs# activation transaction (QC-01, QC-05)
└── cli/src/quran.rs           # `qai quran …` verbs (QC-03, QC-07)
migrations/sqlite/             # append-only (QC-05 adds a new file, never edits)
fixtures/quran/                # richer synthetic edition (D-08), keep adversarial
```

This is a map to existing modules, not a request to create parallel crates. [VERIFIED: .planning/codebase/ARCHITECTURE.md:52-76]

### Pattern 1: Staging-Only Import, Human-Gated Activation

**What:** The importer writes only `quran_stg_*` and ends at `ApprovalRequested`; a separate transaction moves rows to canonical under a granted approval.

**When to use:** Every canonical edition import. Never add an "activate during import" shortcut.

**Verified pattern:** the importer's only `uow.quran()` calls are `clear_staging`, `set_import_run_state`, `insert_stg_*`, `insert_validation_report`, and reads (`get_active`, `get_edition`, `get_import_run`, `get_validation_report`, `list_stg_*`). [VERIFIED: crates/quran-corpus/src/import.rs:452-1103]

### Pattern 2: One Transaction, Approval + Move + Audit

**What:** `activate_edition` verifies the approval, calls the storage mutator, appends the audit event, and commits — all in one `UnitOfWork`; any failure aborts everything.

**When to use:** Any canonical publication or pointer change. [VERIFIED: crates/application/src/quran.rs:301-343]

**Verified approval guard:** `check_approval` requires `decision == Some("approved")` **and** `approval.subject_urn == "quran-edition:{slug}@{version}"`. [VERIFIED: crates/application/src/quran.rs:238-260]

### Pattern 3: Frozen Addressing and Hashing

**What:** `parse(serialize(r)) == r` for every reference; `qai-text-hash-v1` is length-prefixed and domain-separated.

**When to use:** Every stored citation, deep link, tool result, and derived index. Adding a new hash domain is allowed; changing a v1 recipe is not (D-12). [VERIFIED: crates/quran-core/src/reference/mod.rs:27-30] [VERIFIED: crates/quran-corpus/src/hashing.rs:8-27]

### Pattern 4: Generation-Keyed Cache Invalidation

**What:** The reader cache key includes `corpus_generation`; an activation or rollback bumps it, so the key space changes and stale text is unreachable.

**When to use:** Every derived surface (reader, indexes, search). [VERIFIED: crates/application/src/quran_reader.rs:352-361]

### Pattern 5: Verdict-as-Hard-Failure

**What:** `CitationResolver` returns a `QuotationVerdict`; `Mismatch` and `LocationNotFound` must become hard failures rather than warnings.

**When to use:** Any surface that echoes a quotation supplied from outside canonical storage. **This pattern currently has no production caller** (QC-07). [VERIFIED: crates/citations/src/lib.rs:53-77] [VERIFIED: docs/07-technical/quran-citation-spec.md:53-60]

### Anti-Patterns to Avoid

- **Rebuilding the importer, validator, grammar, or hash recipes.** They are frozen and load-bearing for every later phase. [VERIFIED: docs/03-plan/phases/phase-01-core/acceptance.md:324-348]
- **Editing an applied migration.** Migrations are append-only and checksummed; add a new file. [VERIFIED: .planning/PROJECT.md:75]
- **Adding an `insert_ayah`/`write_canonical` repository method** to "fix" a test. The gated-mutator source test forbids it by name. [VERIFIED: crates/storage/src/quran.rs:1921-1939]
- **Making doctor green by writing probes or files.** Nothing that can modify data runs inside `doctor`. [VERIFIED: .agent/coding-rules.md:13-16]
- **Freezing a new hash recipe into v1's slot.** Use a new, domain-separated name (D-12). [VERIFIED: .planning/phases/02-canonical-quran-core/02-CONTEXT.md:34]
- **Treating the `CanonicalWriter` type as already satisfying D-13(b).** The trait has no implementor and the token is publicly constructible. [VERIFIED: crates/provenance/src/lib.rs:155-216]
- **Reusing the Hafs/Uthmani checksum for another reading.** One manifest per edition (ADR-0114 §3). [VERIFIED: docs/02-architecture/upstream-sources.md:286-294]
- **Fabricating an `upstream_edition_slug`, publisher, license, or reviewer.** These are owner gates (OD-01/OD-02). [VERIFIED: docs/05-followups/decisions-needed.md:28-72]

## Don't Hand-Roll

| Problem | Don't build | Use instead | Why |
|---|---|---|---|
| Reference parsing / serialization | A second parser, regex, or ad-hoc splitting | `quran_core::{parse, serialize, resolve, canonical_form}` | Frozen ADR-0102 grammar with 331 golden cases and a never-panics property; every stored citation depends on it. [VERIFIED: crates/quran-core/src/reference/mod.rs:161-165] |
| Tokenization / lossless reconstruction | A new splitter or a whitespace `split()` | `quran_corpus::{tokenize, reconstruct}` + `quran_token_separators` | Separators are recorded exactly and offsets map to grapheme clusters; a naive split loses waqf marks and page-spanning spacing. [VERIFIED: crates/quran-corpus/src/tokenize.rs:1-58] |
| Corpus / ayah / token hashing | Any new hash over the text | `quran_corpus::hashing::{text_hash, structure_hash, token_order_hash}` (frozen) | Changing a v1 recipe invalidates every stored hash and needs a global re-hash migration (D-12). [VERIFIED: crates/quran-corpus/src/hashing.rs:8-27] |
| Unicode validation | Hand-rolled char-range checks | `quran_corpus::unicode::{normalization_form, find_forbidden, first_unexpected}` | ADR-0104's forbidden set (controls, BOM, bidi controls, ZWJ/ZWNJ, private-use, noncharacters) is already encoded. [VERIFIED: crates/quran-corpus/src/unicode.rs:38-53] |
| Edition validation | A new integrity checker | `quran_corpus::validate_edition` + `check_file_hash` | 28 rules with defined severities; a partial report is useless for editorial review. [VERIFIED: crates/quran-corpus/src/validation.rs:165-168] |
| Typed comparison classification | A new difference vocabulary | `quran_corpus::differ::{CLASSIFICATION_VOCABULARY, ComparisonKind, DifferenceClass, classify_difference, diff_ayahs_typed}` | ADR-0114 already fixes the closed vocabulary (`same`, `normalization_only`, `orthographic_difference`, …). [VERIFIED: crates/quran-corpus/src/differ.rs:20-140] [VERIFIED: docs/02-architecture/decisions/ADR-0114-reference-corpus-comparison.md:60-72] |
| Atomic multi-table canonical move | A sequence of application-level writes with compensating deletes | `SqliteQuranRepository::{activate_edition, rollback_edition}` inside one `UnitOfWork` | The move, deprecation, pointer flip, and generation bump are one SQL transaction. [VERIFIED: crates/storage-sqlite/src/quran.rs:631-748] |
| Citation verification | A string comparison at the call site | `citations::CitationResolver::{verify_quotation, resolve_stored}` + `QuranQuotation` | The verdict vocabulary and hard-failure semantics are a published contract. [VERIFIED: crates/citations/src/lib.rs:53-77] |
| Operator diagnostics | A new health command with its own checks | `application::quran_doctor::run_quran_checks` (19 checks) + `cli::doctor` rendering | Keeps doctor read-only and keeps remedies/next-commands in one registry. [VERIFIED: crates/application/src/quran_doctor.rs:55-77] |
| Real-binary CLI assertions | Mocking the binary | `trycmd` with an isolated `QAI_DATA_DIR` | Existing harness already drives the real `qai` binary and asserts exit codes. [VERIFIED: crates/cli/tests/quran.rs:11-20] |

**Key insight:** the hard problems in this phase are *evidence and completeness at the seams*, not missing cryptographic, linguistic, or storage technology. Every gap in the register is a wiring, coverage, or artifact gap — none requires a new library or a new subsystem. [VERIFIED: crates/quran-corpus/src/import.rs:1-17] [VERIFIED: crates/application/src/quran.rs:1-12]

## Common Pitfalls

### Pitfall 1: Believing D-13(b) is already satisfied because the types exist

**What goes wrong:** The plan records "provenance gate proven" and proceeds, while the only real enforcement is SQL triggers on three tables plus an approval-row check inside `activate_edition`. [VERIFIED: crates/provenance/src/lib.rs:201-216] [VERIFIED: crates/application/src/quran.rs:238-260]

**Why it happens:** `CanonicalWriter` and `ApprovalToken` are declared in `crates/provenance` and `.agent/coding-rules.md` asserts them, so a surface read suggests the layer is live. In fact nothing implements `CanonicalWriter`, nothing outside the crate's own test constructs an `ApprovalToken`, and `ApprovalToken::new` is `pub`. [VERIFIED: .agent/coding-rules.md:10-11] [VERIFIED: crates/provenance/src/lib.rs:155-165]

**How to avoid:** Treat D-13(b) as an open decision (QC-06): wire it or retire it with a recorded rationale. Do not write a "gate exists" claim without a test.

**Warning signs:** a test named `approval_token_cannot_be_constructed_directly` whose body constructs the token and asserts `a == a`. [VERIFIED: crates/provenance/src/lib.rs:295-303]

### Pitfall 2: Treating trigger coverage on three tables as "canonical tables reject all non-approved writes"

**What goes wrong:** `quran_surahs`, `quran_token_separators`, `quran_segments`, `quran_divisions`, and every translation table accept arbitrary `INSERT`/`UPDATE`/`DELETE` today. [VERIFIED: migrations/sqlite/0008_quran_structure.up.sql:88-96]

**Why it happens:** The original acceptance criterion named only `quran_ayahs`/`quran_tokens` (`AC-P1-08`), so the trigger set was scoped to those plus the edition-hash columns. [VERIFIED: docs/03-plan/phases/phase-01-core/acceptance.md:55]

**How to avoid:** Enumerate canonical tables from the migrations and assert one trigger set per table in `canonical_tables_declare_the_trigger_set` (QC-05).

**Warning signs:** a raw `UPDATE quran_surahs SET ayah_count = 0` succeeds in a test.

### Pitfall 3: "QV-015 passes" when it was actually skipped

**What goes wrong:** A green import is read as reference comparison success, but `run_import_job` hardcodes `ImportOptions::default()`, so the operator path always emits the single `Info` finding "reference-corpus comparison skipped: no reference corpus configured". [VERIFIED: crates/application/src/quran.rs:83] [VERIFIED: crates/quran-corpus/src/import.rs:176-183]

**Why it happens:** `ImportOptions.reference` is only ever `Some` in Rust tests, and `qai quran import` exposes no flag. [VERIFIED: crates/application/tests/quran_import.rs:220-221]

**How to avoid:** Make the skip state explicit in every operator artifact and in the doctor check (QC-03, QC-10); never let a skipped family be counted as passed (D-10).

**Warning signs:** a report that lists six families as "pass" while `quran.reference_corpus` is a static warn.

### Pitfall 4: Adding `verify_quotation` to a read path that cannot mismatch

**What goes wrong:** Effort is spent wrapping `qai quran get` in `verify_quotation`, which compares the canonical text against… the canonical text. It always returns `ExactMatch` and proves nothing.

**Why it happens:** D-15 says "every current answer path"; the read paths *serve* canonical text from the source of truth rather than echoing a supplied quotation. [VERIFIED: crates/application/src/quran_reader.rs:458-482]

**How to avoid:** Identify the paths that accept an *externally supplied* quotation (today: none in production), build the verifying surface, and enforce the verdict→hard-failure mapping there. Record the read paths as structurally exempt because they read canonical rows directly.

**Warning signs:** a new test where the "expected" and "actual" strings come from the same variable.

### Pitfall 5: Hand-editing a migration or a v1 hash recipe

**What goes wrong:** Adding a trigger to `0008_quran_structure.up.sql` (or touching `hashing.rs`) breaks `migrate-check` / invalidates every stored hash. [VERIFIED: crates/storage-sqlite/src/migrate.rs:168-219]

**Why it happens:** The natural place to add a missing trigger is "next to the other triggers".

**How to avoid:** append a new migration file; put any new hash in a new domain-separated recipe (D-12).

**Warning signs:** `cargo run -q -p xtask -- migrate-check` fails, or a stored `text_hash` no longer recomputes.

### Pitfall 6: Assuming the canonical row carries the upstream identity

**What goes wrong:** A plan states "the edition records its `upstream_edition_slug` and license", but `EditionMeta` is parsed and then dropped at staging/activation; `cmd_import` also hardcodes `license_status: "Unknown"` and a synthetic license JSON. [VERIFIED: crates/quran-corpus/src/format.rs:48-95] [VERIFIED: crates/application/src/quran_cli.rs:641-652]

**Why it happens:** The intermediate format grew richer than the storage schema, and the catalog command (`qai quran catalog`) shows the field working — in a different code path. [VERIFIED: crates/quran-corpus/src/upstream_catalog.rs:192-194]

**How to avoid:** Make the persistence of upstream identity an explicit QC-09 task; verify with an import test, not a catalog test.

**Warning signs:** `qai quran edition show` cannot display an upstream slug.

### Pitfall 7: Assuming the cache key protects against rollback staleness

**What goes wrong:** The generation-keyed key is correct, but only an *activation* transition is tested end to end; a rollback path bug could serve v2 text after returning to v1. [VERIFIED: crates/application/tests/quran_reader.rs:256-323]

**Why it happens:** The cache test was written for AC-P1-19 (post-activation), and rollback's pointer flip reuses the same generation bump.

**How to avoid:** add a rollback cache-invalidation test (QC-01).

**Warning signs:** the only "no stale text" test name mentions activation.

## Domain Findings

### F-1: Import → activation → rollback trace (criterion 1)

**Import.** `pub async fn run_import(db, input, options, cancel, progress)` drives the 13 checkpoints in a fixed order, re-checking cancellation before each. `claimed()` is an idempotent restart: an existing run has its staging cleared and is set back to `"Running"`; a new run inserts a `quran_import_runs` row. `manifest_hashed()` compares the observed SHA-256 of `manifest_text` against `declared_manifest_hash` and fails the run (`ImportFailed`) on mismatch. `validated()` persists a `validation_reports` row and marks the run `Failed` on any Fatal, before staging is touched. `staged()` writes the edition, surahs, ayahs, tokens, separators, and divisions into `quran_stg_*` plus a `canonical_source` provenance record. `roundtrip_verified()` reads staging back and asserts `reconstruct(tokens, separators) == row.text` and that `text_hash`/`token_order_hash` recompute. `approval_requested()` sets the run state to `"Staged"` and returns. [VERIFIED: crates/quran-corpus/src/import.rs:1304-1379] [VERIFIED: crates/quran-corpus/src/import.rs:449-544] [VERIFIED: crates/quran-corpus/src/import.rs:593-832] [VERIFIED: crates/quran-corpus/src/import.rs:834-923] [VERIFIED: crates/quran-corpus/src/import.rs:1168-1183]

**Activation.** `activate_edition` opens one unit of work, calls `check_approval`, looks up the staged edition, calls `uow.quran().activate_edition(...)`, appends a `SourceActivated` audit event, and commits once. The storage mutator requires run state `Staged`, inserts the edition with `status = 'Active'`, copies surahs/ayahs/tokens/separators/divisions, deprecates the prior `Active` rows, deletes and re-inserts the `quran_active_edition` singleton with `generation = current + 1`, then deletes the import run (cascading staging away). [VERIFIED: crates/application/src/quran.rs:301-343] [VERIFIED: crates/storage-sqlite/src/quran.rs:631-748]

**Rollback.** `rollback_edition` performs the same approval check, then flips the pointer to a prior version: it deprecates the current `Active`, marks the target `Active`, and re-inserts the singleton with `generation + 1`. It returns `StorageError::Conflict` (mapped to `ActivationError::AlreadyActive`, `QAI-QUR-0313`) when the target is already `Active`. [VERIFIED: crates/application/src/quran.rs:429-468] [VERIFIED: crates/storage-sqlite/src/quran.rs:751-817] [VERIFIED: crates/application/src/quran.rs:203]

**Uniqueness/idempotency facts worth planning around.** (1) `edition_id` is the import `run_id` (a bare UUID), so a *new run id is required to re-import the same slug@version*; `staged()` explicitly fails when a retry's provenance `versions_json`/`created_by` differ, with the remedy "use a new run id". (2) `activate_edition` deletes `quran_import_runs` for the run, so post-activation "retry is resume" no longer applies. (3) The activation move list omits `quran_stg_segments` → `quran_segments`, so canonical segments stay empty. [VERIFIED: crates/quran-corpus/src/import.rs:673-686] [VERIFIED: crates/quran-corpus/src/import.rs:402-406] [VERIFIED: crates/storage-sqlite/src/quran.rs:743-747] [VERIFIED: crates/storage-sqlite/src/quran.rs:676-707]

**Proven by tests today:** `import_runs_end_to_end_to_staged`, `crash_matrix_all_thirteen_checkpoints_leave_active_untouched` (loops `ImportCheckpoint::ALL` and asserts `get_active()` is `None` after every prefix), `cancel_cleans_staging_and_marks_cancelled`, `validation_failure_aborts_before_staging_with_report`, `hash_mismatch_aborts_at_the_hash_checkpoint`, `worker_runs_the_import_job_to_staged_with_audit`, `activation_service_requires_a_granted_approval`, `rejected_activations_leave_canonical_state_untouched`, `rollback_service_restores_the_prior_version`; plus storage-level `activation_requires_a_staged_run`, `activation_moves_rows_flips_pointer_and_bumps_generation`, `rollback_restores_a_prior_version`. [VERIFIED: crates/application/tests/quran_import.rs:125-186] [VERIFIED: crates/application/tests/quran_import.rs:441-557] [VERIFIED: crates/application/tests/quran_import.rs:601-819] [VERIFIED: crates/storage-sqlite/tests/quran.rs:350-458]

**Missing:** rollback *rejection* coverage, rollback cache invalidation, segments copy, and a process-`SIGKILL` (rather than `stop_after`) crash test. The ledger's own entry is candid: "13-prefix crash matrix (active untouched)" is automated-green while the "process-kill recording + `job retry` ritual" remains pending. [VERIFIED: docs/03-plan/phases/phase-01-core/acceptance.md:57] [VERIFIED: docs/06-progress/task-done-rollup.md:227-229]

### F-2: Byte-exact lookup and pinned edition reference (criterion 2)

The served canonical text is the stored column verbatim: `arabic_text: ayah_row.text.clone()`, with `text_hash: parse_tagged(&ayah_row.text_hash)`, and edition identity carried as `EditionRef { slug, version: SemVer, script, riwayah }`. The reference string is `format!("quran:{}@{}:{}:{}", slug, version, surah, ayah)` and the deep link `/read/{slug}@{version}/{surah}:{ayah}`. `QuranQuotation::new` rejects an empty reference, empty Arabic text, or empty deep link with `QAI-QUR-0009`. [VERIFIED: crates/application/src/quran_reader.rs:454-482] [VERIFIED: crates/quran-core/src/quotation.rs:106-121]

**Pinning.** `EditionSelector` has three variants — `Active`, `Slug(String)` (resolves only to the pointer's edition), and `Pinned { slug, version }`. The `Pinned` arm looks the edition up by `(slug, version)` and never falls back to the pointer. The *stored* per-ayah hash is the byte-exactness anchor; the edition-level `text_hash` is served in `--json` through `QuranEdition`. [VERIFIED: crates/quran-core/src/enums.rs:117-129] [VERIFIED: crates/application/src/quran_reader.rs:380-407] [VERIFIED: crates/application/src/quran_reader.rs:199-202]

**Cache.** `cache_key = "{edition.id}|{edition.version}|{edition.generation}|{serialize(reference)}|{options_json}"`, capacity 1024. `get_ayah` checks the cache first, then expands, then builds and stores the view. A generation bump (activation or rollback) changes every key, so invalidation is wholesale — matching ADR-0113. [VERIFIED: crates/application/src/quran_reader.rs:352-361] [VERIFIED: crates/application/src/quran_reader.rs:729-751]

### F-3: Corpus integrity — QV code → six-family map (criterion 3)

`validate_edition` runs every content rule with **no fail-fast** and returns a `ValidationReport` with `outcome`, `fatal_count`, `error_count`, `warning_count`, and every `Finding`. Severity semantics: `Fatal` cannot reach `Staged`; `Error` may stage but cannot be approved without an explicit recorded override; `Warning`/`Info` are shown. [VERIFIED: crates/quran-corpus/src/validation.rs:5-17] [VERIFIED: crates/quran-corpus/src/validation.rs:37-49] [VERIFIED: crates/quran-corpus/src/validation.rs:125-147]

| Family | Rules (id → severity → verbatim message stem) | Where enforced |
|---|---|---|
| **Counts** | QV-001 `Fatal` "surah count is {}, manifest expects {}"; QV-002 `Fatal` "surah numbers are not exactly 1..={}"; QV-003 `Fatal` "surah {} declares {} ayahs, found {}"; QV-004 `Fatal` "ayah count is {}, manifest expects {}"; QV-005 `Fatal` "ayah numbers are not exactly 1..={}". [VERIFIED: crates/quran-corpus/src/validation.rs:188-282] | `validate_edition`; QV-004 re-checked against stored statistics in `validate_staged`. [VERIFIED: crates/application/src/quran.rs:584-591] |
| **Addressing** | QV-016 `Error` juz contiguity/coverage; QV-017 `Warning` hizb/rub/manzil monotonicity; QV-018 `Error` "page numbers decrease along the global ayah order"; QV-019 `Info` sajdah count; QV-020 `Error` "surah basmala policy disagrees with the edition-wide policy"; QV-023 `Fatal` "ayahs are not in strictly increasing (surah, ayah) order". [VERIFIED: crates/quran-corpus/src/validation.rs:405-542] | `validate_edition` + `quran_divisions` rows built by `build_divisions`. [VERIFIED: crates/quran-corpus/src/import.rs:1192-1272] |
| **Unicode** | QV-006 `Fatal` "ayah text is empty or whitespace-only"; QV-007 `Fatal` "text is not in the declared {form} form"; QV-008 `Fatal` "forbidden U+{:04X} ({}) at byte {}"; QV-009 `Error` "U+{:04X} at byte {} is outside the expected blocks"; QV-027 `Error` "language `{}` is not Arabic; a translation cannot be imported as a canonical edition". [VERIFIED: crates/quran-corpus/src/validation.rs:174-185] [VERIFIED: crates/quran-corpus/src/validation.rs:291-342] | `unicode::{normalization_form, find_forbidden, first_unexpected}` (ADR-0104). [VERIFIED: crates/quran-corpus/src/unicode.rs:19-53] |
| **Checksums** | QV-013 `Fatal` "declared file hash does not match the observed hash" (`check_file_hash`); QV-014 recomputed `text_hash` from staged rows must equal the import-time value. [VERIFIED: crates/quran-corpus/src/validation.rs:551-565] [VERIFIED: crates/quran-corpus/src/import.rs:908-913] | Importer checkpoints `ManifestHashed` and `RoundtripVerified`; `validate_staged` recomputes `text_hash` from staged rows. [VERIFIED: crates/application/src/quran.rs:611-621] |
| **Round-trip** | QV-010 `Fatal` "supplied token positions are not 1..=k in order"; QV-011 `Fatal` "computed/supplied tokens do not reconstruct the ayah text"; QV-012 `Fatal` "offsets [{}, {}) / [{}, {}) do not slice to the surface"; QV-024 recomputed `token_order_hash` from stored rows must equal the import-time value. [VERIFIED: crates/quran-corpus/src/validation.rs:344-402] [VERIFIED: crates/quran-corpus/src/import.rs:914-921] | `validate_edition` (QV-010/011/012) + importer `RoundtripVerified` (QV-011 recompute + QV-024). [VERIFIED: crates/quran-corpus/src/import.rs:863-893] |
| **Reference comparison** | QV-015 — `Fatal` on any byte difference, missing/duplicate/empty row, incompatible edition policy (script, riwayah, qiraah, numbering, basmala), or a `reference_corpus_id`/`reference_text_hash` pin mismatch; **`Info` skip** when `reference` is `None`. [VERIFIED: crates/quran-corpus/src/import.rs:172-252] [VERIFIED: crates/quran-corpus/src/import.rs:925-1045] | Importer checkpoint `ReferenceCompared`. |

**Rules outside the six families (still required):** QV-021 `Warning` missing revelation place (Layer-B metadata); QV-022 `Info` "ayah {} repeats the text of ayah {} (whitelist legitimate refrains)"; QV-000 `Fatal` unparseable document in the registry bridge. [VERIFIED: crates/quran-corpus/src/validation.rs:499-525] [VERIFIED: crates/quran-corpus/src/validation.rs:596-609]

**QV-015 current behavior, precisely.** With `reference: None` and no `expected.reference_corpus_id`/`reference_text_hash` declared, the import emits exactly one finding: `Finding::new("QV-015", Severity::Info, "edition", "reference-corpus comparison skipped: no reference corpus configured")` — a recorded skip, never a silent pass. But if the manifest declares a reference id/hash and no reference document is supplied, the run hard-fails with "requested reference corpus is unavailable; comparison cannot be skipped". When `Some(reference)` is supplied the comparison is order-independent, byte-exact, and fails closed, and a persisted `Info` finding carries the machine-readable evidence object (`method: "exact-ayah-bytes-v1"`, `comparison_kind: "reference"`, `classification_vocabulary: "ADR-0114-v1"`, `reference_corpus_id`, `reference_version`, `reference_text_hash`, `reference_snapshot_hash`, `outcome`). [VERIFIED: crates/quran-corpus/src/import.rs:176-183] [VERIFIED: crates/quran-corpus/src/import.rs:1003-1011] [VERIFIED: crates/quran-corpus/src/import.rs:982-1002]

**QV-025/026 and QV-028.** QV-025/026 "hold by construction" only for the *importer path*: `quran_ayahs.provenance_id`, `quran_segments.provenance_id`, `quran_divisions.provenance_id`, `translation_passages.provenance_id`, and `word_glosses.provenance_id` are `NOT NULL`, but `quran_surahs.metadata_provenance_id` is **nullable** in the DDL and only the importer always supplies it. QV-028 is documented as "a Phase-2 hook that passes vacuously in v1" and must stay vacuous here — it belongs to the normalization/search phase (legacy Phase 2 / roadmap Phase 3), and this phase must not absorb it. [VERIFIED: migrations/sqlite/0008_quran_structure.up.sql:6-19] [VERIFIED: migrations/sqlite/0008_quran_structure.up.sql:75-86] [VERIFIED: migrations/sqlite/0009_quran_divisions.up.sql:5-17] [VERIFIED: migrations/sqlite/0010_quran_translations.up.sql:28-42] [VERIFIED: crates/quran-corpus/src/validation.rs:15-17] [VERIFIED: docs/03-plan/phases/phase-01-core/technology-stack.md:248-250]

### F-4: Reference corpus — current behavior, candidates, ADR-0114 procedure

**Mechanism vs identity.** The *mechanism* is implemented and well-tested (Tier-1 pinned-artifact exact diff, `reference_text_hash` digest check, fail-closed incompatibility, `reference_snapshot_hash` recorded in provenance `versions_json`). The *identity* is owner-gated (`OD-03`). The missing *mechanism* pieces are: no operator-supplied reference, no named/pinned reference-corpus registry, no per-difference `DifferenceClass` in the QV-015 report, no difference-acknowledgment record with reviewer identity, and no distinct-preparer rule. [VERIFIED: crates/quran-corpus/src/import.rs:142-160] [VERIFIED: crates/quran-corpus/src/import.rs:925-1045] [VERIFIED: specs/019-reference-corpus-config/spec.md:21-60] [VERIFIED: docs/02-architecture/decisions/ADR-0114-reference-corpus-comparison.md:88-99]

**Candidate reference corpora (verified facts only, from the in-repo upstream record).**

| Candidate | Verified facts | Fit for QV-015 | License position |
|---|---|---|---|
| `spqrxi/quranchecksum` @ `954244e6` (branch `main`, 2026-07-04) | SHA-256, **NFC**, strip leading/trailing whitespace, verse granularity, 6,236 verses / 114 surahs; surah rollup = `sha256(concat(verse hashes in ayah order))`; whole-Quran root = `sha256(concat(surah hashes))`; `meta.source = tanzil-uthmani`; `meta.orthographic_standard = Madinah Mushaf (KFGQPC, Medina)`; whole-Quran root at that revision `5b8bb60d84ad9fbbc3abbca112234abf77d9e14ce52076ff397656f7f84ed73c`; repository MIT; **hash-only, no text stored**; upstream explicitly states it is *not* a direct KFGQPC export and does not re-verify Tanzil's manual check. [VERIFIED: docs/02-architecture/upstream-sources.md:244-282] | **Hash-only integrity reference** for a compatible Uthmani/Hafs dataset. It cannot supply byte-exact per-ayah text, so it satisfies a *checksum* comparison, not D-09's "independent reference corpus" byte comparison — this is the exact distinction OD-03 must settle. | Repo MIT; per ADR-0101 "usable for compatible datasets only"; the manifest is hash-only. [VERIFIED: docs/02-architecture/decisions/ADR-0101-initial-quran-dataset.md:173-177] |
| `gaitco/quran-database` @ `4e0cb341` | 6,236 `ayahs`, 134 `editions`; ships `manifest/quran-arabic.manifest.json` whose hashing is "deliberately identical to `spqrxi/quranchecksum`, so the two manifests are directly comparable"; Arabic text is Tanzil Uthmani via alquran.cloud; provenance chain KFGQPC → Tanzil → alquran.cloud; `rukus.json` is `metadata_only`/`pending_license_review`. [VERIFIED: docs/02-architecture/upstream-sources.md:189-229] | Text-bearing, so it *could* serve a byte comparison, but its license is `unknown` for the text family. | Repository MIT does not extend to the text. [VERIFIED: docs/02-architecture/upstream-sources.md:198-199] |
| `fawazahmed0/quran-api` `ara-quranuthmanihaf` (vendored locally) | `fixtures/upstream/quran-api/database/chapterverse/ara-quranuthmanihaf.txt` exists in the in-repo mirror (492 files, 6,236 content lines + a 10-line JSON trailer each); mirror is `pending_license_review`, "local development and metadata work only"; 3 files carry empty verse texts the reader rejects fail-closed. [VERIFIED: fixtures/upstream/README.md:1-40] [VERIFIED: live `ls fixtures/upstream/quran-api/database/chapterverse/`] | Local-only; the same upstream as the importing edition would be, so it is a *version* comparison, not an independent reference. | `redistribution_allowed: unknown` per edition. [VERIFIED: docs/02-architecture/upstream-sources.md:30-33] |

**ADR-0114's procedure (locked direction, Draft status).** Comparison is *typed* (six operand-kind pairs); differences carry one closed classification (`same`, `normalization_only`, `orthographic_difference`, `script_difference`, `riwayah_difference`, `edition_difference`, `tokenization_difference`, `translation_difference`, `unknown_difference`); integrity comparison is not cross-riwayah comparison; QV-015 fails unless a difference is acknowledged with reviewer identity; until a reference is configured QV-015 is a recorded skip, never a silent pass; sign-off requires a named reviewer distinct from the implementer. [VERIFIED: docs/02-architecture/decisions/ADR-0114-reference-corpus-comparison.md:41-99]

**This phase must not:** name a corpus, assert a license, or claim a signer. `OD-03` remains 🔴 and the plan records it as a blocked gate with the exact closing action (owner names corpus + procedure + signer → record in ADR-0114 → flip the row). [VERIFIED: docs/05-followups/decisions-needed.md:63-72]

### F-5: Canonical-write fence inventory (criterion 4)

| Layer | Status | Evidence |
|---|---|---|
| SQL triggers | **Partial.** `trg_ayah_no_update` (`QAI-QUR-0001`), `trg_ayah_no_delete` (`QAI-QUR-0003`), `trg_token_no_update` (`QAI-QUR-0004`), `trg_token_no_delete` (`QAI-QUR-0005`), `trg_edition_immutable_hashes` (`QAI-QUR-0002`, UPDATE of `text_hash, structure_hash, token_order_hash, slug, version` only). **No triggers** on `quran_surahs`, `quran_token_separators`, `quran_segments`, `quran_divisions`, `translation_editions`, `translation_passages`, `word_glosses`; and no DELETE trigger on `quran_editions`. | [VERIFIED: migrations/sqlite/0008_quran_structure.up.sql:88-96] [VERIFIED: migrations/sqlite/0007_quran_editions.up.sql:47-51] [VERIFIED: live `grep -n "CREATE TRIGGER" migrations/sqlite/*.up.sql`] |
| Type-level `CanonicalWriter` + `ApprovalToken` | **Declared, not implemented.** The trait has three methods and no implementor; `ApprovalToken::new` is `pub`; no crate outside `crates/provenance` references either type. | [VERIFIED: crates/provenance/src/lib.rs:139-216] [VERIFIED: live `grep -rn 'CanonicalWriter\|ApprovalToken' crates/`] |
| Approval-row gate (what actually guards activation) | **Implemented.** `check_approval` requires the persisted approval to be `approved` and to name `quran-edition:{slug}@{version}`. The activation service then runs inside one unit of work with the audit append. | [VERIFIED: crates/application/src/quran.rs:238-260] [VERIFIED: crates/application/src/quran.rs:301-343] |
| Repository write-surface restriction | **Implemented as a source-scan test.** The `QuranRepository` trait exposes only `activate_edition`, `rollback_edition`, `set_edition_status`, `set_edition_verification` as canonical mutators; row-level canonical inserts are asserted absent by name. Note this is a text scan of the trait source, not a compile-time seal. | [VERIFIED: crates/storage/src/quran.rs:1902-1940] |
| Importer path audit | **No canonical write.** The importer's only `uow.quran()` calls are staging writes, run-state writes, a validation-report insert, and reads. There is no `ApprovalToken` anywhere in `quran-corpus`. | [VERIFIED: crates/quran-corpus/src/import.rs:452-1103] |
| Adversarial write tests | **Partial.** Five raw statements are proven to abort with their exact codes; no test covers the seven untriggered tables, no test asserts `DELETE FROM quran_editions` behaviour, and no test asserts the importer holds no token. | [VERIFIED: crates/storage-sqlite/tests/quran.rs:233-285] |

**Also true and worth reusing:** there is genuinely **no repository method that inserts a canonical ayah/surah/token row** — canonical rows reach the database only through the `INSERT INTO … SELECT FROM quran_stg_*` statements inside `activate_edition`. That is stronger evidence for D-13(c) than the unused `CanonicalWriter` type, and the plan should present it that way. [VERIFIED: live `grep -rn 'INSERT INTO quran_ayahs\|INSERT INTO quran_surahs\|INSERT INTO quran_tokens' crates/storage-sqlite/src crates/storage/src` → only `INSERT INTO quran_editions` at `crates/storage-sqlite/src/quran.rs:658`]

### F-6: `verify_quotation` answer-path audit (criterion 5)

| Path | Emits canonical text? | Calls `verify_quotation`? | Calls `resolve_stored`/`resolve`? | Note |
|---|---|---|---|---|
| `qai quran get` / `context` / `surah` / `division` (`crates/application/src/quran_cli.rs`) | Yes (prints `arabic_text` + pinned reference) | **No** | No | Reads canonical rows directly — cannot mismatch. [VERIFIED: crates/application/src/quran_cli.rs:137-223] |
| `qai quran resolve` | No text; canonical reference only | No | No | Parse + bounds check. [VERIFIED: crates/application/src/quran_cli.rs:297-325] |
| `GET /api/v1/quran/ayahs/{reference}`, `/context/`, `/surahs/{n}`, `/divisions/…`, `/tokens/…` (`crates/server/src/api.rs`) | Yes (via `AyahView`) | **No** | No | Read paths over canonical rows. [VERIFIED: crates/server/src/api.rs:1295-1304] |
| `GET /api/v1/quran/citations/{id}` | Re-serves a resolved citation (hash + verdict, no text) | **No** | **Yes** (`resolve_stored`) | Stored-hash re-verification, not `verify_quotation`. [VERIFIED: crates/server/src/api.rs:584-598] [VERIFIED: crates/server/src/api.rs:1531-1534] |
| `quran.get_ayah` / `quran.get_context` tools (`crates/application/src/quran_tools.rs`) | Yes (via `AyahView`) | **No** | No | `ReaderCitationSource` provides the resolver but no caller invokes it. [VERIFIED: crates/application/src/quran_tools.rs:85-123] |
| `citations` unit tests / `application/tests/quran_tools.rs` | — | **Yes** (test only) | Yes | The only call sites of `verify_quotation` in the tree. [VERIFIED: crates/application/tests/quran_tools.rs:140-160] |

**What "hard failure on mismatch" requires.** (1) A shared mapping from `QuotationVerdict` to a typed diagnostic + CLI exit code (`Mismatch`, `LocationNotFound`, `EditionNotFound` → failure; `ExactMatch`/whitespace/declared-normalization → success) — this mapping does not exist today. (2) At least one *production* surface that accepts an externally supplied quotation and applies the mapping, so the contract is exercised. (3) An explicit recorded rationale for the read paths that are structurally exempt (they read the source of truth, so there is nothing to compare). (4) `MatchAfterDeclaredNormalization` is currently never produced by `CitationResolver::verdict` — only `ExactMatch`, `MatchAfterWhitespaceNormalization`, and `Mismatch` are reachable, because no normalization-rules parameter exists on the resolver. [VERIFIED: crates/citations/src/lib.rs:301-312] [VERIFIED: docs/07-technical/quran-citation-spec.md:67-73]

### F-7: Translation-layer separation (ADR-0112, D-04)

**Present and green.** `translation_editions` requires `translator TEXT NOT NULL CHECK (length(trim(translator)) > 0)`, `aligned_edition_id TEXT NOT NULL REFERENCES quran_editions(id)`, its own `slug`/`version`/`license_json`/`status`, and `UNIQUE (slug, version)`. `import_translations` rejects an empty translator, requires `slug@version` alignment, verifies every passage names a real ayah of the aligned edition (canonical or staged), and rejects empty/duplicate passages atomically; it writes an attributed `canonical_source` provenance record. The reader re-checks alignment (`translation.aligned_edition_id != edition_row.id` → `TranslationNotFound`) before serving. Type-level separation: `AyahView.canonical: QuranQuotation` and `AttributedTranslation::new` refuses a blank translator or blank edition ref. [VERIFIED: migrations/sqlite/0010_quran_translations.up.sql:9-25] [VERIFIED: crates/application/src/quran.rs:693-858] [VERIFIED: crates/application/src/quran_reader.rs:537-584] [VERIFIED: crates/quran-core/src/view.rs:19-45] [VERIFIED: crates/application/tests/quran_translation.rs:36-211]

**Gaps.** (1) `text_hash` is stored as the empty string — no hash recipe is applied to translation passages. (2) `license_json` is synthesised at import with `"status": "Unknown"` and `redistribution_allowed: false`, ignoring any manifest-declared license. (3) There is no test asserting a translation cannot reach a canonical type, though the types make it structurally impossible — the honest statement is "no constructor path exists", proven by inspection rather than by a negative test. [VERIFIED: crates/application/src/quran.rs:782-810]

### F-8: Primary/default designation (D-07)

**Nothing exists.** Schema, model, and selector all lack a primary/default concept: `quran_editions` has no such column; `QuranEdition` has no such field; `EditionSelector` is `Active` / `Slug` / `Pinned` only; `quran_active_edition` is a bare singleton (`singleton`, `edition_id`, `corpus_generation`, `activated_at`, `activated_by`, `approval_id`). The owner's architectural intent (Uthmani script + Ḥafṣ ʿan ʿĀṣim as primary/default; multi-edition/multi-riwayah/multilingual) is recorded only in prose. [VERIFIED: migrations/sqlite/0007_quran_editions.up.sql:3-44] [VERIFIED: crates/quran-core/src/edition.rs:38-88] [VERIFIED: crates/quran-core/src/enums.rs:117-129] [VERIFIED: docs/05-followups/decisions-needed.md:41-49]

D-07 therefore needs an additive flag that *feeds* the existing pointer without redefining it — "primary default" ≠ "only edition". The natural seam is `quran_editions` (a boolean/status column with an append-only migration) plus a reader selector or an edition field, so the pointer keeps its current meaning while a primary default becomes representable and queryable. [ASSUMED: exact column/flag shape and whether it also needs a selector variant]

### F-9: Synthetic fixture breadth (D-08)

**What `test-edition-min` covers.** 5 surahs / 14 ayahs; all four basmala policies exercised across surahs (`counted_as_first_ayah`, `unnumbered_header`, `absent`, `per_surah`); juz 1–2, hizb 1–2, rub 1–7, manzil 1–2, ruku 1–7, page 1–5 (so page and division boundaries exist); one `sajdah: "recommended"`; 3–6 tokens per ayah; a CSV mirror that must reproduce the JSON ayahs exactly; `synthetic: true` asserted. `test-edition-min-v2.json` is the same shape at version `0.2.0` (diff/rollback flows). `test-translation-min.json` has 3 passages; `test-gloss-min.json` has 3 glosses. 16 adversarial corpora exist and are all format-valid. [VERIFIED: fixtures/quran/test-edition-min/manifest.json:1-40] [VERIFIED: crates/quran-corpus/tests/fixtures.rs:106-130] [VERIFIED: crates/quran-corpus/tests/fixtures.rs:152-182] [VERIFIED: fixtures/quran/test-translation-min.json] [VERIFIED: fixtures/quran/test-gloss-min.json]

**What a "richer" fixture must add** so all six families are exercised more deeply (D-08), keeping the existing fixture and adversarial corpora untouched:

- **More surahs/ayahs** (dozens rather than 14) so `text_hash`/`structure_hash` are non-trivial and `global_ayah_index` spans multiple divisions meaningfully.
- **A basmala *inside* an ayah body** (the An-Naml 27:30 shape) — today basmala exists only as per-surah policy metadata. [VERIFIED: docs/03-plan/phases/phase-01-core/acceptance.md:129-145]
- **Pause marks**: `is_pause_mark` is `false` for every existing token because the fixture text contains no U+06D6…U+06ED signs; a fixture with waqf marks exercises the separator/`is_pause_mark` branches of `tokenize`. [VERIFIED: crates/quran-corpus/src/tokenize.rs:23-25] [VERIFIED: fixtures/quran/test-edition-min/manifest.json]
- **A token separated by a mark rather than a plain space** (ADR-0105's attach rule).
- **A long ayah** (many tokens) and an ayah spanning a page boundary with a non-space separator.
- **A muqattaʿat ayah** (disjointed letters as a standalone token) and a legitimate repeated refrain across two ayahs (to exercise QV-022's `Info` path). [VERIFIED: docs/03-plan/phases/phase-01-core/acceptance.md:129-145]
- **A superscript alef (U+0670) and small high signs** to force grapheme-cluster counting to matter. [VERIFIED: docs/02-architecture/decisions/ADR-0104-unicode-policy.md:37-42]
- **A companion reference edition** for the richer fixture so QV-015 can be exercised end to end through the *operator* path once QC-03 lands — a synthetic reference plus its `reference_text_hash` pin. [VERIFIED: crates/application/tests/quran_import.rs:194-261]

Note the fixture must stay `synthetic: true` and non-canonical; the existing test asserts exactly that for `test-edition-min`, and the same assertion should cover any new fixture (D-05/D-06). [VERIFIED: crates/quran-corpus/tests/fixtures.rs:106-113]

### F-10: Owner gates OD-01 / OD-02 / OD-03

All three are 🔴 **unanswered** and re-confirmed 🔴 on 2026-09-24 by an agent verification pass. No agent may close them. The must-be-recorded gates:

| Gate | Question | Blocks (legacy ids) | What the plan must record | Closing action |
|---|---|---|---|---|
| **OD-01** | Which Arabic edition is the canonical dataset (script, riwayah, numbering, normalization), is redistribution licensed, and Option A (bundle) / B (fixture + user import) / C (defer)? | `P1-X01`, `P1-T01`, `P1-T02`, `P1-T56`, `P1-T58`, `AC-P1-01` | Dataset identity + license evidence + A/B/C choice stay pending. The plan may build the generic shape (`upstream_edition_slug`, license record) but must not name a slug, publisher, release, or hash. | Owner records dataset + license evidence + policy in ADR-0101 → ADR moves to Accepted. [VERIFIED: docs/02-architecture/decisions/ADR-0101-initial-quran-dataset.md:209-211] |
| **OD-02** | Who is the qualified reviewer that compared the text against a recognized printed muṣḥaf, and what sample + method did they sign? | `P1-X02`, `P1-T55`, `AC-P1-01` | `verified_by`/`verification_method` plumbing may be exercised, but any recorded value in a plan artifact must be marked test-only. Existing tests already do this honestly ("OD-02 placeholder reviewer (test only — not a real sign-off)"). | Owner names reviewer + sample + method → `qai quran edition verify` stamps the real edition. [VERIFIED: crates/application/tests/quran_verification.rs:26-27] [VERIFIED: crates/application/src/quran.rs:345-427] |
| **OD-03** | Which independent reference corpus, which comparison procedure, and who signs off? | `P1-X03`, `P1-T03`, `P1-T26◐`, QV-015 | Reference-corpus identity/scope/license stay pending; QV-015 stays a recorded skip (`passed_tier1_only` with an explicit exit-gate flag) until the owner names the corpus. Tier-2 (independent editorial chain, signer ≠ reviewer) is pending owner selection. | Owner names corpus + procedure + signer (ADR-0114 §4) → QV-015 becomes evaluable on real data. [VERIFIED: docs/02-architecture/decisions/ADR-0114-reference-corpus-comparison.md:88-99] [VERIFIED: crates/quran-corpus/src/import.rs:152-158] |

Adjacent owner inputs to name but **not** absorb: `OD-04` (debug-reader font — resolved as a system stack; Amiri is a Phase-2 candidate only), `OD-11` (morphology dataset — Phase 3), `OD-12` (normalization catalog + linguist — Phase 3), `OD-14` (server layering — deferred). [VERIFIED: docs/05-followups/decisions-needed.md:74-174] [VERIFIED: docs/03-plan/phases/phase-01-core/acceptance.md:101]

### F-11: Legacy reconciliation (do not double-count, do not contradict)

**Numbering drift is real and must be stated in the plan.** `docs/03-plan/phases/phase-01-core/` is the legacy canonical core (this roadmap's Phase 2). `docs/03-plan/phases/phase-02-rag/` is search/normalization/morphology (this roadmap's Phase 3). `docs/03-plan/current-plan.md` names "Phase 2 — Quran search/linguistics" as the active legacy phase and explicitly warns "Phase numbering is legacy … Do not renumber directories by hand." [VERIFIED: docs/03-plan/current-plan.md:1-14] [VERIFIED: .planning/phases/02-canonical-quran-core/02-CONTEXT.md:20]

**Legacy status that must not be re-read as our completion:**

| Legacy artifact | Claimed status | How this phase reads it |
|---|---|---|
| `phase-01-core/acceptance.md` §1 | 0/21 criteria verified; 19 rows `◐`, the rest `☐`; `AC-P1-01` is the hard gate and is `☐` | **Evidence source, not our criteria.** Our five roadmap criteria are distinct and must be evidenced in their own right. [VERIFIED: docs/03-plan/phases/phase-01-core/acceptance.md:10] |
| `phase-01-core/done.md` §3 | "None verified yet" — every AC row `—` or "partial — automated" | Confirms the "implementation landed, closure never recorded" posture. [VERIFIED: docs/03-plan/phases/phase-01-core/done.md:796-826] |
| `phase-01-core/done.md` §7 | Six unowned deferrals OWN-01…OWN-06, all `_unassigned_` | OWN-01 (ADR-0101), OWN-02 (ADR-0114), OWN-03 (estimate), OWN-04 (axum — ratified later), OWN-05 (Phase-0 exit), OWN-06 (server layering, deferred). [VERIFIED: docs/03-plan/phases/phase-01-core/done.md:978-987] |
| `docs/06-progress/task-done-rollup.md` | Records automated-green ACs "rituals pending", the 13-prefix crash matrix, `doctor --quran` 19 checks + `--deep`, and three fixed end-to-end defects | Use as evidence pointers; do not restate as phase completion. [VERIFIED: docs/06-progress/task-done-rollup.md:200-296] |
| `phase-02-rag/done.md` | 58/114 tasks, 0/50 acceptance criteria, 0/14 ADRs — "In Progress" | Out of scope. Its QV-028 dependency (index build must not alter canonical text) belongs to the normalization/search phase, not here. [VERIFIED: docs/03-plan/phases/phase-02-rag/done.md:1-8] [VERIFIED: docs/03-plan/phases/phase-01-core/technology-stack.md:248-250] |
| `specs/005`, `006`, `012`, `015`, `016`, `019` | Draft feature specs (016 typed comparison, 015 integrity manifests, 019 reference-corpus config) | Authoritative *design* evidence. Note `specs/016`'s typed comparison is implemented in `differ.rs` but not wired (QC-04); `specs/015`'s quranchecksum-compatible manifest generation does not exist as a command; `specs/019`'s registry/acknowledgment model does not exist. [VERIFIED: specs/016-typed-corpus-comparison/spec.md:1-16] [VERIFIED: specs/019-reference-corpus-config/spec.md:1-60] |

**Contradictions to resolve rather than inherit:** (a) the legacy coverage floors in `acceptance.md` §4 ("ratified 2026-09-22 (OD-07)") are not in `xtask/src/coverage.rs` (QC-12); (b) `acceptance.md` says all 16 adversarial fixtures assert one specific rule id while two fixtures have documented alternatives in the ledger (QC-13); (c) `docs/plans/handoff-p1-to-p2.md` asserts "`ApprovalToken` exists only from a persisted human `ApprovalRecord`" — the code does not enforce this: `ApprovalToken::new(approval_id, subject_urn, approved_by)` is a public constructor with no approval lookup, and no crate calls it. The handoff document is the **most quotable instance of the D-13(b) overclaim**, so the plan must not cite it as proof of layer (b). Note also the handoff predates the current migration count (it reports 14; the tree has 19 `*.up.sql`). [VERIFIED: docs/03-plan/phases/phase-01-core/acceptance.md:256-276] [VERIFIED: docs/03-plan/phases/phase-01-core/acceptance.md:305-322] [VERIFIED: docs/plans/handoff-p1-to-p2.md:1-60] [VERIFIED: crates/provenance/src/lib.rs:155-165] [VERIFIED: live `ls migrations/sqlite/*.up.sql | wc -l` → 19]

## Code Examples

### Byte-exact canonical view construction (the reference pattern to preserve)

```rust
// Source: crates/application/src/quran_reader.rs:454-482
let reference = format!(
    "quran:{}@{}:{}:{}",
    edition.slug, edition.version, location.surah, location.ayah
);
let canonical = QuranQuotation::new(QuotationParts {
    reference: reference.clone(),
    surah_number: location.surah,
    surah_name_arabic: surah.name_arabic.clone(),
    surah_name_translit: surah.name_transliteration.clone(),
    ayah_range: (location.ayah, location.ayah),
    arabic_text: ayah_row.text.clone(),
    text_hash: parse_tagged(&ayah_row.text_hash)?,
    edition: EditionRef {
        slug: edition.slug.clone(),
        version: edition_row.version.parse::<SemVer>()?,
        script: parse_script(&edition_row.script),
        riwayah: edition_row.riwayah.clone(),
    },
    translation: None,
    deep_link: format!("/read/{}@{}/{}:{}", edition.slug, edition.version, location.surah, location.ayah),
    page: ayah_row.page.map(|value| value as u32),
    juz: ayah_row.juz.map(|value| value as u16),
})?;
```

The stored per-ayah `text_hash` is what makes the served text byte-exactly verifiable; a plan must not introduce a second construction path that omits it. [VERIFIED: crates/application/src/quran_reader.rs:454-482]

### The approval gate that actually guards activation

```rust
// Source: crates/application/src/quran.rs:238-260
async fn check_approval(
    uow: &mut dyn storage::UnitOfWork,
    approval_id: &str,
    expected_subject: &str,
) -> Result<(), ActivationError> {
    let approval = uow.sources().get_approval(approval_id).await
        .map_err(ActivationError::storage)?
        .ok_or_else(|| ActivationError::ApprovalMissing { id: approval_id.to_string() })?;
    if approval.decision.as_deref() != Some("approved") {
        return Err(ActivationError::ApprovalNotGranted { id: approval_id.to_string() });
    }
    if approval.subject_urn != expected_subject {
        return Err(ActivationError::ApprovalSubjectMismatch {
            id: approval_id.to_string(),
            actual: approval.subject_urn,
            expected: expected_subject.to_string(),
        });
    }
    Ok(())
}
```

This is the layer to prove for the "non-approved writes" criterion — `expected_subject` is always `quran-edition:{slug}@{version}` (D-16). [VERIFIED: crates/application/src/quran.rs:238-260] [VERIFIED: crates/application/src/quran.rs:40-42]

### The adversarial trigger assertion pattern to extend

```rust
// Source: crates/storage-sqlite/tests/quran.rs:248-258
for (sql, code) in [
    ("UPDATE quran_ayahs SET text = 'x' WHERE edition_id = 'ed-1'", "QAI-QUR-0001"),
    ("DELETE FROM quran_ayahs WHERE edition_id = 'ed-1'", "QAI-QUR-0003"),
    ("UPDATE quran_tokens SET surface = 'x' WHERE edition_id = 'ed-1'", "QAI-QUR-0004"),
    ("DELETE FROM quran_tokens WHERE edition_id = 'ed-1'", "QAI-QUR-0005"),
    ("UPDATE quran_editions SET text_hash = 'x' WHERE id = 'ed-1'", "QAI-QUR-0002"),
] {
    let err = sqlx::query(sql).execute(&pool).await.unwrap_err();
    let message = err.to_string();
    assert!(message.contains(code), "{sql} must abort with {code}, got: {message}");
}
```

QC-05 extends this table with one row per newly triggered table (and its new code), rather than adding a separate test. [VERIFIED: crates/storage-sqlite/tests/quran.rs:233-285]

### The QV-015 evidence object the operator surface should surface

```json
{
  "method": "exact-ayah-bytes-v1",
  "comparison_kind": "reference",
  "classification_vocabulary": "ADR-0114-v1",
  "normalization_applied": [],
  "reference_corpus_id": "<slug>",
  "reference_version": "<version>",
  "reference_text_hash": "sha256:<hex>",
  "reference_snapshot_hash": "sha256:<hex>",
  "outcome": "pass"
}
```

This object already exists inside the persisted `ValidationReport` findings JSON; D-11's operator surface should render it rather than invent a new format. [VERIFIED: crates/quran-corpus/src/import.rs:982-1002]

### The "skipped, never silently passed" shape to preserve for the reference family

```rust
// Source: crates/quran-corpus/src/import.rs:176-183
let Some(reference) = reference else {
    return vec![Finding::new(
        "QV-015",
        Severity::Info,
        "edition",
        "reference-corpus comparison skipped: no reference corpus configured",
    )];
};
```

Any D-10 compliance report must classify this as **skipped**, not passed. [VERIFIED: crates/quran-corpus/src/import.rs:172-183]

## State of the Art

| Current state in this repo | Phase 2 target | Impact |
|---|---|---|
| Canonical core implemented and largely tested; legacy ledger records 0/21 criteria formally verified. [VERIFIED: docs/03-plan/phases/phase-01-core/done.md:796-826] | Five roadmap criteria each mapped to code + at least one repeatable check + a recorded gap/blocked gate. [VERIFIED: .planning/phases/02-canonical-quran-core/02-CONTEXT.md:19] | Turns "substantially landed" into auditable completion. |
| `ImportOptions.reference` is test-only; `run_import_job` hardcodes `ImportOptions::default()`. [VERIFIED: crates/application/src/quran.rs:83] | Operator-supplied reference behind the OD-03 gate; recorded skip otherwise. | Reference comparison becomes a real family, not a test-only one. |
| Trigger coverage on 3 canonical tables; `CanonicalWriter`/`ApprovalToken` unused. [VERIFIED: migrations/sqlite/0008_quran_structure.up.sql:88-96] | Full trigger set + an explicit decision on the type-level gate. | Removes a documented-but-unenforced safety claim. |
| `verify_quotation` implemented, zero production callers. [VERIFIED: crates/application/tests/quran_tools.rs:140-160] | Verifying surface + verdict→hard-failure mapping. | Makes ADR-0111's "mismatch is a hard failure" true on a real path. |
| `translation_editions.text_hash` written empty; declared license ignored. [VERIFIED: crates/application/src/quran.rs:805] | Real translation hash (new additive recipe) + declared license. | Completes REQ-data-separation-layers on the translation side. |
| No primary/default flag. [VERIFIED: crates/quran-core/src/enums.rs:117-129] | Explicit primary/default (Uthmani + Ḥafṣ ʿan ʿĀṣim) feeding the pointer. | Represents the owner's stated default without hard-coding one edition. |
| `qai doctor --quran` reference check is hardcoded warn; no `qai quran verify`; no committed integrity artifact. [VERIFIED: crates/application/src/quran_doctor.rs:438-442] | Operator/CI surface + committed artifact (D-11). | Integrity evidence becomes reproducible offline. |

**Deprecated/outdated patterns for this phase:**

- **Do not** cite `.agent/coding-rules.md`'s "canonical rows are written only through a `CanonicalWriter`" or `docs/plans/handoff-p1-to-p2.md`'s "`ApprovalToken` exists only from a persisted human `ApprovalRecord`" as an enforced invariant; neither is implemented. [VERIFIED: .agent/coding-rules.md:10-11] [VERIFIED: docs/plans/handoff-p1-to-p2.md:56-60]
- **Do not** treat `qai quran hashes` as an integrity check; it prints stored values by design. [VERIFIED: crates/application/src/quran_cli.rs:1189-1192]
- **Do not** treat a green import as "reference comparison passed" while `ImportOptions.reference` is `None`. [VERIFIED: crates/quran-corpus/src/import.rs:176-183]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|---|---|---|
| A1 | The recommended four-work-package decomposition (A: fixture/evidence, B: fence, C: quotation, D: translation+primary flag) is the right slice of the gap register. | Summary, Evidence Matrix | Planning effort lands on the wrong seams; the register rows are individually grounded, so a different grouping is low-risk but a wrong *ordering* could block later rows. |
| A2 | Proposed task ids `QC-01`…`QC-13` are placeholders, not repository task ids; AGENTS.md requires real TASK-IDs. | Gap Register | Planner must translate each row into a TASK-ID under `docs/04-tasks/`. |
| A3 | A new append-only migration can add the QC-05 triggers + the QC-11 primary/default flag without touching `0007`–`0012`. | Gap Register | If the migration runner's contiguity rule or checksum verification disallows it, halt for an explicit plan revision (mirroring Phase 1's RA-03). |
| A4 | The translation `text_hash` recipe can be additive and domain-separated (e.g. a new `qai-translation-hash-v*` name) without touching `qai-text-hash-v1`. | Gap Register, F-7 | If a reviewer reads D-12 as forbidding any new recipe, the translation hash needs an owner decision; D-12 explicitly permits additive domain-separated recipes. [VERIFIED: .planning/phases/02-canonical-quran-core/02-CONTEXT.md:34] |
| A5 | Wiring the `CanonicalWriter`/`ApprovalToken` gate is *optional* — retiring it with a recorded rationale is an acceptable resolution of QC-06, since `activate_edition` + triggers + the trait-surface scan already enforce the behavior. | Gap Register, F-5 | If the owner reads ADR-0000's "canonical immutability is type-level" as binding, retirement is not acceptable and the gate must be wired. This is exactly the kind of decision the plan must surface, not assume. |
| A6 | D-15's "every current answer path" is satisfied by enforcing on paths that accept an *externally supplied* quotation, with read paths recorded as structurally exempt. | F-6, QC-07 | If the reviewer reads D-15 literally as "wrap every read command", the scope grows with no verification value; the plan should state the interpretation explicitly for confirmation. |
| A7 | `spqrxi/quranchecksum` is a **hash-only** reference (no text), so it cannot satisfy a byte-exact QV-015 comparison on its own. | F-4 | If the owner intends it as the reference corpus, QV-015's semantics must be restated as a checksum comparison — an owner/ADR decision, not an engineering one. |
| A8 | No registry publish-date or version-compatibility probe was performed this session; no dependency upgrade is proposed. | Standard Stack | A separately-approved upgrade would need its own compatibility and supply-chain review. |
| A9 | ASVS is used only as review vocabulary; exact ASVS control identifiers are not treated as locked requirements. | Security Domain | Security acceptance must still be checked against the locked ADRs and the implementation's tests. |

## Open Questions

1. **Does D-15 require a new operator surface, or only a shared hard-failure mapping?**
   - What we know: `verify_quotation` is implemented and unit-tested; no production caller exists; the read paths read canonical rows directly and cannot mismatch.
   - What's unclear: whether the owner wants a user-visible `qai quran verify-quotation`-style verb now, or only the guarantee that no future path may ignore the verdict.
   - Recommendation: implement the shared mapping **and** one minimal verifying surface so the contract is exercised; record the read-path exemption explicitly for the owner to ratify. [ASSUMED]

2. **Wire or retire the `CanonicalWriter`/`ApprovalToken` layer?**
   - What we know: `.agent/coding-rules.md` states it as an invariant; ADR-0000 describes type-level immutability; the trait has no implementor and the token is publicly constructible.
   - What's unclear: whether the owner treats the type gate as a required enforcement layer or as a reserved design for a later phase.
   - Recommendation: present both options with cost/benefit in the plan and take an explicit decision; do not silently leave it unused. [ASSUMED]

3. **What exactly is the independent reference corpus (D-09)?**
   - What we know: the mechanism exists (Tier-1 exact diff + hash pin); a hash-only candidate (`spqrxi/quranchecksum`) and two text-bearing-but-license-unknown candidates exist; ADR-0114 §4 defines the procedure.
   - What's unclear: identity, scope, license, and signer.
   - Recommendation: record OD-03 as a blocked gate; implement the *mechanism* (operator-supplied reference + typed comparison wiring) and leave identity to the owner. [VERIFIED: docs/05-followups/decisions-needed.md:63-72]

4. **Should QV-015 v1 stay byte-only, or adopt the typed difference classification now?**
   - What we know: `diff_ayahs_typed` / `DifferenceClass` exist and are unit-tested; `compared()` only embeds the vocabulary name as a string.
   - What's unclear: whether classification belongs to QV-015's pass/fail or to a separate typed report (ADR-0114 §4 says a readings comparison "runs as a separate, typed report and never feeds QV-015's pass/fail").
   - Recommendation: follow ADR-0114 §4 — keep QV-015 byte-only for *matching* and record the classification vocabulary as report metadata; do not let classification soften a byte mismatch. [VERIFIED: docs/02-architecture/decisions/ADR-0114-reference-corpus-comparison.md:88-96]

5. **Is the `quran_segments` table intended to stay empty?**
   - What we know: canonical `quran_segments` and staging `quran_stg_segments` both exist; activation never copies segments; nothing writes either.
   - What's unclear: whether segments are reserved for the morphology phase or are an unfinished canonical-core artifact.
   - Recommendation: ask explicitly; if reserved, record it (do not add a trigger that a future phase must then amend). [VERIFIED: migrations/sqlite/0008_quran_structure.up.sql:75-86] [VERIFIED: crates/storage-sqlite/src/quran.rs:676-707]

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|---|---|---|---|---|
| Rust compiler | All workspace compilation | ✓ | `1.97.1` (pinned) | None; the pinned toolchain is installed. [VERIFIED: rust-toolchain.toml:1-5] [VERIFIED: live `rustc --version`] |
| Cargo | Build/test/lockfile | ✓ | `1.97.1` | None. [VERIFIED: live `cargo --version`] |
| SQLite CLI | Fixture/DB inspection | ✓ | `3.51.0` | SQLx bundled SQLite is the runtime; the CLI is only for inspection. [VERIFIED: live `sqlite3 --version`] |
| SQLx (bundled SQLite) | Storage, migrations, tests | ✓ | workspace dep | None. [VERIFIED: .planning/codebase/STACK.md:40-46] |
| `cargo-llvm-cov` | Coverage gate (QC-12) | ✓ | installed at `/Users/ali/.cargo/bin/cargo-llvm-cov` | None needed unless QC-12 is taken in scope. [VERIFIED: live probe] |
| `cargo-deny` | Supply-chain CI gate | **✗ locally** | — | CI enforces it; `xtask ci` warns and skips when absent. [VERIFIED: xtask/src/ci.rs:29-48] [VERIFIED: .github/workflows/ci.yml:14-25] |
| `python3` | Ad-hoc fixture analysis only | ✓ | `3.12.11` | Not a build dependency. [VERIFIED: live `python3 --version`] |
| Docker daemon | Not used by this phase | ✗ | — | N/A — out of scope (Phase 12). |
| Network / external services | QV-015 reference retrieval (D-09) | not used | — | The project is offline-first; `fixtures/upstream/` is a local mirror. Do **not** add a network read to the deterministic path. [VERIFIED: specs/019-reference-corpus-config/spec.md:60] |

**Missing dependencies with no fallback:** none for the Phase 2 code/test scope.
**Missing dependencies with fallback:** `cargo-deny` is locally missing but CI is the enforcing fallback.

## Validation Architecture

`.planning/config.json` is **absent**, so Nyquist validation and security enforcement are both treated as **enabled**. [VERIFIED: live file-not-found probe, 2026-09-24]

### Test Framework

| Property | Value |
|---|---|
| Framework | Rust built-in `#[test]` / `#[tokio::test]`, `proptest` for properties, `trycmd` for the real CLI, `tempfile` for isolated real-SQLite state. |
| Config file | None dedicated; the Cargo workspace + existing test targets are the configuration. [VERIFIED: .planning/codebase/TESTING.md:9-19] |
| Quick run command (canonical area) | `cargo test -p quran-core -p quran-corpus -p citations -p provenance` |
| Quick run command (integration) | `cargo test -p storage-sqlite --test quran && cargo test -p application --test quran_import --test quran_reader --test quran_translation --test quran_verification --test quran_doctor --test quran_tools` |
| Quick run command (surfaces) | `cargo test -p cli --test quran --test catalog --test doctor_json && cargo test -p server --test api` |
| Full suite command | `cargo test --workspace` |
| Formatting/lint gate | `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings` |
| Architecture/migrations | `cargo run -q -p xtask -- arch-check && cargo run -q -p xtask -- migrate-check` |
| Full pre-merge gate | `cargo run -p xtask -- ci` (9 steps incl. gen-schema diff and a real-binary `doctor --json` schema validation). [VERIFIED: xtask/src/ci.rs:15-104] |

### Success Criterion → Test Map

| Criterion | Behavior under test | Test type | Automated command (verified to exist as a target) | File exists? |
|---|---|---|---|---|
| SC-1 import → validation → activation → rollback | 13 checkpoints; crash matrix leaves active unchanged; cancellation cleans staging; rejected activations are inert; rollback restores a prior version; activation moves rows/bumps generation | integration (real SQLite) + CLI snapshot | `cargo test -p application --test quran_import`; `cargo test -p storage-sqlite --test quran`; `cargo test -p cli --test quran` | ✅ |
| SC-2 byte-exact lookup + pinned edition | `QuranQuotation` carries text + stored hash + `slug@version`; ranges/surah/divisions expand; context respects boundaries/caps; no stale text after activation; lookup performance smoke; 331-case grammar | integration + property + golden + CLI snapshot | `cargo test -p application --test quran_reader`; `cargo test -p quran-core --test reference_grammar`; `cargo test -p cli --test quran` | ✅ |
| SC-3 six integrity families | 28 rules run without fail-fast; 16 adversarial fixtures rejected by specific rule id; reference comparison exact/order-independent/fail-closed; recomputed hashes match import-time | unit + integration + golden + snapshot | `cargo test -p quran-corpus`; `cargo test -p application --test quran_doctor`; `cargo test -p cli --test doctor_json` | ✅ (reference family: mechanism ✅, operator path ❌ → QC-03/QC-04) |
| SC-4 canonical-write fence | Raw `UPDATE`/`DELETE` abort with `QAI-QUR-000x`; trigger set present; gated-mutator surface restricted; activation requires a staged run | integration + source-scan unit | `cargo test -p storage-sqlite --test quran`; `cargo test -p storage --lib quran`; `cargo test -p application --test quran_import` | ✅ for 5 statements; ❌ for the 7 untriggered tables → QC-05 |
| SC-5 `verify_quotation` hard failure | Verdicts resolve/fail correctly; mismatch detected; stored citations re-verify | unit + integration | `cargo test -p citations`; `cargo test -p application --test quran_tools`; `cargo test -p server --test api` | ✅ library; ❌ production path → QC-07 |
| REQ-data-separation-layers (translations) | Attribution required; alignment validated; empty/duplicate passages rejected; atomic on rejection; reader re-checks alignment | integration | `cargo test -p application --test quran_translation` | ✅; `text_hash` gap → QC-08 |
| Purity fence (no LLM/vector on the canonical path) | Forbidden edges from `quran-core`/`quran-corpus` to `llm`/`embeddings`/`retrieval` | build gate | `cargo run -q -p xtask -- arch-check` | ✅ |

### What would make each criterion fail (verification tripwires)

- **SC-1:** a prefix run that leaves `get_active()` non-`None`; a cancelled run that leaves `quran_stg_*` rows or a non-`Cancelled` state; an activation that succeeds with a denied/mismatched approval; a rollback whose generation does not increase. Current suite catches all four for *activation*; rollback rejection is the uncovered branch. [VERIFIED: crates/application/tests/quran_import.rs:441-484] [VERIFIED: crates/application/tests/quran_import.rs:688-767]
- **SC-2:** `arabic_text` differing from the stored column; a reference lacking `@version`; a `Pinned` selector silently falling back to the active edition; a cache serving text after a generation change. [VERIFIED: crates/application/src/quran_reader.rs:352-361] [VERIFIED: crates/application/src/quran_reader.rs:380-407]
- **SC-3:** any adversarial fixture passing; a rule firing with the wrong severity; a recomputed hash differing from the stored one; a reference mismatch reported as a pass. `qv-015`'s skip must never appear as "pass". [VERIFIED: crates/quran-corpus/tests/adversarial.rs:13-29]
- **SC-4:** a raw statement on any canonical table succeeding (currently true for 7 tables); `migrate-check` failing because a migration was edited. [VERIFIED: crates/storage-sqlite/tests/quran.rs:233-285]
- **SC-5:** a `Mismatch` verdict returned as success; a missing location returned as a synthesized result. Note `MatchAfterDeclaredNormalization` is unreachable in v1 (no normalization-rules parameter). [VERIFIED: crates/citations/src/lib.rs:301-312]

### Sampling Rate

- **Per task commit:** the focused target for the touched seam, plus `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets -- -D warnings` when Rust files change. [VERIFIED: .agent/definition-of-done.md:1-12]
- **Per wave merge:** the canonical quick suite above + `cargo run -q -p xtask -- arch-check && cargo run -q -p xtask -- migrate-check`.
- **Phase gate:** `cargo test --workspace` and `cargo run -p xtask -- ci` green, **plus** the five success-criterion commands recorded with their output as the phase's evidence-of-record (D-11).

### Wave 0 Gaps

- [ ] `crates/application/tests/quran_import.rs` — add rollback rejection coverage (missing/denied/mismatched approval → pointer unchanged) and a rollback cache-invalidation case. [QC-01]
- [ ] `crates/storage-sqlite/tests/quran.rs` — extend `canonical_triggers_abort_raw_writes_with_codes` to one row per canonical table after QC-05's migration, and extend `canonical_tables_declare_the_trigger_set` to enumerate the full expected trigger set.
- [ ] `crates/cli/tests/quran/` — a new trycmd case for the integrity/verify surface (D-11, QC-03) and the quotation hard-failure exit code (QC-07).
- [ ] `fixtures/quran/` — the richer synthetic edition from D-08 (plus its companion synthetic reference and pinned `reference_text_hash`) and a golden hash file for the new fixture. [VERIFIED: fixtures/quran/golden/ayah_texts.jsonl (14 rows today)]
- [ ] A committed corpus-integrity report artifact (D-11) — no comparable artifact exists in the tree today. [VERIFIED: live repo-wide search for an integrity report artifact]
- [ ] Framework install: **none** — Rust, Cargo, SQLx, trycmd, tempfile, proptest, and all listed test targets already exist. [VERIFIED: .planning/codebase/TESTING.md:9-30]

## Security Domain

Security enforcement is treated as **enabled** because `.planning/config.json` is absent. ASVS is used as a review vocabulary, not as a replacement for the locked ADRs. [CITED: https://github.com/OWASP/ASVS] [ASSUMED: exact ASVS 5.0 control identifiers are not locked here]

### Applicable ASVS Categories

| ASVS Category | Applies | Standard control / evidence |
|---|---|---|
| V2 Authentication | Limited — no remote auth in this phase | The local operator path uses a single seeded principal (`ensure_principal`); remote auth remains a later server concern. Do not add bind/auth changes here. [VERIFIED: crates/application/src/quran.rs:497-514] |
| V3 Session Management | No | No session state is introduced. |
| V4 Access Control | **Yes** | Human approval required for every canonical publication: `check_approval` requires a persisted `approved` row naming the exact edition URN; deny-by-default side effects; importer holds no activation capability. [VERIFIED: crates/application/src/quran.rs:238-260] [VERIFIED: .agent/coding-rules.md:10-22] |
| V5 Input Validation | **Yes** | Manifest parsing + schema (`docs/schemas/quran-edition-source.v1.schema.json`), QV-001…QV-028 with Unicode/forbidden-point rules, reference grammar that never panics on arbitrary input, bound SQL parameters throughout, typed `QAI-*` diagnostics. [VERIFIED: crates/quran-corpus/src/validation.rs:165-545] [VERIFIED: crates/quran-core/tests/reference_grammar.rs:151-158] |
| V6 Cryptography | **Yes** | Reuse the existing SHA-256 `ContentHash` contract and the frozen ADR-0108 recipes; never hand-roll a hash. Algorithm agility is carried by the `ContentHash.algorithm` tag. [VERIFIED: docs/architecture/hashing-spec.md:3-14] [VERIFIED: crates/quran-corpus/src/hashing.rs:8-27] |

### Known Threat Patterns for the Rust / SQLite / CLI stack

| Pattern | STRIDE | Standard mitigation |
|---|---|---|
| Silent canonical corruption via a slight dataset error (NFD vs NFC, BOM, bidi override, homoglyph) | Tampering | QV-007/008/009 plus 16 adversarial fixtures asserting **specific** rule ids. [VERIFIED: crates/quran-corpus/src/unicode.rs:38-53] [VERIFIED: crates/quran-corpus/tests/adversarial.rs:39-46] |
| Unapproved write to canonical text or metadata | Tampering / elevation of privilege | Approval-gated activation in one transaction + insert-only triggers; the plan must close the 7 untriggered tables (QC-05). [VERIFIED: migrations/sqlite/0008_quran_structure.up.sql:88-96] |
| Half-corpus exposure on crash | Tampering / DoS | Staging is physically separate; the pointer flips only in the activation transaction; the 13-prefix crash matrix proves the active pointer is unchanged. [VERIFIED: crates/application/tests/quran_import.rs:441-484] |
| Presenting a translation as the Quran | Spoofing / repudiation | `translation_editions.translator NOT NULL` + non-empty CHECK; `AyahView.canonical` is `QuranQuotation`; `AttributedTranslation` has no blank-translator constructor; QV-027. [VERIFIED: migrations/sqlite/0010_quran_translations.up.sql:9-25] [VERIFIED: crates/quran-core/src/view.rs:29-45] |
| Quoting a verse that does not match the corpus | Repudiation | `verify_quotation` verdicts with mismatch as a hard failure — currently unenforced on any production path (QC-07). [VERIFIED: docs/07-technical/quran-citation-spec.md:53-60] |
| Cross-edition confusion (one riwayah's checksum certifying another) | Tampering / spoofing | `edition_id` is part of every canonical primary key; reference comparison fails closed on incompatible script/riwayah/qiraah/numbering/basmala; one manifest per edition. [VERIFIED: crates/quran-corpus/src/import.rs:185-211] [VERIFIED: docs/02-architecture/upstream-sources.md:286-294] |
| Untrusted import bytes reaching a parser | DoS / injection | Adapters convert to a typed intermediate format first; the reference parser is fuzz-tested to never panic and always return a coded error. [VERIFIED: crates/quran-core/tests/reference_grammar.rs:151-158] |
| JSON injection through fixture/file input | Tampering | All DB access uses SQLx bound parameters; the `citations`/`validation_reports` writers bind every value. [VERIFIED: crates/application/src/quran_tools.rs:184-204] |

## Sources

### Primary (HIGH confidence — opened this session)

- `crates/quran-corpus/src/import.rs:1-1400` — 13 checkpoints, `compare_reference`, `run_import`, staging writes, round-trip verification.
- `crates/quran-corpus/src/validation.rs:1-688` — QV-001…QV-028, severities, `check_file_hash`, `intermediate_hash`, registry bridge.
- `crates/quran-corpus/src/hashing.rs:1-60`, `tokenize.rs:1-58`, `unicode.rs:1-70`, `differ.rs:16-310`, `format.rs:1-150`, `lib.rs:1-65`.
- `crates/quran-corpus/tests/adversarial.rs:1-135`, `tests/fixtures.rs:1-182`.
- `crates/quran-core/src/quotation.rs:1-275`, `view.rs:1-205`, `reference/mod.rs:1-165`, `edition.rs:38-88`, `enums.rs:117-129`.
- `crates/quran-core/tests/reference_grammar.rs:1-166`.
- `crates/citations/src/lib.rs:1-427` — verdicts, resolver, `verify_quotation`, `resolve_stored`.
- `crates/provenance/src/lib.rs:137-368` — `ApprovalToken`, `CanonicalWriter`, vacuous token test.
- `crates/application/src/quran.rs:1-1169` — import handler, activation/rollback, verification stamping, translation/gloss import.
- `crates/application/src/quran_reader.rs:1-994` — edition resolution, cache key, `build_view`, `expand`.
- `crates/application/src/quran_cli.rs:128-760,1161-1212` — `cmd_get`, `cmd_import`, `cmd_validate`, `cmd_hashes`.
- `crates/application/src/quran_doctor.rs:30-459` — 19 check ids incl. the hardcoded `quran.reference_corpus` warn.
- `crates/application/src/quran_tools.rs:1-237` — tool/citation backends, `persist_citation`.
- `crates/application/tests/quran_import.rs:1-819`, `quran_reader.rs:256-323`, `quran_translation.rs:36-211`, `quran_verification.rs:1-267`, `quran_doctor.rs:78-113`, `quran_tools.rs:140-160`.
- `crates/storage/src/quran.rs:1895-1958` — gated-mutator source scan + fail-closed test.
- `crates/storage-sqlite/src/quran.rs:625-844` — canonical activation/rollback SQL.
- `crates/storage-sqlite/tests/quran.rs:233-329` — trigger and verification-stamp tests.
- `migrations/sqlite/0007_quran_editions.up.sql`, `0008_quran_structure.up.sql`, `0009_quran_divisions.up.sql`, `0010_quran_translations.up.sql`, `0011_quran_staging.up.sql`, `0012_quran_validation.up.sql` — DDL, CHECK constraints, triggers.
- `xtask/src/main.rs:28-63`, `xtask/src/ci.rs:15-104`, `xtask/src/coverage.rs:1-36` — gates and coverage floors.
- `fixtures/quran/test-edition-min/manifest.json`, `test-edition-min-v2.json`, `test-translation-min.json`, `test-gloss-min.json`, `fixtures/quran/golden/{ayah_texts,references}.jsonl`, `fixtures/upstream/README.md`.
- `docs/02-architecture/decisions/ADR-0101` (Draft), `ADR-0104`, `ADR-0106`, `ADR-0107`, `ADR-0111`, `ADR-0112`, `ADR-0114` (Draft) — locked and owner-gated decisions.
- `docs/02-architecture/upstream-sources.md:1-319` — verified upstream facts for the three candidate sources.
- `docs/05-followups/decisions-needed.md:1-184` — OD-01…OD-14 statuses.
- `docs/07-technical/quran-citation-spec.md`, `docs/architecture/hashing-spec.md`.
- `specs/016-typed-corpus-comparison/spec.md`, `specs/019-reference-corpus-config/spec.md`.
- `docs/03-plan/phases/phase-01-core/{acceptance,done,tasks,technology-stack}.md`, `docs/03-plan/phases/phase-02-rag/done.md`, `docs/03-plan/current-plan.md`, `docs/06-progress/task-done-rollup.md:200-296`, `docs/plans/handoff-p1-to-p2.md`.
- `.planning/{ROADMAP,REQUIREMENTS,PROJECT,STATE}.md`, `.planning/phases/01-foundations/{01-CONTEXT,01-RESEARCH}.md`, `.planning/phases/02-canonical-quran-core/02-CONTEXT.md`, `.planning/codebase/{ARCHITECTURE,TESTING}.md`, `AGENTS.md`, `.agent/coding-rules.md`.
- Live probes: `rustc`/`cargo`/`sqlite3`/`python3`/`git` versions; `cargo-deny` and `cargo-llvm-cov` availability; `ls migrations/sqlite/*.up.sql` (19); `grep -rn "CREATE TRIGGER"`; `grep -rn 'CanonicalWriter\|ApprovalToken' crates/`; `grep -rn 'INSERT INTO quran_*'`; `ls fixtures/upstream/quran-api/database/chapterverse/`; `ls docs/plans/`; `.planning/config.json` absence; `cargo test --workspace --no-fail-fast` (run in this session, see Metadata).

### Secondary (MEDIUM confidence)

- [Tokio graceful shutdown](https://tokio.rs/tokio/topics/shutdown) — referenced by Phase 1 for the worker host; **not** a Phase 2 concern. [CITED: https://tokio.rs/tokio/topics/shutdown]
- [SQLx `Transaction`](https://docs.rs/sqlx/0.8.6/sqlx/struct.Transaction.html) — official semantics for the explicit commit/rollback used by `UnitOfWork`. [CITED: https://docs.rs/sqlx/0.8.6/sqlx/struct.Transaction.html]

### Tertiary (LOW confidence)

- [OWASP ASVS repository](https://github.com/OWASP/ASVS) — security vocabulary only; exact control identifiers are not locked for this phase. [CITED: https://github.com/OWASP/ASVS]
- No package-registry publish-date probe was performed this session; the Standard Stack table deliberately proposes no upgrade. [ASSUMED: no upgrade recommendation]

## Metadata

**Confidence breakdown:**

- **Standard stack:** HIGH — the pinned toolchain was verified live and every recommended library is already a resolvable workspace dependency used by the existing canonical path. No new package is proposed. [VERIFIED: rust-toolchain.toml:1-5] [VERIFIED: live `rustc --version`, 2026-09-24]
- **Architecture:** HIGH — every claim is a `file:line` from a source file, migration, or test opened in this session, with the discrete values (trigger codes, QV severities, verdict names, check ids, selector variants, checkpoint names) quoted verbatim.
- **Pitfalls:** HIGH — the five material gaps (reference-comparison operator path, trigger coverage, unused type gate, `verify_quotation` call sites, translation `text_hash`) were each confirmed by a direct repo-wide search, not inferred from stale maps.
- **Legacy reconciliation:** HIGH for the numbering drift and verification status (read directly from the boards); MEDIUM for the "what the owner will accept as closure" question, which is a governance call.
- **Reference-corpus and owner gates:** MEDIUM — upstream facts come from the in-repo verified record; identity/license/signer are deliberately left as gates and are **not** asserted anywhere in this document.
- **Future implementation shapes:** LOW — task ids, the primary/default flag shape, the new translation hash recipe name, and the exact verifying surface are explicitly `[ASSUMED]` and need planner/owner confirmation.

**Evidence state at write time:** `cargo test --workspace --no-fail-fast` was started in this session to re-confirm the baseline. Individual suites observed green during that run included `crates/application/tests/quran_doctor.rs::{recomputed_hashes_match_import_time, deep_scan_of_the_fixture_has_no_failures}`; no `test result: FAILED` was observed. The final workspace total must be re-confirmed by the planner before it is recorded as the phase baseline, because a concurrent writer was active in the tree (`crates/application/src/quran_doctor_indexes.rs` modified, `crates/application/tests/doctor_indexes.rs` untracked) and a long compile/test cycle was in flight. [VERIFIED: live `cargo test --workspace --no-fail-fast` invocation, 2026-09-24] [VERIFIED: live git status, 2026-09-24]

**Research date:** 2026-09-24
**Valid until:** 2026-10-24 for stable repository findings (the canonical core, migrations, and ADRs change slowly). Re-check sooner if a new migration lands, if Phase 1's plans are executed and modify shared files, or if the owner answers OD-01/OD-02/OD-03.
