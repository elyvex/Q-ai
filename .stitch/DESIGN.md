---
name: Verdant
colors:
  # ---- foundation (light | dark) ----
  background: '#f7f8f6 | #0d1310'
  surface: '#ffffff | #131b17'
  surface-dim: '#f1f4f0 | #1e2a24'
  surface-bright: '#ffffff | #18221d'
  surface-container-lowest: '#ffffff | #0a0f0c'
  surface-container-low: '#fbfcfa | #131b17'
  surface-container: '#f1f4f0 | #18221d'
  surface-container-high: '#eef2ed | #1e2a24'
  surface-container-highest: '#e4e9e4 | #25322b'
  on-surface: '#26352e | #e4ebe6'
  on-surface-variant: '#5c6b60 | #a9b6ad'
  inverse-surface: '#26352e | #0d1310'
  inverse-on-surface: '#f7f8f6 | #e4ebe6'
  outline: '#859089 | #7b8a80'
  outline-variant: '#e4e9e4 | #25322b'
  surface-tint: '#315e46 | #3f8a63'
  # ---- brand ----
  primary: '#315e46 | #3f8a63'
  on-primary: '#ffffff | #0d1310'
  primary-container: '#eaf3eb | #16281e'
  on-primary-container: '#1d4535 | #d9eee3'
  primary-deep: '#244438 | #12211a'
  accent: '#3d7a58 | #6cc494'
  # ---- functional ----
  success: '#2f7d51 | #6cc494'
  on-success: '#ffffff | #0d1310'
  warning: '#a57b38 | #d9b478'
  on-warning: '#ffffff | #0d1310'
  error: '#a15345 | #e08b7c'
  on-error: '#ffffff | #0d1310'
  info: '#3f6d8c | #7fb3d1'
  on-info: '#ffffff | #0d1310'
  # ---- content layers ----
  layer-canonical: '#4b7057 | #8fd0ab'
  layer-translation: '#5f6f7a | #a3b3c0'
  layer-annotation: '#90734b | #d9b478'
  layer-computed: '#76718e | #b3aed4'
typography:
  display:
    fontFamily: DM Sans
    fontSize: clamp(2rem, 3.2vw, 2.375rem)
    fontWeight: '600'
    lineHeight: 1.15
    letterSpacing: -0.035em
  headline:
    fontFamily: DM Sans
    fontSize: 22px
    fontWeight: '600'
    lineHeight: 1.2
    letterSpacing: -0.02em
  body:
    fontFamily: DM Sans
    fontSize: 14px
    fontWeight: '400'
    lineHeight: 1.7
    letterSpacing: '0'
  body-dense:
    fontFamily: DM Sans
    fontSize: 13px
    fontWeight: '400'
    lineHeight: 1.55
    letterSpacing: '0'
  label-caps:
    fontFamily: DM Sans
    fontSize: 11px
    fontWeight: '700'
    lineHeight: 1.4
    letterSpacing: 0.12em
  arabic:
    fontFamily: Amiri
    fontSize: 26px
    fontWeight: '400'
    lineHeight: 2.1
    letterSpacing: '0'
rounded:
  xs: 4px
  sm: 6px
  DEFAULT: 8px
  md: 8px
  lg: 12px
  xl: 16px
  full: 9999px
spacing:
  unit: 4px
  xs: 4px
  sm: 8px
  md: 12px
  lg: 16px
  xl: 24px
  gutter: 16px
  margin-mobile: 16px
  margin-desktop: 24px
motion:
  fast: 130ms
  base: 160ms
  slow: 220ms
  easing: cubic-bezier(0.2, 0.7, 0.2, 1)
---

# Design System: Q-ai — Verdant

**Project ID:** `themes-smpl/green-white`
**Reference implementation:** `themes-smpl/green-white/src/index.css` + `src/AmbientField.tsx`

---

## 0. Provenance — what was taken from where

This system is a consolidation of four AI-generated samples that lived side by
side in `themes-smpl/`. None of them was usable as-is: three of the four
scattered dozens of near-duplicate hex values per file, and none of them had a
dark mode.

