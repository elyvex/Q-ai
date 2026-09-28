# Phase 4: Quran Graph - Pattern Map

**Mapped:** 2026-09-28
**Files analyzed:** 19 (7 new application modules + 1 migration + 2 fixtures + 2 trycmd + 7 modifications)
**Analogs found:** 19 / 19

All analog paths verified git-tracked via `git ls-files` (non-empty output for every path named below).
No `.gsd/capabilities` mirror paths emitted. Rust is the only language; no new external
dependencies (workspace-pinned crates + `rusqlite` via existing `storage`/`storage-sqlite` seams only).

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `crates/application/src/quran_graph_store.rs` (NEW) | service (port adapter) | request-response (bounded traversal over SQLite) | `crates/application/src/quran_search_cache.rs` + `crates/quran-graph/src/mem.rs` | role-match + semantics-exact |
| `crates/application/src/quran_graph_build.rs` (NEW) | service (projection builder) | batch (staged-batch + fenced publish) | `crates/application/src/quran_index.rs` + `crates/application/src/quran_forms.rs` | exact |
| `crates/application/src/quran_graph_annotations.rs` (NEW) | service (assertion authority + review queue) | CRUD (propose → decide → supersede, tombstone) | `crates/application/src/quran_forms.rs` (single-write-tx) + `crates/quran-graph/src/model.rs` | role-match |
| `crates/application/src/quran_graph_api.rs` (NEW) | service (read services) | request-response | `crates/application/src/quran_search_api.rs` | exact |
| `crates/application/src/quran_graph_export.rs` (NEW) | service (export + static render) | transform / file-I/O | `crates/quran-graph/src/export.rs` | exact |
| `crates/application/src/quran_graph_doctor.rs` (NEW) | service (verify + repair) | batch (read-only checks; explicit repair cmd) | `crates/application/src/quran_doctor.rs` + `crates/application/src/quran_doctor_indexes.rs` | exact |
| `crates/application/src/quran_graph_tools.rs` (NEW; or fold into `quran_tools.rs`) | provider (tool backend) | request-response | `crates/application/src/quran_tools.rs` | exact |
| `migrations/sqlite/0022_quran_graph_fix.up.sql` (NEW) | migration | file-I/O (forward-only DDL) | `migrations/sqlite/0019_quran_graph.up.sql` | exact |
| `fixtures/quran/graph/concept-seed-v1.json` (NEW) | config (fixture) | file-I/O | `fixtures/quran/graph/mini-structural.json` | exact |
| `fixtures/quran/graph/annotation-goldens.json` (NEW) | config (fixture) | file-I/O | `fixtures/quran/graph/mini-structural.json` | exact |
| `crates/cli/tests/quran/graph_s1.trycmd` + `graph_s2.trycmd` (NEW) | test | batch | `crates/cli/tests/quran/search_s1.trycmd` (+ `search_s2.trycmd`) | exact |
| `crates/quran-graph/src/model.rs` (MOD: vocab + disputed) | model | transform | itself | exact |
| `crates/quran-graph/tests/conformance.rs` (MOD: parameterize) | test | request-response | itself | exact |
| `crates/cli/src/quran.rs` (MOD: `GraphAction` extension) | route (CLI surface) | request-response | itself (`GraphAction`, lines 678-737) | exact |
| `crates/application/src/quran_cli.rs` (MOD: SQLite verbs) | controller (CLI impl) | request-response | itself (`cmd_graph_*`, lines 3204-3463) | exact |
| `crates/server/src/api.rs` (MOD: graph read routes) | controller (HTTP) | request-response | itself (search routes + envelope) | exact |
| `crates/tool-registry/src/lib.rs` + `crates/application/src/quran_tools.rs` (MOD: graph tools) | provider | request-response | themselves | exact |
| `docs/08-api/quran-v1-openapi.json` (MOD: route additions) | config | file-I/O | itself (extend in same change as routes) | exact |
| `crates/application/src/lib.rs` (MOD: module wiring) | config | file-I/O | itself (lines 11-25) | exact |

## Pattern Assignments

### `crates/application/src/quran_graph_store.rs` (NEW service/adapter, request-response)

**Analog:** `crates/application/src/quran_search_cache.rs` (application-layer SQLite access pattern) + `crates/quran-graph/src/mem.rs` (traversal semantics to mirror exactly)

**Placement rule (hard):** adapter lives in `application`, NEVER in `storage-sqlite`.
`xtask/allowlist.toml` lines 20-21 vs 26-27:
```toml
[application]
workspace = { allow = ["domain", "storage", "storage-sqlite", "provenance", "audit", "jobs", "sources", "config", "observability", "quran-core", "quran-corpus", "quran-normalization", "quran-search", "quran-morphology", "quran-graph", "citations", "tools", "tool-registry"] }
[storage-sqlite]
workspace = { allow = ["storage", "domain", "quran-core"] }
```
`quran-graph` is allowed from `application`, forbidden from `storage-sqlite` — `arch-check` fails otherwise.

