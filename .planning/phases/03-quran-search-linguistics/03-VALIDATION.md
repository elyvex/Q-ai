---
phase: "3"
slug: "quran-search-linguistics"
# status lifecycle: draft (seeded by plan-phase) → validated (set by validate-phase §6)
# audit-milestone §5.5 distinguishes NOT-VALIDATED (draft) from PARTIAL (validated + nyquist_compliant: false) (#2117)
status: validated
nyquist_compliant: true
wave_0_complete: true
created: "2026-09-26"
validated: "2026-09-28"
---

# Phase 3 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Audited by `validate-phase` on 2026-09-28 against the eight PLAN/SUMMARY files and a
> live re-run of every automated command in this map. No automatable gaps remain; the
> Manual-Only rows are non-automatable owner gates (OD-11 / OD-12 / D-08), not test gaps.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` / `#[tokio::test]`, `proptest` (properties), `trycmd` (real CLI snapshots), `tempfile` (isolated real-SQLite state) |
| **Config file** | none — the Cargo workspace + existing test targets are the configuration; trycmd cases live in `crates/cli/tests/quran/*.trycmd` |
| **Quick run command** | `cargo test -p quran-normalization -p quran-search -p quran-morphology --lib && cargo test -p application --test quran_identity` |
| **Full suite command** | `cargo test --workspace --no-fail-fast` |
| **Gate commands** | `cargo run -q -p xtask -- migrate-check` · `cargo run -q -p xtask -- arch-check` |
| **Estimated runtime** | focused targets seconds; full workspace suite minutes |

**Phase-gate commands re-run green during this audit (2026-09-28):**

```
cargo test -p application --test alpha_smoke                 # 2 passed
cargo test -p application --test canonical_display_identity  # 6 passed
cargo test -p application --test search_goldens              # 3 passed (400 + concatenated)
cargo test -p quran-search --test search_parity              # 3 passed
cargo test -p application --test counting                    # 9 passed
cargo test -p application --test morphology_import           # 15 passed
cargo test -p application --test family_goldens              # 1 passed
cargo test -p application --test doctor_indexes              # 7 passed
cargo test -p application --test search_latency              # 2 passed
cargo test -p application --test quran_tools                 # 9 passed
cargo test -p application --test alpha_e2e                   # 1 passed
cargo test -p cli --test quran                               # 14 passed
cargo test -p server                                         # 22 + 4 passed
cargo test -p storage-sqlite --test quran                    # 11 passed
cargo test -p storage --lib quran                            # 4 passed
cargo test -p tool-registry                                  # 6 passed
cargo test -p quran-morphology --lib license_evidence        # 4 passed
cargo run -q -p xtask -- arch-check                          # OK
cargo run -q -p xtask -- migrate-check                       # OK (21 migrations)
```

The 03-01 SUMMARY recorded `cargo test -p cli --test quran` as a pre-existing red
(the concurrent 01-04 enqueue-only import refactor had left `trycmd` snapshots stale).
That item was refreshed by 03-02 and is now **green (14 passed)** — the historical gap is closed.

---

## Sampling Rate

