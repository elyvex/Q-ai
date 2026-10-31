import { useState } from 'react'

// Word inspector: lemma/root/gloss for one token under the annotation
// layer token. Modes: popover (default), persistent panel, popover-with-pin.
export type InspectorMode = 'popover' | 'panel' | 'pin'

export interface WordInfo {
  surface: string
  lemma?: string
  root?: string
  gloss?: string
}

export default function WordInspector({
  word,
  mode = 'popover',
}: {
  word: WordInfo
  mode?: InspectorMode
}) {
  const [pinned, setPinned] = useState(false)
  const open = mode === 'panel' || pinned
  const note =
    [word.lemma && `lemma ${word.lemma}`, word.root && `root ${word.root}`, word.gloss]
      .filter(Boolean)
      .join(' · ') + (mode === 'pin' && !pinned ? ' (pin to keep)' : '')
  return (
    <span className="ayah-annotation inline-flex items-center gap-2 text-sm" data-inspector-mode={mode}>
      <button
        type="button"
        aria-pressed={mode !== 'popover' || pinned}
        className={
          pinned
            ? 'rounded-full px-2 py-0.5 text-xs'
            : 'rounded-full border px-2 py-0.5 text-xs hover:opacity-80'
        }
        style={pinned ? { background: 'var(--layer-annotation-bg)' } : undefined}
        onClick={() => setPinned((p) => !p)}
      >
        <span className="layer-annotation font-semibold">{word.surface}</span>
      </button>
      {mode !== 'panel' && (open || mode === 'popover') && (
        <span role="note" className="layer-annotation text-xs opacity-80">
          {note}
        </span>
      )}
    </span>
  )
}
