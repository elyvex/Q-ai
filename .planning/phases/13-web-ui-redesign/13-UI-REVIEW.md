# Phase 13 — UI Review

**Audited:** 2026-11-01
**Baseline:** 13-UI-SPEC.md (Concordance, approved 2026-11-01)
**Screenshots:** not captured (no Playwright browser binaries in this environment — `~/.cache/ms-playwright` empty, no system Chrome; `vite preview :4199` served HTTP 200 but all 3 captures failed) — audited against committed screenshots in `docs/ui-redesign/screenshots/` plus full source audit of `web/src`
**Interaction captures:** off (`workflow.ui_interaction_capture` is false) — interaction findings are code-derived

> Screenshot staleness warning: only 5 committed screenshots match the current build (citations/isnad/rag-debug/tools/words-light, captured 07:04 after the 06:55 `dist` build — 9-link nav, green accent). The other ~13 (index/reader/research/graph/explorer light+dark+mobile, mobile-nav, command, captured Oct 31 23:30–Nov 1 00:15) show a superseded 5-link indigo build. Visual-pillar evidence below distinguishes fresh vs stale accordingly.

---

## Pillar Scores

| Pillar | Score | Key Finding |
|--------|-------|-------------|
| 1. Copywriting | 3/4 | Contract strings match nearly verbatim; minor CTA deviations + one blank pre-run state |
| 2. Visuals | 3/4 | Clear focal point, labelled icon controls, ruled structure; citations pre-run renders a void |
| 3. Color | 1/4 | **BLOCKER:** shipped `--primary`/`--accent`/`--ring`/ground are forest-green/warm-white, spec mandates indigo/porcelain; indigo appears nowhere in `web/src` |
| 4. Typography | 2/4 | Font stacks, caps discipline, Arabic metrics correct; display cap exceeded, counts + search Arabic + field sizes off-tier |
| 5. Spacing | 3/4 | Scale-adherent grids/padding; two catalogue sidebars and a few fractional gaps off the declared scale |
| 6. Experience Design | 3/4 | Loading/error/empty/disabled/denied coverage on every view; one blank-panel violation; mobile nav unverified |

**Overall: 15/24**

---

## Top 3 Priority Fixes

1. **BLOCKER — Restore the Concordance accent tokens (or formally amend UI-SPEC §3).** `web/src/index.css` `:root` + `.dark` ship `--primary: oklch(0.42 0.09 145)` (forest green), `--accent`/`--ring` hue 145, warm-white ground (hue 90/120/100/60) — the spec mandates `--primary: oklch(0.478 0.199 266)` indigo on porcelain `oklch(0.985 0.0025 265)`, lifted indigo `oklch(0.705 0.152 266)` in dark. No `266` token exists anywhere in `web/src`. Every primary surface (monogram, active-route rule, Open/Run buttons, seg pressed states, focus ring) renders the palette §2 explicitly rejects. Concrete fix: replace the six hue families in `:root` and `.dark` with the §3 table values, rebuild, re-capture; then delete or re-capture the 13 stale indigo screenshots so evidence matches the build.
2. **Citations pre-run renders a blank panel.** `/citations` before Open shows the form then a full-viewport void (`CitationsWorkbenchView.tsx:155+` — idle verdict/text states render `null`). UI-SPEC §11: empty states are typed and named, never a blank panel. Concrete fix: render `<EmptyState title="no citation is open">` + one sentence ("enter a stored citation id above — the verdict prints here verbatim") in the idle branch, mirroring `ToolsView.tsx:347` / `RagDebugView.tsx:204`.
3. **Typography tier violations.** (a) `page-header.tsx:21` — `sm:text-[2.3rem]` exceeds the §4 display cap of 2.25rem; change to `sm:text-[2.25rem]`. (b) `WordsView.tsx:315,344` — count numerals use `text-2xl`, a fifth display-ish tier; use the mono/count treatment inside an existing tier (e.g. `text-[0.9375rem]` tabular, matching `.ayah-translation` size, or declare the exception in §4). (c) `ResearchView.tsx:105`, `SearchView.tsx:122` — Arabic search-hit snippets at `text-lg` sit far below the 1.75rem Arabic floor; raise to the ayah tier or record a list-density exception in the spec. (d) `CitationsWorkbenchView.tsx:308` — `field … text-base` overrides the §10 13px `.field`; drop the override. (e) `site-header.tsx` kbd `text-[0.6rem]` breaks the 0.625rem floor; use `text-[0.625rem]`.

---

## Detailed Findings

### Pillar 1: Copywriting (3/4)

Contract strings verified against implementation — overwhelmingly exact:

- Empty states match §19 verbatim: "No reference is open" / "A reader never invents a verse…" (`router.tsx:182`, screenshot), "No matches" / "no results for …" (`ResearchView.tsx:89`, `SearchView.tsx:113`), "No graph lines" / "no graph lines selected…" (`GraphView.tsx:145`), sealed RAG note (`RagDebugView.tsx:211` "Unavailable from the browser" + `router.tsx:107` "sealed vault" + QAI-RAG-0326 rows).
- Error strings match §19: "citation mismatch" (`CitationView.tsx:63`), "No edition is loaded…" (`router.tsx:217`), "Use /read/<edition>/<surah>:<ayah>" (`router.tsx:77`), all in `role="alert"` with remedy text.
- CTA voice is verb+noun, lowercase, no emoji, no generic Submit/OK/Cancel/Save anywhere in non-test sources (grep clean).
- Theme toggle announces exact contract action: "Switch to light/dark theme" (`site-header.tsx:103-105`). Skip link present (`app-shell.tsx:23`).