- **After every task commit:** the focused target for the touched seam (`cargo test -p <crate>`), plus `cargo fmt --all -- --check` and `cargo clippy -p <crate> -- -D warnings` when Rust changes.
- **After every plan wave:** `cargo test --workspace` + `cargo run -q -p xtask -- migrate-check && cargo run -q -p xtask -- arch-check`.
- **Before `/gsd-verify-work`:** full workspace suite green, both requirements backed by named automated commands, and the OD-11/OD-12 owner gates recorded as BLOCKED.
- **Max feedback latency:** focused targets (seconds); full suite (minutes).

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 3-01-01 | 01 | 1 | REQ-quran-normalization | T-03-01 / T-03-03 / T-03-05 | Empty/whitespace query rejected at CLI with typed `USAGE` (never all-match/panic); every hit carries trace + canonical span + verified quotation; `limit` clamped | integration | `cargo test -p application --test alpha_smoke` | ✅ | ✅ green |
| 3-01-02 | 01 | 1 | REQ-quran-normalization | T-03-02 | Displayed canonical slice byte-identical to the reader-resolved canonical row at every hit span, all five modes; canonical hash unchanged across search | integration (SC5) | `cargo test -p application --test canonical_display_identity` | ✅ | ✅ green |
| 3-01-03 | 01 | 1 | REQ-quran-normalization | — (doc) | Rule-catalog doc matches implemented N18–N22 / reserved N23–N24; golden + parity checks pinned | golden + unit/property + lint | `cargo test -p application --test search_goldens && cargo test -p quran-search --test search_parity && cargo clippy -p quran-normalization -- -D warnings` | ✅ | ✅ green |
| 3-02-01 | 02 | 2 | REQ-quran-normalization | T-03-06 / T-03-07 | Deterministic concatenated oracle; ≥120 rows, cross-ayah + Persian-codepoint rows carry their own selectors | oracle/fixture | `python3 scripts/gen_concatenated_goldens.py && test "$(grep -c '"tool": "search_concatenated"' fixtures/quran/search/concatenated.jsonl)" -ge 120` | ✅ | ✅ green |
| 3-02-02 | 02 | 2 | REQ-quran-normalization | T-03-06 / T-03-07 / T-03-09 | Concatenated golden runner asserts reference sets, `must_not_contain` precision, segmentation tiling, cross-ayah union | integration (golden) | `cargo test -p application --test search_goldens -- concatenated` | ✅ | ✅ green |
| 3-02-03 | 02 | 2 | REQ-quran-normalization | T-03-07 | CLI `qai quran search --concatenated` renders pinned refs; cross-ayah renders separate hits, never one merged verse | CLI snapshot (SC2) | `cargo test -p cli --test quran` | ✅ | ✅ green |
| 3-03-01 | 03 | 2 | REQ-quran-linguistics | T-03-10 | Lexicon aggregation uses bound parameters only (no string-concatenated SQL); `StorageUnavailable` defaults | unit + integration | `cargo test -p storage --lib quran && cargo test -p storage-sqlite --test quran` | ✅ | ✅ green |
| 3-03-02 | 03 | 2 | REQ-quran-linguistics | T-03-11 / T-03-12 / T-03-13 | Every frequency report embeds `CountingRules` (datasets, handling, profile, checksum); no guessed dataset name; bounded aggregate scalars | integration (matrix) | `cargo test -p application --test counting` | ✅ | ✅ green |
| 3-03-03 | 03 | 2 | REQ-quran-linguistics | T-03-11 | CLI root/lemma frequency + `freq` alias; unknown mode → usage (exit 2), no active lexicon → typed exit 5 | CLI snapshot | `cargo test -p cli --test quran` | ✅ | ✅ green |
| 3-04-01 | 04 | 2 | REQ-quran-linguistics | T-03-14 / T-03-15 / T-03-17 | Typed CHECK-domain relations via `FamilyMember::new`; synthetic rows flagged; no automatic merge/preferred flag | integration | `cargo test -p application --test morphology_import -- family_relations_explained` | ✅ | ✅ green |
| 3-04-02 | 04 | 2 | REQ-quran-linguistics | T-03-16 | `affix_search` fails closed with typed `QAI-MORPH-0004` instead of silent empty; L7 heuristic stays labeled | integration | `cargo test -p application --test morphology_import -- affix` | ✅ | ✅ green |
| 3-04-03 | 04 | 2 | REQ-quran-linguistics | T-03-14 / T-03-15 | Repeatable curated family golden (154 ≥ 120) covers all seven typed kinds, each explained; header stays `pending-linguist` | integration (golden) | `cargo test -p application --test family_goldens` | ✅ | ✅ green |
| 3-05-01 | 05 | 3 | REQ-quran-linguistics | T-03-18 | Top-level `qai quran family` / `lemma` with non-empty kind/id usage guard before any read | CLI snapshot | `cargo test -p cli --test quran` | ✅ | ✅ green |
| 3-05-02 | 05 | 3 | REQ-quran-linguistics | T-03-18 / T-03-19 / T-03-20 / T-03-21 | Typed, attributed, bounded HTTP lexicon routes; no empty 200 (404 `QAI-LEX-0004` / 400 coded) | server contract | `cargo test -p server` | ✅ | ✅ green |
| 3-06-01 | 06 | 4 | REQ-quran-linguistics | T-03-22 / T-03-24 | Activation gate rejects absent/malformed/non-permissive license evidence before promotion; pure typed parser | unit + build | `cargo test -p quran-morphology --lib license_evidence && cargo build -p application` | ✅ | ✅ green |
| 3-06-02 | 06 | 4 | REQ-quran-linguistics | T-03-23 | Operator-supplied license status/json/evidence flags replace hardcoded values; unreadable file → typed usage error | CLI snapshot + build | `cargo test -p cli --test quran && cargo build -p cli` | ✅ | ✅ green |
| 3-06-03 | 06 | 4 | REQ-quran-linguistics | T-03-22 / T-03-24 / T-03-25 | Machine-readable license matrix valid with capture fields; fail-closed/pass activation tests; no canonical/hash path touched | integration | `cargo test -p application --test morphology_import -- license_gate` | ✅ | ✅ green |
| 3-07-01 | 07 | 5 | REQ-quran-linguistics | T-03-27 | `doctor --indexes` reports all 19 checks with stable ids, stays read-only; injected drift → `QAI-IDX-0101` warn | integration (soak) | `cargo test -p application --test doctor_indexes` | ✅ | ✅ green |
| 3-07-02 | 07 | 5 | REQ-quran-normalization | T-03-28 | Budget artifact codifies ADR-0207 p99 ≤ 150 ms; fixture bounds asserted never loosened; full-corpus rows flagged OD-11-dependent | integration | `cargo test -p application --test search_latency` | ✅ | ✅ green |
| 3-07-03 | 07 | 5 | REQ-quran-linguistics | T-03-26 / T-03-29 | Registered tools return attributed envelopes built from verified hits/analyses; empty selector typed invalid-input; no-dataset typed backend error | unit + integration | `cargo test -p tool-registry && cargo test -p application --test quran_tools` | ✅ | ✅ green |
| 3-08-01 | 08 | 6 | REQ-quran-linguistics | T-03-30 / T-03-31 | Owner gates recorded as explicit BLOCKED with closing step; nothing marked closed; source of truth unchanged | doc artifact | `test -s docs/05-followups/phase-03-owner-gates.md && grep -q 'OD-11' docs/05-followups/phase-03-owner-gates.md && grep -q 'OD-12' docs/05-followups/phase-03-owner-gates.md && grep -q 'licenses/qac' docs/05-followups/phase-03-owner-gates.md` | ✅ | ✅ green |
| 3-08-02 | 08 | 6 | REQ-quran-linguistics | T-03-33 | Deferred scope (UI/Phase 5, counting tail incl. hapax_search, legacy tail, L8, graph, editions/multi-RAG) recorded; progress artifacts updated | doc artifact | `test -s docs/05-followups/phase-03-deferrals.md && grep -q 'hapax_search' docs/05-followups/phase-03-deferrals.md && grep -q 'OD-11' docs/06-progress/status.md` | ✅ | ✅ green |
| 3-08-03 | 08 | 6 | REQ-quran-normalization + REQ-quran-linguistics | T-03-31 / T-03-32 | Full D-03 alpha: normalize→forms→index→all five modes→lexicon→frequency/distribution/co-occurrence, trace+span+byte-identical quotation, attribution, complete `CountingRules` | integration (alpha) | `cargo test -p application --test alpha_e2e && cargo test -p cli --test quran` | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