**Imports pattern** — copy from `crates/application/src/quran_search_cache.rs` lines 19-22:
```rust
use storage::Database as _;
use storage_sqlite::SqliteDatabase;

use super::quran_search::{SearchError, SearchOutput, SearchParams};
```
New file uses `use storage::Database as _; use storage_sqlite::SqliteDatabase;` plus
`use quran_graph::{AuthzScope, EdgeFilter, GraphStore, QueryBudgets, ...};` and
`use std::sync::atomic::AtomicBool;`. Take `db: &SqliteDatabase` (read helpers) or
`db: Arc<SqliteDatabase>` (service struct, cf. `quran_forms.rs:590`, `quran_search_api.rs:250`).

**DB access pattern (read-only)** — copy from `crates/application/src/quran_search_cache.rs` lines 60-68
and `crates/application/src/quran_morphology.rs` lines 1580-1587:
```rust
let mut uow = db.write().await.map_err(storage)?;
let row = uow.quran().cache_get(key).await.map_err(storage)?;
// ... read ...
uow.rollback().await.map_err(storage)?;   // reads ALWAYS roll back, never commit
```
Reads open a unit of work, use parameterized single-hop SQL
(`SELECT … WHERE projection_row_id = ? AND src = ? ORDER BY edge, dst_stable_id`),
then `rollback()`. Writes use `uow.commit()` (see build/annotations assignments).

**Core traversal pattern (semantics to mirror EXACTLY)** — copy from `crates/quran-graph/src/mem.rs`:
- Pre-flight first, cancel per batch (`mem.rs:140, 157, 232`):
```rust
budgets.check()?;  // out-of-range is Err(BudgetExceeded), never clamp (model.rs:328-351)
if cancel_requested(cancel) {
    return Ok(PathsResult::incomplete(Vec::new(), "cancelled before path search", 0, 0));
}
```
- Authz during expansion, never post-filter (`mem.rs:63-76`, `80-111`):
```rust
pub fn is_edge_visible(&self, edge: &GraphEdge, authz: &AuthzScope) -> bool {
    let Some(aid) = edge.assertion_id.as_deref() else { return true; };
    match self.assertions.get(aid) {
        Some(record) => {
            if !record.is_effective() { return false; }  // tombstoned hides even when listed
            authz.edge_visible(Some(aid))
        }
        None => authz.visible_assertions.is_none(),  // fail-closed under restricted scope
    }
}
```
- Deterministic ordering (`mem.rs:105-110`, `200-207`): sort adjacent by `(edge, neighbor)`;
sort result nodes by `stable_id`, edges by `(src, edge, dst)`.
- Truncation counters (`mem.rs:170-220`, `278-328`): `max_fanout` per node → truncate + reason;
`max_edges` / `max_nodes` exhaustion → `incomplete(reason)`, never empty-complete.
`NodeNotFound` for unknown stable IDs (`mem.rs:158-159`); unknown *seeds* in subgraph are
skipped → complete-empty (`mem.rs:356`).

**Error handling:** return `quran_graph::GraphError` variants (`error.rs:103-145`) with stable
`QAI-GRAPH-*` codes; map to CLI/HTTP/tool surfaces per Shared Patterns below. Never
string-interpolate stable IDs into SQL — parameterized queries only.

---

### `crates/application/src/quran_graph_build.rs` (NEW service, batch)

**Analog:** `crates/application/src/quran_index.rs` (staged index build + atomic activation) + `crates/application/src/quran_forms.rs` (single-write-tx + MV-018 in-tx check)

**Imports pattern** (`quran_index.rs` lines 14-26):
```rust
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use quran_search::{Diagnostic as IndexDiagnostic, ...};
use storage::Database as _;
use storage::error::StorageError;
use storage_sqlite::SqliteDatabase;
```
New file mirrors with `quran_graph::{build_structural, ...}` pure builders + `storage` seams.
Job params consts pattern (`quran_index.rs` lines 28-52): `QURAN_GRAPH_BUILD_KIND`, per-projection
builder-version consts (cf. `STRUCTURAL_BUILDER_VERSION` / `STRUCTURAL_PROJECTION_ID` in
`structural.rs:46-50`).

**Core build pattern — staged-batch + fenced publish** (`quran_index.rs` lines 1-12 doc + `quran_forms.rs` lines 527-553):
```rust
// 7. Single write transaction: principal, wipe, provenance, inserts,
//    MV-018 post-check, commit.
let at = domain::Timestamp::now().to_string();
crate::quran::ensure_principal(db, &params.invoked_by, &params.invoked_by, &at).await...?;
let mut uow = db.write().await.map_err(FormsError::storage)?;
uow.quran().delete_forms_for_edition(&edition_id).await...?;   // adjacency only — NEVER authority rows
match uow.provenance().insert(record).await {
    Ok(()) => {}
    Err(StorageError::Conflict) => {}   // content-addressed id: retry reuses, never conflicts
    Err(error) => return Err(...),
}
uow.quran().insert_token_forms(token_rows).await...?;          // → graph_nodes / graph_edges inserts
// MV-018 post-check INSIDE the transaction … any drift aborts the commit.
let post = verify_canonical_unchanged_in(&mut uow, &edition_id).await?;
uow.commit().await.map_err(FormsError::storage)?;
```
Graph equivalents: reserve (`graph_projections` row `building` + `graph_build_progress` cursor) →
stage batches (nodes then edges; `stage_edges` rejects dangling/unknown per `mem.rs:459-483`) →
verify (counts + no-dangling + canonical-unchanged) → flip pointer to `active` in ONE tx.
`clear_staged` drops adjacency only — "clearing a projection never discards scholarly history"
(`store.rs:206-208`). Progress checkpoints through the `jobs` leased-worker payload
(verify against `jobs/src/worker.rs` at plan time per RESEARCH A5).

