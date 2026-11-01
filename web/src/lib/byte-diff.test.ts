import { describe, expect, it } from 'vitest'
import { byteDiff, codePointLabel, describeDiff } from './byte-diff'

describe('byteDiff', () => {
  it('reports equality with the code-point count', () => {
    const diff = byteDiff('بِسْمِ', 'بِسْمِ')
    expect(diff.equal).toBe(true)
    expect(diff.firstMismatch).toBeNull()
    expect(diff.expectedLength).toBe(Array.from('بِسْمِ').length)
    expect(diff.actualLength).toBe(diff.expectedLength)
  })

  it('names the code-point index and both code points on a mismatch', () => {
    const diff = byteDiff('abc', 'adc')
    expect(diff.equal).toBe(false)
    expect(diff.firstMismatch).toEqual({ index: 1, expected: 'b', actual: 'd' })
  })

  it('compares by code point, never by UTF-16 index', () => {
    // U+1F600 is one code point in two UTF-16 units; changing the char
    // after it must report index 1, not index 2.
    const diff = byteDiff('😀a', '😀b')
    expect(diff.firstMismatch).toEqual({ index: 1, expected: 'a', actual: 'b' })
  })

  it('reports the shorter length with an empty missing side on a length mismatch', () => {
    const longer = byteDiff('abc', 'ab')
    expect(longer.equal).toBe(false)
    expect(longer.firstMismatch).toEqual({ index: 2, expected: 'c', actual: '' })
    const shorter = byteDiff('ab', 'abc')
    expect(shorter.equal).toBe(false)
    expect(shorter.firstMismatch).toEqual({ index: 2, expected: '', actual: 'c' })
  })
})

describe('codePointLabel', () => {
  it('prints U+ plus uppercase hex padded to four digits', () => {
    expect(codePointLabel('A')).toBe('U+0041')
    expect(codePointLabel('ب')).toBe('U+0628')
    expect(codePointLabel('😀')).toBe('U+1F600')
  })

  it('reads the empty string as the end of the text', () => {
    expect(codePointLabel('')).toBe('end of text')
  })
})

describe('describeDiff', () => {
  it('states byte-identical with the code-point count', () => {
    expect(describeDiff(byteDiff('ab', 'ab'))).toBe('byte-identical · 2 code points')
  })

  it('states the index and both labels on a mismatch', () => {
    expect(describeDiff(byteDiff('ab', 'ac'))).toBe(
      'first difference at code point 1: expected U+0062, found U+0063',
    )
  })

  it('names the missing side on a length mismatch', () => {
    expect(describeDiff(byteDiff('ab', 'a'))).toBe(
      'first difference at code point 1: expected U+0062, found end of text',
    )
  })
})
