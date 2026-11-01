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
