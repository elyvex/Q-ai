---
phase: 04-quran-graph
reviewed: 2026-09-29T00:00:00Z
depth: standard
files_reviewed: 12
files_reviewed_list:
  - crates/application/src/quran_graph_store.rs
  - crates/application/src/quran_graph_build.rs
  - crates/application/src/quran_graph_annotations.rs
  - crates/application/src/quran_graph_api.rs
  - crates/application/src/quran_graph_export.rs
  - crates/application/src/quran_graph_doctor.rs
  - crates/application/src/quran_graph_tools.rs
  - crates/application/src/quran_cli.rs
  - crates/cli/src/quran.rs
  - crates/server/src/api.rs
  - crates/quran-graph/src/mem.rs
  - fixtures/quran/graph/concept-seed-v1.json
findings:
  critical: 0
  warning: 12
  info: 8
  total: 20
status: issues_found
---

# Phase 04: Code Review Report (Quran Graph)

**Reviewed:** 2026-09-29T00:00:00Z
**Depth:** standard
**Files Reviewed:** 12
**Status:** issues_found

## Summary

Reviewed the Phase 04 (plans 04-01 through 04-05) graph implementation: SQLite
store adapter, structural/word-root/annotated builders, annotation review state
machine, read services with explainability, Graph JSON export plus DOT/SVG
rendering, doctor checks plus confirmed repairs, CLI verbs, HTTP routes, and
typed tools. The code is careful overall — tombstone semantics, policy filters,
budget pre-flight, and fail-closed reads are consistently designed.

No blocking defects are reported (advisory review). The findings below are
ordered by importance. The strongest theme: **tombstone invisibility and
policy filtering are enforced on the JSON/traversal paths but bypassed on the
DOT/SVG static-rendering path** (WR-01), and the **word-root/annotated
projections that plans 04-02 built have no read path** on any surface (WR-02).
The remainder are correctness details (silent filter drops, unreported probe
bounds, unvalidated timestamps, TOCTOU on decisions, misleading repair
accounting) plus low-severity hardening notes.

## Warnings

### WR-01: DOT/SVG export renders raw sets, bypassing the tombstone policy filter

**File:** `crates/application/src/quran_cli.rs:4355-4369`
**Issue:** `render_export` runs the JSON branch through
`assemble_export` (which applies `visible_export_sets`), but the DOT/SVG
branch passes `sets.nodes`/`sets.edges` straight into `render_dot`/`render_svg`.
Rejected/superseded assertion edges — including their assertion IDs, which
`render_dot` embeds in edge labels (`quran_graph_export.rs:189-192`) — leak
into DOT/SVG output. The doctor tombstone probe (`quran_graph_doctor.rs:696-709`)
only checks traversal results and JSON bytes, so this leak is never detected.
This contradicts the phase's own D-08 invariant (tombstones hidden from
traversal **and export**).
**Fix:**
```rust
// in render_export, before the Dot/Svg branch:
let scope = AuthzScope::all_visible();
let (nodes, edges, _) = super::quran_graph_annotations::visible_export_sets(
    &sets.nodes, &sets.edges, &sets.assertions,
);
// render_dot(&nodes, &edges, &notice) / render_svg(&nodes, &edges, &seed, &notice)
```

### WR-02: Word-root and annotated projections are built but not readable on any surface

**File:** `crates/application/src/quran_graph_api.rs:749-751`
**Issue:** `GraphApiService::open` pins the structural projection only;
`FileGraphBackend::structural` (`quran_graph_api.rs:1189`) and CLI
`export_db`/`neighbors_graph_db`/etc. (`quran_cli.rs:3582,4287`) all hardcode
`STRUCTURAL_PROJECTION_ID`. `open_for_projection` (which takes a family id)
exists but has no caller on any surface, and the CLI export verb has no
`--projection` flag. Plan 04-02 builds word-root and annotated projections
that no CLI verb, HTTP route, or tool can then query — the data is
write-only, and its traversal/export behavior is unverifiable end to end.
**Fix:** Thread a `--projection` selector (default structural) through the CLI
read/export verbs, `FileGraphBackend`, and the HTTP handlers, or explicitly
record the single-projection scope as a deferred item if intentional.

### WR-03: HTTP path handler silently discards caller-supplied edge_types/direction

**File:** `crates/server/src/api.rs:1566-1577`
**Issue:** `graph_path_handler` parses `edge_types`/`direction` via
`read_options` but the path-family services expand direction-agnostic over
`EdgeFilter::any()` (`quran_graph_api.rs:1011,1044,1074`) and the explanation
reports the effective (any) filter. A caller that restricts edge types gets
unrestricted paths with no error — silent disregard of input. The same
applies to the tool path (`quran_graph_tools.rs:260-266`).
**Fix:** Reject `edge_types`/`direction` on path-mode requests with a typed
`PatternRejected` naming that path modes expand direction-agnostic, or honor
the filter in `up_to_k_paths`/`min_hops`.

