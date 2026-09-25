# Requirements: Q-ai

**Defined:** 2026-09-23
**Core Value:** Trustworthy Quran research: exact canonical text before generated interpretation, every factual claim traceable to its source.
**Source:** `docs/01-requirements/requirements.md` (single PRD); intel `.planning/intel/requirements.md` (32 entries).

## v1 Requirements

### Product Foundation

- [ ] **REQ-product-vision**: Platform covers Quran, vocabulary/morphology/roots/linguistic relations, hadith, Islamic literature, comparative scripture (PRD §1)
- [ ] **REQ-product-principles**: All work obeys the 19 product principles (exact-text-first, traceability, layer separation, no fabrication, locality, deny-by-default, no false consensus) (PRD §2)
- [ ] **REQ-goals-non-goals**: Complete validated Quran research system with exact Arabic search first; no from-scratch foundation model, no sectarian authority claims (PRD §3, §4, §25.14)
- [ ] **REQ-knowledge-domains**: Knowledge domains modeled independently so users can include or exclude them (PRD §5)
- [ ] **REQ-architecture-principles-quality**: Mandatory architecture principles, quality requirements (correctness, performance, reliability, maintainability, extensibility, UX), terminology (PRD §55, §56, §92, §96)

### Data & Quran Corpus

- [x] **REQ-data-separation-layers**: Explicit trust layers A (canonical) / B (publisher metadata) / C (scholarly) / D (computational) / E (user+AI notes) (PRD §6)
- [x] **REQ-quran-corpus**: Validated canonical representation: required hierarchy, edition model, immutable text, stable addressing (PRD §7)
- [x] **REQ-ingestion-validation-eval**: Discover→stage ingestion pipeline; Quran/hadith/citation validation; Quran/hadith/RAG/agent/tool evaluation (PRD §34, §35, §36, §86)
  - Acceptance: a Quran feature is complete only when it cannot modify canonical text accidentally, has corpus-integrity + Unicode/Arabic tests, preserves exact addressing, outputs source versions, labels generated analysis, works without an LLM where deterministic suffices, has performance limits, and has API/UI/domain tests (PRD §46)

### Quran Search, Linguistics & Tools

- [ ] **REQ-quran-normalization**: Search normalization that never replaces displayed canonical text; indexed search forms, user-controlled normalization, concatenated-word search (PRD §8)
- [ ] **REQ-quran-linguistics**: Token linguistic data with multiple analyses and Arabic word families (PRD §9)
- [ ] **REQ-quran-research-tools**: First-class typed tools (UI/API/workflow/agent-usable): exact/normalized search, root/morphology, frequency/distribution, context/comparison, graph, rhetorical/structural, discovery (PRD §11)
- [ ] **REQ-quran-result-contract**: Every Quran tool result includes the stated result fields including a research checksum (PRD §12)
- [ ] **REQ-quran-graph**: First-class graph representation independent of vector search: node/edge types, edge provenance, graph search, explainability, semi-automated annotation (PRD §10)
- [ ] **REQ-quran-display**: Web GUI reading view, research view, citation interaction, visual canonical/translation/annotation distinction (PRD §13)

### Hadith, Tafsir & Scripture

- [ ] **REQ-hadith-corpus**: Source-aware structured hadith records with metadata, Shia collections, attributed grading, hadith tools (PRD §14)
  - Acceptance: complete only when it identifies collection+edition, preserves source numbering, separates isnad/matn where supported, attributes gradings, represents disagreement, resolves citations exactly, and never presents computational identity matching as certainty (PRD §46)
- [ ] **REQ-isnad-narrator-graph**: Isnad/narrator graph with node/edge types and identity-uncertainty handling (PRD §15)
- [ ] **REQ-tafsir-comparative**: Tafsir indexed by work/author/dimensions; comparative scripture (Torah, Tanakh/Hebrew Bible, NT) with comparative tools (PRD §16, §17)

