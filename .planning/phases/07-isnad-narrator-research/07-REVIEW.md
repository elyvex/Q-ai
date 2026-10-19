---
phase: 07-isnad-narrator-research
reviewed: 2026-10-17T00:00:00Z
depth: deep
files_reviewed: 27
files_reviewed_list:
  - migrations/sqlite/0031_narrators.up.sql
  - migrations/sqlite/0032_narrator_rijal_sources.up.sql
  - migrations/sqlite/0033_isnad_edges.up.sql
  - migrations/sqlite/0034_narrator_identity_assertions.up.sql
  - crates/isnad-graph/Cargo.toml
  - crates/isnad-graph/src/lib.rs
  - crates/isnad-graph/tests/conformance.rs
  - crates/quran-graph/src/model.rs
  - crates/quran-graph/src/mem.rs
  - crates/quran-graph/tests/conformance.rs
  - crates/application/Cargo.toml
  - crates/application/src/lib.rs
  - crates/application/src/isnad_graph_store.rs
  - crates/application/src/isnad_reader.rs
  - crates/application/src/isnad_tools.rs
  - crates/application/src/quran_tools.rs
  - crates/application/src/quran_graph_store.rs
  - crates/application/src/quran_graph_annotations.rs
  - crates/application/tests/isnad_identity.rs
  - crates/application/tests/isnad_reader.rs
  - crates/cli/Cargo.toml
  - crates/cli/src/lib.rs
  - crates/cli/src/isnad.rs
  - crates/tool-registry/src/lib.rs
  - crates/server/src/api.rs
  - crates/tui/src/app.rs
  - xtask/allowlist.toml
findings:
  critical: 2
  warning: 11
  info: 9
  total: 22
status: issues_found
---

# Phase 7: Code Review Report

**Reviewed:** 2026-10-17T00:00:00Z
**Depth:** deep
**Files Reviewed:** 27 (Phase 7 range `fc786ca..HEAD`, source only)
**Status:** issues_found

## Summary

Phase 7 adds the isnad/narrator engine: migrations 0031–0034, the `isnad-graph`
port crate, a `SqliteIsnadGraphStore` snapshot adapter, `IsnadReader` service,
`IsnadBackend` tools, CLI `qai isnad …`, server interception on the generic
tool route, and TUI screen stubs.

The owner-gate CHECK constraints themselves are sound and correctly exploit
SQLite's NOT-NULL/NULL semantics (`decision='pending' OR (reviewer IS NOT NULL
AND reviewer != '' AND decided_at IS NOT NULL)`); the D-layer gates are
asymmetric across the three tables but that asymmetry is *specified* in
07-01-PLAN (lines 98/100/102), so it is recorded as Info, not a defect. All SQL
in the new code is statically composed and parameterized — no injection.
Tombstoned-edge suppression is functionally correct for traversal
(`MemGraphStore::is_edge_visible` hides rejected/superseded assertions and the
loader drops `label='rejected'`), and disputed stays effective.

The real defects are in the read path and the mutation surface: the store
loader cannot represent the narrator→hadith edges the schema and its own
conformance test declare valid, the CLI `supersede` verb silently ignores a
required argument and persists an empty claim, the trace display hardcodes
"(uncertain)" while never reading the actual label, and supersession is
non-atomic. Cross-surface duplication (server re-implements the isnad dispatch
table) breaks the documented "CLI/HTTP/tools cannot drift" invariant for isnad
names.

## Narrative Findings (AI reviewer)

### Critical Issues

#### CR-01: Isnad store fails wholesale whenever an edge endpoint is not a loaded narrator node

**File:** `crates/application/src/isnad_graph_store.rs:106-140` (and `crates/quran-graph/src/mem.rs:497-508`)

**Issue:** `SqliteIsnadGraphStore::open` loads **only** `narrators` as nodes, then
hands every surviving `isnad_edges` row to `MemGraphStore::stage_edges`.
`stage_edges` rejects the whole batch if any endpoint is absent:

```rust
if !self.nodes.contains_key(&edge.src) { return Err(BuildFailed { "dangling edge source '{}'" }) }
if !self.nodes.contains_key(&edge.dst) { return Err(BuildFailed { "dangling edge destination '{}'" }) }
```

The schema has no FK on `src_stable_id`/`dst_stable_id`, the edge vocabulary
admits `NARRATES`, and Phase 7's own conformance test
(`crates/isnad-graph/tests/conformance.rs:32-51`) stages exactly
`narrator:b -NARRATES-> hadith:1` as valid. A single such row therefore makes
`open()` return `BuildFailed`, which `trace_paths` maps to a storage error, so
`isnad.trace` fails entirely instead of traversing the narrator subgraph.
D-07-03 explicitly plans to backfill `hadith_records.narrator_names_json` into
narrator/hadith edges, so this is the first thing that will break when
OD-07-01 data lands.

**Fix:** Filter edges to those whose `src`/`dst` are loaded narrator nodes
before staging (skip-with-count rather than abort), and surface the skipped
count so the trace can report incompleteness instead of a hard error:

```rust
let known: std::collections::BTreeSet<&str> =
    nodes.iter().map(|n| n.stable_id.as_str()).collect();
