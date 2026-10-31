import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import GraphView from './GraphView'

describe('GraphView', () => {
  it('renders graph instruments and an explicit empty state', () => {
    render(<GraphView lines={[]} truncated={false} />)
    expect(screen.getByText('Knowledge Graph')).toBeDefined()
    expect(screen.getByText('no graph lines selected — open neighbors or trace a path above')).toBeDefined()
  })

  it('renders frozen graph lines verbatim', () => {
    render(<GraphView lines={['ayah:1:1 -next-> ayah:1:2']} truncated={false} />)
    expect(screen.getByText('ayah:1:1 -next-> ayah:1:2')).toBeDefined()
  })

  it('announces truncation instead of rendering absence (ADR-0217)', () => {
    render(
      <GraphView lines={['a -e-> b']} truncated incompleteReason="budget exceeded" />,
    )
    expect(screen.getByText('truncated: budget exceeded')).toBeDefined()
  })
})
