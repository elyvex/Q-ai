# Requirements (synthesized from 1 PRD: docs/01-requirements/requirements.md)

One PRD covering the full platform. Entries are grouped by PRD section; each
entry's description states what the cited section requires. Acceptance criteria
are recorded only where the source states explicit acceptance/complete-only
conditions; otherwise marked absent (never inferred).

## REQ-product-vision
- source: docs/01-requirements/requirements.md
- description: Q-ai is a research platform for studying the Quran, Quranic vocabulary/morphology/roots/linguistic relationships, hadith collections, Islamic literature, and comparative scripture (Torah, Tanakh, New Testament) (PRD §1).
- acceptance: absent
- scope: PRD §1 product vision

## REQ-product-principles
- source: docs/01-requirements/requirements.md
- description: Q-ai must follow 19 stated principles, including exact text before generated interpretation, claim traceability, structural Quran storage, canonical/translation/annotation separation, no presentation of translations as the original, attributed hadith grading, no silent merging of school views, normalization never modifying display text, untrusted internet content, no fabricated verses/hadith/chains/gradings/citations, quotation/summary/analysis distinction, reproducibility, local-first data, deny-by-default agent/tool permissions, and no false scholarly-consensus or religious-authority claims (PRD §2).
- acceptance: absent (principles are constraints on all acceptance, not separately gated in the source)
- scope: PRD §2 product principles

## REQ-goals-non-goals
- source: docs/01-requirements/requirements.md
- description: Primary goals include a complete validated Quran research system with exact Arabic search; secondary goals extend coverage; initial releases will not build a foundation model from scratch nor declare one sectarian interpretation universally authoritative (PRD §3, §4, §25.14).
- acceptance: absent
- scope: PRD §3 goals; §4 and §25.14 non-goals

## REQ-knowledge-domains
- source: docs/01-requirements/requirements.md
- description: Q-ai must model knowledge domains independently so users can include or exclude them (PRD §5).
- acceptance: absent
- scope: PRD §5 supported knowledge domains

## REQ-data-separation-layers
- source: docs/01-requirements/requirements.md
- description: Q-ai must separate data into explicit trust layers: Layer A canonical source text, Layer B publisher/dataset metadata, Layer C scholarly annotation, Layer D computational annotation, Layer E user and AI notes (PRD §6).
- acceptance: absent
- scope: PRD §6 authoritative data separation

## REQ-quran-corpus
- source: docs/01-requirements/requirements.md
- description: The Quran engine must preserve a validated canonical representation with a required Quran hierarchy, an edition model, immutable canonical text, and stable Quran addressing (PRD §7).
- acceptance: absent
- scope: PRD §7 canonical Quran corpus requirements

## REQ-quran-normalization
- source: docs/01-requirements/requirements.md
- description: Normalization is required for searching but must never replace displayed canonical text; tokens/phrases support separately indexed search forms plus user-controlled normalization and concatenated-word search (PRD §8).
- acceptance: absent
- scope: PRD §8 Quran text normalization

## REQ-quran-linguistics
- source: docs/01-requirements/requirements.md
- description: Quran tokens carry linguistic data with support for multiple analyses and Arabic word families (PRD §9).
- acceptance: absent
- scope: PRD §9 Quran linguistic model

## REQ-quran-graph
- source: docs/01-requirements/requirements.md
- description: The Quran must have a first-class graph representation independent of vector search, with core node types, core edge types, edge provenance, graph search, query explainability, and semi-automated annotation (PRD §10).
- acceptance: absent
- scope: PRD §10 Quran knowledge graph

## REQ-quran-research-tools
- source: docs/01-requirements/requirements.md
- description: Quran research tools must be first-class typed tools usable by UI, API, workflows, and authorized agents, covering exact/normalized search, root/morphology tools, frequency/distribution tools, context/comparison tools, graph tools, rhetorical/structural tools, and discovery tools (PRD §11).
- acceptance: absent
- scope: PRD §11 Quran research tools

## REQ-quran-result-contract
- source: docs/01-requirements/requirements.md
- description: Every Quran tool result must include the stated result fields including a research checksum (PRD §12).
- acceptance: absent
- scope: PRD §12 Quran tool result contract

