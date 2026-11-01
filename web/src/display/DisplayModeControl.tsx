// Display-mode control: three translation modes × three inspector modes,
// plus the three layout frames that own the translation surface (13-13).
export type InspectorMode = 'popover' | 'panel' | 'pin'

export type TranslationMode = 'inline' | 'side-by-side' | 'side-panel'

export type ReaderLayout = 'stacked' | 'side-by-side' | 'interlinear'

const TRANSLATION_MODES: TranslationMode[] = ['inline', 'side-by-side', 'side-panel']
const INSPECTOR_MODES: InspectorMode[] = ['popover', 'panel', 'pin']
const READER_LAYOUTS: ReaderLayout[] = ['stacked', 'side-by-side', 'interlinear']

function ModeRow({
  label,
  modes,
  active,
  onSelect,
  namePrefix = '',
  disabledReason,
}: {
  label: string
  modes: readonly string[]
  active: string
  onSelect: (mode: string) => void
  namePrefix?: string
  disabledReason?: string
}) {
  const disabled = disabledReason !== undefined
  return (
    <div className="flex flex-col gap-1">
      <span className="px-0.5 font-mono text-[0.6rem] uppercase tracking-[0.14em] text-muted-foreground/70">
        {label}
      </span>
      <div className="grid grid-cols-3 gap-0.5" aria-disabled={disabled || undefined}>
        {modes.map((mode) => (
          <button
            key={mode}
            type="button"
            className="seg min-w-0 truncate px-1 text-[0.7rem]"
            aria-pressed={active === mode}
            aria-label={`${namePrefix}${mode}`}
            aria-disabled={disabled || undefined}
            title={disabled ? disabledReason : mode}
            onClick={() => {
              if (!disabled) onSelect(mode)
            }}
          >
            {mode}
          </button>
        ))}
      </div>
      {disabled && (
        <p className="px-0.5 text-[0.7rem] leading-snug text-muted-foreground">{disabledReason}</p>
      )}
    </div>
  )
}

export default function DisplayModeControl({
  translationMode,
  inspectorMode,
  layout,
  onTranslationMode,
  onInspectorMode,
  onLayout,
}: {
  translationMode: TranslationMode
  inspectorMode: InspectorMode
  layout?: ReaderLayout
  onTranslationMode: (mode: TranslationMode) => void
  onInspectorMode: (mode: InspectorMode) => void
  onLayout?: (layout: ReaderLayout) => void
}) {
  const activeLayout = layout ?? 'stacked'
  const translationDisabledReason =
    activeLayout !== 'stacked'
      ? 'the layout owns the translation frame — switch to Stacked to choose how translations collapse'
      : undefined
  return (
    <div role="toolbar" aria-label="display modes" className="flex flex-col gap-3">
      <ModeRow
        label="translation"
        modes={TRANSLATION_MODES}
        active={translationMode}
        onSelect={(mode) => onTranslationMode(mode as TranslationMode)}
        disabledReason={translationDisabledReason}
      />
      <ModeRow
        label="inspector"
        modes={INSPECTOR_MODES}
        active={inspectorMode}
        onSelect={(mode) => onInspectorMode(mode as InspectorMode)}
        namePrefix="inspector "
      />
      <ModeRow
        label="layout"
        modes={READER_LAYOUTS}
        active={activeLayout}
        onSelect={(mode) => onLayout?.(mode as ReaderLayout)}
        namePrefix="layout "
      />
    </div>
  )
}
