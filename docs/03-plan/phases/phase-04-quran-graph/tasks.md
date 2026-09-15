# TASKS — Phase 4: Quran Knowledge Graph

Naming: TASK-4nn. Tasks behind_interfaces, unit+property tested, typed
`Diagnostic` errors, cancellation for long ops, `fmt`/`clippy -D warnings` green.

## Core port + default backend (D4.1)

### TASK-401-graphstore-port
- Define `GraphStore` trait: node resolution, neighbors, bounded paths,
  bounded subgraphs, bounded patterns, projection build/inspect/delete-verify,
  capability discovery.
- Requirements: snapshot + authz + budgets + deadline in every query;
  incomplete-vs-empty distinction; no backend query language leaks.
- AC: conformance suite drafted; SQLite adapter passes (acceptance hooks in).
- Deliverable: D4.1 (port).
- IDs after this: use `TASK-4xx`

### TASK-402-sqlite-cte-adapter
- Implement SQLite adjacency adapter: `graph_nodes`/`graph_edges` per
  ADR-0202 §3; depth/expansion/result budgets; cycle handling; interruption.
- AC: cyclic + high-degree fixtures terminate within limits; budgets enforced
  en-route (not just final LIMIT).
- Deliverable: D4.1.

### TASK-403-batch-frontier-traversal
- Batched frontier expansion with counters for queries CTEs cannot bound
  reliably; equivalent truncation semantics to TASK-402.
- AC: equivalence tests vs CTE adapter on fixtures.
- Depends on: TASK-402.

## Structural projection (D4.2)

### TASK-404-structural-graph-build
- Builder: canonical hierarchy → nodes/edges (`CONTAINS`, `NEXT`); builder +
  input-version provenance; rebuild preserves annotations/review history.
- AC: delete+rebuild projection keeps annotations; canonical lookup works with
  projection missing.

## Word-root graph (D4.3, D4.4) — REQUIRED

### TASK-405-wordroot-projection
- Promote Phase-2 root/lemma/analysis records into graph nodes preserving
  dataset identity/alignment.
- Token —`HAS_ROOT`→ Root ←`HAS_ROOT`— Token hub topology; one-hop ayah reach
  via `IN_AYAH`. No ayah×ayah materialization.
- AC: root families for ق و ل (قال/قل/قالوا/قلتم/يقول/قول), ك ت ب, ع ل م
  resolve with dataset attribution; competing analyses stay separate.

### TASK-406-root-family-tool
- Tool/CLI: given a root (e.g., ق و ل), return ranked ayahs with provenance,
  dataset versions, and expansion counts; typed budgets; explicit truncation.
- AC: `qai graph root-family "ق و ل"` returns expected fixture ayahs; budgets
  enforced; results cite attribution, not just text.
- Depends on: TASK-405.

### TASK-407-wordroot-verification
- Golden fixtures (linguist-reviewed root families); `qai doctor` checks for
  root/lemma projection; generation stamps.
- AC: drift detected when morphology dataset version changes.
- Deliverable: part of D4.9.

## Concept + other graphs (D4.5)

### TASK-408-concept-graph
- Annotated concept/topic projection (`MENTIONS_CONCEPT`) from Phase-2+
  datasets; interpretive edges carry attribution/evidence; never merge
  contradictory scholarly claims.
- AC: every non-structural edge resolves to attribution + evidence.

## Conformance + ops (D4.6, D4.9)

### TASK-409-conformance-suite
- Backend-agnostic tests: neighbors/paths/subgraph/pattern, budgets,
  authz-filtered intermediates, incomplete-result semantics.
- AC: any new adapter must pass suite before activation.

### TASK-410-graph-manifests-doctor
- Per-graph projection ID, source versions, builder version, snapshot;
  `qai doctor` per-graph drift; `qai index rebuild --graph <name>`.
- AC: drift reported for each graph independently.
- Deliverable: D4.9.

## Backend spikes (D4.7, D4.8) — go/no-go

### TASK-411-cozodb-spike
- spike: CozoDB adapter behind `GraphStore`; benchmark recursive/transitive
  word-root workloads vs SQLite-CTE; license audit (MPL-2.0); packaging impact.
- AC: benchmark report + explicit go/no-go; authority stays in SQLite in both
  outcomes.
- Deliverable: D4.7.
- Constraint: Datalog must not leak through the public API.

### TASK-412-sqlite-graph-ext-spike
- spike: SQLite graph extension adapter (Cypher inside SQLite); benchmark;
  maturity re-check (start bar: >1k-node scale, stable releases).
- AC: benchmark report + explicit go/no-go; Cypher must not leak through the
  public API.
- Deliverable: D4.8.