## REQ-quran-display
- source: docs/01-requirements/requirements.md
- description: The Web GUI must present a reading view, research view, citation interaction, and visual distinction between canonical text, translations, and annotations (PRD §13).
- acceptance: absent
- scope: PRD §13 Quran display requirements

## REQ-hadith-corpus
- source: docs/01-requirements/requirements.md
- description: Hadith must use source-aware structured records (not only RAG chunks) with defined structure, metadata, Shia collections, an attributed grading model, and hadith tools (PRD §14).
- acceptance: A hadith feature is complete only when it identifies collection and edition, preserves source numbering, distinguishes isnad and matn where supported, attributes gradings, represents disagreement, resolves citations to exact locations, and avoids presenting computational identity matching as certainty (PRD §46).
- scope: PRD §14 hadith corpus requirements

## REQ-isnad-narrator-graph
- source: docs/01-requirements/requirements.md
- description: Isnad and narrator graph with defined node types, edge types, and identity-uncertainty handling (PRD §15).
- acceptance: absent
- scope: PRD §15 isnad and narrator graph

## REQ-tafsir-comparative
- source: docs/01-requirements/requirements.md
- description: Tafsir indexed by work, author, and stated dimensions; comparative scripture (Torah, Tanakh/Hebrew Bible, New Testament) with comparative tools (PRD §16, §17).
- acceptance: absent
- scope: PRD §16 tafsir and commentary; §17 comparative scripture

## REQ-multi-rag
- source: docs/01-requirements/requirements.md
- description: Q-ai must support multiple independently configured RAG systems with structure-aware ingestion/chunking (Quran, hadith, tafsir, books), smart retrieval with a deterministic and model-assisted router, routing rules, and tool-plan visibility (PRD §18, §19, §20).
- acceptance: absent
- scope: PRD §18 multi-RAG architecture; §19 ingestion and chunking; §20 smart retrieval and tool selection

## REQ-answer-contract
- source: docs/01-requirements/requirements.md
- description: Research answers use a structured evidence model with answer sections, claim-level citations, citation validation, and side-by-side disagreement representation (PRD §21).
- acceptance: absent
- scope: PRD §21 research answer contract

## REQ-source-catalog-trust
- source: docs/01-requirements/requirements.md
- description: A central source catalog with catalog records, source states, internet catalog updates, trusted catalog manifests, update safety, and source genealogy; every source has a configurable trust profile (PRD §22, §23).
- acceptance: absent
- scope: PRD §22 book and source catalog; §23 source quality and trust

## REQ-ux-web
- source: docs/01-requirements/requirements.md
- description: Main web navigation, search modes, research workspace, and export (PRD §24); server dashboard (§51); research query interface with multiple research modes (§52); streaming for LLM responses and research progress (§53).
- acceptance: absent
- scope: PRD §24 user experience; §51 server dashboard; §52 research query interface; §53 streaming

## REQ-tui-cli
- source: docs/01-requirements/requirements.md
- description: TUI built on ratatui/crossterm as the operational cockpit with a detailed screen inventory, command palette, RAG debugging view, and consistent `qai` CLI command tree with global flags and conventions; TUI model connection manager and agent experience screens (PRD §25–§25.4, §60, §78).
- acceptance: absent
- scope: PRD §25 TUI requirements; §25.1–§25.4 screens, palette, RAG debug, CLI design; §60 model connection manager; §78 TUI agent experience

## REQ-generic-rag-platform
- source: docs/01-requirements/requirements.md
- description: Generic document processing (initial formats, metadata, pipeline), modular corpus-aware chunking with per-chunk metadata, reusable portable RAG projects, declarative RAG configuration schema, multiple-RAG composition, import sources, platform-level web GUI views, and generic-content storage architecture (PRD §25.5–§25.12).
- acceptance: absent
- scope: PRD §25.5 generic document processing; §25.6 chunking; §25.7 RAG projects; §25.8 RAG configuration; §25.9 composition; §25.10 import sources; §25.11 web GUI navigation; §25.12 storage for generic content

