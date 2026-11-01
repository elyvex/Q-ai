import { describe, expect, it } from 'vitest'
import { checksumLabel, deepLinkFor, editionOf, rowsOf } from './tool-result'

describe('checksumLabel', () => {
  it('lowercases the algorithm and joins it to the hex with a colon', () => {
    expect(checksumLabel({ algorithm: 'Sha256', hex: 'abc123' })).toBe('sha256:abc123')
  })

  it('returns the empty string for a null or undefined argument', () => {
    expect(checksumLabel(null)).toBe('')
    expect(checksumLabel(undefined)).toBe('')
  })
})

describe('rowsOf', () => {
  it('returns the array when results is an array of plain objects', () => {
    const rows = [
      { reference: '2:1', score: 0.9 },
      { reference: '2:2', score: 0.7 },
    ]
    expect(rowsOf(rows)).toEqual(rows)
  })

  it('drops non-object entries without throwing', () => {
    expect(rowsOf([{ a: 1 }, 'nope', 42, null, [1]])).toEqual([{ a: 1 }])
  })

  it('returns an empty array for a string, number, null, or nested object', () => {
    expect(rowsOf('results')).toEqual([])
    expect(rowsOf(42)).toEqual([])
    expect(rowsOf(null)).toEqual([])
    expect(rowsOf(undefined)).toEqual([])
    expect(rowsOf({ hits: [] })).toEqual([])
  })
})

describe('deepLinkFor', () => {
  it('converts a pinned quran reference into its deep link', () => {
    expect(deepLinkFor('quran:hafs-uthmani@0.1.0:2:1')).toBe('/read/hafs-uthmani@0.1.0/2:1')
  })

  it('returns null when the string does not match', () => {
    expect(deepLinkFor('2:1')).toBeNull()
    expect(deepLinkFor('quran:2:1')).toBeNull()
    expect(deepLinkFor('')).toBeNull()
  })
})

describe('editionOf', () => {
  it('returns null for an empty map', () => {
    expect(editionOf({ edition_versions: {} }, 'hafs-uthmani')).toBeNull()
  })

  it('returns the slug and version for a single-entry map', () => {
    expect(editionOf({ edition_versions: { 'hafs-uthmani': '0.1.0' } }, 'hafs-uthmani')).toEqual({
      slug: 'hafs-uthmani',
      version: '0.1.0',
    })
  })

  it('finds the requested slug in a multi-entry map', () => {
    const reproducibility = { edition_versions: { a: '1.0.0', b: '2.0.0' } }
    expect(editionOf(reproducibility, 'b')).toEqual({ slug: 'b', version: '2.0.0' })
  })

  it('returns null when the requested slug is absent from a multi-entry map', () => {
    const reproducibility = { edition_versions: { a: '1.0.0', b: '2.0.0' } }
    expect(editionOf(reproducibility, 'c')).toBeNull()
  })
})
