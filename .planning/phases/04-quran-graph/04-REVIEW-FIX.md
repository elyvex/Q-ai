---
phase: 04-quran-graph
fixed_at: 2026-10-13T00:00:00Z
review_path: /Users/ali/dev/rust/Q-ai/.planning/phases/04-quran-graph/04-REVIEW.md
iteration: 2
findings_in_scope: 12
fixed: 3
skipped: 9
status: partial
---

# Phase 04: Code Review Fix Report

**Fixed at:** 2026-10-13T00:00:00Z
**Source review:** `.planning/phases/04-quran-graph/04-REVIEW.md` (full-phase stale-refresh, 2026-10-13)
**Iteration:** 2 (iteration 1: three fixes; iteration 2: targeted re-review)

**Summary:**
- Findings in scope: 12 (WR-01…WR-12; `critical_warning` scope, 0 critical; IN-01…IN-10 excluded)
- Fixed: 3 (WR-10, WR-11, WR-12 — each a real defect with a small, safe fix plus a regression test verified to fail without the fix)
- Skipped: 9 (WR-01…WR-09 — each explicitly dispositioned below as an accepted limitation / investigated-no-defect with rationale; code left untouched)

20 of the review's 22 findings are carried verbatim across prior review
cycles on a verified-closed phase (verification 4/4, `04-VERIFICATION.md`),
so stable code was deliberately not churned to "fix" accepted limitations:
every skipped warning records its rationale here and in the REVIEW.md
disposition lines, and remains re-reviewable.

## Fixed Issues

### WR-10: Annotated-seed parse checks dst membership but not src membership

**Files modified:** `crates/application/src/quran_graph_build.rs`, `crates/application/tests/graph_build.rs`
**Commit:** `ed9c367`
**Applied fix:** Exactly the reviewer's suggestion — when a link `src`
carries a `concept:`/`entity:` prefix it is checked against the seed node
set with the same `links[{index}]` `BuildFailed{collect}` shape as the dst
check. Bare `ayah:` srcs stay exempt (they are not seed nodes).
**Verification:** new `annotated_seed_typo_src_fails_at_parse_not_as_dangling_edge`
regression test — verified to FAIL without the fix and pass with it;
`graph_build` 7/7 green.

### WR-11: Neighbor node-budget truncation keeps edges pointing at dropped nodes

**Files modified:** `crates/quran-graph/src/mem.rs`, `crates/quran-graph/tests/conformance.rs`, `crates/cli/tests/quran/graph_s2.trycmd`
**Commits:** `1cd0e3d` (fix + test), `c0b0321` (snapshot)
**Applied fix:** Exactly the reviewer's suggestion — after truncating
`nodes`, retain only edges with both endpoints in the surviving id set,
still flagged `truncated`. Single-point root-cause fix: `SqliteGraphStore`
stages into the same in-memory traversal (`quran_graph_store.rs:236-244`),
so both backends are covered by construction. The `graph_s2.trycmd:57`
snapshot pinned the defective output (`1 node(s), 7 edge(s)`) and was
updated to the corrected page (`1 node(s), 0 edge(s)` — same truncation
flag, reason, and surviving-node quotation line; the `->` detail lines are
canonical-reader quotations per returned node, not edges).
**Verification:** new backend-generic
`truncated_neighbors_keep_no_edges_to_dropped_nodes` conformance test —
verified to FAIL without the fix, green on mem + sqlite with it;
`quran-graph` full suite green; `cli --test quran` 17/17 green after the
one-line snapshot update.

### WR-12: Export human summary counts pre-filter sets while the payload is filtered

**Files modified:** `crates/application/src/quran_cli.rs`
**Commit:** `3cac160`
**Applied fix:** The JSON branch of `render_export` now counts nodes/edges
from the assembled (policy-filtered) document instead of the raw pinned
sets, so message and payload agree by construction (traveling count already
came from the document). DOT/SVG counts deliberately unchanged: those paths
render the raw sets (WR-01, accepted), so raw counts correctly describe what
they write — filtering the counts while rendering raw would understate.
**Verification:** new
`quran_cli::export_count_tests::json_export_message_counts_filtered_payload_not_raw_sets`
unit test — verified to FAIL without the fix (`exported 2 nodes, 2 edges`
overstates the 1-edge payload) and pass with it; lib `quran_cli` tests 7/7
green.

## Skipped Issues

### WR-01: DOT/SVG export renders raw sets, bypassing the tombstone policy filter — accepted

Static local rendering is the documented Phase-4 scope (deferrals ledger
D-13: "static local rendering only"); the authoritative envelope (Graph
JSON v1) is policy-filtered with byte-absence proven by `graph_export`
tests. Re-plumbing the DOT/SVG path changes export bytes on a
verified-closed phase for a local-operator-only surface. Follow-up: apply
`visible_export_sets` to the static views alongside the Phase 5 explorer
work. Code left untouched.

### WR-02: Word-root and annotated projections are built but not readable on any surface — deferred

