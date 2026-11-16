---
phase: 13-web-ui-redesign
verified: 2026-11-16T15:56:47Z
status: human_needed
score: 8/11 must-haves verified
covered_files: [".planning/phases/13-web-ui-redesign/13-01-PLAN.md",".planning/phases/13-web-ui-redesign/13-02-PLAN.md",".planning/phases/13-web-ui-redesign/13-03-PLAN.md",".planning/phases/13-web-ui-redesign/13-04-PLAN.md",".planning/phases/13-web-ui-redesign/13-05-PLAN.md",".planning/phases/13-web-ui-redesign/13-06-PLAN.md",".planning/phases/13-web-ui-redesign/13-07-PLAN.md",".planning/phases/13-web-ui-redesign/13-08-PLAN.md",".planning/phases/13-web-ui-redesign/13-09-PLAN.md",".planning/phases/13-web-ui-redesign/13-10-PLAN.md",".planning/phases/13-web-ui-redesign/13-10-SUMMARY.md",".planning/phases/13-web-ui-redesign/13-11-PLAN.md",".planning/phases/13-web-ui-redesign/13-11-SUMMARY.md",".planning/phases/13-web-ui-redesign/13-12-PLAN.md",".planning/phases/13-web-ui-redesign/13-12-SUMMARY.md",".planning/phases/13-web-ui-redesign/13-13-PLAN.md",".planning/phases/13-web-ui-redesign/13-13-SUMMARY.md",".planning/phases/13-web-ui-redesign/13-14-PLAN.md",".planning/phases/13-web-ui-redesign/13-14-SUMMARY.md",".planning/phases/13-web-ui-redesign/13-15-PLAN.md",".planning/phases/13-web-ui-redesign/13-15-SUMMARY.md","web/package.json","web/public/fonts/amiri-arabic-400-normal.woff2","web/public/fonts/amiri-arabic-700-normal.woff2","web/public/fonts/noto-naskh-arabic-arabic-400-normal.woff2","web/public/fonts/noto-naskh-arabic-arabic-700-normal.woff2","web/src/App.tsx","web/src/api/client.ts","web/src/components/AyahColumn.test.tsx","web/src/components/AyahColumn.tsx","web/src/components/ToolLedger.test.tsx","web/src/components/ToolLedger.tsx","web/src/components/background/AnimatedBackground.tsx","web/src/components/command/command-menu.tsx","web/src/components/layout/app-shell.tsx","web/src/components/layout/site-header.tsx","web/src/components/ui/Badge.tsx","web/src/components/ui/Button.tsx","web/src/components/ui/Card.tsx","web/src/components/ui/EmptyState.tsx","web/src/components/ui/ErrorBoundary.tsx","web/src/components/ui/ErrorState.tsx","web/src/components/ui/IconButton.tsx","web/src/components/ui/Input.tsx","web/src/components/ui/Modal.tsx","web/src/components/ui/Select.tsx","web/src/components/ui/Skeleton.tsx","web/src/components/ui/Spinner.tsx","web/src/components/ui/Tabs.tsx","web/src/components/ui/Toast.tsx","web/src/components/ui/button.tsx","web/src/components/ui/dialog.tsx","web/src/components/ui/input-group.tsx","web/src/components/ui/input.tsx","web/src/components/ui/ui.test.tsx","web/src/display/DisplayModeControl.test.tsx","web/src/display/DisplayModeControl.tsx","web/src/index.css","web/src/lib/byte-diff.test.ts","web/src/lib/byte-diff.ts","web/src/lib/reader-storage.test.ts","web/src/lib/reader-storage.ts","web/src/lib/related.test.ts","web/src/lib/related.ts","web/src/lib/tool-result.test.ts","web/src/lib/tool-result.ts","web/src/lib/tools.test.ts","web/src/lib/tools.ts","web/src/lib/words.test.ts","web/src/lib/words.ts","web/src/router.test.tsx","web/src/router.tsx","web/src/tests/animated-background-shell.test.tsx","web/src/tests/reachability.test.ts","web/src/tokens/layers.css","web/src/tokens/themes.css","web/src/views/CitationsWorkbenchView.test.tsx","web/src/views/CitationsWorkbenchView.tsx","web/src/views/ExplorerView.test.tsx","web/src/views/ExplorerView.tsx","web/src/views/IsnadView.test.tsx","web/src/views/IsnadView.tsx","web/src/views/RagDebugView.test.tsx","web/src/views/RagDebugView.tsx","web/src/views/ReadingView.test.tsx","web/src/views/ReadingView.tsx","web/src/views/ResearchView.test.tsx","web/src/views/ResearchView.tsx","web/src/views/ScriptureView.tsx","web/src/views/SearchView.tsx","web/src/views/TafsirView.tsx","web/src/views/ToolsView.test.tsx","web/src/views/ToolsView.tsx","web/src/views/WordsView.test.tsx","web/src/views/WordsView.tsx"]
covered_digest: "v3:sha256:5e38fcf7b97651b0e7ef102f2cfe520aaaa5dd7a181737497de9951ecd603959"
behavior_unverified: 1
overrides_applied: 0
re_verification:
  previous_status: gaps_found
  previous_score: 8/11
  gaps_closed:
    - "Motion performance within the phase's own budgets (roadmap SC7 + SC11): desktop LCP 448-452 ms, CLS <= 0.0002, perf 1.00, bundle 156.99 kB gz JS + 31.22 kB gz CSS. The mobile-emulation reading (1907-1942 ms) was reclassified as informational by the dated 2026-11-16 clarification on ROADMAP SC11; the reverted code-split experiment is recorded in 13-MOTION-EVIDENCE.md."
  gaps_remaining: []
  regressions: []
