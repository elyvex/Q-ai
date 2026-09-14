# Phase 2 — Acceptance Criteria & Exit Gate

**Phase:** P2 — Quran Search, Arabic Normalization, Morphology & Word Families
**Source:** `plan.md` §18 (Acceptance Criteria), §16 (Testing Strategy), §17
(Performance & Evaluation), §13 (Schema), §12 (Doctor)
**Plan criteria:** 50 (AC-P2-01 … AC-P2-50)
**Gate owner:** task P2-T114
**Status:** 🔴 0 / 50 verified

**Status legend:** ☐ Not verified · ◐ Partially verified · ✗ Failed · ☑ Verified

> **Verification rule:** an AC is ☑ only when an **automated test or scripted check** proves it
> and the evidence artifact (CI run URL, test path, or recording) is recorded in the Evidence
> column and appended to `done.md`. "It works on my machine" is not verification. The
> exit-gate ritual (§5) additionally requires a **recorded live walkthrough** by reviewers who
> are not the implementers (the Arabic linguist + the Phase-1 editorial reviewer).

---

## 1. Acceptance Criteria

### 1.1 Dataset & linguistic foundations (blocking everything)

| ID | Criterion | Verification | Blocks | Evidence | Status |
|---|---|---|---|---|---|
| **AC-P2-01** | ADR-0203 accepted; a licensed morphology dataset is active, **or** the documented user-supplied fallback works end-to-end with the public-domain test lexicon | ADR + scripted run | D2.6, all morphology | | ☐ |
| **AC-P2-02** | ADR-0204/0205 accepted with complete code-point mapping tables; every rule's linguistic loss is documented | ADR review by linguist | D2.1 | | ☐ |

> AC-P2-01 is the Phase-2 morphological hard gate. If no dataset can be bundled, the fallback
> **must be written in ADR-0203** (ship test lexicon; user imports their own dataset) — an
> undocumented absence of data is a gate failure, not an acceptable ambiguity. Tools degrade to
> typed "dataset unavailable" errors naming the missing capability, never to guessed data.

### 1.2 Normalization & offsets (the integrity core)

| ID | Criterion | Verification | Blocks | Evidence | Status |
|---|---|---|---|---|---|
| **AC-P2-03** | All 2,000 normalization golden pairs pass for all indexed profiles | golden suite | D2.1 | | ☐ |
| **AC-P2-04** | `SpanMap` satisfies all 5 properties across every ayah × every profile; offset fidelity is 1.00 | property suite | D2.1 | | ☐ |
| **AC-P2-05** | **Canonical text is byte-identical** (all Phase-1 hashes match) after building every index and importing every dataset — verified by MV-018 in every job and by `doctor` | integrity suite + doctor | D2.2, D2.6, D2.10 | | ☐ |
| **AC-P2-06** | No `SearchHit` can exist without a `NormalizationTrace`; every trace lists the ordered rule ids and flags heuristics | type test + snapshot | D2.5 | | ☐ |

> AC-P2-05 is the I8 guarantee. If a single canonical hash drifts after any build or import,
> Phase 2 does not exit — no partial pass, no whitelist without an ADR.

### 1.3 Search behaviour

| ID | Criterion | Verification | Blocks | Evidence | Status |
|---|---|---|---|---|---|
| **AC-P2-07** | Searching `الرحمن` (no diacritics) returns 1:1, 1:3, 2:163 and the full expected reference set; searching the exact Uthmani form returns the same set | golden query | D2.5 | | ☐ |
| **AC-P2-08** | Searching `بسمالله` (no spaces, no diacritics) returns 1:1 with a segmentation explanation mapping each query part to canonical tokens 1 and 2 | golden query | D2.4 | | ☐ |
| **AC-P2-09** | A Persian-keyboard query (`ک`/`ی`/`ه` variants) matches the correct Arabic text under `L5.codepoints`, and returns zero results under `L0.exact` **with a warning naming the profile to use** — never a silent fold | golden query pair | D2.5 | | ☐ |
| **AC-P2-10** | Profile monotonicity holds: for every golden query, `results(Lₙ) ⊆ results(Lₙ₊₁)` across all indexed profiles | property suite | D2.1, D2.5 | | ☐ |
| **AC-P2-11** | Every search hit maps to exact canonical character and token ranges; slicing the canonical text at that range and re-normalizing reproduces the matched derived substring | property suite | D2.1, D2.5 | | ☐ |
| **AC-P2-12** | The citation resolver validates every search hit; a hit whose quotation fails verification is never returned | citation suite | D2.5 | | ☐ |
| **AC-P2-13** | `quran.search_regex` rejects or bounds all 15 pathological patterns within limits; `terms_examined` and `documents_scanned` are reported; the rate limit is enforced | DoS suite | D2.5 | | ☐ |
| **AC-P2-14** | All 400 search golden queries pass, including `must_not_contain` assertions (no false positives) | golden suite | D2.5 | | ☐ |

