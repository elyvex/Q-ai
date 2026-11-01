import { describe, expect, it } from 'vitest'
import { RELATED_NOTE, seedQuery } from './related'

describe('related seed', () => {
  it('a 60-code-point string truncates to 40 without splitting a surrogate pair', () => {
    const text = `${'ا'.repeat(39)}😀${'ب'.repeat(20)}`
    expect(Array.from(text).length).toBe(60)
    const seed = seedQuery(text)
    expect(Array.from(seed).length).toBe(40)
    // The 40th code point is the emoji — it must survive whole.
    expect(seed).toContain('😀')
  })

  it('a short string returns itself trimmed', () => {
    expect(seedQuery('  نَبلهُم  ')).toBe('نَبلهُم')
  })

  it('whitespace-only returns the empty string', () => {
    expect(seedQuery('   ')).toBe('')
  })

  it('RELATED_NOTE names both corpora and the seeded nature', () => {
    expect(RELATED_NOTE).toContain('tafsir')
    expect(RELATED_NOTE).toContain('hadith')
    expect(RELATED_NOTE).toContain('seeded')
  })
})
