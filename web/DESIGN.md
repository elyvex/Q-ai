# Design System: Q-ai Research Console

## 1. Visual Theme & Atmosphere

Q-ai presents as a polished, scholarly research environment built for precise textual study. The aesthetic balances modern SaaS professionalism with the gravitas appropriate for scripture scholarship. Clean surfaces, generous spacing, and subtle depth create an interface that feels focused and trustworthy. Soft ambient gradients, refined typography, and thoughtful layering convey precision without sterility. The overall mood is "calm authority" — scholarly, considered, and quietly elegant.

## 2. Color Palette & Roles

### Primary Foundation
- **Workspace Background** `#f5f6f8` — Soft neutral canvas that reduces eye strain during long research sessions
- **Surface** `#ffffff` — Primary card and panel surfaces with subtle elevation
- **Surface Subtle** `rgba(255, 255, 255, 0.92)` — Translucent surfaces for overlays and modals

### Accent & Interactive
- **Brand Primary** `#111827` — Primary actions, emphasis, and key interactive elements
- **Brand Accent** `#047857` — Scholarly green used sparingly for canonical/authentic markers
- **Interactive Hover** `#1f2937` — Hover state for primary actions
- **Focus Ring** `rgba(17, 24, 39, 0.1)` — Subtle focus indicators

### Typography & Text Hierarchy
- **Text Primary** `#111827` — Headings and primary content
- **Text Secondary** `#374151` — Body text and secondary content
- **Text Tertiary** `#6b7280` — Labels, captions, and metadata
- **Text Disabled** `#9ca3af` — Disabled states

### Layer System (Provenance)
- **Canonical** `#047857` / `#ecfdf5` — Authenticated Arabic text and canonical sources (green)
- **Translation** `#2563eb` / `#eff6ff` — Translated content and rendered interpretations (blue)
- **Annotation** `#b45309` / `#fffbeb` — User notes, local annotations, and study marks (amber)
- **Computed/Derived** `#7c3aed` / `#f5f3ff` — AI-generated insights, graphs, and computed relations (violet)
- **Destructive** `#dc2626` / `#fef2f2` — Warnings, deletions, and errors (red)

### Borders & Dividers
- **Border Default** `#e5e7eb` — Standard borders and dividers
- **Border Subtle** `#f3f4f6` — Very light separators
- **Border Strong** `#d1d5db` — Emphasized borders

## 3. Typography Rules

### Font Families
- **Sans-serif (Interface)**: `Inter`, `ui-sans-serif`, system-ui stack — Clean, highly legible for UI and body text
- **Arabic (Script)**: `Amiri Quran`, `Amiri`, `Noto Naskh Arabic`, serif — Traditional, beautiful Arabic rendering for Quranic text
- **Monospace (Code/Data)**: `JetBrains Mono`, `ui-monospace` stack — For citations, references, hashes, and technical data

### Hierarchy & Weights
- **Display/H1**: 2.25rem–2.5rem (36-40px), weight 600, tracking tight (-0.02em)
- **H2**: 1.875rem (30px), weight 600, tracking tight
- **H3**: 1.5rem (24px), weight 600
- **H4**: 1.25rem (20px), weight 600
- **Body Large**: 1rem (16px), weight 400, line-height 1.7
- **Body**: 0.875rem–0.9375rem (14-15px), weight 400, line-height 1.6
- **Small/Caption**: 0.75rem–0.8125rem (12-13px), weight 500, line-height 1.5
- **Micro/Label**: 0.6875rem (11px), weight 600, uppercase, tracking 0.14em

### Arabic Typography
- **Verse Text**: 1.75rem–2.25rem (28-36px), weight 400, line-height 2.2, right-to-left
- **Word-level**: Interactive tokens with generous spacing for selection
- **Diacritics**: Full support, preserved exactly as in canonical text

## 4. Component Stylings

