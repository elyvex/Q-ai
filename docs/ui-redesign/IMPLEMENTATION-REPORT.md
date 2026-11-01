# UI Redesign — Implementation Report (pass 3, “Concordance”)

## 1. What was redesigned

The frontend in `web/` was reworked from the shell down, not recoloured.

**New application frame.** The left sidebar is gone. Navigation is a single
56 px top rail — monogram, Newsreader wordmark, five horizontal destinations
with an inset indigo active rule, command trigger, theme toggle, mobile panel —
followed by one 86 rem content measure and a pinned colophon footer. The
navigation information architecture is unchanged (Reader, Research, Graph,
RAG Debug, Explorer); only its shape changed.

**New visual language (“Concordance”).** Cool porcelain ground, iron-ink text,
one deep indigo accent, 5–8 px radii, hairline structure, and a true margin
column instead of a card grid. Serif (Newsreader) for page titles, Inter for
interface, JetBrains Mono for references/counts, Amiri for canonical Arabic.
No parchment, no gold, no forest green, no glass, no gradients.

**Every existing route was rebuilt around that language:**

| Route | Redesign |
|---|---|
| `/` (Reader landing) | Header + explanation + a working deep-link form (validates `/read/<edition>/<s>:<a>` and navigates through the existing route). No invented data. |
| `/read/...` | Two-column scholarly page: ruled verse sections (no cards), mono index + reference, canonical label, translation marked by a violet rule, margin column with display modes, layer legend and typographic note. Citation verification banner retained. |
| `/research` | Instrument index as a numbered vertical rail; query band; ruled result list preserving service order; typed empty state; count chip. |
| `/search` | Five search modes (exact/normalized/phrase/concatenated/regex) over the typed engine; ruled result list. |
| `/hadith` | Fetch by collection + number, or search a collection; verbatim transcript. |
| `/tafsir` | Fetch a work by slug, or search across works; verbatim transcript. |
| `/scripture` | Editions, search, parallel, research; verbatim transcript. |
| `/graph` | Ruled query bench, verbatim edge ledger, truncation banner, “reading the ledger” margin notes. |
| `/rag-debug` | Intentional sealed state, restated typographically with an explanation column. No retrieval path added. |
| `/explorer` | Preset index + request bench + carbon-copy transcript. All 7 presets, method toggle, editable path/body preserved. |

**Shared components added:** `PageHeader`, `EmptyState`, `ErrorNote`,
`LoadingLine`, plus `SiteHeader` rewritten and `AppShell` reduced to frame +
footer. Tokens moved to a single semantic set in `index.css`.

**Two latent defects found and fixed while testing in a browser:**

- The command palette crashed on open: `CommandDialog` rendered its children
  without the `cmdk` `<Command>` context, so `subscribe` was undefined. Fixed in
  `components/ui/command.tsx`; the ⌘K palette now opens and navigates.
- Dark-mode translations rendered as full-width violet bars because
  `--layer-translation-bg` was set to a solid dark tint. Layer backgrounds are
  now `transparent`; separation comes from the accent rule and label.

## 2. Which reference templates influenced the result

None is a base. Influences are ideas, re-expressed:

- **`complete-phase-13-roadmap (2)/(3)`** — provenance as a structural
  vocabulary (four layers, dots, labels, mono references). Adopted as semantics,
  not as tinted cards; their cool-grey canvas is the nearest palette cousin,
  which is why the final accent was moved to indigo and the ground to porcelain.
- **`complete-phase-13-roadmap (0)`** — the margin legend beside the primary
  text and the calm density of a reading page.
- **`complete-phase-13-roadmap (1)`** — one confident display size per page,
  command surface, and mobile drawer behaviour.
- **Rejected:** the shared left sidebar, forest-green/cream/parchment palette,
  gold ornament, card-per-item lists, ambient orbs, and landing-page hero
  compositions that imply features the API does not have.

