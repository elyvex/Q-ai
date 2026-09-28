---
gsd_state_version: "1.0"
current_phase: 4
current_phase_name: Quran Graph
status: complete
stopped_at: Phase 4 context gathered
last_updated: "2026-09-28T14:21:24.988Z"
last_activity: 2026-09-28
last_activity_desc: Completed 03-08-PLAN.md — phase 03 execute-complete (owner gates OD-11/OD-12 BLOCKED)
state_head: 02c9c9e24ef12191da983c0498e373f0ac0fbfab
progress:
  total_phases: 12
  completed_phases: 2
  total_plans: 25
  completed_plans: 20
  percent: 17
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-09-23)

**Core value:** Trustworthy Quran research: exact canonical text before generated interpretation, every factual claim traceable to its source.
**Current focus:** Phase 03 — Quran Search & Linguistics

## Current Position

Phase: 4 (Quran Graph) — READY TO EXECUTE
Plan: 8 of 8
Status: Complete (execute) — alpha proven; OD-11/OD-12 BLOCKED owner gates
Last activity: 2026-09-28 — Completed 03-08-PLAN.md (phase 03 complete: 8/8 plans)

Progress: [██░░░░░░░░] 17%

## Performance Metrics

**Velocity:**

- Total plans completed: 12
- Average duration: -
- Total execution time: -

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 02 | 7 | - | - |
| 01 | 5 | - | - |

**Recent Trend:**

- Last 5 plans: -
- Trend: -

*Updated after each plan completion*
**Per-Plan Metrics:**

| Plan | Duration | Tasks | Files |
|------|----------|-------|-------|
| Phase 02 P01 | 22 min | 3 tasks | 14 files |
| Phase 02 P02 | 40 min | 2 tasks | 6 files |
| Phase 02 P04 | 30 min | 3 tasks | 11 files |
| Phase 02 P03 | 19 min | 3 tasks | 26 files |
| Phase 02 P06 | 12 min | 2 tasks | 4 files |
| Phase 02 P05 | 11 min | 3 tasks | 9 files |
| Phase 02 P07 | 16 min | 3 tasks | 9 files |
| Phase 01-foundations P01 | 45min | 3 tasks | 9 files |
| Phase 01-foundations P02 | 20min | 2 tasks | 8 files |
| Phase 01-foundations P03 | 35min | 3 tasks | 16 files |
| Phase 03 P01 | 19 min | 3 tasks | 3 files |
| Phase 01-foundations P04 | 72min | 3 tasks | 32 files |
| Phase 01-foundations P05 | 150min | 3 tasks | 10 files |
| Phase 03 P02 | 20 min | 3 tasks | 4 files |
| Phase 03 P03 | 23 min | 3 tasks | 8 files |
| Phase 03 P04 | 18 min | 3 tasks | 4 files |
| Phase 03 P05 | 1h 50m | 2 tasks | 12 files |
| Phase 03 P06 | 27 min | 3 tasks | 10 files |
| Phase 03 P07 | 28 min | 3 tasks | 7 files |
| Phase 03 P08 | 50 min | 3 tasks | 5 files |

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table (`<decisions>` block, 25 locked).
Recent decisions affecting current work:

