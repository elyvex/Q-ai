import { describe, expect, it, afterEach } from 'vitest'
import { cleanup, render, screen } from '@testing-library/react'
import ExplorerView, { PRESETS } from './ExplorerView'

afterEach(() => cleanup())

/** Exact templates the server registers in `crates/server/src/api.rs`. */
const ALLOWED_ROUTES = [
  '/api/v1/quran/ayahs/{reference}',
  '/api/v1/quran/resolve',
  '/api/v1/quran/citations/{id}',
  '/api/v1/quran/search/{mode}',
  '/api/v1/quran/family',
  '/api/v1/quran/count/{instrument}',
  '/api/v1/quran/graph/{instrument}',
]

/** The 27-name callable set: every name the tool dispatcher accepts. */
const CALLABLE_TOOLS = [
  'quran.get_ayah',
  'quran.get_context',
  'quran.search',
  'quran.root',
  'quran.lemma',
  'quran.morphology',
  'quran.family',
  'quran.graph_neighbors',
  'quran.graph_path',
  'quran.graph_pattern',
  'quran.graph_subgraph',
  'quran.graph_root_family',
  'hadith.get',
  'hadith.search',
  'hadith.gradings',
  'tafsir.get',
  'tafsir.search',
  'scripture.get',
  'scripture.search',
  'scripture.translations',
  'scripture.editions',
  'scripture.parallel_passages',
  'isnad.trace',
  'isnad.narrator',
  'isnad.rijal',
  'rag.query',
  'rag.debug',
]

/** A template matches only when every `{…}` placeholder covers exactly one non-slash segment. */
function matchesRoute(path: string): boolean {
  return ALLOWED_ROUTES.some((template) => {
    const pattern = template
      .split(/(\{[^}]+\})/)
      .map((part) => (/^\{[^}]+\}$/.test(part) ? '[^/]+' : part.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')))
      .join('')
    return new RegExp(`^${pattern}$`).test(path)
  })
}

const TOOL_PATH = /^\/api\/v1\/quran\/tool\/([a-z_.]+)$/

function toolNameOf(path: string): string | null {
  return TOOL_PATH.exec(path)?.[1] ?? null
}

describe('ExplorerView presets', () => {
  it('names only reachable routes or callable tools', () => {
    expect(PRESETS.length).toBeGreaterThan(0)
    for (const preset of PRESETS) {
      if (matchesRoute(preset.path)) continue
      const tool = toolNameOf(preset.path)
      expect(tool, `${preset.label} targets no known route: ${preset.path}`).not.toBeNull()
      expect(CALLABLE_TOOLS, `${preset.label} names no callable tool`).toContain(tool)
    }
  })

  it('keeps the removed invented paths out', () => {
    const paths = PRESETS.map((preset) => preset.path)
    for (const removed of [
      '/api/v1/scripture/editions',
      '/api/v1/scripture/search',
      '/api/v1/cross-domain/research',
      '/api/v1/rag/debug',
    ]) {
      expect(paths).not.toContain(removed)
    }
  })

  it('names no mutating tool', () => {
    const mutating = /^(rag\.create_project|rag\.create|rag\.import|rag\.source_|isnad\.identity_)/
    for (const preset of PRESETS) {
      const tool = toolNameOf(preset.path)
      if (tool !== null) expect(tool).not.toMatch(mutating)
    }
  })

  it('renders every preset as a labelled button', () => {
    render(<ExplorerView />)
    for (const preset of PRESETS) {
      expect(screen.getByRole('button', { name: new RegExp(preset.label) })).toBeDefined()
    }
  })
})
