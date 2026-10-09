import { describe, expect, it, vi, afterEach } from 'vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/react'
import ReadingView from './ReadingView'
import type { AyahView } from '../api/client'

afterEach(() => cleanup())

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
      fireEvent.click(screen.getByRole('button', { name: new RegExp(mode) }))
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
})
