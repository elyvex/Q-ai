# Phase 4: Quran Graph - Context

**Gathered:** 2026-09-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Turn the canonical corpus + search/linguistics layers into a **navigable, provenance-carrying Quran knowledge graph**: structural (edition/surah/ayah/token/division) + linguistic word-root (root/lemma/word-form/token/ayah) + annotated (concept/topic, named-entity, translation-link, and interpretive cross-reference edges) projections behind the backend-neutral `GraphStore` port — with bounded neighbors/paths/subgraphs/patterns, full review workflow for annotations, explainability on every result, and bounded Graph JSON export. Reachable through the `qai` CLI, the versioned HTTP API, and typed tools with identical results.

This phase is **evidence-driven brownfield hardening + gap closure over the legacy board `docs/03-plan/phases/phase-04-quran-graph/`** (note the numbering drift: that board is titled "Phase 3 / P4" but covers roadmap Phase 4; 30 tasks TASK-401–430, 10 code-landed-partial, 28 acceptance criteria AC-P4-01…28 unverified). It preserves the substantial working code already present (`quran-graph` port, model, budgets, traversal, patterns, mem backend, structural builder, export) and creates work only for missing behavior or weak evidence — above all the application-layer SQLite adapter, services, CLI/HTTP/tool surfaces, annotation lifecycle, manifests/doctor, and budgets enforcement.

**Out of scope:** tafsir/hadith/isnad graph links (roadmap Phases 6–7); the interactive graph explorer / Web GUI / TUI (roadmap Phase 5 — Phase 4 visualization is static local rendering only); LLM-generated relationships (canonical path stays model-free); production auth/RBAC (Phase 11); CozoDB / sqlite-graph-extension adapters beyond optional spikes; GraphML (deferred, lossy view only); any change to canonical rows, reference grammar, morphology invariants, or frozen hash recipes.

**Boundary contract:** `.planning/ROADMAP.md` §Phase 4 (four success criteria) plus `REQ-quran-graph`. The legacy board and its ADRs are authoritative **evidence/reference**, not a mandate to close every legacy task.

</domain>

<decisions>
## Implementation Decisions

### Scope, Posture & Finish Line
- **D-01:** Treat Phase 4 as an evidence-driven brownfield gap-closure phase (mirrors Phase 1 D-01 / Phase 2 D-01 / Phase 3 D-04): map each of the four success criteria to current code + at least one repeatable check, preserve working implementations, and create implementation work only for missing behavior or weak evidence. — **Reversibility:** reversible — a planning-boundary choice.
- **D-02:** Content scope is **structural + word-root + concepts/entities + translation links**. Structural (edition/surah/ayah/token/division, `CONTAINS`/`NEXT`) and word-root (root/lemma/word-form/token/ayah) always; concept/topic + named-entity nodes as annotated projections; `TRANSLATES` edges to existing translation editions with the ADR-0112 layer separation intact (refs-only, never text). Tafsir/hadith/isnad links are deferred to roadmap Phases 6–7. — **Reversibility:** costly — projection tables, the CLI/HTTP/tool surface, and export shape are consumed by Phase 5 UI and later graph phases.
- **D-03:** Word-root projection builds **only from active, attributed morphology records** (synthetic fixture in tests; user-supplied records via the Phase 3 morphology import path when present; typed "no dataset active" behavior otherwise — never heuristics, never guessed roots). OD-11 (dataset/license) and OD-12 (linguist) remain recorded BLOCKED owner gates; this phase does not re-litigate them. Exact build mechanics are agent discretion within invariants I11/I12/I13 + alignment (Phase 3 D-09). — **Reversibility:** costly — the license/attribution metadata and activation gate feed provenance, derived-index manifests, and export.
- **D-04:** Concept/entity nodes come from a **curated, versioned seed list** (major themes, persons, places) committed as a fixture, extended by user/scholar additions through the annotation workflow. Nothing claims scholarly authority beyond its attribution.
- **D-05:** Interpretive cross-reference edges (`PARALLELS`, `CONTRASTS_WITH`, `EXPLAINS`, `RELATED_TO`, plus `SUPPORTED_BY`/`DISPUTED_BY`) are **in scope as annotated edges** with full provenance + review history, per PRD §10.6 and legacy D4.5. — **Reversibility:** costly — edge types and the assertion/evidence shape are a published contract consumed by export and later phases.