**Word-root gate (D-03)** — copy from `crates/application/src/quran_morphology.rs` lines 964-974 + 900-909:
```rust
async fn require_active_dataset(db: &SqliteDatabase, capability: &str)
    -> Result<String, MorphologyToolError> {
    let mut uow = db.write().await.map_err(MorphologyToolError::storage)?;
    let active = uow.quran().active_dataset().await.map_err(MorphologyToolError::storage)?;
    uow.rollback().await.map_err(MorphologyToolError::storage)?;
    active.map(|d| d.id).ok_or_else(|| MorphologyToolError::UnavailableDataset {
        capability: capability.to_string(),   // "no active morphology dataset for {capability}; …"
    })
}
```
No active dataset → typed `QAI-MORPH-0004`-family error naming the capability, CLI exit 5 /
HTTP 404 convention. Never heuristics, never guessed roots. Structural edges carry
`assertion_id None` + `input_version` provenance in `attrs` (`structural.rs:178-188`).

**Pure builder pattern** — copy from `crates/quran-graph/src/structural.rs` lines 149-188:
pure fn over input structs, `BTreeMap`/`BTreeSet` for determinism, `GraphEdge::structural(src, edge, dst, provenance)`,
stable-ID helpers (`ayah_stable_id`, `token_stable_id`, … lines 124-147), `NEXT`-never-crosses-surah
rule, no canonical text embedded (test `no_canonical_text_embedded`, lines 327-332).

---

### `crates/application/src/quran_graph_annotations.rs` (NEW service, CRUD)

**Analog:** `crates/application/src/quran_forms.rs` (single `UnitOfWork` write tx) + `crates/quran-graph/src/model.rs` (assertion shape + tombstone)

**Core state-machine pattern** (`model.rs` lines 178-253):
```rust
pub enum AssertionDecision { Pending, Accepted, Rejected, Superseded, }  // + Disputed per Pitfall 3 decision
impl Assertion {
    pub fn is_tombstoned(&self) -> bool {
        matches!(self.decision, AssertionDecision::Rejected | AssertionDecision::Superseded)
    }
    pub fn is_effective(&self) -> bool { !self.is_tombstoned() }
}
```
Transitions: suggest (layer D requires algorithm/version/confidence per 0019 CHECK line 81) →
accept/reject/correct (reviewer + decided_at required per CHECK line 80) → correct creates a NEW
assertion with `supersedes_id`, old row → `Superseded`. Tombstoned hides immediately from
traversal AND export, retained for audit. Enforce as tests, not comments.

**Write path pattern** — single `UnitOfWork`: assertion + provenance + audit + outbox in ONE
`uow.commit()` (mirror `quran_forms.rs:536-552` above; AC-P4-11). Authority-vs-projection:
adjacency rows derive from effective assertions; rebuild never touches authority rows
(Pitfall 4: scope assertions by `(projection_id, edition_id, dataset_scope)`, never
build-row-FK-only — fixed in migration 0022).

**Error handling:** typed errors with `storage::error::Diagnostic` (code/remedy/next_command),
mirroring `MorphologyToolError` (`quran_morphology.rs:929-962`).

---

### `crates/application/src/quran_graph_api.rs` (NEW service, request-response)

**Analog:** `crates/application/src/quran_search_api.rs` (thin dispatch over services; backend trait faked in tests)

**Module doc pattern** (`quran_search_api.rs` lines 1-8):
```rust
//! Thin dispatch over [`crate::quran_search`] services. Routing lives here so
//! `server` and `cli` gain no new workspace edges (`arch-check`): both crates
//! already depend on `application`. No canonical access happens here beyond
//! what the underlying read-only search services do.
```

**Backend trait pattern** (`quran_search_api.rs` lines 73-90):
```rust
#[async_trait]
pub trait SearchBackend: Send + Sync {
    async fn search_exact(&self, args: ExactArgs) -> Result<SearchOutput, SearchError>;
    ...
}
```
New file: `GraphBackend`-style async trait (`neighbors / paths / subgraph / pattern / root_family`)
implemented by a `GraphApiService`; faked in `server` contract tests. Typed args structs
(`ExactArgs`/`NormalizedArgs` pattern, lines 21-71) carrying `QueryBudgets`, `EdgeFilter`,
`AuthzScope`, cancel flag.

**Validation pattern** (`quran_search_api.rs` lines 92-117):
```rust
pub fn reject(detail: impl Into<String>) -> SearchError {
    SearchError::Index(quran_search::IndexError::QueryRejected { detail: detail.into() })
}
```
Graph: `budgets.check()?` pre-flight + `validate_pattern(&pattern)?` before any I/O
(`pattern.rs:99-126`); bad values are typed rejections, never panics.