## REQ-engineering-baseline
- source: docs/01-requirements/requirements.md
- description: Cross-cutting engineering standards: async architecture with performance targets, typed error handling, secrets handling, validated configuration, observability, dependency principles, security baseline (localhost bind, remote auth, TLS), testing strategy, developer experience, generic adapters, Docker, and local-first design (PRD §25.13).
- acceptance: A feature is complete only when implemented, unit- and integration-tested where required, with typed errors, observability, docs, intended-interface coverage, no layer coupling, validated config, working cancellation/timeouts, redaction, provenance, version handling, access controls, recovery tests, and clean fmt/clippy/test (PRD §58).
- scope: PRD §25.13 performance/error/engineering baseline

## REQ-cli-api
- source: docs/01-requirements/requirements.md
- description: The executable is named `q-ai` or `qai` with specified CLI examples (PRD §26); a versioned API with Quran, hadith, source, and research/query surfaces plus later additions (PRD §27, §83); long-running operations must be cancellable with user-driven cancel of research queries and stated items (PRD §54).
- acceptance: absent
- scope: PRD §26 CLI requirements; §27 API requirements; §83 API additions; §54 cancellation

## REQ-agents-tools-runtime
- source: docs/01-requirements/requirements.md
- description: Agents are first-class but controlled (initial agents, research agencies, agent limits); tools use typed manifests with sandboxing (WASM/WASI preferred), deny-by-default permissions with human approval, MCP integration, and five memory types; AI-assisted tool creation and a workflow engine with stated nodes (PRD §28, §29, §30, §63–§72, §79, §80).
- acceptance: An agent feature is complete only when tool permissions are enforced, inputs/outputs schema-validated, cancellation/timeout and resource limits work, tool/model versions are recorded, citations validated, retrieved content treated as untrusted, and the user can inspect what happened (PRD §46). Tool creation is complete only when a user can describe, review (schemas, sources, permissions), generate, build without modifying the host, test in isolation, run corpus-accuracy tests, inspect security results, approve/reject, version, sandbox-execute, inspect run telemetry, and immediately disable/revoke it; unpublished tools must not reach production agents (PRD §90). Agent/tool/corpus/retrieval features additionally require permission checks, cancellation, timeouts, limits, schema validation, redaction, audit events, streaming where applicable, tested retry/failure behavior, prompt-injection tests, unauthorized-access tests, documented side effects, recorded model/prompt/tool/source/edition versions, sandbox confinement of generated code, inspectability, resolving citation links, matching quotations, reversible corpus updates, rebuildable indexes, and no silent canonical modification (PRD §93).
- scope: PRD §28 agentic runtime; §29 tool runtime and security; §30 AI-assisted tool creation; §63 agent definition; §64 runtime; §65 run events; §66 agencies; §67 tool system; §68 agent-created tools; §69 sandbox; §70 permissions and approval; §71 MCP; §72 memory; §79 tool builder UI; §80 workflow engine

## REQ-models-routing
- source: docs/01-requirements/requirements.md
- description: Abstractions for LLMs and embeddings with provider support, a unified model capability interface, and model routing with fallback aliases (PRD §31, §61, §62).
- acceptance: absent
- scope: PRD §31 model and provider support; §61 capability interface; §62 routing and fallback

## REQ-storage-architecture
- source: docs/01-requirements/requirements.md
- description: Multiple storage types behind abstractions: relational metadata store (SQLite initial, PostgreSQL future/server), full-text index, vector store, graph store, object/file storage; suggested Rust workspace layout (PRD §32, §33, §87).
- acceptance: absent
- scope: PRD §32 storage architecture; §33 suggested Rust workspace; §87 updated project structure

## REQ-ingestion-validation-eval
- source: docs/01-requirements/requirements.md
- description: Data ingestion pipeline (discover → stated stages); validation requirements for Quran (surah count, valid verse identifiers, no missing/duplicates, stable token order, Unicode validity, checksum, round-trip, reference-corpus comparison, normalization not modifying display), hadith, and citations; evaluation for Quran, hadith, RAG, agents, and tools/agencies (PRD §34, §35, §36, §86).
- acceptance: A Quran feature is complete only when it cannot modify canonical text accidentally, has corpus-integrity tests, Unicode/Arabic normalization tests, preserves exact addressing, outputs source versions, labels generated analysis, works without an LLM where deterministic processing suffices, has performance limits, and has API/UI/domain tests (PRD §46).
- scope: PRD §34 ingestion pipeline; §35 validation; §36 evaluation; §86 evaluation requirements

