import { describe, expect, it } from 'vitest'
import { DEFAULT_PREFS, PREFS_KEY, loadPrefs, savePrefs } from './reader-storage'

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
})
