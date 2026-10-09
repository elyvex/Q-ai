import { describe, expect, it, afterEach } from 'vitest'
import { cleanup, render } from '@testing-library/react'
import ResearchView from './ResearchView'
import type { ResearchHit } from './ResearchView'

afterEach(() => cleanup())

const OUT_OF_ORDER: ResearchHit[] = [
  { reference: 'quran:test@0.1.0:6:1', snippet: 'zeta' },
  { reference: 'quran:test@0.1.0:1:1', snippet: 'alpha' },
]

describe('ResearchView', () => {
  it('renders hits in service-returned order (no client re-sort)', () => {
    render(<ResearchView query="q" hits={OUT_OF_ORDER} />)
    const rendered = Array.from(document.querySelectorAll('.research-hit')).map(
      (el) => el.textContent,
    )
    expect(rendered[0]).toContain('zeta')
    expect(rendered[1]).toContain('alpha')
  })

  it('zero results render an explicit empty state', () => {
    render(<ResearchView query="zzz-no-match" hits={[]} />)
    expect(document.querySelector('.research-empty')).not.toBeNull()
    expect(document.body.textContent).toMatch(/no results/i)
  })

  it('dataset text renders as text, never HTML', () => {
    render(
      <ResearchView
        query="q"
        hits={[{ reference: 'quran:test@0.1.0:1:1', snippet: '<img src=x onerror=alert(1)>' }]}
      />,
    )
    expect(document.querySelector('.research-hit img')).toBeNull()
    expect(document.body.textContent).toContain('<img src=x onerror=alert(1)>')
  })
})
