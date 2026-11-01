import { afterEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/react'
import WordsView from './WordsView'
import type { ToolResultData } from '../lib/tool-result'

afterEach(() => {
  cleanup()
  vi.unstubAllGlobals()
})

function wordToolData(
  toolName: string,
  results: unknown,
  editionVersions: Record<string, string> = { test: '0.1.0' },
): ToolResultData {
  return {
    tool_name: toolName,
    tool_version: '1.0.0',
    query: { root: 'ktb' },
    normalization_rules: [],
    edition_id: null,
    edition_version: null,
    results,
    canonical_references: [],
    analysis_sources: [],
    confidence: null,
    warnings: [],
    execution_time_ms: 3.1,
    reproducibility: {
      checksum: { algorithm: 'Sha256', hex: 'aa' },
      query_hash: { algorithm: 'Sha256', hex: 'bb' },
      tool_plan: [`${toolName}@1.0.0`],
      source_versions: {},
      edition_versions: editionVersions,
    },
    research_checksum: { algorithm: 'Sha256', hex: 'deadbeef' },
  }
}

function stubData(data: ToolResultData) {
  vi.stubGlobal(
    'fetch',
    vi.fn(async () => ({ ok: true, json: async () => ({ api_version: 'v1', data }) })),
  )
}

function stubFailure(code: string, summary: string) {
  vi.stubGlobal(
    'fetch',
    vi.fn(async () => ({
      ok: false,
      status: 500,
      json: async () => ({ error: { code, summary } }),
    })),
  )
}

function typeReference(value: string) {
  fireEvent.change(screen.getByLabelText(/reference, root or lemma/i), { target: { value } })
}

async function lookUp() {
  fireEvent.click(screen.getByRole('button', { name: 'Look up' }))
}

describe('WordsView lexicon', () => {
  it('renders root occurrence rows with working reader deep links', async () => {
    stubData(
      wordToolData('quran.root', [
        { surah: 2, ayah: 1, position: 3, dataset: 'stored-forms' },
        { surah: 2, ayah: 2, position: 1, dataset: 'stored-forms' },
      ]),
    )
    render(<WordsView />)
    typeReference('ktb')
    await lookUp()

    const first = await screen.findByRole('link', { name: '2:1' })
    expect(first.getAttribute('href')).toBe('/read/test@0.1.0/2:1')
    const second = screen.getByRole('link', { name: '2:2' })
    expect(second.getAttribute('href')).toBe('/read/test@0.1.0/2:2')
    expect(screen.getAllByText('stored-forms').length).toBeGreaterThanOrEqual(2)
    expect(screen.getByText('dataset attribution')).toBeDefined()
  })

  it('defaults a token query to family and renders the explanation in full', async () => {
    let requestedUrl = ''
    let requestedBody = ''
    vi.stubGlobal(
      'fetch',
      vi.fn(async (url: unknown, init: unknown) => {
        requestedUrl = String(url)
        requestedBody = String((init as { body?: unknown }).body)
        return {
          ok: true,
          json: async () => ({
            api_version: 'v1',
            data: wordToolData('quran.family', [
              {
                id: 'token:2:1:4',
                kind: 'token',
                relation: 'same-lemma',
                explanation: 'Shares the lemma kataba with the queried token.',
                dataset: 'stored-forms',
              },
            ]),
          }),
        }
      }),
    )
    render(<WordsView />)
    typeReference('2:1:3')

    expect(screen.getByRole('button', { name: 'family' }).getAttribute('aria-pressed')).toBe('true')
    expect((screen.getByLabelText(/member id/i) as HTMLInputElement).value).toBe('token:2:1:3')

    await lookUp()

    expect(requestedUrl).toMatch(/quran\.family/)
    expect(requestedBody).toMatch(/token:2:1:3/)
    await screen.findByText('Shares the lemma kataba with the queried token.')
    expect(screen.getByText('same-lemma')).toBeDefined()
  })

  it('renders the plain reference and the margin note when no single edition can be pinned', async () => {
    stubData(
      wordToolData(
        'quran.root',
        [{ surah: 2, ayah: 1, position: 3, dataset: 'stored-forms' }],
        {},
      ),
    )
    render(<WordsView />)
    typeReference('ktb')
    await lookUp()

    await screen.findByText('2:1')
    expect(screen.queryByRole('link')).toBeNull()
    expect(screen.getByText(/No single edition can be pinned/)).toBeDefined()
  })

  it('renders the server code when the lexicon call fails', async () => {
    stubFailure('QAI-MORPH-0007', 'no active morphology dataset')
    render(<WordsView />)
    typeReference('ktb')
    fireEvent.click(screen.getByRole('button', { name: 'Look up' }))

    await screen.findByText(/QAI-MORPH-0007/)
    expect(screen.getByText(/no active morphology dataset/)).toBeDefined()
  })
})
