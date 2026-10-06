# Phase 5: Rich Quran Experience - Context

**Gathered:** 2026-10-06
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver the **user-facing experience layer** for the already-shipped engine: a **Web GUI** (React + Vite SPA embedded in the `qai` binary) for reading, researching, and citing the Quran; a **TUI cockpit** (ratatui/crossterm) for daily operations and research; and the **typed-tool / result-contract completion** that makes every Quran tool return identical results with a research checksum across UI, API, and CLI. It realizes roadmap SC1–SC5: RTL reading with translation panels, word inspector, and deep links; citation copy that re-verifies at the exact source location; UI/API/CLI parity with a research checksum; a TUI cockpit (command palette + RAG debug view); and unmistakable visual separation of canonical text, translation, and annotation.

This phase **consumes** the Phase 1–4 engine surfaces (canonical reader, search/linguistics, counting, graph, tool registry, versioned HTTP API, CLI) and adds **no new canonical/linguistic/graph semantics**. It is the interface and result-contract layer on top of them.

**In scope (from ROADMAP §Phase 5 + REQ-quran-display / REQ-quran-research-tools / REQ-quran-result-contract / REQ-tui-cli):**
- Web GUI: reading view, research view, citation interaction, visual canonical/translation/annotation distinction (PRD §13, §24, §51, §52, §53).
- TUI operational cockpit: screen inventory, command palette, RAG debug view (PRD §25–§25.4, §60, §78).
- Research tools reachable from UI, API, and CLI with identical results + research checksum (PRD §11, §12).
- Result contract completion: every Quran tool result carries the stated fields incl. a research checksum (PRD §12).

**Out of scope:** any change to canonical text, reference grammar, morphology invariants, or frozen hash recipes (locked ADR-0102…0113); new graph semantics or interactive graph **explorer** (Phase 4 delivered static rendering only — the explorer lands here as a GUI/TUI *view*, but graph store/traversal/annotation **contracts** are frozen); hadith/tafsir (Phase 6); isnad (Phase 7); real multi-RAG retrieval + routing (Phase 8 — this phase only scaffolds the RAG debug view against the typed envelope); comparative scripture (Phase 9); agent runtime / tool creation / workflows (Phase 10); production auth/RBAC/TLS and server management (Phases 11–12).

**Boundary contract:** `.planning/ROADMAP.md` §Phase 5 (five success criteria) plus `REQ-quran-display`, `REQ-quran-research-tools`, `REQ-quran-result-contract`, `REQ-tui-cli`. The legacy boards and ADRs are authoritative **evidence/reference**, not a mandate to close every legacy task.

</domain>

<decisions>
## Implementation Decisions

### Web GUI Delivery Shape
- **D-01:** Ship the Web GUI as a **single-page application** (not server-rendered HTML), consuming the versioned HTTP API (`/api/v1`) — **Reversibility:** costly — the SPA/API boundary and route shapes are consumed by the UI and later phases (streaming, auth) and are hard to un-wind once published.
- **D-02:** Build the SPA with **JavaScript + TypeScript, React + Vite** (not Rust/WASM). The `qai` binary stays Rust; the SPA is a separate build artifact — **Reversibility:** costly — the frontend toolchain and component ecosystem become the project's UI foundation.
- **D-03:** **Embed the built SPA assets in the `qai` binary** and serve them same-origin with `/api/v1` (single self-contained binary, offline by default, no CORS). Not serve-from-disk, not dev-server-only — **Reversibility:** costly — changes the release/build pipeline and the "single binary" deployment guarantee.
- **D-04:** The SPA must run **fully offline against a local `qai serve`**; no CDN or network egress by default (honors the local-first / deny-egress constraint). — **Reversibility:** reversible — a packaging constraint.

