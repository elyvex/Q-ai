---
phase: 03-quran-search-linguistics
plan: 02
subsystem: testing
tags: [rust, quran-search, concatenated, skeleton, l6, golden-set, trycmd, cli, cross-ayah, oracle]

# Dependency graph
requires:
  - phase: 03-quran-search-linguistics
    provides: "03-01 SC5 canonical-display pin + alpha smoke tracer; the SC1 400-row golden-runner harness and disabling of the empty-query fallback edge at the CLI boundary"
  - phase: 02-canonical-quran-core
    provides: "search_concatenated / segment_concatenated / verify_concatenated_window services, the L6 skeleton store (ayah + 3-ayah window rows), trigram postings, and the CLI --concatenated/--cross-ayah/--max-ayah-span flags"
provides:
  - "scripts/gen_concatenated_goldens.py — independent Python oracle applying L6's full ordered rule list and modelling the surah-scoped 3-ayah window skeletons the service searches"
  - "fixtures/quran/search/concatenated.jsonl — 349-row concatenated golden set (>= 120 required; 16 cross-ayah, 10 Persian-codepoint) with explicit per-row allow_cross_ayah/max_ayah_span selectors and expected segmentation"
  - "crates/application/tests/search_goldens.rs — second fixture loader, `search_concatenated` dispatcher arm passing each row's own selectors, and fn all_concatenated_search_goldens_pass (segmentation tiling + boundary semantics)"
  - "crates/cli/tests/quran/search_s2.trycmd — SC2 CLI snapshots: the سانبل→3:3 anchor and the بلشا→3:1,3:2 cross-ayah pair"
affects: [03-03, 03-04, 03-05, 03-06, 03-07, 03-08, phase-04-graph, phase-05-result-contract]

# Actuals (#2632) — same estimateTokens scale as the plan's estimate (chars/4 over the realized diff)
actuals:
  tokens: 280
  tasks: 3
  commits: 4
plan_head_before: 27409d1a4eb61d66274b9524ba74473f31c1b89a

tech-stack:
  added: []
  patterns:
    - "Independent search oracle: a pure-Python normalizer reproduces L6's full ordered rule list and the surah-scoped 3-ayah window store, computing every expectation without the Rust stack"
    - "Per-row selector pass-through: golden rows carry their own allow_cross_ayah/max_ayah_span; the dispatcher never substitutes a literal or default"
    - "Grouped expected-segmentation assertions: one expected_segmentation list may span multiple (surah,ayah) hits and is asserted per-hit group, so it works for both ayah-level and cross-ayah rows"

key-files:
  created:
    - scripts/gen_concatenated_goldens.py
    - fixtures/quran/search/concatenated.jsonl
  modified:
    - crates/application/tests/search_goldens.rs
    - crates/cli/tests/quran/search_s2.trycmd

key-decisions:
  - "The concatenated oracle is independent of the Rust search stack: it re-implements L6's ordered rule list (N01,N11,N16,N04,N14,N03,N05,N02,N07,N06,N08,N10,N13,N09,N15,N12,N17) in Python and derives expected reference sets from substring presence over per-ayah skeletons plus surah-scoped 3-ayah window skeletons (mirroring skeletons_for_surah and migration 0014's ayah_end - ayah_start <= 2 CHECK)."
  - "Expected segmentation is computed from the same derived-offset tiling the service uses; for cross-ayah rows it spans all overlapped ayahs, so one row-level expectation proves the union tiling."
  - "Cross-ayah rows are deliberately restricted to clean boundary cases (exactly one matching window, both overlapped ayahs non-ayah-level) so the boundary parts provably tile the whole query; the generic assertion also proves ayah-level-win dedup (non-boundary refs are never boundary-labeled)."
  - "The plan named crates/cli/tests/quran/search.trycmd, which does not exist; the host-backed search segment is search_s2.trycmd, so the block lands there (see Deviations)."

patterns-established:
  - "Concatenated golden row contract: {tool, input, allow_cross_ayah, max_ayah_span, expected_references, expected_total, must_not_contain, expected_segmentation[, expected_rules_contain][, expected_boundary_refs]}"
  - "Anti-vacuity discipline: the concatenated suite asserts >= 120 rows, >= 10 cross-ayah rows, >= 10 Persian rows, and non-empty boundaries before any pass can be claimed"