### Annotation & Review Workflow
- **D-06:** **Full review queue**: researchers manually create typed edges; algorithmically suggested edges queue with supporting evidence shown, then accept / reject / correct with reviewer + timestamp recorded. No computational suggestion becomes a verified edge without an explicit human decision. — **Reversibility:** costly — the review lifecycle shapes the annotation service, CLI commands, and audit events.
- **D-07:** Disagreement and history via **verification statuses + supersession**: edges carry `verification_status` (unverified/verified/disputed/superseded/rejected); corrections create new assertions and supersede old ones rather than overwriting; `SUPPORTED_BY`/`DISPUTED_BY` are first-class edges. Rebuilding the graph projection never discards human annotations or review history. — **Reversibility:** costly — the history shape is consumed by traversal filters, export, and later isnad/scholarly phases.
- **D-08:** Rejected/superseded assertions are **tombstoned** (hidden from traversal and export immediately, retained for audit), with explicit repair/GC commands and per-projection manifests + read-only doctor checks. Follows legacy D4.9. — **Reversibility:** costly — tombstone semantics touch traversal, export, and storage GC.

### Traversal, Budgets & Patterns
- **D-09:** Ship **all path modes**: reachability check, shortest (min-hop) path, and up-to-K ranked paths — covering roadmap SC2 and PRD §10.4 path queries. Every mode honors budgets and the "no path requires completeness" rule. — **Reversibility:** costly — path modes are API/tool surface consumed by Phase 5 and later retrieval.
- **D-10:** Budget values, truncation rendering, and the pattern set are agent discretion **within hard rules**: ADR-0217 ranges and semantics (pre-flight violation is an error; in-flight exhaustion is a typed partial with `truncated` + non-empty `incomplete_reason`; truncation is never rendered as absence); no backend query language crosses the port. Defaults already implemented in `quran-graph` (6 hops / 500 nodes / 2000 edges / 128 fanout / 10 paths / 5s) are the starting point.

### Surfaces, Parity & Visualization
- **D-11:** **Full CLI tree**: `qai graph neighbors/path/subgraph/export` reads plus `build/inspect/review/doctor-repair` management. Build management is CLI-only; **no HTTP mutation routes and no new mutation agent tools** in this phase (legacy scope fence). — **Reversibility:** costly — CLI verbs are a versioned operator surface.
- **D-12:** **Full read parity**: every CLI read op has an HTTP route and a typed tool (`quran.graph_neighbors`, `graph_path`, …) with identical results under the versioned `Envelope` + `ToolResult`/reproducibility contract. — **Reversibility:** costly — routes and tool schemas are consumed by Phase 5 UI and the agent runtime.
- **D-13:** Basic local visualization is **static rendering only**: Graph JSON v1 file plus a static SVG/DOT rendering of a neighborhood/subgraph for local inspection. No interactive explorer — that belongs to Phase 5. — **Reversibility:** reversible — rendering is additive and deferred UI is untouched.
- **D-14:** **Full PRD §10.5 explainability** on every result in both human and JSON output: start/end nodes, traversed path, edge types, edge provenance, confidence, applied filters, query duration. — **Reversibility:** costly — the explainability payload is a published result contract downstream agents and later UI consume (extends Phase 3 D-11).

