---
status: approved
reviewed_at: 2026-11-01
phase: 13
---

# UI Design Specification — Q-ai “Concordance”

Status: **implemented** (`web/src`). This is the contract every current and
future Q-ai screen follows.

## 1. Visual direction

**Concordance** is a working instrument, not a dashboard and not a book.
The page is a sheet of warm white; type does the work; one saturated accent marks
the active instrument. A serif carries the written word (page titles, verse
headings), Inter carries the interface, JetBrains Mono carries references,
counts and raw output. Structure is drawn with hairlines and a margin column,
not with floating cards.

Three rules that decide most questions:

1. **The text is the interface.** No chrome competes with canonical Arabic.
2. **One accent.** Forest green marks interactivity; provenance colours are the
   only other chromatic information and they are always semantic.
Focal point: the page title (Newsreader display) anchors the eye on every
screen; the margin column carries supporting controls and never competes.
Icon-only header controls carry an `aria-label` and never rely on the icon
alone.

## 2. References

No template is the base. From `temp-inspire/` this pass keeps only ideas —
provenance as layout vocabulary, a margin legend, one display size per page, a
command surface — and rejects their shared left-sidebar/parchment/gold
silhouette. Full analysis: `REFERENCE-ANALYSIS.md`.

## 3. Palette

Light — warm white and ink:

| Token | Value | Role |
|---|---|---|
| `--background` | `oklch(0.99 0.004 90)` | page ground (warm white) |
| `--foreground` | `oklch(0.22 0.012 120)` | primary text (ink) |
| `--card` | `oklch(1 0 0)` | control surfaces, reading slab |
| `--muted` | `oklch(0.965 0.006 100)` | quiet fills, code chips |
| `--muted-foreground` | `oklch(0.5 0.012 120)` | secondary text, metadata |
| `--border` | `oklch(0.905 0.005 265)` | every hairline |
| `--input` | `oklch(0.88 0.01 100)` | field stroke |
| `--primary` | `oklch(0.42 0.09 145)` | the instrument accent (forest green) |
| `--primary-foreground` | `oklch(0.99 0.002 90)` | text on accent |
| `--accent` | `oklch(0.95 0.018 145)` | selected row / hover wash |
| `--ring` | `oklch(0.5 0.1 145)` | focus ring |
| `--destructive` | `oklch(0.545 0.2 27)` | failure only |

Split: 60 % warm-white ground, 30 % ink text and rules, 10 % forest-green
accent reserved for interactive states and the active route. Provenance colours are
semantic and never count toward the split.

Dark — warm charcoal (not an inversion):


| Token | Value |
|---|---|
| `--background` | `oklch(0.17 0.006 60)` |
| `--foreground` | `oklch(0.93 0.006 90)` |
| `--card` | `oklch(0.21 0.006 60)` |
| `--border` | `oklch(1 0 0 / 10%)` |
| `--primary` | `oklch(0.72 0.1 145)` (lifted for contrast on dark) |
| `--accent` | `oklch(0.28 0.03 145)` |
| `--code-bg` | `oklch(0.14 0.006 60)` |

### Provenance (semantic, never decorative)

| Layer | Light token | Meaning |
|---|---|---|
| Canonical | `--canon` `oklch(0.49 0.083 192)` verdigris | served verbatim |
| Translation | `--trans` `oklch(0.51 0.108 312)` violet-ink | attributed, separate |
| Annotation | `--annot` `oklch(0.575 0.108 68)` ochre | lexical / editorial notes |
| Computed | `--comp` `oklch(0.525 0.13 355)` madder | derived or AI-generated |

Each has a `*-soft` companion for the rare filled state. The layer contract in
`tokens/layers.css` is unchanged: canonical text may only carry the canonical
token; the layer backgrounds default to `transparent` so a translation is marked
by an accent rule and label, not a tinted block.

## 4. Typography

```css
--font-serif:  "Newsreader Variable", Newsreader, "Iowan Old Style", Georgia, serif;
--font-sans:   Inter, ui-sans-serif, system-ui, …;
--font-arabic: Amiri, "Scheherazade New", "Noto Naskh Arabic", "Geeza Pro", serif;
--font-mono:   "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, monospace;
```

| Role | Class / size | Notes |
|---|---|---|
| Label / caption | `.kicker`, `.chip`, `.medallion` — 0.625 rem | mono, uppercase caps; 1 px steps within tier permitted |
| Body / UI | `.btn`, `.field`, `.seg` — 0.875 rem | Inter; the working size (line-height 1.6) |
| Translation | `.ayah-translation` — 0.9375 rem | 1 px above body for attribution |
| Arabic / display | `.ayah-canonical`, `.display` — 1.75 → 2.25 rem | Amiri for Arabic, Newsreader for titles; one scale, RTL |

