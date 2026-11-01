import { describe, expect, it, vi, afterEach, beforeEach } from 'vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/react'
import ReadingView from './ReadingView'
import { PREFS_KEY } from '../lib/reader-storage'
import type { AyahView } from '../api/client'

afterEach(() => cleanup())
beforeEach(() => window.localStorage.clear())

function ayah(surah: number, ayah: number, arabicText: string, translations = ['hello']): AyahView {
  return {
    canonical: {
      reference: `quran:test@0.1.0:${surah}:${ayah}`,
      arabicText,
    },
    translations: translations.map((text, i) => ({ translator: `t${i}`, text })),
  }
}

describe('ReadingView', () => {
  it('renders canonical text byte-exact, rtl, in surah order', () => {
    render(<ReadingView ayahs={[ayah(2, 2, 'ب'), ayah(2, 1, 'ا')]} onResolve={vi.fn()} />)
    // Client never re-sorts: the order given is the order rendered.
    const rendered = Array.from(document.querySelectorAll('.ayah-canonical')).map(
      (el) => el.textContent,
    )
    expect(rendered).toEqual(['ب', 'ا'])
    const rtl = document.querySelector('[dir="rtl"][lang="ar"]')
    expect(rtl).not.toBeNull()
  })

  it('switches all three translation modes and all three inspector modes', () => {
    const { container } = render(<ReadingView ayahs={[ayah(2, 1, 'ا')]} onResolve={vi.fn()} />)
    for (const mode of ['inline', 'side-by-side', 'side-panel']) {
      fireEvent.click(screen.getByRole('button', { name: mode }))
      expect(container.querySelector(`[data-translation-mode="${mode}"]`)).not.toBeNull()
    }
    for (const mode of ['popover', 'panel', 'pin']) {
      fireEvent.click(screen.getByRole('button', { name: new RegExp(`inspector.*${mode}`) }))
      expect(container.querySelector(`[data-inspector-mode="${mode}"]`)).not.toBeNull()
    }
  })

  it('adjacent ayahs are separate elements; empty translation has an explicit state', () => {
    render(<ReadingView ayahs={[ayah(2, 1, 'ا', []), ayah(2, 2, 'ب')]} onResolve={vi.fn()} />)
    expect(document.querySelectorAll('.ayah-unit')).toHaveLength(2)
    expect(screen.getByText(/no translation available/i)).toBeDefined()
  })

  it('never transforms canonical text', () => {
    const text = 'نَبلهُم'
    render(<ReadingView ayahs={[ayah(2, 1, text)]} onResolve={vi.fn()} />)
    expect(document.querySelector('.ayah-canonical')?.textContent).toBe(text)
  })

  it('the side-by-side layout grid holds the canonical cell first', () => {
    const { container } = render(
      <ReadingView ayahs={[ayah(2, 1, 'ا', ['hello', 'world'])]} onResolve={vi.fn()} />,
    )
    fireEvent.click(screen.getByRole('button', { name: 'layout side-by-side' }))
    expect(container.querySelector('[data-ayah-layout="side-by-side"]')).not.toBeNull()
    const article = container.querySelector('.ayah-unit') as HTMLElement
    const grid = article.querySelector('.md\\:grid-cols-2') as HTMLElement
    expect(grid).not.toBeNull()
    expect(grid.children).toHaveLength(2)
    // Canonical cell first in document order at every width.
    expect(grid.children[0]?.querySelector('.ayah-canonical')).not.toBeNull()
    expect(grid.children[1]?.getAttribute('dir')).toBe('ltr')
    expect(grid.children[1]?.querySelector('.ayah-canonical')).toBeNull()
  })

  it('the interlinear layout renders one ruled line per translation', () => {
    const { container } = render(
      <ReadingView ayahs={[ayah(2, 1, 'ا', ['hello', 'world'])]} onResolve={vi.fn()} />,
    )
    fireEvent.click(screen.getByRole('button', { name: 'layout interlinear' }))
    const article = container.querySelector('.ayah-unit') as HTMLElement
    const lines = article.querySelectorAll('[data-interlinear]')
    expect(lines).toHaveLength(2)
    for (const line of Array.from(lines)) {
      expect(line.getAttribute('dir')).toBe('ltr')
    }
  })

  it.each(['stacked', 'side-by-side', 'interlinear'] as const)(
    'the Arabic element keeps dir="rtl" and the canonical class in the %s layout',
    (layout) => {
      const { container } = render(
        <ReadingView ayahs={[ayah(2, 1, 'نَبلهُم')]} onResolve={vi.fn()} />,
      )
      fireEvent.click(screen.getByRole('button', { name: `layout ${layout}` }))
      const arabic = container.querySelector('.ayah-unit .ayah-canonical') as HTMLElement
      expect(arabic).not.toBeNull()
      expect(arabic.getAttribute('dir')).toBe('rtl')
      expect(arabic.getAttribute('lang')).toBe('ar')
      expect(arabic.textContent).toBe('نَبلهُم')
    },
  )

  it('the size stepper marks the active size and passes it to every column', () => {
    const { container } = render(
      <ReadingView ayahs={[ayah(2, 1, 'ا'), ayah(2, 2, 'ب')]} onResolve={vi.fn()} />,
    )
    fireEvent.click(screen.getByRole('button', { name: 'large' }))
    expect(screen.getByRole('button', { name: 'large' }).getAttribute('aria-pressed')).toBe(
      'true',
    )
    expect(container.querySelector('[data-ayah-size="lg"]')).not.toBeNull()
    for (const el of Array.from(container.querySelectorAll('.ayah-canonical'))) {
      expect(el.getAttribute('data-ayah-size')).toBe('lg')
    }
  })

  it('the chosen size and layout survive a reload via localStorage', () => {
    const first = render(<ReadingView ayahs={[ayah(2, 1, 'ا')]} onResolve={vi.fn()} />)
    fireEvent.click(screen.getByRole('button', { name: 'large' }))
    fireEvent.click(screen.getByRole('button', { name: 'layout interlinear' }))
    const saved = JSON.parse(window.localStorage.getItem(PREFS_KEY) as string)
    expect(saved.size).toBe('lg')
    expect(saved.layout).toBe('interlinear')
    first.unmount()
    const second = render(<ReadingView ayahs={[ayah(2, 1, 'ا')]} onResolve={vi.fn()} />)
    expect(second.container.querySelector('[data-ayah-size="lg"]')).not.toBeNull()
    expect(second.container.querySelector('[data-ayah-layout="interlinear"]')).not.toBeNull()
  })
})