> AC-P2-09 is deliberately paired. "The Persian query found nothing" does not satisfy it under
> `L0`: the tool must return zero results **plus the actionable warning**. A silent fold under
> `L0` fails the AC even when `L5` works.

### 1.4 Morphology import & multi-analysis discipline

| ID | Criterion | Verification | Blocks | Evidence | Status |
|---|---|---|---|---|---|
| **AC-P2-15** | Morphology import: all 18 adversarial fixtures are rejected with the **specific** expected MV rule id | adversarial suite | D2.6 | | ☐ |
| **AC-P2-16** | Alignment never modifies `quran_tokens`; `AlignmentTable` unmatched ratio is reported per surah and gates approval above threshold | integrity + import test | D2.6 | | ☐ |
| **AC-P2-17** | The schema contains **no** `is_correct`, `is_primary`, or `selected` column on analyses; a token with competing analyses returns all of them with attribution | schema test + golden case | D2.7 | | ☐ |
| **AC-P2-18** | `quran.morphology_compare` returns per-field verdicts (`identical`/`compatible_variant`/`conflicting`/`only_in_one`) and has **no** resolution or synthesis field | contract test + code review | D2.7 | | ☐ |
| **AC-P2-19** | `analysis_sources` always reports `analyses_returned` and `analyses_suppressed`; suppression is never silent under any `AnalysisPolicy` | policy matrix test | D2.7 | | ☐ |
| **AC-P2-20** | Every root, lemma, analysis, and derived form row has a provenance record at Layer B or D — never Layer A | doctor `quran.morphology.provenance` | D2.6 | | ☐ |
| **AC-P2-21** | Every Layer D row carries algorithm, version, and confidence, and cannot reach `human_verified` without a reviewer id | DB CHECK tests | D2.6 | | ☐ |
| **AC-P2-22** | Cross-dataset roots with differing spellings are **linked as reviewable suggestions**, never merged; the review queue shows the evidence before acceptance | non-merge test + queue test | D2.8 | | ☐ |
| **AC-P2-23** | All 300 root and 200 lemma golden cases pass with precision 1.00 and recall ≥ 0.99 against dataset ground truth | evaluation harness | D2.7 | | ☐ |
| **AC-P2-24** | Morphological segmentation matches the source dataset at ≥ 0.995 (import faithfulness) | evaluation harness | D2.6 | | ☐ |

> AC-P2-15/17/18 together implement I11. "The import failed" does not satisfy AC-P2-15: each
> adversarial dataset must fail with the **documented MV rule id**. A compare tool with a
> "winner" field fails AC-P2-18 even when every verdict is correct.

### 1.5 Word families & computational labelling

| ID | Criterion | Verification | Blocks | Evidence | Status |
|---|---|---|---|---|---|
| **AC-P2-25** | `quran.word_family` returns members grouped by relation class; every member carries a human-readable explanation; the 120-family curated set scores ≥ 0.95 on linguist review | family suite + linguist sign-off | D2.8 | | ☐ |
| **AC-P2-26** | Computational suggestions are **off by default**, require opt-in, carry confidence, and render with the mandatory "not verified scholarship" label in CLI, API, and tool output | default-behavior test + snapshot | D2.8 | | ☐ |
| **AC-P2-27** | A suggestion becomes `ScholarVerified` only through the review queue with a recorded reviewer, timestamp, and displayed evidence | promotion-flow test | D2.8 | | ☐ |

