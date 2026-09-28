# Phase 4: Quran Graph - Research

**Researched:** 2026-09-28
**Domain:** Quran knowledge graph — typed port, SQLite adjacency projection, bounded traversal, annotation review, Graph JSON export
**Confidence:** HIGH (brownfield; port/model/traversal/export already implemented and read this session; gaps identified against PRD/AC text)

## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Treat Phase 4 as an evidence-driven brownfield gap-closure phase (mirrors Phase 1 D-01 / Phase 2 D-01 / Phase 3 D-04): map each of the four success criteria to current code + at least one repeatable check, preserve working implementations, and create implementation work only for missing behavior or weak evidence. — **Reversibility:** reversible — a planning-boundary choice.
- **D-02:** Content scope is **structural + word-root + concepts/entities + translation links**. Structural (edition/surah/ayah/token/division, `CONTAINS`/`NEXT`) and word-root (root/lemma/word-form/token/ayah) always; concept/topic + named-entity nodes as annotated projections; `TRANSLATES` edges to existing translation editions with the ADR-0112 layer separation intact (refs-only, never text). Tafsir/hadith/isnad links are deferred to roadmap Phases 6–7. — **Reversibility:** costly — projection tables, the CLI/HTTP/tool surface, and export shape are consumed by Phase 5 UI and later graph phases.
- **D-03:** Word-root projection builds **only from active, attributed morphology records** (synthetic fixture in tests; user-supplied records via the Phase 3 morphology import path when present; typed "no dataset active" behavior otherwise — never heuristics, never guessed roots). OD-11 (dataset/license) and OD-12 (linguist) remain recorded BLOCKED owner gates; this phase does not re-litigate them. Exact build mechanics are agent discretion within invariants I11/I12/I13 + alignment (Phase 3 D-09). — **Reversibility:** costly — the license/attribution metadata and activation gate feed provenance, derived-index manifests, and export.
- **D-04:** Concept/entity nodes come from a **curated, versioned seed list** (major themes, persons, places) committed as a fixture, extended by user/scholar additions through the annotation workflow. Nothing claims scholarly authority beyond its attribution.
- **D-05:** Interpretive cross-reference edges (`PARALLELS`, `CONTRASTS_WITH`, `EXPLAINS`, `RELATED_TO`, plus `SUPPORTED_BY`/`DISPUTED_BY`) are **in scope as annotated edges** with full provenance + review history, per PRD §10.6 and legacy D4.5. — **Reversibility:** costly — edge types and the assertion/evidence shape are a published contract consumed by export and later phases.
- **D-06:** **Full review queue**: researchers manually create typed edges; algorithmically suggested edges queue with supporting evidence shown, then accept / reject / correct with reviewer + timestamp recorded. No computational suggestion becomes a verified edge without an explicit human decision. — **Reversibility:** costly — the review lifecycle shapes the annotation service, CLI commands, and audit events.
- **D-07:** Disagreement and history via **verification statuses + supersession**: edges carry `verification_status` (unverified/verified/disputed/superseded/rejected); corrections create new assertions and supersede old ones rather than overwriting; `SUPPORTED_BY`/`DISPUTED_BY` are first-class edges. Rebuilding the graph projection never discards human annotations or review history. — **Reversibility:** costly — the history shape is consumed by traversal filters, export, and later isnad/scholarly phases.
- **D-08:** Rejected/superseded assertions are **tombstoned** (hidden from traversal and export immediately, retained for audit), with explicit repair/GC commands and per-projection manifests + read-only doctor checks. Follows legacy D4.9. — **Reversibility:** costly — tombstone semantics touch traversal, export, and storage GC.
- **D-09:** Ship **all path modes**: reachability check, shortest (min-hop) path, and up-to-K ranked paths — covering roadmap SC2 and PRD §10.4 path queries. Every mode honors budgets and the "no path requires completeness" rule. — **Reversibility:** costly — path modes are API/tool surface consumed by Phase 5 and later retrieval.
- **D-10:** Budget values, truncation rendering, and the pattern set are agent discretion **within hard rules**: ADR-0217 ranges and semantics (pre-flight violation is an error; in-flight exhaustion is a typed partial with `truncated` + non-empty `incomplete_reason`; truncation is never rendered as absence); no backend query language crosses the port. Defaults already implemented in `quran-graph` (6 hops / 500 nodes / 2000 edges / 128 fanout / 10 paths / 5s) are the starting point.
- **D-11:** **Full CLI tree**: `qai graph neighbors/path/subgraph/export` reads plus `build/inspect/review/doctor-repair` management. Build management is CLI-only; **no HTTP mutation routes and no new mutation agent tools** in this phase (legacy scope fence). — **Reversibility:** costly — CLI verbs are a versioned operator surface.
- **D-12:** **Full read parity**: every CLI read op has an HTTP route and a typed tool (`quran.graph_neighbors`, `graph_path`, …) with identical results under the versioned `Envelope` + `ToolResult`/reproducibility contract. — **Reversibility:** costly — routes and tool schemas are consumed by Phase 5 UI and the agent runtime.
- **D-13:** Basic local visualization is **static rendering only**: Graph JSON v1 file plus a static SVG/DOT rendering of a neighborhood/subgraph for local inspection. No interactive explorer — that belongs to Phase 5. — **Reversibility:** reversible — rendering is additive and deferred UI is untouched.
- **D-14:** **Full PRD §10.5 explainability** on every result in both human and JSON output: start/end nodes, traversed path, edge types, edge provenance, confidence, applied filters, query duration. — **Reversibility:** costly — the explainability payload is a published result contract downstream agents and later UI consume (extends Phase 3 D-11).

### Agent's Discretion

- Word-root projection build mechanics (batching, job checkpoints, index layout) so long as morphology invariants I11/I12/I13 + alignment hold and canonical bytes are never mutated.
- Edge authorship/attribution model (single operator vs attributed scholars) so long as every non-structural edge carries PRD §10.3 provenance and deny-by-default + approval-gated canonical rules hold.
- Exact budget values within ADR-0217 ranges; truncation UX details so long as partial results are never rendered as absence; pattern allowlist vs custom typed patterns so long as no query language crosses the port.
- Checkpoint payload schemas, backoff constants, fixture naming, CLI flag naming, and operator-facing wording may follow existing project conventions (Phase 1 D-16 convention) so long as the locked behavior above is preserved.