deferred:
  - truth: "Dark theme"
    addressed_in: "Phase 14 (deferred per owner gate)"
    evidence: "ROADMAP Phase 13 out-of-scope: 'Light theme only. Dark theme is deferred.' 13-CONTEXT.md records both themes shipping via next-themes; informational, not an unmet item."
behavior_unverified_items:
  - truth: "Micro-animations run at >= 60 fps (roadmap SC7), separate from the LCP/CLS/perf/bundle gates that now pass"
    test: "Record a frame-rate trace (DevTools performance panel or rAF sampler) while scrolling a view and while the gradient field animates on real hardware"
    expected: "Sustained ~60 fps with no long tasks attributable to the field; CLS stays < 0.1 (already measured at <= 0.0003)"
    why_human: "The field animates a composited transform only and CLS is zero, but no frame-rate sample exists in the repo or the evidence file; presence of a transform keyframe cannot prove frame pacing"
human_verification:
  - test: "Font licensing owner gate (OD-NEW): ratify the license of each bundled Arabic and UI font"
    expected: "Each font's license recorded in the license matrix; OD-NEW closed"
    why_human: "Owner-only decision; blocks REQ-security-licensing-observability closure"
  - test: "Run /rag-debug against a live engine and confirm the checksum vs QAI-RAG-0310"
    expected: "rag.debug returns hits + route decision with research_checksum; QAI-RAG-0310 shown when backend unconfigured"
    why_human: "Needs a running Rust engine with a loaded edition; 13-UAT.md test 1 is still `[pending]`"
  - test: "Cold-load /words, /tools, /citations, /isnad in the served build and run one instrument each against a live engine"
    expected: "Routes load, rail highlight active, instruments return checksums matching the CLI"
    why_human: "jsdom covers route wiring only; needs served build + running engine; 13-UAT.md tests 2-6 still `[pending]`"
  - test: "Accessibility audit: axe zero serious/critical on every route, keyboard-only core flows (Lighthouse a11y is measured and green)"
    expected: "No serious/critical axe violations; read/search/copy-citation/ask/approve completable by keyboard alone"
    why_human: "Lighthouse accessibility = 1.00 closes the '>= 90' half; axe-per-route and keyboard-only flows have no automated run in the repo"
  - test: "E2E browser walkthrough (read -> search -> inspect -> graph -> ask -> verify-citations -> export) without touching CLI/API"
    expected: "Full walkthrough completes in the browser"
    why_human: "No Playwright suite present (no config/spec on disk); deferred to Phase 14's E2E lane"
  - test: "60 fps motion sample: trace the animated field and a route transition on real hardware"
    expected: "Sustained ~60 fps, no layout shift, no long task caused by the field"
    why_human: "No frame-rate trace exists; a CSS transform keyframe proves implementation, not frame pacing"
