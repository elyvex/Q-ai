// Per-verse related lookup seed (13-13): the one cross-domain question the
// reader can answer — trace this verse's wording into the tafsir and hadith
// corpora. The seed is a dumb prefix of the verse text: no commentary is
// implied, and `RELATED_NOTE` says so beside every count (T-13-14).
export const RELATED_NOTE =
  'these are text-search hits seeded by this verse\u2019s own wording, not tafsir or hadith commentary attached to the verse.'

/**
 * First `maxChars` Unicode code points of the trimmed text. Built from
 * `Array.from` so a combining mark or a surrogate pair is never split.
 */
export function seedQuery(arabicText: string, maxChars = 40): string {
  return Array.from(arabicText.trim()).slice(0, maxChars).join('')
}