**Explainability payload (D-14):** extend Phase 3 D-11 ("every match reports why it matched") —
every result carries start/end nodes, ordered traversed path, edge types + per-edge provenance
(structural input-version vs assertion id + reviewer/decision/timestamp), applied filters
(budgets, edge filter, direction, authz scope descriptor — never hidden IDs), snapshot/build
identity (`projection_id` + `builder_version` + `corpus_generation`), completion
(`truncated` + `incomplete_reason`), query duration ms.

---

### `crates/application/src/quran_graph_export.rs` (NEW service, transform/file-I/O)

**Analog:** `crates/quran-graph/src/export.rs` (entire file is the contract)

**Core export pattern** (`export.rs` lines 46-82):
```rust
pub fn export_json(nodes: &[GraphNode], edges: &[GraphEdge], assertions: &[Assertion],
    manifest: &ProjectionManifest) -> serde_json::Value { ... }
pub fn export_json_with_notice(nodes, edges, assertions, manifest, notice: &ExportNotice)
    -> serde_json::Value {
    serde_json::json!({
        "format": GRAPH_JSON_FORMAT,   // "quran-graph-json-v1"
        "manifest": manifest,
        "nodes": nodes, "edges": edges, "assertions": assertions,
        "counts": { "nodes": ..., "edges": ..., "assertions": ... },
        "truncation": { "truncated": ..., "incomplete_reason": ... },
    })
}
```

**Policy filter pattern (MUST precede serialization)** (`export.rs` lines 84-109):
```rust
pub fn retain_visible<F>(nodes: &[GraphNode], edges: &[GraphEdge], keep: F)
    -> (Vec<GraphNode>, Vec<GraphEdge>)
where F: Fn(&GraphEdge) -> bool {
    (nodes.to_vec(), edges.iter().filter(|e| keep(e)).cloned().collect())
}
pub fn assertion_allowlist_predicate(visible: &HashSet<String>)
    -> impl Fn(&GraphEdge) -> bool + use<> { ... }
```
Assertions travel with edges: only assertion records for kept edges' `assertion_id`s, each
effective (test `interpretive_edges_travel_with_their_assertions`, lines 199-232); restricted
IDs absent from serialized bytes (test `restricted_evidence_never_appears…`, lines 234-256).

**Static rendering (D-13):** hand-emitted DOT (~30 lines string building) + minimal layered SVG
(by hop distance from seed, RTL labels, truncation banner). No new deps — `dot` binary is NOT
installed (RESEARCH verified). CLI `export` writes files; no browser tier.

---

### `crates/application/src/quran_graph_doctor.rs` (NEW service, batch)

**Analog:** `crates/application/src/quran_doctor.rs` (check framework) + `crates/application/src/quran_doctor_indexes.rs` (derived-layer checks)

**Check type pattern** (`quran_doctor.rs` lines 15-53):
```rust
pub enum CheckLevel { Pass, Warn, Fail, Skipped, }
impl CheckLevel {
    pub const fn as_str(self) -> &'static str { ... }  // "pass"/"warn"/"fail"/"skipped"
}
pub struct QuranDoctorCheck {
    pub id: &'static str,          // `quran.graph.*` ids for new checks
    pub status: CheckLevel,
    pub summary: String,
    pub remedy: Option<String>,
    pub next_command: Option<String>,
}
fn pass(id: &'static str, summary: String) -> QuranDoctorCheck { ... }
fn fail(id: &'static str, summary: String, remedy: &str) -> QuranDoctorCheck { ... }  // next: "qai quran validate"
```

**Read-only runner pattern** (`quran_doctor.rs` lines 95-110; `quran_doctor_indexes.rs` lines 1-14):
```rust
pub async fn run_quran_checks(db: &dyn storage::Database, deep: bool)
    -> Result<Vec<QuranDoctorCheck>, storage::error::StorageError> {
    let mut uow = db.write().await?;
    let active = uow.quran().get_active().await?;
    // ... every check rolls back; no write method invoked ...
    uow.rollback().await?;
}
```
New: `run_quran_graph_checks(db, deep)` — per-projection manifest verify (generation stamp vs
corpus, dataset versions, dependency snapshot), drift detection, tombstone invisibility probe,
dangling-edge scan. Severity policy from `quran_doctor_indexes.rs:16-20`: missing optional
subsystem → `Skipped`, stale-but-servable → `Warn`, corruption/dangling/failed tripwire → `Fail`;
fresh DB with nothing built → `Skipped`, never `Fail`. Doctor stays read-only; repair is a
separate explicit confirmed command with audit (before/after state-compare tests).

---

### `crates/application/src/quran_graph_tools.rs` (NEW provider, request-response)

**Analog:** `crates/application/src/quran_tools.rs` (tool + citation backends over reader)

