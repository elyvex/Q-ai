# Phase 13: Web UI Redesign - Context

**Gathered:** 2026-10-19
**Discussed:** 2026-10-19 (Adaptive Questioning Complete)
**Status:** Ready for planning / execution

<domain>
## Phase Boundary

Users get a from-scratch redesign of the Vite + React web UI in an elegant white/gray style, with a pure CSS animated background, polished controls, and dual Light/Dark theme support. Every Q-ai feature from Phases 1–12 (read, search, inspect, graph, hadith/tafsir, isnad, multi-RAG, comparative scripture, agents, management) is reachable through the UI. The presentation follows a spacious editorial layout with generous margins and typography-first readability, while providing slide-over drawers and side-by-side synchronized comparison columns for deep research. The redesign adds no backend capability; the Rust backend has no diff beyond defect fixes found during the phase.

</domain>

<decisions>
## Implementation Decisions

### Visual Language & Design System
- **Palette:** White canvas (`#FFFFFF`) in light mode, dark obsidian canvas (`#0B1020` / `#111827`) in dark mode; warm-gray / slate neutral scales (`#F5F5F4` … `#292524`), and one restrained teal accent hue (`#0F766E` / `#2DD4BF`) used only for interactive states. Content surfaces carry zero gradients.
- **Theme Modes (Discussed):** Ship both Light and Dark themes immediately. Theme toggle in top header wired to document root `data-theme` and existing tokens in `themes.css`.
- **Layout & Density (Discussed):** Spacious editorial layout with generous margins and typography-centric design. Secondary tools, inspections, and metadata slide out in refined drawers, keeping reading surfaces clean and distraction-free.
- **Controls:** 8–10px radius, 1px neutral border, subtle elevation. Hover lifts 1px, press scales to 0.98, transitions run ~150ms ease-out. Focus uses a visible 2px ring with offset. Every control provides disabled, loading, and error states.
- **Animated Background (Discussed):** Pure CSS subtle gradient field behind content (`pointer-events: none; z-index: -1`). Strictly CSS-only with no canvas animation loop, zero idle CPU overhead, automatically pauses when the browser tab is hidden, and completely disables motion when `prefers-reduced-motion` is active.
- **Typography & Font Strategy (Discussed):** Self-hosted open-source WOFF2 font files bundled locally in `web/public/fonts/` (`Amiri` and `Noto Naskh Arabic` under Open Font License). Recorded in `docs/05-followups/decisions-needed.md` under OD-15 with exact license ratification commands. Canonical Arabic text rendered in large Naskh-class face in near-black (light) / high-contrast off-white (dark). Translations in neutral sans gray-700 / gray-300.
- **Iconography (Discussed):** Use `lucide-react` icon library for crisp, comprehensive icons across navigation, toolbars, and status badges.
- **Layer Integrity:** Canonical text, translations, and annotations remain strictly distinguishable in grayscale using weights, rules, badges, and icons without relying solely on color. `--layer-*` CSS variables are strictly maintained.

### Reading & Research Surfaces
- **Multi-Translation & Exegesis Layout (Discussed):** Side-by-side synchronized parallel columns for comparing up to 3 translations or tafsir commentaries simultaneously, with synchronized verse locking.
- **Feature Reachability:** A generated feature-to-route reachability matrix test (`web/src/tests/reachability.test.ts`) covers every capability from Phases 1–12:
  - Quran reading, surah/juz/hizb navigation, continuous/mushaf mode.
  - Search (exact, normalized, phrase, concatenated, regex) with explainability and frequency sparklines.
  - Lexical research (family, lemma, root frequency, distribution, co-occurrence) with `research_checksum`.
  - Linguistics explorer (paradigm tables, word-family trees, attestations).
  - Graph explorer (interactive SVG/canvas node-edge canvas, neighbor expansion, shortest paths with edge provenance, export to JSON/DOT/SVG).
  - Hadith browser (collections, matn/isnad separation, grading badges) & search.
  - Tafsir comparative panel (multi-mufassir parallel columns with disagreement preserved).
  - Isnad narrator DAG visualization (uncertainty dashed lines, narrator detail, identity proposals approve/reject).
  - RAG dashboard (TOML config forms, status badges, hybrid retrieval scores, router checksum).
  - Agent research view (streamed progress, tool trace, claim-level citations with re-verification, cancel).
  - Comparative scripture (editions, parallel translations, computational suggestions with labels).
  - Management & ops (source catalog pipeline, audit trail table, job monitor, doctor health cards, config origins, system stats).
- Zero CLI- or API-only features remain.

### Architecture & Engineering
- **Client Foundation:** React 19 + TypeScript + Vite + Vitest + Testing Library in `web/`.
- **Routing:** Route scaffold with code splitting (`React.lazy` / `Suspense`) ensuring initial bundle stays $\le 350\text{ KB}$ gzipped. Preserves all deep links (`/read/{slug}@{version}/{surah}:{ayah}`, `?citation=...`).
- **Command Palette:** Quick navigation via `Ctrl/Cmd+K` global command palette.
- **API Interaction:** Single typed API client (`web/src/api/client.ts`) calling existing `/api/v1/` Axum endpoints and `/api/v1/quran/tool/{name}`.
- **Security & Purity:** Strict CSP compliance, zero third-party network requests, no remote CDNs, no `dangerouslySetInnerHTML`.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `web/src/api/client.ts`: Typed API client covering `fetchAyahs`, `search`, `family`, `rootFrequency`, `lemmaFrequency`, `frequency`, `distribution`, `cooccurrence`, `graphNeighbors`, `graphPath`, `runTool`.
- `web/src/tokens/layers.css` and `themes.css`: Defined layer tokens (`--layer-canonical-*`, `--layer-translation-*`, `--layer-annotation-*`) and theme overrides (`[data-theme='dark']`).
- `crates/server/src/api.rs`: Complete Axum API covering Quran, Hadith, Tafsir, Graph, Scripture, Search, Lexicon, and dynamic tool dispatch.
- `crates/server/src/webassets.rs`: Embedded asset serving with fallback to `index.html`.

### Invariants & Guardrails
- `crates/application/tests/layer_separation.rs`: Source-scan asserting canonical props purity, no canonical class on translations, no HTML injection, no remote URLs, no bundled fonts without license approval.
- `crates/server/tests/api.rs`: `spa_fallback_serves_index_and_preserves_api` asserts SPA client routes serve `index.html` while preserving API 404s.

</code_context>

<specifics>
## Specific Ideas
- Tracer slice in Plan 13-01 proves design tokens, Reachability Matrix generator, self-hosted WOFF2 fonts, and the redesigned Reader view end-to-end before expanding to the remaining views.
- Modular component library built with testing coverage before assembling complex domain views.
- Cross-surface parity tests validating that data returned in the web views matches CLI/API outputs byte-for-byte with identical `research_checksum`.

</specifics>

<deferred>
## Deferred Ideas
- UI chrome localization beyond English (Arabic/Persian UI labels deferred to post-Phase 13; Arabic scripture rendering is fully supported with RTL).
- Any new backend endpoints beyond bug fixes.
- Native mobile packaging (Electron/Tauri/Capacitor).

</deferred>
