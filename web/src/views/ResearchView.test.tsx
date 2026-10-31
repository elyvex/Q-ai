import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import ResearchView from './ResearchView'

describe('ResearchView', () => {
  it('renders research workspace', () => {
    render(<ResearchView query="" />)
    expect(screen.getByText('Research Workspace')).toBeDefined()
  })
})
