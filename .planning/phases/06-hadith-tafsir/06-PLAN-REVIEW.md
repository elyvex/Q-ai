# Phase 6 Plan Review — Hadith & Tafsir

**Reviewer:** Plan quality reviewer (subagent)
**Date:** 2026-10-12
**Plans reviewed:** 06-01, 06-02, 06-03, 06-04
**Verdict:** PASS with warnings

---

## Summary

The 4 plans for Phase 6 are well-structured, follow the established tracer-first pattern, cover all phase requirements and success criteria, and maintain consistent quality across frontmatter, task structure, threat models, and verification commands. The plans are ready for execution with minor warnings.

---

## 1. Frontmatter Completeness

| Plan | phase | plan | title | wave | depends_on | files_modified | autonomous | requirements | must_haves |
|------|-------|------|-------|------|------------|----------------|------------|--------------|------------|
| 06-01 | ✓ | ✓ | ✓ | ✓ (1) | ✓ ([]) | ✓ (12) | ✓ | ✓ (hadith-corpus) | ✓ |
| 06-02 | ✓ | ✓ | ✓ | ✓ (2) | ✓ (06-01) | ✓ (10) | ✓ | ✓ (tafsir-comparative) | ✓ |
| 06-03 | ✓ | ✓ | ✓ | ✓ (3) | ✓ (06-01, 06-02) | ✓ (6) | ✓ | ✓ (both) | ✓ |
| 06-04 | ✓ | ✓ | ✓ | ✓ (4) | ✓ (06-01, 06-02, 06-03) | ✓ (9) | ✓ | ✓ (both) | ✓ |

**Status:** PASS

---

## 2. Task Acceptance Criteria

All 12 tasks across 4 plans have `<done>` sections with measurable, testable criteria:

| Plan | Task | Done criteria count | Quality |
|------|------|---------------------|---------|
| 06-01 | 1 (tracer) | 6 | Specific, measurable |
| 06-01 | 2 (auto) | 6 | Specific, measurable |
| 06-01 | 3 (auto) | 6 | Specific, measurable |
| 06-02 | 1 (auto) | 5 | Specific, measurable |
| 06-02 | 2 (auto) | 6 | Specific, measurable |
| 06-02 | 3 (auto) | 5 | Specific, measurable |
| 06-03 | 1 (auto) | 5 | Specific, measurable |
| 06-03 | 2 (auto) | 6 | Specific, measurable |
| 06-03 | 3 (auto) | 6 | Specific, measurable |
| 06-04 | 1 (auto) | 6 | Specific, measurable |
| 06-04 | 2 (auto) | 6 | Specific, measurable |
| 06-04 | 3 (auto) | 6 | Specific, measurable |

**Status:** PASS

---

## 3. Files Specified

All tasks have `<files>` sections listing concrete file paths. All frontmatter `files_modified` lists are consistent with task-level file lists (with one exception noted below in Issues).

| Plan | Files in frontmatter | Files in tasks | Consistent |
|------|----------------------|---------------|------------|
| 06-01 | 12 | 12 | ✓ |
| 06-02 | 10 | 10 | ✓ |
| 06-03 | 6 | 6 | ✓ |
| 06-04 | 9 | 10 | ✗ (1 extra in tasks) |

**Status:** PASS with 1 warning

---

## 4. Verification Commands

All tasks have `<verify><automated>` blocks with cargo test commands:

| Plan | Task | Verify command |
|------|------|---------------|
| 06-01 | 1 | `cargo test -p hadith-core` |
| 06-01 | 2 | `cargo test -p application -- hadith_search` |
| 06-01 | 3 | `cargo test -p tool-registry && cargo test -p application -- hadith_tools` |
| 06-02 | 1 | `cargo test -p tafsir` |
| 06-02 | 2 | `cargo test -p application -- tafsir_search` |
| 06-02 | 3 | `cargo test -p tool-registry && cargo test -p application -- tafsir_tools` |
| 06-03 | 1 | `cargo test -p application -- verse_link` |
| 06-03 | 2 | `cargo test -p application -- verse_link` |
| 06-03 | 3 | `cargo test -p application -- disagreement` |
| 06-04 | 1 | `cargo test -p hadith-ingestion` |
| 06-04 | 2 | `cargo test -p cli` |
| 06-04 | 3 | `cargo test -p server && cargo test --workspace --parity` |

