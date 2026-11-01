import { describe, expect, it } from 'vitest'
import { familyKindFor, parseWordInput, tokenId } from './words'

describe('parseWordInput', () => {
  it('parses surah:ayah:position as a token', () => {
    expect(parseWordInput('2:1:3')).toEqual({ kind: 'token', surah: 2, ayah: 1, position: 3 })
  })

  it('parses surah:ayah as an ayah', () => {
    expect(parseWordInput('2:1')).toEqual({ kind: 'ayah', surah: 2, ayah: 1 })
  })

  it('trims surrounding whitespace before shaping', () => {
    expect(parseWordInput('  2:1:3  ')).toEqual({ kind: 'token', surah: 2, ayah: 1, position: 3 })
  })

  it('returns null for the empty string', () => {
    expect(parseWordInput('')).toBeNull()
    expect(parseWordInput('   ')).toBeNull()
  })

  it('treats a bare Arabic word as text', () => {
    expect(parseWordInput('رحمة')).toEqual({ kind: 'text', text: 'رحمة' })
  })

  it('treats a bare Latin word as text', () => {
    expect(parseWordInput('mercy')).toEqual({ kind: 'text', text: 'mercy' })
  })

  it('rejects non-positive numbers', () => {
    expect(parseWordInput('0:0')).toBeNull()
    expect(parseWordInput('0:1')).toBeNull()
    expect(parseWordInput('2:1:0')).toBeNull()
    expect(parseWordInput('-2:1')).toBeNull()
  })

  it('rejects non-numeric and non-finite parts', () => {
    expect(parseWordInput('2:1:x')).toBeNull()
    expect(parseWordInput('2:x')).toBeNull()
    expect(parseWordInput('2.5:1')).toBeNull()
    expect(parseWordInput('Infinity:1')).toBeNull()
  })

  it('treats a non-reference colon string as text', () => {
    expect(parseWordInput('a:b:c:d')).toEqual({ kind: 'text', text: 'a:b:c:d' })
  })
})

describe('tokenId', () => {
  it('builds the exact backend family-member id', () => {
    expect(tokenId({ surah: 2, ayah: 1, position: 3 })).toBe('token:2:1:3')
  })
})

describe('familyKindFor', () => {
  it('maps a token query to token', () => {
    expect(familyKindFor({ kind: 'token', surah: 2, ayah: 1, position: 3 })).toBe('token')
  })

  it('maps an ayah query to token', () => {
    expect(familyKindFor({ kind: 'ayah', surah: 2, ayah: 1 })).toBe('token')
  })

  it('maps a text query to root', () => {
    expect(familyKindFor({ kind: 'text', text: 'رحمة' })).toBe('root')
  })
})