| Source | What was harvested |
|:---|:---|
| `complete-phase-13-roadmap` | The **green/sage token set** — forest-green primary `#315e46`, page canvas `#f7f8f6`, ink `#26352e`, hairline `#e4e9e4`, selection `#d9eee3`/`#1d4535`. The `.ambient-wash` blurred-drift backdrop. The **glass sidebar** (`rgba(250,251,249,.9)` + `blur(22px)`). The **layer colour code** (canonical `#578064`, translation `#717d83`, annotation `#90734b`, derived `#76718e`). Tight 4–9 px radii, 150–160 ms transitions, the dense-console type scale. |
| `complete-phase-13-roadmap (1)` | **Editorial restraint** — DM Sans + Amiri pairing, 1.7–1.9 line-heights, `letter-spacing: -1.4px` display headings, green-tinted 8-digit-alpha shadows (`#405b2c19`), the collapsible sidebar, the `.ambient-background` drifting radial ellipse, and the rotating `.orbital` rings. |
| `complete-phase-13-roadmap (2)` | The **token architecture** (Tailwind v4 `@theme` block, single source of truth) and the **component primitive set** — `.btn` / `.card` / `.input` / `.toggle` / `.nav-item` / `.segmented` / `.spinner` / `.progress` / `.code` / `.layer-*` frames with distinct border treatments. The **canvas constellation background** and the **keyframe vocabulary** (`fade-up`, `pop`, `sheen`, `pulse-ring`, `dash`, `bar-stripe`, `drift-a/b/c`). The `prefers-reduced-motion` kill-switch. |
| `complete-phase-13-roadmap (3) 2` | Byte-identical duplicate of `(2)` — no unique contribution; treated as a confirmation that `(2)`'s token set was the intended one. |

**What was fixed during extraction:** 150+ near-duplicate hex values collapsed
into 20 named tokens; the 8 px–21 px type scale replaced by a 7-step scale; the
`font-weight: 620/650/720` soup replaced by 400/500/600/700; and an automatic
dark mode added where none existed.

**The moving background** — the part worth keeping — is the combination of the
`ambient-wash` blurred orbs (sample 0), the drifting ellipse (sample 1) and the
constellation canvas (sample 2), extended with a **pointer-reactive layer** that
none of the four had: a soft emerald torch that eases toward the cursor, gentle
particle attraction, and links that brighten near the pointer.

---

## 1. Visual Theme & Atmosphere

Verdant is the visual language of a **quiet, well-kept manuscript room**. It is
not a "green theme" in the way a dashboard is a blue theme — the green is
*botanical*, the residue of leaves and pressed paper, and it is applied with
restraint so that the white does the talking. The surface is a very slightly
warm off-white (`#f7f8f6`), never a pure `#fff` page, and every rule on it is a
hairline in a green-tinted grey. The result reads as *paper under a window*
rather than *screen under a lamp*.

The atmosphere is **calm, spacious, and alive at the edges**. Dense data —
verses, chains, provenance, run logs — is allowed to be dense, and is supported
by a 13 px dense type scale, but the chrome around it never competes. Depth
comes from an **ambient field**: a fixed backdrop of blurred emerald orbs, a
faint dot grid masked to the centre, and a constellation of drifting particles
that link up as they pass near one another. It is the only "loud" element and
it lives entirely behind `-z-1`, so it colours the room without ever touching
the text. When the pointer moves, the field answers — a soft torch of light
eases toward the cursor and the nearby constellation links brighten.

In dark mode the same room becomes a **forest at night**. The canvas drops to a
green-tinted near-black (`#0d1310`), not a neutral `#111`; the brand green
*lifts* to `#3f8a63` so it stays legible on dark; the ambient orbs brighten
and shift cooler so the field still glows. Nothing inverts — every dark value
was chosen independently, because an inverted light theme is a design failure.

---

## 2. Color Palette & Roles

### Primary Foundation

| Name | Light | Dark | Role |
|:---|:---|:---|:---|
| **Canvas** | `#f7f8f6` | `#0d1310` | Page background. Warm in light, green-tinted in dark. |
| **Paper** | `#ffffff` | `#131b17` | Cards, panels, raised surfaces. |
| **Paper Soft** | `#fbfcfa` | `#18221d` | Secondary panels, table headers, hovered rows. |
| **Paper Sunken** | `#f1f4f0` | `#1e2a24` | Wells, segmented-control tracks, code chips. |
| **Well** | `#eef2ed` | `#0a0f0c` | Inset regions, recessed areas. |
| **Hairline** | `#e4e9e4` | `#25322b` | The default 1 px border. Nothing heavier unless it means something. |
| **Hairline Strong** | `#d5ddd6` | `#35463b` | Borders that signal interactivity (hover, focus). |
| **Overlay** | `rgba(255,255,255,.88)` | `rgba(19,27,23,.86)` | Frosted topbar / sidebar glass. |

