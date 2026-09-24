# Roadmap: Q-ai

## Overview

From an empty workspace to a trustworthy local-first Quran research platform: foundations and provenance first, then the canonical Quran engine with verifiable citations, then linguistics, graph, and rich interfaces, then hadith/tafsir/isnad, then multi-RAG and comparative scripture, then controlled agentic research, and finally server management and production hardening. Every phase passes the PRD quality gates and definition-of-done (§46/§58/§90/§93) before the next begins. MVP acceptance (§44.3) is met at Phase 10; Phases 11–12 are post-MVP.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Foundations** - Workspace, config, SQLite, jobs, provenance, audit, CLI skeleton
- [ ] **Phase 2: Canonical Quran Core** - Validated import, addressing, tokens, exact lookup, integrity
- [ ] **Phase 3: Quran Search & Linguistics** - Normalization, roots/lemmas, morphology, frequency tools
- [ ] **Phase 4: Quran Graph** - Graph store, traversal, annotations, visualization
- [ ] **Phase 5: Rich Quran Experience** - Web GUI, TUI cockpit, research tools, result contract
- [ ] **Phase 6: Hadith & Tafsir** - Structured records, grading, verse links, tafsir indexing
- [ ] **Phase 7: Isnad & Narrator Research** - Identity review, chain paths, uncertainty
- [ ] **Phase 8: Multi-RAG Platform** - Full-text, vector, hybrid, reranking, generic RAG projects
- [ ] **Phase 9: Comparative Scripture** - Edition-aware scripture model, parallel-passage research
- [ ] **Phase 10: Agentic Research & Tool Creation** - Agents, tool sandbox, workflows, MVP acceptance
- [ ] **Phase 11: Server & Management** - Versioned API, streaming, auth, catalog, governance
- [ ] **Phase 12: Production Hardening** - PG/Qdrant adapters, TLS, Docker, backups, recovery

## Phase Details

### Phase 1: Foundations
**Goal**: A developer can build the workspace, configure it, and run `qai` with provenance-safe storage and background jobs.
**Depends on**: Nothing (first phase)
**Requirements**: REQ-product-vision, REQ-product-principles, REQ-goals-non-goals, REQ-engineering-baseline, REQ-storage-architecture, REQ-cli-api, REQ-architecture-principles-quality
**Success Criteria** (what must be TRUE):
  1. Developer can build the workspace and run `qai --help` showing the stable command tree
  2. Operator can configure via CLI > env > file > defaults with validation errors that name the remedy
  3. System records provenance and append-only audit events for every state-changing operation
  4. Background jobs enqueue, lease, checkpoint, and cancel without an external broker
  5. `xtask arch-check` passes and CI fails on any forbidden crate dependency
**Plans**: 5 plans

Plans:
- [ ] 01-01-PLAN.md — Fresh-workspace operator readiness tracer: config origins, explicit migration, read-only doctor, and persisted audit verification
- [ ] 01-02-PLAN.md — Audited durable mutations with same-UnitOfWork provenance/audit/outbox coverage and rollback proof
- [ ] 01-03-PLAN.md — Durable job checkpoints, per-kind retry, cooperative cancellation, and lifecycle audit controls
- [ ] 01-04-PLAN.md — Long-lived `qai serve` worker host with safe shutdown and enqueue-only one-shot import boundary
- [ ] 01-05-PLAN.md — Registry/git architecture enforcement, final evidence gates, and TASK-001 repository closure

### Phase 2: Canonical Quran Core
**Goal**: One validated Quran edition is importable, addressable, and provably immutable.
**Depends on**: Phase 1
**Requirements**: REQ-quran-corpus, REQ-data-separation-layers, REQ-ingestion-validation-eval
**Success Criteria** (what must be TRUE):
  1. Operator can import a Quran edition through staging → validation → atomic activation with rollback
  2. User can look up any surah:ayah and receive byte-exact canonical Arabic with pinned edition reference
  3. Corpus integrity checks (counts, addressing, Unicode, checksums, round-trip, reference comparison) pass
  4. Canonical tables reject all non-approved writes; importer has no code path to canonical tables
  5. Every quotation verifies via `verify_quotation` with mismatch as a hard failure