requirements-completed: [REQ-quran-normalization]

# Coverage metadata (#1602)
coverage:
  - id: D1
    description: "Independent concatenated golden set: scripts/gen_concatenated_goldens.py regenerates fixtures/quran/search/concatenated.jsonl deterministically (349 rows; 16 cross-ayah; 10 Persian-codepoint; header reviewed_by: pending-linguist), including the سانبل→3:3 anchor and window-derived cross-ayah expectations."
    requirement: REQ-quran-normalization
    verification:
      - kind: other
        ref: "python3 scripts/gen_concatenated_goldens.py && test \"$(grep -c '\"tool\": \"search_concatenated\"' fixtures/quran/search/concatenated.jsonl)\" -ge 120"
        status: pass
      - kind: other
        ref: "shasum -a 256 fixtures/quran/search/concatenated.jsonl (byte-identical across two regenerations)"
        status: pass
    human_judgment: false
  - id: D2
    description: "Golden runner: crates/application/tests/search_goldens.rs loads concatenated.jsonl, dispatches via a search_concatenated arm that passes each row's own allow_cross_ayah/max_ayah_span, and asserts reference sets, totals, must_not_contain precision, segmentation tiling, disclosed L6 folds (N10), boundary labeling, ayah-level-win dedup, and cross-ayah union tiling. The existing 400-row assertion is unchanged."
    requirement: REQ-quran-normalization
    verification:
      - kind: integration
        ref: "cargo test -p application --test search_goldens -- concatenated"
        status: pass
      - kind: integration
        ref: "cargo test -p application --test search_goldens"
        status: pass
    human_judgment: false
  - id: D3
    description: "CLI-level SC2 evidence in crates/cli/tests/quran/search_s2.trycmd: `qai quran search 'سانبل' --concatenated` renders the pinned reference 3:3 with the L6.skeleton rule set, and `qai quran search 'بلشا' --concatenated --cross-ayah` renders two separate pinned hits (3:1 and 3:2) — never one merged verse."
    requirement: REQ-quran-normalization
    verification:
      - kind: integration
        ref: "cargo test -p cli --test quran"
        status: pass
    human_judgment: false
  - id: D4
    description: "No forbidden crate dependency edge introduced by the new oracle/fixture/runner (arch-check)."
    requirement: REQ-quran-normalization
    verification:
      - kind: other
        ref: "cargo run -q -p xtask -- arch-check"
        status: pass
    human_judgment: false

# Metrics
duration: 20 min
completed: 2026-09-28
status: complete
---

# Phase 3 Plan 02: Quran Search & Linguistics — Concatenated (SC2) Golden Evidence Summary

**An independent Python oracle plus a 349-row concatenated golden set (16 cross-ayah, 10 Persian-codepoint) now prove spaceless search end-to-end — segmentation tiling, N10 fold disclosure, `spans_ayah_boundary` labeling, and ayah-level-win dedup — with matching CLI snapshots for the `سانبل`→3:3 anchor and the `بلشا`→3:1,3:2 cross-ayah pair.**

## Performance

- **Duration:** 20 min
- **Started:** 2026-09-28T01:14:41Z
- **Completed:** 2026-09-28T01:35:04Z
- **Tasks:** 3/3
- **Files modified:** 4 (2 created, 2 modified)

## Accomplishments