### Deferred Ideas (OUT OF SCOPE)

- Tafsir / hadith / isnad graph links and ingestion → roadmap Phases 6–7.
- Interactive graph explorer, Web GUI reading/research views, TUI cockpit → roadmap Phase 5.
- CozoDB and sqlite-graph-extension adapter spikes (legacy TASK-411/412) → optional within this phase only as go/no-go spikes behind the same port; SQLite remains the single source of truth either way.
- GraphML export → deferred lossy view generated from Graph JSON v1, per ADR-0218 (never the authoritative form).
- Production auth/RBAC, TLS, server management → roadmap Phase 11; PostgreSQL/Qdrant adapters, Docker, backups → Phase 12.
- Additional Quran qira'at / editions beyond the active-edition model → their roadmap phases.

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| REQ-quran-graph | First-class graph representation independent of vector search: node/edge types (PRD §10.1/§10.2), edge provenance (§10.3), graph search (§10.4), explainability (§10.5), semi-automated annotation (§10.6) | Port + model + budgets + traversal + patterns + export already landed in `quran-graph`; gaps are the SQLite adapter, services, CLI/HTTP/tool surfaces, annotation lifecycle, manifests/doctor — each mapped below |

Boundary note: `REQ-quran-research-tools` / `REQ-quran-result-contract` / `REQ-tui-cli` / `REQ-quran-display` belong to Phase 5 per CONTEXT — but the graph tool schemas, `ToolResult` envelopes, and explainability payloads designed here are their upstream contract, so they must be built to the §12 shape now, not retrofitted later.

## Summary

Phase 4 is a brownfield gap-closure phase: the typed graph contract (`GraphStore` port, node/edge/assertion model, `QueryBudgets`, batched traversal, typed patterns, pure structural builder, Graph JSON v1 export, in-memory conformance backend) is implemented and tested in `crates/quran-graph` (38 tests green per the legacy ledger), and migration `0019_quran_graph` creates the five SQLite tables. What is missing is everything that turns the port into a product: the application-layer SQLite adapter implementing the same port, the word-root / concept / entity / annotation projection builders, the assertion-authority + review-queue service, durable build lifecycle (reserve → stage → verify → fenced publish), CLI read/management surface backed by SQLite (current CLI is file-backed `MemGraphStore` only), HTTP read routes, typed graph tools, per-projection manifests + read-only doctor + explicit repair, and static DOT/SVG rendering.

**Primary recommendation:** Implement the SQLite adapter in `crates/application` (NOT `crates/storage-sqlite` — the arch-check allowlist forbids that edge), drive all reads through batched frontier expansion over indexed adjacency (mirroring `MemGraphStore` semantics exactly), fix the two confirmed schema gaps (multi-assertion UNIQUE, disputed status) with a forward-only migration, extend the edge vocabulary for D-02/D-05/D-07, and deliver surfaces in the proven Phase 3 order: CLI → HTTP → tools, each behind the existing conformance suite plus new SQLite-backed evidence.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Graph traversal / path / subgraph / pattern queries | API / Backend (`application` services) | — | Deterministic, budgeted, authz-filtered expansion; must be identical across CLI/HTTP/tools |
| Graph persistence (adjacency + assertion authority) | Database / Storage (SQLite via `application` adapter) | — | Authority-vs-projection separation; rebuildable projections, immutable assertions |
| Projection builds (structural, word-root, annotated) | API / Backend (durable job workers) | — | Staged-batch progress + cancellation + fenced publication per ADR-0003/0702 pattern |
| Annotation review queue (create/suggest/accept/reject/correct) | API / Backend (`application` service + CLI) | — | CLI-only management per D-11; atomic assertion+provenance+audit+outbox commit |
| Read surfaces (neighbors/path/subgraph/export) | API / Backend (CLI thin wrappers, HTTP routes, typed tools) | — | Thin edges with identical results under `Envelope` + `ToolResult` contract |
| Static visualization (DOT/SVG files) | API / Backend (CLI `export` renders files) | — | No browser tier in this phase; files inspected locally, interactive explorer is Phase 5 |
| Citation quotation of graph hits | API / Backend (reader service) | — | Graph carries refs+hashes only; text resolves through the canonical reader, never from graph records |

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `quran-graph` (workspace port crate) | 0.0.0 (path dep) | Typed contract: `GraphStore`, model, budgets, traversal, patterns, structural builder, export, `MemGraphStore` conformance target | Already landed; every adapter must pass its conformance suite [VERIFIED: crates/quran-graph/src/lib.rs:1-57] |
| `rusqlite` via `storage-sqlite`'s `UnitOfWork`/`ReadTx` | workspace-pinned | SQLite adjacency adapter data access | Sole-adapter pattern (ADR-0001); adapter lives in `application` using existing seams, no new storage crate |
| `tokio` + `jobs` crate (DB-backed leased jobs) | workspace-pinned | Durable staged-batch projection builds with checkpoints + cancellation | ADR-0003; Phase 1 pattern for worker host and progress events |
| `axum` (HTTP) + versioned `Envelope` | workspace-pinned | Graph read routes under `/api/v1/quran/graph/…` | Phase 3 established 9 search/lexicon routes + envelope/ETag conventions [VERIFIED: crates/server/src/api.rs:1428-1470] |
| `tool-registry` + `tools` (`ToolResult`, `reproducibility`) | workspace-pinned | Typed graph tools `quran.graph_neighbors/path/subgraph/pattern` (+ root-family) | PRD §12 result contract; Phase 3 registered 7 tools with attributed envelopes [VERIFIED: crates/tool-registry/src/lib.rs:264-274] |
| `clap` (`GraphAction` enum) + `CommandOutput` | workspace-pinned | `qai quran graph …` CLI tree | Existing `GraphAction::{Build,Inspect,Neighbors,Path,RootFamily,Export}` is the extension point [VERIFIED: crates/cli/src/quran.rs:678-737] |
| `serde_json` | workspace-pinned | Graph JSON v1 documents, DOT/SVG text emission, manifest payloads | Already the export serialization; DOT is plain-text emission (no graphviz dep) |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `trycmd` | workspace-pinned | CLI snapshot tests `crates/cli/tests/quran/graph_*.trycmd` | Every new CLI verb (AC-P4-22 pattern from Phase 3 `*_s1/s2.trycmd` split) |
| `quran-morphology` services (`root_search`, dataset activation) | path dep | Word-root projection input + root-family ranked ayahs | Build reads active `<slug>@<version>` dataset; no-dataset → typed `QAI-MORPH-0004`, never invented roots |
| `quran_doctor.rs` check framework (`QuranDoctorCheck`, `CheckLevel`) | in-tree | Per-projection manifest verification + drift detection | Mirror `run_quran_checks` pattern; doctor stays read-only, repair is a separate explicit command |
| `citations` (`QuotationVerdict`, `require_exact`) | path dep | Neighbor views link each hit to a pinned canonical ref (ADR-0111) | First usable vertical slice: build → inspect → neighbors → canonical quotation |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Batched frontier expansion in application | Recursive CTEs in SQL | ADR-0202 allows CTE only as a proved-equivalent optimization; frontier gives portable budget/cancel/authz semantics — stay with frontier, CTE only if measured need |
| SQLite adapter in `application` | Adapter in `storage-sqlite` | Forbidden without allowlist change: `[storage-sqlite] workspace.allow = ["storage","domain","quran-core"]` — `quran-graph` is not listed, `arch-check` fails. Keep adapter in `application` (already allows `quran-graph`) [VERIFIED: xtask/allowlist.toml] |
| Hand-emitted DOT + minimal SVG | `petgraph` + `graphviz-rust` crates | New deps need allowlist + legitimacy vetting for zero gain: DOT output is ~30 lines of string building; SVG for a bounded neighborhood needs only a simple layered layout. No new dependencies |
| New `crates/graph` implementation | Leave `crates/graph` placeholder untouched | CONTEXT code-insight: scope ownership unresolved — decision is planner-level. Recommendation: all Phase 4 work in `quran-graph` + `application`; `graph` stays reserved for later cross-corpus graphs |