### Accent & Interactive

| Name | Light | Dark | Role |
|:---|:---|:---|:---|
| **Forest** (primary) | `#315e46` | `#3f8a63` | Primary buttons, active nav, brand marks. |
| **Forest Deep** | `#244438` | `#12211a` | Brand tile, sidebar brand mark, solid dark chips. |
| **Forest Hover** | `#274e39` | `#4c9e73` | Primary button hover / pressed. |
| **Forest Ink** | `#1d4535` | `#d9eee3` | Text on a green wash. |
| **Sage Wash** | `#eaf3eb` | `#16281e` | Active nav row, brand-soft chips, badge backgrounds. |
| **Sage Edge** | `#c7dbcd` | `#24483a` | Border of a sage wash. |
| **Leaf** | `#3d7a58` | `#6cc494` | Links, focus border, inline accents. |
| **Selection** | bg `#d9eee3` / fg `#1d4535` | bg `#1d4535` / fg `#d9eee3` | Text selection. |

### Typography & Text Hierarchy

| Name | Light | Dark | Role |
|:---|:---|:---|:---|
| **Ink** | `#26352e` | `#e4ebe6` | Headings and primary body. Green-black, never pure black. |
| **Ink Soft** | `#5c6b60` | `#a9b6ad` | Body copy, descriptions. |
| **Ink Faint** | `#859089` | `#7b8a80` | Labels, meta, placeholders. |
| **Ink Ghost** | `#a3aca5` | `#5c6b61` | Section kickers, disabled, axis labels. |

### Functional States

| Name | Light | Dark | Role |
|:---|:---|:---|:---|
| **Success** | `#2f7d51` | `#6cc494` | Indexing complete, chain verified. |
| **Warning** | `#a57b38` | `#d9b478` | Partial indexing, disputed grading. |
| **Danger** | `#a15345` | `#e08b7c` | Destructive actions, failed runs. |
| **Info** | `#3f6d8c` | `#7fb3d1` | Informational notices. |

### Content Layers (domain vocabulary)

These four are **not** theme colours — they are the structural grammar of the
product and must never be swapped for the brand green.

| Layer | Light | Dark | Frame treatment |
|:---|:---|:---|:---|
| **Canonical** | `#4b7057` | `#8fd0ab` | Solid 4 px left rule, solid border. |
| **Translation** | `#5f6f7a` | `#a3b3c0` | Solid 4 px left rule, slate-grey. |
| **Annotation** | `#90734b` | `#d9b478` | **Double** 4 px left rule, dashed border. |
| **Computed** | `#76718e` | `#b3aed4` | **Dotted** 4 px left rule, dashed border. |

The double/dotted/solid distinction is deliberate: border *style* carries the
layer identity even where the hue cannot.

---

## 3. Typography Rules

### Hierarchy & Weights

Two families, three voices.

- **DM Sans** (fallback Inter → system UI) for every Latin string. Geometric
  but humanist, with a large x-height that survives the 13 px dense scale.
  Weights used: **400** body, **500** UI/medium, **600** headings and buttons,
  **700** kickers only.
- **Amiri** (fallback Noto Naskh Arabic → Scheherazade New → Geeza Pro) for
  every Arabic string, `direction: rtl`, `unicode-bidi: isolate`, line-height
  **2.1**, `font-feature-settings: "liga","calt"`. Canonical text is never
  styled by altering characters — only by changing its frame.
- **JetBrains Mono** (fallback `ui-monospace`) for URNs, hashes, command output
  and the `.code` block.

Scale (7 steps, `rem`):

| Token | Size | Use |
|:---|:---|:---|
| `--text-3xl` | `clamp(2rem, 3.2vw, 2.375rem)` | Page title |
| `--text-2xl` | `1.75rem` | Section title |
| `--text-xl` | `1.375rem` | Large stat, panel title |
| `--text-lg` | `1.125rem` | Card title |
| `--text-md` | `1rem` | Lead paragraph |
| `--text-base` | `0.875rem` | Body |
| `--text-sm` | `0.8125rem` | Dense body, table cells |
| `--text-xs` | `0.75rem` | Labels, badges |
| `--text-2xs` | `0.6875rem` | Micro labels, kickers |

