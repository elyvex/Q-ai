# Roadmap: Q-ai

## Overview

From an empty workspace to a trustworthy local-first Quran research platform: foundations and provenance first, then the canonical Quran engine with verifiable citations, then linguistics, graph, and rich interfaces, then hadith/tafsir/isnad, then multi-RAG and comparative scripture, then controlled agentic research, and finally server management and production hardening. Every phase passes the PRD quality gates and definition-of-done (§46/§58/§90/§93) before the next begins. MVP acceptance (§44.3) is met at Phase 10; Phases 11–12 are post-MVP.

## Phases

**Phase Numbering:**

- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Foundations** - Workspace, config, SQLite, jobs, provenance, audit, CLI skeleton (completed 2026-09-28)
- [x] **Phase 2: Canonical Quran Core** - Validated import, addressing, tokens, exact lookup, integrity (completed 2026-09-25)
- [x] **Phase 3: Quran Search & Linguistics** - Normalization, roots/lemmas, morphology, frequency tools (completed 2026-09-28)
- [x] **Phase 3.5: Residual Closure (INSERTED)** - Engineering remainder of Phases 1-3, status truth restoration, owner-gate readiness (completed 2026-10-02)
- [x] **Phase 4: Quran Graph** - Graph store, traversal, annotations, visualization (completed 2026-09-29)
- [x] **Phase 5: Rich Quran Experience** - Web GUI, TUI cockpit, research tools, result contract
- [x] **Phase 6: Hadith & Tafsir** - Structured records, grading, verse links, tafsir indexing (completed 2026-10-16)
- [x] **Phase 7: Isnad & Narrator Research** - Identity review, chain paths, uncertainty (completed 2026-10-17)
- [x] **Phase 8: Multi-RAG Platform** - Full-text, vector, hybrid, reranking, generic RAG projects (completed 2026-10-18)
- [x] **Phase 9: Comparative Scripture** - Edition-aware scripture model, parallel-passage research (completed 2026-10-19)
- [x] **Phase 10: Agentic Research & Tool Creation** - Agents, tool sandbox, workflows, MVP acceptance (completed 2026-10-19)
- [x] **Phase 11: Server & Management** - Versioned API, streaming, auth, catalog, governance (completed 2026-10-19)
- [x] **Phase 12: Production Hardening** - PG/Qdrant adapters, TLS, Docker, backups, recovery (completed 2026-10-19)

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

**Plans**: 5/5 plans executed

Plans:

- [x] 01-01-PLAN.md — Fresh-workspace operator readiness tracer: config origins, explicit migration, read-only doctor, and persisted audit verification
- [x] 01-02-PLAN.md — Audited durable mutations with same-UnitOfWork provenance/audit/outbox coverage and rollback proof
- [x] 01-03-PLAN.md — Durable job checkpoints, per-kind retry, cooperative cancellation, and lifecycle audit controls
- [x] 01-04-PLAN.md — Long-lived `qai serve` worker host with safe shutdown and enqueue-only one-shot import boundary
- [x] 01-05-PLAN.md — Registry/git architecture enforcement, final evidence gates, and TASK-001 repository closure

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

**Plans**: 8/8 plans complete (7 executed + 1 gap-closure)

Plans:
**Wave 1**

- [x] 02-01-PLAN.md — TRACER: edition upstream identity, declared license, and primary/default designation persisted from manifest to operator surface

**Wave 2** *(blocked on Wave 1 completion)*

- [x] 02-02-PLAN.md — Richer synthetic fixture, companion synthetic reference, and the identity/primary/license fixture
- [x] 02-04-PLAN.md — Canonical-write fence completion: append-only trigger set, non-forgeable ApprovalToken + real CanonicalWriter, importer code-path audit, rollback rejection and cache-invalidation coverage

**Wave 3** *(blocked on Wave 2 completion)*

- [x] 02-03-PLAN.md — Operator integrity evidence: `qai quran verify` six-family surface, state-derived reference check, stored hash on lookup, reference-comparison operator path, and the identity/primary operator snapshot
- [x] 02-06-PLAN.md — Translation-layer completion: additive translation hash recipe, verbatim declared license, and a layer-separation negative test

**Wave 4** *(blocked on Wave 3 completion)*

- [x] 02-05-PLAN.md — Quotation hard-failure wiring: shared verdict mapping, `qai quran verify-quotation`, and HTTP answer-path enforcement

**Wave 5** *(blocked on Wave 4 completion)*

- [x] 02-07-PLAN.md — Owner gates OD-01/OD-02/OD-03 recorded as blocked, the owner-ratifiable D-15 read-path item, spec-less edge-probe disposition, coverage-gate reconciliation, TASK-002 closure, and committed evidence of record

**Gap closure (G-02-3)**

- [x] 02-08-PLAN.md — Rebuild the D-15 answer-path exemption ledger on the emission frame, correct every copy of the false completeness claim, and add a source-scan drift guard

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

**Plans**: 8/8 plans executed

Plans:
**Wave 1**

- [x] 03-01-PLAN.md — TRACER: end-to-end normalize → index → search with explainability, the per-hit canonical-display (SC5) pin, and the normalization rule-doc correction (SC1)

**Wave 2** *(blocked on Wave 1 completion)*

- [x] 03-02-PLAN.md — SC2 concatenated golden set: independent oracle + >= 120-case fixture + segmentation-tiling runner + CLI snapshot (G-03)
- [x] 03-03-PLAN.md — SC4 counting: storage aggregation + real root/lemma frequency + selectable multi-analysis modes + CLI `freq` (G-01/G-07/G-13)
- [x] 03-04-PLAN.md — SC3 family engine: all typed relation builders + typed affix error + 120-family golden set (G-05/G-09)

**Wave 3** *(blocked on Wave 2 completion)*

- [x] 03-05-PLAN.md — SC3/SC4 surfaces: `qai quran family`/`lemma` CLI + versioned HTTP family and root/lemma-frequency routes (G-02/G-10)

**Wave 4** *(blocked on Wave 3 completion)*

- [x] 03-06-PLAN.md — Licensing: machine-readable license matrix + fail-closed activation-rejection gate + CLI license plumbing (G-04/D-07)
- [x] 03-07-PLAN.md — Hardening: `doctor --indexes` end-to-end soak + performance budgets + attributed tool registry (G-08/G-06/G-11)