### 1.6 Counting discipline (anti-numerology)

| ID | Criterion | Verification | Blocks | Evidence | Status |
|---|---|---|---|---|---|
| **AC-P2-28** | Every numeric output carries a complete `CountingRules` block; two runs with identical rules produce identical numbers; changing `multi_analysis_handling` visibly changes the reported count and the rules block | determinism suite | D2.9 | | ☐ |
| **AC-P2-29** | `quran.numeric_report` contains no interpretive commentary; `quran.interval_analysis` and `quran.missing_expected_form` emit their mandatory disclaimers verbatim | snapshot tests | D2.9 | | ☐ |
| **AC-P2-30** | `quran.hapax_search` results change with profile and the output states the profile prominently — proving counts are rule-relative, not absolute | golden case | D2.9 | | ☐ |

### 1.7 Index lifecycle & consistency

| ID | Criterion | Verification | Blocks | Evidence | Status |
|---|---|---|---|---|---|
| **AC-P2-31** | Index builds are atomic: killing the build at any stage never activates a partial index; the previous generation continues serving | crash matrix | D2.10 | | ☐ |
| **AC-P2-32** | `qai quran index rebuild --all` reconstructs every index from canonical sources + manifests, and a full cold rebuild completes in < 6 minutes | timed run | D2.10 | | ☐ |
| **AC-P2-33** | Bumping any input version (corpus generation, profile version, dataset version, tokenizer version) produces the exact expected drift report from `doctor --indexes`, and every affected tool result carries a `QAI-IDX-0101` staleness warning | drift suite | D2.10 | | ☐ |
| **AC-P2-34** | Drift is never auto-repaired; `doctor` remains read-only and only suggests `qai quran morphology reindex` | read-only test | D2.10 | | ☐ |
| **AC-P2-35** | No stale cached result is served across a generation bump | cache-consistency test | D2.10 | | ☐ |

### 1.8 Architecture, tools, explainability

| ID | Criterion | Verification | Blocks | Evidence | Status |
|---|---|---|---|---|---|
| **AC-P2-36** | `quran-normalization`, `quran-search`, and `quran-morphology` have no dependency on `llm`, `embeddings`, `retrieval`, or any vector store crate | architecture test | D2.1, D2.3, D2.6 | | ☐ |
| **AC-P2-37** | All 22 Phase-2 tools conform to the §12 contract, are `ReadOnly`, populate `normalization_rules`, and produce identical `reproducibility.checksum` for identical inputs on the same generation | tool conformance suite | D2.5, D2.7, D2.9 | | ☐ |
| **AC-P2-38** | `qai quran normalize --explain` shows the rule-by-rule transformation with offset maps and flags heuristic rules | snapshot test | D2.12 | | ☐ |
| **AC-P2-39** | `POST /api/v1/quran/normalization/preview` returns the same step-by-step trace as the CLI, from the same code path | parity test | D2.11 | | ☐ |
| **AC-P2-40** | The query tokenizer and index tokenizer produce identical output for 5,000 random ayah substrings | parity test | D2.3 | | ☐ |

### 1.9 Doctor, evaluation, adapters, docs, soak

