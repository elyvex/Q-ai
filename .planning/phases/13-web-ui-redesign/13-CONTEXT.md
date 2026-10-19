# Phase 13: Web UI Redesign - Context

**Gathered:** 2026-10-19
**Status:** Ready for planning

<domain>
## Phase Boundary

Users get a from-scratch redesign of the Vite + React web UI in an elegant light white/gray style, with an animated background and polished buttons and controls. Every Q-ai feature from Phases 1–12 (read, search, inspect, graph, hadith/tafsir, isnad, multi-RAG, comparative scripture, agents, management) is reachable through the UI. The result is a cohesive, production-grade single-page application that feels like a dedicated desktop research tool rendered in the browser. The redesign adds no backend capability; the Rust backend has no diff beyond defect fixes found during the phase.

</domain>

<decisions>
## Implementation Decisions

### Visual Language & Design System
- **Palette:** White canvas (`#FFFFFF`), warm-gray/neutral gray scale (`#F5F5F4` … `#292524`), and one restrained accent hue used only for interactive state. Content surfaces carry zero gradients.
- **Controls:** 8–10px radius, 1px neutral border, subtle elevation. Hover lifts 1px, press scales to 0.98, transitions run ~150ms ease-out. Focus uses a visible 2px ring with offset. Every control provides disabled, loading, and error states.
- **Animated Background:** Slow, low-opacity CSS gradient field behind content. CSS-only, no canvas animation loop. Must respect `prefers-reduced-motion`, pause when tab is hidden, set `pointer-events: none`, and never reduce WCAG 2.2 AA text contrast.
- **Typography & Font Strategy:** Canonical Arabic in a large Naskh-class face (`Amiri`, `Scheherazade New`, `Noto Naskh Arabic`, `Geeza Pro`, serif) in near-black ink. Translations in neutral sans gray-700. Annotations use a tinted accent with a left rule. Bundled webfonts require OD-15 license ratification; until OD-15 is resolved, the system/local fallback stack guarantees license purity and satisfies `layer_separation.rs`.
- **Layer Integrity:** Canonical text, translations, and annotations remain strictly distinguishable in grayscale using weights, rules, badges, and icons without relying solely on color. `--layer-*` CSS variables are strictly maintained.
- **Theme Scope:** Ships an elegant light theme. Dark theme plumbing is supported via token variables, but the UI ships light-only for Phase 13.

### Feature Reachability
- A generated feature-to-route reachability matrix file covers every capability from Phases 1–12:
  - Quran reading, surah/juz/hizb navigation, continuous/mushaf mode.
  - Search (exact, normalized, phrase, concatenated, regex) with explainability and frequency sparklines.
  - Lexical research (family, lemma, root frequency, distribution, co-occurrence) with `research_checksum`.
  - Linguistics explorer (paradigm tables, word-family trees, attestations).
  - Graph explorer (interactive SVG/canvas node-edge canvas, neighbor expansion, shortest paths with edge provenance, export to JSON/DOT/SVG).
  - Hadith browser (collections, matn/isnad separation, grading badges) & search.
  - Tafsir comparative panel (multi-mufassir parallel columns, disagreement preserved).
  - Isnad narrator DAG visualization (uncertainty dashed lines, narrator detail, identity proposals approve/reject).
  - RAG dashboard (TOML config forms, status badges, hybrid retrieval scores, router checksum).
  - Agent research view (streamed progress, tool trace, claim-level citations with re-verification, cancel).
  - Comparative scripture (editions, parallel translations, computational suggestions with labels).
  - Management & ops (source catalog pipeline, audit trail table, job monitor, doctor health cards, config origins, system stats).
- Zero CLI- or API-only features remain.

### Architecture & Engineering
- **Client Foundation:** React 19 + TypeScript + Vite + Vitest + Testing Library in `web/`.
- **Routing:** Route scaffold with code splitting (`React.lazy` / `Suspense`) ensuring initial bundle stays ≤ 350 KB gzipped. Preserves all deep links (`/read/{slug}@{version}/{surah}:{ayah}`, `?citation=...`).
- **Command Palette:** Quick navigation via `Ctrl/Cmd+K` global command palette.
- **API Interaction:** Single typed API client (`web/src/api/client.ts`) calling existing `/api/v1/` Axum endpoints and `/api/v1/quran/tool/{name}`.
- **Security & Purity:** Strict CSP compliance, zero third-party network requests, no remote CDNs, no `dangerouslySetInnerHTML`.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `web/src/api/client.ts`: Typed API client covering `fetchAyahs`, `search`, `family`, `rootFrequency`, `lemmaFrequency`, `frequency`, `distribution`, `cooccurrence`, `graphNeighbors`, `graphPath`, `runTool`.
- `web/src/tokens/layers.css` and `themes.css`: Defined layer tokens (`--layer-canonical-*`, `--layer-translation-*`, `--layer-annotation-*`).
- `crates/server/src/api.rs`: Complete Axum API covering Quran, Hadith, Tafsir, Graph, Scripture, Search, Lexicon, and dynamic tool dispatch.
- `crates/server/src/webassets.rs`: Embedded asset serving with fallback to `index.html`.

### Invariants & Guardrails
- `crates/application/tests/layer_separation.rs`: Source-scan asserting canonical props purity, no canonical class on translations, no HTML injection, no remote URLs, no bundled fonts with `url(...)` without license approval.
- `crates/server/tests/api.rs`: `spa_fallback_serves_index_and_preserves_api` asserts SPA client routes serve `index.html` while preserving API 404s.

</code_context>

<specifics>
## Specific Ideas
- Tracer slice in Plan 13-01 proves design tokens, Reachability Matrix generator, and the redesigned Reader view end-to-end before expanding to the remaining views.
- Modular component library built with testing coverage before assembling complex domain views.
- Cross-surface parity tests validating that data returned in the web views matches CLI/API outputs byte-for-byte with identical `research_checksum`.

</specifics>

<deferred>
## Deferred Ideas
- Dark mode toggle in production UI (plumbing preserved in CSS variables; UI ships light-only).
- UI chrome localization beyond English (Arabic/Persian UI labels deferred to post-Phase 13; Arabic scripture rendering is fully supported with RTL).
- Any new backend endpoints beyond bug fixes.
- Native mobile packaging (Electron/Tauri/Capacitor).

</deferred>
