import { describe, expect, it } from 'vitest'
import {
  DENIED,
  TOOLS,
  ToolParamError,
  cliCommand,
  httpRequest,
  paramsJson,
  type ToolSpec,
} from './tools'

const GET_AYAH = TOOLS.find((tool) => tool.name === 'quran.get_ayah')

function requireAyah(): ToolSpec {
  if (GET_AYAH === undefined) throw new Error('quran.get_ayah missing from TOOLS')
  return GET_AYAH
}

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

describe('tools catalogue', () => {
  it('seeds quran.get_ayah with its four registry fields', () => {
    const spec = requireAyah()
    expect(spec.fields.map((field) => field.key)).toEqual([
      'reference',
      'translations',
      'glosses',
      'tokens',
    ])
    expect(spec.fields.find((field) => field.key === 'reference')?.required).toBe(true)
    expect(
      spec.fields.find((field) => field.key === 'translations')?.hint,
    ).toContain('translation slugs')
  })

  it('seeds the denial list with a shaped entry', () => {
    expect(DENIED.length).toBeGreaterThan(0)
    for (const denied of DENIED) {
      expect(denied.name.length).toBeGreaterThan(0)
      expect(denied.code).toMatch(/^QAI-[A-Z]+-\d+$/)
      expect(denied.reason.length).toBeGreaterThan(0)
    }
  })

  it('derives every http path from the tool name', () => {
    for (const tool of TOOLS) {
      expect(httpRequest(tool.name, {}).path).toBe(`/api/v1/quran/tool/${tool.name}`)
    }
  })

  it('names the callable set exactly: 27 names, no duplicates', () => {
    const names = TOOLS.map((tool) => tool.name)
    expect(names).toHaveLength(27)
    expect(new Set(names).size).toBe(27)
    expect([...names].sort()).toEqual([...CALLABLE_TOOLS].sort())
  })

  it('keeps every denied name out of the callable catalogue', () => {
    const names = new Set(TOOLS.map((tool) => tool.name))
    for (const denied of DENIED) {
      expect(names.has(denied.name)).toBe(false)
    }
  })
})

describe('paramsJson', () => {
  it('drops every empty optional field', () => {
    expect(paramsJson(requireAyah(), { reference: '2:1' })).toEqual({ reference: '2:1' })
  })

  it('parses a json field into the parameter object', () => {
    expect(
      paramsJson(requireAyah(), { reference: '2:1', translations: '["en-sahih"]' }),
    ).toEqual({ reference: '2:1', translations: ['en-sahih'] })
  })

  it('throws a ToolParamError naming the field for malformed json', () => {
    try {
      paramsJson(requireAyah(), { reference: '2:1', translations: '[oops' })
      expect.unreachable('malformed json must throw')
    } catch (error: unknown) {
      expect(error).toBeInstanceOf(ToolParamError)
      expect((error as ToolParamError).field).toBe('translations')
    }
  })

  it('throws a ToolParamError naming the field for a missing required field', () => {
    try {
      paramsJson(requireAyah(), {})
      expect.unreachable('a missing required field must throw')
    } catch (error: unknown) {
      expect(error).toBeInstanceOf(ToolParamError)
      expect((error as ToolParamError).field).toBe('reference')
    }
  })

  it('coerces a numeric field on a real spec and drops empty optionals', () => {
    const morphology = TOOLS.find((tool) => tool.name === 'quran.morphology')
    if (morphology === undefined) throw new Error('quran.morphology missing from TOOLS')
    expect(paramsJson(morphology, { surah: '2', ayah: '1', position: '3' })).toEqual({
      surah: 2,
      ayah: 1,
      position: 3,
    })
    const context = TOOLS.find((tool) => tool.name === 'quran.get_context')
    if (context === undefined) throw new Error('quran.get_context missing from TOOLS')
    expect(paramsJson(context, { reference: '2:1', before: '', boundary: '' })).toEqual({
      reference: '2:1',
    })
    expect(paramsJson(context, { reference: '2:1', before: '2', max_ayahs: '11' })).toEqual({
      reference: '2:1',
      before: 2,
      max_ayahs: 11,
    })
  })

  it('coerces numbers and maps bools on a synthetic spec', () => {
    const spec: ToolSpec = {
      name: 'synthetic.probe',
      label: 'Probe',
      domain: 'quran',
      summary: 'probe',
      fields: [
        { key: 'limit', label: 'Limit', kind: 'number', required: false },
        { key: 'glosses', label: 'Glosses', kind: 'bool', required: false },
      ],
    }
    expect(paramsJson(spec, { limit: '5', glosses: 'true' })).toEqual({
      limit: 5,
      glosses: true,
    })
    try {
      paramsJson(spec, { limit: 'many' })
      expect.unreachable('a non-numeric number field must throw')
    } catch (error: unknown) {
      expect(error).toBeInstanceOf(ToolParamError)
      expect((error as ToolParamError).field).toBe('limit')
    }
  })
})

describe('surface derivations', () => {
  it('prints the CLI line for the exact parameter object', () => {
    expect(cliCommand('quran.get_ayah', { reference: '2:1' })).toBe(
      `qai quran tool quran.get_ayah --params '{"reference":"2:1"}' --json`,
    )
  })

  it('escapes embedded single quotes as quote-backslash-quote-quote', () => {
    expect(cliCommand('quran.search', { text: "mercy's" })).toBe(
      `qai quran tool quran.search --params '{"text":"mercy'\\''s"}' --json`,
    )
  })

  it('prints POST against the tool path with a pretty body', () => {
    const request = httpRequest('quran.get_ayah', { reference: '2:1' })
    expect(request.method).toBe('POST')
    expect(request.path).toBe('/api/v1/quran/tool/quran.get_ayah')
    expect(JSON.parse(request.body) as unknown).toEqual({ reference: '2:1' })
    expect(request.body).toContain('\n')
  })
})
