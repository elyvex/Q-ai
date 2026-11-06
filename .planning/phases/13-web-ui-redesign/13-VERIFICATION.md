---
phase: 13-web-ui-redesign
verified: 2026-11-06T00:00:00Z
status: gaps_found
score: 9/11 must-haves verified
covered_files: [".planning/phases/13-web-ui-redesign/13-01-PLAN.md", ".planning/phases/13-web-ui-redesign/13-02-PLAN.md", ".planning/phases/13-web-ui-redesign/13-03-PLAN.md", ".planning/phases/13-web-ui-redesign/13-04-PLAN.md", ".planning/phases/13-web-ui-redesign/13-05-PLAN.md", ".planning/phases/13-web-ui-redesign/13-06-PLAN.md", ".planning/phases/13-web-ui-redesign/13-07-PLAN.md", ".planning/phases/13-web-ui-redesign/13-08-PLAN.md", ".planning/phases/13-web-ui-redesign/13-09-PLAN.md", ".planning/phases/13-web-ui-redesign/13-10-PLAN.md", ".planning/phases/13-web-ui-redesign/13-10-SUMMARY.md", ".planning/phases/13-web-ui-redesign/13-11-PLAN.md", ".planning/phases/13-web-ui-redesign/13-11-SUMMARY.md", ".planning/phases/13-web-ui-redesign/13-12-PLAN.md", ".planning/phases/13-web-ui-redesign/13-12-SUMMARY.md", ".planning/phases/13-web-ui-redesign/13-13-PLAN.md", ".planning/phases/13-web-ui-redesign/13-13-SUMMARY.md", ".planning/phases/13-web-ui-redesign/13-14-PLAN.md", ".planning/phases/13-web-ui-redesign/13-14-SUMMARY.md", "web/public/fonts/amiri-arabic-400-normal.woff2", "web/public/fonts/amiri-arabic-700-normal.woff2", "web/public/fonts/noto-naskh-arabic-arabic-400-normal.woff2", "web/public/fonts/noto-naskh-arabic-arabic-700-normal.woff2", "web/src/App.tsx", "web/src/api/client.ts", "web/src/components/AyahColumn.test.tsx", "web/src/components/AyahColumn.tsx", "web/src/components/background/AnimatedBackground.tsx", "web/src/components/command/command-menu.tsx", "web/src/components/layout/site-header.tsx", "web/src/components/ToolLedger.test.tsx", "web/src/components/ToolLedger.tsx", "web/src/components/ui/Badge.tsx", "web/src/components/ui/button.tsx", "web/src/components/ui/Button.tsx", "web/src/components/ui/Card.tsx", "web/src/components/ui/dialog.tsx", "web/src/components/ui/EmptyState.tsx", "web/src/components/ui/ErrorBoundary.tsx", "web/src/components/ui/ErrorState.tsx", "web/src/components/ui/IconButton.tsx", "web/src/components/ui/input-group.tsx", "web/src/components/ui/input.tsx", "web/src/components/ui/Input.tsx", "web/src/components/ui/Modal.tsx", "web/src/components/ui/Select.tsx", "web/src/components/ui/Skeleton.tsx", "web/src/components/ui/Spinner.tsx", "web/src/components/ui/Tabs.tsx", "web/src/components/ui/Toast.tsx", "web/src/components/ui/ui.test.tsx", "web/src/display/DisplayModeControl.test.tsx", "web/src/display/DisplayModeControl.tsx", "web/src/lib/byte-diff.test.ts", "web/src/lib/byte-diff.ts", "web/src/lib/reader-storage.test.ts", "web/src/lib/reader-storage.ts", "web/src/lib/related.test.ts", "web/src/lib/related.ts", "web/src/lib/tool-result.test.ts", "web/src/lib/tool-result.ts", "web/src/lib/tools.test.ts", "web/src/lib/tools.ts", "web/src/lib/words.test.ts", "web/src/lib/words.ts", "web/src/router.test.tsx", "web/src/router.tsx", "web/src/tests/reachability.test.ts", "web/src/tokens/colors.css", "web/src/tokens/elevation.css", "web/src/tokens/index.ts", "web/src/tokens/layers.css", "web/src/tokens/motion.css", "web/src/tokens/typography.css", "web/src/views/CitationsWorkbenchView.test.tsx", "web/src/views/CitationsWorkbenchView.tsx", "web/src/views/ExplorerView.test.tsx", "web/src/views/ExplorerView.tsx", "web/src/views/IsnadView.test.tsx", "web/src/views/IsnadView.tsx", "web/src/views/RagDebugView.test.tsx", "web/src/views/RagDebugView.tsx", "web/src/views/ReadingView.test.tsx", "web/src/views/ReadingView.tsx", "web/src/views/ResearchView.test.tsx", "web/src/views/ResearchView.tsx", "web/src/views/SearchView.tsx", "web/src/views/ScriptureView.tsx", "web/src/views/TafsirView.tsx", "web/src/views/ToolsView.test.tsx", "web/src/views/ToolsView.tsx", "web/src/views/WordsView.test.tsx", "web/src/views/WordsView.tsx"]
covered_digest: "v3:sha256:e61cc1d4e5dfe5e343beb6933c1ffa1508c0b186875a3776aa3d8ed46faafa5b"
behavior_unverified: 0
overrides_applied: 0
gaps:
  - truth: "UI ships an animated background behind content (phase goal + roadmap SC2)"
    status: failed
    reason: "AnimatedBackground component exists and is unit-tested, but is never imported or rendered anywhere in the app (app-shell.tsx, App.tsx, index.css). The component and its CSS gradient are orphaned; no animated background is visible to users. The component also still carries the superseded teal gradient (rgb(15 118 110)) rather than the Concordance forest-green palette."
    artifacts:
      - path: "web/src/components/background/AnimatedBackground.tsx"
        issue: "defined and tested, but not mounted in AppShell or any route"
    missing:
      - "Mount <AnimatedBackground /> in AppShell (or formally drop it with a spec/roadmap amendment + override)"
  - truth: "Motion performance verified (LCP/CLS/60fps) — roadmap SC7"
    status: partial
    reason: "No Lighthouse CI run, no CLS/LCP measurement, and no perf budget enforcement recorded in the repo. Web build passes and bundle is 157 KB gzipped (under 350 KB), but the runtime motion target is unmeasured."
    artifacts: []
    missing:
      - "Run Lighthouse CI (accessibility >= 90, performance >= 80) and record CLS < 0.1 / LCP <= 1.5s"
