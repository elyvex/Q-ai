# Phase 3 (dir: phase-04) — Quran Knowledge Graph — Implementation Plan

**Phase:** PRD Phase 3 — Quran Graph (directory retains Phase-4 numbering; reconciliation is
tracked in `README.md` and must not be renumbered by hand)
**Status:** Planned (reviewed 2026-09-17 against code + PRD + ADR-0202/0702)
**Depends on:** Phase 1 canonical corpus (usable now); Phase 2 morphology datasets
(blocking for M5 and full phase exit, not structural engineering); ADR-0202 ratification; Phase-3 subset of ADR-0702
**Scope source:** PRD `docs/01-requirements/requirements.md:3880–3892` (combined with the
shorter Phase-3 list at 3724–3733), ADR-0202, ADR-0702 (Proposed)

---

## 1. Executive summary

Phase 3 delivers the typed Quran knowledge graph: a backend-neutral `GraphStore` port
(ADR-0202) over SQLite adjacency tables, structural/linguistic/annotated projections,
bounded traversal, provenance-carrying annotations, CLI/API/tool surfaces, Graph JSON
export, and basic local visualization.

Two facts shape the plan:

1. **Structural work can start now.** `QuranRepository` (crates/storage/src/quran.rs:611)
   already exposes editions, surahs, ayahs, tokens, separators, and divisions. No FTS or
   morphology is required for the structural projection.
2. **The word-root graph is blocked on Phase 2 morphology.** `crates/quran-morphology` is a
   placeholder. Root/lemma identities may only come from attributed, versioned datasets —
   normalization or heuristic affix stripping never substitutes (ADR-0202 §8.1).

Additional planning inputs from code inspection (2026-09-17):

- Read paths currently pool-query rather than pin a transaction
  (crates/storage-sqlite/src/lib.rs:195) — consistent input snapshots are a prerequisite.
- Quran activation increments its own pointer generation
  (crates/storage-sqlite/src/quran.rs:682) — do not equate with `corpus_generations`.
- The outbox relay records dispatch without real consumers
  (crates/storage/src/workflows.rs:112) — graph delivery needs durable fan-out.
- The index builder (crates/application/src/quran_index.rs:262) is a pattern to reuse with
  fixes: atomic build reservation, input revalidation before publication, fenced CAS
  pointer flip, and real content hashing.
- Worker checkpoints are saved on failure, not continuously
  (crates/jobs/src/worker.rs:128) — graph builds need durable staged-batch progress.
- The only web view is the debug reader (crates/server/src/api.rs:694) — visualization is
  real deliverable work, not assumed infrastructure.

## 2. In scope (this phase)

- `GraphStore` port + SQLite adjacency adapter, using bounded batched frontier traversal
  first. Recursive CTE execution is optional and must demonstrate equivalent budgets and
  cancellation before activation; two execution strategies are not a release prerequisite.
- Structural projection: edition, surah, ayah, token, division nodes; `CONTAINS`, `NEXT`
  (PRD alias `PRECEDES`/`FOLLOWS` recorded as aliases, not competing semantics).
- Word-root projection (root/lemma/word-form/token/ayah) from Phase-2 morphology records.
- Concept/topic and named-entity annotated projections.
- Authoritative annotation service: scholar/user assertions, evidence, review decisions,
  supersession — separate from disposable adjacency.
- Bounded neighbors, paths (reachability / min-hop / up-to-K), subgraphs, allowlisted
  typed patterns.
- Provenance on every non-structural edge (PRD §10.3); explainability on every result
  (PRD §10.5).
- CLI commands, HTTP routes, typed tools (`quran.graph_neighbors`, `graph_path`,
  `graph_subgraph`, `graph_pattern`), Graph JSON export, GraphML where practical.
- Basic local visualization (bounded exploration, evidence inspection, provenance labels,
  RTL, accessible non-graph alternative).
- Per-projection manifests, `qai doctor` drift checks, rebuild/delete-verify, cancellation,
  recovery, GC that preserves authority.
- Backend conformance suite (D4.6) gating any future adapter.