All Wave 0 gaps declared in `03-RESEARCH.md` § Validation Architecture are now satisfied
and green in the per-task map above:

- [x] `fixtures/quran/search/concatenated.jsonl` — 349 rows (16 cross-ayah, Persian-codepoint), deterministic oracle (covers SC2 / G-03). → 3-02-01
- [x] `crates/application/tests/search_goldens.rs` — extended with the `search_concatenated` arm + tiling assertions (covers SC2). → 3-02-02
- [x] `crates/application/tests/counting.rs` — `root_lemma_frequency` + `multi_analysis_modes` (covers SC4 / G-01,G-07). → 3-03-02
- [x] `crates/application/tests/morphology_import.rs` — license-gate fail-closed/pass + full family-builder tests (covers D-07 / G-04, SC3 / G-05). → 3-04-01, 3-06-03
- [x] `crates/application/tests/family_goldens.rs` + curated family set — 154 synthetic families (≥120) (stays `reviewed_by: pending-linguist` until OD-12). → 3-04-03
- [x] `crates/cli/tests/quran/` — `family_s1/s2`, `search_s2`, `counting_graph_s1/s2` snapshots (covers SC2/SC3 CLI surfaces / G-02). → 3-02-03, 3-03-03, 3-05-01
- [x] Per-hit canonical-slice byte-identity test (covers SC5 / Q5). → 3-01-02
- [x] Framework install: **none** — Rust, Cargo, SQLx, trycmd, tempfile, proptest already present.

