---
status: testing
phase: 03-quran-search-linguistics
source: [03-VERIFICATION.md]
started: 2026-09-28T08:12:00Z
updated: 2026-09-28T08:12:00Z
---

## Current Test

number: 1
name: OD-11 — capture the QAC license and ratify ADR-0203
expected: |
  An owner-written license capture exists at licenses/qac/{LICENSE.txt,capture.json,attribution.txt}
  per licenses/README.md; the license-matrix `qac` entry either flips to captured permissive values
  (Option A) or stays user-supplied import (Option B); the decision is recorded in
  docs/05-followups/decisions-needed.md / owner-decisions.md.
awaiting: user response

## Tests

### 1. OD-11 — capture the QAC license and ratify ADR-0203
expected: |
  An owner-written license capture exists at licenses/qac/ per licenses/README.md, the `qac`
  license-matrix entry is resolved (Option A bundle or Option B user-supplied), and the decision
  is recorded. Closes the blocker on activating licensed morphology.
result: [pending]

### 2. OD-12 / D-08 — named Arabic linguist ratifies the linguistic conventions
expected: |
  A named qualified linguist signs off on the normalization rule catalog N01–N24, the profile
  ladder L0–L8, the morphology tagset, root conventions, and alignment semantics, recorded under
  docs/reviews/. ADR-0204/0205/0210/0215 move to Accepted; golden/curated headers move off
  `reviewed_by: pending-linguist`.
result: [pending]

### 3. Counting convention ratification (ADR-0211)
expected: |
  A linguist/owner ratifies what "one occurrence" means under multi-analysis
  (SingleSource / AllAnalyses / OneVotePerToken). ADR-0211 moves Draft -> Accepted.
result: [pending]

### 4. Full-corpus performance budgets
expected: |
  With a licensed corpus active, run `cargo test -p application --test search_latency` and confirm
  the `applies_to: full-corpus` rows in fixtures/quran/performance/budgets.json execute and that
  concatenated p99 <= 150 ms (ADR-0207) is met on reference hardware.
result: [pending]

### 5. SC4 scope — co-occurrence target (root/lemma vs form/profile)
expected: |
  Owner/planner confirms the accepted SC4 interpretation. `frequency` and per-surah `distribution`
  are delivered for root/lemma; `cooccurrence` is form/profile-targeted, not lexicon root/lemma-scoped.
  Either accept, or file a follow-up for lexicon root/lemma-scoped co-occurrence.
result: [pending]

### 6. Live `qai serve` HTTP lexicon smoke
expected: |
  Start `qai serve` on a migrated temp db and POST /api/v1/quran/family and
  /api/v1/quran/count/{root,lemma}-frequency: typed coded errors for no-dataset / empty selector /
  unknown mode (never an empty 200), and attributed relations/counts when a dataset is active.
result: [pending]

### 7. WR-03 disposition — license-status derivation
expected: |
  Disposition of 03-REVIEW.md WR-03: `derive_license_status` maps any non-empty `spdx_id` to
  `OpenLicense`. Either accept as a documented risk, or require an explicit `--license-status` /
  allowlist so status is never inferred from an arbitrary string (plan 03-06 prohibition T-03-23).
result: [pending]

## Summary

total: 7
passed: 0
issues: 0
pending: 7
skipped: 0
blocked: 0

## Gaps
