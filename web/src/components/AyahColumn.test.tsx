import { describe, expect, it } from 'vitest'
import { render, screen } from '@testing-library/react'
import AyahColumn from './AyahColumn'
import LayerBadge from './LayerBadge'
import TranslationPanel from './TranslationPanel'

describe('AyahColumn (canonical slot, ADR-0112)', () => {
  it('renders Arabic text with the canonical token class, rtl + lang', () => {
    const { container } = render(<AyahColumn arabicText="نَبلهُم" reference="2:1" />)
    const el = container.querySelector('.ayah-canonical')
    expect(el).not.toBeNull()
    expect(el?.textContent).toContain('نَبلهُم')
    expect(el?.getAttribute('dir')).toBe('rtl')
    expect(el?.getAttribute('lang')).toBe('ar')
  })

  it('byte-exact canonical text (no transformation)', () => {
    const text = 'نَبلهُم هم'
    render(<AyahColumn arabicText={text} reference="2:1" />)
    expect(screen.getByText(text)).toBeDefined()
  })

  it('type guard: no translation prop exists on the canonical slot', () => {
    render(
      // @ts-expect-error — ADR-0112: canonical slot accepts no translation prop
      <AyahColumn arabicText="نَبلهُم" reference="2:1" translation={{ text: 'x', translator: 'y' }} />,
    )
  })
})

describe('LayerBadge', () => {
  it('emits the matching layer token class per layer', () => {
    const { container, rerender } = render(<LayerBadge layer="canonical" label="Q" />)
    expect(container.querySelector('.layer-canonical')).not.toBeNull()
    rerender(<LayerBadge layer="translation" label="T" />)
    expect(container.querySelector('.layer-translation')).not.toBeNull()
    rerender(<LayerBadge layer="annotation" label="A" />)
    expect(container.querySelector('.layer-annotation')).not.toBeNull()
  })
})

describe('TranslationPanel', () => {
  it('renders under the translation token class as text (no HTML injection)', () => {
    const { container } = render(
      <TranslationPanel translator="evil" text="<img src=x onerror=alert(1)>" />,
    )
    const el = container.querySelector('.ayah-translation')
    expect(el).not.toBeNull()
    expect(el?.querySelector('img')).toBeNull()
    expect(el?.textContent).toContain('<img src=x onerror=alert(1)>')
  })
})