**Installation:**
```bash
# No new dependencies. Phase 4 uses workspace-pinned crates only.
cargo build -p quran-graph -p application -p cli -p server
```

**Version verification:** No new external packages are introduced by this phase (verified by reading `crates/quran-graph/Cargo.toml` — deps are `domain`, `quran-core`, `serde`, `serde_json`, `thiserror`, all workspace-internal or already-pinned). No registry lookup required.

## Package Legitimacy Audit

> Phase 4 installs **zero** new external packages. The SQLite adapter uses the existing `storage`/`storage-sqlite` seams; DOT/SVG rendering is plain-text emission via `serde_json`/`std::fmt` (already pinned); traversal is application-level Rust over the port.

| Package | Registry | Age | Downloads | Source Repo | Verdict | Disposition |
|---------|----------|-----|-----------|-------------|---------|-------------|
| *(none introduced)* | — | — | — | — | — | Approved (nothing to install) |

**Packages removed due to [SLOP] verdict:** none
**Packages flagged as suspicious [SUS]:** none

*If the planner later proposes CozoDB / sqlite-graph spikes (deferred, optional), those ARE new dependencies and must pass the Package Legitimacy Gate + `xtask allowlist` amendment at spike time — not in this phase's main plans.*

## Architecture Patterns

### System Architecture Diagram

```
Operator / Researcher
  │  qai quran graph … (clap GraphAction → application::quran_cli free fns)
  │  HTTP /api/v1/quran/graph/… (axum → application graph services)
  │  Agent tools quran.graph_* (tool-registry → GraphBackend trait)
  ▼
┌─ application graph services ──────────────────────────────┐
│ quran_graph_store.rs (SQLite GraphStore adapter)          │
│ quran_graph_build.rs (staged-batch builders + fenced      │
│   publish; structural / word-root / annotated projections)│
│ quran_graph_annotations.rs (assertion authority + review  │
│   queue: propose → accept/reject/correct → supersede)     │
│ quran_graph_api.rs (read services: neighbors/paths/      │
│   subgraph/pattern + explainability payload)              │
│ quran_graph_export.rs (Graph JSON v1 + DOT/SVG render)    │
│ quran_graph_doctor.rs (manifest verify + drift, read-only)│
└──────────┬────────────────────────────────────────────────┘
           │  storage::Database / UnitOfWork / ReadTx seams
           ▼
SQLite: canonical tables (read-only) · graph_projections /
graph_build_progress / graph_nodes / graph_edges /
graph_assertions (0019 + forward fix)
           │
           │  every result joins back through
           ▼
quran_reader (canonical quotation) · citations (verify_quotation)
```

Data-flow for the primary use case (neighbor view around a verse): CLI/HTTP/tool parses stable ID (`ayah:1:1`) → service loads active projection manifest → SQLite adapter expands bounded frontier (budgets pre-checked, cancel flag, authz+tombstone applied during expansion) → hits resolved to pinned refs via reader → explainability payload assembled (start/end, path, edge types + provenance, filters, duration) → `Envelope`/`ToolResult`/human rendering with `truncated` verbatim.

### Recommended Project Structure

```
crates/application/src/
├── quran_graph_store.rs        # SQLite GraphStore adapter (port impl over UnitOfWork)
├── quran_graph_build.rs        # projection builders: structural/word-root/annotated + manifests
├── quran_graph_annotations.rs  # assertion authority + review queue state machine
├── quran_graph_api.rs          # read services (neighbors/paths/subgraph/pattern/root-family)
├── quran_graph_export.rs       # Graph JSON v1 assembly + DOT/SVG static rendering
├── quran_graph_doctor.rs       # per-projection manifest verify + drift (read-only) + repair cmds
└── quran_graph_tools.rs        # GraphBackend impl for tool-registry (attributed envelopes)
crates/cli/tests/quran/
├── graph_s1.trycmd / graph_s2.trycmd   # host-backed segments (import split pattern from Phase 3)
fixtures/quran/graph/
├── mini-structural.json        # exists [VERIFIED: fixtures/quran/graph/]
├── concept-seed-v1.json        # NEW: D-04 curated concept/entity seed list
└── annotation-goldens.json     # NEW: review-workflow golden fixtures
migrations/sqlite/
└── 0022_quran_graph_fix.up.sql # NEW: multi-assertion UNIQUE + disputed status (never edit 0019)
```

### Pattern 1: Port-Preserving SQLite Adapter (batched frontier, not CTE)

**What:** The SQLite adapter implements `GraphStore` by looping over a `neighbors(node)` single-hop SQL query with in-Rust counters, exactly mirroring `MemGraphStore` semantics: budgets validated first (`budgets.check()?`), cancel flag per batch, `ORDER BY edge, neighbor` for determinism, authz + effective-tombstone joined during expansion.