- [Bootstrap]: 25 ADR-locked decisions constrain all phases; 8 proposed ADRs are explicit decision points, not assumptions
- [Bootstrap]: PRD Phase 9+10 merged into roadmap Phase 10 (shared agent/tool runtime + gates)
- [Bootstrap]: MVP acceptance lands in Phase 10; Phases 11–12 post-MVP
- [Phase 02]: Edition identity (upstream_edition_slug, qai_edition_id, is_primary) is stored verbatim from the manifest; absent values stay NULL/None/false (OD-01).
- [Phase 02]: Canonical license_json stores the manifest-declared license object verbatim; the reader maps unmodelled statuses to LicenseStatus::Unknown and never invents redistribution permission.
- [Phase 02]: EditionSelector::Primary resolves the single flagged edition and returns typed errors when zero or more than one is flagged; it never falls back to the active pointer (D-07).
- [Phase 02]: Renumbered the rich fixture to surahs 1..=6 (identity to 1) so Fatal QV-002 passes and plan 02-03 can import/activate both fixtures.
- [Phase 02]: Golden rows are edition-scoped by an EDITION field because multiple synthetic editions are each numbered 1..=N.
- [Phase 02]: The reference_text_hash pin reuses the existing text_hash recipe verbatim; no new recipe was invented and renumbering does not change it.
- [Phase 02]: The CanonicalWriter/ApprovalToken gate is wired, not retired: ADR-0000 locks canonical immutability as a type-level property and .agent/coding-rules.md states the invariant, so ApprovalGate implements CanonicalWriter and ApprovalToken is only mintable from a persisted granted approval row (from_approval_row).
- [Phase 02]: Canonical quran_segments stays empty in Phase 2 (reserved for the morphology phase); the frozen activation copy list is unchanged and a future phase adds a forward migration plus a copy step.
- [Phase 02]: qai quran verify classifies the six integrity families from persisted state; a skipped family is serialized as skipped and never as pass (D-10), and the command is read-only (T-02-07).
- [Phase 02]: The independent reference corpus reaches run_import through the import payload (ImportInput.reference_manifest_text); identity/pins come verbatim from the operator document and QV-015 stays byte-only fail-closed, with ADR-0114 DifferenceClass as report metadata only (OD-03 open).
- [Phase 02]: qai quran edition show surfaces the manifest-declared license status verbatim from the canonical row (never inferred) alongside upstream identity and the primary marker (T-02-36).
- [Phase 02]: Translation content hash uses an additive, domain-separated qai-translation-hash-v1 recipe over passages sorted ascending by (surah, ayah); the frozen qai-text-hash-v1/structure_hash/token_order_hash recipes and domain strings are untouched (D-12, ADR-0108), and the digest is independent of manifest passage order (T-02-25).
- [Phase 02]: A declared translation license stores the manifest SPDX identifier character-for-character (status OpenLicense); an undeclared license stays explicit Unknown and no redistribution/export permission is invented (T-02-24, OD-01 remains the owner gate).
- [Phase 02]: Translation layer separation is proven at table, type, and test level: canonical and translation rows live in quran_* versus translation_* tables, AyahView.canonical is a QuranQuotation, and a source scan asserts no canonical view constructor takes a translator/language pair (D-04, ADR-0112, T-02-22).
- [Phase 02]: The shared verdict-to-hard-failure mapping lives in the citations domain crate (QuotationVerdict::is_hard_failure + citations::require_exact); exit-code and HTTP-status concerns stay in the callers so citations stays free of CLI concerns (D-15, ADR-0010 code separation).
- [Phase 02]: The shared application verifier (verify_canonical_quotation) calls the resolver's resolve path directly because it returns the resolved canonical hash in a single fetch; verify_quotation delegates to the same resolve, so the verdict is identical and the hash is never computed from the supplied text (T-02-19).
- [Phase 02]: The HTTP citation handler is the single enforcement point for stored verdicts, so any backend's hard-failing verdict returns a typed error instead of a 200 envelope; no new route, stored-citation field, or error-code map was added (T-02-18).
- [Phase 02]: The direct-read answer paths (tool quran.get_ayah/get_context, CLI direct reads, HTTP direct reads) are structurally exempt from verify_quotation because they serve canonical text and cannot mismatch by construction; wrapping them would compare canonical text to itself (Pitfall 4).
- [Phase 02]: MatchAfterDeclaredNormalization is unreachable in v1 (the resolver has no normalization-rules parameter); the tests assert the reachable verdict set excludes it rather than claiming behaviour for it.
- [Phase 02]: Phase 02-07: owner gates OD-01/OD-02/OD-03 recorded as blocked with closing commands; D-15 read-path exemption recorded as owner-ratifiable; committed corpus-integrity artifact; coverage gate reconciled (citations 79% vs 85% floor recorded); TASK-002 closed active→completed.
- [Phase 01]: 01-01: seed_synthetic_chain lives in application as documented pub test-support (arch-check edge boundary); serve/search creating-paths deferred to owning phases
- [Phase 01]: [Phase 01-02]: One AuditedMutation application seam over the existing UnitOfWork — no second transaction manager, hash format, or parallel audit writer (D-01, D-10)
- [Phase 01]: 01-03: immediate checkpoint persistence through the queue; per-kind policy with enqueue snapshot; request-only cancellation with three locked dispositions; checkpoint/heartbeat excluded from audit chain (D-14/D-15/D-16/D-11)
- [Phase 03]: [Phase 03-01]: SC5 is pinned by a per-hit test (canonical_display_identity.rs) that independently resolves each hit's ayah through the reader and asserts the displayed quotation is byte-identical to the canonical row at the hit's canonical span across all five search modes; canonical stored hashes are unchanged across searches (I8/SC5).
- [Phase 03]: [Phase 03-01]: The empty/whitespace query fallback edge is disposed at the CLI boundary: cmd_search returns a typed usage error (exit::USAGE, 'provide query text') before opening the service; search_normalized has no empty guard and legitimately returns Ok with zero hits, so the guard is asserted where the operator actually calls it.
- [Phase 01-foundations]: 01-04: qai serve owns the durable worker host (run_until_shutdown over a watch channel, default registry, readiness gate, signal-driven joined shutdown); one-shot quran import is enqueue-only with a queued job result (D-13..D-16)
- [Phase 01-foundations]: 01-04: CLI corpus flows run host-backed as 22 import-boundary trycmd segments with read-only terminal-state sync; QV-015 mismatch fails through the host to an inspectable dead letter with nothing staged
- [Phase 01-foundations]: 01-05: registry/git external policy derived from live cargo metadata (23 crates, git empty, no package churn) with inline mutation coverage; TASK-001 closed active→completed (indexes 0/1) with a record-specific checker; tree-wide fmt + quran_identity reds quarantined as foreign concurrent-session files, quoted and untouched (D-01/D-02/D-03)
- [Phase 03]: [Phase 03-02]: The concatenated golden oracle is independent of the Rust search stack — a pure-Python re-implementation of L6's full ordered rule list plus the surah-scoped 3-ayah window store computes every expectation, so agreement with the service is evidence rather than tautology.
- [Phase 03]: [Phase 03-02]: Concatenated golden rows carry their own allow_cross_ayah/max_ayah_span selectors and the dispatcher passes them through (never a literal or default); cross-ayah rows are restricted to clean boundary cases so the boundary parts provably tile the whole query, and ayah-level-win dedup is asserted on every row.
- [Phase 03]: [Phase 03-02]: The plan's CLI target crates/cli/tests/quran/search.trycmd does not exist; SC2 CLI snapshots land in the real host-backed segment crates/cli/tests/quran/search_s2.trycmd (Rule 3 correction).
- [Phase 03]: 03-03: Multi-analysis counting semantics (SingleSource = single-matching-analysis token; AllAnalyses = one vote per analysis row; OneVotePerToken = one vote per token) are code-implemented but not linguist-ratified (OD-12 BLOCKED, ADR-0211 Draft); asserted on the synthetic lexicon only and never elect an authoritative winner (I11/ADR-0209).
- [Phase 03]: 03-03: root/lemma frequency and the selectable-mode rules block read the active <slug>@<version> dataset from the active quran_datasets row; no active dataset stays a typed CountingError::UnavailableDataset (QAI-CNT-0005), never an empty report (SC4/G-01/G-07).
- [Phase 03]: [Phase 03]: 03-04: Every typed family relation kind (same_form/same_lemma/same_stem/same_root/derived/inflectional/affix) is built as an explained, dataset-attributed, status-proposed row via one db.write() UoW; relation strings come from FamilyRelation::relation_name() so the 0017 CHECK domain is satisfied by construction, and a non-merge test asserts no is_correct/is_primary/selected column exists and no analysis is ever flipped off imported (I11/I13/ADR-0210).
- [Phase 03]: [Phase 03]: 03-04: affix_search fails closed with MorphologyToolError::UnavailableDataset naming 'affix/morpheme index' (QAI-MORPH-0004) instead of the silent Ok(Vec::new()) on the active-dataset path; the labeled L7 heuristic branch is unchanged for the no-dataset case (G-09).
- [Phase 03]: [Phase 03]: 03-04: The family synthetic lexicon must keep lemma->root functional (lemma = k%9, root = k%3) because quran_lemmas is UNIQUE(dataset_id, lemma); the first scheme (root k%4 / lemma k%5) tripped activation with a unique-constraint violation (Rule 1 fix).
- [Phase 03]: [Phase 03]: 03-04: The 154-family curated golden set stays reviewed_by: pending-linguist / reviewed_at: null with every row synthetic_test_only, and the runner pins that labeling; linguistic correctness awaits OD-12/D-08 (ADR-0210/0215 not-Accepted).
- [Phase 03]: [Phase 03]: 03-05: The lexicon surfaces are CLI/HTTP-only wrappers over the existing services — cmd_family + a new LexiconBackend/LexiconApiService both call word_family/root_frequency/lemma_frequency, so SC3/SC4 results cannot drift between surfaces (G-02/G-10/D-10).
- [Phase 03]: [Phase 03]: 03-05: A missing morphology dataset is the same typed capability error on both surfaces — exit 5 (QAI-MORPH-0004, the existing tool_exit mapping) at the CLI and 404 (new QAI-LEX-0004 Diagnostic namespace, 'an unavailable capability is a state problem') at HTTP — never an empty list or an empty 200 envelope (T-03-21).
- [Phase 03]: [Phase 03]: 03-05: qai quran family/lemma are TOP-LEVEL QuranAction variants per D-10 (nesting under MorphologyAction would render qai quran morphology family); the published FrequencyReport keeps its CountingRules block under the field name 'rules' rather than being renamed to counting_rules.
- [Phase 03]: [Phase 03]: 03-05: The plan's named CLI artifact crates/cli/tests/quran/family.trycmd became family_s1.trycmd + family_s2.trycmd because the host-backed harness splits a queued import from its dependent commands (the identical correction 03-02/03-03 recorded); a new quran_family_snapshots runner entry was required or the file would never execute.
- [Phase 03]: 03-06: The morphology license evidence reaches gate 6 through the dataset row: run_morphology_import upserts the row (state staged) with the operator's license_status/license_json/attribution and activate_morphology validates it before flipping state to active — no migration (the 0017 row already carries the columns), and every activation path shares one enforcement point (D-07/G-04).
- [Phase 03]: 03-06: Activation promotion no longer re-upserts the dataset row (which hardcoded license_status Unspecified / license_json {}): it calls set_dataset_state(active), and the import-side upsert is skipped when the existing row is already active, so a validated dataset's evidence can never be silently downgraded (T-03-22).
- [Phase 03]: 03-06: The permissive allowlist is the permissive half of domain::LicenseStatus (PublicDomain/OpenLicense/PermissionGranted/UserOwned); Unspecified/metadata_only/pending_license_review/Unknown/empty and any unrecognized status are refused, and redistribution_allowed is the binding bundling gate. The gate code is append-only QAI-MORPH-6 mapping to exit::VALIDATION.
- [Phase 03]: 03-06: OD-11 stays BLOCKED and is implemented as a gate rather than resolved: the qac entry in fixtures/quran/morphology/license-matrix.json is pending_license_review/redistribution_allowed:false and QAC ships only via user-supplied import until licenses/qac/ is captured per licenses/README.md; the gate validates license evidence structurally, not the truth of the captured terms.
- [Phase 03]: 03-07: G-08 closed by an end-to-end doctor soak (import+activate -> forms -> index -> morphology import+activate -> search per mode). 'All 19 green' is realized as 19 stable ids with no failure: the token-level index (quran.fts.token_index) stays skipped by design (no backend), never a silent pass/false fail. The drift case produces QAI-IDX-0101 through the SearchHit stale-index warning after a v2 edition is activated while the serving generation still points at v1, and doctor reports quran.index.drift as warn (read-only, never repairs).
- [Phase 03]: 03-07: G-06 closed by fixtures/quran/performance/budgets.json — a machine-readable table codifying ADR-0207's concatenated p99<=150ms, the plan 17.1 p50/p99 rows, and fixture_bound_ms=2000, each with applies_to + rationale. search_latency.rs loads it (include_str!), gates the fixture runs against fixture_bound_ms and asserts it is not loosened; the full-corpus rows are recorded OD-11-dependent and never executed on the 14-ayah fixture.
- [Phase 03]: 03-07: G-11/D-13 closed by extending QuranBackend with backend_search/backend_root/backend_lemma/backend_morphology/backend_family (default typed-unsupported so no backend is forced to implement them) and registering 7 tool names. ReaderToolBackend builds attributed envelopes: quran.search carries normalization_rules from the served hit's NormalizationTrace (I9) plus canonical sources, the lexicon tools carry dataset attribution, and an unavailable dataset surfaces the typed QAI-MORPH-0004 backend error rather than an empty result.
- [Phase 03]: 03-08: OD-11 and OD-12/D-08 are recorded as explicit BLOCKED owner gates with exact closing steps (licenses/qac capture per licenses/README.md; named Arabic linguist + ADR-0204/0205/0210/0215 acceptance); decisions-needed.md stays the source of truth and the plan closes nothing (T-03-30).
- [Phase 03]: 03-08: The D-03 alpha is proven on the synthetic fixture only — one full-chain test (alpha_e2e::alpha_end_to_end_synthetic) runs normalize -> forms -> index -> five search modes -> morphology/root/lemma/family -> frequency/distribution/co-occurrence, gating every hit on trace + canonical span + byte-identical quotation (SC5), every lexicon result on dataset attribution, and every numeric report on a complete CountingRules; no linguistic correctness or real-data behavior is claimed (OD-11/OD-12 BLOCKED, T-03-31/T-03-32).
- [Phase 03]: 03-08: The deferred scope is recorded in docs/05-followups/phase-03-deferrals.md (UI + result-contract checksum -> Phase 5; counting tail incl. hapax_search -> later; legacy phase-02-rag tail -> follow-ups; transliteration/L8; graph -> Phase 4; editions/multi-RAG), and the 03-VALIDATION.md per-task map is deferred to /gsd-validate-phase with the PLAN <verify> blocks as the authoritative Phase-3 map (T-03-33).

### Pending Todos

None yet.

### Blockers/Concerns

- ADR-0101 initial Quran dataset + license needs human sign-off before Phase 2 import work
- ADR-0201/0202/0701/0301 (retrieval/graph/vector/RAG) undecided — Phase 4/8 plans must include decision points
- ADR-0702 foundational subset (outbox/revisions/jobs/tombstones) required from Phases 0–3

## Deferred Items

Items acknowledged and deferred at milestone close, most recent first:

| Category | Item | Status | Deferred At | Milestone |
|----------|------|--------|-------------|-----------|
| *(none)* | | | | |

## Session Continuity

Last session: 2026-09-28T13:45:24.796Z
Stopped at: Phase 4 context gathered
Resume file: .planning/phases/04-quran-graph/04-CONTEXT.md