## 3. Out of scope (this phase)

- CozoDB / sqlite-graph adapter spikes (TASK-411/412) remain optional go/no-go items and
  may slip without blocking phase exit.
- Hadith, tafsir, isnad, comparative-scripture ingestion (Phases 5–8).
- LLM/model-generated relationship discovery (canonical path stays model-free, I2).
- Production auth/RBAC/TLS (Phase 11); local authorization context only.
- Full Phase-4 graph explorer, saved-workspace integration, TUI screens.
- Unrestricted SQL/Cypher/Datalog exposure (forbidden by ADR-0202).

## 4. Crate ownership & boundaries

| Crate | Responsibility | Must not |
|---|---|---|
| `domain` | Shared graph IDs, revision/snapshot primitives (extended) | gain I/O |
| `graph` | Backend-neutral types, query contracts, budgets, completion semantics, conformance harness | depend on SQLx, quran-core, or any Quran type |
| `quran-graph` | Quran schema, deterministic projectors (structural, linguistic, concept), family validation | depend on a concrete backend |
| `storage` | Assertion/review/projection-catalog repository traits, UoW additions | depend on `graph` (map DTOs at adapter boundary) |
| `storage-sqlite` | Real schemas, adjacency execution, true read snapshots | leak SQL above the port |
| `application` | Composition, snapshot capture, build lifecycle, policy, job handlers, service APIs | — |
| `server` / `cli` / `tool-registry` | Thin transport/tool adapters over application services | implement traversal logic |

- `xtask/allowlist.toml` gains reviewed dependency edges as an explicit M0 task
  (currently no graph sections exist; unlisted crates get empty dependency sets).
- Canonical Arabic text is never copied into graph payloads — references + hashes only
  (ADR-0202 §8.1; quotations resolve through the reader).
- Backend query languages never escape `GraphStore`.

## 5. Milestones

Dependency order:

```text
M0 → M1 → M2 → M3
M1 → M4 authority; M3 + authority → M4 projections
M3 + Phase-2 morphology → M5
M3 + M4 + M5 → M6
M1 + M2 → M7 foundations; M6 + recovery evidence → full exit
```

### M0 — Contracts, decisions, gates (TASK-413, TASK-414)

- Ratify ADR-0202 (`Proposed` → `Accepted`) and the Phase-3 subset of ADR-0702.
- Define schema versioning, stable node/assertion identity, allowed endpoint pairs,
  direction rules, duplicate-edge identity (multiple assertions per triple preserved).