**Wave 5** *(blocked on Wave 4 completion)*

- [x] 03-08-PLAN.md — Owner gates OD-11/OD-12 recorded BLOCKED + deferral ledger + alpha end-to-end evidence (D-03/D-08)

### Phase 3.5: Residual Closure (INSERTED)

**Goal**: Close the engineering remainder Phases 1–3 left behind, and end the split-brain status condition, without re-opening a closed phase and without answering any owner decision.
**Depends on**: Phases 1, 2, 3 (runs after Phase 4; does not gate it)
**Requirements**: REQ-architecture-principles-quality, REQ-quran-normalization, REQ-quran-linguistics, REQ-quran-research-tools, REQ-cli-api, REQ-ingestion-validation-eval, REQ-engineering-baseline, REQ-security-licensing-observability, REQ-open-decisions-status
**Context**: `.planning/phases/03.5-residual-closure/03.5-CONTEXT.md`

**Success Criteria** (what must be TRUE):

  1. No phase board row contradicts a committed SUMMARY, and a reader of any one status file learns the same phase state
  2. The counting and discovery tools are reachable from CLI and HTTP with identical results and fail closed when their dataset is absent
  3. Index drift is detectable on a schedule without a human, and a rebuild is bounded by a committed budget
  4. Linguistics regressions are measurable against a committed versioned dataset, and every shipped tool passes one conformance contract
  5. Every owner decision that remains open is recorded with the exact command that closes it, and none of them is answered

**Plans**: 4/4 plans executed

- [x] 03.5-01-PLAN.md — W1 board + ledger truth restoration, W2 documentation set and SECURITY.md (wave 1)
- [x] 03.5-02-PLAN.md — W4 counting/discovery surface, W3 reconciliation job, W8 cold-rebuild budget (wave 2)
- [x] 03.5-03-PLAN.md — W6 evaluation harness, W7 tool conformance, W5 goldens, W8 full soak (wave 3)
- [x] 03.5-04-PLAN.md — W9 Phase-1 hardening, W10 test-gate robustness, phase gate + evidence matrix (wave 4)

### Phase 4: Quran Graph

**Goal**: Users can explore structural and linguistic relationships of the Quran as a navigable graph.
**Depends on**: Phase 3
**Requirements**: REQ-quran-graph
**Success Criteria** (what must be TRUE):

  1. User can open a neighbor view around any verse, word, root, or concept
  2. User can find paths between two graph nodes with each edge's provenance shown
  3. User can export a subgraph with edge provenance intact
  4. Graph queries explain why each result was returned

**Plans**: 6/6 plans complete

Plans:

- [x] 04-06-PLAN.md

**Wave 1**

- [x] 04-01-PLAN.md — TRACER: structural build → inspect → neighbors → canonical quotation over SQLite, schema fix, vocab, conformance, fixtures

**Wave 2** *(blocked on 04-01)*

- [x] 04-02-PLAN.md — Annotation authority, review queue, word-root/annotated builders, rebuild preservation, review CLI

**Wave 3** *(blocked on 04-02)*

- [x] 04-03-PLAN.md — Path modes, subgraph, patterns, root-family, CLI read tree, Graph JSON export, DOT/SVG render

**Wave 4** *(blocked on 04-03)*

- [x] 04-04-PLAN.md — HTTP read routes, OpenAPI, typed tools, cross-surface parity, explainability contract

**Wave 5** *(blocked on 04-04: shared lib.rs wiring)*

- [x] 04-05-PLAN.md — Read-only doctor, explicit repair/GC, owner-gate and deferral ledgers, full phase gate

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

**Plans**: 9 plans

Plans:
**Wave 1**

- [x] 05-01-PLAN.md — TRACER: `research_checksum` contract end-to-end (tool identity + params + inputs + payload) across all 12 tools, HTTP `Meta` + the generic typed-tool route `/api/v1/quran/tool/{name}`, OpenAPI, and CLI
- [x] 05-02-PLAN.md — Arch/allowlist plumbing + TUI foundation: ratatui/crossterm app shell, palette, dashboard, `qai tui` over shared services

**Wave 2** *(blocked on Wave 1)*

- [x] 05-03-PLAN.md — Cross-surface parity gate: one fixture through service + HTTP (generic tool route) + CLI, byte-identical payload + identical checksum (D-16)
- [x] 05-04-PLAN.md — SPA scaffold (React+Vite+TS, pinned, incl. the Vitest/jsdom/@testing-library test toolchain) + same-origin embedded serving with a fallback that never swallows `/api/*` (D-01/D-02/D-03/D-04)
- [x] 05-05-PLAN.md — TUI cockpit screens: ops (doctor/jobs/index), reader/search, graph nav + annotation review, typed RAG debug (D-10/D-11/D-12)

**Wave 3** *(blocked on Wave 2)*

- [x] 05-06-PLAN.md — SPA reading view: layer tokens, themes, 3 translation + 3 inspector modes, RTL, typed client (D-05…D-09)

**Wave 4** *(blocked on Wave 3)*

- [x] 05-09-PLAN.md — SPA research view + graph view (truncation verbatim) + honest typed RAG debug view (D-11)

**Wave 5** *(blocked on Wave 4)*

- [x] 05-08-PLAN.md — Citation copy/open: stable URN + deep link, server-side re-verify + mismatch hard-failure via `/api/v1/quran/citations/{id}` (D-17)

**Wave 6** *(blocked on Wave 5)*

- [x] 05-07-PLAN.md — CI wiring (SPA build + parity steps, 9→11), layer-separation source scan, owner-gate recording, validation map + roadmap

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

**Plans**: 5/5 plans complete

Plans:
**Wave 1**

- [x] 06-01-PLAN.md — TRACER: hadith data model, storage, search, and typed tool end-to-end

**Wave 2** *(blocked on Wave 1)*

- [x] 06-02-PLAN.md — Tafsir data model, storage, search, and typed tool

**Wave 3** *(blocked on Wave 2)*

- [x] 06-03-PLAN.md — Verse linking, owner gate, and disagreement display

**Wave 4** *(blocked on Wave 3)*

