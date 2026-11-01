// Word-inspector input parsing (13-11): one free-text field becomes a
// discriminated query — a token, an ayah, or lexicon text. Anything the
// field cannot shape is rejected, never guessed.
export type WordQuery =
  | { kind: 'token'; surah: number; ayah: number; position: number }
  | { kind: 'ayah'; surah: number; ayah: number }
  | { kind: 'text'; text: string }

function positiveInt(raw: string): number | null {
  if (raw.trim() === '') return null
  const n = Number(raw)
  if (!Number.isFinite(n) || !Number.isInteger(n) || n <= 0) return null
  return n
}

/**
 * Trimmed input shapes one query: `surah:ayah:position` is a token,
 * `surah:ayah` is an ayah, any other non-empty string is lexicon text.
 * The empty string — and colon shapes with non-positive, non-integer or
 * non-finite parts — return null.
 */
export function parseWordInput(raw: string): WordQuery | null {
  const trimmed = raw.trim()
  if (trimmed === '') return null
  const parts = trimmed.split(':')
  if (parts.length === 2 || parts.length === 3) {
    const nums = parts.map(positiveInt)
    if (nums.some((n) => n === null)) return null
    const [surah, ayah, position] = nums as number[]
    if (position !== undefined) return { kind: 'token', surah, ayah, position }
    return { kind: 'ayah', surah, ayah }
  }
  return { kind: 'text', text: trimmed }
}

/** Exact id the backend builds for a family member. */
export function tokenId(q: { surah: number; ayah: number; position: number }): string {
  return `token:${q.surah}:${q.ayah}:${q.position}`
}

/**
 * Default `quran.family` kind for a query: token and ayah queries resolve
 * by token id, lexicon text resolves by root.
 */
export function familyKindFor(q: WordQuery): 'token' | 'root' | 'lemma' {
  switch (q.kind) {
    case 'token':
    case 'ayah':
      return 'token'
    case 'text':
      return 'root'
  }
}