**When to use:** For every traversal op (neighbors, bounded_paths, subgraph, pattern_query). Recursive CTEs only as a later proved-equivalent optimization.

**Example:**
```rust
// Source: crates/quran-graph/src/mem.rs:223-262 (reference semantics to mirror)
// + crates/quran-graph/src/model.rs:328-351 (budget pre-flight)
budgets.check()?; // pre-flight: out-of-range is Err(BudgetExceeded), never clamp
if cancel_requested(cancel) {
    return Ok(PathsResult::incomplete(Vec::new(), "cancelled before path search", 0, 0));
}
// per-batch: single-hop SELECT … WHERE projection_row_id = ? AND src = ?
//   ORDER BY edge, dst_stable_id  (+ assertion join filtering tombstoned/restricted)
// counters: expanded_nodes / expanded_edges; on exhaustion → incomplete(reason), never empty-complete
```

### Pattern 2: Authority-vs-Projection Write Path

**What:** Annotation writes commit assertion + provenance + audit + outbox in one `UnitOfWork` (AC-P4-11); adjacency rows are derived from effective assertions and rebuilt without touching authority (AC-P4-03). `clear_staged` drops adjacency only — "clearing a projection never discards scholarly history" [VERIFIED: crates/quran-graph/src/store.rs:206-208].

**When to use:** Every annotation/review/build operation. Mirror Phase 1's `AuditedMutation` single-seam pattern — no second transaction manager.

### Pattern 3: Conformance-Gated Adapter Activation

**What:** `crates/quran-graph/tests/conformance.rs` (8 tests: budgets≠empty, authz intermediates, staging, capabilities) runs against `MemGraphStore` today; the SQLite adapter must pass the identical suite (parameterized over backends) before activation. New SQLite-specific tests: determinism under insertion-order variation, `NodeNotFound` vs truncated-empty distinction.

**When to use:** Adapter plan: first make conformance suite backend-generic, then implement adapter, then gate activation on green.

### Pattern 4: Explainability Payload on Every Result

**What:** Extend Phase 3 D-11 ("every match reports why it matched") to graph: start/end nodes, traversed path (ordered node IDs + edges), edge types + per-edge provenance (structural builder/input-version vs assertion id + reviewer/decision/timestamp), applied filters (budgets, edge filter, direction, authz scope descriptor — never hidden IDs), snapshot/build identity (projection_id + builder_version + corpus_generation), completion status (`truncated` + `incomplete_reason`), query duration ms. PRD §10.5 + D-14.

### Pattern 5: Review State Machine (propose → decide → supersede)

**What:** `AssertionDecision::{Pending, Accepted, Rejected, Superseded}` exists [VERIFIED: crates/quran-graph/src/model.rs:178-186]; the service adds transitions: suggest (layer D, algorithm/version/confidence required by migration CHECK) → accept/reject/correct (reviewer + decided_at required by CHECK) → correct creates a NEW assertion with `supersedes_id`, old row → `Superseded`. Tombstoned (`Rejected`/`Superseded`) rows hide immediately, retained for audit. Enforce as tests (acceptance.md pattern), not comments.

### Anti-Patterns to Avoid

- **Putting the adapter in `storage-sqlite`:** `arch-check` fails (allowlist forbids `storage-sqlite → quran-graph`). Adapter lives in `application`.
- **Final-`LIMIT`-as-budget:** rejected by ADR-0217 — expansion itself must be capped; a trailing LIMIT protects nothing and silently changes meaning.
- **Rendering `truncated:true` as absence:** "no path" requires `truncated == false`. CLI human output must print the incomplete reason verbatim (e.g. `(truncated: node budget 500 exhausted)`), never just an empty list.
- **Post-filtering authz:** hidden intermediates must neither appear in nor influence paths/counts — filter during expansion (the `is_edge_visible` join), never after.
- **Editing migration 0019:** forward-only convention — schema fixes go in a new `0022_*.up.sql` + `checksums.json` update, verified by `cargo xtask migrate-check`.
- **Storing Arabic text in graph records:** refs + hashes only (`no_canonical_text_embedded` test pattern); quotations resolve through the reader (SC5-equivalent pin per hit).
- **Exposing query language:** the `tests/no_query_language.rs` pin must keep passing — no SQL/Cypher/Datalog fragments in public API names or formatted outputs.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Up-to-K path enumeration | Custom graph algorithm crate | Existing `traverse::up_to_k_paths` + `MemGraphStore::bounded_paths` BFS logic, ported to batched SQL expansion | Semantics (shortest-first, stable-ID tie-break, budget-explicit) are specified and tested; reimplementation risks ordering drift between backends |
| Budget validation | Ad-hoc limit checks per endpoint | `QueryBudgets::check()` + `ExportNotice` | Single source of ranges (ADR-0217); pre-flight-error vs in-flight-partial distinction is a correctness property |
| Provenance envelope | Custom annotation schema | `Assertion` + `graph_assertions` table + PRD §10.3 seven-field rule | Review history, supersession, and export's assertions-travel-with-edges rule all key off this shape |
| Reproducibility checksum | Custom hashing | `tools::reproducibility()` | §12.1 contract; identical inputs must reproduce identical outputs across CLI/HTTP/tools |
| CLI output duality | Custom print logic | `CommandOutput::{ok, err}` + `json_or_err` | Human + `--json` parity with stable exit codes (`exit::USAGE/VALIDATION/NOT_FOUND/INTERNAL`) |
| HTTP envelope | Custom response shape | `Envelope { api_version, data, meta }` + ETag pattern | Versioned contract Phase 5 UI consumes; OpenAPI file must be extended in the same change |
| Job durability | In-memory build progress | `jobs` crate leased workers + `graph_build_progress` table | Crash-resume from durable staged progress (AC-P4-07), not memory |
| DOT/SVG output | Graphviz binary dependency or render crate | Plain-text DOT emitter + minimal hand-laid SVG for bounded neighborhoods | `dot` binary is NOT installed here [VERIFIED: shell `command -v dot` → not found]; file output keeps Phase 4 hermetic, rendering happens in external tools |

**Key insight:** This phase's risk is not algorithms (they exist and are tested) but *semantic parity* between two backends plus *lifecycle integrity* (builds, reviews, tombstones). Every custom reimplementation of an already-specified behavior is a parity bug waiting to happen — reuse, parameterize, conformance-test.

