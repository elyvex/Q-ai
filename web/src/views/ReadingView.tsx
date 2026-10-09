// RTL reading view (D-05…D-09): single centered column, collapsible side
// panels, switchable translation/inspector modes. Canonical text renders
// byte-exact in document order — the client never re-sorts, normalizes, or
// merges adjacent units. Dataset text renders as text (no innerHTML).
import { useState } from 'react'
import AyahColumn from '../components/AyahColumn'
import TranslationPanel from '../components/TranslationPanel'
import WordInspector, { type InspectorMode } from '../components/WordInspector'
import DisplayModeControl, { type TranslationMode } from '../display/DisplayModeControl'
import type { AyahView } from '../api/client'

export default function ReadingView({
  ayahs,
  onResolve,
}: {
  ayahs: AyahView[]
  onResolve: (reference: string) => void
}) {
  const [translationMode, setTranslationMode] = useState<TranslationMode>('inline')
  const [inspectorMode, setInspectorMode] = useState<InspectorMode>('popover')
  void onResolve

  return (
    <main data-translation-mode={translationMode} data-inspector-mode={inspectorMode}>
      <DisplayModeControl
        translationMode={translationMode}
        inspectorMode={inspectorMode}
        onTranslationMode={setTranslationMode}
        onInspectorMode={setInspectorMode}
      />
      <div className="reading-column" dir="rtl" lang="ar">
        {ayahs.map((ayah) => (
          <article key={ayah.canonical.reference} className="ayah-unit">
            <AyahColumn arabicText={ayah.canonical.arabicText} reference={ayah.canonical.reference} />
            {ayah.translations.length === 0 ? (
              <p className="ayah-translation" data-empty-translation="true">
                no translation available
              </p>
            ) : translationMode === 'inline' ? (
              ayah.translations.map((t) => (
                <TranslationPanel key={t.translator} translator={t.translator} text={t.text} />
              ))
            ) : translationMode === 'side-by-side' ? (
              <div className="side-by-side" dir="ltr">
                {ayah.translations.map((t) => (
                  <TranslationPanel key={t.translator} translator={t.translator} text={t.text} />
                ))}
              </div>
            ) : (
              <details className="side-panel">
                <summary>translations ({ayah.translations.length})</summary>
                {ayah.translations.map((t) => (
                  <TranslationPanel key={t.translator} translator={t.translator} text={t.text} />
                ))}
              </details>
            )}
            <WordInspector word={{ surface: '•' }} mode={inspectorMode} />
          </article>
        ))}
      </div>
    </main>
  )
}