Threading a `--projection` selector through CLI/HTTP/tools is a feature
(multi-projection scope), not a bug fix. The shipped contract is the
structural read scope; word-root production waits on the OD-11 dataset
block and annotated stays mechanics-only per OD-12. Defer to the phase that
ratifies the datasets. Code left untouched.

### WR-03: HTTP path handler silently discards caller-supplied edge_types/direction — accepted

Rejecting `edge_types`/`direction` on path modes is a breaking contract
change for existing callers, and honoring them needs an algorithm change in
`up_to_k_paths`/`min_hops`. The effective (`any`) filter is already reported
in every `Explanation.applied_filters`, so no caller is misled about what
ran. Follow-up: typed `PatternRejected` or filter-honoring traverse. Code
left untouched.

### WR-04: Shallow tombstone probe truncation is unreported — accepted

The shallow `Pass` reports the bounded sample the phase documents (`--deep`
upgrades to the full scan); flipping it to `Warn` changes doctor output
shape asserted by `graph_doctor` tests and CLI snapshots on a verified
phase. The `--deep` rerun guidance already exists via the `Inconclusive`
path. Follow-up: cap-bit `Warn` mirroring `check_dangling`. Code left
untouched.

### WR-05: Tombstone-GC retention math may silently never match — investigated, no defect

Premise does not reproduce: SQLite's `julianday` parses the stored RFC 3339
`…T…Z`-with-microseconds shape (checked directly —
`julianday('2026-10-13T00:00:00.123456Z')` returns non-NULL), the failure
direction is fail-safe (rows retained, never wrongly collected), and the
`graph_doctor` GC cases are green. No code change.

### WR-06: Quarantine repair accounting (`edges_after: 0`) — accepted

`0 after` reads as dangling-after (the quarantine target set is empty
post-repair); renaming the audited `records` keys breaks stable audit
consumers and the `graph_doctor` repair assertions on a verified phase.
Follow-up: `dangling_before` / `dangling_after: 0` / `edges_retained`
triple. Code left untouched.

### WR-07: `decided_at` is free-form and never validated — accepted

Strict `Timestamp` validation at `decide`/`correct` is a breaking input
change on a stable CLI (existing rows and tests carry hand-written
`decided_at` strings); ordering holds for the `now_rfc3339`-written values
the CLI actually produces. Follow-up with a legacy-row migration. Code left
untouched.

### WR-08: Review decision probe-then-act race — accepted

Local-first single-operator binary: no concurrent writers exist in practice,
and the fix (in-transaction re-guard / conditional `UPDATE`) needs
concurrency test infrastructure the suite does not have. Last-writer-wins
with divergent audit rows is acceptable at research scale. Follow-up for
multi-operator phases (11/12). Code left untouched.

### WR-09: `review --edition` accepts arbitrary strings — accepted

Rejecting unknown editions is a breaking behavior change (authority scoping
currently permits forward-declared editions); scope filters fail closed
(miss, never leak). Follow-up: edition lookup with typed not-found
rejection. Code left untouched.

## Verification (where it ran)

All gates ran in the main checkout (`/Users/ali/dev/rust/Q-ai`, branch
`agent-06-01-hadith-tracer`; no isolated worktree per orchestrator scope).

- `cargo test -p quran-graph` — all suites green (lib 22, conformance 20 incl. new WR-11 test, plus export/model suites)
- `cargo test -p application --test graph_build --test graph_export --test graph_doctor --test graph_neighbors --test graph_review` — 7/10/5/2/8 green (incl. new WR-10 test)
- `cargo test -p application --test graph_paths --test graph_tools --test graph_explain` — 4/11/6 green
- `cargo test -p application --lib quran_cli` — 7/7 green (incl. new WR-12 test)
- `cargo test -p cli --test quran` — 17/17 green (incl. one-line WR-11 snapshot correction)
- `cargo test -p server --test graph` — 10/10 green
- `cargo test -p tool-registry` — green; `cargo test -p cli --test parity_cross_surface` — 5/5 green
- `cargo run -p xtask -- arch-check` — OK (no forbidden edges)
- `cargo run -p xtask -- migrate-check` — OK (26 migrations, checksums stable)
- `cargo fmt --all -- --check` — clean
- `cargo clippy -p application -p quran-graph --all-targets` — no new warnings on touched files (two pre-existing warnings elsewhere: `domain` lib, macro-crate reference)

## Notes

- Each fix was committed atomically per finding (`ed9c367` WR-10,
  `1cd0e3d` WR-11, `c0b0321` WR-11 snapshot, `3cac160` WR-12); every new
  regression test was verified to fail without its fix.
- The WR-11 snapshot correction is the only snapshot change: `graph_s2`
  asserted the defect (1 node carrying 7 edges). No other snapshot, OpenAPI,
  migration, or fixture file was touched.
- Re-review (iteration 2, targeted): the three fixes confirmed at HEAD via
  the fail-without/pass-with tests plus the blast-radius suites above; the
  nine accepted warnings and ten infos are unchanged — no new findings
  introduced, so no iteration 3 was spent.

---

_Fixed: 2026-10-13T00:00:00Z_
_Fixer: the agent (gsd-code-fixer role, fix mode)_
_Iteration: 2_
