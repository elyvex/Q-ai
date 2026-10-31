# UI Design Specification - Q-ai Research Workspace Redesign

## 1. Design Philosophy

**"Calm Authority"** - A scholarly research environment that conveys trustworthiness, precision, and intellectual depth. The interface should feel like a premium research tool (inspired by Linear, Raycast, Notion) while remaining distinctly appropriate for Quranic and Islamic scholarship.

Core principles:
- **Provenance-first**: Every layer of content (canonical, translation, annotation, computed) is clearly distinguished
- **Exactness**: Canonical text is never altered - UI must respect and highlight this
- **Clarity over decoration**: Information density appropriate for research without clutter
- **Arabic-native**: First-class support for Arabic script, RTL layouts, proper diacritics
- **Local-first**: Subtle indicators of local workspace nature
- **Keyboard-first**: Command palette and shortcuts where sensible

## 2. Visual Direction

Blend Template (1)'s polish + Template (2)'s domain appropriateness + Template (0)'s scholarly warmth.

### Aesthetic Keywords
- Scholarly, refined, authoritative
- Clean surfaces with subtle depth
- Warm neutrals with purposeful color accents
- Restrained motion
- High information density without feeling cramped

## 3. Color System — "Illuminated Codex"

### Light Mode (warm parchment)
```css
/* Foundation — iron-gall ink on parchment */
--background: oklch(0.982 0.01 88);
--foreground: oklch(0.245 0.018 76);
--card: oklch(0.995 0.004 90);              /* vellum panel */
--card-foreground: oklch(0.245 0.018 76);

/* Interactive — ink buttons, teal focus */
--primary: oklch(0.265 0.02 76);
--primary-foreground: oklch(0.98 0.008 88);
--secondary: oklch(0.952 0.014 86);
--accent: oklch(0.945 0.02 86);
--muted: oklch(0.952 0.014 86);
--muted-foreground: oklch(0.52 0.02 80);

/* Borders & Inputs — hairlines */
--border: oklch(0.905 0.016 84);
--input: oklch(0.9 0.016 84);
--ring: oklch(0.5 0.07 168);

/* Status */
--destructive: oklch(0.52 0.19 27);

/* The bound spine — the sidebar stays ink in both modes */
--sidebar: oklch(0.243 0.018 76);
--sidebar-foreground: oklch(0.925 0.012 84);
--sidebar-primary: oklch(0.78 0.12 85);     /* gold mark */
```

### Provenance Colors (Critical — structural, not decorative)
```css
/* Canonical layer — teal-green */
--canon: oklch(0.5 0.103 168);
--canon-soft: oklch(0.955 0.028 168);

/* Translation layer — indigo */
--trans: oklch(0.45 0.112 262);
--trans-soft: oklch(0.953 0.02 262);

/* Annotation layer — amber */
--annot: oklch(0.55 0.117 60);
--annot-soft: oklch(0.968 0.033 78);

/* Computed/Derived (AI) — violet */
--comp: oklch(0.5 0.14 300);
--comp-soft: oklch(0.956 0.023 300);

/* Canonical ornament — gold (medallions, ticks, active segments) */
--gold: oklch(0.62 0.11 82);
--gold-soft: oklch(0.955 0.03 88);
```

### Dark Mode (warm charcoal)
```css
--background: oklch(0.172 0.012 78);
--foreground: oklch(0.925 0.014 84);
--card: oklch(0.208 0.012 78);
--primary: oklch(0.92 0.012 85);
--border: oklch(1 0 0 / 9%);
--ring: oklch(0.68 0.08 168);
/* Provenance values lifted one hue step: e.g. --canon: oklch(0.72 0.11 168),
   --gold: oklch(0.8 0.12 85), --gold-soft: oklch(0.3 0.035 85). */
```

**Layer tokens (D-08/D-09):** `tokens/layers.css` defines the `:root`
light defaults consumed by `.ayah-canonical` / `.ayah-translation` /
`.ayah-annotation`. The `.dark` overrides for `--layer-*-fg/bg/accent` live
**unlayered** in `index.css`: in the cascade, unlayered declarations outrank
`@layer` rules regardless of source order, so the previous layered overrides
never applied (canonical strips stayed white in dark mode).

## 4. Typography

### Font Stack
```css
--font-sans: 'Inter', ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
--font-heading: 'Inter', var(--font-sans);
--font-arabic: 'Amiri Quran', 'Amiri', 'Noto Naskh Arabic', 'Traditional Arabic', serif;
--font-mono: 'JetBrains Mono', ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
```

### Scale
| Style | Size | Weight | Line Height | Letter Spacing |
|-------|------|--------|-------------|----------------|
| Display | 2.25rem - 2.5rem (36-40px) | 600-700 | 1.1-1.2 | -0.02em |
| H1 | 1.875rem (30px) | 600 | 1.2 | -0.02em |
| H2 | 1.5rem (24px) | 600 | 1.25 | -0.01em |
| H3 | 1.25rem (20px) | 600 | 1.3 | -0.01em |
| H4 | 1.125rem (18px) | 600 | 1.4 | -0.01em |
| Body Large | 1rem (16px) | 400 | 1.7 | 0 |
| Body | 0.875rem-0.9375rem (14-15px) | 400 | 1.6-1.7 | 0 |
| Small | 0.8125rem (13px) | 400-500 | 1.5 | 0 |
| Caption | 0.75rem (12px) | 500 | 1.5 | 0 |
| Label/Micro | 0.6875rem (11px) | 600 | 1.4 | 0.08em (uppercase) |

