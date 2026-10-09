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
    <div role="toolbar" aria-label="display modes">
      {TRANSLATION_MODES.map((mode) => (
        <button
          key={mode}
          type="button"
          aria-pressed={translationMode === mode}
          onClick={() => onTranslationMode(mode)}
        >
          {mode}
        </button>
      ))}
      {INSPECTOR_MODES.map((mode) => (
        <button
          key={mode}
          type="button"
          aria-pressed={inspectorMode === mode}
          onClick={() => onInspectorMode(mode)}
        >
          {`inspector ${mode}`}
        </button>
      ))}
    </div>
  )
}