- [x] 06-04-PLAN.md — Cross-surface parity, CLI/API routes, and ingestion pipeline

### Phase 7: Isnad & Narrator Research

**Goal**: Users can trace narration chains and judge narrator identity with explicit uncertainty.
**Depends on**: Phase 6
**Requirements**: REQ-isnad-narrator-graph
**Success Criteria** (what must be TRUE):

  1. User can view a narrator record with name variants and rijal sources
  2. User can trace chain paths between narrators with uncertainty marked on each link
  3. Reviewer can approve or reject a narrator-identity proposal in a recorded workflow

**Plans**: TBD

- [x] 07-01-PLAN.md
- [x] 07-02-PLAN.md
- [x] 07-03-PLAN.md
- [x] 07-04-PLAN.md
- [x] 07-05-PLAN.md

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

**Plans**: 5/6 plans executed

Plans:

- [x] 08-01-PLAN.md — TRACER: migrations 0035-0038 + embeddings/reranking traits + rag core + RagBackend + app modules
- [x] 08-02-PLAN.md — Config (TOML) + project lifecycle (create/import) + portability
- [x] 08-03-PLAN.md — Hybrid retrieval (FTS5 + vector, RRF, metadata filters)
- [x] 08-04-PLAN.md — Routing transparency (route_decision + checksum) + debug view (CLI/TUI/server)
- [x] 08-05-PLAN.md — Tombstoning/deactivation propagation + immediate disappearance
- [x] 08-06-PLAN.md — Cross-surface parity + owner gates + deferrals + validation map + phase gate

### Phase 9: Comparative Scripture:

**Goal**: Users can research Torah, Tanakh, and New Testament passages alongside the Quran with honest labeling.
**Depends on**: Phase 8
**Requirements**: REQ-knowledge-domains
**Success Criteria** (what must be TRUE):

  1. User can browse scripture editions with chapter/verse addressing and translation comparison
  2. User can run parallel-passage research with computational suggestions explicitly labeled as such
  3. User can include or exclude any knowledge domain (Quran, hadith, tafsir, scripture, books) per query

**Plans**: 3/6 plans executed

Plans:

- [x] 09-01-PLAN.md — TRACER: scripture edition/book/verse domain types + migrations 0039-0041 + reader + scripture.get tool
- [x] 09-02-PLAN.md — Translations (0042) + search forms/FTS (0043) + scripture.search/translations tools + CLI
- [x] 09-03-PLAN.md — Parallel passages: migration 0044 + verse_links scripture source + owner gate + scripture.parallel_passages
- [ ] 09-04-PLAN.md — KnowledgeDomain filter + cross-domain research aggregator
- [ ] 09-05-PLAN.md — HTTP routes + TUI panels + cross-surface parity
- [ ] 09-06-PLAN.md — Owner gates + deferrals + validation map + phase gate

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
Phases execute in numeric order: 1 → 2 → 3 → 3.5 → 4 → 5 → 6 → 7 → 8 → 9 → 10 → 11 → 12.
Phase 3.5 is an INSERTED gap-closure phase; it runs after Phase 4 and does not gate it.

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundations | 5/5 | Complete    | 2026-09-28 |
| 2. Canonical Quran Core | 8/8 | Complete    | 2026-10-06 |
| 3. Quran Search & Linguistics | 8/8 | Complete (owner gates OD-11/OD-12 open) | 2026-09-28 |
| 3.5. Residual Closure (INSERTED) | 4/4 | Complete (W1–W10 evidence matrix; owner gates OD-01…OD-14 open) | 2026-10-02 |
| 4. Quran Graph | 6/6 | Complete    | 2026-10-16 |
| 5. Rich Quran Experience | 9/9 | Complete (owner gates OD-04/OD-11/OD-12 open) | 2026-10-09 |
| 6. Hadith & Tafsir | 5/5 | Complete    | 2026-10-16 |
| 7. Isnad & Narrator Research | 5/5 | Complete    | 2026-10-17 |
| 8. Multi-RAG Platform | 6/6 | Complete    | 2026-10-18 |
| 9. Comparative Scripture | 6/6 | Complete    | 2026-10-19 |
| 10. Agentic Research & Tool Creation | 1/TBD | Complete    | 2026-10-19 |
| 11. Server & Management | 1/TBD | Complete    | 2026-10-19 |
| 12. Production Hardening | 1/TBD | Complete    | 2026-10-19 |

### Phase 13: Web UI Redesign

Sources identified for aggregation:

- **R0** — the roadmap containing the Phase 13 placeholder (attached three times as identical copies: files 2, 3, and 4; treated as one source).
- **R1** — the 9-plan / 7-wave Phase 13 definition (file 5).
- **R2** — the 8-plan / 5-wave Phase 13 definition with design brief, owner gates, deferrals, and roadmap entries (file 6).

Below is the consolidated, roadmap-ready Phase 13 definition, followed by the required lossless-aggregation report.

---

# 1. Consolidated Document

## Phase 13: Web UI Redesign

**Goal:** Users get a from-scratch redesign of the Vite + React web UI in an elegant light white/gray style, with an animated background and polished buttons and controls. Every Q-ai feature from Phases 1–12 (read, search, inspect, graph, hadith/tafsir, isnad, multi-RAG, comparative scripture, agents, management) is reachable through the UI. The result is a cohesive, production-grade single-page application that feels like a dedicated desktop research tool rendered in the browser. The redesign adds no backend capability. `[R1, R2]`

**Depends on:** Phase 12 `[R1, R2]`

**Requirements:** REQ-ux-web, REQ-quran-display, REQ-quran-research-tools, REQ-quran-result-contract, REQ-cli-api, REQ-hadith-corpus, REQ-tafsir-comparative, REQ-isnad-narrator-graph, REQ-multi-rag, REQ-generic-rag-platform, REQ-knowledge-domains, REQ-agents-tools-runtime, REQ-source-catalog-trust, REQ-retrieval-governance, REQ-quran-graph `[R1]`; plus REQ-security-licensing-observability `[R2]` (added for font licensing and the no-third-party-network rule).

**Pre-execution note:** R2 cannot see the repository and references the existing SPA, layer tokens, and typed client as described in the roadmap. Run `/gsd-plan-phase 13` first to check those references against the code. `[R2]`

