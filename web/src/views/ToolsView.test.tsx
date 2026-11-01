import { afterEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/react'
import ToolsView from './ToolsView'

afterEach(() => {
  cleanup()
  vi.unstubAllGlobals()
})

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
