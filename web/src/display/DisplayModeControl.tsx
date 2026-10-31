// Display-mode control: three translation modes × three inspector modes.
// Every mode renders a distinct layout in ReadingView (D-05/D-06/D-07).
export type InspectorMode = 'popover' | 'panel' | 'pin'

export type TranslationMode = 'inline' | 'side-by-side' | 'side-panel'

const TRANSLATION_MODES: TranslationMode[] = ['inline', 'side-by-side', 'side-panel']
const INSPECTOR_MODES: InspectorMode[] = ['popover', 'panel', 'pin']

function ModeRow({
  label,
  modes,
  active,
  onSelect,
  namePrefix = '',
}: {
  label: string
  modes: readonly string[]
  active: string
  onSelect: (mode: string) => void
  namePrefix?: string
}) {
  return (
    <div className="flex flex-col gap-1">
      <span className="px-0.5 font-mono text-[0.6rem] uppercase tracking-[0.14em] text-muted-foreground/70">
        {label}
      </span>
      <div className="grid grid-cols-3 gap-0.5">
        {modes.map((mode) => (
          <button
            key={mode}
            type="button"
            className="seg min-w-0 truncate px-1 text-[0.7rem]"
            aria-pressed={active === mode}
            aria-label={`${namePrefix}${mode}`}
            title={mode}
            onClick={() => onSelect(mode)}
          >
            {mode}
          </button>
        ))}
      </div>
    </div>
  )
}

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
    <div role="toolbar" aria-label="display modes" className="flex flex-col gap-3">
      <ModeRow
        label="translation"
        modes={TRANSLATION_MODES}
        active={translationMode}
        onSelect={(mode) => onTranslationMode(mode as TranslationMode)}
      />
      <ModeRow
        label="inspector"
        modes={INSPECTOR_MODES}
        active={inspectorMode}
        onSelect={(mode) => onInspectorMode(mode as InspectorMode)}
        namePrefix="inspector "
      />
    </div>
  )
}