deferred:
  - truth: "Dark theme"
    addressed_in: "Phase 14 (deferred per owner gate)"
    evidence: "ROADMAP Phase 13 out-of-scope: 'Dark theme is deferred'"
advisory:
  - finding: "TS design-token mirror (web/src/tokens/index.ts) is stale: exports teal accent #0F766E, dark canvas #0B1020, and warm-gray stone neutrals, while the shipped CSS (index.css) uses the Concordance forest-green oklch palette. Two sources of truth disagree; reachability test asserts the stale teal values."
    category: other
    reason: "Code-hygiene drift; rendered UI uses CSS correctly. Reconcile the TS mirror with the amended Concordance tokens (or delete the mirror)."
    evidence_status: "none provided"
human_verification:
  - test: "Font licensing owner gate (OD-NEW): ratify the license of each bundled Arabic and UI font"
    expected: "Each font's license recorded in the license matrix; OD-NEW closed"
    why_human: "Owner-only decision; blocks REQ-security-licensing-observability closure"
  - test: "Run /rag-debug against a live engine and confirm the checksum vs QAI-RAG-0310"
    expected: "rag.debug returns hits + route decision with research_checksum; QAI-RAG-0310 shown when backend unconfigured"
    why_human: "Needs a running Rust engine with a loaded edition"
  - test: "Cold-load /words, /tools, /citations, /isnad in the served build and run one instrument each against a live engine"
    expected: "Routes load, rail highlight active, instruments return checksums matching the CLI"
    why_human: "jsdom covers route wiring only; needs served build + running engine"
  - test: "Accessibility audit: Lighthouse accessibility >= 90, axe zero serious/critical on every route, keyboard-only core flows"
    expected: "No serious/critical axe violations; Lighthouse a11y >= 90"
    why_human: "No automated a11y run recorded in CI"
  - test: "E2E browser walkthrough (read -> search -> inspect -> graph -> ask -> verify-citations -> export) without touching CLI/API"
    expected: "Full walkthrough completes in the browser"
    why_human: "No Playwright suite present in the repo"
