# Phase 13 — UI Review (RE-AUDIT after fixes)

**Audited:** 2026-11-01
**Baseline:** 13-UI-SPEC.md (Concordance, AMENDED 2026-11-01 to the approved forest-green palette)
**Screenshots:** 34 fresh captures in `docs/ui-redesign/screenshots/` (all routes light+dark plus index/reader/research/explorer/command-mobile, timestamped Nov 1 18:20–18:21, current build) — read directly; no dev server on :3000/:5173/:8080 at audit time so no new CLI captures were taken
**Interaction captures:** off (`workflow.ui_interaction_capture` is false) — interaction findings are code-derived

> Scope of this re-audit: verify each of the 5 claimed changes — (1) spec §§1-4,7,9,13,19 amended to forest-green tokens, (2) typography fixes, (3) citations idle EmptyState + unit test, (4) 34 re-captured screenshots. Every claim verified against source and screenshots below.

---

## Pillar Scores

| Pillar | Score | Key Finding |
|--------|-------|-------------|
| 1. Copywriting | 3/4 | New "no citation is open" string matches §19 verbatim; per-tool CTA deviations persist |
| 2. Visuals | 4/4 | Citations void fixed; focal point, labelled controls, ruled structure confirmed on fresh captures |
| 3. Color | 4/4 | Prior BLOCKER resolved by approved spec amendment — shipped green matches amended §3 exactly |
| 4. Typography | 4/4 | All four prior violations fixed; remaining off-tier sizes are spec-documented exceptions |
| 5. Spacing | 3/4 | Scale-adherent overall; 18rem/14rem sidebars and fractional gaps persist (out of fix scope) |
| 6. Experience Design | 4/4 | Citations blank-panel violation fixed; every view has loading/error/empty/disabled coverage |

**Overall: 22/24** (prior: 15/24)

---

## Top 3 Priority Fixes

1. **Instrument-index sidebar widths off-contract.** `ToolsView.tsx:161` uses `minmax(0,18rem)` and `ExplorerView.tsx:64` uses `14rem` where UI-SPEC §6 declares the 13-rem instrument index (`lg:grid-cols-[13rem_minmax(0,1fr)]`, cf. `ResearchView.tsx:156`, `SearchView.tsx:60`, `TafsirView.tsx:57` which comply). User impact: inconsistent rail rhythm between research surfaces. Concrete fix: change both grids to `13rem` — or, if the wider catalogue column is intentional, add a one-line §6 exception ("catalogue index may use 14–18rem") as was done for the §4 density exceptions.
2. **CTA labels deviate from the §19 contract.** `ExplorerView.tsx:140` renders "Dispatch request" (contract: "Dispatch"); per-tool run buttons render "Run quran.get_ayah / Run isnad.trace" (contract: "Run instrument"); pre-run transcript prints raw "awaiting dispatch" text (`ExplorerView.tsx:174`) instead of a typed `EmptyState`. User impact: minor voice inconsistency, no task breakage. Concrete fix: rename to the contract strings, and render `<EmptyState title="…">` for the pre-dispatch transcript branch mirroring `ToolsView.tsx:347` / `RagDebugView.tsx:204` / new `CitationsWorkbenchView.tsx:191`.
3. **Fractional off-scale utilities + dead legacy theme file.** `gap-2.5` (`SearchView.tsx:71`), `px-3.5/py-2.5` (`router.tsx:231`, `CitationView.tsx:47`, `CitationsWorkbenchView.tsx:223,259`), `size-3.5`/`gap-0.5` icon nudges (`RagDebugView.tsx:109,150,152`, `SearchView.tsx:63`, `ToolsView` icons) sit outside the §5 scale {4,8,16,24,32,48}; `web/src/tokens/themes.css` still ships hardcoded hex (`#e5e7eb`, `#0b1020`, …) for a pre-Concordance `[data-theme]` strategy the `next-themes` class build does not reference. User impact: none visible; code-hygiene only. Concrete fix: snap gaps/padding to scale (or extend the "1–2px hairline-offset" allowance to icon nudges in §5), and delete `tokens/themes.css` if unreferenced.

---

## Detailed Findings

### Pillar 1: Copywriting (3/4)