## Common Pitfalls

### Pitfall 1: `graph_edges` UNIQUE forbids multi-assertion edges (AC-P4-05 violation in schema)

**What goes wrong:** Migration 0019 declares `UNIQUE (projection_row_id, src_stable_id, edge, dst_stable_id)` [VERIFIED: migrations/sqlite/0019_quran_graph.up.sql:47-57] — but the model contract says "the stable edge identity includes `assertion_id`, not just the triple" [VERIFIED: crates/quran-graph/src/model.rs:119-120] and AC-P4-05 requires the same triple to carry multiple attributed assertions. The second scholar's assertion on an existing triple fails with a constraint violation.

**Why it happens:** Skeleton migration written before the TASK-413 identity reconciliation landed in the model.

**How to avoid:** Forward-only `0022` migration replacing the UNIQUE with `(projection_row_id, src_stable_id, edge, dst_stable_id, assertion_id)` (NULL-safe: structural edges share `assertion_id NULL` — use `COALESCE` unique index or keep triple-unique only where `assertion_id IS NULL` via partial indexes). Gate with a multi-assertion test.

**Warning signs:** `UNIQUE constraint failed: graph_edges…` on the second annotation of the same pair.

### Pitfall 2: Edge vocabulary missing D-02/D-05/D-07 predicates

**What goes wrong:** `EDGE_VOCABULARY` has 15 entries [VERIFIED: crates/quran-graph/src/model.rs:73-89] —
`"CONTAINS", "NEXT", "MENTIONS_CONCEPT", "REFERS_TO", "MENTIONS", "CITES", "CONTRASTS_WITH", "PARALLELS", "EXPLAINS", "HAS_LEMMA", "HAS_ROOT", "HAS_STEM", "DERIVED_FROM", "SAME_ROOT_AS", "SAME_LEMMA_AS"` —
but locked decisions require `TRANSLATES` (D-02), `RELATED_TO` (D-05), `SUPPORTED_BY`/`DISPUTED_BY` (D-05, D-07), and PRD §10.2 additionally lists `PRECEDES`/`FOLLOWS` (deliberately rejected per TASK-414 — do NOT re-add), `SIMILAR_TO`, `ASSERTED_BY`, `GENERATED_BY`.

**Why it happens:** Vocabulary landed with the structural slice; annotated-slice predicates were never added.

**How to avoid:** One vocabulary-extension task: add `TRANSLATES`, `RELATED_TO`, `SUPPORTED_BY`, `DISPUTED_BY` (exact D-scope), update the `vocabulary_has_expected_entries` test count, and record the PRD-§10.2 remainder as explicitly deferred with rationale. Staging and `validate_pattern` both key off this list, so one change propagates.

**Warning signs:** `PatternRejected { unknown edge 'SUPPORTED_BY' }` on a D-05 acceptance test.

### Pitfall 3: `disputed` verification status has no model state

**What goes wrong:** D-07 mandates `verification_status` of `unverified/verified/disputed/superseded/rejected`, but `AssertionDecision` is `{Pending, Accepted, Rejected, Superseded}` [VERIFIED: crates/quran-graph/src/model.rs:178-186] with migration CHECK `decision IN ('pending','accepted','rejected','superseded')` [VERIFIED: 0019:70-72]. There is no `disputed` state, and the names don't match the decision text.

**Why it happens:** Model predates the CONTEXT wording.

**How to avoid:** Planner decision (agent discretion): either map explicitly (`pending→unverified`, `accepted→verified`, add `disputed` variant + CHECK + migration) or document the mapping in the review-service plan. `SUPPORTED_BY`/`DISPUTED_BY`-as-edges (D-07) partially covers dispute expression, but a disputed *status* on the assertion itself still needs a home — recommend adding the variant since `disputed` edges without disputed statuses will confuse the Phase 5 UI.

**Warning signs:** Review-queue tests that can't represent "scholar X disputes this edge" without rejecting it.

### Pitfall 4: Assertions FK'd to build rows break rebuild-preservation (AC-P4-03)

**What goes wrong:** `graph_assertions.projection_row_id REFERENCES graph_projections(id)` ties authority records to a single build row. Deleting/superseding a build row orphans or cascades authority — contradicting "rebuilding the graph projection must not discard human annotations or review history" (ADR-0202 §1).

**Why it happens:** Skeleton schema; legacy done.md already flags assertion-authority as remaining TASK-416 scope.

**How to avoid:** Scope assertions by `(projection_id, edition_id, dataset_scope)` rather than build-row FK (forward migration), or document the FK as staging-only with a promotion step that re-keys authority to the projection family. Cover with the AC-P4-03 delete+rebuild test on a populated fixture.

**Warning signs:** Rebuild test loses review rows; FK violation on build-row cleanup.

### Pitfall 5: File-backed CLI semantics leaking into the SQLite CLI

**What goes wrong:** Current `cmd_graph_*` reads a JSON file into `MemGraphStore` per invocation [VERIFIED: crates/application/src/quran_cli.rs:3204-3236]; the SQLite CLI must instead open the DB, resolve the *active* projection + generation stamp, and honor the same budgets — a naive port keeps `--file` flags and per-command staging, losing manifest/drift semantics.

**Why it happens:** TASK-424 slice was file-backed by design.

**How to avoid:** New CLI verbs take `--db` (existing convention) + projection selection (default: active), never `--file` (keep file verbs only for the fixture/debug path). Add `Subgraph` + `Pattern` verbs (absent today) and `review`/`doctor-repair` management verbs per D-11.

**Warning signs:** Two CLIs disagree on the same corpus; doctor can't see what the CLI built.

### Pitfall 6: Word-root projection inventing roots when no dataset is active

**What goes wrong:** OD-11 BLOCKED means production often has no morphology dataset; a builder that falls back to heuristics or empty-success violates D-03 and AC-P4-19.

**Why it happens:** Convenience fallback.

**How to avoid:** Mirror Phase 3 exactly: no active dataset → typed `QAI-MORPH-0004`-family error naming the capability (`UnavailableDataset`), CLI exit 5 / HTTP 404 convention from 03-05. Synthetic fixtures carry `synthetic_test_only` labeling; never elect an authoritative winner among competing analyses (I11).

### Pitfall 7: `crates/graph` scope creep

