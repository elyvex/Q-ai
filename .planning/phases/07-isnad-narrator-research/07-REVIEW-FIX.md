---
phase: 07-isnad-narrator-research
fixed_at: 2026-10-17T16:35:00Z
review_path: .planning/phases/07-isnad-narrator-research/07-REVIEW.md
iteration: 1
findings_in_scope: 13
fixed: 13
skipped: 0
status: all_fixed
---

# Phase 7: Code Review Fix Report

**Fixed at:** 2026-10-17T16:35:00Z
**Source review:** .planning/phases/07-isnad-narrator-research/07-REVIEW.md
**Iteration:** 1

**Summary:**

- Findings in scope (Critical + Warning): 13
- Fixed: 13
- Skipped: 0

**Fix scope:** `critical_warning` — the 2 Critical and 11 Warning findings. The 9
Info findings (IN-01…IN-09) were out of scope for this run and are listed under
"Intentionally deferred" below.

**Note on continuity:** CR-01/02 and WR-01/02/04/05/06 were already committed
atomically by an earlier, interrupted fixer pass; this run verified them, then
committed WR-03, WR-07, WR-08, WR-09, WR-10, WR-11 (plus three `style` commits
required to keep the workspace `fmt`/`clippy -D warnings` gates green).

## Fixed Issues

### CR-01: Isnad store fails wholesale whenever an edge endpoint is not a loaded narrator node

**Files modified:** `crates/application/src/isnad_graph_store.rs`, `crates/application/src/isnad_reader.rs`
**Commit:** `e33b942`
**Applied fix:** Edges whose `src`/`dst` are not loaded narrator nodes are now
skipped-with-count at load (`skipped_edges`) instead of aborting `open()`.
`trace_paths` surfaces the skipped count as `incomplete_reason` so a partial
trace reports incompleteness rather than a hard storage error.

### CR-02: `qai isnad identity supersede` ignores `--new-id` and writes an empty-claim successor

**Files modified:** `crates/cli/src/isnad.rs`
**Commit:** `1324657`
**Applied fix:** `IdentityAction::Supersede` now takes the same
`--claim`/`--evidence`/`--source-location`/`--provenance-layer`/`--algorithm`/
`--algorithm-version`/`--confidence`/`--reviewer` arguments as `propose` and
passes a real `IdentityProposalParams` to `backend_supersede_identity`; the
discarded `--new-id` flag was removed.

### WR-01: Trace display hardcodes "(uncertain)" and never reads the edge label

**Files modified:** `crates/cli/src/isnad.rs`
**Commit:** `08f339b`
**Applied fix:** The trace formatter reads `attrs.label` (default `"uncertain"`)
and emits the suffix only when the label is not `"confirmed"`.

### WR-02: `isnad.narrator` silently swallows rijal-source errors

**Files modified:** `crates/application/src/isnad_tools.rs`
**Commit:** `78d9aa5`
**Applied fix:** The secondary rijal read now maps an error to an explicit
`warnings` entry (`rijal sources unavailable [<code>]: <err>`) plus an empty
list, so a storage failure is distinguishable from a genuinely empty result.

### WR-03: `supersede_identity` is not atomic — orphan tombstones and unlinked successors

**Files modified:** `crates/application/src/isnad_reader.rs`, `crates/application/tests/isnad_identity.rs`
**Commit:** `129b92e`
**Applied fix:** Supersession is now one `sqlx` transaction: the predecessor
tombstone `UPDATE` and the successor `INSERT` (with `supersedes_id` bound in
the INSERT, never a follow-up `UPDATE`) commit together or not at all.
Pre-write validation runs before the first write. Two regression tests assert
that a refused supersede writes nothing and that the successor is chained at
insert time.

### WR-04: Six `.expect("checksum")` panic sites in a request path

**Files modified:** `crates/application/src/isnad_tools.rs`
**Commit:** `a267fda`
**Applied fix:** Every `research_checksum(...)` call now propagates via
`.map_err(|e| ToolError::Backend { code: "QAI-ISNAD-0001", detail })` instead
of `.expect("checksum")` — typed errors, never a panic.

### WR-05: Decision tool results report the tool name as the decision

**Files modified:** `crates/application/src/isnad_tools.rs`
**Commit:** `457794c`
**Applied fix:** `make_decision_result` now takes an explicit `decision`
argument; `isnad.identity_accept` reports `"decision": "accepted"`,
`_reject` → `"rejected"`, `_dispute` → `"disputed"`.

### WR-06: `confidence` is never range-validated

**Files modified:** `crates/application/src/isnad_reader.rs`
**Commit:** `1a04e04`
**Applied fix:** `validate_confidence` rejects non-finite and out-of-`0.0..=1.0`
values before insert (faithful to the `attributed metadata` contract; the DB
CHECK only enforces NOT NULL).

### WR-07: Server duplicates the isnad dispatch table, breaking the "cannot drift" invariant

**Files modified:** `crates/server/src/api.rs`, `crates/cli/src/lib.rs`, `crates/application/tests/isnad_reader.rs`, `crates/server/tests/{api,citation_open,graph}.rs`, `crates/cli/tests/parity_cross_surface.rs`
**Commit:** `a3c7a03`
**Applied fix:** The serve arm now wires the isnad backend onto the shared
reader registry (`tools.set_isnad_backend(...)`), and `run_isnad_dispatch` plus
the `AppState.isnad` intercept were deleted. `isnad.*` now dispatches through
the same `dispatch_registered_tool` name→tool table as the CLI. A new
integration test drives `isnad.narrator` through the shared dispatcher both
unwired (fails closed) and wired (serves the narrator with a checksum).