| ID | Criterion | Verification | Blocks | Evidence | Status |
|---|---|---|---|---|---|
| **AC-P2-41** | All 19 Phase-2 `doctor` checks are implemented; `quran.search.smoke` fails loudly on any tokenizer or profile regression | doctor run + mutation test | D2.13 | | ☐ |
| **AC-P2-42** | Nightly reconciliation verifies doc counts, a 1 % sampled round-trip, lexicon foreign keys, orphan generations, and MV-018 | nightly job | D2.10 | | ☐ |
| **AC-P2-43** | All latency targets in §17.1 are met and gated in CI with ≤ 20 % regression tolerance | benchmark suite | D2.5, D2.7, D2.9 | | ☐ |
| **AC-P2-44** | All hard accuracy gates in §17.2 pass; evaluation results are versioned and diffed against the previous run | evaluation harness | D2.13 | | ☐ |
| **AC-P2-45** | Two morphology adapters exist for structurally different dataset shapes, proving a new dataset needs only an adapter + manifest | adapter tests | D2.6 | | ☐ |
| **AC-P2-46** | The linguist has signed off on the normalization golden set, root/lemma golden set, and family curated set; sign-off records are stored in `docs/reviews/` | signed review records | D2.13 | | ☐ |
| **AC-P2-47** | ADRs 0201, 0203–0216 are `Accepted` with all §48 fields; ADR-0206 is recorded as `Reserved` with the decision deferred to Phase 4 | ADR lint | D2.13 | | ☐ |
| **AC-P2-48** | Docs complete: normalization spec with mapping tables, profile catalog, search cookbook, morphology adapter guide, counting-rules explainer, reindex runbook | doc review checklist | D2.13 | | ☐ |
| **AC-P2-49** | Full soak passes: rebuild all indexes → 50,000 randomized queries across all tools → `doctor` → reconciliation, with zero integrity findings, zero panics, and zero canonical-hash changes | nightly soak | D2.13 | | ☐ |
| **AC-P2-50** | The MVP linguistic workflow (§44.3) succeeds end-to-end via CLI **and** API: search without diacritics → search without spaces → select a word → inspect lemma/root/morphology/word-family → view all occurrences | scripted acceptance run | D2.11, D2.12 | | ☐ |

> A `Mismatch` citation verdict on any search hit must be wired into a **hard failure** on the
> answer path (AC-P2-12). This is the mechanism Phase 9's citation verifier reuses; Phase 2
> must not leave it advisory.

---

## 2. Integrity, Property & Safety Suites (D2.13)

The permanent regression net (PRD §35.1, §46, §58). Every subsection below must exist and be
green; a missing suite fails the gate even if the mapped AC appears satisfied by other means.

### 2.1 Golden sets (versioned, in `fixtures/quran/`)

| Set | Size | Content |
|---|---|---|
| `normalization/pairs.jsonl` | 2,000 | `{input, profile, expected_output, expected_rules_applied}` |
| `normalization/spanmaps.jsonl` | 300 | `{ayah_ref, profile, derived_range, expected_canonical_range}` |
| `search/queries.jsonl` | 400 | `{tool, input, expected_references[], expected_total, must_not_contain[]}` |
| `search/concatenated.jsonl` | 120 | Space-free queries incl. cross-ayah and Persian-codepoint cases |
| `search/regex.jsonl` | 60 | Patterns incl. 15 pathological ones expected to be rejected/bounded |
| `lexicon/roots.jsonl` | 300 | `{root, dataset, expected_token_count, expected_lemmas[]}` |
| `lexicon/lemmas.jsonl` | 200 | `{lemma, expected_forms[], expected_count}` |
| `morphology/analyses.jsonl` | 400 | `{reference, dataset, expected_segments[], expected_features}` |
| `families/curated.jsonl` | 120 | Linguist-reviewed families with expected relation classes |
| `counting/reports.jsonl` | 80 | `{target, counting_rules, expected_count}` — the anti-numerology set |
| `adversarial/morphology/*` | 18 | Faulty datasets → expected MV rule id |

**Governance:** golden sets are reviewed and signed off by the Arabic linguist (P2-T11, T12,
T92) and stored with a `reviewed_by` + `reviewed_at` header. Changing an expected value
requires a linguist review recorded in the PR — the same discipline as canonical text changes.

### 2.2 Property tests (`plan.md` §16.2)