### Arabic Typography
- **Verse Text**: 1.75rem - 2.25rem, weight 400, line-height 2.0-2.3, RTL direction
- **Word Tokens**: Interactive with generous padding, proper diacritics support
- **Translations**: LTR, consistent with body text

## 5. Spacing & Layout

### Spacing Scale (4px base)
0, 0.5, 1, 1.5, 2, 2.5, 3, 4, 5, 6, 8, 10, 12, 16, 20, 24 (in rem units)

### Border Radius
```css
--radius-sm: 0.375rem;   /* 6px */
--radius: 0.5rem;        /* 8px */
--radius-md: 0.625rem;   /* 10px */
--radius-lg: 0.75rem;    /* 12px */
--radius-xl: 1rem;       /* 16px */
--radius-2xl: 1.25rem;   /* 20px */
--radius-3xl: 1.5rem;    /* 24px */
```

### Shadows
```css
--shadow-sm: 0 1px 2px 0 rgb(0 0 0 / 0.02);
--shadow: 0 1px 3px 0 rgb(0 0 0 / 0.03), 0 1px 2px -1px rgb(0 0 0 / 0.03);
--shadow-md: 0 4px 6px -1px rgb(0 0 0 / 0.04), 0 2px 4px -2px rgb(0 0 0 / 0.04);
--shadow-lg: 0 10px 15px -3px rgb(0 0 0 / 0.05), 0 4px 6px -4px rgb(0 0 0 / 0.05);
```

### Layout
- **Sidebar Width**: 256px (expanded), 48px icon rail (collapsed)
- **Header Height**: 48px, sticky with translucent backdrop blur
- **Content Max Width**: 1152px (max-w-6xl), generous 8/10-side padding
- **Mobile Breakpoint**: 640px (sm visually densifies chrome), no separate nav page — sidebar folds to a sheet

## 6. Application Shell

```
┌─────────────────────────────────────────────────────────────┐
│ [Sidebar] │ [Header: Breadcrumbs | Search (⌘K) | Actions]   │
│           ├─────────────────────────────────────────────────┤
│           │                                                   │
│           │                 Main Content                     │
│           │                                                   │
└─────────────────────────────────────────────────────────────┘
```

### Sidebar Structure
**Groups:**
1. **Qurʾān** (1.x) - Reader, Research, Knowledge Graph
2. **Instruments** (2.x) - RAG Debug, API Explorer

Items carry folio numbers (1.1, 2.1 …); "Reader" highlights only on a real
`/read/` deep link.

**Footer:** Local-first indicator with provenance note

### Header
- Breadcrumbs
- Command menu trigger (Search... ⌘K)
- Theme toggle
- Notifications (optional)
- User/workspace context

## 7. Component Standards — bespoke primitives

The pass-2 chrome is built on custom primitives in `index.css` (no generic
card/badge shell):

| Primitive | Role |
|-----------|------|
| `.kicker` | Eyebrow label — mono caps, gold tick before |
| `.medallion` | Verse-number ring — gold border + inset halo, mono numeral |
| `.plate` | Raised vellum panel for toolbars/consoles (shadow sparingly) |
| `.toolbar` / `.seg` | Segmented selector band; active segments get a gold inset underline |
| `.ibtn` (`.ibtn-primary`) | Compact ink/outlined action buttons |
| `.chip` | Mono pill for references/versions (gold-soft) |
| `.prodot` | 6px provenance dot, colored by layer |

Empty states: icon + mono hint + hint text — never a blank panel. Loading:
gold pulse dot + "unrolling the page…" / "awaiting dispatch" copy.

## 8. Navigation & Routing

Keep existing routes exactly:
- `/` and `/read/{slug}@{version}/{s}:{a}` (+ `?citation=` deep links) - Reader
- `/research` - Research workspace (9 lexical instruments)
- `/graph` - Knowledge Graph (neighbors / path)
- `/rag-debug` - RAG Debugging
- `/explorer` - API Explorer

May add routes if justified by existing functionality, but prefer enhancing existing views.

## 9. RTL/LTR Support

- **Arabic content**: `dir="rtl" lang="ar"` on Arabic containers
- **UI chrome**: `dir="ltr"` always
- **Mixed content**: Handle at component level
- **Fonts**: Ensure Arabic fonts load properly
- **Text alignment**: `text-right` for Arabic, appropriate for translations

## 10. Accessibility

- Semantic HTML
- ARIA labels where needed
- Keyboard navigation
- Focus visible states
- Color contrast meets WCAG AA
- Screen reader friendly
- Respect `prefers-reduced-motion`

## 11. Animation Principles

- **Subtle**: 150-300ms for most transitions
- **Easing**: ease-out for entrances, ease-in for exits
- **Purposeful**: Only animate to provide feedback or guide attention
- **Respect preferences**: `prefers-reduced-motion` disables/reduces animations
- **No gratuitous scroll animations**

## 12. Implementation Priorities

1. **Phase 1**: Design tokens, CSS variables, theme system
2. **Phase 2**: Shell layout (sidebar, header, app structure)
3. **Phase 3**: Core components (Button, Card, etc.)
4. **Phase 4**: Navigation and routing updates
5. **Phase 5**: Redesign existing views (Reader, Research, Graph, etc.)
6. **Phase 6**: Polish, animations, responsive, dark mode
7. **Phase 7**: Testing and refinement