### WR-04: Shallow tombstone probe truncation is unreported (bounded pass presented as full pass)

**File:** `crates/application/src/quran_graph_doctor.rs:657-659`
**Issue:** `probe_tombstones` truncates probed edge sources to
`SHALLOW_TOMBSTONE_SRC_CAP` (50) in shallow mode, but unlike the dangling
scan (`check_dangling`, which warns when bounded), `check_tombstones` reports
a full `Pass` ("N tombstoned assertion(s) invisible…") after a partial probe.
Unprobed tombstoned sources are claimed invisible without evidence.
**Fix:** Track whether `srcs` was truncated and return `Warn` naming `--deep`
in shallow mode when the cap bit, mirroring `check_dangling`.

### WR-05: Tombstone-GC retention math may silently never match (julianday vs RFC 3339 input)

**File:** `crates/application/src/quran_graph_doctor.rs:1055-1065`
**Issue:** GC eligibility is computed as
`(julianday('now') - julianday(decided_at)) > ?` where `decided_at` is a
`domain::Timestamp` rendering (`primitives.rs:167-178`, `time` crate Rfc3339:
`…T…Z` with microseconds). If SQLite's `julianday` does not parse that exact
shape it returns NULL, the comparison is NULL, and **zero rows are ever
collected** — a silently dead feature. Failure direction is fail-safe (data
retained), but operators get a passing "collected 0" report either way.
**Fix:** Verify `julianday` parses the stored shape in a test, or compute the
cutoff host-side (parse `decided_at` with `time`, compare in Rust) instead of
in SQL.

### WR-06: Quarantine repair accounting is wrong (`edges_after: 0` hardcoded)

**File:** `crates/application/src/quran_graph_doctor.rs:998-1017`
**Issue:** `repair_quarantine_dangling` records `"edges_after": 0` and reports
"(N before, 0 after)" where `before` counts only the dangling edges found.
After quarantine, all healthy edges still exist — "0 after" misstates the
table state and the audit record is factually wrong.
**Fix:** Count remaining edges per build row after deletion and report
`dangling_before` / `dangling_after: 0` / `edges_retained` distinctly.

### WR-07: `decided_at` is free-form and never validated as a timestamp

**File:** `crates/application/src/quran_graph_annotations.rs:1085-1087`
**Issue:** `decided_at_or_now` stores any caller string verbatim
(`DecideInput.decided_at`, `CorrectInput.decided_at`). Non-RFC3339 values
break the ordering assumptions (`review_queue`/`review_history` sort,
`authority` `ORDER BY decided_at` in `stage_effective_assertion_edges`) and
the GC age math in WR-05. A typo'd `--decided-at` silently corrupts
deterministic ordering.
**Fix:** Parse with `domain::Timestamp` (reject `TimestampParseError` as a
typed rejection) at the service boundary in `decide`/`correct`.

### WR-08: Review decision has a probe-then-act race (TOCTOU outside the transaction)

**File:** `crates/application/src/quran_graph_annotations.rs:1101-1116`
**Issue:** `decide` fetches the current row on one connection, drops it, then
opens a transaction and `UPDATE`s without re-checking the decision inside the
transaction. Two concurrent `accept`/`reject` calls both pass the guard;
last-writer-wins with two divergent provenance/audit rows. Same shape in
`correct` (read at `1265-1268`, write at `1336+`). Low severity in a
local-first single-operator binary, but the audit trail can then record two
conflicting decisions for one assertion.
**Fix:** Re-fetch and re-guard inside the write transaction (or add
`… WHERE id = ? AND decision IN (…)` and treat zero rows-affected as a
transition rejection).

### WR-09: `review --edition` accepts arbitrary strings with no existence check

**File:** `crates/application/src/quran_cli.rs:4489-4508`
**Issue:** `review_edition_id` returns any non-empty `--edition` verbatim.
Assertions can be authority-scoped to nonexistent editions, fragmenting the
`(projection_id, edition_id, dataset_scope)` authority keyspace and making
`review_queue`/`stage_effective_assertion_edges` scope filters silently miss
them.
**Fix:** Look up the edition id (as the default path does for the active
edition) and return a typed not-found rejection for unknown editions.

### WR-10: Annotated-seed parse checks dst membership but not src membership

**File:** `crates/application/src/quran_graph_build.rs:1285-1315`
**Issue:** `parse_annotated_seed` rejects unknown `dst` seed ids but never
checks that a `concept:`/`entity:` `src` is a known seed node. A typo'd seed
`src` passes parse and fails much later as a "dangling edge source" stage
error, misattributing a seed typo to the build pipeline.
**Fix:** Check `src` against `known` (when it carries a `concept:`/`entity:`
prefix) with the same `links[{index}]` error shape as the dst check.