- `SpanMap` round-trip containment — every ayah × every profile. → AC-P2-04
- `SpanMap` composition associativity — random rule chains. → AC-P2-04
- Rule idempotency (for rules declaring it) — random Unicode strings. → AC-P2-03
- No rule panics on arbitrary Unicode — fuzz, 10⁶ inputs. → AC-P2-03
- Normalized-search recall ⊇ exact-search results — every golden query. → AC-P2-10
- Profile monotonicity: `results(Lₙ) ⊆ results(Lₙ₊₁)` — all indexed profiles. → AC-P2-10
- Concatenated search finds every exact-search hit for space-free queries — golden set. → AC-P2-08
- Frequency counts equal `COUNT(*)` over the occurrence list from the same rules — all targets. → AC-P2-28
- Family membership is symmetric for symmetric relations — all families. → AC-P2-25
- Index doc_count equals relational count — after every build. → AC-P2-42
- Canonical text hashes unchanged after every build/import — MV-018, every job. → AC-P2-05

### 2.3 Integrity & safety suites (`plan.md` §16.3)

| Suite | Proves | AC |
|---|---|---|
| `tests/integrity/canonical_untouched.rs` | Building every index and importing every dataset leaves Phase-1 hashes identical (I8) | 05 |
| `tests/integrity/no_authoritative_analysis.rs` | Schema has no `is_correct`/`is_primary`; suppression is always reported (I11) | 17, 19 |
| `tests/integrity/layer_d_labeling.rs` | Every computational row has algorithm+version+confidence and cannot be `human_verified` without a reviewer (I12) | 20, 21 |
| `tests/integrity/no_llm_dependency.rs` | `quran-normalization`, `quran-search`, `quran-morphology` do not depend on `llm`/`embeddings` | 36 |
| `tests/integrity/trace_required.rs` | No `SearchHit` can be constructed without a `NormalizationTrace` (I9) | 06 |
| `tests/security/regex_dos.rs` | 15 pathological patterns bounded within limits (I16) | 13 |
| `tests/security/rate_limits.rs` | Regex and heavy-scan tools rate-limited per principal | 13 |
| `tests/recovery/index_build.rs` | Crash/cancel at each build stage never activates a partial index | 31 |
| `tests/recovery/morphology_import.rs` | Crash/cancel at each of 12 checkpoints; resume correctness | 31 |
| `tests/consistency/drift.rs` | Bumping any input version produces the exact expected doctor drift report | 33, 34 |
| `tests/consistency/cache.rs` | No stale cached result served across a generation bump | 35 |

### 2.4 Latency targets (`plan.md` §17.1; reference: 4-core laptop, cold OS cache, warm index)

| Operation | p50 | p99 | Cap |
|---|---|---|---|
| `search_exact` (single token) | < 5 ms | < 25 ms | 2 s timeout |
| `search_normalized` (indexed profile) | < 8 ms | < 40 ms | 2 s |
| `search_normalized` (ad-hoc rules) | < 60 ms | < 250 ms | 5 s |
| `search_phrase` (3 tokens, slop 0) | < 12 ms | < 60 ms | 2 s |
| `search_concatenated` (3–20 chars) | < 35 ms | < 150 ms | 3 s |
| `search_regex` (anchored) | < 80 ms | < 500 ms | 3 s hard |
| `root_search` (frequent root, 339 hits) | < 20 ms | < 90 ms | 2 s |
| `word_family` (large root, no suggestions) | < 60 ms | < 250 ms | 5 s |
| `morphology` (single token, all datasets) | < 6 ms | < 30 ms | 2 s |
| `frequency` (root, breakdown by surah) | < 40 ms | < 180 ms | 5 s |
| `collocation` (lemma, window 10) | < 250 ms | < 1.2 s | 10 s |
| `near_duplicate_passages` (full corpus) | < 2.5 s | < 8 s | 30 s |
| `normalize --explain` (single string) | < 1 ms | < 5 ms | — |
| Full index rebuild (all indexes) | < 4 min | < 6 min | — |

All are CI-gated benchmarks with a 20 % regression tolerance; exceeding it fails the build.
→ AC-P2-43, AC-P2-32.

### 2.5 Accuracy gates (`plan.md` §17.2; PRD §36.1, §47)