### Agent's Discretion
- Word-root projection build mechanics (batching, job checkpoints, index layout) so long as morphology invariants I11/I12/I13 + alignment hold and canonical bytes are never mutated.
- Edge authorship/attribution model (single operator vs attributed scholars) so long as every non-structural edge carries PRD §10.3 provenance and deny-by-default + approval-gated canonical rules hold.
- Exact budget values within ADR-0217 ranges; truncation UX details so long as partial results are never rendered as absence; pattern allowlist vs custom typed patterns so long as no query language crosses the port.
- Checkpoint payload schemas, backoff constants, fixture naming, CLI flag naming, and operator-facing wording may follow existing project conventions (Phase 1 D-16 convention) so long as the locked behavior above is preserved.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Contract
- `.planning/ROADMAP.md` §Phase 4 — goal, four success criteria, dependency on Phase 3, and phase boundary.
- `.planning/REQUIREMENTS.md` — `REQ-quran-graph` (PRD §10); note `REQ-quran-display` / `REQ-quran-research-tools` / `REQ-quran-result-contract` / `REQ-tui-cli` belong to Phase 5, hadith/tafsir to Phase 6, isnad to Phase 7.
- `.planning/PROJECT.md` — 25 locked ADRs, constraints, out-of-scope, local-first direction; proposed ADRs incl. ADR-0202/0217/0218/0702.
- `.planning/STATE.md` — current position and the Phase-3→4 handoff (OD-11, OD-12 BLOCKED).
- `docs/01-requirements/requirements.md` — authoritative living PRD: §10 graph (§10.1 node types, §10.2 edge types, §10.3 provenance, §10.4 search, §10.5 explainability, §10.6 annotation), §11.5 graph tools, §24.4 export, §46 completeness rule.

### Legacy Phase Board (authoritative evidence — numbering drift: "Phase 3 / P4" = roadmap Phase 4)
- `docs/03-plan/phases/phase-04-quran-graph/README.md` — objective, backend policy, deliverables D4.1–D4.9, scope summary, task totals (do not renumber).
- `docs/03-plan/phases/phase-04-quran-graph/plan.md` — full plan: findings, crate ownership, milestones M0–M7, risks, critical path.
- `docs/03-plan/phases/phase-04-quran-graph/tasks.md` — task board TASK-401–430 + swimlane X.
- `docs/03-plan/phases/phase-04-quran-graph/acceptance.md` — AC-P4-01…28 matrix, required suites, DoD, exit gate.
- `docs/03-plan/phases/phase-04-quran-graph/done.md` — append-only completion ledger (10 code-landed-partial).

### Decided Graph Architecture (Proposed — ratification tracked by legacy swimlane X)
- `docs/02-architecture/decisions/ADR-0202-graph-store.md` — relational adjacency + bounded traversal first; `GraphStore` port; authority-vs-projection; no query-language leakage; backend policy incl. optional CozoDB / sqlite-graph spikes.
- `docs/02-architecture/decisions/ADR-0217-graph-query-limits.md` — budget defaults/ranges, pre-flight vs in-flight semantics, no-path-requires-completeness, determinism (implemented in `crates/quran-graph/src/model.rs`, `traverse.rs`, `pattern.rs`, `mem.rs`).
- `docs/02-architecture/decisions/ADR-0218-graph-export-formats.md` — Graph JSON v1 mandatory envelope, assertions-travel-with-edges, explicit truncation, policy-on-export, GraphML deferred (implemented in `crates/quran-graph/src/export.rs`).