### Reading View Layout
- **D-05:** Primary reading layout is a **single centered RTL column with collapsible side panels** (navigation, translation, word inspector, research tools). Not a three-pane workspace, not a continuous mushaf page — **Reversibility:** costly — the shell layout is the container every view mounts into.
- **D-06:** **Translations are user-switchable among all three modes**: (a) inline beneath each ayah, (b) side-by-side columns, (c) toggleable side panel. Ship a display-mode control; do not pick one. — **Reversibility:** costly — three render paths are a UI contract and an a11y/RTL surface.
- **D-07:** **Word inspector is user-switchable among all three interaction modes**: (a) click/tap popover, (b) persistent side panel, (c) popover with a pin-to-panel action. — **Reversibility:** costly — interaction contract consumed by GUI, TUI, and later agent/UI reuse.
- **D-08:** **SC5 visual layer distinction uses all three approaches**: a **shared layer token set** (canonical / translation / annotation) as the baseline applied everywhere, **per-view styling** where a view needs it, **and user-configurable themes**. The shared token set is the consistency mechanism that keeps GUI/TUI/CLI from drifting. — **Reversibility:** costly — the layer token vocabulary is a published design contract reused by TUI and later phases.
- **D-09:** Canonical text, translations, and annotations remain **visually and structurally distinct in every view** (extends Phase 3 SC5 "displayed canonical text is never modified by normalization"); no view may render a translation or annotation in a canonical slot (ADR-0112). — **Reversibility:** one-way — undoing would violate a locked layer-separation contract.

### TUI Cockpit Depth
- **D-10:** Ship an **ops cockpit now**: command palette, doctor/health, jobs, index & corpus status, plus reader + search, plus **graph navigation and annotation-review screens**. Full GUI-parity TUI and thin-TUI-over-CLI were rejected. — **Reversibility:** costly — screen inventory and palette command set are a user-facing operator contract.
- **D-11:** Build the **RAG debug screen now against the typed `Envelope`**, rendering a typed "RAG not configured / unavailable" state until Phase 8 wires retrieval. Do **not** stub retrieval or defer the screen — SC4 is met honestly. — **Reversibility:** costly — the debug-view shape is consumed by Phase 8 retrieval and the agent runtime.
- **D-12:** The TUI reads data through the **shared application services layer** (same services / typed tools the API and CLI use), not by shelling out to the CLI and not by talking HTTP to the server. This makes parity structural. — **Reversibility:** costly — the data-access boundary determines whether parity is provable.
- **D-13:** TUI framework is **ratatui + crossterm** (PRD §25). — **Reversibility:** reversible — matches the specified stack.

### Result Contract & Research Checksum
- **D-14:** The **research checksum covers inputs + tool identity + result payload**: canonical inputs (refs / edition / `corpus_generation`), tool id + version, normalized parameters, and the result payload — computed with the frozen canonical-JSON + `sha256` recipe (ADR-0006) and domain-separated per ADR-0108 style. Recomputable and comparable across surfaces. — **Reversibility:** costly — the checksum definition is a published reproducibility contract; changing its inputs invalidates prior checksums.
- **D-15:** The checksum is a **required field on the shared `ToolResult` / `Envelope` contract** (extend the existing `crates/tools` contract), so every tool result carries it — not computed only at the presentation layer, not a separate verify endpoint. — **Reversibility:** one-way — a required field on a published contract consumed by all surfaces and later the agent runtime.
- **D-16:** Prove UI/API/CLI parity with a **cross-surface golden gate**: one shared fixture drives every tool through CLI, HTTP, and the UI service path and asserts **byte-identical payload + identical checksum**; enforced as a repeatable check (SC3). — **Reversibility:** costly — the gate is the evidence mechanism for the phase's central success criterion.
- **D-17:** Citation copy yields a **stable citation URN + deep link** (ADR-0111); opening it resolves and **re-verifies via `verify_quotation`**, displaying verified/mismatch, with **mismatch as a hard failure** on the answer path (SC2). Not a plain-text quote with manual verification. — **Reversibility:** one-way — citation identity and hard-fail-on-mismatch are locked contracts (ADR-0111).