### Buttons
- **Primary**: `bg-gray-900` text-white, rounded-xl (0.75rem), height 2.5rem, subtle shadow
- **Secondary/Outline**: `bg-white` border `border-gray-200`, rounded-xl, hover `bg-gray-50`
- **Ghost**: Transparent background, hover `bg-gray-100`
- **Soft**: Subtle background tints for secondary actions
- **Icon Button**: Square, rounded-lg, consistent sizing

### Cards & Surfaces
- **Default Card**: `bg-white/92` with `border-gray-200`, rounded-2xl (1.1rem), subtle shadow `0 1px 2px rgba(17,24,39,0.04), 0 8px 24px -12px rgba(17,24,39,0.08)`
- **Hover Card**: Lift on hover with stronger shadow and border color change
- **Elevated**: Slight backdrop blur for overlays and modals

### Navigation (Sidebar)
- **Width**: 17rem (272px) on desktop, collapsible
- **Groups**: Uppercase labels with tracking, gray-400 color
- **Items**: Rounded-xl, padding, hover states, active state with subtle background
- **Logo**: Custom mark with subtle gradient, status indicator

### Command Palette (⌘K)
- **Modal**: Centered, max-width-xl, backdrop blur
- **Search Input**: Clean, minimal with keyboard hints
- **Results**: Grouped by type, keyboard navigation highlighted
- **Quick Actions**: Navigation items, search results, references

### Inputs & Forms
- **Default**: Height 2.5rem, rounded-lg/xl, border `border-gray-200`, focus ring
- **Select**: Consistent styling with chevron icons
- **Search**: Icon prefix, clear button

### Badges & Pills
- **Layer Badges**: Color-coded by provenance (canonical/translation/annotation/computed)
- **Status Badges**: Role, state, counts
- **Pills**: Rounded-full for tags and metadata

### Tables
- **Clean**: Subtle borders, hover states, consistent padding
- **Data Tables**: For narrators, sources, audit logs

## 5. Layout Principles

### Grid & Structure
- **App Shell**: Sidebar + main content area with sticky header
- **Max Width**: Content constrained appropriately per view (reader can be wider, forms narrower)
- **Responsive**: Mobile-first with collapsible sidebar

### Spacing Strategy (8pt Grid)
- **Base Unit**: 0.25rem (4px)
- **Common Spacing**: 0.5, 0.75, 1, 1.5, 2, 3, 4, 6, 8, 12, 16 (multiples related to 4px)
- **Section Gaps**: Generous vertical rhythm (1.5rem-2rem between major sections)
- **Component Padding**: Consistent internal spacing

### Whitespace Philosophy
- Generous breathing room for readability
- Cards have comfortable padding (1.25rem)
- Forms have clear field spacing
- Arabic text gets extra vertical space for readability

### Responsive Behavior
- **Desktop (lg+)**: Full sidebar visible
- **Tablet**: Collapsible sidebar
- **Mobile**: Sheet/drawer navigation, stacked layouts

## 6. Design System Notes for Stitch Generation

### Language to Use
- **Overall**: "Scholarly research workspace", "Provenance-first interface", "Calm authority aesthetic"
- **Surfaces**: "Soft white cards with subtle shadows", "Clean backgrounds"
- **Typography**: "Inter for interface, Amiri Quran for Arabic scripture"
- **Interactions**: "Thoughtful micro-interactions", "Keyboard-first navigation"

### Color References
Always reference colors by role: canonical green for authenticated content, translation blue for rendered text, annotation amber for notes, computed violet for derived insights.

### Key Patterns
1. **Provenance Visibility**: Every layer (canonical/translation/annotation) clearly labeled and color-coded
2. **Command-First**: ⌘K for quick navigation everywhere
3. **Arabic-First Reading**: RTL support, proper Arabic typography
4. **Research-Oriented**: Citations, verification, source tracking prominent
5. **Local-First**: Clear indicators of local workspace, no cloud dependency
6. **Tool-Rich**: Comprehensive feature set organized logically

### Component Prompts for Stitch
When generating new screens, emphasize:
- Clean sidebar with grouped navigation
- Top header with breadcrumbs and command search
- Card-based layouts with proper spacing
- Layer badges for content provenance
- Consistent button and input styling
- Arabic text rendering support where applicable
