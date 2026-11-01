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
- **Palette (Concordance):** Porcelain ground `--background: oklch(0.985 0.0025 265)` in light mode, ink-and-chalk `oklch(0.158 0.008 265)` in dark mode; one deep indigo accent `--primary: oklch(0.478 0.199 266)` (lifted to `oklch(0.705 0.152 266)` in dark) used only for interactive states. Provenance colours are semantic and never decorative: canonical `--canon oklch(0.49 0.083 192)`, translation `--trans oklch(0.51 0.108 312)`, annotation `--annot oklch(0.575 0.108 68)`, computed `--comp oklch(0.525 0.13 355)`. Content surfaces carry zero gradients. (Supersedes the earlier white-canvas/teal palette; the implemented design is the Concordance contract.)
- **Theme Modes:** Ship both Light and Dark themes immediately via `next-themes` (`class` strategy, `defaultTheme: system`, persisted in `localStorage`). Theme toggle lives in the top rail. Dark is designed separately — deeper card surfaces, lifted indigo, 10 % white borders, no shadow tricks.
- **Layout & Density:** Horizontal top rail (not a sidebar) over a single 86 rem measure with a pinned colophon; spacious editorial layout with generous margins and typography-first readability. Secondary controls and metadata live in a 16 rem margin column that follows content on mobile. No slide-over drawers in the shipped design; controls are in-page.
- **Controls:** 5–8 px radius, 1 px neutral border, subtle elevation only on control clusters. Hover raises to `--muted`; press uses the accent wash; transitions run ~120 ms. Focus uses a visible 2 px `--ring` outline at 2 px offset. Every control provides disabled, loading, and error states.
- **Animated Background:** None. The earlier pure-CSS gradient field was dropped — the interface is instant and motionless except a 120 ms colour/border transition and a single pulse for an unresolved status dot. `prefers-reduced-motion` removes transitions globally.
- **Typography & Font Strategy:** Self-hosted WOFF2 via `@fontsource`: Inter (UI), Newsreader Variable (titles/wordmark only), JetBrains Mono (references/counts), Amiri + Noto Naskh Arabic (canonical Arabic, RTL, `lang="ar"`). No remote CDNs. Canonical Arabic renders at 1.75 → 2.25 rem with 2.15–2.2 line height; Uthmani marks untouched.
- **Iconography:** `lucide-react` across navigation, toolbars, and status badges.
- **Layer Integrity:** Canonical text, translations, and annotations remain strictly distinguishable in grayscale using rules, labels, dots, and weights — not colour alone. `--layer-*` CSS variables in `web/src/tokens/layers.css` are strictly maintained; layer backgrounds default to `transparent`.

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