Line-heights: display **1.15–1.2**, card title **1.35**, body **1.7**, dense
body **1.55**, Arabic **2.1**.
Letter-spacing: display `-0.035em` → `-0.02em`; body `0`; kickers
`0.12em` uppercase; micro-labels `0.13em`.

### Spacing Principles

A strict **4 px base unit**. The scale is `4 / 8 / 12 / 16 / 20 / 24 / 32 /
40 / 48 / 64`. Inside a card the rhythm is 12 → 16 → 20; between sections it
is 32 → 48. Gaps inside a row of controls are 8; gaps between controls in a
toolbar are 12. Never mix spacing units within one axis — if a card needs
20 px padding, it needs 20 px on all four sides.

Type spacing sits *inside* the grid, not on it: `margin-top: 8px` between a
kicker and a heading, `10px` between a heading and its body copy, `14px`
between a paragraph and the next control.

---

## 4. Component Stylings

### Buttons

- Height **36 px** (30 px small, 32 px square icon). Radius **8 px**. Horizontal
  padding **13 px** (10 px small). Icon gap **8 px**.
- **Primary** — Forest fill, white text, `0 5px 11px rgba(39,82,57,.18)`
  shadow. On hover it lifts 1 px and darkens; a **sheen** sweep (a 40 %-wide
  white gradient skewed −18°, travelling `translateX(-120%) → 220%` over
  900 ms) crosses it once per hover.
- **Secondary** — Paper fill, hairline border, `--text-2` label; hovers to
  `Paper Soft` with a Sage Edge border.
- **Ghost** — transparent, no border; hovers to `Paper Sunken`.
- **Danger** — soft red wash with a red-tinted border.
- Every state transitions `color / background / border / box-shadow / transform`
  over **160 ms** with `cubic-bezier(.2,.7,.2,1)`. `:active` scales to
  **0.97**. Disabled sits at **46 %** opacity and `cursor: not-allowed`.
- Focus uses the **ring** (`0 0 0 3px rgba(52,126,99,.27)`), never an outline
  colour change.

### Cards & Containers

- Radius **12 px**, 1 px hairline border, `--shadow-1`
  (`0 1px 2px rgba(25,43,31,.05)`). `--card-hover` lifts **2 px**, swaps the
  border to `--border-strong` and escalates to `--shadow-2`.
- The **glass** surface is the frosted variant for sticky chrome: 88 % paper +
  `blur(14px) saturate(1.3)`.
- The `.code` block is deliberately *not* themed: near-black `#0f172a` in light
  and `#0a100d` in dark, with light mono text, radius 12 px.
- Empty states: 280 px minimum height, a 40 px rounded icon tile in
  `Paper Sunken`, a 13 px title and a 10–13 px muted body.

### Navigation

- **Sidebar**: 254 px wide, sticky, `height: 100vh`, `border-right` hairline,
  frosted `rgba(250,251,249,.9)` + `blur(22px)`. Brand row 81 px tall with a
  37 px Forest Deep rounded tile, an 18 px/720 name at `-0.05em`, and an 8 px
  uppercase caption.
- **Nav item**: 36–38 px min-height, radius 8 px, `--text-3` label at 12–13 px.
  Hover → `Paper Sunken` + the icon shifts to Leaf and nudges **1 px right**.
  Active → Sage Wash background, Sage Edge border, Forest text at weight 600.
- **Group titles**: 9 px, weight 700, `0.12em`, uppercase, `--text-4`.
- **Topbar**: 64–69 px, sticky, hairline bottom, frosted. Contains the
  breadcrumb, a 356 px search pill (hairline border, white fill, magnifier
  icon + `⌘K` kbd chip), a status cluster and a 31 px circular avatar that
  lifts 1 px on hover.
- **Mobile**: the sidebar collapses behind a hamburger; the topbar drops its
  status cluster and shrinks the search pill to `flex: 1`.

### Inputs & Forms

