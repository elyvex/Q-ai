import { describe, expect, it } from 'vitest'
import {
  BOOKMARKS_KEY,
  DEFAULT_PREFS,
  PREFS_KEY,
  addBookmark,
  isBookmarked,
  loadBookmarks,
  loadPrefs,
  removeBookmark,
  saveBookmarks,
  savePrefs,
} from './reader-storage'

function fakeStorage(initial: Record<string, string> = {}): Storage {
  const store = new Map(Object.entries(initial))
  return {
    getItem: (key: string) => (store.has(key) ? (store.get(key) as string) : null),
    setItem: (key: string, value: string) => {
      store.set(key, value)
    },
    removeItem: (key: string) => {
      store.delete(key)
    },
    clear: () => store.clear(),
    key: (index: number) => Array.from(store.keys())[index] ?? null,
    get length() {
      return store.size
    },
  } as Storage
}

describe('reader-storage prefs', () => {
  it('a missing key yields defaults', () => {
    expect(loadPrefs(fakeStorage())).toEqual(DEFAULT_PREFS)
  })

  it('a partial object keeps valid fields and repairs invalid ones', () => {
    const storage = fakeStorage({
      [PREFS_KEY]: JSON.stringify({ size: 'lg', layout: 'sideways', bogus: 1 }),
    })
    expect(loadPrefs(storage)).toEqual({ ...DEFAULT_PREFS, size: 'lg' })
  })

  it('a non-object yields defaults', () => {
    for (const raw of ['[1,2]', '"md"', '42', 'null']) {
      expect(loadPrefs(fakeStorage({ [PREFS_KEY]: raw }))).toEqual(DEFAULT_PREFS)
    }
  })

  it('invalid JSON yields defaults', () => {
    expect(loadPrefs(fakeStorage({ [PREFS_KEY]: '{oops' }))).toEqual(DEFAULT_PREFS)
  })

  it('a round trip returns what was saved', () => {
    const storage = fakeStorage()
    const prefs = {
      size: 'sm' as const,
      layout: 'interlinear' as const,
      translationMode: 'side-panel' as const,
      inspectorMode: 'pin' as const,
    }
    savePrefs(storage, prefs)
    expect(loadPrefs(storage)).toEqual(prefs)
  })

  it('bookmarks: add, duplicate-add, remove, isBookmarked', () => {
    const a = { reference: 'quran:test@0.1.0:2:1', label: '2:1', addedAt: '2026-11-01T00:00:00Z' }
    const b = { reference: 'quran:test@0.1.0:2:2', label: '2:2', addedAt: '2026-11-01T00:00:01Z' }
    let list = addBookmark([], a)
    list = addBookmark(list, b)
    expect(list).toHaveLength(2)
    // Duplicate add is idempotent — the original entry survives untouched.
    const again = addBookmark(list, { ...a, addedAt: 'later' })
    expect(again).toBe(list)
    expect(isBookmarked(list, a.reference)).toBe(true)
    expect(isBookmarked(list, 'quran:test@0.1.0:9:9')).toBe(false)
    list = removeBookmark(list, a.reference)
    expect(list).toEqual([b])
    expect(isBookmarked(list, a.reference)).toBe(false)
  })

  it('bookmarks: a malformed list yields an empty list', () => {
    for (const raw of ['{oops', '"x"', '42', 'null', '{"reference":"q"}']) {
      expect(loadBookmarks(fakeStorage({ [BOOKMARKS_KEY]: raw }))).toEqual([])
    }
  })

  it('bookmarks: one bad entry among good ones is dropped, dupes keep the first', () => {
    const good = { reference: 'quran:test@0.1.0:2:1', label: '2:1', addedAt: 't' }
    const dupe = { ...good, addedAt: 'later' }
    const bad = { reference: 'quran:test@0.1.0:2:2' }
    const storage = fakeStorage()
    saveBookmarks(storage, [good, bad, dupe] as never)
    expect(loadBookmarks(storage)).toEqual([good])
    expect(storage.getItem(BOOKMARKS_KEY)).toContain('2:1')
  })
})
