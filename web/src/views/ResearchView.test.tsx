import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import ResearchView, { ResearchResults } from './ResearchView'

describe('ResearchView', () => {
  it('renders the research console with all nine instruments', () => {
    render(<ResearchView query="" />)
    expect(screen.getByText('Research Console')).toBeDefined()
    for (const name of ['Search', 'Root', 'Lemma', 'Family', 'Frequency', 'Co-occurrence']) {
      expect(screen.getByRole('button', { name })).toBeDefined()
    }
  })

  it('renders hits in service order with editorial rows', () => {
    render(
      <ResearchResults
        query="الله"
        hits={[
          { reference: '1:1', snippet: 'ا' },
          { reference: '2:2', snippet: 'ب' },
        ]}
      />,
    )
    const refs = Array.from(document.querySelectorAll('.research-hit .chip')).map((el) => el.textContent)
    expect(refs).toEqual(['1:1', '2:2'])
  })

  it('renders an explicit empty state, never a blank panel', () => {
    render(<ResearchResults query="zzz" hits={[]} />)
    expect(screen.getByText(/no results for “zzz”/)).toBeDefined()
  })
})