- **Height 38 px**, radius **8 px**, 1 px hairline, Paper fill, 14 px text.
  Textareas drop to `padding: 10px 12px`, line-height 1.45, `resize: vertical`.
- Placeholder is `--text-4`. Hover → `--border-strong`. Focus → Leaf border
  **plus** the ring.
- **Select** is a bordered flex wrapper with the chevron absolutely positioned
  at the right and the native `appearance: none`.
- **Toggle**: 38 × 22 px pill, 16 px knob, `cubic-bezier(.3,.7,.3,1.2)` slide,
  Forest when on, `Paper Sunken` when off.
- **Segmented control**: 3 px inset padding, 8 px radius, `Paper Sunken` track,
  active segment becomes Paper with `--shadow-1` and weight 600.
- **Badge**: pill, hairline border, `Paper Soft` fill, 11 px weight 600.

### Layer Frames (domain-specific)

The four content layers each own a frame, a hue and a *border style*:

- **Canonical** — Paper fill, solid Sage Edge border, 4 px **solid** Forest rule.
- **Translation** — soft slate fill, hairline border, 4 px **solid** slate rule.
- **Annotation** — warm parchment fill, **dashed** amber border, 4 px
  **double** amber rule.
- **Computed** — pale violet fill, **dashed** violet border, 4 px **dotted**
  violet rule.

Each frame is 12 px radius. A layer label sits inside the frame as an 11 px
uppercase kicker with a 6 px coloured dot. This is the single most important
rule in the system: canonical text may only ever appear inside a canonical
frame, and a translation may never receive a canonical token.

### The Ambient Field (the moving background)

Four stacked layers, all behind `-z-1`, all `pointer-events: none`:

1. **Base wash** — three radial gradients (emerald at 88 %/3 %, sage at
   57 %/79 %) over a 145° white-to-white linear gradient.
2. **Orbs** — three 36 rem blurred (90 px) circles at 16–28 % opacity drifting
   on `drift-a` (26 s), `drift-b` (32 s) and `drift-c` (38 s), each combining a
   60 px translate with a 0.95–1.12 scale.
3. **Dot grid** — 1 px dots on a 26 px lattice at 50 % opacity, masked to a
   radial ellipse so it fades out at the edges.
4. **Constellation canvas** — up to 70 particles (density = `area / 22000`),
   each drifting at ±0.25 px/frame, bouncing off the edges, drawn as 0.6–2 px
   dots at 55 % alpha. Any two particles within **140 px** are joined by a 1 px
   line whose alpha falls off linearly to zero.

**The pointer layer** (the addition none of the samples had):

- A soft emerald **torch** — a 280 px radial gradient composited with
  `lighter` — follows the cursor, easing at **8 % per frame** so it trails
  rather than snaps.
- Particles within **220 px** of the pointer are gently pulled toward it
  (0.06 impulse, speed-capped at 0.9) — the constellation seems to lean in.
- Links within **180 px** of the pointer brighten by up to `+0.3` alpha.

`prefers-reduced-motion: reduce` freezes the constellation to a single static
frame, disables the torch, and reduces every CSS animation to 0.001 ms.

---

## 5. Layout Principles

### Grid & Structure

- **Sidebar 254 px** (230 px collapsed, 76 px icon-only) + **main column**
  `flex: 1`.
- Content max-width **1440 px**, centred, with page padding of **24–38 px**
  horizontally.
- The reader is a two-column grid: `minmax(0, 1fr) 246px` with a **44 px**
  gutter — the primary column plus a 246 px inspector rail. Below 960 px the
  rail drops and the grid becomes single-column.
- Verses are a `34px minmax(0, 1fr)` grid — number gutter, then content —
  with 16 px column gap and 22/20 px vertical padding, separated by hairlines.
- Sidebar sub-groups use a 3 px gap; a nav group is separated from the one
  above by a 1 px rule and 14–17 px of space.

### Whitespace Strategy

- 4 px base unit, 8 px sub-grid.
- Section rhythm: 24 px inside a panel, 32–48 px between sections, 43 px before
  the footer rule.
- Card interior: 15–20 px padding; a record header row is 15/17 px with a
  3 px gap between the source name and its meta.
- The one exception to "airy": the **data console** (management, runs, audit)
  is intentionally dense — 9–11 px type, 3–8 px gaps. Density is a mode, not a
  default; the reader and research views stay at 13–16 px.