**Module pattern** (`quran_tools.rs` lines 1-9):
```rust
//! Implements `tool_registry::QuranBackend` and `citations::CitationSource`
//! for [`QuranReaderService`](crate::quran_reader::QuranReaderService), plus
//! citation persistence through the `citations` table. Tools stay read-only;
//! every result carries edition identity, canonical references, and a
//! deterministic reproducibility checksum.
```

**Backend impl pattern** (`quran_tools.rs` lines 54-97): `meta_from_edition` / `backend_meta`
building `BackendMeta` (edition_version/slug/id, corpus_generation, text_hash, script, riwayah,
numbering); error mapping fns:
```rust
pub fn to_tool_error(error: ReaderError) -> ToolError {
    use storage::error::Diagnostic;
    ToolError::Backend { code: error.code().to_string(), detail: error.to_string() }
}
fn morphology_tool_error(error: crate::quran_morphology::MorphologyToolError) -> ToolError { ... }
```
Graph: `GraphBackend` impl (`quran.graph_neighbors/path/subgraph/pattern` + root-family),
`BackendMeta` gains projection identity (`projection_id`, `builder_version`,
`corpus_generation`) so `reproducibility.source_versions` pins the graph build.
Read-only `uow` + `rollback()` for meta reads (lines 59-70). No mutation tools in this phase
(D-11 scope fence).

---

### `migrations/sqlite/0022_quran_graph_fix.up.sql` (NEW migration, file-I/O)

**Analog:** `migrations/sqlite/0019_quran_graph.up.sql` (all 90 lines)

**Conventions:** forward-only, no `.down.sql` (0019 header lines 1-11); never edit 0019;
update `checksums.json`; verify with `cargo xtask migrate-check`. Table/index style to copy:
```sql
CREATE TABLE graph_edges (
  id                TEXT PRIMARY KEY,
  projection_row_id TEXT NOT NULL REFERENCES graph_projections(id),
  src_stable_id     TEXT NOT NULL,
  edge              TEXT NOT NULL,
  dst_stable_id     TEXT NOT NULL,
  assertion_id      TEXT REFERENCES graph_assertions(id),
  ...
  UNIQUE (projection_row_id, src_stable_id, edge, dst_stable_id)
);
```
0022 fixes (RESEARCH Pitfalls 1/3/4): (a) replace triple-UNIQUE with
`(projection_row_id, src_stable_id, edge, dst_stable_id, assertion_id)` NULL-safe
(partial indexes where `assertion_id IS NULL`); (b) extend `decision` CHECK with `disputed`
(or document pending→unverified mapping); (c) re-scope `graph_assertions` authority key to
`(projection_id, edition_id, dataset_scope)` rather than build-row FK. CHECK-constraint style
to copy (0019 lines 72/80-81):
```sql
decision TEXT NOT NULL CHECK (decision IN ('pending','accepted','rejected','superseded')),
CHECK (decision = 'pending' OR (reviewer IS NOT NULL AND reviewer != '' AND decided_at IS NOT NULL)),
CHECK (provenance_layer != 'D' OR (algorithm IS NOT NULL AND algorithm_version IS NOT NULL AND confidence IS NOT NULL))
```

---

### `fixtures/quran/graph/concept-seed-v1.json` + `annotation-goldens.json` (NEW fixtures, file-I/O)

**Analog:** `fixtures/quran/graph/mini-structural.json` (entire 17-line file):
```json
{
  "edition_id": "test-min",
  "input_version": "v1",
  "surahs": [{ "number": 1 }, { "number": 2 }],
  "ayahs": [
    { "surah": 1, "ayah": 1, "text": "mini one one" },
    ...
  ],
  "tokens": [
    { "surah": 1, "ayah": 1, "position": 1, "surface": "w1" },
    ...
  ],
  "divisions": [{ "id": "juz-1", "kind": "juz" }]
}
```
Same shape conventions: `test-min` edition, synthetic labels, versioned filename (`-v1`).
Concept seed: curated themes/persons/places with `synthetic_test_only`-style labeling, version
field, attribution per entry (D-04; content is owner/scholar input — agents build mechanics
only). Annotation goldens: labeled assertion rows (claim/evidence/source_location/reviewer/
decision/supersedes/layer-D fields) exercising suggest→accept/reject/correct→supersede +
tombstone invisibility.

---

### `crates/cli/tests/quran/graph_s1.trycmd` + `graph_s2.trycmd` (NEW tests, batch)

**Analog:** `crates/cli/tests/quran/search_s1.trycmd` (host-backed import-split pattern)

**Segment pattern** (`search_s1.trycmd` lines 1-19):
```
# Segment 1 of 2 (host-backed, D-13): queue the import. The harness
# synchronizes on the host-owned terminal state before segment 2 activates,
# rebuilds forms, builds the index, and searches.

```console
$ qai db migrate
migrations applied; schema version [..]

```

```console
$ qai quran import ../../fixtures/quran/test-edition-min/manifest.json
queued test-edition-min@0.1.0 import as job [..] (state: Queued)
...
```
```
`[..]` wildcard for volatile ids; split `graph_s1` (migrate → import → build → inspect)
+ `graph_s2` (neighbors → path → subgraph → export) mirroring Phase 3 `*_s1/s2` split;
registered in `crates/cli/tests/quran.rs` runner. Human output asserts truncation rendered
verbatim (`(truncated: …)`), never empty-masked.