Newsreader carries titles and the wordmark only — never UI controls. No font
size below 0.625 rem, and nothing is letter-spaced except mono caps. All sizes
are one of four declared tiers; intermediate steps stay within a tier.

Documented exceptions (density or platform, never style):
- Search/research hit snippets render Arabic at `text-lg` (1.125 rem,
  `leading-[1.95]`) — list density forbids the 1.75 rem ayah tier in a ruled
  result row; the snippet is a finding aid, not a reading text.
- The reader size stepper previews `sm/md/lg` with `text-xs/text-base/text-xl`
  "A" glyphs — the glyphs demonstrate the sizes, they are not a type tier.
- Vendored `input`/`textarea` primitives use `text-base` falling back to
  `md:text-sm` — the 16 px base suppresses iOS focus zoom; the `.field` atom
  itself stays 13 px.

## 5. Spacing scale

Tailwind 4 px base; the interface uses `4, 8, 16, 24, 32, 48 px`
(and 64 for page gutters on `≥ 1280`). Section spacing is `mt-8` (32 px)
after a page header; block spacing inside a section is 16–24 px; page padding
is 16 px mobile, 24 px tablet, 32 px desktop. Hairline offsets of 1–2 px are
permitted for border alignment only and are not spacing.

## 6. Layout

- Shell: full-width 56 px top rail, content measured at `max-width: 86rem`
  (`1376 px`), footer colophon pinned by a flex column.
- Content lead measure: `max-w-2xl` for headers, `max-w-[86rem]` for workspaces.
- Two-column workspaces: `lg:grid-cols-[13rem_minmax(0,1fr)]` (instrument index)
  and `lg:grid-cols-[minmax(0,1fr)_16rem]` (margin column).
- The margin column is the only place a `border-l` is used at desktop; on mobile
  it follows the content in document order.
- Reading page: text column + 16 rem margin (display controls, layer legend,
  colophon note). Canonical slabs are full-column width so long verses breathe.
- Reader landing: deep-link form + workspace directory listing the four real
  sibling routes — no invented destinations.

## 7. Navigation

Horizontal, because the product has nine destinations:

- Left: 28 px forest-green monogram (ق), `Q-ai` in Newsreader, “CONCORDANCE” in mono
  caps from `lg`.
- Links: 13 px, muted → ink on hover; the active route carries a 2 px forest-green
  rule inset from the link, `aria-current="page"`.
- Right: command trigger (`Search ⌘K` from `sm`), theme toggle, and on `<md` a
  menu button that expands an in-flow navigation panel (`aria-expanded`,
  `aria-controls`).
- Reader is active on `/` and on every `/read/*` deep link.
- A skip link precedes the header.

## 8. Components

Bespoke atoms in `index.css`, composed in views:

Enumerated by `rg -rn 'components/ui/' web/src` — 6 primitives
(`button`, `command`, `dialog`, `input`, `input-group`, `textarea`) plus 14
shared components — `web@0.0.0` — 2026-11-01. Bespoke manual design system;
no third-party registries. Treat as a known-good set, not a closed allowlist.

| Class | Use |
|---|---|
| `.kicker` | mono section eyebrow |
| `.display` | serif page title |
| `.sheet` | bordered control surface (rare) |
| `.band` | ruled block that belongs to the page |
| `.margin-note` | 12 px explanatory text in the margin column |
| `.btn` `.btn-primary` `.btn-quiet` `.btn-sm` | actions, 32/26 px tall |
| `.field` | every input / textarea |
| `.seg` | segmented display-mode control |
| `.chip` | mono reference / status chip |
| `.prodot` | provenance dot |
| `.medallion` | ruled verse index |
| `.carbon` | raw transcript surface |

Shared React components: `PageHeader`, `EmptyState`, `ErrorNote`, `LoadingLine`,
`AppShell` + `SiteHeader`, `CommandMenu`, `DisplayModeControl` (owns the
`TranslationMode` / `InspectorMode` types), `AyahColumn`, `TranslationPanel`,
`CitationCopy`. There is no word-token component: the ayah endpoint ships no
word payload, so the reader shows an honest inspector standby instead of a
placeholder token.

## 9. Buttons and interaction states

- Default: card surface, hairline, ink label. Hover raises to `--muted` and
  `--input`; disabled drops to 50 %.
- Primary: solid forest green with white label; hover mixes 12 % black. Exactly one
  primary action per region (run, dispatch, open).
- Quiet: transparent, muted label; used for icon-only header controls.
- Focus: 2 px `--ring` outline at 2 px offset, always visible, never removed.
- Pressed segments use the accent wash plus a 25 % accent border — no shadows.

## 10. Forms

`.field` is 32 px tall, 5 px radius, 13 px text, card background, visible focus
ring. Labels are `.kicker`; hints are 12 px muted. Validation errors are
`role="alert"` in `--destructive` directly beneath the field (`aria-invalid`).
JSON bodies and references use mono. Enter submits single-field forms; nothing
submits implicitly on blur.

