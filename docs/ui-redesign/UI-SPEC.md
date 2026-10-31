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

## 3. Color System

### Light Mode
```css
/* Foundation */
--background: oklch(0.985 0.002 90);      /* Warm off-white */
--foreground: oklch(0.18 0.01 250);       /* Near-black */
--card: oklch(1 0 0);                     /* Pure white */
--card-foreground: oklch(0.18 0.01 250);
--popover: oklch(1 0 0);
--popover-foreground: oklch(0.18 0.01 250);

/* Interactive */
--primary: oklch(0.25 0.02 250);         /* Deep slate */
--primary-foreground: oklch(0.98 0 0);
--secondary: oklch(0.96 0.005 90);
--secondary-foreground: oklch(0.25 0.02 250);
--accent: oklch(0.96 0.008 90);
--accent-foreground: oklch(0.25 0.02 250);

/* Muted */
--muted: oklch(0.96 0.005 90);
--muted-foreground: oklch(0.5 0.01 250);

/* Borders & Inputs */
--border: oklch(0.9 0.005 90);
--input: oklch(0.9 0.005 90);
--ring: oklch(0.4 0.02 250);

/* Status */
--destructive: oklch(0.55 0.22 25);
```

### Provenance Colors (Critical)
```css
/* Canonical - authenticated Arabic text */
--provenance-canon: oklch(0.45 0.12 160);
--provenance-canon-bg: oklch(0.98 0.02 160);
--provenance-canon-border: oklch(0.9 0.05 160);

/* Translation */
--provenance-trans: oklch(0.5 0.15 250);
--provenance-trans-bg: oklch(0.98 0.015 250);
--provenance-trans-border: oklch(0.9 0.04 250);

/* Annotation */
--provenance-annot: oklch(0.55 0.15 60);
--provenance-annot-bg: oklch(0.99 0.02 60);
--provenance-annot-border: oklch(0.92 0.05 60);

/* Computed/Derived (AI) */
--provenance-comp: oklch(0.5 0.18 290);
--provenance-comp-bg: oklch(0.98 0.015 290);
--provenance-comp-border: oklch(0.9 0.04 290);
```

### Dark Mode
```css
--background: oklch(0.12 0.01 250);
--foreground: oklch(0.95 0.005 90);
--card: oklch(0.16 0.01 250);
--card-foreground: oklch(0.95 0.005 90);
--primary: oklch(0.9 0.005 90);
--primary-foreground: oklch(0.18 0.01 250);
--border: oklch(1 0 0 / 10%);
--muted: oklch(0.22 0.01 250);
--muted-foreground: oklch(0.65 0.01 250);
/* Provenance colors adjusted for dark mode with appropriate opacity */
```

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
- **Sidebar Width**: 256px (expanded), 64px (collapsed icon mode)
- **Header Height**: 64px
- **Content Max Width**: 1200px (default), 1400px (for data-heavy views), full width for reader
- **Mobile Breakpoint**: 768px (md), 1024px (lg)

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
1. **Quran** - Reader, Search, Words/Lexicon, Graph
2. **Research** - Hadith & Tafsir, Comparative Scripture, RAG Projects, Research Agents
3. **Data & Tools** - Sources & Library, Tools & Workflows, API Explorer
4. **System** - Administration, Settings

**Footer:** Local-first indicator with provenance note

### Header
- Breadcrumbs
- Command menu trigger (Search... ⌘K)
- Theme toggle
- Notifications (optional)
- User/workspace context

## 7. Component Standards

### Button Variants
- **default/primary**: Solid background, high emphasis
- **secondary**: Outlined or subtle background
- **ghost**: Transparent, hover background
- **outline**: Border only
- **destructive**: For dangerous actions

### Cards
- Default: `border bg-card rounded-xl shadow-sm p-6`
- Interactive: hover states with subtle shadow increase
- With header: CardHeader, CardTitle, CardDescription, CardContent, CardFooter

### Badges (Provenance)
```tsx
<Badge variant="canon">Canonical</Badge>      // green
<Badge variant="trans">Translation</Badge>   // blue  
<Badge variant="annot">Annotation</Badge>    // amber
<Badge variant="comp">Computed</Badge>       // violet
```

### Empty States
Consistent empty state component with icon, title, description, action button

### Loading States
Skeleton loaders, spinners where appropriate

## 8. Navigation & Routing

Keep existing routes but enhance navigation:
- `/` or `/read/...` - Reader (Quran reading)
- `/search` - Search & Discovery
- `/research` - Research workspace
- `/graph` - Knowledge Graph
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