### Design brief (binding for all plans)

- **Palette:** white canvas (`#FFFFFF`), a gray neutral scale, and one restrained accent used only for interactive state. No gradients on content surfaces. R1 specifies the scale as a warm-gray `#F5F5F4` … `#292524` scale; R2 calls it a gray neutral scale. `[R1, R2]`
- **Animated background:** a slow, low-opacity gradient field behind content. R2 requires CSS-only, with no canvas loop; R1 permits a CSS/canvas particle or gradient-mesh animation. Both require `prefers-reduced-motion` support; R2 additionally requires that it pauses when the tab is hidden, never captures pointer events, and never reduces text contrast. `[R1, R2]` *(see conflict C-1)*
- **Controls:** 8–10px radius, 1px neutral border, subtle elevation. Hover lifts 1px, press scales to 0.98, transitions run about 150ms ease-out. Focus uses a visible 2px ring with offset. Every control has disabled, loading, and error states. `[R2]`
- **Typography:** canonical Arabic in a large Naskh-class face in near-black ink. Translations use a neutral sans in gray-700. Annotations use a tinted accent with a left rule. Self-hosted fonts only. R1 specifies the Arabic stack as `Amiri`/`Noto Naskh Arabic` plus a system stack. `[R1, R2]` *(see conflict C-7)*
- **Layer distinction is not color-only:** canonical, translation, and annotation text must stay distinguishable in grayscale, using weight, rules, labels, and icons. `[R2]`
- **Light theme only.** Dark theme is deferred. R1 additionally proposed a dark-mode toggle wired to tokens but shipping light-only initially. `[R2, R1]` *(see conflict C-3)*

### Success Criteria (what must be TRUE)

1. **Feature reachability:** a generated feature-to-route matrix covers every Phase 1–12 feature (read, search, normalization, linguistics, graph, hadith, tafsir, isnad, multi-RAG, scripture, agents, tool lifecycle, API auth, source catalog, management dashboard, and the capabilities named in R1: inspect, comparative scripture, management). Every row has a UI route. Zero rows are CLI- or API-only. `[R1 SC1, R2 SC1]`
2. **Light theme with motion discipline:** the UI ships an elegant light white/gray theme with an animated background, styled buttons/controls, and a consistent visual language across all views. All text tokens meet WCAG 2.2 AA contrast against their surface. The animated background respects reduced motion, pauses when hidden, and passes an automated check that it does not capture pointer events. `[R1 SC2, R2 SC2]`
3. **Layer integrity:** canonical text, translations, and annotations are visually unmistakable in every redesigned view, including grayscale; layer tokens are preserved. A visual-contract test asserts the layer tokens on each view. `[R1 SC3, R2 SC3]`
4. **Tokenized design system:** colors, typography, spacing, shadows, and radii are tokenized so every component derives from a single source of truth. `[R1 SC5]`
5. **Interactive states:** every interactive element has visible focus, hover, active, disabled, loading, and error states. `[R1 SC6, R2 design brief]`
6. **Arabic and RTL:** Arabic content renders RTL with correct bidirectional isolation, line-breaking, and font stack across Chrome, Firefox, and Safari. `[R1 SC7]`
7. **Motion performance:** page transitions, skeleton loaders, and micro-animations run at ≥ 60 fps with no layout shift (CLS < 0.1). `[R1 SC8]`
8. **End-to-end browser walkthrough:** the full read → search → inspect → graph → ask → verify-citations → export walkthrough completes in the browser without touching CLI or API directly. `[R1 SC9]`
9. **Accessibility:** Lighthouse accessibility score ≥ 90; automated axe scans report zero serious or critical violations on every route; every control is keyboard-navigable and screen-reader-labeled; core flows (read, search, copy citation, ask, approve a source) are completable by keyboard alone. `[R1 SC10, R2 SC4]`
10. **Gates green:** the rebuilt SPA passes the existing build and cross-surface parity gates with no regressions. The Rust backend has no diff beyond defect fixes found during the phase. `[R1 SC4, R2 SC5]`
11. **Performance and offline:** initial JS and CSS stay within a committed budget (R1 gives explicit targets: initial bundle ≤ 350 KB gzipped via code-split, LCP ≤ 1.5 s, CLS < 0.1; Lighthouse performance ≥ 80 in CI). The app makes zero third-party network requests. Fonts are self-hosted and recorded in the license matrix. `[R1, R2 SC6]`

### Plans

**Plans:** 9 plans across 7 waves `[R1]` *(R2 specified 8 plans across 5 waves; see conflict C-4 and the mapping table below.)*

#### Wave 1 — Design System & App Shell

- **13-01 — Design tokens, fonts, reachability tracer & component library.**
  - Design tokens: light palette (white `#FFFFFF` canvas; warm-gray/gray neutral scale `#F5F5F4` … `#292524`; single restrained accent hue), spacing, radius, elevation/shadow, and motion tokens; exported as CSS custom properties and TypeScript constants. `[R1, R2]`
  - Typography: system + Arabic `Amiri`/`Noto Naskh Arabic` stack; canonical Arabic in a large Naskh-class face in near-black ink; translations in neutral sans gray-700; annotations with tinted accent and left rule; self-hosted fonts with license-matrix entries. `[R1, R2]`
  - Feature-to-route reachability generator, failing until green. `[R2]`
  - Primitive components: `Button` (R1: 5 variants × 4 sizes × 6 states; R2: primary, secondary, ghost, danger), `IconButton`, `Input`, `TextArea`, `Select`, `Chip`, `Badge`, `Toggle`, `Tooltip`, `Card`, `Modal`, `Dialog`, `Drawer`, `DropdownMenu`, `Tabs`, `Accordion`, `Skeleton`, `Spinner`, `Toast`/`Snackbar`, `EmptyState`, `ErrorState`, `ErrorBoundary`, and the `AnimatedBackground` component with its reduced-motion, visibility, and pointer-passthrough behavior. `[R1, R2]`
  - Each component has Vitest + Testing Library coverage. Storybook catalog (or lightweight equivalent) for visual review. `[R1]`
  - TRACER slice: the redesigned Reader view end-to-end against the live typed client, including layer tokens. `[R2]`
