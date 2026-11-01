// Reader display prefs (13-13): three sizes × three layouts, persisted in
// localStorage under `qai.reader.prefs.v1`. The storage object is injected
// so tests pin the contract without a DOM global. Loads never throw:
// a missing key, invalid JSON, a non-object, or an out-of-range value falls
// back to the default for that single field while valid siblings survive
// (T-13-13: browser-writable storage never reaches a class name unchecked).
export type ReaderSize = 'sm' | 'md' | 'lg'
export type ReaderLayout = 'stacked' | 'side-by-side' | 'interlinear'
export type TranslationMode = 'inline' | 'side-by-side' | 'side-panel'
export type InspectorMode = 'popover' | 'panel' | 'pin'

export interface ReaderPrefs {
  size: ReaderSize
  layout: ReaderLayout
  translationMode: TranslationMode
  inspectorMode: InspectorMode
}

export const PREFS_KEY = 'qai.reader.prefs.v1'

export const DEFAULT_PREFS: ReaderPrefs = {
  size: 'md',
  layout: 'stacked',
  translationMode: 'inline',
  inspectorMode: 'popover',
}

const SIZES: readonly ReaderSize[] = ['sm', 'md', 'lg']
const LAYOUTS: readonly ReaderLayout[] = ['stacked', 'side-by-side', 'interlinear']
const TRANSLATION_MODES: readonly TranslationMode[] = ['inline', 'side-by-side', 'side-panel']
const INSPECTOR_MODES: readonly InspectorMode[] = ['popover', 'panel', 'pin']

function pick<T extends string>(value: unknown, allowed: readonly T[], fallback: T): T {
  return typeof value === 'string' && (allowed as readonly string[]).includes(value)
    ? (value as T)
    : fallback
}

export function loadPrefs(storage: Pick<Storage, 'getItem'>): ReaderPrefs {
  let raw: string | null = null
  try {
    raw = storage.getItem(PREFS_KEY)
  } catch {
    return { ...DEFAULT_PREFS }
  }
  if (raw === null) return { ...DEFAULT_PREFS }
  let parsed: unknown
  try {
    parsed = JSON.parse(raw)
  } catch {
    return { ...DEFAULT_PREFS }
  }
  if (typeof parsed !== 'object' || parsed === null || Array.isArray(parsed)) {
    return { ...DEFAULT_PREFS }
  }
  const obj = parsed as Record<string, unknown>
  return {
    size: pick(obj.size, SIZES, DEFAULT_PREFS.size),
    layout: pick(obj.layout, LAYOUTS, DEFAULT_PREFS.layout),
    translationMode: pick(obj.translationMode, TRANSLATION_MODES, DEFAULT_PREFS.translationMode),
    inspectorMode: pick(obj.inspectorMode, INSPECTOR_MODES, DEFAULT_PREFS.inspectorMode),
  }
}

export function savePrefs(storage: Pick<Storage, 'setItem'>, prefs: ReaderPrefs): void {
  storage.setItem(PREFS_KEY, JSON.stringify(prefs))
}