---

### `crates/quran-graph/src/model.rs` (MODIFY model, transform)

**Analog:** itself — vocabulary block lines 67-94, decision enum lines 178-186, budgets lines 294-351.

Vocabulary extension (Pitfall 2): add `TRANSLATES`, `RELATED_TO`, `SUPPORTED_BY`,
`DISPUTED_BY` to `EDGE_VOCABULARY` (currently 15 entries, lines 73-89); update
`vocabulary_has_expected_entries` count assert (line 473: `assert_eq!(EDGE_VOCABULARY.len(), 15)`).
Do NOT re-add `PRECEDES`/`FOLLOWS` (rejected per TASK-414; test lines 470-471 pins rejection).
`is_allowed_edge` (lines 91-94) propagates to staging + `validate_pattern` automatically.
Disputed status (Pitfall 3): add `Disputed` variant to `AssertionDecision` (lines 178-186) +
`tombstone` semantics decision (`is_tombstoned`, lines 245-247 — disputed is effective, not
tombstoned) + migration CHECK update. Budget defaults stay (lines 315-326:
6 hops / 500 nodes / 10 paths / 2000 edges / 128 fanout / 5s); ranges in `check()` (lines 331-351).

---

### `crates/quran-graph/tests/conformance.rs` (MODIFY test, request-response)

**Analog:** itself — fixture + harness lines 1-80.

```rust
fn open() -> (AtomicBool, AuthzScope, QueryBudgets) {
    (AtomicBool::new(false), AuthzScope::all_visible(), QueryBudgets::default())
}
fn triangle() -> MemGraphStore { ... }  // s -> mid -> t asserted + s -> t structural (lines 42-65)
```
Parameterize the 8 tests over backends (`MemGraphStore` → generic `impl GraphStore` or
test-matrix fn), keeping the triangle/authz/tombstone/budget fixtures byte-identical;
add SQLite-only tests: determinism under insertion-order variation,
`NodeNotFound` vs truncated-empty distinction. SQLite adapter gates on green before activation.

---

### `crates/cli/src/quran.rs` (MODIFY route/CLI, request-response)

**Analog:** itself — `GraphAction` enum lines 678-737 + dispatch lines 1013-1027.

```rust
pub enum GraphAction {
    Build { #[arg(long)] out: Option<String>, },
    Inspect { #[arg(long)] file: String, },
    Neighbors { #[arg(long)] file: String, #[arg(long)] node: String,
                #[arg(long, default_value_t = 1)] hops: usize, },
    Path { #[arg(long)] file: String, #[arg(long)] from: String,
           #[arg(long)] to: String, #[arg(long, default_value_t = 4)] hops: usize, },
    RootFamily { root: String, #[arg(long, default_value_t = 25)] limit: usize, },
    Export { #[arg(long)] file: String, #[arg(long)] out: Option<String>, },
}
```
Extension: new SQLite verbs take `--db` (existing convention) + projection selection
(default: active), never `--file` (file verbs stay fixture/debug-only); add `Subgraph` +
`Pattern` verbs (absent today) + `review`/`doctor-repair` management verbs per D-11.
Dispatch shape (lines 1013-1027):
```rust
GraphAction::Build { out } => {
    application::quran_cli::cmd_graph_build(db_path, out.as_deref()).await
}
```
No HTTP mutation routes and no new mutation agent tools (D-11 scope fence).

---

### `crates/application/src/quran_cli.rs` (MODIFY controller, request-response)

**Analog:** itself — `CommandOutput` + `exit` + `open_db` (lines 21-110) and `cmd_graph_*` (lines 3204-3463).

```rust
pub mod exit {
    pub const OK: i32 = 0; pub const GENERIC: i32 = 1; pub const USAGE: i32 = 2;
    pub const VALIDATION: i32 = 3; pub const POLICY: i32 = 4; pub const NOT_FOUND: i32 = 5;
    pub const CONFLICT: i32 = 6; pub const CANCELLED: i32 = 7; pub const INTERNAL: i32 = 70;
}
pub struct CommandOutput { pub exit: i32, pub human: String, pub json: serde_json::Value, }
impl CommandOutput {
    pub fn ok(human: String, json: serde_json::Value) -> Self { Self { exit: exit::OK, human, json } }
    pub fn err(exit: i32, message: String) -> Self { ... }
}
async fn open_db(db_path: &str) -> Result<SqliteDatabase, StorageError> {
    if std::fs::symlink_metadata(db_path).is_err() {
        return Err(StorageError::MigrationRequired { at_schema: 0, required: 0 });
    }
    SqliteDatabase::new(db_path, 4, true).await
}
```
Per-verb shape to copy (`cmd_graph_neighbors`, lines 3361-3391): read/file-or-db →
`stage_graph` (or SQLite adapter open + active-projection resolve) → `QueryBudgets { max_hops: hops, ..default }`
+ `AtomicBool::new(false)` + `AuthzScope::all_visible()` → port call → `json_or_err(human, &json)`
with `(truncated)` suffix in human text and `truncated`/`incomplete_reason` in JSON;
`NodeNotFound` → `exit::NOT_FOUND`, budget pre-flight → `exit::VALIDATION`-family via
`tool_exit`-style mapping. New verbs: `Subgraph`, `Pattern`, `review` (propose/accept/reject/
correct), `doctor-repair`, `build --db` (SQLite-backed; file verbs kept debug-only).