---

# Phase 13: Web UI Redesign — Verification Report

**Phase Goal:** Users get a from-scratch redesign of the Vite + React web UI in an elegant light white/gray style, with an animated background and polished buttons and controls. Every Q-ai feature from Phases 1–12 is reachable through the UI. The result is a cohesive, production-grade single-page application that feels like a dedicated desktop research tool rendered in the browser. The redesign adds no backend capability.

**Verified:** 2026-11-16T15:56:47Z
**Status:** human_needed
**Re-verification:** Yes — after the operator's dated clarification of roadmap SC11

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Feature reachability: all Phase 1–12 capabilities have an active registered route | ✓ VERIFIED | `web/src/tests/reachability.test.ts` asserts 15 capabilities against `ROUTES` (incl. aliases); passes in the 24-file / 187-test run. `router.tsx` registers `/`, `/search`, `/research`, `/words`, `/tools`, `/citations`, `/hadith`, `/isnad`, `/tafsir`, `/scripture`, `/graph`, `/rag-debug`, `/explorer` |
| 2 | Light white/gray theme with polished controls | ✓ VERIFIED | `index.css` light anchors pinned by the design-contract guard: `--background: oklch(0.99 0.004 90)`, `--foreground: oklch(0.22 0.012 120)`, single green accent `--primary: oklch(0.42 0.09 145)`; primitive library under `components/ui/*` |
| 3 | Animated background behind content | ✓ VERIFIED | `app-shell.tsx:20` mounts `<AnimatedBackground />` as the shell's first child, before the skip link. `App.tsx` carries `min-h-screen font-sans antialiased` only; page ground lives on `body`. `.gradient-field` + `@keyframes gradient-drift` asserted by the guard; component has no colour literal. Prior negative proof (removing the mount turns 3 guard cases red) still holds — the guard file is unchanged |
| 4 | Layer integrity: canonical/translation/annotation distinct, tokens preserved | ✓ VERIFIED | Verifier ran `cargo test -p application --test layer_separation` — 7/7 pass, including `spa_layer_tokens_define_all_three_families` and `spa_non_canonical_components_wear_no_canonical_class`; `tokens/layers.css` defines all three families |
| 5 | Tokenized design system, single source of truth | ✓ VERIFIED | `web/src/tokens/` contains only `layers.css` and `themes.css`; the guard asserts the five retired mirrors stay absent and pins the palette/field/font anchors |
| 6 | Interactive states on every control | ✓ VERIFIED | `components/ui/ui.test.tsx` covers disabled + loading spinner, invalid + error text, focus management, keyboard tab navigation, dismissible toasts; view tests green in the same 187-test run |
| 7 | Arabic + RTL with self-hosted fonts | ✓ VERIFIED | `AyahColumn.tsx:22` `dir="rtl" lang="ar"`; five `@fontsource` families imported in `index.css`; guard asserts no `@import`/`url()` carries an `http(s):` origin; `spa_has_no_remote_url_or_bundled_font` passes |
| 8 | Motion performance within the phase's budgets (LCP ≤ 1.5 s desktop, CLS < 0.1, perf ≥ 80, bundle ≤ 350 kB gz, ≥ 60 fps) | ⚠️ PRESENT_BEHAVIOR_UNVERIFIED | All numeric gates pass on the recorded desktop runs: LCP 448–452 ms, CLS ≤ 0.0002, perf 1.00, bundle 156.99 kB gz JS + 31.22 kB gz CSS. The ≥ 60 fps clause has no frame-rate trace (`13-MOTION-EVIDENCE.md` says so explicitly) → human item. Mobile emulation 1907–1942 ms is **informational** per the amended SC11 |
| 9 | End-to-end browser walkthrough without CLI/API | ? UNCERTAIN | No Playwright config/spec on disk (`@playwright/test` is a devDependency only). Routes are wired and unit-tested; the walkthrough is unproven → human |
| 10 | Accessibility (Lighthouse a11y ≥ 90, axe 0 serious/critical, keyboard flows) | ? UNCERTAIN (narrowed) | Lighthouse accessibility = 1.00 on all four recorded runs — the ≥ 90 half has evidence. No axe run and no keyboard-only E2E → human |
| 11 | Gates green (build + tests + layer_separation, no backend diff) | ✓ VERIFIED | Verifier ran them on this tree: `npm test` 24 files / 187 tests green; `npm run build` green, emitting `index-DGCuZuVF.js` 525.93 kB raw / 156.99 kB gzip and `index-DY9tnza9.css` 111.33 kB / 31.22 kB gzip (same hashes as the last verified state); `cargo test -p application --test layer_separation` 7/7; `git status` shows no modified `.rs` file |

