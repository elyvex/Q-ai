// Guard for G-13-1: the animated background must be mounted by the real app
// shell, must not be painted over, and must take its paint from the tokenised
// `.gradient-field` atom rather than a hard-coded gradient in the component.
//
// Ceiling: the DOM side proves the mount, the attributes and the ancestor
// chain; the source side proves the app root does not reintroduce an opaque
// page ground and that the atom + keyframes still exist. It cannot prove
// rendered contrast or motion smoothness — those stay Lighthouse/human checks.
import { afterEach, describe, expect, it, vi } from 'vitest'
import { cleanup, render } from '@testing-library/react'
import { readFileSync } from 'node:fs'
import { AppShell } from '@/components/layout/app-shell'

const PAGE_GROUND = /\bbg-background\b/

function matchMediaStub(matches: boolean) {
  return (query: string) => ({
    matches: matches && query.includes('prefers-reduced-motion'),
    media: query,
    onchange: null,
    addEventListener: () => {},
    removeEventListener: () => {},
    addListener: () => {},
    removeListener: () => {},
    dispatchEvent: () => false,
  })
}

function ancestors(element: Element): Element[] {
  const chain: Element[] = []
  let node = element.parentElement
  while (node && node !== document.body) {
    chain.push(node)
    node = node.parentElement
  }
  return chain
}

function source(relativePath: string) {
  return readFileSync(new URL(relativePath, import.meta.url), 'utf8')
}

afterEach(() => {
  cleanup()
  vi.unstubAllGlobals()
})

describe('AnimatedBackground is wired into the app shell (G-13-1)', () => {
  it('mounts the animated background in the app shell', () => {
    vi.stubGlobal('matchMedia', matchMediaStub(false))
    render(<AppShell currentPath="/">{null}</AppShell>)

    const field = document.querySelector('[data-slot="animated-background"]')
    expect(field, 'AnimatedBackground is NOT MOUNTED in the app shell').not.toBeNull()

    const element = field as HTMLElement
    expect(element.getAttribute('aria-hidden')).toBe('true')
    expect(element.style.pointerEvents).toBe('none')
    expect(element.style.zIndex).toBe('-1')
    expect(Number(element.style.opacity)).toBeLessThanOrEqual(0.15)
    expect(element.style.animationPlayState).toBe('running')
    expect(element.dataset.paused).toBe('false')
  })

  it('paints no opaque page ground over the field', () => {
    vi.stubGlobal('matchMedia', matchMediaStub(false))
    render(<AppShell currentPath="/">{null}</AppShell>)

    const field = document.querySelector('[data-slot="animated-background"]') as Element
    const opaque = ancestors(field).filter((node) => PAGE_GROUND.test(node.className))
    expect(
      opaque.map((node) => node.className),
      'an opaque page ground repaints over the animated field',
    ).toEqual([])
  })

  it('the app root does not repaint the page ground', () => {
    const app = source('../App.tsx')
    expect(app).toContain('<Router')
    expect(PAGE_GROUND.test(app)).toBe(false)
  })

  it('stops under prefers-reduced-motion', () => {
    vi.stubGlobal('matchMedia', matchMediaStub(true))
    render(<AppShell currentPath="/">{null}</AppShell>)

    const element = document.querySelector('[data-slot="animated-background"]') as HTMLElement
    expect(element.dataset.reducedMotion).toBe('true')
    expect(element.style.animationDuration).toBe('0ms')
  })

  it('the visuals are a tokenised atom, not an orphaned inline gradient', () => {
    const css = source('../index.css')
    expect(css).toContain('.gradient-field')
    expect(css).toContain('@keyframes gradient-drift')
    expect(css).toContain('var(--primary)')

    const component = source('../components/background/AnimatedBackground.tsx')
    expect(component).toContain('gradient-field')
    expect(component).not.toMatch(/radial-gradient|rgb\(/)
  })
})
