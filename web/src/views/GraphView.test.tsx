import { describe, expect, it, afterEach } from 'vitest'
import { cleanup, render } from '@testing-library/react'
import GraphView from './GraphView'

afterEach(() => cleanup())

describe('GraphView', () => {
  it('truncated payload renders truncated + reason verbatim, never absence', () => {
    render(
      <GraphView
        lines={['ayah:1:1 -NEXT-> ayah:1:2']}
        truncated={true}
        incompleteReason="max_edges budget cut the search short"
      />,
    )
    expect(document.body.textContent).toContain('truncated')
    expect(document.body.textContent).toContain('max_edges budget cut the search short')
    expect(document.body.textContent?.toLowerCase()).not.toContain('no path')
  })

  it('complete payload renders without the truncation block', () => {
    render(<GraphView lines={['ayah:1:1 -NEXT-> ayah:1:2']} truncated={false} />)
    expect(document.body.textContent).not.toContain('truncated')
  })
})