**Plans**: TBD

### Phase 3: Quran Search & Linguistics
**Goal**: Users can find and analyze Quranic words across orthographic variation without ever seeing altered display text.
**Depends on**: Phase 2
**Requirements**: REQ-quran-normalization, REQ-quran-linguistics
**Success Criteria** (what must be TRUE):
  1. User can search an Arabic phrase without diacritics and get exact-location hits
  2. User can search a concatenated (spaceless) phrase and get exact-location hits
  3. User can select a word and inspect its lemma, root, morphological analyses, and word family
  4. User can view frequency, distribution, and co-occurrence for any root or lemma
  5. Displayed canonical text is never modified by normalization in any result
**Plans**: TBD

### Phase 4: Quran Graph
**Goal**: Users can explore structural and linguistic relationships of the Quran as a navigable graph.
**Depends on**: Phase 3
**Requirements**: REQ-quran-graph
**Success Criteria** (what must be TRUE):
  1. User can open a neighbor view around any verse, word, root, or concept
  2. User can find paths between two graph nodes with each edge's provenance shown
  3. User can export a subgraph with edge provenance intact
  4. Graph queries explain why each result was returned
**Plans**: TBD

### Phase 5: Rich Quran Experience
**Goal**: Users can read, research, and cite the Quran from the Web GUI, TUI, and CLI with typed tools.
**Depends on**: Phase 4
**Requirements**: REQ-quran-display, REQ-quran-research-tools, REQ-quran-result-contract, REQ-tui-cli
**Success Criteria** (what must be TRUE):
  1. User can read Arabic with RTL layout, translation panels, word inspector, and deep links
  2. User can copy a citation that re-verifies at the exact source location when opened
  3. User can drive every research tool from the UI, API, and CLI with identical results + research checksum
  4. Operator can use the TUI cockpit (command palette, RAG debug view) for daily operations
  5. Canonical text, translations, and annotations are visually unmistakable in every view
**Plans**: TBD
**UI hint**: yes

### Phase 6: Hadith & Tafsir
**Goal**: Users can research structured hadith and tafsir linked to exact Quran locations with attributed gradings.
**Depends on**: Phase 5
**Requirements**: REQ-hadith-corpus, REQ-tafsir-comparative
**Success Criteria** (what must be TRUE):
  1. User can search hadith (exact + normalized) with collection, edition, and source numbering shown
  2. User can read matn/isnad separation where supported, with every grading attributed to its scholar
  3. User can navigate from any verse to linked hadith and tafsir passages
  4. Scholarly disagreement is shown side-by-side, never silently merged
  5. Computational identity matches are labeled as uncertain, never as certainty
**Plans**: TBD

### Phase 7: Isnad & Narrator Research
**Goal**: Users can trace narration chains and judge narrator identity with explicit uncertainty.
**Depends on**: Phase 6
**Requirements**: REQ-isnad-narrator-graph
**Success Criteria** (what must be TRUE):
  1. User can view a narrator record with name variants and rijal sources
  2. User can trace chain paths between narrators with uncertainty marked on each link
  3. Reviewer can approve or reject a narrator-identity proposal in a recorded workflow
**Plans**: TBD

### Phase 8: Multi-RAG Platform
**Goal**: Users can build, configure, and debug multiple independent RAG systems over any corpus.
**Depends on**: Phase 7
**Requirements**: REQ-multi-rag, REQ-generic-rag-platform
**Success Criteria** (what must be TRUE):
  1. User can create a portable RAG project from a declarative config and import documents into it
  2. User gets hybrid (full-text + vector) retrieval with reranking and metadata filters
  3. User can see which router, tools, and sources were chosen for any query
  4. Operator can debug any retrieval in the RAG view and reproduce the same result set
  5. Tombstoned or deactivated sources disappear from results immediately
**Plans**: TBD