- **13-02 — App shell, layout, routing & animated background.**
  - Responsive sidebar navigation: collapsible, icon + label, active indicator, grouped sections: Quran · Hadith · Tafsir · Isnad · Scripture · RAG · Agents · Management. `[R1]`
  - Top header bar: breadcrumb, global search trigger, settings gear, connection indicator. Content area with scroll container. `[R1]`
  - Full route table with preserved deep links; route scaffold (`react-router` v6) with lazy code-split per feature area. `[R1, R2]`
  - Web command palette (Ctrl/Cmd+K) and keyboard shortcuts. `[R2]`
  - Strict CSP-compatible setup. `[R2]`
  - Animated background behind the content layer: subtle, performant CSS/canvas particle or gradient-mesh animation (R1) / CSS-only gradient field with no canvas loop (R2); respects `prefers-reduced-motion`, pauses when the tab is hidden, never captures pointer events, and never reduces text contrast. `[R1, R2]`
  - Global error boundary, 404 page, loading shell. `[R1]`
  - Dark-mode toggle wired to tokens but shipping light-only initially (R1); dark theme deferred (R2). `[R1, R2]`

#### Wave 2 — Quran Reading & Search *(blocked on Wave 1)*

- **13-03 — Quran reader & search views.**
  - Mushaf-style reading view with ayah-by-ayah Arabic (RTL, proper bidirectional isolation, `dir="rtl"` + `unicode-bidi: isolate`), inline verse numbers, surah/juz/hizb navigator, page/continuous scroll mode. `[R1]`
  - Translation panel: up to 3 translations side-by-side, collapsible. `[R1]`
  - Word inspector popover: click any word → lemma, root, morphological tags, word-family link. `[R1]`
  - Citation copy button producing a stable URN deep-link; paste/open triggers server-side `verify_quotation`; mismatch = hard-failure toast. Explicit re-verify and mismatch states. `[R1, R2]`
  - Search bar with diacritics-free and concatenated (spaceless) input; result list with highlighted matches and exact-location badges; canonical display never modified (SC5 pin); normalization and concatenation explainability. `[R1, R2]`
  - Frequency sparkline per root in search results. `[R1]`
  - Research tools: family, lemma, frequency, distribution, and co-occurrence, showing `research_checksum`. `[R2]`

#### Wave 3 — Linguistics & Graph *(blocked on Wave 2)*

- **13-04 — Linguistics explorer.** `[R1]`
  - Root/lemma detail page: definition, morphological paradigm table, word-family tree, all Quranic attestations with surah:ayah links.
  - Frequency dashboard: sortable root/lemma table, bar chart of surah distribution, co-occurrence heatmap.
  - Filters by surah, juz, Meccan/Medinan.
  - Word-family graph rendered inline as a force-directed mini-graph of typed relations: root → lemma → derived form.
  - Every data point links back to canonical verse display.
- **13-05 — Graph explorer.** `[R1, R2]`
  - Interactive force-directed or hierarchical graph canvas (lightweight lib: `@xyflow/react` or `d3-force` with a React wrapper). `[R1]`
  - Neighbor view: click any node (verse, word, root, concept) → expand neighbors with edge-type color coding and provenance badge. `[R1, R2]`
  - Path finder: select two nodes → animated shortest-path highlighting with edge provenance tooltip; paths with per-edge provenance. `[R1, R2]`
  - Subgraph export: select region → download JSON / DOT / SVG. `[R1, R2]`
  - Explain panel: every query result shows the structured explanation (why returned, which edges, confidence). `[R1]`
  - Truncation shown verbatim. `[R2]`
  - Node search + filter sidebar. Canvas controls: zoom, pan, fit, reset, full-screen. `[R1]`

#### Wave 4 — Hadith, Tafsir & Isnad *(blocked on Wave 3)*

- **13-06 — Hadith, tafsir & isnad views.** `[R1, R2]`
  - Hadith browser: collection/book/chapter tree navigation; hadith card with matn in Arabic, isnad separated, grading badges attributed to the scholar, edition + source numbering.
  - Hadith search (exact + normalized), result cards, filters by collection/grading/narrator.
  - Tafsir panel: verse-linked tafsir passages side-by-side from multiple mufassirun; disagreements shown in parallel columns — never merged.
  - Isnad chain visualization: horizontal/vertical DAG of narrators per hadith; each node = narrator card (name variants, rijal source links); uncertainty marked with dashed edges + explicit label; per-link uncertainty.
  - Narrator detail page: name variants, rijal sources, identity proposals with approve/reject workflow (reviewer role); identity-review workflow.
  - Verse → hadith/tafsir link panel accessible from the Quran reader view.
  - Uncertainty labels throughout.

#### Wave 5 — RAG, Agents & Comparative Scripture *(blocked on Wave 4)*

- **13-07 — RAG dashboard, agent research & scripture views.** `[R1, R2]`
  - RAG project list: create, import, and configure via a form mapped to the TOML schema.
  - Document list with status badges (active / tombstoned / deactivated).
  - Query playground: enter query → hybrid retrieval results (FTS + vector scores, RRF rank), reranking trace, metadata filters, router decision with checksum, expandable debug panel per result (chunk text, embedding distance, source provenance).
  - Honest RAG debug view: router, tools, and sources chosen, plus a reproducible result-set checksum.
  - Agent research view: ask a question → streaming progress panel (selected tools, evidence snippets as they arrive); final answer card with claim-level citations; each citation has a one-click re-verify button (calls `verify_quotation`, green check / red X inline); streamed agent research with cancel.
  - Run trace panel: bounded resource usage, cancellation button, full event log.
  - Tool lifecycle UI: describe → review → generate → test → security review → approve → publish → revoke.
  - Comparative scripture browser: edition selector, book/chapter/verse navigation, parallel translation comparison, parallel-passage research with computational suggestions labeled `[computational]`.
  - Knowledge-domain filter toggle bar (Quran · Hadith · Tafsir · Scripture · Books) that scopes every query; per-query domain include/exclude filters.

#### Wave 6 — Management & Operations *(blocked on Wave 5)*

