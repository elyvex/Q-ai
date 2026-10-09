import { describe, expect, it, afterEach } from 'vitest'
import { cleanup, render } from '@testing-library/react'
import RagDebugView from './RagDebugView'

afterEach(() => cleanup())

describe('RagDebugView', () => {
  it('typed unavailable state, zero results, no retrieval path', () => {
    const { container } = render(<RagDebugView />)
    expect(document.body.textContent).toMatch(/not configured|unavailable/i)
    expect(container.querySelectorAll('.rag-result')).toHaveLength(0)
  })
})
