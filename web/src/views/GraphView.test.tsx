import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import GraphView from './GraphView'

describe('GraphView', () => {
  it('renders empty graph message', () => {
    render(<GraphView lines={[]} truncated={false} />)
    expect(screen.getByText('Knowledge Graph')).toBeDefined()
  })
})