---

## Manual-Only Verifications

These are non-automatable scholarly/legal owner gates (OD-11, OD-12/D-08), not missing
tests. They are the only reason a surface result may not be treated as ground truth.

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Normalization catalog + golden-set linguistic correctness (OD-12) | REQ-quran-normalization | No qualified Arabic linguist has ratified the N01–N24 catalog/tagset; ADR-0204/0205 remain Draft. Golden header stays `reviewed_by: pending-linguist`. | Name a qualified linguist, record sign-off in `docs/reviews/`, then flip ADR-0204/0205 to Accepted. |
| Family relation tagset / root conventions / alignment semantics (OD-12, D-08) | REQ-quran-linguistics | ADR-0210/0215 BLOCKED; the 154-family golden set is synthetic (`reviewed_by: pending-linguist`). | Linguist ratifies tagset, roots, alignment and the seven typed relation semantics; record sign-off in `docs/reviews/`. |
| Counting convention — what "one occurrence" means under multi-analysis | REQ-quran-linguistics | ADR-0211 Draft; counts (1 / 127 / 64) are code-asserted on a synthetic lexicon, not linguist-ratified. | Linguist/owner ratifies `single_source` / `all_analyses` / `one_vote_per_token` semantics. |
| QAC dataset license + morphology dataset selection (OD-11) | REQ-quran-linguistics | No `licenses/qac/` capture made; ADR-0203 Option A vs B not ratified. Gate validates evidence *structurally*, cannot certify terms. | Owner captures license evidence and ratifies ADR-0203; the `qac` matrix entry stays `pending_license_review` until then. |
| Full-corpus performance budgets (OD-11-dependent) | REQ-quran-normalization | Full-corpus p50/p99 rows in `fixtures/quran/performance/budgets.json` cannot execute on the synthetic fixture. | Activate a licensed corpus, then re-run `cargo test -p application --test search_latency`; assert concatenated p99 ≤ 150 ms (ADR-0207). |
| Live `qai serve` HTTP lexicon smoke (SC3/SC4) | REQ-quran-linguistics | Performed manually in 03-05 (404 `QAI-LEX-0004` / 400 `QAI-LEX-0002`); the live-server path is not part of CI. | Start `qai serve` on a migrated temp db; POST `/api/v1/quran/family` and `/api/v1/quran/count/{root,lemma}-frequency`; assert typed error statuses for no-dataset / empty selector / unknown mode. |

*If none: "All phase behaviors have automated verification."*

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies (23/23 tasks map to a green command)
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references (all eight research-declared gaps closed)
- [x] No watch-mode flags
- [x] Feedback latency acceptable (focused targets seconds; full suite minutes)
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-09-28 (validate-phase audit; all automatable requirements green,
manual-only rows are owner gates OD-11 / OD-12 / D-08)

---

## Validation Audit 2026-09-28

| Metric | Count |
|--------|-------|
| Automated gaps found | 0 |
| Resolved | 1 (historical `cargo test -p cli --test quran` red from 03-01, refreshed by 03-02 — now 14 passed) |
| Escalated | 0 |
| Manual-only (owner-gated) | 6 |
| Tasks with green automated verify | 23 / 23 |
| Requirements automatable-covered | 2 / 2 (`REQ-quran-normalization`, `REQ-quran-linguistics`) |

**Per-requirement verdict:**

- **REQ-quran-normalization — COVERED.** SC1 (golden suite + 5,000-substring parity), SC2 (concatenated golden + CLI), SC5 (per-hit byte identity + canonical-hash stability across all five modes), performance budget, and the alpha e2e all re-run green.
- **REQ-quran-linguistics — COVERED (functional).** SC3 (family/lemma CLI + HTTP routes, typed-unavailable behavior), SC4 (root/lemma frequency + multi-analysis matrix + `CountingRules`), license gate, doctor soak, and attributed tools all re-run green. Scholarly correctness remains manual-only (OD-12/D-08) and licensed-dataset applicability remains manual-only (OD-11).

**No test files were generated by this audit** — every declared behavior already had a
real, passing automated test. No production source was modified.