- **13-08 — Management dashboard.** `[R1, R2]`
  - Source catalog: discover → quarantine → license-review → stage → diff → approve → rollback pipeline; each step a distinct card state with action buttons.
  - Audit trail: filterable, searchable append-only event log table with timestamp, actor, operation, entity, diff preview.
  - Job monitor: active/queued/completed/failed jobs table, progress bars, cancel button, checkpoint detail expandable row.
  - Doctor panel: corpus / index / cross-store health cards with status indicators (green/amber/red), explicit repair action buttons, last-run timestamp.
  - Settings: config origins display (CLI > env > file > defaults), validation errors with remedy text.
  - Auth/RBAC: user list, role assignment, session list (if auth enabled); auth/RBAC login. The UI hides role-gated actions for clarity, but the server remains the enforcement point.
  - System info: version, edition, database stats, index stats, uptime.
  - Ops dashboard also covers budgets, run events, and conversation sessions.

#### Wave 7 — Polish, Accessibility & Parity Gate *(blocked on Wave 6)*

- **13-09 — Animations, accessibility, performance & CI parity.** `[R1, R2]`
  - Page transition animations (fade/slide, route-aware).
  - Skeleton loaders for every async panel.
  - Micro-interactions: button press scale, toast slide-in, modal backdrop fade, graph node hover glow, search result stagger.
  - All animations respect `prefers-reduced-motion`.
  - Accessibility audit: ARIA labels on every control, landmark regions, skip-to-content link, focus-trap in modals/drawers, keyboard navigation for the graph canvas (arrow keys + Enter to expand), color contrast ≥ 4.5:1 on all text.
  - Axe and keyboard test suites; contrast and motion checks; layer visual-contract tests.
  - Performance budget: initial bundle ≤ 350 KB gzipped (code-split), LCP ≤ 1.5 s, CLS < 0.1; performance budget file and enforcement.
  - Cross-surface parity gate: run the existing fixture suite (Phase 5 contract) against every HTTP-backed UI view, assert byte-identical payloads + identical `research_checksum`.
  - CI steps: SPA lint → SPA unit tests → SPA build → Lighthouse CI (accessibility ≥ 90, performance ≥ 80) → parity integration tests; SPA build and parity steps retained.
  - Reachability matrix turned green.
  - Owner-gate recording, validation map, phase-gate evidence matrix, roadmap and board truth update.

### Owner gates and deferrals `[R2]`

- **OD-NEW (next free ID, OD-15 if still unused):** ratify the license of each bundled Arabic and UI font. Recorded BLOCKED with the exact command that closes it.
- **Deferred:** dark theme, UI chrome localization beyond English, and any new backend endpoint discovered during the phase. Each gets a deferral entry with rationale and a target phase.

### Out of scope `[R2]`

New product features; backend API changes beyond defect fixes; native mobile shells.

### Progress table row

| Phase | Plans Complete | Status | Completed |
|---|---|---|---|
| 13. Web UI Redesign | 0/9 | Not started | - |

R2 proposed `0/8` for its 8-plan structure; the consolidated 9-plan structure yields `0/9`. `[R1, R2]`

### Top-of-roadmap checklist entry `[R2]`

- [ ] **Phase 13: Web UI Redesign** — Light white/gray redesign, animated background, polished controls, full feature reachability

### R2 → consolidated plan mapping

| R2 plan | Consolidated plan |
|---|---|
| 13-01 TRACER | 13-01 (tokens, fonts, reachability) + tracer slice of 13-03 |
| 13-02 Primitives and motion | 13-01 |
| 13-03 App shell, routing, command palette | 13-02 |
| 13-04 Quran surfaces | 13-03 |
| 13-05 Knowledge surfaces | 13-05 (graph) + 13-06 (hadith/tafsir/isnad) + 13-07 (scripture) |
| 13-06 Ask, RAG, and agents | 13-07 |
| 13-07 Management | 13-08 |
| 13-08 Quality gates and close | 13-09 |

R1's plan numbers 13-01 through 13-09 map directly to the consolidated numbers, with R2 content folded into the corresponding plans.

---

# 2. Unique Information by Source

## Unique to R1 (9-plan definition)

- The expanded 15-item requirements list, including REQ-cli-api, REQ-hadith-corpus, REQ-tafsir-comparative, REQ-isnad-narrator-graph, REQ-multi-rag, REQ-generic-rag-platform, REQ-knowledge-domains, REQ-agents-tools-runtime, REQ-source-catalog-trust, REQ-retrieval-governance, and REQ-quran-graph.
- Palette specifics: white `#FFFFFF` and warm-gray `#F5F5F4` … `#292524` scale, single accent hue.
- Typography stack names: `Amiri` / `Noto Naskh Arabic` plus system stack.
- `Button` scale: 5 variants × 4 sizes × 6 states.
- Extended primitive list including `TextArea`, `Chip`, `Toggle`, `Modal`, `Drawer`, `DropdownMenu`, `Accordion`, `Spinner`, `Snackbar`, and `ErrorBoundary`.
- Storybook catalog (or lightweight equivalent) for visual review.
- `react-router` v6 with lazy code-split per feature area.
- Dark-mode toggle wired to tokens but shipping light-only initially.
- Animated background permitted as CSS/canvas particle or gradient-mesh animation.
- Detailed reader requirements: ayah-by-ayah rendering, inline verse numbers, surah/juz/hizb navigator, page/continuous scroll mode, up to 3 translations, word-inspector popover, URN deep-link, hard-failure toast, diacritics-free and concatenated search, exact-location badges, SC5 pin, frequency sparkline.
- Full linguistics-explorer detail: paradigm table, word-family tree, all attestations with surah:ayah links, sortable root/lemma table, surah-distribution bar chart, co-occurrence heatmap, surah/juz/Meccan-Medinan filters, force-directed mini-graph, canonical back-links.
- Full graph-explorer detail: `@xyflow/react` or `d3-force`, neighbor expansion, edge-type color coding, provenance badges, animated shortest-path highlighting, edge provenance tooltips, JSON/DOT/SVG export, explain panel, node search, filter sidebar, zoom/pan/fit/reset/full-screen.
- Full hadith/tafsir/isnad detail: collection/book/chapter tree, matn/isnad separation, scholar-attributed grading badges, edition and source numbering, exact + normalized search, collection/grading/narrator filters, side-by-side tafsir columns, horizontal/vertical isnad DAG, narrator cards with name variants and rijal links, dashed uncertainty edges, reviewer approve/reject workflow, verse-link panel.
- Full RAG detail: TOML-mapped configuration form, document status badges, FTS + vector scores, RRF rank, reranking trace, metadata filters, router decision with checksum, debug panel with chunk text, embedding distance, source provenance.
- Full agent detail: streaming progress, selected tools, live evidence snippets, final answer card, claim-level citations, one-click re-verify with green check / red X, run trace, bounded resource usage, cancellation, full event log.
- Full scripture detail: edition selector, book/chapter/verse navigation, parallel translation comparison, `[computational]` labels, knowledge-domain filter bar.
- Full management detail: distinct source-catalog card states, audit table columns (timestamp, actor, operation, entity, diff preview), job table and checkpoint rows, doctor green/amber/red cards, config-origins display, auth/RBAC user/role/session lists, system info (version, edition, database stats, index stats, uptime).
- Full polish detail: fade/slide route-aware transitions, skeleton loaders, the named micro-interactions, ARIA/landmarks/skip-link/focus-trap/graph keyboard navigation, contrast ≥ 4.5:1, bundle ≤ 350 KB gzipped, LCP ≤ 1.5 s, CLS < 0.1, Lighthouse accessibility ≥ 90 and performance ≥ 80, CI step order.
- Success criteria 5–10 (tokenization, interactive states, RTL across three browsers, ≥ 60 fps, no CLI/API walkthrough, Lighthouse/keyboard/screen-reader).

