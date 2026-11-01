import { afterEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/react'
import RagDebugView from './RagDebugView'
import type { ToolResultData } from '../lib/tool-result'

afterEach(() => {
  cleanup()
  vi.unstubAllGlobals()
})

function toolData(overrides: Partial<ToolResultData> = {}): ToolResultData {
  return {
    tool_name: 'rag.debug',
    tool_version: '1.0.0',
    query: { project: 'default', query: 'mercy' },
    normalization_rules: [],
    edition_id: null,
    edition_version: null,
    results: {
      hits: [{ doc_id: 'd1', score: 0.9, source: 'quran', rank: 1 }],
      route_decision: {
        router: 'hybrid',
        tools: ['lexical', 'vector'],
        sources: ['quran', 'tafsir'],
        scores: { lexical: 0.9 },
        final_ranking: ['d1'],
        checksum: 'abc123',
        candidates: 14,
        merged: 8,
        reranked: 5,
        excluded: 3,
      },
    },
    canonical_references: [],
    analysis_sources: [],
    confidence: null,
    warnings: [],
    execution_time_ms: 12.5,
    reproducibility: {
      checksum: { algorithm: 'Sha256', hex: 'aa' },
      query_hash: { algorithm: 'Sha256', hex: 'bb' },
      tool_plan: ['rag.debug@1.0.0'],
      source_versions: {},
      edition_versions: {},
    },
    research_checksum: { algorithm: 'Sha256', hex: 'deadbeef' },
    ...overrides,
  }
}

function stubSuccess(data: ToolResultData) {
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

async function runForm() {
  fireEvent.click(screen.getByRole('button', { name: /run debug query/i }))
  await screen.findByText('rag.debug')
}

describe('RagDebugView', () => {
  it('renders the tool name, checksum and route-decision fields on success', async () => {
    stubSuccess(toolData())
    render(<RagDebugView />)
    await runForm()

    expect(screen.getByText('rag.debug')).toBeDefined()
    expect(document.body.textContent).toMatch(/sha256:deadbeef/)
    expect(screen.getByText('hybrid')).toBeDefined()
    expect(screen.getByText('lexical, vector')).toBeDefined()
    expect(screen.getByText('quran, tafsir')).toBeDefined()
    expect(screen.getByText('14')).toBeDefined()
    expect(screen.getByText('abc123')).toBeDefined()
  })

  it('renders the server code and detail when the RAG backend is not configured', async () => {
    stubFailure('QAI-RAG-0310', 'RAG backend is not configured for project default')
    render(<RagDebugView />)
    fireEvent.click(screen.getByRole('button', { name: /run debug query/i }))

    await screen.findByText(/QAI-RAG-0310/)
    expect(screen.getByText(/RAG backend is not configured/)).toBeDefined()
    expect(screen.queryByText('rag.debug')).toBeNull()
  })

  it('lists the mutating RAG tools as disabled rows with the policy code', () => {
    render(<RagDebugView />)
    for (const name of ['rag.create_project', 'rag.import', 'rag.source_activate']) {
      const row = screen.getByText(name).closest('li')
      expect(row?.getAttribute('aria-disabled')).toBe('true')
      expect(row?.querySelector('button')).toBeNull()
    }
    expect(screen.getAllByText('QAI-RAG-0326').length).toBeGreaterThan(0)
  })

  it('prints absent for a route-decision field the server omitted, never a zero', async () => {
    const data = toolData()
    const results = data.results as Record<string, unknown>
    const decision = { ...(results.route_decision as Record<string, unknown>) }
    delete decision.reranked
    stubSuccess({ ...data, results: { ...results, route_decision: decision } })
    render(<RagDebugView />)
    await runForm()

    const label = screen.getByText('reranked')
    expect(label.closest('div')?.textContent).toMatch(/absent/)
    expect(label.closest('div')?.textContent).not.toMatch(/: 0/)
  })

  it('runs rag.query through the dispatcher when the query mode is selected', async () => {
    const fetchMock = vi.fn(async () => ({
      ok: true,
      json: async () => ({ api_version: 'v1', data: toolData({ tool_name: 'rag.query' }) }),
    }))
    vi.stubGlobal('fetch', fetchMock)
    render(<RagDebugView />)
    fireEvent.click(screen.getByRole('button', { name: 'query' }))
    fireEvent.click(screen.getByRole('button', { name: /run query query/i }))
    await screen.findByText('rag.query')

    expect(fetchMock).toHaveBeenCalledOnce()
    expect(String(fetchMock.mock.calls[0][0])).toMatch(/rag\.query/)
  })
})