## REQ-security-licensing-observability
- source: docs/01-requirements/requirements.md
- description: Security controls (localhost bind, remote auth, production TLS, additional requirements); copyright/licensing gates (recorded licensing status before activation); structured logging/tracing with provenance; performance, reliability/backups, and authentication/access-control requirements (PRD §37, §38, §39, §40, §41, §42, §84, §85).
- acceptance: absent
- scope: PRD §37 security; §38 copyright and licensing; §39 observability and provenance; §40 performance; §41 reliability and backups; §42 authentication and access control; §84 additional security; §85 reliability

## REQ-retrieval-governance
- source: docs/01-requirements/requirements.md
- description: Prompt-injection defense treating retrieved content as untrusted data; access-control-aware retrieval with authorization applied before results; idempotent version-aware data lifecycle with content-hash dedup and index consistency; audit trail and provenance for audited actions; budgets and resource governance; conversation/session management (PRD §74, §75, §76, §81, §82, §77).
- acceptance: absent
- scope: PRD §74 prompt-injection defense; §75 access-control-aware retrieval; §76 data lifecycle and index consistency; §81 budgets; §82 audit trail and provenance; §77 conversation and session management

## REQ-phases-mvp
- source: docs/01-requirements/requirements.md
- description: Implementation order across Phases 0–12 (foundations; canonical Quran core; search/linguistics; graph; rich experience; hadith/tafsir; isnad; multi-RAG; comparative scripture; agentic research; tool creation/workflows; server/management; hardening); MVP includes local-first app with Web GUI/CLI/basic TUI, SQLite, one validated Quran edition, navigation, exact/diacritic-insensitive/concatenated/root/lemma/word-family search, morphology display, frequency tools, basic graph, attributed translations, citations, tool-using research chat, providers, run inspection, integrity tests, manual import, and versioned manifests, with stated excludes and acceptance workflows (PRD §43, §44, §45, §88, §89).
- acceptance: The MVP acceptance workflow and post-MVP acceptance workflow (select Quran, Al-Kafi, Bihar al-Anwar, and tafsir sources with downstream steps) are stated in PRD §44.3 and §45; product success metrics (integrity pass rate, retrieval accuracy, and stated items) in PRD §47.
- scope: PRD §43 development phases; §44 MVP definition; §45 post-MVP acceptance; §47 success metrics; §88 updated phases; §89 revised MVP

## REQ-architecture-principles-quality
- source: docs/01-requirements/requirements.md
- description: Mandatory architecture principles (separation of concerns, dependency inversion, and stated items; updated principles including deny-by-default tool permissions and human control over external side effects); quality requirements for correctness, performance, reliability, maintainability, extensibility, and UX; future features; terminology definitions (PRD §55, §56, §57, §65.1–§65.2 equivalent terminology §96, §92).
- acceptance: absent
- scope: PRD §55 architecture principles; §56 quality requirements; §57 future features; §92 updated architecture principles; §96 terminology

## REQ-open-decisions-status
- source: docs/01-requirements/requirements.md
- description: Open technical decisions requiring ADRs (Quran dataset/license, morphology dataset, normalization and transliteration standards, hadith/tafsir sources and licensing, graph storage, full-text engine, vector store, embedding model, reranker, narrator identity, hadith numbering, frontend framework, RTL terminal, WebSocket vs SSE, manifest signing, catalog trust, graph export, citation verification, rate limits — PRD §48; plus TUI/streaming and stated additions — PRD §91; plus generic-platform items — PRD §25.15); current project status checklists; doctor/corpus diagnostics; living-PRD rules; change logs (PRD §49, §50, §94, §95, §97).
- acceptance: absent (open items are inputs to future ADRs, not acceptance gates)
- scope: PRD §48 open technical decisions; §91 additional open decisions; §25.15 generic open decisions; §49 status; §50 doctor diagnostics; §94 living PRD rules; §95/§97 change logs