## Unique to R2 (8-plan definition)

- The pre-execution caveat: cannot see the repository; references the existing SPA, layer tokens, and typed client; run `/gsd-plan-phase 13` first.
- Explicit statement that the redesign adds no backend capability.
- Addition of REQ-security-licensing-observability with its rationale (font licensing and the no-third-party-network rule).
- The entire binding design brief: no gradients on content surfaces; CSS-only gradient field with no canvas loop; pause when tab hidden; no pointer capture; no contrast reduction; 8–10px radius, 1px neutral border, subtle elevation, 1px hover lift, 0.98 press scale, ~150 ms ease-out, 2px focus ring with offset; Naskh-class Arabic in near-black; translations in gray-700; annotations with tinted accent and left rule; self-hosted fonts only; layer distinction not color-only; light theme only.
- Success criteria specifics: generated feature-to-route matrix; WCAG 2.2 AA; automated pointer-event check; visual-contract test per view; keyboard-only core flows; axe zero serious/critical; Rust backend no diff beyond defect fixes; zero third-party network requests; fonts recorded in the license matrix.
- TRACER plan: design tokens plus self-hosted fonts with license-matrix entries, the failing-until-green reachability generator, and the Reader view end-to-end against the live typed client.
- Primitive list additions: `IconButton`, `ErrorState`, `AnimatedBackground` with reduced-motion, visibility, and pointer-passthrough behavior.
- Command palette (Ctrl/Cmd+K), keyboard shortcuts, strict CSP-compatible setup, full route table with preserved deep links.
- Research tools (family, lemma, frequency, distribution, co-occurrence) showing `research_checksum`.
- "Truncation shown verbatim" for graph output.
- Per-query domain include/exclude filters.
- Explicit tool lifecycle UI (describe → review → generate → test → security review → approve → publish → revoke).
- Reproducible result-set checksum in the RAG debug view.
- Management UI hides role-gated actions for clarity while the server remains the enforcement point.
- Quality-gate plan additions: axe and keyboard test suites, contrast and motion checks, layer visual-contract tests, performance budget file and enforcement, reachability matrix turned green, roadmap and board truth update.
- Owner gates and deferrals: OD-NEW (OD-15 if free) for font licenses, recorded BLOCKED with the closing command; deferrals for dark theme, UI chrome localization beyond English, and any new backend endpoint; each with rationale and target phase.
- Out-of-scope statement: new product features, backend API changes beyond defect fixes, native mobile shells.
- Progress table row (`0/8`) and top-of-roadmap checklist entry.

## Shared by both

- Phase number and title, dependency on Phase 12, the four core REQs (REQ-ux-web, REQ-quran-display, REQ-quran-research-tools, REQ-quran-result-contract).
- Light white/gray elegant theme, animated background, polished controls, consistent visual language.
- Every feature reachable through the UI; no CLI/API-only features.
- Layer-token preservation and visual distinction of canonical, translation, and annotation text.
- Build and cross-surface parity gates with no regressions.
- Accessibility and performance expectations.
- Component-driven design system and responsive shell.
- Quran reader, search, graph, hadith/tafsir, isnad, RAG, agents, scripture, management coverage.

---

# 3. Conflicts and Contradictions

**C-1 — Animated background implementation.**
R1 permits "CSS/canvas particle or gradient-mesh animation." R2 requires a "CSS-only, no canvas loop" gradient field. These are incompatible implementation constraints if R2 is binding. **Status: unresolved.** The consolidated design brief preserves both and flags the difference.

**C-2 — Button variant count and names.**
R1 specifies `Button` with 5 variants × 4 sizes × 6 states but does not name the variants. R2 names four variants: primary, secondary, ghost, danger. The fifth R1 variant is unspecified. **Status: unresolved.** The consolidated plan lists both the R1 scale and the R2 names.

**C-3 — Dark mode.**
R1 includes a dark-mode toggle wired to tokens while shipping light-only initially. R2 states "Light theme only. Dark theme is deferred." R1 includes plumbing for a future toggle; R2 excludes even the toggle from this phase. **Status: unresolved.** The consolidated document records both positions and lists dark theme as an R2 deferral.

**C-4 — Plan count and wave structure.**
R1 defines 9 plans across 7 waves. R2 defines 8 plans across 5 waves. The consolidated document adopts R1's 9-plan / 7-wave frame and maps R2's plans into it. **Status: structural disagreement, disclosed and mapped, not silently resolved.** If R2's structure is authoritative, the mapping table allows reversal.

**C-5 — Requirements list composition.**
R1 lists 15 requirements and omits REQ-security-licensing-observability. R2 lists 5 requirements and adds REQ-security-licensing-observability. This is additive rather than contradictory, but the two requirement sets are not identical. **Status: difference preserved; consolidated list is the union.**

