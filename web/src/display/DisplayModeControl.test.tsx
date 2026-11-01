import { describe, expect, it, vi, afterEach } from 'vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/react'
import DisplayModeControl from './DisplayModeControl'

afterEach(() => cleanup())

function renderControl(layout: 'stacked' | 'side-by-side' | 'interlinear' = 'stacked') {
  const onTranslationMode = vi.fn()
  const onInspectorMode = vi.fn()
  const onLayout = vi.fn()
  render(
    <DisplayModeControl
      translationMode="inline"
      inspectorMode="popover"
      layout={layout}
      onTranslationMode={onTranslationMode}
      onInspectorMode={onInspectorMode}
      onLayout={onLayout}
    />,
  )
  return { onTranslationMode, onInspectorMode, onLayout }
}

describe('DisplayModeControl layout row', () => {
  it('the layout row reports the active value', () => {
    renderControl('side-by-side')
    const active = screen.getByRole('button', { name: 'layout side-by-side' })
    expect(active.getAttribute('aria-pressed')).toBe('true')
    expect(
      screen.getByRole('button', { name: 'layout stacked' }).getAttribute('aria-pressed'),
    ).toBe('false')
  })

  it('the translation row is disabled with the reason when the layout is not stacked', () => {
    const { onTranslationMode } = renderControl('interlinear')
    const inlineButton = screen.getByRole('button', { name: 'inline' })
    expect(inlineButton.getAttribute('aria-disabled')).toBe('true')
    fireEvent.click(inlineButton)
    expect(onTranslationMode).not.toHaveBeenCalled()
    expect(
      screen.getByText(/the layout owns the translation frame — switch to Stacked/i),
    ).toBeDefined()
  })

  it('the translation row stays enabled when stacked', () => {
    const { onTranslationMode } = renderControl('stacked')
    const inlineButton = screen.getByRole('button', { name: 'inline' })
    expect(inlineButton.getAttribute('aria-disabled')).toBeNull()
    fireEvent.click(inlineButton)
    expect(onTranslationMode).toHaveBeenCalledWith('inline')
  })
})