- **G-03 closed — the missing concatenated fixture exists and is independently derived.** `scripts/gen_concatenated_goldens.py` re-implements L6.skeleton's full ordered rule list in pure Python and models exactly the store the service searches (one skeleton per ayah plus one per sliding 3-ayah window, stride 1, surah-scoped). It asserts the two builder identities the service relies on: `L6(join(texts, " ")) == concat(per-ayah L6 skeletons)` (so a window offset maps to overlapped ayah numbers) and `concat(token L6 skeletons) == ayah L6 skeleton` (so segmentation tiles). It emits 349 rows deterministically (verified byte-identical across two runs).
- **SC2 golden run green.** `cargo test -p application --test search_goldens` → 3 passed; the new `all_concatenated_search_goldens_pass` runs all 349 rows through the real service on a real built index, asserting exact reference sets, totals, `must_not_contain` precision anchors, segmentation tiling, disclosed `N10` folds, `spans_ayah_boundary` on every `expected_boundary_refs` hit, and ayah-level-win dedup. The existing 400-row assertion is untouched (still 3/3 tests green).
- **Per-row selector pass-through is asserted, not assumed.** Each row carries its own `allow_cross_ayah`/`max_ayah_span`; the dispatcher passes them through and never uses a literal or default (this is exactly what the RED commit proved failing).
- **CLI SC2 surface pinned (D-10).** `search_s2.trycmd` now snapshots the corpus-derivable anchor `سانبل` → one pinned hit at `3:3` with the `L6.skeleton` rule set, and the cross-ayah query `بلشا` → two separate pinned hits (`3:1`, `3:2`), proving a boundary-spanning fragment is never rendered as one merged verse. The `MV-018 canonical-unchanged: pass (sha256:…)` setup line remains intact.
- **No new production API symbol.** This plan adds evidence only; the SC2 service surface pre-existed.

## Task Commits

Each task was committed atomically (Task 2 is `tdd="true"`: RED → GREEN):

1. **Task 1: Author the independent concatenated-golden oracle and the 120+ case fixture** - `44d31f8` (test)
2. **Task 2 (RED): Add the failing concatenated golden runner** - `0a5d4ff` (test)
3. **Task 2 (GREEN): Wire the `search_concatenated` dispatcher arm** - `ef4b2bf` (feat)
4. **Task 3: CLI-level SC2 snapshots** - `288030b` (test)

**Plan metadata:** (this commit) (docs: complete plan)

_Note: Task 2 is a test-only `tdd="true"` task over a pre-existing service, so the RED is anchored on the selector contract rather than on new production behaviour: the RED dispatcher passed literal defaults `(false, 3)` and failed at golden 74 (`لشا`: left `{}` vs right `{3:1, 3:2}`); GREEN passed each row's own selectors and the full 349 rows went green._

## Files Created/Modified

- `scripts/gen_concatenated_goldens.py` — independent pure-Python L6 + surah-scoped 3-ayah window oracle; emits the concatenated golden JSONL.
- `fixtures/quran/search/concatenated.jsonl` — 349 `search_concatenated` rows (16 cross-ayah, 10 Persian-codepoint), header `reviewed_by: pending-linguist` (OD-12).
- `crates/application/tests/search_goldens.rs` — second fixture constant + loader, `ConcatenatedRow`/`ExpectedSegment` structs, `search_concatenated` dispatcher arm, and `all_concatenated_search_goldens_pass`; `references_of` refactored onto a shared `short_ref` helper.
- `crates/cli/tests/quran/search_s2.trycmd` — anchor + `--concatenated --cross-ayah` console snapshots.

## Decisions Made

- **Oracle independence:** the oracle recomputes expectations from first principles (Python string matching over L6 skeletons), not by shelling out to the service; its agreement with the service is therefore real evidence, not a tautology.
- **Cross-ayah rows are restricted to "clean" boundary cases** (exactly one matching window; neither overlapped ayah is an ayah-level hit) so the union of the boundary hits' segmentations provably tiles the whole query. The generic per-hit assertion still pins ayah-level-win dedup on every row.
- **`position` in `expected_segmentation` is the 0-based part index** (documented), while `canonical_token` is the 1-based canonical token position; the runner asserts `query_part` + `canonical_token` per hit group.
- **No basmala row** was added: no basmala exists in `fixtures/quran/test-edition-min`, so any such row would be underivable (the basmala segmentation case is already covered by the hand-built `search_tools.rs::concatenated_matches_with_segmentation`).
- **`queries.jsonl` is byte-unchanged** — this is an additive companion fixture.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] The plan's CLI target path does not exist**

