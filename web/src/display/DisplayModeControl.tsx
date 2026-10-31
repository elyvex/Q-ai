// Display-mode control: three translation modes × three inspector modes.
// Every mode renders a distinct layout in ReadingView (D-05/D-06/D-07).
import type { InspectorMode } from '../components/WordInspector'

export type TranslationMode = 'inline' | 'side-by-side' | 'side-panel'

const TRANSLATION_MODES: TranslationMode[] = ['inline', 'side-by-side', 'side-panel']
const INSPECTOR_MODES: InspectorMode[] = ['popover', 'panel', 'pin']

export default function DisplayModeControl({
  translationMode,
  inspectorMode,
  onTranslationMode,
  onInspectorMode,
}: {
  translationMode: TranslationMode
  inspectorMode: InspectorMode
  onTranslationMode: (mode: TranslationMode) => void
  onInspectorMode: (mode: InspectorMode) => void
}) {
  const btn =
    'rounded-lg px-2.5 py-1.5 text-xs font-medium transition-colors aria-pressed:bg-primary aria-pressed:text-primary-foreground hover:bg-muted aria-pressed:hover:bg-primary'
  return (
    <div role="toolbar" aria-label="display modes" className="flex flex-wrap gap-1">
      <span className="px-1 pt-1.5 text-[0.6rem] font-semibold uppercase tracking-wider text-muted-foreground">
        translation
      </span>
      {TRANSLATION_MODES.map((mode) => (
        <button key={mode} type="button" className={btn} aria-pressed={translationMode === mode} onClick={() => onTranslationMode(mode)}>
          {mode}
        </button>
      ))}
      <span className="px-1 pt-1.5 text-[0.6rem] font-semibold uppercase tracking-wider text-muted-foreground">
        inspector
      </span>
      {INSPECTOR_MODES.map((mode) => (
        <button
          key={mode}
          type="button"
          className={btn}
          aria-pressed={inspectorMode === mode}
          onClick={() => onInspectorMode(mode)}
        >
          {`inspector ${mode}`}
        </button>
      ))}
    </div>
  )
}