## 11. Tables, lists, information

- Search hits, graph edges and presets are ruled lists: 1 px dividers on a card
  surface, 14–16 px padding, mono index or reference, content in service order.
- No client-side sorting of API results; order is stated (“service order”).
- Empty states are typed and named (`.kicker` label + one sentence); they never
  render a blank panel. Absence is never disguised as data.
- Counts are mono, tabular, zero-padded where they act as an index.

## 12. Dialogs and overlays

Command palette: centred at one third, 8 px radius, 1 px border, dimmed backdrop
(no glass). Items are 32 px rows with icon + label; groups use mono caps
headings. The palette is the only overlay in the product; navigation and content
stay in the page.

## 13. Themes

`next-themes` with `class` strategy (`defaultTheme: system`, persisted in
`localStorage`). Dark is designed separately: warm-charcoal surfaces, lifted
forest green, 10 % white borders, no shadow tricks. Provenance hues are lifted one
step for legibility. The theme toggle lives in the top rail and announces its
action.

## 14. Breakpoints

| Width | Behaviour |
|---|---|
| `< 640` | single column; horizontal nav collapses to the menu panel; page padding 16 px; margin column follows content |
| `640–767` | search pill appears, secondary metadata visible |
| `768–1023` | horizontal nav appears; two-column only where content allows |
| `≥ 1024` | margin columns and instrument indices become side rails; page padding 32 px |
| `≥ 1280` | same grid at 86 rem measure |

## 15. RTL / LTR and Arabic

- Arabic is never mirrored by a global `dir`; the canonical element itself is
  `dir="rtl" lang="ar"`, translations and metadata stay LTR.
- Arabic keeps Amiri metrics: 2.15–2.2 line height, no letter-spacing, no
  synthetic weights, Uthmani marks untouched.
- Layout uses logical spacing (`gap`, `ps/pe` where directional) so a future
  RTL locale does not need mirrored CSS.
- Mixed content (reference + Arabic) is separated into distinct elements, never
  bidi-reordered as one string.

## 16. Accessibility

- Landmarks: header/nav/main/footer; one `h1` per page; skip link first.
- Active navigation uses `aria-current="page"`; toggles use `aria-expanded` /
  `aria-controls`; mode buttons use `aria-pressed`.
- Focus is always visible (2 px ring, 2 px offset); overlays rely on the dialog
  primitive's focus trap and Escape handling.
- Live regions: transcript output (`aria-live="polite"`), loading states
  (`role="status"`), errors (`role="alert"`).
- Contrast: body text ≥ 4.5:1 in both themes; provenance dots are always paired
  with text labels, never colour alone.
- Reduced motion removes transitions and animations globally.

## 17. Motion

120 ms colour/border transitions; no entrance animation, no parallax, no
gradient drift. One pulse is permitted for an unresolved status dot. The
interface should feel instant.

## 18. Reuse strategy

- Tokens live only in `index.css` (`@theme` + `:root` + `.dark`).
- Repeated visual grammar becomes a CSS atom (`.kicker`, `.field`, `.btn`, …),
  not a React wrapper.
- Repeated page structure becomes a component (`PageHeader`, `EmptyState`).
- A view never hard-codes a colour; it uses a token-backed utility or atom.
- New pages start from `PageHeader` + one of the two workspace grids; new data
## 19. Copywriting Contract

Concrete strings the implementation uses; new surfaces follow the same voice
(lowercase, terse, solution-oriented, no emoji).

- Empty states (`.kicker` label + one sentence):
  - Reader, no reference: "No reference is open" / "A reader never invents a
    verse. Open a reference from a citation, the command menu, or the link
    field beside this note."
  - Research, no results: "No matches" / "no results for “{query}” — try a
    different spelling or normalization"
  - Graph, no edges: "No graph lines" / "no graph lines selected — open
    neighbors or trace a path above"
  - RAG, sealed: "sealed" / "RAG not configured / unavailable"
  - Citations, none open: "no citation is open" / "enter a stored citation id
    above — the verdict prints here verbatim"
- Errors (role="alert", solution path): "citation mismatch — the stored quote
  does not match the canonical text — {detail}"; "No edition is loaded for this
  reference — register and activate a Quran edition, then reopen this deep
  link"; deep-link validation "Use /read/<edition>/<surah>:<ayah>".
- CTAs (verb + noun): "Run instrument", "Dispatch", "Open", "Open neighbors",
  "Trace path", "copy citation", "Switch to light/dark theme".
- Destructive actions: none ship in this phase; any future one requires an
  explicit confirmation step stated in its spec.

## 20. Registry Safety

Bespoke manual design system (tokens + atoms in `index.css`); no third-party
registries, no component installs at runtime, no remote CDNs. Every primitive
is vendored source under `components/ui/` and reviewed in-tree. Provenance
recorded in Section 8.
