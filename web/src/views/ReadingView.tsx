// RTL reading view (D-05…D-09): a scholarly page, not a card stack.
// Canonical text renders byte-exact in document order — the client never
// re-sorts, normalizes, or merges adjacent units. Dataset text renders as
// text (no innerHTML). Display controls live in the margin column.
import { useState } from 'react'
import AyahColumn from '../components/AyahColumn'
import { PageHeader } from '../components/layout/page-header'
import TranslationPanel from '../components/TranslationPanel'
import DisplayModeControl, {
  type InspectorMode,
  type TranslationMode,
} from '../display/DisplayModeControl'
import type { AyahView } from '../api/client'

function refTail(reference: string): string {
  // `quran:<slug>@<ver>:<s>:<a>` → verse ordinal for the ruled index.
  const parts = reference.split(':')
  return parts.length >= 4 ? parts[3] : '?'
}

function refShort(reference: string): string {
  const parts = reference.split(':')
  return parts.length >= 4 ? `${parts[2]}:${parts[3]}` : reference
}

const LAYERS = [
  { label: 'Canonical', note: 'served verbatim', color: 'bg-canon' },
  { label: 'Translation', note: 'attributed, separate', color: 'bg-trans' },
  { label: 'Annotation', note: 'lexical notes', color: 'bg-annot' },
]

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

  // "quran:<slug@ver>:<s>:<a>" → "s:a" for a readable page title.
  const parts = ayahs[0]?.canonical.reference.split(':') ?? []
  const surah = parts.length >= 4 ? parts[2] : undefined
  const verse = parts.length >= 4 ? `${parts[2]}:${parts[3]}` : undefined

  return (
    <div data-translation-mode={translationMode} data-inspector-mode={inspectorMode}>
      <PageHeader
        eyebrow="Qurʾān · Canonical edition"
        title={
          surah !== undefined ? (
            <span className="flex flex-wrap items-baseline gap-x-3">
              <span>Sūrah {surah}</span>
              <span className="font-mono text-[0.9rem] font-normal tracking-normal text-muted-foreground">
                {verse}
              </span>
            </span>
          ) : (
            'Reader'
          )
        }
        lede="Every line is served from the verified corpus. The canonical layer is untouched; translations and lexical notes travel alongside it as labelled layers."
      />

      <div className="mt-8 grid gap-x-12 gap-y-8 lg:grid-cols-[minmax(0,1fr)_16rem]">
        <div className="min-w-0">
          <div className="reading-column">
            {ayahs.map((ayah, index) => (
              <article
                key={ayah.canonical.reference}
                className="ayah-unit border-t py-8 first:border-t-0 first:pt-0"
              >
                <div className="mb-5 flex items-center gap-2.5" dir="ltr">
                  <span className="medallion" aria-hidden="true">
                    {refTail(ayah.canonical.reference)}
                  </span>
                  <span className="font-mono text-[0.7rem] text-muted-foreground">
                    {refShort(ayah.canonical.reference)}
                  </span>
                  <span className="ml-auto inline-flex items-center gap-1.5 font-mono text-[0.6rem] uppercase tracking-[0.14em] text-canon">
                    <span className="prodot bg-canon" aria-hidden="true" />
                    canonical
                  </span>
                  <span className="hidden font-mono text-[0.6rem] text-muted-foreground/50 sm:inline">
                    {String(index + 1).padStart(2, '0')}
                  </span>
                </div>

                <div className="rounded-md border bg-card px-3 py-4 sm:px-5 sm:py-5">
                  <AyahColumn
                    arabicText={ayah.canonical.arabicText}
                    reference={ayah.canonical.reference}
                  />
                </div>

                {ayah.translations.length === 0 ? (
                  <p
                    className="ayah-translation mt-4 rounded-md border border-dashed px-3 py-2 text-[0.8125rem] italic"
                    data-empty-translation="true"
                    dir="ltr"
                  >
                    no translation available for this reference
                  </p>
                ) : translationMode === 'inline' ? (
                  <div className="mt-3 space-y-2" dir="ltr">
                    {ayah.translations.map((t) => (
                      <TranslationPanel key={t.translator} translator={t.translator} text={t.text} />
                    ))}
                  </div>
                ) : translationMode === 'side-by-side' ? (
                  <div className="mt-5 grid gap-x-6 gap-y-3 md:grid-cols-2" dir="ltr">
                    {ayah.translations.map((t) => (
                      <div key={t.translator} className="border-t border-dashed pt-3">
                        <TranslationPanel translator={t.translator} text={t.text} />
                      </div>
                    ))}
                  </div>
                ) : (
                  <details
                    className="side-panel mt-4 border-t pt-3 text-[0.8125rem]"
                    dir="ltr"
                  >
                    <summary className="cursor-pointer select-none font-medium text-muted-foreground transition-colors hover:text-foreground">
                      translations ({ayah.translations.length})
                    </summary>
                    <div className="mt-2.5 space-y-2">
                      {ayah.translations.map((t) => (
                        <TranslationPanel key={t.translator} translator={t.translator} text={t.text} />
                      ))}
                    </div>
                  </details>
                )}

                {/* Lexical inspector standby: no word payload ships with the
                    ayah endpoint, so nothing here pretends to inspect a token.
                    The three modes still frame the standby differently, as the
                    contract requires — per-token notes render once the engine
                    returns word data. */}
                <div
                  className="ayah-annotation-row mt-4"
                  dir="ltr"
                  data-inspector-mode={inspectorMode}
                >
                  <span className="font-mono text-[0.6rem] uppercase tracking-[0.14em] text-muted-foreground/60">
                    note
                  </span>
                  {inspectorMode === 'panel' ? (
                    <p className="mt-1.5 rounded-md border px-3 py-2 text-[0.75rem] leading-relaxed text-muted-foreground">
                      lexical notes will dock here, per token, when the engine returns word data
                    </p>
                  ) : (
                    <p className="mt-1 text-[0.75rem] leading-relaxed text-muted-foreground">
                      lexical notes attach per token when the engine returns word data
                      {inspectorMode === 'pin' && ' — pin to keep'}
                    </p>
                  )}
                </div>
              </article>
            ))}
          </div>

          <p className="mt-10 flex items-center gap-2 border-t pt-4 font-mono text-[0.625rem] uppercase tracking-[0.14em] text-muted-foreground/70">
            ٱلْمَطْبَعَةُ الْقُرْآنِيَّة
            <span className="h-px flex-1 bg-border" aria-hidden="true" />
            hash-verified corpus
          </p>
        </div>

        <aside className="space-y-6 lg:sticky lg:top-20 lg:h-fit lg:border-l lg:pl-6" aria-label="Reading controls">
          <div>
            <p className="kicker mb-2">Display</p>
            <div className="rounded-lg border bg-card p-2.5">
              <DisplayModeControl
                translationMode={translationMode}
                inspectorMode={inspectorMode}
                onTranslationMode={setTranslationMode}
                onInspectorMode={setInspectorMode}
              />
            </div>
          </div>

          <div>
            <p className="kicker mb-2.5">Layers</p>
            <ul className="space-y-2">
              {LAYERS.map((layer) => (
                <li key={layer.label} className="flex items-center gap-2 text-[0.75rem]">
                  <span className={`prodot ${layer.color}`} aria-hidden="true" />
                  <span className="font-medium text-foreground">{layer.label}</span>
                  <span className="ml-auto text-muted-foreground">{layer.note}</span>
                </li>
              ))}
            </ul>
          </div>

          <p className="margin-note border-t pt-4">
            Arabic is set in Amiri with Uthmani marks intact. The interface never edits, re-orders
            or re-metrics the canonical text.
          </p>
        </aside>
      </div>
    </div>
  )
}