### Retrieval, Answers & Sources

- [ ] **REQ-multi-rag**: Multiple independently configured RAG systems with structure-aware ingestion/chunking, smart retrieval (deterministic + model-assisted router), routing rules, tool-plan visibility (PRD §18, §19, §20)
- [ ] **REQ-generic-rag-platform**: Generic document processing/chunking, portable RAG projects, declarative RAG config, composition, import sources, web views, generic-content storage (PRD §25.5–§25.12)
- [ ] **REQ-answer-contract**: Structured evidence answers: sections, claim-level citations, citation validation, side-by-side disagreement (PRD §21)
- [ ] **REQ-source-catalog-trust**: Central source catalog (records, states, internet updates, signed manifests, update safety, genealogy) + per-source trust profiles (PRD §22, §23)

### Interfaces

- [ ] **REQ-ux-web**: Web navigation, search modes, research workspace, export; server dashboard; multi-mode research query interface; streaming (PRD §24, §51, §52, §53)
- [ ] **REQ-tui-cli**: ratatui/crossterm TUI operational cockpit (screen inventory, palette, RAG debug), consistent `qai` CLI tree, model connection manager, agent screens (PRD §25–§25.4, §60, §78)
- [ ] **REQ-cli-api**: Executable `qai`; versioned API (Quran, hadith, source, research/query surfaces); cancellable long-running operations (PRD §26, §27, §83, §54)

### Agents, Models & Engineering

- [ ] **REQ-agents-tools-runtime**: Controlled first-class agents (definitions, agencies, limits); typed tool manifests, WASM/WASI sandboxing, deny-by-default approvals, MCP, five memory types; AI-assisted tool creation; workflow engine (PRD §28, §29, §30, §63–§72, §79, §80)
  - Acceptance: agent features complete only with enforced permissions, schema-validated I/O, cancellation/timeouts/limits, recorded versions, validated citations, untrusted-content handling, inspectability (PRD §46); tool creation complete only via describe→review→generate→isolated-test→accuracy-test→security-review→approve→version→sandbox→telemetry→revocable flow, unpublished tools never reach production agents (PRD §90); plus full §93 checklist (redaction, audit, streaming, retry/failure tests, prompt-injection + unauthorized-access tests, reversibility, rebuildable indexes, no silent canonical modification)
- [ ] **REQ-models-routing**: LLM/embedding abstractions, provider support, unified capability interface, routing with fallbacks (PRD §31, §61, §62)
- [ ] **REQ-engineering-baseline**: Async architecture + perf targets, typed errors, secrets, validated config, observability, dependency principles, localhost/auth/TLS baseline, testing, DX, adapters, Docker, local-first (PRD §25.13)
  - Acceptance: feature complete only when implemented, tested (unit+integration), typed errors, observability, docs, interface coverage, no layer coupling, validated config, cancellation/timeouts, redaction, provenance, versioning, access controls, recovery tests, clean fmt/clippy/test (PRD §58)
- [ ] **REQ-storage-architecture**: Relational (SQLite now, PostgreSQL server), full-text, vector, graph, object/file stores behind abstractions; Rust workspace layout (PRD §32, §33, §87)
- [ ] **REQ-security-licensing-observability**: Localhost bind, remote auth, production TLS; licensing gates before activation; structured logging/tracing with provenance; performance, backups, auth/access control (PRD §37–§42, §84, §85)
- [ ] **REQ-retrieval-governance**: Prompt-injection defense (retrieved content untrusted), access-control-aware retrieval, idempotent version-aware lifecycle, audit trail + provenance, budgets, sessions (PRD §74, §75, §76, §81, §82, §77)

### Roadmap & Acceptance