### Alignment & Visual Balance

- Page titles are **left-aligned** and never centred; a centred hero reads as
  marketing, and this is a tool.
- The inspector rail's content is left-aligned but its *kickers* are uppercase
  with `0.115em` tracking, creating a vertical rhythm of tiny anchors.
- Arabic is **right-aligned** with `word-spacing: 2 px`; Latin body copy is
  left-aligned. The two never share a column.
- Icon buttons are square; text buttons are content-width. Never both.

### Responsive Behavior & Touch

Breakpoints: **1180** (rail narrows to 225 px), **960** (rail collapses),
**820** (topbar compresses), **720** (sidebar becomes a drawer, page padding
drops to 16 px), **640** (segmented controls and stacked toolbars).

Touch targets are **≥ 32 px** in the console chrome and **≥ 38 px** for
inputs; the toggle is 38 × 22 px with a 38 × 38 px invisible hit area.

---

## 6. Design System Notes for Stitch Generation

### Language to Use

"Quiet, paper-like, botanical. Warm off-white paper, forest-green ink, hairline
rules the colour of dried sage. Deep shadow, generous breathing room in the
reading views, dense and instrumental in the console views. A living
constellation drifts behind everything — slow, soft, never distracting. White
and green, and nothing else competing for attention."

Avoid: "vibrant", "gradient mesh", "neon", "glassmorphism hero", purple,
neon-glow, heavy drop shadows, pill-shaped everything.

### Color References

- Paper — `#f7f8f6` page, `#ffffff` card (dark: `#0d1310` / `#131b17`)
- Forest — `#315e46` primary, `#274e39` hover, `#244438` deep (dark:
  `#3f8a63` / `#4c9e73` / `#12211a`)
- Sage wash — `#eaf3eb`, edge `#c7dbcd` (dark: `#16281e` / `#24483a`)
- Leaf — `#3d7a58` links and focus (dark: `#6cc494`)
- Ink — `#26352e` / `#5c6b60` / `#859089` / `#a3aca5`
  (dark: `#e4ebe6` / `#a9b6ad` / `#7b8a80` / `#5c6b61`)
- Hairline — `#e4e9e4`, strong `#d5ddd6` (dark: `#25322b` / `#35463b`)
- Layers — canonical `#4b7057`, translation `#5f6f7a`, annotation `#90734b`,
  computed `#76718e`
- Status — success `#2f7d51`, warning `#a57b38`, danger `#a15345`, info `#3f6d8c`

### Component Prompts

1. **Reading view** — "A calm scripture reader on warm off-white paper. A
   254 px frosted left sidebar of hairline-ruled navigation with a deep forest
   brand tile. A centred column of verses: a small circled verse number in a
   34 px gutter, right-aligned Amiri Arabic at a 2.1 line-height inside a
   canonical frame with a solid 4 px green left rule, then a slate translation
   beneath. A 246 px inspector rail on the right. Behind everything, a slow
   emerald constellation and blurred sage orbs."

2. **Console view** — "A dense operational console in the same palette:
   9–11 px type, 4 px grid gaps, hairline-bordered tables, a 36 px Forest
   primary button with a sheen sweep on hover, Sage Wash active states, and
   6 px status dots with a soft pulse ring. Still paper-white and forest
   green — the density comes from spacing, not from a darker surface."

3. **Ambient backdrop** — "A full-viewport living backdrop: three enormous
   blurred emerald orbs drifting on 26/32/38 second loops, a 26 px dot lattice
   masked to a central ellipse, and a constellation of 70 drifting particles
   that link when they pass within 140 px. A soft torch of emerald light eases
   toward the cursor and the nearby links brighten."

### Incremental Iteration

- Add a screen by composing the existing primitives. Do **not** introduce new
  colours, radii or type sizes — if a value is missing, it means the layout is
  wrong, not the token set.
- Any new accent must be registered as a named token with a light/dark pair
  before it is used once.
- Every colour decision must survive the dark mode. If a hue has no dark
  counterpart, it is not a colour yet.
- New motion must respect `prefers-reduced-motion` and must not exceed 400 ms
  for UI feedback, or 220 ms for the ambient field.
- New layer-like distinctions must be expressed in **border style**, not only
  in hue — solid for canonical and translation, double for annotation, dotted
  for computed.
