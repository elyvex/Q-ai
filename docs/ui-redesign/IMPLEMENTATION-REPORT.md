# UI Redesign Implementation Report — pass 2 ("Illuminated Codex")

Rebuilt after the first pass was rejected as template-cloned: pass 1 leaned on
stock shadcn surface patterns and flattened real view functionality into
placeholders. Pass 2 is a bespoke design language with every original
capability restored.

## Design language: Illuminated Codex

- Warm parchment light theme (`oklch` warm-hue surfaces), warm-charcoal dark
  theme; **ink-dark sidebar in both modes** (the bound spine).
- Gold canonical accents: verse-number **medallions**, `.kicker` eyebrows with
  gold tick, gold-inset active segments.
- Hairline borders, vellum `.plate` panels, editorial one/two-column layouts —
  no stock gray-canvas white-card grid.
- Typography: Inter (UI), **Amiri** for canonical Arabic, Noto Naskh Arabic
  fallback, JetBrains Mono for references/edges.
- Provenance tokens are structural: canon (teal-green) = canonical layer,
  trans (indigo) = translations, annot (amber) = annotations/alerts,
  comp (violet) = computed data.

## Bespoke primitives (`src/index.css`)

`.kicker`, `.medallion`, `.plate`, `.toolbar`/`.seg`, `.ibtn` (+ primary),
`.chip`, `.prodot`, `.rule-line` — token-driven, light/dark aware. `legacy.css`
was **deleted** (conflicting global styles, largely unused); its import in
`App.tsx` removed. Fonts and layer tokens now load through `index.css`.

## Functionality restored (was flattened in pass 1)

- **ResearchView** — all 9 instruments wired to the typed client: `search`,
  `quran.root`, `quran.lemma`, `family`, root/lemma `frequency`,
  `distribution`, `cooccurrence`. Hits render in service-returned order; an
  explicit empty state replaces blank panels.
- **GraphView** — live *neighbors* and *path* queries against
  `graphNeighbors`/`graphPath`; frozen-payload lines and ADR-0217 truncation
  banner (`truncated: <reason>`) preserved verbatim.
- **RagDebugView** — typed unavailability (D-11) kept as a deliberate "sealed
  vault" panel; text matches `/not configured|unavailable/i` with zero
  `.rag-result` elements.
- **ExplorerView** — all 7 request presets, method toggling, editable path +
  JSON body, raw status transcript, verbatim failures.
- **CitationView** — server-side verification banner reading
  "verified at the exact source location", URN/deep-link echoed verbatim
  (`[data-urn]`, `[data-deep-link]`), copy control.
- **ReadingView** — byte-exact canonical column (`.ayah-unit` /
  `.ayah-canonical`, `[dir="rtl"][lang="ar"]`), three translation modes ×
  three inspector modes via `[data-translation-mode]` /
  `[data-inspector-mode]`, gold medallions, empty-translation notice.
- **Router** — routes, `parseReadPath`, `parseCitationId`, engine-failed-
  closed alert, citation deep-link wiring unchanged.

Contrast fix found during screenshot QA: the `.dark` layer-token overrides
(`--layer-canonical-*`, `--layer-translation-*`, `--layer-annotation-*`) were
inside `@layer components` and lost to the unlayered `:root` defaults in
`tokens/layers.css`, leaving dark-mode canonical strips white. They now live
unlayered in `index.css` (cascade: unlayered beats layered regardless of
order).

## Shell

shadcn `ui/sidebar` is kept only for collapse/rail/mobile mechanics, restyled
through sidebar tokens; nav is folio-numbered (1.1, 2.1 …) with a "Local-first"
footer note. Header: breadcrumb (`section ⁄ page` + reference chip), ⌘K
command dialog, light/dark toggle (next-themes, class dark, localStorage
`theme`).

## Build & test results

- `npm run build` — success (chunk-size warning only, pre-existing).
- `npx vitest run` — **6 files / 19 tests passing**, covering byte-exact
  canonical rendering + RTL, all translation/inspector modes, service-order
  hits, explicit empty states, truncation banner, URN/deep-link verbatim echo,
  citation id parsing, and RAG unavailability contract.
- Zero `.rag-result` at rest; sticky test contracts unchanged.

## Screenshots (`docs/ui-redesign/screenshots/`)

- Index / research / graph / rag-debug / explorer / reader-deep-link — light
  **and** dark (12).
- Mobile (390×844): index / research / explorer (3).
- `reader-live-{light,dark}.png` — reader rendered from a **mocked API**
  (Playwright `page.route` on `/api/v1/quran/ayahs/**` and
  `/api/v1/quran/citations/**`): shows real verse typography (Amiri, RTL),
  medallions, inline translations, canonical-layer labels, and the verified
  citation banner end-to-end.

Captured with Python Playwright + system Chrome against `vite preview`
(port 4199).

## Dependencies

- Added in pass 1 and kept: `@fontsource` inter / jetbrains-mono / amiri /
  noto-naskh-arabic; `framer-motion` installed and unused — safe to remove.
- Dropped in pass 2: `legacy.css`.

## Known limitations

- Command dialog supports navigation + one sample deep link (search-a-index
  wiring awaits an index endpoint).
- Graph/RAG panels render frozen/absent states without a running backend —
  by design; live queries fire when an engine is reachable.
- framer-motion remains an unused dependency.