### Agent's Discretion
- Component library and styling approach within React (so long as the shared layer token set and offline constraint hold).
- Routing / deep-link URL scheme, navigation structure, and page composition within the single-column shell.
- SPA build/bundling specifics (Vite config, asset embedding mechanism, dev workflow) so long as the binary is self-contained and offline.
- TUI keybindings, palette command inventory, widget decomposition, and terminal theme mapping so long as ratatui/crossterm, the shared services layer, and the layer token set are preserved.
- Exact `research_checksum` encoding/length and field naming within the frozen hashing recipe.
- Which research tools appear in which surface first (ordering), so long as every in-scope tool reaches UI + API + CLI with parity.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Contract
- `.planning/ROADMAP.md` §Phase 5 — goal, five success criteria, dependency on Phase 4, `UI hint: yes`.
- `.planning/REQUIREMENTS.md` — `REQ-quran-display` (PRD §13), `REQ-quran-research-tools` (PRD §11), `REQ-quran-result-contract` (PRD §12), `REQ-tui-cli` (PRD §25–§25.4, §60, §78).
- `.planning/PROJECT.md` — 25 locked ADRs, constraints (local-first, layer separation, citation contract), out-of-scope, proposed ADRs.
- `.planning/STATE.md` — current position and the Phase-4→5 handoff (OD-11, OD-12 BLOCKED; graph M5 blocked).
- `docs/01-requirements/requirements.md` — authoritative living PRD: §11 research tools, §12 result contract, §13 display, §24/§51/§52/§53 web UX + research workspace + streaming, §25–§25.4 TUI cockpit, §46 completeness rule, §60/§78 agent screens.

### Locked Architecture Decisions (must be respected)
- `docs/02-architecture/decisions/ADR-0111-citation-identity-and-deep-links.md` — citation identity/URNs, deep links; `verify_quotation` verdicts with mismatch as hard failure (SC2).
- `docs/02-architecture/decisions/ADR-0112-translation-alignment-and-attribution.md` — translation layer separation; no type-level path from translation into the canonical slot (SC5).
- `docs/02-architecture/decisions/ADR-0006-hashing-canonical.md` — canonical JSON + `sha256:<hex>` recipe the research checksum reuses.
- `docs/02-architecture/decisions/ADR-0008-provenance-representation.md`, `ADR-0009-audit-integrity.md`, `ADR-0010-error-taxonomy.md` — provenance/audit/error conventions all surfaces honor.
- `docs/02-architecture/decisions/ADR-0011-observability.md` — tracing/metrics conventions (OTLP opt-in, redaction).
- `docs/02-architecture/decisions/ADR-0004-configuration-precedence.md` — `bind 0.0.0.0` without TLS + auth hard-fails (server bind guard).
- `docs/02-architecture/decisions/ADR-0000-project-architecture.md` — layered workspace, single `qai` binary, local-first.
- `docs/02-architecture/decisions/ADR-0012-workspace-boundaries.md` — crate layer graph (`xtask arch-check`); new crates (`tui`) must respect it.

### Result Contract & Tools
- `crates/tools/src/lib.rs` — existing `ToolResult` contract to extend with the required `research_checksum`.
- `docs/08-api/quran-v1-openapi.json` — versioned API envelope conventions the SPA and TUI consume.
- `.planning/phases/04-quran-graph/04-CONTEXT.md` — Phase 4 D-11/D-12/D-13/D-14: read parity contract, explainability payload, and the deferred interactive explorer this phase delivers as a view.
- `.planning/phases/03-quran-search-linguistics/03-CONTEXT.md` — Phase 3 D-11 (explainability), D-15 (display pin); canonical-display invariants.

### Graph (view consumed here)
- `docs/02-architecture/decisions/ADR-0218-graph-export-formats.md` — Graph JSON v1 envelope the GUI/TUI graph views render.
- `docs/02-architecture/decisions/ADR-0217-graph-query-limits.md` — budgets/truncation semantics the graph views must surface (never render truncation as absence).

### Specs & Technical References
- `docs/07-technical/quran-citation-spec.md` — citation identity and resolution for deep links.
- `.planning/codebase/STACK.md` — current stack (axum, sqlx, clap, ratatui not yet wired) and conventions for adding the SPA/TUI toolchain.
- `.planning/codebase/STRUCTURE.md` — composition root, port/adapter rules, where new code goes; `crates/tui` is an empty placeholder.