---

### `crates/server/src/api.rs` (MODIFY controller, request-response)

**Analog:** itself — envelope + error mapping + router.

Envelope + ETag (`api.rs` lines 252-278):
```rust
fn json_response<T: Serialize>(status: StatusCode, body: &T, etag: Option<HeaderValue>, content_language: bool) -> Response {
    // content-type application/json; cache-control public, max-age=3600; optional etag
}
fn ok_envelope<T: Serialize>(data: T, meta: Meta, headers: &HeaderMap) -> Response {
    let etag = etag_for(&meta);
    if etag.as_ref().is_some_and(|current| headers.get("if-none-match") == Some(current)) {
        return StatusCode::NOT_MODIFIED.into_response();
    }
    json_response(StatusCode::OK, &Envelope { api_version: API_VERSION, data, meta }, etag, true)
}
```
Typed error→status (`api.rs` lines 831-864) — graph mapping to add beside `search_error_status`:
budget pre-flight violation → 422 + `QAI-GRAPH-0002`; unknown node → 404 + `QAI-GRAPH-0004`;
truncated result is still 200 with `truncated:true` (never error, never empty-masked).
Route registration (`api.rs` lines 1462-1470):
```rust
.route("/api/v1/quran/search/exact", post(search_exact_handler))
...
.route("/api/v1/quran/family", post(lexicon_family_handler))
```
Add `POST /api/v1/quran/graph/neighbors|path|subgraph|pattern` (+ root-family) with
`{ node/edge_types?/direction?/budgets? }` bodies → `200 Envelope { data: { nodes, edges, explanation, truncated, incomplete_reason }, meta }`.
Read-only only — no mutation routes (D-11). Extend `docs/08-api/quran-v1-openapi.json` in the
same change.

---

### `crates/tool-registry/src/lib.rs` + `crates/application/src/quran_tools.rs` (MODIFY provider, request-response)

**Analog:** themselves — `TOOL_NAMES` + attributed envelope.

```rust
// tool-registry/src/lib.rs:263-271
pub const TOOL_NAMES: [&'static str; 7] = [
    "quran.get_ayah", "quran.get_context", "quran.search", "quran.root",
    "quran.lemma", "quran.morphology", "quran.family",
];
```
Add `"quran.graph_neighbors"`, `"quran.graph_path"`, `"quran.graph_subgraph"`,
`"quran.graph_pattern"` (+ root-family) each with SemVer 1.0.0 consts (cf. `ROOT_TOOL_VERSION`).
Attributed envelope (`lib.rs:296-323`): `ToolResult { tool_name, tool_version, query,
normalization_rules, edition_id, edition_version, canonical_references, analysis_sources,
results, confidence, warnings, execution_time_ms, reproducibility }` built via
`tools::reproducibility(tool, version, &query, slug, version, map, generation)`.
`BackendMeta` gains projection identity (`projection_id`, `builder_version`,
`corpus_generation`).

---

### `docs/08-api/quran-v1-openapi.json` + `crates/application/src/lib.rs` (MODIFY config, file-I/O)

**Analog:** themselves. OpenAPI: add the four graph POST routes with the `Envelope` schema
used by the 9 search/lexicon routes — same change as `api.rs` routes, never a separate drift.
`lib.rs` wiring (`application/src/lib.rs` lines 11-25):
```rust
pub mod quran;
pub mod quran_cli;
...
pub mod quran_tools;
```
Add `pub mod quran_graph_api; pub mod quran_graph_annotations; pub mod quran_graph_build;
pub mod quran_graph_doctor; pub mod quran_graph_export; pub mod quran_graph_store;
pub mod quran_graph_tools;` in the same alphabetical block. `crates/graph` stays untouched
(empty placeholder; planner records ownership decision — zero Phase 4 code there).

---

## Shared Patterns

### Authorization — apply during expansion, never post-filter
**Source:** `crates/quran-graph/src/store.rs:82-121` (`AuthzScope`) + `crates/quran-graph/src/mem.rs:63-76` (`is_edge_visible`)
**Apply to:** `quran_graph_store.rs`, `quran_graph_api.rs`, `quran_graph_export.rs`, all read surfaces
```rust
pub struct AuthzScope { pub visible_assertions: Option<HashSet<String>>, }
// None = unrestricted; Some(set) = structural + listed assertions only.
// Tombstoned (rejected/superseded) hides even when listed; unknown IDs fail-closed under restriction.
```
Export uses `assertion_allowlist_predicate` + `retain_visible` pre-serialization (`export.rs:88-109`).