- **Found during:** Task 3 (CLI-level SC2 snapshot)
- **Issue:** `files_modified` and Task 3 name `crates/cli/tests/quran/search.trycmd`, but that file does not exist. The host-backed CLI search evidence lives in the trycmd segments `crates/cli/tests/quran/search_s1.trycmd` (queue the import) and `crates/cli/tests/quran/search_s2.trycmd` (activate → forms rebuild → index rebuild → search).
- **Fix:** Added the two concatenated console blocks to `crates/cli/tests/quran/search_s2.trycmd`, the segment that owns the actual search commands and the `MV-018 canonical-unchanged` setup line. Snapshots were generated with `TRYCMD=overwrite` and then hand-reviewed for correctness (the literal match-count lines were made explicit to match the file's convention).
- **Files modified:** `crates/cli/tests/quran/search_s2.trycmd`
- **Verification:** `cargo test -p cli --test quran` → 13 passed.
- **Committed in:** `288030b` (Task 3 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking — non-existent path corrected to the real segment).
**Impact on plan:** None on scope. The SC2 CLI evidence lands in the segment that actually exercises the search surface; every other plan element executed as written.

## Issues Encountered

- **`cargo test -p cli --test quran` was flaky once under concurrent load.** One run during final verification reported 1 failure; three subsequent runs (including the immediately preceding run) reported 13/13 pass. The failure was not reproduced and the new snapshot cases are deterministic (pure parsing of a fixed fixture). This tree is shared with other concurrent agent sessions, and the `serve`-host tests are timing/port-sensitive — the flake is judged environmental, not causal. No test in `search_s2.trycmd` failed on any run.
- **Concurrent sessions committed to `main` mid-plan.** Three foreign commits (`92ebdaa`, `dee0c5c`, `e68c88f`, scope `01-05`) landed between the plan-start ledger base and this plan's first commit. Only this plan's four declared files were ever staged; nothing foreign was touched or reverted. Because of this, `git rev-list --count <ledger-base>..HEAD` over-counts (7) — the `commits: 4` actual is this plan's own commit count.

## User Setup Required

None — no external service configuration required.

## Known Stubs

None — this plan adds a fixture, an oracle script, a golden-runner arm, and CLI snapshots. No stub, placeholder, or hardcoded-empty value flows to any UI or result. (Grep for `TODO`/`FIXME`/`placeholder`/`coming soon`/`not available` across the new script and runner: no matches.)

## Threat Flags

None — no new network endpoint, auth path, file-access pattern, or trust-boundary schema change. The work binds untrusted spaceless query text through the existing L6 normalize→skeleton→verify path and asserts the plan's threat register: `must_not_contain` precision anchors (T-03-06), per-row cross-ayah selectors + `spans_ayah_boundary` + ayah-level-win dedup (T-03-07), bounded recall with `max_ayah_span` default 3 (T-03-08), and non-empty segmentation on every hit (T-03-09). The oracle is stdlib Python; no new packages (T-03-SC).

## Owner Gates (BLOCKED — not silently passed)

- **OD-12 (normalization catalog + named linguist, ADR-0204/0205)** — BLOCKED. The concatenated set ships as `reviewed_by: pending-linguist`; the Persian-codepoint rows are mechanical, not linguist-ratified. Closing step: name a qualified Arabic linguist, record sign-off in `docs/reviews/`, then flip ADR-0204/0205 to Accepted.
- **OD-11 (morphology dataset/license)** — BLOCKED (not touched by this plan; recorded for phase coherence).

## Next Phase Readiness

- SC2 now has a named repeatable check (`cargo test -p application --test search_goldens -- concatenated`) that a reviewer can paste and see 349 independently-derived concatenated cases pass, including 16 cross-ayah rows whose window-derived expectations are non-empty.
- SC2 also has CLI evidence with pinned canonical references (`search_s2.trycmd`), so the alpha surface (D-03) is reachable and operator-visible.
- The cross-ayah "never present a fragment as one verse" rule is asserted, not assumed: every cross-ayah row selects cross-ayah mode explicitly and asserts `spans_ayah_boundary = true` on its overlapped refs.
- Open concern for the phase: OD-12 remains the human gate; the golden header intentionally stays `pending-linguist`.

## Self-Check: PASSED

- FOUND: `scripts/gen_concatenated_goldens.py`
- FOUND: `fixtures/quran/search/concatenated.jsonl`
- FOUND: `crates/application/tests/search_goldens.rs`
- FOUND: `crates/cli/tests/quran/search_s2.trycmd`
- FOUND commit: `44d31f8` (Task 1), `0a5d4ff` (Task 2 RED), `ef4b2bf` (Task 2 GREEN), `288030b` (Task 3)