| Metric | Target | Gate |
|---|---|---|
| Exact search precision / recall | 1.00 / 1.00 | **Hard** |
| Diacritic-insensitive search recall (`L3`) | ≥ 0.99 | Hard |
| Diacritic-insensitive search precision | ≥ 0.95 | Soft (warn) |
| Concatenated phrase search accuracy | ≥ 0.97 | Hard |
| Cross-ayah concatenated recall | ≥ 0.90 | Soft |
| Root-search precision (vs. dataset ground truth) | 1.00 | Hard |
| Root-search recall | ≥ 0.99 | Hard |
| Lemma-search accuracy | ≥ 0.99 | Hard |
| Morphological segmentation accuracy (vs. dataset) | ≥ 0.995 | Hard (import faithfulness, not NLP) |
| Word-family explanation correctness (linguist-rated) | ≥ 0.95 | Hard on the 120-family curated set |
| Offset-mapping fidelity | 1.00 | Hard |
| Frequency-count reproducibility | 1.00 | Hard |
| Canonical-text integrity after all builds | 1.00 | Hard, zero tolerance |
| Citation resolution for search hits | 1.00 | Hard |

Evaluation runs are stored with `dataset_version`, `code_version`, `index_manifest_hash`,
and diffed against the previous run in CI; a regression on any hard gate blocks merge.
→ AC-P2-44.

---

## 3. Definition of Done

### 3.1 Per-deliverable checklist (PRD §58, §93 + `plan.md` §20)

Every Phase-2 deliverable must satisfy **all** of the following. This is the PR template
(extends the Phase-1 checklist):

- [ ] Canonical text provably unchanged (MV-018 runs in the same job that produced the artifact)
- [ ] Derived data written only to derived tables/indexes, at Layer B or D, with provenance
- [ ] Every user-visible result carries the ordered normalization rule set actually applied
- [ ] Every match maps back to exact canonical character, byte, and token ranges
- [ ] Heuristic and computational outputs are labeled as such in CLI, API, and tool payloads
- [ ] Competing analyses are preserved and attributed; suppression is counted and reported
- [ ] Every numeric output carries complete, reproducible counting rules
- [ ] Deterministic operations require no LLM (architecture test proves the absence of the dep)
- [ ] Resource limits enforced: result caps, timeouts, regex budgets, rate limits
- [ ] Derived artifacts record `corpus_generation` + all input versions; drift is detectable
- [ ] Index/dataset activation is atomic, approved, audited, and single-step reversible
- [ ] Golden-set and property tests exist and are linguist-reviewed where linguistic judgment applies
- [ ] Latency benchmarks gated in CI
- [ ] `qai doctor` can detect the failure mode this deliverable introduces
- [ ] Implemented behind an explicit interface; no cross-layer coupling (`arch-check` green)
- [ ] Typed errors implementing `Diagnostic` with remedy + next command (codes `QAI-NORM-*` / `QAI-IDX-*`)
- [ ] `cargo fmt`, `cargo clippy -D warnings`, `cargo test`, `cargo deny` all green

### 3.2 Required test suites

| Suite | Proves | AC |
|---|---|---|
| `tests/normalization/golden.rs` | 2,000 golden pairs green for all indexed profiles | 03 |
| `tests/normalization/spanmap.rs` | 5 SpanMap properties across every ayah × every profile | 04 |
| `tests/normalization/fuzz.rs` | Idempotency + associativity + no-panic on arbitrary Unicode | 03 |
| `tests/search/golden.rs` | 400 queries incl. `must_not_contain` | 07, 10, 14 |
| `tests/search/concatenated.rs` | 120 space-free cases incl. cross-ayah + Persian code points | 08, 09 |
| `tests/search/regex.rs` | 60 patterns incl. 15 pathological bounded/rejected | 13 |
| `tests/search/parity.rs` | 5,000-substring query/index tokenizer parity | 40 |
| `tests/morphology/adversarial.rs` | 18 faulty datasets rejected with specific MV ids | 15 |
| `tests/morphology/non_merge.rs` | No authoritative column; suppression always reported | 17, 19 |
| `tests/morphology/compare.rs` | Verdicts without resolution; no synthesis field | 18 |
| `tests/lexicon/roots_lemmas.rs` | 300 root + 200 lemma cases at gated precision/recall | 23, 24 |
| `tests/family/curated.rs` | 120 linguist-reviewed families with explanations | 25 |
| `tests/counting/determinism.rs` | Same rules ⇒ same number; disclaimers verbatim | 28, 29, 30 |
| `tests/tools/contract.rs` | All 22 tools: §12 contract, `ReadOnly`, checksum determinism | 37 |
| `tests/cli/normalize_snapshots.rs` | `--explain` traces, heuristic flags, preview parity | 38, 39 |
| `tests/doctor/indexes.rs` | 19 checks, read-only, smoke suite, JSON schema | 41 |
| `tests/api/search_v1.rs` | Envelope, SSE streaming, preview endpoint | 37, 39, 50 |