### Error Handling — typed diagnostics → stable exits → HTTP statuses → tool errors
**Sources:** `crates/quran-graph/src/error.rs:85-145` · `crates/application/src/quran_cli.rs:21-68` · `crates/server/src/api.rs:831-864` · `crates/tools/src/lib.rs:85-104`
```rust
// Graph codes (error.rs:86-101): QAI-GRAPH-0001 UnknownProjection … 0006 AuthzDenied; never renumbered.
// GraphError::Diagnostic gives code/summary/location/remedy/next_command (e.g. BudgetExceeded → "qai graph subgraph --help").
// CLI: CommandOutput::{ok(human, json) | err(exit::CODE, msg)}; human + --json parity, never panic.
// HTTP: budget pre-flight → 422; unknown node/projection → 404; truncated → 200 + truncated:true.
// Tools: ToolError::Backend { code, detail } delegating the namespaced code string.
```

### Validation — budgets pre-flight + pattern allowlist + no-query-language pin
**Sources:** `crates/quran-graph/src/model.rs:328-351` · `crates/quran-graph/src/pattern.rs:97-126` · `crates/quran-graph/tests/no_query_language.rs`
```rust
budgets.check()?;            // ranges 1..=32 hops etc.; violation is Err, never clamp
validate_pattern(pattern)?;  // 1..=8 steps, allowlisted edges, rejects raw-text-looking names
```
`stage_edges` rejects unknown predicates (`PatternRejected`) and dangling endpoints
(`BuildFailed`). `tests/no_query_language.rs` must keep passing — no SQL/Cypher/Datalog
fragments in public API names or formatted outputs; parameterized SQL only.

### Truncation Rendering — explicit partial, never absence
**Sources:** `crates/quran-graph/src/model.rs:353-375` (`TraversalResult`) · `crates/quran-graph/src/mem.rs:208-220` · `crates/application/src/quran_cli.rs:3375-3388`
```rust
// Complete-empty (truncated:false + empty) = genuinely nothing. Partial = truncated:true + non-empty incomplete_reason.
// "No path" requires truncated == false. CLI human output appends " (truncated)" + prints reason verbatim.
```

### Provenance & Refs-Only — no canonical text in graph records
**Sources:** `crates/quran-graph/src/structural.rs:12-15` + test `no_canonical_text_embedded` · `crates/quran-graph/src/model.rs:129-137`
Graph payloads carry IDs, numbers, `input_version`/assertion pointers only; quotations resolve
through the reader + `citations::verify_quotation` per hit (ADR-0111). Every non-structural edge
references an assertion with PRD §10.3 seven-field provenance; interpretive edges export only
with their assertion records (`export.rs` tests).

### Generation-Stamped Rebuildable Projections
**Sources:** `crates/application/src/quran_index.rs:1-12` · `crates/quran-graph/src/model.rs:264-292` (`ProjectionManifest`)
Manifests pin `projection_id + builder_version + edition_id + corpus_generation +
dataset_versions + dependency_snapshot + status + created_at`; activation is a pointer flip in
one tx; previous generation retained for single-step rollback; GC is explicit (`qai index gc`
pattern — graph gets explicit repair/GC commands, doctor never mutates).

### Read-Only Doctor + Explicit Repair
**Sources:** `crates/application/src/quran_doctor.rs:1-8,95-110` · `crates/application/src/quran_doctor_indexes.rs:1-20`
```rust
//! Read-only by construction: every check runs inside one unit of work that is
//! rolled back, and no write method is invoked.
```
Repair/GC are separate explicit confirmed commands with audit events; covered by
before/after state-compare immutability tests.

### Tool Result Contract (§12)
**Sources:** `crates/tools/src/lib.rs:54-83` (`ToolResult`) · `crates/tool-registry/src/lib.rs:296-323`
Every graph tool returns the full `ToolResult` envelope with `canonical_references`,
`analysis_sources`, and `reproducibility(...)` checksum pinning edition + graph build —
identical results across CLI/HTTP/tools under the versioned `Envelope`.

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| — | — | — | None. Every Phase 4 file has a tracked in-tree analog (brownfield phase; port/model/traversal/export + all surfaces landed in Phases 1-3). Closest-to-novel work is the `disputed` decision variant + multi-assertion UNIQUE (Pitfall 1/3), but both extend existing analogs (`model.rs`, `0019`) rather than needing greenfield patterns. RESEARCH.md patterns cover any residual gap. |

## Metadata

**Analog search scope:** `crates/quran-graph/src/`, `crates/quran-graph/tests/`, `crates/application/src/quran_*.rs`, `crates/cli/src/quran.rs`, `crates/cli/tests/quran/`, `crates/server/src/api.rs`, `crates/tool-registry/src/lib.rs`, `crates/tools/src/lib.rs`, `migrations/sqlite/`, `fixtures/quran/graph/`, `xtask/allowlist.toml`, `crates/application/src/lib.rs`
**Files scanned:** ~30 (15 read fully, ~10 via targeted grep, 5 via listing)
**Pattern extraction date:** 2026-09-28
**Project instructions:** `./AGENTS.md` read (Rust primary; docs numbered tree; TASK/ADR/AC naming; completion flow). No `.claude/skills/` or `.agents/skills/` directories exist — no skill indexes loaded.