edges.retain(|e| known.contains(e.src.as_str()) && known.contains(e.dst.as_str()));
// or load referenced endpoints as minimal GraphNode { kind: Hadith, .. } placeholders
```

#### CR-02: `qai isnad identity supersede` ignores `--new-id` and writes an empty-claim successor

**File:** `crates/cli/src/isnad.rs:378-389`

**Issue:** The verb defines `--new-id` as a required argument, then discards it
and always supersedes with a hardcoded empty proposal:

```rust
IdentityAction::Supersede { id, new_id, reviewer } => {
    let _ = new_id;                                   // silently ignored
    let proposal = IdentityProposalParams {
        claim: serde_json::json!({}), evidence: serde_json::json!({}),
        source_location: String::new(), provenance_layer: "B".to_string(), ...
    };
    backend.backend_supersede_identity(&id, &proposal, &reviewer).await
```

Result: the predecessor is tombstoned as `superseded` and a successor
assertion with `claim_json = "{}"` is persisted as the authoritative record.
The user's stated replacement reference is dropped, the audit chain carries no
claim, and the CLI still prints "superseded {id} (superseded) …" as if the
operation succeeded meaningfully. (07-01-PLAN:186 moved the service signature to
`new_proposal: IdentityProposalParams`; the CLI flag was never reconciled.)

**Fix:** Remove `--new-id` and accept the replacement claim/evidence the same way
`propose` does, or fail loudly when `--new-id` is supplied:

```rust
IdentityAction::Supersede { id, claim, evidence, reviewer, .. } => {
    let proposal = IdentityProposalParams { claim: parse_json(&claim), evidence: parse_json(&evidence), .. };
```

### Warnings

#### WR-01: Trace display hardcodes "(uncertain)" and never reads the edge label

**File:** `crates/cli/src/isnad.rs:213-225`

**Issue:** The formatter reads `provenance_layer` and `confidence` from
`attrs` but emits a literal `" (uncertain)"` for every edge, so a
`label = 'confirmed'` link is rendered as uncertain (and a `rejected` link can
never reach here — it is filtered, but confirmed is mislabelled). 07-01-PLAN:174
requires the suffix only "until confirmed", and D-07 encodes the label as the
uncertainty signal.

**Fix:**
```rust
let label = attrs.get("label").and_then(|v| v.as_str()).unwrap_or("uncertain");
let suffix = if label == "confirmed" { "" } else { " (uncertain)" };
```

#### WR-02: `isnad.narrator` silently swallows rijal-source errors

**File:** `crates/application/src/isnad_tools.rs:47`

**Issue:** `self.reader.get_rijal_sources(id).await.unwrap_or_default()` turns any
storage failure into "zero scholars", indistinguishable from a genuinely empty
result — the opposite of the repo's "explicit incompleteness" principle.

**Fix:** Propagate with `?`, or attach a `warnings` entry when the secondary
read fails:
```rust
let (rijal, rijal_error) = match self.reader.get_rijal_sources(id).await {
    Ok(v) => (v, None),
    Err(e) => (Vec::new(), Some(e.to_string())),
};
```

#### WR-03: `supersede_identity` is not atomic — orphan tombstones and unlinked successors

**File:** `crates/application/src/isnad_reader.rs:260-283`

**Issue:** Three independent writes run without a transaction:
`decide(old, "superseded")` → `propose_identity(new)` → `UPDATE … SET supersedes_id = ?`.
If step 2 or 3 fails, the old assertion is already tombstoned with no successor
(or the successor exists but is not chained). T-07-04 (audit chain) is the
mitigation this breaks.

**Fix:** Wrap the three statements in one `sqlx` transaction on `write_pool()`
(and set `supersedes_id` in the INSERT rather than a follow-up UPDATE).

#### WR-04: Six `.expect("checksum")` panic sites in a request path

**File:** `crates/application/src/isnad_tools.rs:83,129,178,225,305,352`

**Issue:** `research_checksum(...)` returns `Result<ContentHash, HashingError>`;
every call site `.expect("checksum")`. `rg` shows this pattern exists **nowhere
else in the workspace** — the isnad tools are the only place a hashing failure
panics (inside an axum handler / CLI task) instead of returning a typed error.
Reachability is low (serde_json cannot carry non-finite floats), but the
contract for this route is "typed errors, never panics".

**Fix:** `.map_err(|e| ToolError::Backend { code: "QAI-ISNAD-0001".into(), detail: e.to_string() })?`.

#### WR-05: Decision tool results report the tool name as the decision

**File:** `crates/application/src/isnad_tools.rs:317`

**Issue:** `make_decision_result` emits `json!({ "id": …, "decision": name })`
where `name` is the **tool name** — so `isnad.identity_accept` returns
`"decision": "isnad.identity_accept"`, HTTP clients that key on `decision` never
see `"accepted"`, and the value differs from the DB's `decision` column. The
loopback CLI works around it by re-reading `decided_at`/`reviewer` in
`fetch_decided`, which is a symptom of the same confusion.

**Fix:** Pass the decision value explicitly: `("isnad.identity_accept", "accepted")`.

#### WR-06: `confidence` is never range-validated

**File:** `crates/application/src/isnad_reader.rs:195-215`

**Issue:** The D-layer gate only checks `confidence.is_none()`. A caller may
persist `confidence = 1e9` or `-1.0`; the DB CHECK (0034:22-23) enforces
`NOT NULL` only, and `ConfidenceBand::of` then bands `1e9` as `High` and `-1.0`
as `Low`. For a system whose contract is that confidence is *attributed*
metadata, an out-of-domain score is a mislabelled research claim.

**Fix:** Reject non-finite / out-of-`0.0..=1.0` values before insert:
```rust
if let Some(c) = confidence && !(c.is_finite() && (0.0..=1.0).contains(&c)) {
    return Err(IsnadReaderError::InvalidInput("confidence must be within 0.0..=1.0".into()));
}
```

#### WR-07: Server duplicates the isnad dispatch table, breaking the "cannot drift" invariant

**File:** `crates/server/src/api.rs:520-631,649` vs `crates/application/src/quran_tools.rs:878-922`

**Issue:** `run_isnad_dispatch` is a hand-copied second name→backend table for the
eight isnad tools, bypassing `dispatch_registered_tool`. The OpenAPI
description of `POST /api/v1/quran/tool/{name}` (and the module comment at
`api.rs:759-767`) state the route "dispatches through the shared name-to-tool
table the CLI `quran tool` verb uses, so tool identity and `research_checksum`
cannot diverge between surfaces" — that is now false for isnad names, and the
two tables have already drifted in shape (the server path builds
`IsnadToolBackend::new(reader)` per request; the shared path calls
`ToolRegistry::isnad_backend`, which is never set in the serve arm).

**Fix:** Set the isnad backend on the shared registry in the serve arm and let
`dispatch_registered_tool` handle `isnad.*`; delete `run_isnad_dispatch`.

#### WR-08: Mutating tools exposed on an otherwise read-only, unauthenticated route

**File:** `crates/server/src/api.rs:649` + `crates/application/src/quran_tools.rs:898-922`

**Issue:** `POST /api/v1/quran/tool/{name}` previously fronted read-only
capabilities (graph/lexicon/search tools; the D-11 comment at `api.rs:1775`
frames this family as read-only). Phase 7 routes
`isnad.identity_accept|reject|dispute|supersede` — all INSERT/UPDATE — through
it with no permission gate of any kind (`rg` finds no permission/deny layer in
`tool-registry`). Impact is bounded by the loopback-only bind enforced at
`crates/cli/src/lib.rs:325`, but AGENTS.md states tools use "deny-by-default
permissions", and the mutation capability is undocumented in
`docs/08-api/quran-v1-openapi.json`.

**Fix:** Either gate the `isnad.identity_*` names behind an explicit
opt-in/permission check, or document the mutation surface and its reviewer
requirement in the OpenAPI spec.

#### WR-09: TUI isnad screens render unrelated content

**File:** `crates/tui/src/app.rs:571,573`

**Issue:** `Screen::IsnadNarrator` and `Screen::IsnadIdentity` both render
`render_annotation_review(frame, &self.annotations)` — the *annotation rows*
list — and `Screen::IsnadTrace` renders a static hint. Both are reachable via
the command palette (`crates/tui/src/palette.rs:41-43`) and `Screen::from_name`,
so "Isnad narrator" opens a screen showing annotation-review data. 07-03-PLAN:59
required a narrator record screen (stable-id header, primary name, variants,
rijal rows) and a chain-trace surface with the uncertainty footer; only the
palette/`Screen` wiring was done.

**Fix:** Point the narrator screen at a narrator-specific renderer (or a hint),
and do not alias it to the annotation-review widget.

#### WR-10: CLI silently coerces malformed `--claim` / `--evidence` JSON into a string

**File:** `crates/cli/src/isnad.rs:431-433`

**Issue:** `parse_json` falls back to `Value::String(s.to_string())` on parse
failure, so `--claim 'not json'` is accepted and persisted as the JSON string
`"\"not json\""` in `claim_json`, with a `(pending)` success message. There is no
validation that a claim is an object, which the research record implies.

**Fix:** Return a usage error for unparseable input instead of downgrading the type.

#### WR-11: No narrator slug/id validation, despite threat T-07-05 claiming it

**File:** `crates/isnad-graph/src/lib.rs:24-52`

**Issue:** `NarratorId::new(slug)` interpolates the slug verbatim and
`FromStr` only checks the `narrator:` prefix and non-empty length. 07-01-PLAN:223
lists "slug validation" as the T-07-05 mitigation; no such validation exists
anywhere in the phase (`NarratorId` is the only constructor). Storage is
parameterized so this is not injection, but it is an unvalidated trust-boundary
value that becomes a primary key and a canonical reference.

**Fix:** Constrain the slug charset/length in one place and use it from both
`new` and `from_str`:
```rust
fn validate_slug(slug: &str) -> Result<(), String> {
    if slug.is_empty() || slug.len() > 128
        || !slug.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return Err(format!("invalid narrator slug `{slug}`"));
    }
    Ok(())
}
```

### Info

#### IN-01: `ISNAD_TOOL_NAMES` is dead and isnad tools stay out of `tool_names()`

**File:** `crates/tool-registry/src/lib.rs:1819` (`[&str; 6]`, never referenced)

07-01-PLAN:176 allows "extend `TOOL_NAMES` **or** `ISNAD_TOOL_NAMES`", but the
implementation did neither for `TOOL_NAMES`: `ToolRegistry::TOOL_NAMES`
(`lib.rs:752`, still 17 entries) omits every `isnad.*` name, so
`tool_names()` and the `assert_eq!(registry.tool_names().len(), 17)` assertions
never see them, while the CLI/HTTP dispatchers happily serve them. The constant
that does exist is unused and lists 6 of the 8 implemented tools
(`identity_dispute`, `identity_supersede` absent). **Fix:** either fold the
isnad names into `TOOL_NAMES` or delete `ISNAD_TOOL_NAMES`.

#### IN-02: Dead / duplicated public API in `isnad-graph`

**File:** `crates/isnad-graph/src/lib.rs:89-103,105-125`

`IsnadLinkLabel` (+`as_str`) has no caller anywhere. `ConfidenceBand` duplicates
`application::verse_link::ConfidenceBand` (identical thresholds) and is reached
only from tests; production never bands isnad confidence. **Fix:** delete
`IsnadLinkLabel`; either consume `ConfidenceBand` where confidence is displayed
(the CLI trace currently formats a bare float) or drop it.

#### IN-03: `isnad-graph` declares two unused dependencies

**File:** `crates/isnad-graph/Cargo.toml:8-12` (and the matching allowlist entries `xtask/allowlist.toml:143,341`)

`domain` and `uuid` are declared and allowlisted but `src/lib.rs` uses neither
(`uuid` is used by `application::isnad_reader`). **Fix:** remove both from the
manifest and the allowlist.

#### IN-04: Store module doc contradicts its own load behavior

**File:** `crates/application/src/isnad_graph_store.rs:1-5` vs `106-125`

The header claims "Tombstoned edges (rejected/superseded assertions, or label
'rejected') are excluded at load time"; the code drops only `label='rejected'`
and defers assertion tombstoning to `is_edge_visible` at traversal time.
Traversal is correct, but `MemGraphStore::dump()` (export surface) would emit
tombstoned edges. **Fix:** correct the doc, or filter assertions-tombstoned
edges at load if `dump()` is ever exposed for isnad.

#### IN-05: D-layer gate asymmetry across the three tables

**Files:** `migrations/sqlite/0032:21`, `0033:21`, `0034:22-23`

0032 requires only `confidence`; 0033 requires only `algorithm` +
`algorithm_version`; 0034 requires all three. This is what 07-01-PLAN specified,
but the consequence is that a layer-`D` `isnad_edges` row is legal with no
confidence, and the CLI trace then prints an empty confidence for a derived
link while labelling it uncertain. **Fix:** align 0033 with 0034 (add
`AND confidence IS NOT NULL`) when the edge write path is built.

#### IN-06: Rijal reads have no deterministic order

**File:** `crates/application/src/isnad_reader.rs:137`

`get_rijal_sources` has no `ORDER BY`, yet its result is embedded verbatim in
the `isnad.narrator` / `isnad.rijal` payloads that feed `research_checksum`.
Row order is not guaranteed by SQLite, so the advertised reproducibility
checksum is not stable across plans/versions. **Fix:** `ORDER BY scholar_name, id`.

#### IN-07: Placeholder manifest values

**File:** `crates/application/src/isnad_graph_store.rs:145-157`

`manifest()` returns `id == projection_id`, `edition_id: ""`, `created_at: ""`,
`corpus_generation: 0`, `manifest: null` — no build identity that a consumer
could pin. **Fix:** populate or document as intentionally opaque.

#### IN-08: Unknown-tool errors misname the tool

**Files:** `crates/server/src/api.rs:627-630`, `crates/application/src/quran_tools.rs` (default arm)

An unknown `isnad.*` name (and every unknown tool in the shared dispatcher)
returns `ToolError::InvalidInput { tool: "quran.tool" }` rather than the
requested name. **Fix:** use `other`/`name`.

#### IN-09: Shared-dispatcher isnad arms are unreachable in the serve path

`crates/application/src/quran_tools.rs:878-922` is only reached when
`AppState.isnad` is `None`, in which case it returns "isnad backend not
configured" — so the eight arms are effectively dead in the serve process and
duplicate WR-07's table. **Fix:** see WR-07.

---

_Reviewed: 2026-10-17T00:00:00Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: deep_