Details and runtime findings (two references crash on mount) are in
`REFERENCE-ANALYSIS.md`.

## 3. Skills and tools actually used

- **shadcn/ui skill** — informed primitive usage and the fix to the command
  dialog; the project keeps its existing Base UI + Tailwind v4 setup.
- **design-md / stitch skills** — reviewed for a design-system contract, but
  **Stitch MCP is not configured in this environment** (no Stitch tools were
  available), so no screenshots or design systems were generated in Stitch. The
  contract was written directly from the implemented tokens.
- **GSD UI workflows** — not installed as invocable commands here; the review
  was done manually with Playwright screenshots.
- **Playwright (system Chrome)** — reference capture, live app capture, mobile
  viewport, dark mode, command palette and mobile navigation checks.

## 4. New or modified components

New: `components/layout/page-header.tsx`, `components/states.tsx`.

Rewritten: `components/layout/app-shell.tsx`,
`components/layout/site-header.tsx`, `components/command/command-menu.tsx`
(copy + wrapper fix), `components/TranslationPanel.tsx`,
`components/CitationCopy.tsx`, `display/DisplayModeControl.tsx`,
`views/ReadingView.tsx`, `views/ResearchView.tsx`, `views/GraphView.tsx`,
`views/RagDebugView.tsx`, `views/ExplorerView.tsx`, `views/CitationView.tsx`,
`router.tsx`.

Removed: `components/navigation/app-sidebar.tsx` (its only consumer was the old
shell). `components/ui/sidebar.tsx` is untouched and currently unused; other
stock shadcn primitives were already unused before this pass.

Tokens/styles: `index.css` rewritten; `tokens/layers.css` backgrounds set to
transparent, colour semantics preserved.

## 5. Dependencies

- **Added:** `@fontsource-variable/newsreader` (self-hosted serif; one package,
  no network font calls).
- **Removed:** none.
- No backend, API contract, schema, Rust toolchain, or reference-template files
  were touched.

## 6. Validation

| Check | Result |
|---|---|
| `npm run build` (`tsc -b && vite build`) | passes; pre-existing chunk-size warning only |
| `npx vitest run` | **6 files / 19 tests passing** — canonical byte-exactness and RTL, translation/inspector modes, service-order hits, explicit empty states, truncation banner, URN/deep-link verbatim, citation parsing, RAG unavailability |
| Browser (Playwright, 1440×1000 + 390×844) | all five routes, reader deep link, citation banner, light + dark |
| Interactions | horizontal nav, active states, mobile nav panel, ⌘K palette, instrument switching, research run with mocked API, method toggle, preset selection, deep-link form validation |
| Accessibility spot checks | skip link, one `h1`, `aria-current`, `aria-expanded/controls`, `aria-pressed`, labelled inputs, focus ring, live regions |

Screenshots: `docs/ui-redesign/screenshots/` — 12 desktop (6 routes × light/dark)
+ 4 mobile (index, research, explorer, reader) + mobile nav + command palette.
Reader screenshots use Playwright route mocks for `/api/v1/quran/ayahs/**` and
`/api/v1/quran/citations/**` because the Rust engine is not running; search
results are mocked for the same reason. Nothing was mocked in production code.

## 8. Enrichment passes (2026-11-01)

Recheck of the shipped pass found five small defects, fixed without changing
any route, contract, or test:

- Display-mode segments wrapped mid-word in the 16 rem margin (`side- /
  by-side`). The mode rows are now 3-column grids with truncated labels;
  visible labels and `aria-label`s are unchanged.
- The reader rendered a fake `•` token through `WordInspector` with no word
  data behind it. Replaced by an honest inspector standby (inline in
  popover/pin, boxed in panel); `components/WordInspector.tsx` deleted and the
  `InspectorMode` type moved into `DisplayModeControl`, its owner.
- The citation banner's filled mint header fought the palette; it is now a
  plain ruled card with the canon accent kept on the label only.