### 3.3 CI jobs that must be green

```text
check        -> cargo fmt --check; cargo clippy --all-targets -- -D warnings
arch         -> cargo xtask arch-check; cargo xtask migrate-check   (allowlist extended for
                quran-normalization/quran-search/quran-morphology; NO llm/embeddings edges)
test-linux   -> cargo test --workspace --all-features
test-macos   -> cargo test --workspace
test-windows -> cargo test --workspace
deny         -> cargo deny check advisories bans licenses sources
schemas      -> cargo xtask gen-schema && git diff --exit-code docs/schemas/
doctor       -> qai doctor --indexes --json | validate against docs/schemas/doctor.v1.schema.json
linguistics  -> normalization + search + morphology golden/property/adversarial suites
benchmarks   -> criterion benches; §2.4 table with 20% regression tolerance; cold rebuild < 6 min
evaluation   -> versioned accuracy gates (§2.5); hard-gate regression blocks merge
coverage     -> cargo llvm-cov; gates per §4
msrv         -> build with pinned MSRV
```

---

## 4. Coverage Gates

| Crates / modules | Line coverage | Status |
|---|---|---|
| `quran-normalization` (highest-risk pure-logic crate) | **≥ 92%** | ☐ |
| `quran-search` | **≥ 85%** | ☐ |
| `quran-morphology` | **≥ 85%** | ☐ |
| tools layer (all 22 Phase-2 tools) | **≥ 80%** | ☐ |
| `cli`, `server` (search/linguistics surfaces) | Smoke + snapshot; **no numeric gate** | ☐ |

Coverage is a floor, not a target — it does not substitute for the named suites in §3.2.
The SpanMap properties, the MV-018 integrity suite, and the non-merge tests must be green
even if coverage is above the floor.

---

## 5. Exit-Gate Ritual

A **recorded walkthrough** in which reviewers — not the implementers (the Arabic linguist +
the Phase-1 editorial reviewer) — perform the following **live on a clean machine**:

| Order | AC | Demonstration |
|---|---|---|
| 1 | AC-P2-05 | Build every index and import every dataset; show all Phase-1 canonical hashes still match |
| 2 | AC-P2-07 | Search `الرحمن` without diacritics; show 1:1, 1:3, 2:163 and the full expected set |
| 3 | AC-P2-08 | Search `بسمالله` without spaces; show 1:1 with the token-1/token-2 segmentation |
| 4 | AC-P2-09 | Search the Persian-keyboard variant; show the `L5` match and the `L0` zero-result warning |
| 5 | AC-P2-17 | Show a token with competing analyses returning all of them with attribution |
| 6 | AC-P2-22 | Show cross-dataset roots linked as suggestions in the review queue, never merged |
| 7 | AC-P2-26 | Show computational suggestions off by default with the mandatory label when enabled |
| 8 | AC-P2-28 | Run the same count under two `multi_analysis_handling` modes; show differing numbers with differing rules |
| 9 | AC-P2-31 | `SIGKILL` an index build mid-flight; show the previous generation still serving |
| 10 | AC-P2-33 | Bump a profile version; show the exact expected drift report + `QAI-IDX-0101` warnings |
| 11 | AC-P2-50 | Run the MVP linguistic workflow end-to-end via CLI **and** API |

Recording is archived and linked from `done.md`. The gate is **not** closed by a green CI run
alone — these are the criteria most likely to pass in CI while being wrong in practice.

---

## 6. Sign-Off