WARNINGs (why not 4): "Dispatch request" (`ExplorerView.tsx:140`) vs contract "Dispatch"; per-tool "Run quran.get_ayah / Run isnad.trace" vs contract "Run instrument"; Explorer pre-run transcript prints raw "awaiting dispatch" text (`ExplorerView.tsx:174`) instead of a typed `EmptyState`; `/citations` pre-run has no empty copy at all (blank — see Pillar 6).

### Pillar 2: Visuals (3/4)

- Focal point: every page opens with a Newsreader `.display` title under a mono kicker (confirmed in all 8 screenshots read + `PageHeader`).
- Icon-only controls all labelled: command trigger, mobile command button, theme toggle, nav menu (`site-header.tsx:84,96,103,119`), bookmark toggles (`aria-pressed`), seg buttons (`aria-pressed` via DisplayModeControl).
- Hierarchy through size/weight/color: kicker caps → serif title → muted lede → hairline → workspace; ruled 1px-divider lists with mono indices; provenance dots always beside text labels, never alone (reader screenshot legend: "served verbatim / attributed, separate / lexical notes").
- Layout contract held: 86rem measure, 56px rail, 28px monogram, 2px inset active rule, margin column `border-l` only at desktop following content on mobile (`ReadingView.tsx:240,415`), 9-destination horizontal nav with `aria-current="page"` (`site-header.tsx:60`), Reader active on `/` + `/citations` also-list.

WARNINGs: Citations pre-run void (process: blank panel); sticky rail uses `bg-background/85 backdrop-blur-md` (`site-header.tsx:38`) — glassiness the spec never authorizes (§12 bans glass on the palette backdrop; rail unspecified — treat as minor); screenshot evidence set is half-stale (see header note), so the 5-link mobile-nav and dark-mode captures cannot verify the current 9-link build.

### Pillar 3: Color (1/4) — BLOCKER

- **The shipped accent is not the specified accent.** `index.css:85,91,96` (`:root`) and `:132,138,143` (`.dark`): primary/accent/ring at hue 145/forest-green, ground/foreground/muted at warm hues 90–120. Spec §3 + §13 require indigo 266 on porcelain 265 (light) and lifted indigo on ink-chalk (dark). Pixel-sampled fresh screenshots confirm: tools/words/citations/isnad-light monogram `#295a2c` (green) vs stale index/reader-light `#284dcb` (indigo). `grep -rn 266 web/src` returns only the self-contradicting comment at `index.css:19-20` ("Concordance on green/white … one ultramarine instrument accent").
- History: `c0278b3` (05:03, *after* spec approval at 04:08) "add search, hadith, tafsir, scripture views + green/white theme" reintroduced the green the spec §2 explicitly rejects ("rejects their … forest-green … silhouette") without amending UI-SPEC. Either revert to the §3 table or amend the contract — a build that contradicts its approved spec on the single most visible token cannot pass.
- What passes: provenance tokens match §3 exactly (`--canon 0.49/0.083/192`, `--trans`, `--annot`, `--comp` + softs, lifted dark variants); accent confined to interactive elements + active route (no decorative misuse found); `--destructive` correct in both themes; borders hairline `oklch(0.905 0.005 265)`; destructive/failure red only. Hardcoded hex exists only in legacy `tokens/themes.css` layer overrides (pre-Concordance `[data-theme]` set, apparently unreferenced by the `next-themes` class strategy — dead code, recommend removal). 60/30/10 distribution structurally plausible (porcelain ground, ink text, indigo/green reserved).

### Pillar 4: Typography (2/4)

Correct: self-hosted WOFF2 via `@fontsource` (Inter 400/500/600, Newsreader Variable, JetBrains Mono, Amiri 400/700 + Noto Naskh) — zero remote CDNs (`index.css:4-12`); `font-serif` used only for wordmark + `.display` (`site-header.tsx:47`, `page-header.tsx:21`); mono caps kickers with letterspacing, no other letterspaced text; Arabic `dir="rtl" lang="ar"`, Amiri, `leading-[2.15/2.2]`, no synthetic weights, Uthmani marks intact (reader screenshot); body 0.875rem tier respected by `.btn/.field/.seg`; tabular zero-padded counts in mono.

Deviations (each WARNING, collectively a 2): display `sm:text-[2.3rem]` > 2.25rem cap (`page-header.tsx:21`); `text-2xl` count numerals outside the four tiers (`WordsView.tsx:315,344`); Arabic at `text-lg` in search/research hits below the 1.75rem Arabic floor (`ResearchView.tsx:105`, `SearchView.tsx:122`); `text-base` (16px) in vendored `input.tsx`/`textarea.tsx` primitives and the `CitationsWorkbenchView.tsx:308` `.field` override vs §10's 13px field; `kbd` at `text-[0.6rem]` below the 0.625rem floor; seg stepper mixes `text-xs/text-base/text-xl` (`ReadingView.tsx:437`).