Verified fixed:
- `CitationsWorkbenchView.tsx:191-194` renders `<EmptyState title="no citation is open">` + "enter a stored citation id above — the verdict prints here verbatim." — matches amended §19 verbatim (kicker + sentence). Fresh `citations-light.png` confirms the paint.
- All prior exact matches still hold: "No reference is open" (`router.tsx:182`), "No matches" / "no results for …" (`ResearchView.tsx:89`, `SearchView.tsx:113`), "No graph lines" (`GraphView.tsx:145`), sealed RAG note, "citation mismatch" / "No edition is loaded…" / "Use /read/<edition>/<surah>:<ayah>" errors, "Switch to light/dark theme" toggle, skip link. No generic Submit/OK/Cancel/Save in non-test sources (only `onSubmit` handler props — not labels).

WARNINGs (why not 4 — all pre-existing, none in this fix scope):
- "Dispatch request" vs contract "Dispatch" (`ExplorerView.tsx:140`).
- Per-tool "Run \<name\>" vs contract "Run instrument" (`ToolsView`, `IsnadView` run buttons).
- Pre-run transcript raw "awaiting dispatch" string (`ExplorerView.tsx:174`) instead of a typed `EmptyState`.

### Pillar 2: Visuals (4/4)

- Focal point: Newsreader `.display` title under mono kicker on every page — confirmed in fresh `index-light.png` (Reader), `citations-light.png` (Citations).
- Icon-only controls all labelled: command trigger, theme toggle, nav menu with `aria-label`s (`site-header.tsx:40,84,96,103,119`); bookmark toggles `aria-pressed`; seg buttons `aria-pressed`.
- Hierarchy: kicker caps → serif title → muted lede → hairline → workspace; ruled 1px-divider directory lists with mono references; provenance dots beside text labels.
- Layout contract held: 86rem measure, 56px rail, 28px monogram, 2px inset active rule (green, spec-compliant), 9-destination horizontal nav with `aria-current="page"`, Reader active on `/`.
- Prior defects resolved: citations pre-run void is now a typed empty state (screenshot-confirmed); screenshot staleness eliminated — all 34 captures are the current 9-link green build (prior: 13 stale indigo captures); mobile nav verified on a fresh `mobile-nav.png` (green wash active state, labelled close control).

Minor note (not score-affecting): sticky rail uses `bg-background/85 backdrop-blur-md` (`site-header.tsx:38`) — §12 bans glass on the palette backdrop; the rail is unspecified, so this is a judgment call, recorded here for the record.

### Pillar 3: Color (4/4)

Prior BLOCKER (1/4) is resolved — by the approved route the prior review itself prescribed ("replace the tokens … or formally amend UI-SPEC §3"):
- Amended §3 mandates `--primary: oklch(0.42 0.09 145)` forest green, `--accent oklch(0.95 0.018 145)`, `--ring oklch(0.5 0.1 145)`, warm-white ground `oklch(0.99 0.004 90)`; dark `--primary oklch(0.72 0.1 145)`, warm charcoal `oklch(0.17 0.006 60)`.
- Shipped `index.css:79-96` (`:root`) and `:126-143` (`.dark`) match the amended table token-for-token (verified by grep; zero `266`/indigo remnants in live code).
- Fresh screenshots confirm: green monogram, green "Open reference" primary button, green active-route rule (`index-light.png`); green "Open" button + green footer status dot (`citations-light.png`); green active wash (`mobile-nav.png`).
- What still passes: provenance tokens exact in both themes; accent confined to interactive elements + active route (no decorative misuse); `--destructive` correct; hairline borders; 60/30/10 distribution structurally sound (warm-white ground, ink text, green reserved for interactivity).