### Phase 9: Comparative Scripture
**Goal**: Users can research Torah, Tanakh, and New Testament passages alongside the Quran with honest labeling.
**Depends on**: Phase 8
**Requirements**: REQ-knowledge-domains
**Success Criteria** (what must be TRUE):
  1. User can browse scripture editions with chapter/verse addressing and translation comparison
  2. User can run parallel-passage research with computational suggestions explicitly labeled as such
  3. User can include or exclude any knowledge domain (Quran, hadith, tafsir, scripture, books) per query
**Plans**: TBD

### Phase 10: Agentic Research & Tool Creation
**Goal**: Users can delegate research to controlled agents and create sandboxed tools; MVP acceptance passes.
**Depends on**: Phase 9
**Requirements**: REQ-agents-tools-runtime, REQ-models-routing, REQ-answer-contract, REQ-phases-mvp
**Success Criteria** (what must be TRUE):
  1. User can ask a research question, inspect selected tools and evidence, and receive an answer with claim-level citations that all re-verify
  2. User can cancel a running research query and see bounded resource usage in the run trace
  3. Tool author can describe → review → generate → isolated-test → security-review → approve → publish a tool, and revoke it instantly
  4. The full MVP acceptance workflow (§44.3: install → read → search → inspect → graph → ask → verify citations → export) succeeds end to end
  5. Retrieved content is treated as untrusted data on every agent path (prompt-injection tests pass)
**Plans**: TBD

### Phase 11: Server & Management
**Goal**: Operators can serve Q-ai remotely and manage sources, agents, tools, and audit from the dashboard.
**Depends on**: Phase 10
**Requirements**: REQ-source-catalog-trust, REQ-ux-web, REQ-retrieval-governance
**Success Criteria** (what must be TRUE):
  1. Operator can run the versioned HTTP API with auth/RBAC and stream research progress
  2. Curator can discover, quarantine, license-review, stage, diff, approve, and roll back a source update
  3. Manager can inspect budgets, run events, audit trail, and conversation sessions from the dashboard
  4. Access-control-aware retrieval enforces authorization before results on every path
**Plans**: TBD
**UI hint**: yes

### Phase 12: Production Hardening
**Goal**: Q-ai is deployable, recoverable, and clean: all gates green, diagnostics trusted, open decisions closed.
**Depends on**: Phase 11
**Requirements**: REQ-security-licensing-observability, REQ-open-decisions-status
**Success Criteria** (what must be TRUE):
  1. Operator can deploy via Docker with TLS, PostgreSQL/Qdrant adapters, backups, and disaster recovery
  2. `qai doctor` reports corpus, index, and cross-store health with explicit repair actions
  3. Performance and security test suites pass with licensing gates enforced before activation
  4. Every open technical decision is resolved in an ADR or explicitly deferred with rationale
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9 → 10 → 11 → 12

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundations | 0/5 | Not started | - |
| 2. Canonical Quran Core | 0/TBD | Not started | - |
| 3. Quran Search & Linguistics | 0/TBD | Not started | - |
| 4. Quran Graph | 0/TBD | Not started | - |
| 5. Rich Quran Experience | 0/TBD | Not started | - |
| 6. Hadith & Tafsir | 0/TBD | Not started | - |
| 7. Isnad & Narrator Research | 0/TBD | Not started | - |
| 8. Multi-RAG Platform | 0/TBD | Not started | - |
| 9. Comparative Scripture | 0/TBD | Not started | - |
| 10. Agentic Research & Tool Creation | 0/TBD | Not started | - |
| 11. Server & Management | 0/TBD | Not started | - |
| 12. Production Hardening | 0/TBD | Not started | - |

---
*Notes: Roadmap phases 1–12 track PRD §43/§88 Phases 0–12; PRD Phase 9 (agentic) + Phase 10 (tool creation/workflows) merge into roadmap Phase 10 (shared runtime + acceptance gates). Single-requirement phases (4, 7, 9) are standalone major capabilities, not thin slices. MVP acceptance (§44.3) lands in Phase 10; Phases 11–12 are post-MVP.*