- [ ] **REQ-phases-mvp**: Build order Phases 0–12; MVP = local-first app (GUI/CLI/basic TUI, SQLite, one validated edition, navigation, exact/diacritic-insensitive/concatenated/root/lemma/family search, morphology display, frequency tools, basic graph, attributed translations, citations, tool-using research chat, providers, run inspection, integrity tests, manual import, versioned manifests); MVP + post-MVP acceptance workflows; success metrics (PRD §43–§45, §47, §88, §89)
- [ ] **REQ-open-decisions-status**: Open ADRs resolved or explicitly deferred; status checklists; `qai doctor` diagnostics; living-PRD rules; change logs (PRD §48, §49, §50, §91, §94, §95, §97, §25.15)

## v2 Requirements

Deferred post-MVP extensions (from PRD §44.2 excludes — acknowledged, not in current roadmap):

- **V2-01**: Additional hadith collections beyond the initial approved set
- **V2-02**: Additional Quran qira'at beyond the initial validated edition(s)
- **V2-03**: Full autonomous internet source updates (manual import + approval inbox first)
- **V2-04**: Automated narrator identity resolution (review workflow first; never presented as certainty)
- **V2-05**: Multi-user collaboration

## Out of Scope

| Feature | Reason |
|---------|--------|
| Foundation model built from scratch | Not a model lab; local-first research platform (PRD §4, §25.14) |
| One sectarian interpretation as authoritative | Platform represents disagreement, never resolves by fiat (PRD §4) |
| Every vector DB natively | Backend-neutral ports + selected adapters only (PRD §25.14) |
| Unrestricted generated tools / arbitrary shell | Deny-by-default permissions; sandbox confinement (PRD §44.2) |
| Authoritative religious rulings | No false scholarly-consensus claims (PRD §2, §44.2) |
| Unreviewed AI changes to canonical corpora | Human ApprovalToken required for canonical writes (ADR-0000, PRD §44.2) |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| REQ-product-vision | Phase 1 | Pending |
| REQ-product-principles | Phase 1 | Pending |
| REQ-goals-non-goals | Phase 1 | Pending |
| REQ-engineering-baseline | Phase 1 | Pending |
| REQ-storage-architecture | Phase 1 | Pending |
| REQ-cli-api | Phase 1 | Pending |
| REQ-architecture-principles-quality | Phase 1 | Pending |
| REQ-quran-corpus | Phase 2 | Complete |
| REQ-data-separation-layers | Phase 2 | Complete |
| REQ-ingestion-validation-eval | Phase 2 | Complete |
| REQ-quran-normalization | Phase 3 | Pending |
| REQ-quran-linguistics | Phase 3 | Pending |
| REQ-quran-graph | Phase 4 | Pending |
| REQ-quran-display | Phase 5 | Pending |
| REQ-quran-research-tools | Phase 5 | Pending |
| REQ-quran-result-contract | Phase 5 | Pending |
| REQ-tui-cli | Phase 5 | Pending |
| REQ-hadith-corpus | Phase 6 | Pending |
| REQ-tafsir-comparative | Phase 6 | Pending |
| REQ-isnad-narrator-graph | Phase 7 | Pending |
| REQ-multi-rag | Phase 8 | Pending |
| REQ-generic-rag-platform | Phase 8 | Pending |
| REQ-knowledge-domains | Phase 9 | Pending |
| REQ-agents-tools-runtime | Phase 10 | Pending |
| REQ-models-routing | Phase 10 | Pending |
| REQ-answer-contract | Phase 10 | Pending |
| REQ-phases-mvp | Phase 10 | Pending |
| REQ-source-catalog-trust | Phase 11 | Pending |
| REQ-ux-web | Phase 11 | Pending |
| REQ-retrieval-governance | Phase 11 | Pending |
| REQ-security-licensing-observability | Phase 12 | Pending |
| REQ-open-decisions-status | Phase 12 | Pending |

**Coverage:**

- v1 requirements: 32 total
- Mapped to phases: 32
- Unmapped: 0 ✓

---
*Requirements defined: 2026-09-23*
*Last updated: 2026-09-23 after ingest bootstrap*