- The research margin note tripled the query hint (placeholder + standby +
  margin); it now states the keyboard contract instead.
- The reader landing is no longer half-empty: a directory lists the four real
  sibling routes with one-line descriptions.

Validation repeated: `npm run build` passes, 19/19 vitest, Playwright
re-capture (reader light/dark/mobile, index, research) plus mode-switch and
no-page-error checks.

Follow-up cleanup, same constraints:

- Dead-code sweep of `components/ui/`: 15 stock primitives with no importer
  deleted (`sidebar`, `sheet`, `tooltip`, `sonner`, `avatar`, `badge`,
  `breadcrumb`, `card`, `dropdown-menu`, `navigation-menu`, `select`, `table`,
  `tabs`, `separator`, `skeleton`). Six remain, all in the command-dialog
  chain (`button`, `command`, `dialog`, `input`, `input-group`, `textarea`).
- `TooltipProvider` removed from `App.tsx` — it wrapped zero tooltips.
- `@fontsource-variable/geist` removed from dependencies (no longer imported
  by `index.css`).
- Verified: build passes, 19/19 vitest, command palette + theme toggle
  smoke-tested in Chrome with zero page errors.

## 7. Known limitations

- The reader landing depends on a valid edition slug; the prefilled
  `/read/test@0.1.0/2:1` only resolves when that fixture edition is loaded.
- Command palette lists navigation plus one sample deep link; there is no
  content index endpoint to search yet.
- RAG, Words, Tools, Citations and Isnad screens render typed empty/error
  states without a running engine — by design, not a regression.

## 9. Gap-closure pass (2026-11-01, waves 8–12, plans 13-10…13-14)

The inventory of unimplemented reference features from the previous session
became five executable plans; all five ran sequentially (shared
`router.tsx` / `site-header.tsx` edits force serialization) and all verify
green.

| Plan | Route(s) | What it added |
|---|---|---|
| 13-10 | `/rag-debug`, `/explorer` | `lib/tool-result.ts` + shared `ToolLedger.tsx`; working `rag.debug`/`rag.query` runner with route-decision panel and `QAI-RAG-0326` denied rows; four 404 presets replaced with dispatcher-tool entries plus an allow-list reachability test |
| 13-11 | `/words` | Morphology/root/lemma/family lookups with canonical deep links; all five `count/*` instruments, each count printed with its `rules` block and checksum |
| 13-12 | `/tools` | Catalogue of all 27 callable tools with registry-exact typed forms; UI/CLI/HTTP surface toggle derived from one param object; both denied families as inert rows |
| 13-13 | (reader) | Three Arabic sizes inside the UI-SPEC §4/§15 bands, three layouts, localStorage prefs/bookmarks, on-demand seeded `tafsir.search` + `hadith.search` related lookup |
| 13-14 | `/citations`, `/isnad` | Verdict/hash/URN/re-verify plus labelled local tamper simulation (`CitationsWorkbenchView`, distinct from `CitationView`); read-only `isnad.trace`/`narrator`/`rijal` with `QAI-ISNAD-0326` rows |

Validation: `npm run build` passes; `npx vitest run` **20 files / 150
tests pass** (was 6/19); all 13 routes load in system Chrome via
`vite preview` with correct `h1` and zero page errors; new screenshots:
`words-light.png`, `tools-light.png`, `citations-light.png`,
`isnad-light.png`, refreshed `rag-debug-light.png`. No `crates/` edits, no
new dependencies, no invented data; denied tools render as disabled rows
with their policy codes.

Pre-existing failure outside this pass: `cargo test -p application
--test layer_separation
spa_non_canonical_components_wear_no_canonical_class` fails because it
`expect("component exists")` on `web/src/components/WordInspector.tsx`,
deleted in `f86be81` before these waves. One-line fix (drop the deleted
path from the array) left undone — `crates/` is off-limits for UI plans.