### WR-08: Mutating tools exposed on an otherwise read-only, unauthenticated route

**Files modified:** `crates/tool-registry/src/lib.rs`, `crates/server/src/api.rs`, `crates/cli/src/lib.rs`, `docs/08-api/quran-v1-openapi.json`, `crates/server/tests/{api,citation_open,graph}.rs`, `crates/cli/tests/parity_cross_surface.rs`
**Commit:** `9923fb6`
**Applied fix:** Added `tool_registry::is_mutating_isnad_tool` (the five
`isnad.identity_*` writers). The generic HTTP tool route refuses them by default
with a typed `403 / QAI-ISNAD-0326`; the serve arm opts in via
`policy.tool_execution_default == "allow"` (`AppState.allow_isnad_mutations`,
deny-by-default). Read-only isnad tools and all `quran.*`/`hadith.*`/`tafsir.*`
tools are unaffected. The OpenAPI description documents the gate, the mutation
surface, and the reviewer requirement. Covered by a server test (403 by default,
read-only not gated, opt-in lifts the gate) and a registry unit test.

### WR-09: TUI isnad screens render unrelated content

**Files modified:** `crates/tui/src/app.rs`
**Commit:** `6d2561f`
**Applied fix:** `Screen::IsnadNarrator` / `Screen::IsnadTrace` /
`Screen::IsnadIdentity` no longer alias the annotation-review rows. Each renders
an honest, actionable empty state (pointing at the equivalent `qai isnad …`
verb) since no narrator-selection / trace-input / identity-queue state exists in
the cockpit yet.

### WR-10: CLI silently coerces malformed `--claim` / `--evidence` JSON into a string

**Files modified:** `crates/cli/src/isnad.rs`
**Commit:** `7a35b15`
**Applied fix:** `parse_json` now returns `Result`; an unparseable `--claim` or
`--evidence` produces a usage error (exit code `USAGE`) instead of being
persisted as a JSON string via `Value::String`. Applied to both `propose` and
`supersede`.

### WR-11: No narrator slug/id validation, despite threat T-07-05 claiming it

**Files modified:** `crates/isnad-graph/src/lib.rs`
**Commit:** `729f6af`
**Applied fix:** Added `validate_slug` (1..=128 chars of `[A-Za-z0-9_-]`) used by
both `NarratorId::new` (now fallible) and `FromStr`, so a slug can no longer
become a primary key / canonical reference outside that charset. New test covers
empty, whitespace, separators, traversal, and over/at-length inputs.

## Gate / hygiene commits (not review findings)

Three `style(07)` commits were required because earlier phase-7 commits had left
the workspace `fmt` / `clippy -D warnings` gates red; they change no behavior:

- `d3bb116` — rustfmt `application` files left unformatted by the WR-02 fix.
- `a8994e2` — rustfmt `cli/src/isnad.rs` left unformatted by the WR-01 fix.
- `c3a4cc1` — repair a rustfmt regression in `hadith-ingestion/src/validate.rs`
  introduced by the phase-7 `style` commit `fb98e8a`.
- `e5ad5ec` — clear pre-existing clippy warnings in phase-7 files
  (`isnad-graph/tests/conformance.rs` unused imports + needless borrows;
  `server/src/api.rs` test `unused_mut`) so `clippy --workspace --all-targets
  -- -D warnings` passes.

## Intentionally deferred

The 9 **Info** findings were outside this run's fix scope
(`critical_warning`), and were not in the requested priority set. Recommended
follow-ups, in rough value order:

- **IN-06** `get_rijal_sources` has no `ORDER BY` → the advertised
  reproducibility checksum is not stable. One-line fix when the read path is
  next touched.
- **IN-01** `ISNAD_TOOL_NAMES` is dead and lists only 6 of 8 isnad tools;
  `ToolRegistry::TOOL_NAMES` still omits every `isnad.*` name. Fold the isnad
  names into `TOOL_NAMES` (or delete the constant).
- **IN-04** `isnad_graph_store` module doc contradicts load behavior
  (assertion tombstoning is deferred to traversal, not applied at load).
- **IN-09 / IN-08 / IN-02 / IN-03 / IN-05 / IN-07** — see REVIEW.md; all
  cosmetic/dead-code/doc/placeholder items.

## Verification

- `cargo test --workspace` → **1317 passed, 0 failed** (run in the main
  checkout; `workflow.use_worktrees` is `false` for this repo, so no isolated
  worktree was created — the numbers are reproducible from the working tree).
- `cargo fmt --all --check` → clean workspace-wide.
- `cargo clippy --workspace --all-targets -- -D warnings` → clean.
- Per-fix: affected crate tests re-run and passing
  (`isnad-graph`, `application` isnad_identity/isnad_reader, `tui`,
  `server`, `cli --test parity_cross_surface`, `tool-registry`).

**Known flaky test (pre-existing, unrelated to these fixes):**
`cli --test quran :: serve_hosts_the_worker_and_shuts_down_joined` waits up to
90s for a seeded job to reach a terminal state and is timing-sensitive under
parallel workspace load; it failed once and passed on subsequent runs.

---

_Fixed: 2026-10-17T16:35:00Z_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 1_