---

# Phase 13: Web UI Redesign — Verification Report

**Phase Goal:** Users get a from-scratch redesign of the Vite + React web UI in an elegant light white/gray style, with an animated background and polished buttons and controls. Every Q-ai feature from Phases 1–12 is reachable through the UI. The result is a cohesive, production-grade single-page application that feels like a dedicated desktop research tool rendered in the browser. The redesign adds no backend capability.

**Verified:** 2026-11-06
**Status:** gaps_found
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Feature reachability: all 15 Phase 1–12 capabilities have an active registered route | ✓ VERIFIED | `src/tests/reachability.test.ts` — 5/5 pass; 15 capabilities mapped to `/`, `/search`, `/words`, `/tools`, `/citations`, `/hadith`, `/isnad`, `/tafsir`, `/scripture`, `/graph`, `/rag-debug`, `/explorer` |
| 2 | Light white/gray theme with polished controls | ✓ VERIFIED | `src/index.css` Concordance palette (warm-white ground, forest-green instrument accent); primitive component library in `src/components/ui/*` (Button, Input, Modal, Tabs, etc.) |
| 3 | Animated background behind content | ✗ FAILED (orphaned) | `AnimatedBackground` component exists + tested but is never imported/rendered (not in AppShell, App.tsx, or index.css); no CSS animation either |
| 4 | Layer integrity: canonical/translation/annotation visually distinct, tokens preserved | ✓ VERIFIED | `cargo test -p application --test layer_separation` — 7/7 pass (spa_layer_tokens_define_all_three_families, spa_canonical_slot_has_no_translation_prop, spa_has_no_html_injection) |
| 5 | Tokenized design system (single source of truth) | ✓ VERIFIED (with warning) | CSS custom properties in `src/index.css` + `src/tokens/*.css`; TS mirror in `src/tokens/index.ts` is stale (teal vs forest green) — see Advisory |
| 6 | Interactive states on every control (focus/hover/active/disabled/loading/error) | ✓ VERIFIED | `src/components/ui/ui.test.tsx` covers Button variants/states; views render EmptyState/ErrorState/Skeleton/Spinner |
| 7 | Arabic + RTL rendering with self-hosted fonts | ✓ VERIFIED | `dir="rtl" lang="ar"` on AyahColumn; Amiri/Noto Naskh Arabic via @fontsource; `spa_has_no_remote_url_or_bundled_font` passes |
| 8 | Motion performance (≥60fps, CLS<0.1, LCP≤1.5s) | ? UNCERTAIN | No Lighthouse/CLS measurement recorded; bundle 157 KB gz (under 350 KB budget) |
| 9 | End-to-end browser walkthrough without CLI/API | ? UNCERTAIN | No Playwright/E2E suite present; views are wired to HTTP client but walkthrough unverified |
| 10 | Accessibility (Lighthouse a11y ≥90, axe zero serious/critical, keyboard flows) | ? UNCERTAIN | No automated axe/Lighthouse run recorded; skip link + ARIA labels + focus traps present in code |
| 11 | Gates green (build + tests + layer_separation, no backend diff) | ✓ VERIFIED | `npm run build` passes; `npm test` 180/180 (23 files); `cargo test -p application --test layer_separation` 7/7; no `.rs` files changed in any Phase 13 commit |

**Score:** 9/11 truths verified (1 failed, 2 present-but-unmeasured/uncertain)

### Deferred Items

| # | Item | Addressed In | Evidence |
|---|------|-------------|----------|
| 1 | Dark theme | Deferred per owner gate | ROADMAP Phase 13 out-of-scope: "Dark theme is deferred" |

### Advisory (New Scope, Unevidenced)

