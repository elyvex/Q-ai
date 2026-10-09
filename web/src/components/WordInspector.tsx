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
  return (
    <span className="ayah-annotation" data-inspector-mode={mode}>
      <button type="button" onClick={() => setPinned((p) => !p)}>
        {word.surface}
      </button>
      {(open || mode === 'popover') && (
        <span role="note">
          {[word.lemma && `lemma ${word.lemma}`, word.root && `root ${word.root}`, word.gloss]
            .filter(Boolean)
            .join(' · ')}
          {mode === 'pin' && !pinned && ' (pin to keep)'}
        </span>
      )}
    </span>
  )
}
