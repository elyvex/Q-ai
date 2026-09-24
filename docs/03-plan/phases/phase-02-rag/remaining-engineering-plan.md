# Phase 2 — Remaining Engineering Plan

**Date:** 2026-09-24  
**Status:** Proposed execution plan; not an acceptance or ADR sign-off  
**Scope:** `docs/03-plan/phases/phase-02-rag/` (Quran Search, Arabic Normalization, Morphology & Word Families)

## 1. Objective and boundaries

Implement every Phase-2 task that can be completed from the current repository and
synthetic/test data. Keep tasks open or partial whenever completion depends on a
licensed upstream dataset, a named linguist/editorial reviewer, an owner decision,
a full-corpus run, or the Phase-2 exit ritual. Engineering evidence never closes those
gates by implication.

The live task board is the source of truth. At the time this plan was drafted it
contained **59 ☑ / 30 ◐ / 25 ☐** rows. T73 (FTS morphology projection) is complete;
all other non-☑ rows are assigned to a wave below.

## 2. Licensing direction

The owner expressed a preference for a very permissive policy. Record it as a
**proposed policy**, not as legal verification:

- Project-owned Rust code: **MIT is the leading candidate**.
- Project-authored synthetic fixtures: consider **CC0-1.0/public-domain dedication**,
  with an explicit `synthetic_test_only` marker.
- Quran text, translations, morphology, and other upstream data: **never relicensed
  by the repository license**. Each source keeps its own license, attribution, and
  redistribution evidence.
- Until the source-specific matrix is complete, morphology remains on the documented
  fallback: schema + importer + synthetic test lexicon + user-supplied datasets.
  Unknown redistribution rights mean `metadata_only` / `pending_license_review`, not
  bundling and not guessed data.

P2-T01 and P2-T02 therefore produce a source/license matrix and an automated gate
that rejects activation when required license/attribution evidence is absent.

## 3. Execution rules

1. Every implementation unit keeps its existing `P2-Tnn` identity.
2. Each wave ends with targeted tests, Clippy, formatting, and ledger/status updates.
3. Competing linguistic analyses remain side-by-side; no winner or synthesis field is
   introduced.
4. Canonical text is never rewritten by normalization, morphology, indexing, counting,
   reconciliation, or evaluation.
5. Missing data produces typed capability-unavailable errors with the missing field or
   gate named.
6. Acceptance criteria, ADRs, and owner/linguist gates are not marked complete by
   synthetic fixtures alone.

## 4. Work waves

### Wave 0 — Decision, licensing, and evidence packet

| Tasks | Work | Completion evidence | Gate |
|---|---|---|---|
| P2-T01, P2-T02 | Dataset survey and per-source license/redistribution matrix; record SPDX/license JSON, attribution, edition/tokenization, coverage, and multi-analysis depth. | Versioned matrix + machine-readable license-gate tests. | Owner/source verification required before bundling. |
| P2-T03 | Alignment strategy/specification: DirectKey, AlignmentTable, unmatched ratios, edition-relative keys. | Spec plus adversarial alignment fixtures. | Linguist/owner review remains explicit. |
| P2-T04–T10 | Normalization code-point catalog, loss notes, profile ladder, tagset/root/counting policies, and ADR drafts. | Draft documents, generated tables, schema/policy tests. | Do not mark ADRs Accepted without review. |
| P2-T11, T12, T21 | Generate normalization and root/lemma golden sets and the reusable harness; use synthetic cases until curated data is available. | Versioned fixtures, deterministic runner, precision/recall report. | Curated/linguist-reviewed portions remain gated. |
| P2-T59, T62, T74 | Chosen-dataset adapter decision, native-tag mapping, and ADR-0209 draft. | Adapter contract tests and native-tag preservation tests. | Final provider and linguist sign-off remain open. |

### Wave 1 — Morphology, lexicons, and word families

| Tasks | Work | Completion evidence |
|---|---|---|
| P2-T64 | Finish lexicon builder outputs, counts, coverage, and derived-index integration. | Real SQLite activation/index tests and count invariants. |
| P2-T65 | Add cross-dataset root-unification suggestions to the morphology review queue; never merge. | Non-merge test, evidence payload, reviewer-only promotion test. |
| P2-T67 | Make activation enqueue/rebuild the FTS generation after approval. | Job/enqueue test plus pointer/generation test. |
| P2-T68 | Complete coverage reports, unmatched-token reports, and configurable approval threshold. | Boundary tests at/below/above threshold. |
| P2-T72 | Add process-kill evidence for every import checkpoint, not only cooperative cancellation. | Checkpoint matrix with killed-process fixtures. |
| P2-T78–T81 | Complete root/lemma/pattern/affix tools and typed dataset-unavailable paths. | Tool contract tests, dataset backend tests, heuristic backend tests. |
| P2-T82 | Add root/lemma browse endpoints and `qai quran morphology root list`. | API + CLI snapshot tests. |
| P2-T84, T85, T87, T88 | Complete five-path family resolution, relation builders, opt-in computational suggestions, and reviewer promotion. | Explanation/provenance/review-queue matrix. |
| P2-T89, T90 | Expose morphology/family APIs and complete CLI dispatch, including pattern/browse paths. | Shared contract tests across application, API, and CLI. |
| P2-T91, T92 | Build root/lemma and family golden suites; label synthetic versus linguist-reviewed cases. | Versioned reports with explicit reviewer status. |