### Open / Owner-Gated Decisions
- `docs/05-followups/decisions-needed.md` — OD-01…OD-14 (all human-only). Relevant here: OD-11 (morphology dataset/license) and OD-12 (normalization catalog + linguist) still gate the linguistic surfaces the inspector displays; agents must not close them.
- `docs/05-followups/phase-03-deferrals.md` — UI/result-contract research checksum deferred **to this phase** (Phase 5).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/server/src/api.rs` — axum `router()` / `serve()` with `/api/v1` routes, `Envelope` DTOs, and an existing server-rendered `debug_reader_handler` (`/debug/read/{edition}/{surah}`) — the closest precedent for a reading surface; the SPA is served same-origin from this server.
- `crates/cli/` — clap tree + `quran.rs` + `doctor.rs` + `exit_code.rs`; the CLI surface the parity gate exercises.
- `crates/tools/` + `crates/tool-registry/` — `ToolResult` contract and registered tools (`quran.get_ayah`, `quran.get_context`, graph tools) to extend with `research_checksum`.
- `crates/application/` — composition root; `quran_reader.rs` (canonical lookup + cache), `quran_cli.rs`, `quran_search.rs`, `quran_counting.rs`, `quran_lexicon_api.rs`, graph services — the **shared services layer** the TUI (D-12) and API call.
- `crates/quran-core/src/view.rs` — `AyahView` (canonical Arabic `QuranQuotation` only) — the typed view the reading UI renders.
- `crates/quran-graph/src/export.rs` — Graph JSON v1 + truncation notice — the payload the GUI/TUI graph views render.
- `crates/observability/` — tracing/metrics catalog the UI/TUI ops screens surface (doctor, jobs, index status).

### Established Patterns
- **Port/adapter + composition root**: only `crates/application` wires concrete backends; new UI/TUI code consumes services, never storage directly (ADR-0000/ADR-0012).
- **Typed envelopes + trycmd snapshots**: `crates/cli/tests/quran/*.trycmd` and `crates/server/tests/api.rs` establish the golden-snapshot pattern the cross-surface parity gate extends.
- **Frozen hashing/canonical-JSON** (ADR-0006) and domain-separated recipes (ADR-0108) — reuse, do not invent a new digest.
- **Local-first / deny egress**: `security.allow_network_egress = false`; the SPA and TUI must not require network access.
- **Brownfield gap-closure posture** (Phase 1 D-01, Phase 2 D-01, Phase 3 D-04, Phase 4 D-01): map each success criterion to current code + a repeatable check; create work only for missing behavior or weak evidence.

### Integration Points
- `crates/server` router: add same-origin static asset serving for the embedded SPA + citation deep-link routes.
- `crates/tui` (currently `//! Phase 9 placeholder`) — implement the ratatui/crossterm cockpit here; confirm/adjust the placeholder phase label to Phase 5.
- `crates/tools` / `crates/tool-registry`: add the required `research_checksum` field and populate it in every tool result.
- `crates/cli`: expose the research tools + checksum in human/JSON output; add citation-copy/deep-link verbs as needed.
- `xtask` / CI: add the cross-surface parity gate and SPA build/embed step to the existing 9-step `xtask ci`.

</code_context>

<specifics>
## Specific Ideas

- **"Switchable" is a deliberate theme**: the user chose *all three* translation modes and *all three* word-inspector modes — flexibility is a first-class requirement, not an either/or. Plan a display-mode control rather than a single hardcoded layout.
- **Layer distinction is layered too**: shared token set (baseline) + per-view styling + user themes — the user wants consistency *and* configurability, with the token set as the anti-drift mechanism for SC5.
- **Honesty about the RAG view**: the user explicitly wants the debug screen built now but rendering a *typed unavailable* state until Phase 8 — do not fake retrieval to satisfy SC4.
- **Parity by construction**: TUI via the shared services layer + a cross-surface golden gate — the user wants parity *proven*, not asserted.
- **Graph view scope**: TUI ships graph navigation + annotation-review screens (Phase 4 contracts are frozen; this phase renders them as views only).

</specifics>

<deferred>
## Deferred Ideas

- **Real multi-RAG retrieval, routing, reranking, and the populated RAG debug view** — Phase 8 (this phase scaffolds the screen against the typed envelope only).
- **Interactive graph *explorer* beyond navigation/annotation-review** — the deeper exploratory UX belongs with graph/retrieval work; this phase delivers a navigable view over frozen Phase-4 contracts.
- **Streaming answers / research-query interface, server dashboard, auth/RBAC** — Phases 10–11.
- **Hadith/tafsir/isnad UI surfaces** — Phases 6–7.
- **Comparative-scripture views** — Phase 9.
- **Additional frontend niceties** (offline PWA install, keyboard-driven power mode, export/print polish) — revisit after the core reading/research flows land.

</deferred>

---

*Phase: 5-Rich Quran Experience*
*Context gathered: 2026-10-06*