### WR-11: Neighbor node-budget truncation keeps edges pointing at dropped nodes

**File:** `crates/quran-graph/src/mem.rs:225-230`
**Issue:** When `nodes` exceeds `max_nodes`, nodes are truncated but `edges`
are not pruned to the surviving set, so results (and explanations) carry
edges whose endpoints are absent from `nodes`. Consumers joining edges to
nodes see dangling references in a supposedly complete result page.
**Fix:** After truncating `nodes`, retain only edges with both endpoints in
the surviving id set (still flagged `truncated`).

### WR-12: Export human summary counts pre-filter sets while the payload is filtered

**File:** `crates/application/src/quran_cli.rs:4316-4317`
**Issue:** `render_export` reports `sets.nodes.len()`/`sets.edges.len()` (raw
pinned sets, including tombstoned edges) while the serialized JSON went
through `assemble_export`'s policy filter. With any rejected/superseded
assertions present, the message overstates what was actually exported.
**Fix:** Count from the filtered `(nodes, edges, traveling)` triple instead.

## Info

### IN-01: `escape_dot` does not escape newlines/control characters

**File:** `crates/application/src/quran_graph_export.rs:145-147`
**Issue:** Only `\` and `"` are escaped. A stable ID containing a newline
(user-supplied seed ids flow into DOT output) breaks the DOT document
structure and could inject directives. Local-operator surface only.
**Fix:** Escape `\n`/`\r` (and other control chars) in `escape_dot`.

### IN-02: `FileGraphBackend` opens a fresh database pool per request without closing

**File:** `crates/application/src/quran_graph_api.rs:1197-1205`
**Issue:** Every HTTP read constructs a `SqliteDatabase` (pool of 4) that is
dropped, never `close()`d, unlike the build paths that call `pool.close()`.
Dropped sqlx pools clean up lazily; under load this churns connections.
**Fix:** Share one `SqliteDatabase`/`GraphApiService` handle in `AppState`
(or call `close()`), consistent with the per-request rebuild-pickup comment
by reopening only the projection snapshot.

### IN-03: Tool references degrade to `quran:@:s:a` with no active edition

**File:** `crates/application/src/quran_graph_tools.rs:86-101`
**Issue:** `explanation_references` formats `quran:{slug}@{version}:…` from
possibly-empty snapshot meta, producing malformed `quran:@:1:2` references.
**Fix:** Fall back to the bare stable ID when slug/version are empty.

### IN-04: Server error bodies can carry filesystem paths

**File:** `crates/server/src/api.rs:1438-1455`
**Issue:** `graph_error_response` surfaces `summary` strings that embed
`db_path` (e.g. "no database at '…'"). Minor local info disclosure on the
HTTP surface.
**Fix:** Strip or generalize paths in server-rendered summaries.

### IN-05: Direction parsing is case-sensitive, node-kind parsing is not

**File:** `crates/application/src/quran_graph_api.rs:284-293`
**Issue:** `read_options` accepts only lowercase `both|outgoing|incoming`
while CLI pattern steps lowercase kinds before matching
(`quran_cli.rs:3381`). `Both` is a typed rejection on one surface and fine
on another.
**Fix:** Lowercase (and trim) the direction string before matching.

### IN-06: Structural collection holds a write transaction over N+1 reads

**File:** `crates/application/src/quran_graph_build.rs:60-89`
**Issue:** `collect_structural_input` opens a `write()` unit of work and runs
one `get_tokens` query per ayah before rolling back — thousands of queries
under a write lock, blocking concurrent writers for the whole collection.
**Fix:** Use a read handle, or batch token reads per edition instead of per
ayah.

### IN-07: `live_rows_refused` counts global live rows, not scope-relevant ones

**File:** `crates/application/src/quran_graph_doctor.rs:1066-1071`
**Issue:** The GC report counts all pending/accepted/disputed rows
table-wide, including rows outside any retention scope — the number does not
answer "how many live rows did this GC run consider and refuse."
**Fix:** Count live rows older than the cutoff (the actual refuse set), or
rename to `live_rows_total`.

### IN-08: File-backed export silently drops asserted edges

**File:** `crates/application/src/quran_cli.rs:4259-4276`
**Issue:** `export_file` builds `ProjectionSets` with `assertions: vec![]`,
so `assemble_export`'s allowlist drops every edge carrying an assertion id.
Documented in a comment as structural-only, but the output gives no signal
that asserted edges were filtered rather than absent.
**Fix:** Emit a warning naming the dropped asserted-edge count in the human
message when the source file contains asserted edges.

---

_Reviewed: 2026-09-29T00:00:00Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
