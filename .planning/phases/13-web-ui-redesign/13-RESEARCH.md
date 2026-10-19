# Phase 13: Web UI Redesign - Research

**Researched:** 2026-10-19
**Domain:** From-scratch redesign of Vite + React Single Page Application (SPA), Design System, and Full Feature Reachability
**Confidence:** HIGH (repository verified against active code and architecture)

## Summary

Phase 13 delivers an elegant, production-grade redesign of the Q-ai web user interface in a light white/gray aesthetic with an animated CSS background, refined interactive controls, and 100% reachability for all Phase 1–12 capabilities. The SPA renders in browser but functions with the responsiveness, density, and depth of a professional research workstation.

A critical requirement of Phase 13 is that **the redesign adds no backend capability**. All domain functionality is already implemented across Phases 1–12 and exposed via Axum HTTP endpoints in `crates/server/src/api.rs` and generic tool dispatch `/api/v1/quran/tool/{name}`. The web client connects to these existing endpoints through an expanded typed client (`web/src/api/client.ts`).

### Repository Verification Findings

1. **Existing Web SPA**:
   - Located at `web/`, configured with React 19 (`19.3.0`), TypeScript (`~6.0.2`), Vite (`8.3.3`), Vitest (`5.0.3`), `@testing-library/react` (`16.3.3`), and Oxlint (`1.81.0`).
   - Current build produces clean bundles (~231 KB JS / 72 KB gzipped) in ~226ms.
   - Current test suite passes all 18 unit tests across views and components.
2. **Layer Tokens & Purity**:
   - `web/src/tokens/layers.css` provides `--layer-canonical-*`, `--layer-translation-*`, `--layer-annotation-*`.
   - `crates/application/tests/layer_separation.rs` actively enforces layer isolation, prohibiting HTML injection (`dangerouslySetInnerHTML`), external URLs (no CDNs, only localhost), and unregistered bundled webfonts.
3. **Backend API Coverage**:
   - `crates/server/src/api.rs` hosts endpoints for read, search (5 modes), counting & distributions, graph operations (neighbors, path, subgraph, pattern, root-family), hadith & tafsir, verse links, and scripture.
   - Any tool not exposed as an explicit REST route is invokable via `POST /api/v1/quran/tool/{name}`, which dispatches via `application::quran_tools::dispatch_registered_tool`.
4. **Embedded SPA Serving**:
   - `crates/server/src/webassets.rs` serves `web/dist` via `rust-embed` with `spa_fallback` redirecting client routes to `index.html`. Integration test `spa_fallback_serves_index_and_preserves_api` in `crates/server/tests/api.rs` validates this in CI.

---

## Conflict & Uncertainty Resolutions

| ID | Issue | Analysis & Resolution |
|---|---|---|
| **C-1** | Animated Background | **Resolution:** Pure CSS-animated subtle gradient field (`pointer-events: none; z-index: -1`). Avoids canvas RAF loop overhead, automatically suspends/pauses when not visible or when `prefers-reduced-motion` is active, and guarantees no interference with WCAG AA contrast. |
| **C-2** | Button Variants & Scale | **Resolution:** 5 variants (`primary`, `secondary`, `outline`, `ghost`, `danger`) across 4 sizes (`sm`, `md`, `lg`, `xl`) and 6 interaction states (`default`, `hover`, `active`/`pressed`, `focus`, `disabled`, `loading`). |
| **C-3** | Dark Mode Scope | **Resolution:** Design system tokens support theming via CSS variables (`--layer-*`, `--surface-*`), but the UI ships an elegant light theme only. Dark mode toggle is deferred (R2 owner decision). |
| **C-4** | Plan Structure | **Resolution:** 9 plans across 7 waves as framed in the consolidated ROADMAP.md, incorporating all R1 visual specifications and R2 architectural/owner gates. |
| **C-7** | Font Strategy & OD-15 | **Resolution:** Naskh-class canonical Arabic font stack (`'Amiri', 'Scheherazade New', 'Noto Naskh Arabic', 'Geeza Pro', serif`) with system fallbacks. Self-hosted font files are integrated into the repo under open licenses (OFL) and recorded in the font license matrix to satisfy OD-15 and pass `layer_separation.rs`. |
| **C-9** | Backend Scope | **Resolution:** Zero new backend features. The Rust backend has no diff beyond defect fixes. |
| **C-10** | Reachability Scope | **Resolution:** Union of all Phase 1–12 features into an automated feature-to-route reachability matrix test (`web/src/tests/reachability.test.ts`) that asserts every single feature has a corresponding UI route and interactive view. |

