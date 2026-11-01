import { describe, expect, it } from 'vitest'
import { DIRECTORY, ROUTES } from './router'
import { NAVIGATION } from './components/command/command-menu'
import { isActive } from './components/layout/site-header'

describe('words route wiring', () => {
  it('lists /words in ROUTES, NAVIGATION and DIRECTORY', () => {
    expect(ROUTES.some((route) => route.path === '/words')).toBe(true)
    expect(NAVIGATION.some((item) => item.href === '/words')).toBe(true)
    expect(DIRECTORY.some((item) => item.path === '/words')).toBe(true)
  })

  it('lights the Research rail entry while /words is open', () => {
    expect(isActive('/research', '/words', ['/words'])).toBe(true)
    expect(isActive('/research', '/words')).toBe(false)
  })

  it('pins the also-before-/ ordering', () => {
    expect(isActive('/', '/citations', ['/citations'])).toBe(true)
  })
})

describe('tools route wiring', () => {
  it('lists /tools in ROUTES, NAVIGATION and DIRECTORY', () => {
    expect(ROUTES.some((route) => route.path === '/tools')).toBe(true)
    expect(NAVIGATION.some((item) => item.href === '/tools')).toBe(true)
    expect(DIRECTORY.some((item) => item.path === '/tools')).toBe(true)
  })

  it('lights the Research rail entry while /tools is open', () => {
    expect(isActive('/research', '/tools', ['/words', '/tools'])).toBe(true)
  })
})