### Wave 2 — Search, index lifecycle, and performance

| Tasks | Work | Completion evidence |
|---|---|---|
| P2-T35 | Reconcile and verify the recently added single-step index rollback, including complete retained manifests. | Lifecycle rollback, eviction, and serving tests. |
| P2-T37 | Process-kill matrix for every index-build stage. | Killed-process harness; pointer remains unchanged. |
| P2-T38 | Full cold-rebuild benchmark and CI threshold gate. | Reproducible benchmark artifact and threshold check. |
| P2-T53 | Expand search goldens to the full 400-query reference set, including negative assertions. | Golden report with exact reference sets. |
| P2-T55 | Complete latency benchmarks and CI budgets. | Machine-readable benchmark report. |
| P2-T39, T56 | Finish ADR-0201/0208/0213 and search-regex/limit ADR updates. | Draft ADRs and link/index consistency checks. |

### Wave 3 — Counting and discovery correctness

| Tasks | Work | Completion evidence |
|---|---|---|
| P2-T95 | Finish exact SQL frequency modes and all `MultiAnalysisHandling` variants. | Determinism and count-rule matrix. |
| P2-T96 | Complete distribution partitions, provenance, and disagreement warnings. | Partition/provenance fixtures. |
| P2-T97 | Complete token/ayah/segment co-occurrence windows and cross-ayah flags. | Window and boundary tests. |
| P2-T101 | Complete hapax and unusual-usage behavior with prominent profile labels. | Profile-sensitive goldens. |
| P2-T102 | Complete MinHash candidate generation plus exact verification and aligned spans. | Precision/recall and false-positive tests. |
| P2-T104 | Finish counting/discovery API and CLI parity. | Shared contract and snapshot tests. |

### Wave 4 — Doctor, reconciliation, evaluation, and tool contracts

| Tasks | Work | Completion evidence |
|---|---|---|
| P2-T105 | Implement the 19 Phase-2 doctor checks, including search smoke and morphology provenance. | One deterministic report per check, typed repair actions. |
| P2-T106 | Report precise index-input drift and emit `QAI-IDX-0101` warnings. | Version/edition/rule/dataset mismatch fixtures. |
| P2-T107 | Add nightly reconciliation: sampled index verification, MV-018, and pointer consistency. | Scheduled job and simulated-drift tests. |
| P2-T108 | Add evaluation metric definitions, versioned datasets, baselines, and gate reports. | Reproducible evaluation command and report schema. |
| P2-T110 | Generate conformance coverage for all 22 Phase-2 tools with identical core semantics. | Tool-contract matrix and failure snapshots. |
| P2-T111 | Run full rebuild → 50,000 randomized queries → doctor → reconciliation soak. | Timestamped soak artifact and reproducible seed. |

### Wave 5 — Documentation, ADRs, and exit review

| Tasks | Work | Completion evidence |
|---|---|---|
| P2-T112 | Complete ADR-0216 and the ADR index update. | ADR links/status consistency check. |
| P2-T113 | Publish normalization spec, profile catalog, search cookbook, morphology adapter guide, counting explainer, and reindex runbook. | Documentation review + command examples. |
| P2-T114 | Run the Phase-2 exit review and prepare the Phase-3 handoff. | Complete evidence index; blocked gates listed explicitly. |

## 5. Definition of Done for each engineering wave

- Implementation and tests are committed in the repository's normal workflow.
- Targeted and regression tests pass; formatting, Clippy, `arch-check`, and
  `migrate-check` remain green.
- The task board, `done.md`, progress rollup, and changelog agree.
- Synthetic data is labeled synthetic and never presented as scholarly ground truth.
- No task is marked ☑ when its owner, linguist, licensing, full-corpus, or exit gate
  remains unresolved; those rows remain ◐/☐/⊘ with a named blocker.

## 6. Immediate next execution slice

Start Wave 0's machine-readable license gate and Wave 1's T65/T78–T81/T82 work in
parallel where dependencies permit. Do not wait for a final dataset license to build
the fallback path. The first implementation checkpoint should produce:

1. a license matrix and activation rejection test;
2. `quran.pattern_search` with typed unavailable behavior when no pattern field exists;
3. root/lemma browse API/CLI surfaces;
4. a fresh evidence report showing exactly which owner/linguist gates remain.
