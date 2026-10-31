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
  return (
    <div role="toolbar" aria-label="display modes" className="flex flex-col gap-1.5">
      <div className="flex items-center gap-1">
        <span className="basis-full px-0.5 pb-0.5 text-[0.6rem] font-semibold uppercase tracking-[0.14em] text-muted-foreground/70">
          translation
        </span>
        {TRANSLATION_MODES.map((mode) => (
          <button
            key={mode}
            type="button"
            className="seg"
            aria-pressed={translationMode === mode}
            aria-label={mode}
            onClick={() => onTranslationMode(mode)}
          >
            {mode}
          </button>
        ))}
      </div>
      <div className="flex items-center gap-1">
        <span className="basis-full px-0.5 pb-0.5 text-[0.6rem] font-semibold uppercase tracking-[0.14em] text-muted-foreground/70">
          inspector
        </span>
        {INSPECTOR_MODES.map((mode) => (
          <button
            key={mode}
            type="button"
            className="seg"
            aria-pressed={inspectorMode === mode}
            aria-label={`inspector ${mode}`}
            onClick={() => onInspectorMode(mode)}
          >
            {mode}
          </button>
        ))}
      </div>
    </div>
  )
}
