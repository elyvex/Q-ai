import { afterEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, within } from '@testing-library/react'
import ToolsView from './ToolsView'
import { DENIED } from '../lib/tools'
import type { ToolResultData } from '../lib/tool-result'

afterEach(() => {
  cleanup()
  vi.unstubAllGlobals()
})

function ayahToolData(): ToolResultData {
  return {
    tool_name: 'quran.get_ayah',
    tool_version: '1.0.0',
    query: { reference: '2:1' },
    normalization_rules: [],
    edition_id: null,
    edition_version: null,
    results: [{ reference: '2:1', text: '...' }],
    canonical_references: ['quran:test@0.1.0:2:1'],
    analysis_sources: [],
    confidence: null,
    warnings: [],
    execution_time_ms: 3.1,
    reproducibility: {
      checksum: { algorithm: 'Sha256', hex: 'aa' },
      query_hash: { algorithm: 'Sha256', hex: 'bb' },
      tool_plan: ['quran.get_ayah@1.0.0'],
      source_versions: {},
      edition_versions: { test: '0.1.0' },
    },
    research_checksum: { algorithm: 'Sha256', hex: 'deadbeef' },
  }
}

describe('ToolsView forms', () => {
  it('renders the default tool with its labelled fields and a disabled Run', () => {
    render(<ToolsView />)
    expect(screen.getAllByText('quran.get_ayah').length).toBeGreaterThan(0)
    expect(screen.getByLabelText('Reference')).toBeDefined()
    expect(screen.getByLabelText(/Translations/)).toBeDefined()
    expect(screen.getByRole('button', { name: 'Run quran.get_ayah' })).toHaveProperty(
      'disabled',
      true,
    )
    expect(screen.getByText('required: Reference')).toBeDefined()
  })

  it('selects another tool and renders its labelled fields', () => {
    render(<ToolsView />)
    fireEvent.click(screen.getByRole('button', { name: /quran\.search/ }))
    expect(screen.getByLabelText('Text')).toBeDefined()
    expect(screen.getByLabelText(/Edition/)).toBeDefined()
    expect(screen.getByLabelText(/Limit/)).toBeDefined()
    expect(screen.getByRole('button', { name: 'Run quran.search' })).toHaveProperty(
      'disabled',
      true,
    )
    expect(screen.getByText('required: Text')).toBeDefined()
  })

  it('enables Run once the required field is filled', () => {
    render(<ToolsView />)
    fireEvent.change(screen.getByLabelText('Reference'), { target: { value: '2:1' } })
    expect(screen.getByRole('button', { name: 'Run quran.get_ayah' })).toHaveProperty(
      'disabled',
      false,
    )
    expect(screen.queryByText('required: Reference')).toBeNull()
  })
})

describe('ToolsView denied tools', () => {
  it('renders every denied row as disabled with its policy code', () => {
    const { container } = render(<ToolsView />)
    const rows = container.querySelectorAll('li[aria-disabled="true"]')
    expect(rows.length).toBe(DENIED.length)
    for (const denied of DENIED) {
      const row = screen.getByText(denied.name).closest('li')
      expect(row?.getAttribute('aria-disabled')).toBe('true')
      expect(within(row as HTMLElement).getByText(denied.code)).toBeDefined()
      expect(row?.querySelector('button')).toBeNull()
    }
    expect(screen.getByText('rag.create_project')).toBeDefined()
    expect(screen.getByText('isnad.identity_supersede')).toBeDefined()
  })
})

describe('ToolsView honest failure', () => {
  it('renders the server code and summary with no ledger table', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => ({
        ok: false,
        status: 400,
        json: async () => ({
          error: {
            code: 'QAI-SCR-0310',
            summary: 'scripture parallel passages is not supported by this backend',
            remedy: 'serve a backend that implements parallel passages',
          },
        }),
      })),
    )
    render(<ToolsView />)
    fireEvent.click(screen.getByRole('button', { name: /parallel_passages/ }))
    fireEvent.change(screen.getByLabelText('Reference'), { target: { value: 'gen:1:1' } })
    fireEvent.click(screen.getByRole('button', { name: 'Run scripture.parallel_passages' }))

    await screen.findByText(/QAI-SCR-0310/)
    expect(
      screen.getByText('scripture parallel passages is not supported by this backend'),
    ).toBeDefined()
    expect(
      screen.getByText('serve a backend that implements parallel passages'),
    ).toBeDefined()
    expect(screen.queryByRole('table')).toBeNull()
  })
})

describe('ToolsView cli surface', () => {
  it('prints the CLI line for the exact params that produced the checksum', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => ({ ok: true, json: async () => ({ api_version: 'v1', data: ayahToolData() }) })),
    )
    render(<ToolsView />)
    fireEvent.change(screen.getByLabelText('Reference'), { target: { value: '2:1' } })
    fireEvent.click(screen.getByRole('button', { name: 'Run quran.get_ayah' }))
    await screen.findByText(/deadbeef/)

    fireEvent.click(screen.getByRole('button', { name: 'cli' }))
    const invocation = screen.getByText(/qai quran tool/)
    expect(invocation.textContent).toContain('quran.get_ayah')
    expect(invocation.textContent).toContain('{"reference":"2:1"}')
    expect(invocation.textContent).toContain('--json')
  })
})
