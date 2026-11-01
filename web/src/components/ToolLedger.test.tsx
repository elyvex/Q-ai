import { describe, expect, it, afterEach } from 'vitest'
import { cleanup, render, screen } from '@testing-library/react'
import ToolLedger from './ToolLedger'
import type { ToolResultData } from '../lib/tool-result'

afterEach(() => cleanup())

function envelope(overrides: Partial<ToolResultData> = {}): ToolResultData {
  return {
    tool_name: 'rag.debug',
    tool_version: '1.0.0',
    query: { project: 'default', query: 'mercy' },
    normalization_rules: [],
    edition_id: null,
    edition_version: null,
    results: null,
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
    research_checksum: { algorithm: 'Sha256', hex: 'cc' },
    ...overrides,
  }
}

describe('ToolLedger', () => {
  it('renders the header line with the tool name, version, time and server checksum', () => {
    render(<ToolLedger result={envelope()} />)
    expect(screen.getByText('rag.debug')).toBeDefined()
    expect(document.body.textContent).toMatch(/1\.0\.0/)
    expect(document.body.textContent).toMatch(/12\.5 ms/)
    expect(document.body.textContent).toMatch(/research_checksum \(server\) sha256:cc/)
  })

  it('renders warnings when present and nothing extra when absent', () => {
    const { rerender } = render(<ToolLedger result={envelope()} />)
    expect(document.body.textContent).not.toMatch(/slow index/)
    rerender(<ToolLedger result={envelope({ warnings: ['slow index'] })} />)
    expect(screen.getByText('slow index')).toBeDefined()
  })

  it('links pinned quran references and prints the rest as plain mono text', () => {
    render(
      <ToolLedger
        result={envelope({ canonical_references: ['quran:hafs-uthmani@0.1.0:2:1', 'unpinned'] })}
      />,
    )
    const link = screen.getByText('quran:hafs-uthmani@0.1.0:2:1')
    expect(link.tagName).toBe('A')
    expect(link.getAttribute('href')).toBe('/read/hafs-uthmani@0.1.0/2:1')
    expect(screen.getByText('unpinned').tagName).not.toBe('A')
  })

  it('renders results as a table with one column per first-row key', () => {
    render(
      <ToolLedger
        result={envelope({ results: [{ doc_id: 'd1', score: 0.9 }, { doc_id: 'd2', score: 0.4 }] })}
      />,
    )
    expect(screen.getByText('doc_id')).toBeDefined()
    expect(screen.getByText('d1')).toBeDefined()
    expect(screen.getByText('0.9')).toBeDefined()
  })

  it('falls back to pretty-printed JSON when results carry no rows', () => {
    render(<ToolLedger result={envelope({ results: { hits: 3 } })} />)
    expect(document.body.querySelector('pre')?.textContent).toMatch(/"hits": 3/)
  })

  it('renders an empty state when results are null', () => {
    render(<ToolLedger result={envelope({ results: null })} />)
    expect(screen.getByText('the tool returned no rows')).toBeDefined()
  })
})