---

## Architecture & Technology Choices

### Component Hierarchy
```text
web/src/
├── api/                  # Typed client and envelope handling
│   ├── client.ts         # Core /api/v1 endpoints
│   ├── tools.ts          # Generic tool runner and registry mapping
│   └── types.ts          # Domain entity interfaces
├── tokens/               # Design token definitions
│   ├── colors.css        # Canvas, neutrals, accent, status tokens
│   ├── typography.css    # Arabic Naskh, Latin sans, mono, scales
│   ├── elevation.css     # Shadows, borders, radii
│   ├── motion.css        # Durations, easings, transitions
│   └── layers.css        # Preserved layer tokens for canonical/trans/annot
├── components/           # Reusable UI Primitives
│   ├── ui/               # Button, Input, Select, Badge, Card, Modal, etc.
│   ├── layout/           # AppShell, Sidebar, Header, CommandPalette
│   ├── display/          # AyahCard, LayerBadge, WordInspector, CitationCopy
│   └── background/       # AnimatedBackground
├── views/                # Route Views
│   ├── reader/           # Mushaf reading, surah/juz nav, translations
│   ├── search/           # Search view with 5 modes & sparklines
│   ├── linguistics/      # Root/lemma explorer, paradigm, word family
│   ├── graph/            # Node-edge graph canvas, neighbors, paths
│   ├── hadith/           # Hadith browser, isnad DAG, rijal detail
│   ├── tafsir/           # Comparative multi-mufassir view
│   ├── scripture/        # Comparative scripture reader & parallels
│   ├── rag/              # RAG dashboard, project list, honest debug
│   ├── agents/           # Agent research streaming, tool lifecycle
│   └── management/       # Source catalog, jobs, doctor, audit trail
└── router.tsx            # Route configuration and lazy loaders
```

### Performance & Bundle Targets
- **Initial Bundle Target:** $\le 350\text{ KB}$ gzipped via route-based code splitting (`React.lazy`).
- **Largest Contentful Paint (LCP):** $\le 1.5\text{ s}$.
- **Cumulative Layout Shift (CLS):** $< 0.1$.
- **Lighthouse CI Targets:** Accessibility $\ge 90$, Performance $\ge 80$.
- **Zero Third-Party Calls:** No remote CDNs, telemetry, or external fonts.

---

## 7-Wave Execution Structure

- **Wave 1 (Foundation):**
  - `13-01`: Design Tokens, Typography, Reachability Generator & Component Library (with Reader TRACER).
  - `13-02`: App Shell, Sidebar, Routing, Command Palette & Animated Background.
- **Wave 2 (Quran Core):**
  - `13-03`: Quran Reading & Multi-Mode Search Views.
- **Wave 3 (Linguistics & Graph):**
  - `13-04`: Linguistics Explorer & Morphology Dashboard.
  - `13-05`: Graph Explorer, Neighbor Navigation & Shortest Path Canvas.
- **Wave 4 (Hadith & Tafsir):**
  - `13-06`: Hadith Browser, Tafsir Comparison & Isnad Chain DAG.
- **Wave 5 (RAG & Comparative):**
  - `13-07`: Multi-RAG Dashboard, Agent Research & Comparative Scripture.
- **Wave 6 (Operations):**
  - `13-08`: Management Dashboard, Source Catalog, Jobs, Audit & Doctor.
- **Wave 7 (Hardening & Gates):**
  - `13-09`: Motion Polish, Accessibility Audit, Bundle Budget & Parity Gates.