**Score:** 8/11 truths verified (1 present-behavior-unverified, 2 uncertain → human verification)

### Deferred Items

| # | Item | Addressed In | Evidence |
|---|------|-------------|----------|
| 1 | Dark theme | Deferred per roadmap out-of-scope | ROADMAP Phase 13: "Light theme only. Dark theme is deferred." |

### Advisory (New Scope, Unevidenced)

None.

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `web/src/components/layout/app-shell.tsx` | Mounts the field on every route | ✓ VERIFIED | First child of the root `div`, before the skip link |
| `web/src/components/background/AnimatedBackground.tsx` | Behaviour only, no colour literal | ✓ VERIFIED | `aria-hidden`, `pointer-events: none`, `z-index: -1`, opacity ≤ 0.15, visibility pause, reduced-motion stop, `gradient-field` class |
| `web/src/index.css` | Tokenised atom + keyframes | ✓ VERIFIED | `.gradient-field` in `@layer components`; `@keyframes gradient-drift` unlayered, transform-only |
| `web/src/App.tsx` | No opaque page ground over the field | ✓ VERIFIED | `min-h-screen font-sans antialiased` only; ground is on `body` |
| `web/src/tests/animated-background-shell.test.tsx` | Fails if the mount/atom/keyframes/ground regress | ✓ VERIFIED | 5 cases, unchanged since the negative proof |
| `web/src/tokens/` | One source: `layers.css` + `themes.css` only | ✓ VERIFIED | `ls web/src/tokens/` shows exactly those two files |
| `.planning/phases/13-web-ui-redesign/13-MOTION-EVIDENCE.md` | Measured budgets with per-row verdict, plus rejected experiments | ✓ VERIFIED | Desktop PASS rows; mobile row kept as a measured FAIL and reclassified informational by ROADMAP SC11; "Experiments tried and rejected" records the code-split measurement (156.99 → 136.82 kB gz; `/search` LCP 1942 → 3778 ms) and the revert |
| `web/src/router.tsx` | 13 routes reaching every view | ✓ VERIFIED | 13 routes; eager imports are the accepted design after the code-split experiment was measured and reverted |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| AppShell | AnimatedBackground | mount | ✓ WIRED | `app-shell.tsx:5` import, `:20` render; asserted by the guard test |
| `.gradient-field` | `--primary` / `--ring` | `color-mix` | ✓ WIRED | `index.css` atom built from both tokens; component has no colour literal |
| `.gradient-field` | `@keyframes gradient-drift` | `animation: gradient-drift 24s …` | ✓ WIRED | Asserted by the design-contract guard |
| App.tsx | body page ground | no `bg-background` on the wrapper | ✓ WIRED | Ground on `body`; negative-z field not repainted |
| Router | Views | direct imports | ✓ WIRED | All 13 routes render a view; `git diff HEAD -- web/src/router.tsx` is empty (byte-identical to the verified state) |