### Locked / Core Architecture Decisions
- `docs/02-architecture/decisions/ADR-0000-project-architecture.md` — layered workspace, single `qai` binary, local-first.
- `docs/02-architecture/decisions/ADR-0001-relational-store.md` — SQLite authority, PostgreSQL-portable contracts; graph structures are rebuildable projections.
- `docs/02-architecture/decisions/ADR-0002-migration-strategy.md` — append-only checksummed migrations, forward-only canonical policy.
- `docs/02-architecture/decisions/ADR-0003-durable-job-system.md` — DB-backed leased jobs; graph builds need staged-batch progress + cancellation.
- `docs/02-architecture/decisions/ADR-0006-hashing-canonical.md`, `ADR-0007-manifest-format-signing.md`, `ADR-0008-provenance-representation.md`, `ADR-0009-audit-integrity.md`, `ADR-0010-error-taxonomy.md`, `ADR-0012-workspace-boundaries.md` — hashing, manifests, provenance, audit, error codes, crate boundaries this phase must respect.
- `docs/02-architecture/decisions/ADR-0104-unicode-policy.md` — NFC storage, grapheme counts with byte offsets.
- `docs/02-architecture/decisions/ADR-0105-canonical-tokenization.md` — whitespace-preserving surface tokens; alignment target for linguistic nodes.
- `docs/02-architecture/decisions/ADR-0106-canonical-storage-layout.md` — canonical vs staging vs derived table layout.
- `docs/02-architecture/decisions/ADR-0107-atomic-activation-rollback.md` — pointer flip + generation bump; graph projections are generation-stamped.
- `docs/02-architecture/decisions/ADR-0108-corpus-hashing-scheme.md` — frozen `qai-text-hash-v1` recipes; additive domain-separated recipes only.
- `docs/02-architecture/decisions/ADR-0111-citation-identity-and-deep-links.md` — citation identity/URNs; graph neighbor views link to pinned canonical refs.
- `docs/02-architecture/decisions/ADR-0112-translation-alignment-and-attribution.md` — translation layer separation; `TRANSLATES` edges stay refs-only.
- `docs/02-architecture/decisions/ADR-0203-quran-morphology-dataset.md` — **Draft**; morphology provider, alignment guarantee, license rule, Option B fallback (word-root data source).
- `docs/02-architecture/decisions/ADR-0204-normalization-rules.md`, `ADR-0205-profile-ladder.md`, `ADR-0210-root-convention.md`, `ADR-0211-counting-rules.md`, `ADR-0215-tagset.md` — linguistic conventions the word-root projection inherits.
- `docs/02-architecture/decisions/ADR-0213-index-generations.md` — generation stamping, atomic activation, drift (pattern for graph projection manifests).

### Open / Owner-Gated Decisions
- `docs/05-followups/decisions-needed.md` — **OD-11** (morphology dataset/license) and **OD-12** (normalization catalog + linguist), both human-only and BLOCKED; plus any P4-X01/X02/X03/X05 ratification items the researcher confirms.
- `docs/05-followups/owner-decisions.md` — owner's recorded architectural intent.
- `docs/02-architecture/upstream-sources.md` — verified upstream facts/revisions.

### Specs & Technical References
- `docs/07-technical/quran-citation-spec.md` — citation identity and resolution (graph hits link to pinned refs).
- `docs/08-api/quran-v1-openapi.json` — versioned API envelope conventions new graph routes must follow.
- `docs/06-progress/task-done-rollup.md` — reconciliation record.

### Codebase Maps & Fixtures
- `.planning/codebase/ARCHITECTURE.md`, `.planning/codebase/STRUCTURE.md` — composition root, port/adapter rules, where new code goes.
- `fixtures/quran/test-edition-min/`, `fixtures/quran/adversarial/`, `fixtures/quran/golden/`, `fixtures/quran/lexicon/` — synthetic corpora and labeled test lexicon this phase exercises.

### Prior Phases
- `.planning/phases/03-quran-search-linguistics/03-CONTEXT.md` — Phase 3 decisions inherited (D-05/D-07/D-08/D-09/D-10/D-11/D-15, owner-gate posture).
- `.planning/phases/02-canonical-quran-core/02-CONTEXT.md` — canonical safety model, fixture-complete posture, frozen recipes.
- `.planning/phases/01-foundations/01-CONTEXT.md` — brownfield gap-closure pattern, operator bootstrap, audit/job lifecycle rules.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/quran-graph` — typed graph contract already code-landed-partial: `model.rs` (node/edge types, `QueryBudgets`), `store.rs` (`GraphStore` port), `traverse.rs` (bounded traversal), `pattern.rs` (typed patterns), `mem.rs` (`MemGraphStore` reference backend + conformance target), `structural.rs` (pure structural builder), `export.rs` (Graph JSON v1 + truncation notice), `error.rs`. The decided production SQLite adapter lives in the application layer and is **not yet implemented**; it must implement the same port and pass the same conformance suite.
- `crates/graph` — still an empty `//! Phase N` placeholder; scope ownership vs `quran-graph` needs an explicit decision during planning (do not implement early without resolving).
- `crates/quran-morphology` — pure lexicon types + adapters + alignment + family relations; bundles no dataset (word-root projection input contract).
- `crates/quran-normalization` / `crates/quran-search` — derived-layer pattern to mirror: read-only over canonical, generation-stamped rebuildable projections, explainability payloads.
- `crates/application` — composition root where the SQLite graph adapter, projection builders, annotation service, review workflow, export, and doctor checks land (new `quran_graph*.rs` service modules).
- `crates/storage` / `crates/storage-sqlite` — port traits + sole adapter; new graph repositories + append-only migrations follow the established `NNNN_*.up.sql` + `checksums.json` flow.
- `crates/cli` / `crates/server` / `crates/tool-registry` — thin edges: `qai graph …` tree, Axum routes under the versioned envelope, typed tools with `ToolResult`/reproducibility data.