**C-6 — Performance budget specificity.**
R1 gives explicit numbers: initial bundle ≤ 350 KB gzipped, LCP ≤ 1.5 s, CLS < 0.1, Lighthouse performance ≥ 80. R2 says "within a committed budget" without values. R2 does not contradict R1's numbers, but it does not endorse them either. **Status: R1 values retained as the concrete targets; R2's "committed budget" requirement retained.**

**C-7 — Font sourcing.**
R1 specifies a "system + Arabic `Amiri`/`Noto Naskh Arabic` stack." R2 requires "Self-hosted fonts only." A system-font component may conflict with a strict self-hosted-only rule. **Status: unresolved.** The consolidated typography bullet preserves both and flags the difference.

**C-8 — Accessibility measurement.**
R1 sets Lighthouse accessibility ≥ 90. R2 sets WCAG 2.2 AA contrast, axe zero serious/critical violations, and keyboard-only core flows. These use different tools and thresholds; they are complementary rather than contradictory. **Status: no direct contradiction; both retained.**

**C-9 — Backend-change scope.**
R2 explicitly states the redesign adds no backend capability, the Rust backend has no diff beyond defect fixes, and backend API changes beyond defect fixes are out of scope. R1 does not state this constraint. **Status: R2 adds a constraint; R1 is silent, not contradictory.** The consolidated document adopts R2's constraint and attributes it.

**C-10 — Feature enumeration in the reachability criterion.**
R1 enumerates read, search, inspect, graph, hadith/tafsir, isnad, RAG, agents, comparative scripture, management. R2 enumerates read, search, normalization, linguistics, graph, hadith, tafsir, isnad, multi-RAG, scripture, agents, tool lifecycle, API auth, source catalog, management dashboard. R2's list is broader; R1's "inspect" and "comparative scripture" are not verbatim in R2's list, and R2's "normalization," "tool lifecycle," "API auth," and "source catalog" are not verbatim in R1's. **Status: additive difference; consolidated criterion includes the union.**

---

# 4. Uncertainties, Gaps, and Limitations

- **Repository verification is absent.** R2 explicitly cannot see the repository and references the existing SPA, layer tokens, and typed client. Whether those artifacts match the roadmap descriptions is unverified. R2 recommends running `/gsd-plan-phase 13` before execution.
- **No source specifies the fifth R1 button variant.** The names of all five variants, four sizes, and six states are not enumerated in either source.
- **Exact design-token values are incomplete.** Only `#FFFFFF` and the range endpoints `#F5F5F4` … `#292524` are given; intermediate gray steps, the accent hue, typography sizes, spacing scale, radii, and shadow values are not specified.
- **Animated background remains technically undecided** between a CSS-only gradient field (R2) and a CSS/canvas particle or gradient-mesh approach (R1).
- **Dark-mode scope is undecided**: a wired-but-hidden toggle (R1) versus full deferral (R2).
- **Font licensing is unresolved.** R2 records it as owner gate OD-NEW (OD-15 if free), BLOCKED pending ratification, with the exact closing command unspecified in the supplied text.
- **Plan structure is a choice, not a fact.** The consolidated 9-plan frame follows R1; R2's 8-plan frame is preserved through the mapping table.
- **Route names and existing deep-link structure are not supplied.** R2 requires preserved deep links but neither source lists the existing routes.
- **Cross-surface parity fixture details are not supplied.** Both sources reference the Phase 5 fixture suite and `research_checksum`; the actual fixture cases are not in the supplied material.
- **R1 has no owner gates, deferrals, out-of-scope section, progress row, or checklist entry.** These are supplied only by R2 and are therefore attributed to R2 in the consolidated document.
- **R2 has no per-plan visual detail comparable to R1's** for linguistics, graph, hadith, RAG, and management screens; the consolidated document relies on R1 for those specifics and does not infer missing R2 positions.
- **No source states whether the existing SPA is replaced or incrementally migrated.** R2 says "from-scratch redesign" and "replaces the placeholder block"; R1 says "redesigned-from-scratch." The exact migration/removal strategy for existing SPA code is unstated.

---

# 5. Coverage and Integrity Report

**Sources processed:**

- **R0** — roadmap with Phase 13 placeholder. Supplied three times as identical files (files 2, 3, 4); verified identical in content and treated as one source.
- **R1** — 9-plan / 7-wave Phase 13 definition (file 5). Fully processed.
- **R2** — 8-plan / 5-wave Phase 13 definition with design brief, owner gates, deferrals, and roadmap entries (file 6). Fully processed.

**Source-by-source comparison performed:** Yes. Every plan, success criterion, design-brief bullet, requirement, owner gate, deferral, out-of-scope item, progress row, and checklist entry in R1 and R2 was extracted and mapped. R0's placeholder Phase 13 block (goal, requirements, four success criteria, zero plans) was compared against both expansions.

**Content that could not be faithfully integrated:** None was omitted. The only structural choice was adopting R1's 9-plan / 7-wave frame for the consolidated roadmap block and mapping R2's 8 plans into it; the mapping table preserves R2's structure and all R2-specific content. The unresolved technical conflicts (C-1, C-2, C-3, C-7) are preserved rather than resolved.

**Technical artifacts:** No code, formulas, tables, schemas, configuration files, or command-line instructions were present in the source responses beyond named libraries, IDs, CSS values, font names, and plan identifiers. These were preserved verbatim. No technical content was silently modified or repaired.

**External information:** None was introduced. No web search, repository access, or prior knowledge was used to fill gaps. The R2 caveat about repository verification is preserved as an uncertainty.

**Exhaustiveness claim:** This report does not claim the consolidated output is exhaustive beyond the supplied inputs. It is a complete consolidation of R0, R1, and R2 as provided. Verification against the actual repository, existing SPA code, typed client, route table, and Phase 5 parity fixtures remains outstanding and is explicitly flagged.

**UI hint**: yes

---
*Notes: Roadmap phases 1–12 track PRD §43/§88 Phases 0–12; PRD Phase 9 (agentic) + Phase 10 (tool creation/workflows) merge into roadmap Phase 10 (shared runtime + acceptance gates). Single-requirement phases (4, 7, 9) are standalone major capabilities, not thin slices. MVP acceptance (§44.3) lands in Phase 10; Phases 11–12 are post-MVP.*
