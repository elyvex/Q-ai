import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { ROUTES } from '../router'

// vitest runs with cwd at the web package root; import.meta.url is not a
// file: URL under this transform, so resolve assets from the project root.
const asset = (relativePath: string) => resolve(process.cwd(), relativePath)

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

describe('Concordance design contract (single source of truth)', () => {
  it('pins the light palette anchors in index.css', () => {
    const css = readFileSync(asset('src/index.css'), 'utf8')
    expect(css).toContain('--background: oklch(0.99 0.004 90)')
    expect(css).toContain('--foreground: oklch(0.22 0.012 120)')
    expect(css).toContain('--primary: oklch(0.42 0.09 145)')
    expect(css).toContain('--accent: oklch(0.95 0.018 145)')
    expect(css).toContain('--ring: oklch(0.5 0.1 145)')
  })

  it('keeps the animated field atom and its keyframes', () => {
    const css = readFileSync(asset('src/index.css'), 'utf8')
    expect(css).toContain('.gradient-field')
    expect(css).toContain('@keyframes gradient-drift')
  })

  it('self-hosts every font family and references no remote origin', () => {
    const css = readFileSync(asset('src/index.css'), 'utf8')
    for (const family of [
      '@fontsource/inter',
      '@fontsource-variable/newsreader',
      '@fontsource/jetbrains-mono',
      '@fontsource/amiri',
      '@fontsource/noto-naskh-arabic',
    ]) {
      expect(css).toContain(family)
    }
    const remote = css.match(/(@import|url\()\s*["']?https?:/)
    expect(remote, 'index.css must not reference a remote origin').toBeNull()
  })

  it('declares all three provenance layer families in layers.css', () => {
    const layers = readFileSync(asset('src/tokens/layers.css'), 'utf8')
    expect(layers).toContain('--layer-canonical-')
    expect(layers).toContain('--layer-translation-')
    expect(layers).toContain('--layer-annotation-')
  })

  it('retires the stale token mirrors so one palette remains', () => {
    for (const retired of [
      'src/tokens/index.ts',
      'src/tokens/colors.css',
      'src/tokens/elevation.css',
      'src/tokens/typography.css',
      'src/tokens/motion.css',
    ]) {
      expect(existsSync(asset(retired)), retired).toBe(false)
    }
    expect(existsSync(asset('src/tokens/layers.css'))).toBe(true)
    expect(existsSync(asset('src/tokens/themes.css'))).toBe(true)
  })
})

