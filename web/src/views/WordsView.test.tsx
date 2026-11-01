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

function stubReport(report: unknown) {
  vi.stubGlobal(
    'fetch',
    vi.fn(async () => ({ ok: true, json: async () => ({ api_version: 'v1', data: report }) })),
  )
}

function frequencyReport(count: number) {
  return {
    target: 'ktb',
    rules: {
      profile: 'L3.diacritics',
      profile_version: 'v3',
      datasets: ['stored-forms'],
      multi_analysis_handling: 'SingleSource',
      window: null,
      exclusions: [],
    },
    count,
    by_surah: { 11: 40, 13: 7 },
    checksum: 'sha256:freq-1',
  }
}

describe('WordsView counts', () => {
  it('renders a frequency report with its count, rules block and checksum', async () => {
    stubReport(frequencyReport(47))
    render(<WordsView />)
    typeReference('ktb')
    fireEvent.click(screen.getByRole('button', { name: 'Run root frequency' }))

    await screen.findByText('sha256:freq-1')
    expect(screen.getByText('47')).toBeDefined()
    expect(screen.getByText('L3.diacritics')).toBeDefined()
    expect(screen.getByText('v3')).toBeDefined()
    expect(screen.getByText('stored-forms')).toBeDefined()
    expect(screen.getAllByText('none').length).toBeGreaterThanOrEqual(2)
    expect(screen.getByText('11')).toBeDefined()
    expect(screen.getByText('13')).toBeDefined()
  })

  it('prints a distribution report with its provenance and warnings', async () => {
    stubReport({
      frequency: frequencyReport(5),
      partition_provenance: 'partitions keyed by surah number (canonical structure)',
      warnings: ['single-source counts over stored derived forms'],
    })
    render(<WordsView />)
    typeReference('ktb')
    fireEvent.click(screen.getByRole('button', { name: 'Run distribution' }))

    await screen.findByText('partitions keyed by surah number (canonical structure)')
    expect(screen.getByText('single-source counts over stored derived forms')).toBeDefined()
  })

  it('renders a zero count as an explicit empty state with the rules still on screen', async () => {
    stubReport(frequencyReport(0))
    render(<WordsView />)
    typeReference('ktb')
    fireEvent.click(screen.getByRole('button', { name: 'Run token frequency' }))

    await screen.findByText('zero under these rules')
    expect(screen.getByText('L3.diacritics')).toBeDefined()
  })

  it('renders co-occurrence hits with their boundary flag', async () => {
    stubReport({
      rules: {
        profile: 'L3.diacritics',
        profile_version: 'v3',
        datasets: ['stored-forms'],
        multi_analysis_handling: 'SingleSource',
        window: 'token:5',
        exclusions: [],
      },
      hits: [{ form: 'kataba', count: 3, spans_ayah_boundary: true }],
    })
    render(<WordsView />)
    typeReference('ktb')
    fireEvent.click(screen.getByRole('button', { name: 'Run cooccurrence' }))

    await screen.findByText('spans_ayah_boundary')
    expect(screen.getByText('kataba')).toBeDefined()
    expect(screen.getByText('true')).toBeDefined()
    expect(screen.getByText('token:5')).toBeDefined()
  })

  it('keeps the co-occurrence control disabled while the window is not positive', () => {
    render(<WordsView />)
    typeReference('ktb')
    fireEvent.change(screen.getByLabelText(/^window$/i), { target: { value: '0' } })

    const run = screen.getByRole('button', { name: 'Run cooccurrence' })
    expect(run.hasAttribute('disabled')).toBe(true)
    expect(screen.getByText('enter a positive window')).toBeDefined()
  })
})

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