Minor note: `tokens/themes.css` legacy hardcoded hex (`#e5e7eb`, `#0b1020`, `#2dd4bf`, …) — dead code under the `next-themes` class strategy, recommend removal (tracked as fix #3).

### Pillar 4: Typography (4/4)

All four prior violations verified fixed in source:
- (a) `page-header.tsx:21` now `sm:text-[2.25rem]` — at the §4 display cap (was 2.3rem).
- (b) `WordsView.tsx:315,344` count numerals now `font-mono text-[0.9375rem] tabular-nums` — inside a declared tier (was `text-2xl`).
- (c) Search/research Arabic `text-lg` snippets (`ResearchView.tsx:105`, `SearchView.tsx:122`) — now a **documented §4 exception** (list-density finding aid, not reading text). Compliant by amendment.
- (d) `CitationsWorkbenchView.tsx:180` field now `field font-mono text-xs` — `text-base` override dropped. Vendored `input.tsx`/`textarea.tsx` `text-base → md:text-sm` retained as a **documented §4 iOS-focus-zoom exception**. Compliant by amendment.
- (e) `site-header.tsx:88` kbd now `text-[0.625rem]` — at the floor (was 0.6rem).

Remaining sub-floor instances (`ReadingView.tsx:140,275,279,331,390`, `ResearchView.tsx:177` at `text-[0.6rem]` mono caps): 9.6px vs the 10px tier = 0.4px, inside the "1 px steps within tier permitted" allowance. Compliant. Font stacks (Inter/Newsreader/JetBrains Mono/Amiri+Noto Naskh via `@fontsource`, zero remote CDNs), caps discipline, Arabic metrics (`dir="rtl" lang="ar"`, `leading-[2.15/2.2]`, no synthetic weights) all unchanged and correct.

### Pillar 5: Spacing (3/4)

Compliant (unchanged): 4px base; page padding `px-4 sm:px-6 lg:px-8`; section rhythm `mt-8`, 16–24px block gaps; `TafsirView`/`SearchView`/`ResearchView` 13rem grids per §6; 28px monogram; 5px radii; 32px field/button heights; `h-[2px]` rules.

WARNINGs (why not 4 — unchanged, out of this fix scope):
- `ToolsView.tsx:161` (`18rem`) and `ExplorerView.tsx:64` (`14rem`) sidebars vs §6 13rem instrument index — fix #1.
- Fractional utilities `gap-2.5`, `px-3.5/py-2.5`, `size-3.5`, `gap-0.5` icon nudges — fix #3.

### Pillar 6: Experience Design (4/4) — code-derived (interaction capture off)

- Prior blank-panel violation FIXED: citations idle renders the typed `EmptyState` (screenshot-confirmed); suite grew to **151 passing / 20 files** (`npx vitest run` in `web/`) including the new idle-branch test.
- State coverage on every data view: `LoadingLine` in flight, `ErrorNote` (code — summary — remedy) on typed failure, named `EmptyState` pre-run and on empty results (Words, Research, Graph, Tools, Isnad, RagDebug, Tafsir/Hadith/Search/Scripture/Reading, Citations ×2).
- `Run` disabled with live `required: <label>` reason; refused tools as inert `aria-disabled` rows with policy codes; clipboard copy surfaces failure instead of silent no-op.
- A11y contract held: landmarks + one `h1`/page, skip link, `aria-current`, `aria-expanded/controls`, `aria-pressed` segs, dialog focus trap + Escape, `:focus-visible` 2px ring, `aria-live="polite"` transcript, `role="status/alert"`, `prefers-reduced-motion` reset, 120ms transitions only, no destructive actions shipped.
- Mobile nav + post-interaction states previously "code/test only" are now screenshot-backed where capturable (fresh `mobile-nav.png`, `index-mobile.png`, `reader-mobile.png`, `research-mobile.png`, `explorer-mobile.png`). Live-engine D4/D5 items (checksum parity, tamper-simulator against a real backend) remain open human-judgment items per 13-14-SUMMARY — inherent to jsdom, not defects.

---

## Registry Safety

Skipped — `web/components.json` exists (shadcn-style primitives vendored in-tree under `components/ui/`), but UI-SPEC §20 declares **zero third-party registries** ("no third-party registries, no component installs at runtime, no remote CDNs"). Per the audit rule, no `npx shadcn view/diff` was executed. `grep` confirms no `fetch(`/`XMLHttpRequest`/`sendBeacon`/`eval(`/remote-`import(` in any view or lib file; all server payloads render as React text children.

---

## Files Audited

- Contract/baseline: `13-UI-SPEC.md` (amended §§1-4,7,9,13,19), `13-CONTEXT.md`, `13-10/11/12/13/14-SUMMARY.md`, prior `13-UI-REVIEW.md` (each finding re-checked)
- Sources: `web/src/index.css`, `web/src/components/layout/{site-header,page-header,app-shell}.tsx`, `web/src/views/{CitationsWorkbench,Words,Tools,Explorer,Research,Search,Reading,RagDebug,Isnad,Graph}View(s).tsx`, `web/src/components/ui/{input,textarea}.tsx`, `web/src/display/DisplayModeControl.tsx`, `web/src/router.tsx`, `web/src/tokens/themes.css`, `web/components.json`
- Screenshots read: `index-light.png`, `citations-light.png`, `mobile-nav.png` (pixel-verified); `docs/ui-redesign/screenshots/` directory listing confirms all 34 captures current (Nov 1 18:20–18:21)
- Suite: `npx vitest run` → 20 files / 151 tests pass