### Pillar 5: Spacing (3/4)

Compliant: 4px Tailwind base; page padding 16/24/32 across breakpoints (`app-shell.tsx:27,32,39`: `px-4 sm:px-6 lg:px-8`); section rhythm `mt-8` after headers, 16–24px block gaps (`gap-x-10/12`, `space-y-6`); workspace grids match §6 (`ReadingView 1fr/16rem`, `ResearchView 13rem/1fr`); size-7 monogram (28px), 5px radii, 32px `.field`/`.btn` heights, hairline `h-[2px]` rules.

WARNINGs: `ToolsView.tsx:161` catalogue sidebar is `18rem` and `ExplorerView.tsx:64` is `14rem` where §6 declares the 13rem instrument index — adopt or document; fractional off-scale gaps `gap-2.5` (`SearchView.tsx:71`), `mt-2.5` lede (`page-header.tsx:23`), `px-3.5/py-2.5` (`CitationView.tsx:47`) — 10px steps outside {4,8,16,24,32,48}; `size-3.5/gap-0.5` icon nudges (`ToolsView.tsx:182,227,331`, `SearchView.tsx:63,103`, `TafsirView.tsx:66`) exceed the "1–2px hairline-offset" allowance since they size gaps, not borders. Spacing-utility distribution is otherwise disciplined (top: `mb-2` 46, `gap-2` 33, `mt-3` 24 — all on-scale).

### Pillar 6: Experience Design (3/4) — code-derived (interaction capture off)

- State coverage is the phase's strength: every data view renders `LoadingLine` in flight, `ErrorNote` (code — summary — remedy) on typed failure, and a named `EmptyState` pre-run and on empty results (Words 4 sites, Research, Graph, Tools, Isnad, RagDebug, Tafsir/Hadith/Search/Scripture/Reading). `Run` stays disabled with a live `required: <label>` reason (`ToolsView.tsx:329`, `IsnadView.tsx:178`); refused tools render as inert `aria-disabled` rows with policy codes, no form, no control; clipboard copy surfaces a failure message instead of a silent no-op.
- A11y contract held: landmarks + one `h1` per page, skip link, `aria-current`, `aria-expanded/controls` menu, `aria-pressed` segs, focus trap + Escape in `CommandDialog`, `:focus-visible` 2px ring at 2px offset (`index.css:177`), `aria-live="polite"` transcript, `role="status/alert"` loaders/errors, `prefers-reduced-motion` global reset (`index.css:206`), 120ms color/border transitions only, no entrance animation. No destructive actions ship, so no confirmation step is owed.
- WARNINGs: `/citations` idle renders nothing below the form (blank-panel §11 violation — Top Fix #2); mobile navigation panel and all post-interaction states (focus ring paint, palette open, seg switching, copy feedback, bookmark persistence across reload) verified by code/test only, not by capture — the only mobile evidence (`mobile-nav.png`, `index-mobile.png`) depicts the superseded build; `byteDiff` tamper simulator and re-verify counter covered by unit tests (6 + 11 tests) but never exercised against a live engine (open D4/D5 human-judgment items in 13-14-SUMMARY).

---

## Registry Safety

Skipped — `web/components.json` exists (shadcn-style primitives vendored in-tree under `components/ui/`), but UI-SPEC §20 declares **zero third-party registries** ("no third-party registries, no component installs at runtime, no remote CDNs"). Per the audit rule (only runs when UI-SPEC lists third-party entries), no `npx shadcn view/diff` was executed. `grep` confirms no `fetch(`/`XMLHttpRequest`/`sendBeacon`/`eval(`/remote-`import(` in any new view or lib file; all server payloads render as React text children.

---

## Files Audited

- Contract/baseline: `13-UI-SPEC.md` (§§1–20), `13-CONTEXT.md`, `13-10/11/12/13/14-SUMMARY.md`
- Sources: `web/src/index.css`, `web/src/tokens/layers.css`, `web/src/tokens/themes.css`, `web/src/components/layout/{site-header,app-shell,page-header}.tsx`, `web/src/components/command/command-menu.tsx`, `web/src/components/states.tsx`, `web/src/components/{AyahColumn,TranslationPanel,CitationCopy,ToolLedger}.tsx`, `web/src/views/{Reading,Research,Words,Tools,Graph,RagDebug,Explorer,CitationsWorkbench,Citation,Isnad,Hadith,Tafsir,Scripture,Search}View(s).tsx`, `web/src/display/DisplayModeControl.tsx`, `web/src/lib/{tools,words,related,byte-diff,tool-result,reader-storage}.ts`, `web/src/router.tsx`, `web/components.json`
- Screenshots read: index/reader/research/graph (light), index-dark, index/reader-mobile, mobile-nav, command-light (stale set — superseded build); citations/isnad/rag-debug/tools/words-light (fresh — current build)