### Data-Flow Trace (Level 4)

Not applicable — the web views fetch from the Rust HTTP API; data flow is covered by unit tests stubbing `fetch` and by the parity checksum contract (`research_checksum`). The background is decorative and takes no data.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Web test suite | `cd web && npm test` | 24 files / 187 tests pass (14.12 s) | ✓ PASS |
| Web build | `cd web && npm run build` | green; `index-DGCuZuVF.js` 156.99 kB gz + `index-DY9tnza9.css` 31.22 kB gz (under 350 kB) | ✓ PASS |
| Layer separation | `cargo test -p application --test layer_separation` | 7/7 pass | ✓ PASS |
| No backend diff | `git status --porcelain` | only planning/trail files; no `.rs` modified | ✓ PASS |
| Debt markers | `rg 'TBD\|FIXME\|XXX' web/src` | no matches | ✓ PASS |

### Probe Execution

Not applicable — no probe scripts for this phase.

### Requirements Coverage

| Requirement | Source Plan(s) | Description | Status | Evidence |
|-------------|----------------|-------------|--------|----------|
| REQ-ux-web | 13-10..13-15 | Web navigation, search, research workspace, export, streaming | ✓ SATISFIED (partial) | 13 routes + directory + command menu; streaming stays Phase 14 |
| REQ-quran-display | 13-11, 13-13 | Reading view, citation interaction, layer distinction | ✓ SATISFIED | ReadingView, AyahColumn layer tokens, citation copy |
| REQ-quran-research-tools | 13-11, 13-12 | Typed tools usable from UI/API | ✓ SATISFIED | `/tools` (27 tools), `/words` (morphology + 5 counts) |
| REQ-quran-result-contract | 13-10, 13-11, 13-12, 13-14 | Result fields incl. `research_checksum` | ✓ SATISFIED | ToolLedger prints checksum; byte-diff; citations verdict/hash/URN |
| REQ-cli-api | 13-12 | CLI/API parity surfaced | ✓ SATISFIED | ToolsView prints `qai …` + HTTP equivalents from sent params |
| REQ-hadith-corpus | 13-13, 13-14 | Hadith browser + isnad | ✓ SATISFIED | HadithView, IsnadView |
| REQ-tafsir-comparative | 13-13 | Tafsir + comparative scripture | ✓ SATISFIED | TafsirView, related lookup, ScriptureView |
| REQ-isnad-narrator-graph | 13-14 | Isnad/narrator graph + identity uncertainty | ✓ SATISFIED | IsnadView read-only + denied identity mutations |
| REQ-multi-rag | 13-10 | Multi-index retrieval + debug | ✓ SATISFIED | RagDebugView runs `rag.debug`/`query` with route decision |
| REQ-generic-rag-platform | 13-10 | RAG project surfaces | ✓ SATISFIED | Explorer / RAG debug surfaces |
| REQ-knowledge-domains | 13-13 | Domain filters | ✓ SATISFIED | ScriptureView domain filter |
| REQ-agents-tools-runtime | 13-12 | Tool manifests, sandbox, approvals | ✓ SATISFIED (partial) | `/tools` catalogue; sandbox/approval UI is Phase 14 |
| REQ-source-catalog-trust | 13-14 | Source catalogue trust surfaces | ✓ SATISFIED (partial) | Citations/Explorer provenance; full catalogue is Phase 14 |
| REQ-retrieval-governance | 13-10 | Prompt-injection defense, audit, budgets | ✓ SATISFIED (partial) | `QAI-RAG-0326` denied list; route decision verbatim |
| REQ-quran-graph | 13-14 | Graph traversal UI | ✓ SATISFIED | GraphView neighbors/paths |
| REQ-security-licensing-observability | (roadmap R2) | Font licensing, no third-party requests | ? NEEDS HUMAN | Self-hosted fonts and the no-remote-origin guard pass; font-licensing owner gate OD-NEW still open |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `web/src/index.css` | 445 | `will-change: transform` on a fixed full-screen element | ℹ️ Info | Intentional for a composited transform; promoted layer only |
| `web/src/router.tsx` | — | Eager imports, no `React.lazy`/`Suspense` | ℹ️ Info | Evaluated and rejected with measurements (13-MOTION-EVIDENCE.md): split cut the bundle 20 kB gz but made `/search` mobile LCP 1942 → 3778 ms. Not a gate; desktop is inside every budget |