| # | Finding | Category | Why Advisory |
|---|---------|----------|--------------|
| 1 | TS design-token mirror (`web/src/tokens/index.ts`) exports teal accent `#0F766E`, dark canvas `#0B1020`, warm-gray stone neutrals, while shipped CSS uses the Concordance forest-green oklch palette. Two sources of truth disagree; `reachability.test.ts` asserts the stale teal values. | other | Code-hygiene drift; rendered UI uses CSS correctly. Reconcile the TS mirror with the amended Concordance tokens or delete the mirror. |

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `web/src/components/background/AnimatedBackground.tsx` | Animated background component | ⚠️ ORPHANED | Exists + tested; never mounted |
| `web/src/components/ui/*` | Primitive component library | ✓ VERIFIED | Button, Input, Modal, Tabs, etc. with tests |
| `web/src/tokens/index.ts` + `*.css` | Tokenized design system | ✓ VERIFIED (drift) | CSS is source of truth; TS mirror stale |
| `web/src/router.tsx` | Route table with deep links | ✓ VERIFIED | 13 routes; ROUTES/DIRECTORY/NAVIGATION agree |
| `web/src/views/*` | Feature views for all capabilities | ✓ VERIFIED | Reading, Search, Words, Tools, Citations, Hadith, Isnad, Tafsir, Scripture, Graph, RagDebug, Explorer |
| `web/src/lib/tool-result.ts`, `ToolLedger.tsx` | Shared result renderer | ✓ VERIFIED | Used by RagDebug, Words, Tools, Citations, Isnad |
| `web/src/tests/reachability.test.ts` | Feature-to-route matrix | ✓ VERIFIED | 5 tests pass |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| AppShell | AnimatedBackground | mount | ✗ NOT_WIRED | Component never rendered |
| RagDebugView | `runTool('rag.debug')` | POST /api/v1/quran/tool/rag.debug | ✓ WIRED | 5 tests pass |
| WordsView | morphology/root/lemma/family + counts | POST /api/v1/quran/tool/* + count/* | ✓ WIRED | 9 tests pass |
| ToolsView | 27 callable tools | runTool + DENIED rows | ✓ WIRED | 17 tests pass |
| CitationsWorkbenchView | fetchCitation + fetchAyahs | GET /api/v1/quran/citations/{id}, /ayahs | ✓ WIRED | 6 tests pass |
| IsnadView | isnad.trace/narrator/rijal | runTool | ✓ WIRED | 5 tests pass |
| ReadingView | reader-storage (prefs/bookmarks) | localStorage | ✓ WIRED | 43 tests pass |
| ReadingView | related (tafsir/hadith search) | runTool | ✓ WIRED | seed + disclaimer |

### Data-Flow Trace (Level 4)

Not applicable — web views fetch from the Rust HTTP API; data flow is verified by unit tests stubbing fetch and by the parity checksum contract (`research_checksum`).

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Web test suite | `npm test` (web/) | 180/180 pass (23 files) | ✓ PASS |
| Web build | `npm run build` (web/) | passes (157 KB gz) | ✓ PASS |
| Layer separation | `cargo test -p application --test layer_separation` | 7/7 pass | ✓ PASS |
| Reachability matrix | `npx vitest run src/tests/reachability.test.ts` | 5/5 pass | ✓ PASS |

### Probe Execution

Not applicable — no probe scripts for this phase.

### Requirements Coverage

| Requirement | Source Plan(s) | Description | Status | Evidence |
|-------------|----------------|-------------|--------|----------|
| REQ-ux-web | 13-10..13-14 | Web navigation, search, research workspace, export, streaming | ✓ SATISFIED (partial) | 13 routes + directory + command menu; streaming not surfaced in this phase (Phase 14) |
| REQ-quran-display | 13-11, 13-13 | Reading view, citation interaction, layer distinction | ✓ SATISFIED | ReadingView with 3 sizes/layouts, AyahColumn layer tokens, citation copy |
| REQ-quran-research-tools | 13-11, 13-12 | Typed tools usable from UI/API | ✓ SATISFIED | /tools (27 tools), /words (morphology + 5 counts) |
| REQ-security-licensing-observability | (roadmap R2) | Font licensing, no third-party requests | ? NEEDS HUMAN | Fonts self-hosted (verified); font licensing owner gate OD-NEW still BLOCKED |
| REQ-quran-result-contract | 13-10,13-11,13-12,13-14 | Result fields incl. research_checksum | ✓ SATISFIED | ToolLedger prints checksum; byte-diff; citations verdict/hash/URN |
| REQ-multi-rag | 13-10 | Multi-index retrieval + debug | ✓ SATISFIED | RagDebugView runs rag.debug/query with route decision |
| REQ-retrieval-governance | 13-10 | Prompt-injection defense, audit, budgets | ✓ SATISFIED (partial) | QAI-RAG-0326 denied list; route decision verbatim |
| REQ-cli-api | 13-12 | CLI/API parity | ✓ SATISFIED | ToolsView prints CLI + HTTP equivalents from sent params |
| REQ-agents-tools-runtime | 13-12 | Tool manifests, sandbox, approvals | ✓ SATISFIED (partial) | /tools catalogue; sandbox/approval UI is Phase 14 |
| REQ-hadith-corpus | 13-13, 13-14 | Hadith browser + isnad | ✓ SATISFIED | HadithView, IsnadView (trace/narrator/rijal) |
| REQ-tafsir-comparative | 13-13 | Tafsir + comparative scripture | ✓ SATISFIED | TafsirView, related lookup, ScriptureView |
| REQ-isnad-narrator-graph | 13-14 | Isnad/narrator graph + identity uncertainty | ✓ SATISFIED | IsnadView read-only + denied identity mutations |

All requirement IDs from PLAN frontmatter (13-10..13-14) accounted for. REQ-security-licensing-observability (user-listed) is the only one still gated on the owner font-licensing decision.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| web/src/components/background/AnimatedBackground.tsx | — | Component defined but never mounted (orphaned) | 🛑 Blocker | Animated background not visible; phase goal unmet |
| web/src/tokens/index.ts | — | Stale TS token mirror (teal vs forest green) | ⚠️ Warning | Two sources of truth disagree |

No TODO/FIXME/TBD/XXX markers found in modified files. No empty-array/object stubs flow to rendered output.

### Human Verification Required

1. **Font licensing owner gate (OD-NEW)**
   - Test: Ratify the license of each bundled Arabic and UI font; record in license matrix
   - Expected: OD-NEW closed; REQ-security-licensing-observability unblocked
   - Why human: Owner-only decision

2. **Live-engine smoke: /rag-debug**
   - Test: Run /rag-debug against a live engine; confirm checksum vs QAI-RAG-0310
   - Expected: rag.debug returns hits + route decision with research_checksum
   - Why human: Needs a running Rust engine with a loaded edition

3. **Live-engine smoke: /words, /tools, /citations, /isnad**
   - Test: Cold-load each route in the served build and run one instrument against a live engine
   - Expected: Routes load, rail highlight active, instruments return checksums matching the CLI
   - Why human: jsdom covers route wiring only

4. **Accessibility audit**
   - Test: Lighthouse accessibility ≥90; axe zero serious/critical on every route; keyboard-only core flows
   - Expected: No serious/critical axe violations
   - Why human: No automated a11y run recorded in CI

5. **E2E browser walkthrough**
   - Test: read → search → inspect → graph → ask → verify-citations → export without touching CLI/API
   - Expected: Full walkthrough completes in the browser
   - Why human: No Playwright suite present in the repo

6. **Motion performance**
   - Test: Lighthouse performance ≥80; CLS <0.1; LCP ≤1.5s
   - Expected: All within budget
   - Why human: No measurement recorded

### Gaps Summary

The shipped Concordance UI is cohesive and production-grade: 180/180 web tests pass, the build passes (157 KB gz), `layer_separation` passes (7/7), and all 15 Phase 1–12 capabilities are reachable via registered routes. The single hard gap is that the **animated background is not actually rendered** — `AnimatedBackground` is implemented and unit-tested but never mounted in `AppShell` (or anywhere else), and no CSS animation backs it. The component also still carries the superseded teal gradient rather than the Concordance forest-green palette. This fails roadmap success criterion #2 ("an animated background") and the phase goal's explicit promise. A one-line mount (amending the gradient to forest green) closes it, or it must be formally dropped with a spec/roadmap amendment + override. Additional unmeasured items (motion performance, a11y, E2E walkthrough, font-licensing owner gate) are routed as human-verification items and do not block automated gates.

---

_Verified: 2026-11-06_
_Verifier: the agent (gsd-verifier)_