**Status:** PASS with 1 warning (see Issues)

---

## 5. Dependency Correctness

```
Wave 1: 06-01 (tracer, depends_on: [])
Wave 2: 06-02 (depends_on: [06-01])
Wave 3: 06-03 (depends_on: [06-01, 06-02])
Wave 4: 06-04 (depends_on: [06-01, 06-02, 06-03])
```

- All referenced plans exist ✓
- No circular dependencies ✓
- Wave numbers consistent with dependency depth ✓
- Forward references only (later plans depend on earlier) ✓

**Status:** PASS

---

## 6. Threat Models

All 4 plans have complete threat models with trust boundaries and STRIDE registers:

| Plan | Threats | Dispositions | Categories covered |
|------|---------|-------------|-------------------|
| 06-01 | 6 (T-06-01–05 + T-06-SC) | All mitigate | Tampering, Spoofing, Repudiation |
| 06-02 | 3 (T-06-05–07) | All mitigate | Tampering, Repudiation |
| 06-03 | 3 (T-06-08–10) | All mitigate | Spoofing, Repudiation, Tampering |
| 06-04 | 3 (T-06-11–13) | All mitigate | Tampering, Repudiation, Spoofing |

All threats have disposition = "mitigate" with concrete mitigation plans. No threats are marked "accept" or "transfer" which would require additional justification.

**Status:** PASS

---

## 7. Tracer-First Pattern

06-01 is correctly designated as the tracer:
- Task 1 has `type="tracer"` ✓
- Covers domain types → storage → search → typed tool end-to-end ✓
- Proves the architectural path before expansion ✓
- Follows the pattern established in Phase 2 (02-01), Phase 3 (03-01), Phase 4 (04-01), Phase 5 (05-01) ✓

**Status:** PASS

---

## 8. Requirements Coverage

| Requirement | Plans | Coverage |
|-------------|-------|----------|
| REQ-hadith-corpus | 06-01, 06-03, 06-04 | Complete |
| REQ-tafsir-comparative | 06-02, 06-03, 06-04 | Complete |

Both roadmap requirements are covered by multiple plans with specific tasks addressing each.

**Status:** PASS

---

## 9. Success Criteria Coverage

| SC | Description | Covered by | Status |
|----|-------------|-----------|--------|
| SC1 | Hadith search (exact + normalized) with collection, edition, source numbering | 06-01 Task 2, 06-04 Task 3 | ✓ |
| SC2 | Matn/isnad separation + grading attribution | 06-01 Tasks 1,3; 06-03 Task 3 | ✓ |
| SC3 | Verse → hadith/tafsir navigation | 06-03 Tasks 1,2; 06-04 Task 3 | ✓ |
| SC4 | Disagreement shown side-by-side | 06-03 Task 3 | ✓ |
| SC5 | Computational matches labeled uncertain | 06-03 Task 2 | ✓ |

All 5 success criteria from the roadmap have implementing tasks.

**Status:** PASS

---

## 10. Scope Assessment

| Plan | Tasks | Files | Wave | Assessment |
|------|-------|-------|------|------------|
| 06-01 | 3 | 12 | 1 | Within budget (tracer can be denser) |
| 06-02 | 3 | 10 | 2 | Within budget |
| 06-03 | 3 | 6 | 3 | Within budget |
| 06-04 | 3 | 9 | 4 | Within budget |

All plans are at 3 tasks (target: 2-3). File counts are within the 5-8 target range for 06-03, slightly above for 06-01 (12) and 06-02 (10) but justified by the migration-heavy nature of the work.

**Status:** PASS

---

## 11. Context Compliance

The plans reference D-01 through D-07 as "locked decisions" but the CONTEXT.md does not use D-XX numbering. The decisions exist in CONTEXT.md's `<decisions>` section but are organized by topic (Data Model, Grading Attribution, Verse Linking, etc.) rather than by ID.