No `TBD`/`FIXME`/`XXX` markers in `web/src`. No orphaned component.

### Human Verification Required

1. **Font licensing owner gate (OD-NEW)**
   - Test: Ratify the license of each bundled Arabic and UI font; record in the license matrix
   - Expected: OD-NEW closed; REQ-security-licensing-observability unblocked
   - Why human: Owner-only decision

2. **Live-engine smoke: /rag-debug**
   - Test: Run `/rag-debug` against a live engine; confirm checksum vs `QAI-RAG-0310`
   - Expected: `rag.debug` returns hits + route decision with `research_checksum`
   - Why human: Needs a running Rust engine with a loaded edition; `13-UAT.md` test 1 is `[pending]`

3. **Live-engine smoke: /words, /tools, /citations, /isnad**
   - Test: Cold-load each route in the served build and run one instrument each against a live engine
   - Expected: Routes load, rail highlight active, instruments return checksums matching the CLI
   - Why human: jsdom covers route wiring only; `13-UAT.md` tests 2–6 `[pending]`

4. **Accessibility audit (axe + keyboard)**
   - Test: axe scan on every route; keyboard-only completion of read/search/copy-citation/ask/approve flows
   - Expected: Zero serious/critical violations; flows completable by keyboard alone
   - Why human: Lighthouse accessibility is measured at 1.00; axe-per-route and keyboard-only flows have no automated run

5. **E2E browser walkthrough**
   - Test: read → search → inspect → graph → ask → verify-citations → export without touching CLI/API
   - Expected: Full walkthrough completes in the browser
   - Why human: No Playwright suite exists (deferred to Phase 14's E2E lane)

6. **60 fps motion sample**
   - Test: Record a frame-rate trace while scrolling a view and while the gradient field animates on real hardware
   - Expected: Sustained ~60 fps, no layout shift, no long task attributable to the field
   - Why human: No frame-rate trace exists; a transform-only keyframe proves implementation, not frame pacing

### Gaps Summary

No blocking gaps remain. The previous single blocker — mobile-emulated LCP at 1.91–1.94 s — was resolved by an explicit, dated criterion amendment rather than by silence: ROADMAP SC11 now records LCP ≤ 1.5 s as the **desktop** reading, matching the goal's "dedicated desktop research tool rendered in the browser", and the mobile reading is recorded as informational. The amendment is backed by measurement, not assertion: desktop LCP 448–452 ms, CLS ≤ 0.0002, Lighthouse performance 1.00, accessibility 1.00, initial bundle 156.99 kB gz JS + 31.22 kB gz CSS. The road not taken is documented too — route-level code splitting was implemented, measured (initial chunk 136.82 kB gz, but `/search` mobile LCP 1942 → 3778 ms) and reverted, and `web/src/router.tsx` is byte-identical to the last verified state (`git diff HEAD` empty).

All three gates pass on the verifier's own runs against this tree. What remains is not implementation: five human/live-engine items (font licensing owner gate, live `/rag-debug` checksum, live `/words` `/tools` `/citations` `/isnad` instruments, axe + keyboard-only audit, E2E walkthrough) plus one behavioral item the prior report already named — no frame-rate trace exists for the ≥ 60 fps clause of roadmap SC7. Hence `human_needed`, not `passed` and not `gaps_found`.

---

_Verified: 2026-11-16T15:56:47Z_
_Verifier: the agent (gsd-verifier)_