### Established Patterns
- Ports-and-adapters with backend traits (`GraphStore`, `QuranApiBackend`, `QuranBackend`); fakes in contract tests; **no backend query language crosses a port** (pinned by test).
- Bounded, budgeted, cancellable traversal with typed incomplete results; determinism (equal budgets + equal visible input → equal ordering).
- Graph records carry **references + hashes only** — no canonical Arabic text in nodes/edges/exports (resolved through the reader).
- Generation-stamped rebuildable projections; authority (assertions, review history) vs projection (adjacency) separation; tombstones hidden immediately, history retained.
- Approval-gated canonical writes; atomic domain + provenance + audit + outbox commits; typed `QAI-*` diagnostics mapped to stable CLI exit codes.
- Purity fence: graph model/traversal crates must never gain `llm`/`embeddings`/`retrieval`/vector deps (`xtask/allowlist.toml`).

### Integration Points
- `crates/application/src/` — new graph services (`quran_graph*.rs`), wiring through `lib.rs`, CLI impls via `quran_cli.rs` free functions.
- `crates/cli/src/` — `qai graph` command tree + `doctor.rs` graph checks with remedies and next commands.
- `crates/server/src/api.rs` — graph read routes under `/api/v1` envelope conventions.
- `crates/tool-registry/src/lib.rs` + `crates/application/src/quran_tools.rs` — typed graph tools with attribution.
- `migrations/sqlite/` — new forward-only graph migration(s) (next free number, never edit applied migrations).
- `crates/application/src/quran_doctor.rs` — per-projection manifest verification + explicit repair surface.

</code_context>

<specifics>
## Specific Ideas

- Begin planning with an **evidence matrix** (criterion → existing implementation → repeatable evidence → gap → planned task if needed), mirroring Phases 1–3.
- Owner gates (**OD-11** dataset/license, **OD-12** linguist, plus P4-X ratifications the researcher confirms) must appear in every relevant plan as explicit **BLOCKED** items with the exact command/step needed to close them — never a silent pass.
- Enforce graph invariants as **tests, not comments**: budget pre-flight errors, in-flight typed incompletes, no-path-requires-completeness, assertions-travel-with-edges, no-canonical-text-in-graph-records, tombstone invisibility, no-query-language leakage, generation-stamp freshness.
- The first usable vertical slice is **build → inspect → bounded neighbors → canonical quotation**: projection build from the synthetic fixture, manifest inspection, neighbor view around a verse, each hit linking to a pinned canonical ref.
- Label heuristic/computational suggestions explicitly and keep them out of verified results until a recorded human decision accepts them.

</specifics>

<deferred>
## Deferred Ideas

- Tafsir / hadith / isnad graph links and ingestion → roadmap Phases 6–7.
- Interactive graph explorer, Web GUI reading/research views, TUI cockpit → roadmap Phase 5.
- CozoDB and sqlite-graph-extension adapter spikes (legacy TASK-411/412) → optional within this phase only as go/no-go spikes behind the same port; SQLite remains the single source of truth either way.
- GraphML export → deferred lossy view generated from Graph JSON v1, per ADR-0218 (never the authoritative form).
- Production auth/RBAC, TLS, server management → roadmap Phase 11; PostgreSQL/Qdrant adapters, Docker, backups → Phase 12.
- Additional Quran qira'at / editions beyond the active-edition model → their roadmap phases.

</deferred>

---

*Phase: 4-Quran Graph*
*Context gathered: 2026-09-28*
