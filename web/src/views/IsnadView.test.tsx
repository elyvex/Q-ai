import { afterEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/react'
import IsnadView from './IsnadView'
import { DENIED } from '../lib/tools'
import type { ToolResultData } from '../lib/tool-result'

afterEach(() => {
  cleanup()
  vi.unstubAllGlobals()
})

const IDENTITY_NAMES = DENIED.filter((tool) => tool.name.startsWith('isnad.identity')).map(
  (tool) => tool.name,
)

function toolData(name: string): ToolResultData {
  return {
    tool_name: name,
    tool_version: '1.0.0',
    query: { from: 'a', to: 'b' },
    normalization_rules: [],
    edition_id: null,
    edition_version: null,
    results: [{ path: 'a → b' }],
    canonical_references: [],
    analysis_sources: [],
    confidence: null,
    warnings: [],
    execution_time_ms: 1.2,
    reproducibility: {
      checksum: { algorithm: 'Sha256', hex: 'aa' },
      query_hash: { algorithm: 'Sha256', hex: 'bb' },
      tool_plan: [`${name}@1.0.0`],
      source_versions: {},
      edition_versions: {},
    },
    research_checksum: { algorithm: 'Sha256', hex: 'cafe01' },
  }
}

function stubTools() {
  const calls: { url: string; body: unknown }[] = []
  vi.stubGlobal(
    'fetch',
    vi.fn(async (url: unknown, init?: { body?: string }) => {
      const target = String(url)
      calls.push({
        url: target,
        body: typeof init?.body === 'string' ? (JSON.parse(init.body) as unknown) : init?.body,
      })
      const name = target.split('/').pop() ?? target
      return {
        ok: true,
        json: async () => ({ api_version: 'v1', data: toolData(name), meta: {} }),
      }
    }),
  )
  return calls
}

describe('IsnadView instruments', () => {
  it('a trace run renders the ledger header with the tool name and checksum', async () => {
    stubTools()
    render(<IsnadView />)
    fireEvent.change(screen.getByLabelText('From'), { target: { value: 'n1' } })
    fireEvent.change(screen.getByLabelText('To'), { target: { value: 'n2' } })
    fireEvent.click(screen.getByRole('button', { name: 'Run isnad.trace' }))
    await screen.findByText('isnad.trace')
    expect(screen.getByText(/cafe01/)).toBeDefined()
  })

  it('switching to rijal runs isnad.rijal with the typed id', async () => {
    const calls = stubTools()
    render(<IsnadView />)
    fireEvent.click(screen.getByRole('button', { name: 'rijal' }))
    fireEvent.change(screen.getByLabelText('Id'), { target: { value: 'narr-7' } })
    fireEvent.click(screen.getByRole('button', { name: 'Run isnad.rijal' }))
    await screen.findByText('isnad.rijal')
    const last = calls[calls.length - 1]
    expect(last?.url).toContain('isnad.rijal')
    expect(last?.body).toEqual({ id: 'narr-7' })
  })

  it('Run is disabled with the required-field reason while from is blank', () => {
    render(<IsnadView />)
    expect(screen.getByRole('button', { name: 'Run isnad.trace' })).toHaveProperty(
      'disabled',
      true,
    )
    expect(screen.getByText('required: From')).toBeDefined()
  })

  it('a typed error renders its code', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => ({
        ok: false,
        status: 503,
        json: async () => ({
          error: { code: 'QAI-ISNAD-0310', summary: 'isnad backend not configured' },
        }),
      })),
    )
    render(<IsnadView />)
    fireEvent.change(screen.getByLabelText('From'), { target: { value: 'n1' } })
    fireEvent.change(screen.getByLabelText('To'), { target: { value: 'n2' } })
    fireEvent.click(screen.getByRole('button', { name: 'Run isnad.trace' }))
    await screen.findByText(/QAI-ISNAD-0310/)
    expect(screen.getByText(/isnad backend not configured/)).toBeDefined()
  })
})

describe('IsnadView denied mutations', () => {
  it('every identity tool renders as a disabled row with QAI-ISNAD-0326', () => {
    const { container } = render(<IsnadView />)
    expect(IDENTITY_NAMES).toEqual([
      'isnad.identity_propose',
      'isnad.identity_accept',
      'isnad.identity_reject',
      'isnad.identity_dispute',
      'isnad.identity_supersede',
    ])
    const rows = container.querySelectorAll('li[aria-disabled="true"]')
    expect(rows.length).toBe(IDENTITY_NAMES.length)
    for (const name of IDENTITY_NAMES) {
      expect(screen.getByText(name)).toBeDefined()
    }
    expect(screen.getAllByText('QAI-ISNAD-0326').length).toBe(IDENTITY_NAMES.length)
    expect(screen.getAllByText(/trusted operator surface is `qai isnad/).length).toBe(
      IDENTITY_NAMES.length,
    )
  })
})