- Reconcile `PRECEDES`/`FOLLOWS` vs `NEXT`; record alias policy.
- Define path modes, completion/truncation semantics, budget defaults (recorded in the
  graph query safety ADR the PRD requests at requirements.md:6639 — drafted as
  `ADR-0217` (Proposed, owner ratification = P4-X03); defaults and "truncated ≠ empty"
  semantics already implemented in `quran-graph`'s `QueryBudgets`).
- Define local authorization context, review permissions, revocation, and pinned
  historical-edition read semantics.
- Update allowlist + crate manifests; write the requirement-to-task matrix.
- **Exit:** accepted decisions, allowlist green, no unnamed upstream blocker.

### M1 — Foundations: snapshots, migrations, lifecycle (TASK-401 plus TASK-415–418)

- `GraphStore` trait: node resolution, neighbors, bounded paths/subgraphs/patterns,
  projection build/inspect/delete-verify, capability discovery. Snapshot + authz + budgets
  + deadline on every query; incomplete ≠ empty.
- Source revision vs dependency snapshot vs build attempt vs published projection — four
  distinct identities.
- True read transactions (fix pooled `ReadTx`) + bounded bulk canonical reads.
- Durable dependency snapshots (restart-safe inputs).
- New migrations: assertion authority, snapshots, build catalog, adjacency, delivery
  ledger — numbers allocated from `migrations/sqlite/` at implementation time; atomic
  DDL + ledger recording (harden runner if needed: storage-sqlite/src/migrate.rs:168).
- Atomic build reservation; fenced CAS publication.
- Atomically bridge Quran activation/rollback and relevant source changes to graph
  dependency revisions, audit and outbox intent. Do not conflate existing generation counters.
- Durable outbox fan-out with per-target delivery; lease fencing for graph jobs.
- **Exit:** no partial commits on injected failure; expired workers cannot publish;
  snapshot reads never mix revisions.

### M2 — Structural projection (TASK-404, TASK-419)

- Nodes: edition, surah, ayah, token, division. Edges: `CONTAINS`, `NEXT`
  (surah-boundary behavior explicit).
- Lifecycle: capture validated inputs → reserve unique build → staged resumable batches →
  validate identities/endpoints/membership/canonical hashes → recheck dependencies +
  ownership → atomic publication → retain previous verified build.
- Durable staged-batch progress (not in-memory checkpoints).
- **Exit:** deterministic structural graph; zero dangling edges; cancellation preserves the
  prior publication; canonical lookup works with the projection missing.
- **First usable slice after TASK-402:** build → inspect → bounded neighbors → canonical
  quotation. M2 alone delivers internal build/inspection, not public traversal commands.

### M3 — Bounded traversal + conformance (TASK-402, TASK-403, TASK-409, TASK-420)

- Neighbors, filtered subgraphs, reachability / min-hop / up-to-K paths, small typed
  patterns (allowlisted predicates, parameterized values).
- Budgets enforced **during expansion** (depth, expanded nodes/edges, frontier, results,
  pattern size, wall clock) — never a final `LIMIT` only.
- Cancellation stops backend work (SQLite interrupt/progress in-adapter).
- Deterministic ordering; direction semantics explicit.
- Authz/source-restriction/tombstone filtering applied to intermediates and evidence, not
  just returned nodes.
- Backend-agnostic conformance suite: cycles, high-degree hubs, parallel edges, budget
  exhaustion ≠ "no path", filtered-intermediate leakage tests.
- **Exit:** cyclic/high-degree fixtures terminate within limits; hidden intermediates
  cannot influence disclosed paths or counts.

### M4 — Annotations, concepts, entities (TASK-408, TASK-421–423)

- Immutable/versioned assertion records + evidence links + review decisions, stored
  authoritatively and separately from adjacency.
- User-authored and attributed scholarly relationships (`CITES`, `CONTRASTS_WITH`,
  `PARALLELS`, `EXPLAINS`, …); concept/topic and named-entity annotations.
- Accept / reject / correct / supersede workflows; reviewer + decision + timestamp
  retained (PRD §10.6).
- Original algorithm/dataset attribution preserved alongside review decisions.
- Multiple contradictory / independently sourced assertions coexist — never merged.
- Assertion + audit + revision + outbox commit atomically (one UoW).
- No inference engine in this phase; computational suggestions are representable, never
  generated here.
- **Exit:** every non-structural edge resolves to attribution + evidence; rebuilds
  preserve annotations and review history; unreviewed suggestions can never become
  verified automatically.

### M5 — Linguistic projections + root family (TASK-405, TASK-406, TASK-407) — ⊘ blocked on Phase 2 morphology

- Dataset-qualified root/lemma/form/analysis identities from attributed Phase-2 records
  with token alignment; competing analyses stay separate edges (`HAS_ROOT`, `HAS_LEMMA`,
  `INFLECTS_TO`, `ROOT_OF`).
- Root hub topology: `Token —HAS_ROOT→ Root ←HAS_ROOT— Token`, `Token —IN_AYAH→ Ayah`.
  No ayah×ayah materialization (ADR-0202 §8.1). Note: root→ayah is **two** traversal edges.
- Root-family service: ranked ayahs with provenance, dataset versions, expansion counts;
  deterministic documented ranking (not implied scholarly confidence).
- Independent linguistic dependency manifests + drift detection.
- **Exit:** competing analyses never merge; dataset-version change trips drift; reviewed
  golden families (ق و ل، ك ت ب، ع ل م) pass. Synthetic fixtures prove mechanics only.

### M6 — Interfaces, export, visualization (TASK-424–428)

- CLI: `qai graph build|inspect|neighbors|path|subgraph|pattern|root-family|export` — new
  grammar + dispatch (none exists today).
- Read-only tools: `quran.graph_neighbors`, `quran.graph_path`, `quran.graph_subgraph`,
  `quran.graph_pattern`, root-family and `quran.entity_timeline` adapters (PRD §11.5,
  requirements.md:789–807), with `ToolResult` reproducibility binding query, visibility,
  projection/build/dataset versions and budgets. Build management stays CLI-only.
- HTTP: read-only versioned routes for neighbors/path/subgraph/pattern/root-family,
  entity timeline and inspection (neighbors/path/query per PRD §27.1), with explicit transport
  semantics for missing-projection vs empty, incomplete, stale, and refused queries.
  ETags/cache keys bind projection + dependency versions + budgets + visibility — not the
  Quran text hash alone.
- Graph JSON export (identities, paths, attribution, versions, truncation); GraphML where
  practical (PRD §24.4). Format decision drafted as `ADR-0218` (Proposed; owner
  ratification = P4-X05): `quran-graph-json-v1` is the mandatory lossless format and
  GraphML is a declared-lossy derived view.
- Basic local visualization: bounded exploration, evidence inspection, provenance-category
  labels, RTL Arabic, accessible table/list alternative, working reader navigation.
- **Exit:** CLI/API/tool semantic parity; no unbounded dump; exports cannot leak
  restricted evidence.

### M7 — Operations, recovery, exit (TASK-410, TASK-429, TASK-430)

- Read-only per-projection doctor checks (missing inputs, stale deps, dangling refs,
  tombstone policy) + explicit rebuild/repair commands; doctor never repairs implicitly.
- Worker registration so builds actually run under the job system (serve wires reader/API
  only today; generic `job` CLI is a stub).
- GC preserving authority + pinned readers; restore/rebuild verification.
- Performance measurements on representative graphs; cancellation tests; progress events.
- **Exit:** recovery verified; runbook recorded; unresolved editorial gates remain
  explicitly pending with named owners.

## 6. Acceptance criteria

See `acceptance.md` (AC-P4-01 … AC-P4-28), including a criterion-to-task ownership matrix. Highlights:

1. Canonical text/structure/token-order integrity preserved by every build.
2. Edition and competing-analysis identities never silently merge.
3. Rebuilds preserve original assertions, evidence, review history.
4. Identical pinned inputs → identical semantic graph content (timing excluded).
5. Partial/cancelled/stale builds never replace verified publication.
6. Duplicate delivery/restart do not duplicate logical records.
7. Every query bounded; incomplete ≠ empty.
8. Access restrictions/tombstones apply during traversal and export.
9. Root-family results carry attributed, reviewed evidence.
10. Doctor read-only; repairs separate + confirmed.
11. Basic visualization exposes provenance and limits.
12. Canonical reading works when graph projection is unavailable.

## 7. Verification gates

Per AGENTS.md / CONTRIBUTING.md, for every task:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace            # serial for application if parallel flakes
cargo xtask arch-check
cargo xtask migrate-check
cargo xtask ci                    # full 9-step gate at milestones
```

Plus phase-specific suites: graph conformance, snapshot/race/crash-fault-injection,
determinism, CLI trycmd, HTTP contract + OpenAPI, doctor immutability, and (M6)
browser/accessibility checks for visualization.

## 8. Risks & known traps

- **Trap 5 (briefing):** don't renumber phases; this directory is PRD Phase 3.
- **Trap 4:** allocate migrations from the live tree, never plan tables.
- Morphology dependency: M5 cannot start until ADR-0203-backed datasets land in Phase 2;
  do not fake roots from normalization.
- Estimate honesty: M1/M4 are consistently underestimated classes of work; re-estimate
  after M1 spike, record actuals in `done.md`, flag gaps as owner decisions (trap 10).
- Concurrent writers: run `git status` before touching shared files
  (storage-sqlite UoW lists, allowlist, migrations).