| Plan references | CONTEXT.md equivalent | Traceable |
|-----------------|----------------------|-----------|
| D-01 | "source numbering preserved verbatim" | Yes, by content |
| D-02 | "matn_text is canonical-only" | Yes, by content |
| D-03 | "grading records are per-scholar, never consensus-merged" | Yes, by content |
| D-04 | "grading attribution" | Yes, by content |
| D-05 | "all surfaces call the same application service layer" | Yes, by content |
| D-06 | "tafsir indexed by work/author/dimensions" | Yes, by content |
| D-07 | "computational matches carry confidence score and uncertain label" | Yes, by content |

The content maps 1:1 but the D-XX IDs are not formally defined in CONTEXT.md.

**Status:** WARNING

---

## 12. Cross-Plan Data Contracts

| Shared data | Producer | Consumer | Contract |
|-------------|----------|----------|----------|
| hadith_records | 06-01 | 06-03, 06-04 | source_id FK in verse_links |
| tafsir_records | 06-02 | 06-03, 06-04 | source_id FK in verse_links |
| hadith_gradings | 06-01 | 06-03 | Per-scholar records, no consensus |
| ToolResult envelope | 06-01, 06-02 | 06-04 | Same envelope across all tools |

No conflicting transformations detected. The verse_links table in 06-03 is the integration point and both 06-01 and 06-02 produce compatible source_id values.

**Status:** PASS

---

## 13. AGENTS.md Compliance

| Directive | Compliant | Notes |
|-----------|-----------|-------|
| Rust primary language | ✓ | All tasks target Rust crates |
| Provenance-safe storage | ✓ | All records have source_version_id, text_hash, research_checksum |
| Canonical text separation | ✓ | matn_text/text columns are canonical-only, search forms in separate tables |
| No fabrication | ✓ | All tool results carry research_checksum + ReproducibilityData |
| Deny-by-default permissions | ✓ | Owner gate on verse links, reviewer on grading decisions |
| Task tracking | ✓ | Plans reference requirements, not individual TASK-nnn files |

**Status:** PASS

---

## Issues Found

### WARNING-1: Decision ID Traceability Gap

**Severity:** WARNING
**Dimension:** context_compliance
**Description:** Plans reference D-01 through D-07 as "locked decisions" but CONTEXT.md does not use D-XX numbering. While the content maps 1:1, a reader cannot formally verify the mapping without guessing. This could cause confusion during execution if the executor needs to verify a decision's scope.
**Required property:** Decision references in plans must be traceable to formally defined decisions in CONTEXT.md.
**Fix (non-binding):** Add D-XX IDs to CONTEXT.md decisions section, or remove D-XX references from plans and reference by topic name instead.

### WARNING-2: Plan 06-04 Task 2 File Inconsistency

**Severity:** WARNING
**Dimension:** task_completeness
**Description:** Task 2 `<files>` lists `crates/cli/src/lib.rs` but this file is absent from the plan's frontmatter `files_modified` list. This creates an inconsistency in file tracking — the executor may not know if this file is expected to be modified or just read.
**Required property:** Task `<files>` lists must be a subset of (or equal to) the plan's `files_modified` frontmatter.
**Fix (non-binding):** Add `crates/cli/src/lib.rs` to frontmatter `files_modified` or remove it from the task's `<files>` if it is not actually modified.

### WARNING-3: Non-Standard Cargo Test Flag

**Severity:** WARNING
**Dimension:** task_completeness
**Description:** Plan 06-04 Task 3 verify command uses `cargo test --workspace --parity`. The `--parity` flag is not a standard cargo test flag. This may be a custom test target defined in the workspace, or it may be an error. If invalid, the verify command will fail during execution.
**Required property:** Verify commands must use valid cargo test syntax.
**Fix (non-binding):** Verify that `--parity` is a valid test target or replace with a standard pattern like `cargo test --workspace parity_gate` or `cargo test --workspace --test parity`.

---

## Recommendation

**PASS with warnings.** The plans are well-structured, comprehensive, and ready for execution. The 3 warnings are minor traceability and consistency issues that should be addressed but do not block execution:

1. Add formal D-XX IDs to CONTEXT.md or remove D-XX references from plans
2. Reconcile `crates/cli/src/lib.rs` in Plan 06-04 Task 2
3. Verify `--parity` flag validity in Plan 06-04 Task 3

The tracer-first pattern is correctly followed, all requirements and success criteria are covered, dependencies are valid, threat models are complete, and scope is within budget.