| Gate | Requirement | Owner | Date | Status |
|---|---|---|---|---|
| All 50 plan criteria verified | §1.1–1.9 fully ☑ | | | ☐ |
| Coverage gates met | §4 | | | ☐ |
| DoD satisfied per deliverable | §3.1 × 13 deliverables | | | ☐ |
| All required suites green | §3.2 | | | ☐ |
| All latency targets met | §2.4 | | | ☐ |
| All hard accuracy gates pass | §2.5 | | | ☐ |
| 14 ADRs accepted + 2 reserved, §48-complete | ADR-0201, 0203–0216 (0202/0206 Reserved) | | | ☐ |
| 6 migrations applied & checksummed | `migrations/sqlite/` (`0020`–`0025`) | | | ☐ |
| Linguist sign-off recorded | P2-T11/T12/T92 (`docs/reviews/`) | | | ☐ |
| Exit-gate ritual recorded | §5 (11 live steps) | | | ☐ |
| Handoff doc published | `docs/plans/handoff-p2-to-p3.md` (P2-T114) | | | ☐ |
| Swimlane X decisions owned & open | `tasks.md` §1 — P2-X01…X05 | | | ☐ |
| **Phase 2 accepted → Phase 3 unblocked** | All rows above ☑ | | | ☐ |

---

## 7. Handoff Assets Phase 3 Must Not Re-Invent

Verified as part of P2-T114. Phase 3 inherits and must build on these; re-deriving any of
them invalidates the Phase-2 version stamps.

| Asset | Location | Phase-3 usage |
|---|---|---|
| `quran_roots`, `quran_lemmas` rows | `quran-morphology` | promoted to `Root` / `Lemma` graph nodes |
| `quran_token_analyses`, `quran_morphemes` | `quran-morphology` | `HAS_LEMMA`, `HAS_ROOT`, `HAS_STEM`, `HAS_PREFIX`, `HAS_SUFFIX`, `HAS_ANALYSIS` edges |
| `quran_derivations` (Layer B) | `quran-morphology` | `DERIVED_FROM` edges with dataset provenance |
| `word_family_relations` (Layer C) | `quran-morphology` | `SAME_ROOT_AS`, `SAME_LEMMA_AS` scholar-verified edges |
| `near_duplicate_passages` output | `quran-search` | candidate `PARALLELS` / `SIMILAR_TO` edges → `review_queue` |
| Collocation / co-occurrence stats | `quran-search` | weights and candidate generation for `RELATED_TO` suggestions |
| `NormalizationPipeline` + profiles | `quran-normalization` | node key normalization; graph-query text matching |
| `SpanMap` | `quran-normalization` | mapping graph annotations to canonical spans |
| `FullTextIndex` trait + Tantivy backend | `quran-search` | reused verbatim in Phase 5 (hadith) and Phase 7 (multi-RAG lexical leg) |
| Index generation stamping + drift detection | `quran-search` | graph store adopts the identical manifest pattern |
| `review_queue` promotion flow | `application` | the mechanism for semi-automated graph annotation |
| `CountingRules` | `application::tools` | graph metrics and path counts inherit the same discipline |
| Tool contract with populated `normalization_rules` | `application::tools` | graph tools complete the contract with `graph_version` |
| Error namespaces `QAI-NORM-*`, `QAI-IDX-*` | `domain::diagnostic` registry | `QAI-GRAPH-*` reserved next |
| Golden sets + evaluation harness | `fixtures/`, `tests/` | extended with graph path-correctness sets |

**Handoff document:** `docs/plans/handoff-p2-to-p3.md`, produced by task P2-T114, listing the
above plus the frozen profile catalog with versions, the active morphology dataset and its
attribution string, the Layer D rows awaiting review, the deferred-tool list
(`README.md` §4.2) with target phases, and every place Phase 3 must record `graph_version`.

> Phase 3 must **not** invent a second normalizer, a second tokenizer, a second root
> convention, or a second counting scheme. Any proposal to do so requires an ADR that
> explains why the Phase-2 artifact cannot be reused and how existing indexes/citations
> survive the change.