**What goes wrong:** Implementer puts new code in `crates/graph` (still a `//! Phase 3 placeholder` [VERIFIED: crates/graph/src/lib.rs]) instead of `quran-graph` + `application`, splitting the port and breaking the conformance story.

**How to avoid:** Planner records the ownership decision explicitly (recommendation: `graph` stays reserved; zero Phase 4 code there) and reviewers reject PRs touching it.

## Code Examples

Verified patterns from in-tree sources (all read this session):

### Budgeted neighbor query (service-level shape)

```rust
// Source: crates/quran-graph/src/model.rs:328-351 + src/store.rs:153-162
use quran_graph::{AuthzScope, EdgeFilter, GraphStore, QueryBudgets};
use std::sync::atomic::AtomicBool;

let budgets = QueryBudgets { max_hops: hops, ..QueryBudgets::default() };
budgets.check()?; // pre-flight error, never silent clamp
let cancel = AtomicBool::new(false);
let scope = AuthzScope::all_visible();
let result = store.neighbors("ayah:1:1", &EdgeFilter::any(), &budgets, &cancel, &scope)?;
// result.truncated == false + empty nodes  =>  genuinely no neighbors
// result.truncated == true                 =>  render result.incomplete_reason verbatim
```

### Export with tombstone/authz policy + truncation notice

```rust
// Source: crates/quran-graph/src/export.rs:59-109
use quran_graph::{ExportNotice, export_json_with_notice, retain_visible,
                  assertion_allowlist_predicate};

let (kept_nodes, kept_edges) =
    retain_visible(&nodes, &edges, assertion_allowlist_predicate(&visible_ids));
// kept_assertions: ONLY the assertion records for kept_edges' assertion_ids,
//   each effective (not rejected/superseded) — assertions travel with edges.
let notice = if truncated {
    ExportNotice::incomplete(reason) // non-empty incomplete_reason, ADR-0217 rule 2
} else {
    ExportNotice::complete()
};
let doc = export_json_with_notice(&kept_nodes, &kept_edges, &kept_assertions, &manifest, &notice);
assert_eq!(doc["format"], "quran-graph-json-v1");
```

### CLI command shape (human + JSON duality, stable exits)

```rust
// Source: crates/application/src/quran_cli.rs:45-61,2413 + crates/cli/src/quran.rs:693-700
// application free fn returns CommandOutput::{ok(human, json) | err(exit::CODE, msg)}
// cli parses flags (GraphAction::Neighbors { file, node, hops }) and dispatches;
// NEW SQLite verbs follow the same shape but take db projection selection, not --file.
```

### HTTP read route (envelope + error mapping)

```rust
// Source: crates/server/src/api.rs:277 (json_response envelope), :831-847 (typed error→status)
// POST /api/v1/quran/graph/neighbors  { node, edge_types?, direction?, budgets? }
//   → 200 Envelope { api_version: API_VERSION, data: { nodes, edges, explanation,
//        truncated, incomplete_reason }, meta } with ETag
//   → budget pre-flight violation → 422 + QAI-GRAPH-0002; unknown node → 404 + QAI-GRAPH-0004;
//      truncated result is still 200 with truncated:true (never an error, never empty-masked)
```

### Typed tool registration (attributed envelope + reproducibility)

