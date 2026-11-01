// Code-point-exact text comparison for the citations workbench. The
// comparison is by Unicode code point (`Array.from`, never UTF-16 index),
// so Arabic with combining marks is compared the way the reader sees it.
// Lengths are code-point counts. This module never hashes: the only
// `text_hash` the screen shows is the string the server returned.
export interface ByteMismatch {
  index: number
  expected: string
  actual: string
}

export interface ByteDiff {
  equal: boolean
  expectedLength: number
  actualLength: number
  firstMismatch: ByteMismatch | null
}

/** Compare two texts code point by code point. */
export function byteDiff(expected: string, actual: string): ByteDiff {
  const expectedPoints = Array.from(expected)
  const actualPoints = Array.from(actual)
  const shared = Math.min(expectedPoints.length, actualPoints.length)
  for (let i = 0; i < shared; i++) {
    if (expectedPoints[i] !== actualPoints[i]) {
      return {
        equal: false,
        expectedLength: expectedPoints.length,
        actualLength: actualPoints.length,
        firstMismatch: {
          index: i,
          expected: expectedPoints[i] as string,
          actual: actualPoints[i] as string,
        },
      }
    }
  }
  if (expectedPoints.length === actualPoints.length) {
    return {
      equal: true,
      expectedLength: expectedPoints.length,
      actualLength: actualPoints.length,
      firstMismatch: null,
    }
  }
  // Shared prefix, different lengths: the shorter side is missing text.
  return {
    equal: false,
    expectedLength: expectedPoints.length,
    actualLength: actualPoints.length,
    firstMismatch: {
      index: shared,
      expected: shared < expectedPoints.length ? (expectedPoints[shared] as string) : '',
      actual: shared < actualPoints.length ? (actualPoints[shared] as string) : '',
    },
  }
}

/**
 * `U+` plus the code point in uppercase hex padded to four digits — the
 * label the diff readout prints. The empty string is the missing side of a
 * length mismatch, not a character, so it reads as `end of text`.
 */
export function codePointLabel(value: string): string {
  if (value === '') return 'end of text'
  const point = value.codePointAt(0)
  if (point === undefined) return 'end of text'
  return `U+${point.toString(16).toUpperCase().padStart(4, '0')}`
}

/** One line for the readout: the count when equal, the index and both labels when not. */
export function describeDiff(diff: ByteDiff): string {
  if (diff.equal || diff.firstMismatch === null) {
    return `byte-identical · ${diff.expectedLength} code points`
  }
  const mismatch = diff.firstMismatch
  return `first difference at code point ${mismatch.index}: expected ${codePointLabel(mismatch.expected)}, found ${codePointLabel(mismatch.actual)}`
}
