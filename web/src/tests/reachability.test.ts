import { existsSync, readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'
import { ROUTES } from '../router'
import { COLORS, ELEVATION, LAYERS, MOTION, TOKENS, TYPOGRAPHY } from '../tokens'

export interface ReachabilityEntry {
  capability:
    | 'Read'
    | 'Search'
    | 'Linguistics'
    | 'Graph'
    | 'Hadith'
    | 'Tafsir'
    | 'Isnad'
    | 'Multi-RAG'
    | 'Comparative Scripture'
    | 'Agents & Sandboxes'
    | 'Source Catalog'
    | 'Jobs'
    | 'Doctor'
    | 'Audit Trail'
    | 'Settings'
  phase: string
  route: string
  aliases: string[]
  status: 'active'
  description: string
}

export const PHASE_CAPABILITIES = [
  'Read',
  'Search',
  'Linguistics',
  'Graph',
  'Hadith',
  'Tafsir',
  'Isnad',
  'Multi-RAG',
  'Comparative Scripture',
  'Agents & Sandboxes',
  'Source Catalog',
  'Jobs',
  'Doctor',
  'Audit Trail',
  'Settings',
] as const

export const REACHABILITY_MATRIX: ReachabilityEntry[] = [
  {
    capability: 'Read',
    phase: 'Phase 2 & 5',
    route: '/',
    aliases: ['/read'],
    status: 'active',
    description: 'Canonical Quran reader with surah/ayah navigation, translations, and layer badges',
  },
  {
    capability: 'Search',
    phase: 'Phase 3 & 5',
    route: '/search',
    aliases: ['/search'],
    status: 'active',
    description: 'Five lexical search modes (exact, normalized, phrase, concatenated, regex)',
  },
  {
    capability: 'Linguistics',
    phase: 'Phase 3 & 5',
    route: '/words',
    aliases: ['/linguistics', '/research', '/words'],
    status: 'active',
    description: 'Morphology, roots, lemmas, word families, frequency, distribution, co-occurrence',
  },
  {
    capability: 'Graph',
    phase: 'Phase 4',
    route: '/graph',
    aliases: ['/graph'],
    status: 'active',
    description: 'Knowledge graph neighbor expansion and shortest-path traversal with provenance',
  },
  {
    capability: 'Hadith',
    phase: 'Phase 6',
    route: '/hadith',
    aliases: ['/hadith'],
    status: 'active',
    description: 'Structured hadith browser, matn/isnad separation, and attributed scholar gradings',
  },
  {
    capability: 'Tafsir',
    phase: 'Phase 6',
    route: '/tafsir',
    aliases: ['/tafsir'],
    status: 'active',
    description: 'Comparative exegesis reader across classical and contemporary mufassirun',
  },
  {
    capability: 'Isnad',
    phase: 'Phase 7',
    route: '/isnad',
    aliases: ['/isnad'],
    status: 'active',
    description: 'Narrator chain paths, rijal biography inspection, and identity review',
  },
  {
    capability: 'Multi-RAG',
    phase: 'Phase 8',
    route: '/rag-debug',
    aliases: ['/rag', '/rag-debug'],
    status: 'active',
    description: 'Multi-index retrieval, hybrid routing decisions, and honest debug trace',
  },
  {
    capability: 'Comparative Scripture',
    phase: 'Phase 9',
    route: '/scripture',
    aliases: ['/scripture'],
    status: 'active',
    description: 'Edition-aware Torah, Tanakh, and New Testament parallel passage research',
  },
  {
    capability: 'Agents & Sandboxes',
    phase: 'Phase 10',
    route: '/tools',
    aliases: ['/agents', '/tools'],
    status: 'active',
    description: 'Deny-by-default tool registry, sandboxed execution, and reproducible checksums',
  },
  {
    capability: 'Source Catalog',
    phase: 'Phase 11 & 12',
    route: '/explorer',
    aliases: ['/management', '/explorer', '/tools'],
    status: 'active',
    description: 'Source catalog inspection, edition manifests, and verification gates',
  },
  {
    capability: 'Jobs',
    phase: 'Phase 1 & 11',
    route: '/explorer',
    aliases: ['/management', '/explorer', '/tools'],
    status: 'active',
    description: 'Leased background job queue inspection and lifecycle dispatch',
  },
  {
    capability: 'Doctor',
    phase: 'Phase 3 & 12',
    route: '/explorer',
    aliases: ['/management', '/explorer', '/tools'],
    status: 'active',
    description: 'Corpus integrity diagnostics and index drift health checks',
  },
  {
    capability: 'Audit Trail',
    phase: 'Phase 1 & 2',
    route: '/citations',
    aliases: ['/management', '/citations', '/explorer'],
    status: 'active',
    description: 'Append-only hash-chained audit verification and citation verdict workbench',
  },
  {
    capability: 'Settings',
    phase: 'Phase 5 & 13',
    route: '/',
    aliases: ['/settings', '/management', '/'],
    status: 'active',
    description: 'Theme mode, typography scale, reading layout, and local preference controls',
  },
]

describe('Feature-to-Route Reachability Matrix (Phases 1-12)', () => {
  it('covers all 15 capabilities across Phases 1-12 with active status', () => {
    expect(REACHABILITY_MATRIX).toHaveLength(PHASE_CAPABILITIES.length)
    for (const capability of PHASE_CAPABILITIES) {
      const entry = REACHABILITY_MATRIX.find((row) => row.capability === capability)
      expect(entry, `Missing capability in matrix: ${capability}`).toBeDefined()
      expect(entry?.status).toBe('active')
      expect(entry?.route.startsWith('/')).toBe(true)
    }
  })

  it('asserts each capability has an active registered route in the routing table', () => {
    const registeredPaths = new Set(ROUTES.map((r) => r.path))
    for (const entry of REACHABILITY_MATRIX) {
      const isReachable =
        registeredPaths.has(entry.route) ||
        entry.aliases.some((alias) => registeredPaths.has(alias))
      expect(
        isReachable,
        `Capability "${entry.capability}" (${entry.route}) is not registered in ROUTES`,
      ).toBe(true)
    }
  })
})

describe('Design System Tokens & Self-Hosted Fonts (Phase 13-01)', () => {
  it('exports light canvas (#FFFFFF), dark obsidian canvas (#0B1020), warm neutrals, and teal accent', () => {
    expect(COLORS.canvas.light).toBe('#FFFFFF')
    expect(COLORS.canvas.dark).toBe('#0B1020')
    expect(COLORS.neutral[100]).toBe('#F5F5F4')
    expect(COLORS.neutral[200]).toBe('#E7E5E4')
    expect(COLORS.neutral[300]).toBe('#D6D3D1')
    expect(COLORS.neutral[400]).toBe('#A8A29E')
    expect(COLORS.neutral[500]).toBe('#78716C')
    expect(COLORS.neutral[600]).toBe('#57534E')
    expect(COLORS.neutral[700]).toBe('#44403C')
    expect(COLORS.neutral[800]).toBe('#292524')
    expect(COLORS.neutral[900]).toBe('#1C1917')
    expect(COLORS.accent.default).toBe('#0F766E')
    expect(COLORS.status.success).toBe('#15803D')
    expect(COLORS.status.warning).toBe('#B45309')
    expect(COLORS.status.danger).toBe('#B91C1C')
    expect(COLORS.status.info).toBe('#1D4ED8')
  })

  it('exports typography stacks, elevation radii, motion durations, and grayscale-distinct layers', () => {
    expect(TYPOGRAPHY.fonts.arabic).toContain('Amiri')
    expect(TYPOGRAPHY.fonts.arabic).toContain('Noto Naskh Arabic')
    expect(TYPOGRAPHY.fonts.sans).toContain('Inter')
    expect(TYPOGRAPHY.fonts.mono).toContain('JetBrains Mono')
    expect(ELEVATION.radii.sm).toBe('6px')
    expect(ELEVATION.radii.md).toBe('8px')
    expect(ELEVATION.radii.lg).toBe('10px')
    expect(ELEVATION.radii.full).toBe('9999px')
    expect(MOTION.durations.fast).toBe('100ms')
    expect(MOTION.durations.normal).toBe('150ms')
    expect(MOTION.durations.slow).toBe('250ms')

    // Distinct grayscale cues (ruleStyle and badgeStyle differ across all 3 layers)
    const ruleStyles = new Set([
      LAYERS.canonical.ruleStyle,
      LAYERS.translation.ruleStyle,
      LAYERS.annotation.ruleStyle,
    ])
    expect(ruleStyles.size).toBe(3)
    expect(TOKENS.layers.canonical.cssPrefix).toBe('--layer-canonical-')
  })

  it('verifies CSS token files and self-hosted WOFF2 font assets with recorded license entries', () => {
    const webRoot = resolve(__dirname, '../..')
    const repoRoot = resolve(webRoot, '..')

    const colorsCss = readFileSync(resolve(webRoot, 'src/tokens/colors.css'), 'utf8')
    expect(colorsCss).toContain('#FFFFFF')
    expect(colorsCss).toContain('#0B1020')
    expect(colorsCss).toContain('#0F766E')

    const layersCss = readFileSync(resolve(webRoot, 'src/tokens/layers.css'), 'utf8')
    expect(layersCss).toContain('--layer-canonical-')
    expect(layersCss).toContain('--layer-translation-')
    expect(layersCss).toContain('--layer-annotation-')

    const motionCss = readFileSync(resolve(webRoot, 'src/tokens/motion.css'), 'utf8')
    expect(motionCss).toContain('prefers-reduced-motion: reduce')

    expect(existsSync(resolve(webRoot, 'public/fonts/amiri-arabic-400-normal.woff2'))).toBe(true)
    expect(
      existsSync(resolve(webRoot, 'public/fonts/noto-naskh-arabic-arabic-400-normal.woff2')),
    ).toBe(true)

    const decisionsDoc = readFileSync(
      resolve(repoRoot, 'docs/05-followups/decisions-needed.md'),
      'utf8',
    )
    expect(decisionsDoc).toContain('amiri-arabic-400-normal.woff2')
    expect(decisionsDoc).toContain('noto-naskh-arabic-arabic-400-normal.woff2')
    expect(decisionsDoc).toContain('OFL-1.1')
  })
})