```rust
// Source: crates/tool-registry/src/lib.rs:264-274 (TOOL_NAMES), :502-523 (attributed),
//         crates/tools/src/lib.rs:56-83 (ToolResult), :128+ (reproducibility)
// TOOL_NAMES += "quran.graph_neighbors", "quran.graph_path", "quran.graph_subgraph",
//              "quran.graph_pattern"  (each with SemVer 1.0.0 consts like ROOT_TOOL_VERSION)
// BackendMeta gains projection identity (projection_id, builder_version, corpus_generation)
//   so reproducibility.source_versions pins the graph build, not just the edition.
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Placeholder `quran-graph` / `graph` crates | Typed port + mem backend + budgets + traversal + patterns + structural builder + export, 38 tests green | Legacy board slices (commits `5a32a15`…`b8baad0` per done.md) | Phase 4 starts from contract, not from zero |
| `PRECEDES`/`FOLLOWS` edge aliases | Single canonical `NEXT` predicate | TASK-414 reconciliation | Temporal ordering has one name; plans must not reintroduce aliases |
| In-memory/file-backed graph CLI | (this phase) SQLite-backed CLI + HTTP + tools | Now | File verbs become fixture/debug-only; active-projection semantics take over |
| Unbounded graph traversal | ADR-0217 budgets + typed incompletes (Proposed, P4-X03 ratification pending) | 2026-09-24 draft | Every new surface must thread budgets + render truncation |
| Ad-hoc export | `quran-graph-json-v1` mandatory envelope (Proposed, P4-X05 ratification pending) | 2026-09-24 draft | Export shape is a versioned contract (`format` marker), GraphML only as derived-lossy later |

**Deprecated/outdated:**
- GraphML as primary export: deferred per ADR-0218, never the authoritative form.
- CozoDB / sqlite-graph-extension as authorities: optional go/no-go spikes only; SQLite stays source of truth either way.
- `graph_edges.budgets_json` column: written by no model struct — adapter should ignore it (or drop in a later migration), never treat it as the budget mechanism.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `graph` binary/DOT renderer absent on user machines too (verified absent only on this dev machine via `command -v dot`) | Standard Stack / Don't Hand-Roll | LOW — DOT file output is viewable anywhere regardless; only bundled auto-render would break |
| A2 | `cargo xtask ci` is the 9-step full gate (cited from legacy acceptance.md, not re-verified by running it) | Validation Architecture | LOW — planner should confirm step list at plan time; running it is the gate itself |
| A3 | Owner will accept `disputed` as a new `AssertionDecision` variant (recommendation, not a decision) | Pitfall 3 | MEDIUM — if rejected, D-07 dispute representation must be edges-only; plan needs a fallback branch |
| A4 | Assertion re-scoping (Pitfall 4) is acceptable as a forward migration rather than a 0019 edit | Pitfall 4 | LOW — forward-only is the locked convention; the only question is exact new key shape |
| A5 | Existing `jobs` worker host (Phase 1 D-13..D-16) can host graph build workers without modification | Architecture Patterns | MEDIUM — if build progress payloads don't fit the job checkpoint schema, a small extension task is needed; verify against `jobs/src/worker.rs` at plan time |
| A6 | Concept/entity seed values (which themes/persons/places) need no research beyond owner curation | D-04 / Open Questions | MEDIUM — seed content is a scholarly-content decision; agents must not invent authority, only the fixture mechanics |

## Open Questions

> Status (2026-09-28 plan-check): all 5 items are intentionally carried, not awaiting research. Q1/Q2 stay BLOCKED/pending-ratification per CONTEXT.md D-03 (owner-only; plans carry explicit BLOCKED items with closing commands). Q3 is decided in 04-01 (reserve `crates/graph`, zero Phase 4 code). Q4 uses the synthetic-content fallback posture. Q5 ships DOT + minimal layered SVG per D-13. Resolving owner gates here would contradict D-03 — hence no `(RESOLVED)` markers by design.

1. **OD-11 (morphology dataset/license) + OD-12 (linguist) — BLOCKED owner gates**
   - What we know: Recorded BLOCKED in decisions-needed.md (both 🔴 unanswered); Phase 3 closed with synthetic-only evidence; D-03 explicitly carries them forward without re-litigation.
   - What's unclear: Dataset identity, license evidence, linguist name — all human-only.
   - Recommendation: Every word-root plan carries an explicit BLOCKED item with the exact closing command (`licenses/qac` capture per licenses/README.md; linguist sign-off on ADR-0210/0215); word-root acceptance on synthetic fixtures only, production-gated.

2. **P4-X01/X02/X03/X05 ratifications (ADR-0202, ADR-0702 subset, ADR-0217, ADR-0218)**
   - What we know: All four are Proposed; drafts landed 2026-09-24; no agent may flip status (done.md).
   - What's unclear: Whether the owner ratifies before/during Phase 4 execution.
   - Recommendation: Plans proceed on the Proposed texts as the working contract and tag each gate "pending owner ratification (P4-X0n)"; ratification itself is owner work, never an agent task.

3. **`crates/graph` ownership**
   - What we know: Empty placeholder; CONTEXT requires an explicit planning decision.
   - What's unclear: Intended future role (cross-corpus graphs? Phase 7 isnad?).
   - Recommendation: Decide in planning (recommend: reserve, zero Phase 4 code), record in one line, enforce in review.

4. **Concept/entity seed content authority**
   - What we know: D-04 requires a curated versioned seed list; nothing in-tree defines its entries.
   - What's unclear: Who curates "major themes, persons, places" and what counts as sufficient for exit.
   - Recommendation: Mechanics (fixture format, versioning, extension workflow) are agent work; the seed *content* is an owner/scholar input — plan a content-delivery dependency with a synthetic-content fallback for all automated tests, same posture as OD-11/OD-12.

5. **Static SVG layout expectations**
   - What we know: D-13 requires static SVG/DOT rendering of a neighborhood/subgraph; no layout algorithm is specified.
   - What's unclear: Whether a simple layered/radial layout suffices for exit or a reviewer expects graphviz-quality output.
   - Recommendation: Ship DOT (exact, tool-renderable) + minimal readable SVG (layered by hop distance from seed, RTL labels, truncation banner); treat layout polish as discretionary within D-13.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain (cargo/rustc) | All builds/tests | ✓ [VERIFIED: shell] | cargo 1.97.1 / rustc 1.97.1, edition 2024 | — |
| SQLite (via rusqlite, bundled) | SQLite adapter, migrations | ✓ (bundled; migration 0019 applied per `migrate-check` ledger) | — | — |
| graphviz `dot` binary | Nothing (explicitly NOT required) | ✗ [VERIFIED: shell `command -v dot` → not found] | — | DOT file output for external rendering; hand-laid SVG in-tree |
| Existing fixtures (`test-edition-min`, morphology synthetic, `graph/mini-structural.json`) | Builds, conformance, goldens | ✓ [VERIFIED: fixtures/quran/ listing] | — | — |
| Production morphology dataset (OD-11) | Word-root production projection | ✗ (BLOCKED owner gate) | — | Synthetic fixture; typed unavailable-dataset behavior |

**Missing dependencies with no fallback:**
- OD-11 licensed dataset + OD-12 linguist sign-off (owner-only; gate word-root production claims, not engineering).

**Missing dependencies with fallback:**
- graphviz binary → DOT text + in-tree SVG (D-13 satisfied without it).

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (unit + integration) + trycmd (CLI snapshots) + axum test routers (HTTP contract) |
| Config file | Cargo workspace; `crates/cli/tests/quran.rs` runner; `xtask` gates (`arch-check`, `migrate-check`, `ci`) |
| Quick run command | `cargo test -p quran-graph` (38 tests: lib 22 + conformance 8 + no-query-language 2 + structural 1 + traversal 5) |
| Full suite command | `cargo xtask ci` (9-step gate per legacy acceptance.md) |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| REQ-quran-graph SC1 | Neighbor view around verse/word/root/concept | integration + trycmd + HTTP contract + tool test | `cargo test -p application --test graph_neighbors` + `trycmd tests/quran/graph_s*.trycmd` | ❌ Wave 0 (new) |
| REQ-quran-graph SC2 | Paths (reachability/min-hop/up-to-K) with per-edge provenance | integration (SQLite adapter) + conformance extension | `cargo test -p quran-graph --test conformance` (exists for mem; parameterize for SQLite) | ❌ Wave 0 (parameterize + SQLite backend) |
| REQ-quran-graph SC3 | Subgraph export with provenance intact | unit (export) + policy-leak test | existing `export.rs` tests + new SQLite-backed export test | ❌ Wave 0 (SQLite-backed half) |
| REQ-quran-graph SC4 | Explainability on every result | contract tests over all query types × all surfaces | new `graph_explain` contract test | ❌ Wave 0 (new) |
| REQ-quran-graph (builds) | Staged-batch build + fenced publish + crash resume | fault-injection (kill at batch boundary) | new `graph_build` tests | ❌ Wave 0 (new) |
| REQ-quran-graph (review) | Suggest → accept/reject/correct → supersede; tombstone invisibility | state-machine tests + rebuild-preservation test | new `graph_review` tests | ❌ Wave 0 (new) |
| REQ-quran-graph (doctor) | Read-only manifest verify + drift per projection; explicit repair | immutability (before/after state compare) | new doctor tests | ❌ Wave 0 (new) |

### Sampling Rate
- **Per task commit:** `cargo test -p quran-graph` + affected crate tests + `cargo fmt --check` + `clippy -D warnings`
- **Per wave merge:** above + `cargo xtask arch-check` + `cargo xtask migrate-check`
- **Phase gate:** Full `cargo xtask ci` green + recorded non-implementer live walkthrough before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] Backend-generic conformance harness (`MemGraphStore` → parameterize over `SQLiteGraphStore`)
- [ ] `crates/application/tests/graph_*.rs` integration files (neighbors/paths/subgraph/pattern/review/build/export/doctor)
- [ ] `crates/cli/tests/quran/graph_s1.trycmd` + `graph_s2.trycmd` (+ runner entries; host-backed import-split pattern)
- [ ] `crates/server/tests/` graph contract tests + `docs/08-api/quran-v1-openapi.json` graph route additions
- [ ] `fixtures/quran/graph/concept-seed-v1.json` + annotation golden fixtures (synthetic-labeled)
- [ ] Migration `0022` + `checksums.json` update (UNIQUE fix, disputed status, assertion scoping)

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | Local-first single operator; production auth is Phase 11 scope (deferred) |
| V3 Session Management | no | No sessions in Phase 4 surfaces |
| V4 Access Control | yes | `AuthzScope` applied during expansion (never post-filter); export uses `assertion_allowlist_predicate`; tombstoned rows invisible to all scopes |
| V5 Input Validation | yes | `QueryBudgets::check()` pre-flight; `validate_pattern` allowlist (rejects raw query text); stable-ID resolution → typed `NodeNotFound`, never string-interpolated SQL (parameterized queries only) |
| V6 Cryptography | partial | No new crypto; reuse frozen hash recipes + `tools::reproducibility` checksums; assertion/export integrity rides existing audit chain (ADR-0009) |

### Known Threat Patterns for SQLite-backed graph + CLI/HTTP/tools

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Graph-expansion DoS (root-hub fanout, cycles) | Denial of service | ADR-0217 budgets enforced during expansion + native timeout; cycle/high-degree conformance fixtures |
| Raw query language injection via pattern API | Injection | Typed `Pattern` + allowlisted vocabulary only; `no_query_language` pin test; parameterized SQL |
| Restricted-evidence leak via traversal intermediates or export | Information disclosure | Authz+tombstone filtering during expansion AND pre-serialization (`retain_visible`); policy-leak tests asserting restricted IDs absent from serialized bytes |
| Truncation misrendered as negative claim | Tampering (integrity of results) | `truncated` + non-empty `incomplete_reason` on every partial; "no path requires completeness" enforced in type + test |
| Doctor side-effect mutation | Tampering | Read-only connection + before/after state comparison tests; repair only via explicit confirmed command with audit |
| Scholarly-opinion laundering (computational edge presented as verified) | Spoofing | Layer-D attribution + pending-by-default + explicit human decision gate; suggestions labeled in every surface |

## Sources

### Primary (HIGH confidence)
- `crates/quran-graph/src/model.rs`, `store.rs`, `traverse.rs`, `pattern.rs`, `structural.rs`, `export.rs`, `error.rs`, `mem.rs`, `lib.rs` — read this session; all structural claims (vocabulary, budgets, error codes, stable IDs, tombstone semantics) quoted verbatim above
- `migrations/sqlite/0019_quran_graph.up.sql` — read this session (5 tables, CHECK lists, UNIQUE constraint)
- `docs/02-architecture/decisions/ADR-0202-graph-store.md`, `ADR-0217-graph-query-limits.md`, `ADR-0218-graph-export-formats.md` — read this session
- `docs/01-requirements/requirements.md` §§10–12 (lines 525–880) — read this session
- `docs/03-plan/phases/phase-04-quran-graph/README.md`, `acceptance.md`, `done.md` — read this session
- `crates/application/src/quran_cli.rs` (graph section), `crates/cli/src/quran.rs` (`GraphAction`), `crates/server/src/api.rs` (routes/envelope), `crates/tool-registry/src/lib.rs` (TOOL_NAMES), `crates/tools/src/lib.rs` (`ToolResult`), `xtask/allowlist.toml` — inspected via read/grep this session

### Secondary (MEDIUM confidence)
- `.planning/phases/03-quran-search-linguistics/03-CONTEXT.md` (D-09/D-10/D-11/D-15 inherited patterns) — read relevant excerpts
- `docs/05-followups/decisions-needed.md` (OD-11/OD-12 BLOCKED status) — read excerpts

### Tertiary (LOW confidence)
- None relied upon — external claims (toolchain versions, `dot` absence) verified via shell; all other external matters marked [ASSUMED] in the Assumptions Log.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — zero new deps; every crate/pattern verified in-tree this session.
- Architecture: HIGH — port semantics, adapter placement (allowlist), and schema gaps all evidenced by file reads with line citations.
- Pitfalls: HIGH — each pitfall names the exact file/lines and the failing behavior; two (UNIQUE, vocabulary) are checkable in under a minute.

**Research date:** 2026-09-28
**Valid until:** 2026-10-28 (stable domain; brownfield code changes only via Phase 4 execution itself)

## Project Constraints (from AGENTS.md)

- Rust is the primary language — all new code is Rust; follow phase plans for crate boundaries.
- No domain code depends on `api`, `server`, `tui`, `cli`, or any concrete provider (adapter lives in `application`, port in `quran-graph` — compliant).
- Canonical rows written only through `CanonicalWriter` + `ApprovalToken` (graph builds are derived projections; canonical-unchanged check per AC-P4-01).
- Every derived artifact records the version of everything it was derived from (projection manifests: builder_version, edition_id, corpus_generation, dataset_versions, dependency snapshot).
- Deny-by-default: permissions, network, filesystem, side effects default to empty sets (review queue, export policy, no mutation HTTP/tools in this phase).
- Nothing that can modify data runs inside `doctor` (read-only checks + explicit repair commands).
- Secrets never logged/embedded; imported internet content untrusted until validated (concept-seed and annotation inputs are operator-supplied content — validate shape, attribute source).
- No model may fabricate a verse, hadith, chain, grading, or citation (canonical path stays model-free; computational suggestions never self-promote).
- Naming: phases `phase-NN-slug/`, tasks `TASK-nnn-slug.md`, ADRs `ADR-nnnn-title.md`, AC `AC-Pn-nn`.
- Completion: implement → tests → lint/checks → task doc → phase progress → rollup → follow-ups → CHANGELOG.
