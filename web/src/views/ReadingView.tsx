// RTL reading view (D-05…D-09): a scholarly page, not a card stack.
// Canonical text renders byte-exact in document order — the client never
// re-sorts, normalizes, or merges adjacent units. Dataset text renders as
// text (no innerHTML). Display controls live in the margin column.
// Size + layout prefs and bookmarks persist in localStorage (13-13);
// related lookups are per-verse and on demand, labelled as seeded hits.
import { useEffect, useState } from 'react'
import AyahColumn from '../components/AyahColumn'
import { PageHeader } from '../components/layout/page-header'
import TranslationPanel from '../components/TranslationPanel'
import DisplayModeControl, {
  type InspectorMode,
  type ReaderLayout,
  type TranslationMode,
} from '../display/DisplayModeControl'
import { EmptyState } from '../components/states'
import { ApiRequestError, runTool, type AyahView } from '../api/client'
import { deepLinkFor, rowsOf, type ToolResultData } from '../lib/tool-result'
import { RELATED_NOTE, seedQuery } from '../lib/related'
import {
  addBookmark,
  isBookmarked,
  loadBookmarks,
  loadPrefs,
  removeBookmark,
  saveBookmarks,
  savePrefs,
  type Bookmark,
  type ReaderSize,
} from '../lib/reader-storage'

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

const SIZE_LABELS: Record<ReaderSize, string> = { sm: 'small', md: 'medium', lg: 'large' }
const SIZES: readonly ReaderSize[] = ['sm', 'md', 'lg']

/** Rows when `results` is an object array, the `hits` array's length when it
    carries one, otherwise `unknown` — never a fabricated `0`. */
function countHits(data: ToolResultData): string {
  if (Array.isArray(data.results)) return String(rowsOf(data.results).length)
  if (
    typeof data.results === 'object' &&
    data.results !== null &&
    Array.isArray((data.results as Record<string, unknown>).hits)
  ) {
    return String(((data.results as Record<string, unknown>).hits as unknown[]).length)
  }
  return 'unknown'
}

function errorText(error: unknown): string {
  if (error instanceof ApiRequestError) {
    return `${error.code ?? 'error'} — ${error.message}`
  }
  return error instanceof Error ? error.message : 'unknown error'
}

type RelatedState =
  | { status: 'idle' }
  | { status: 'loading' }
  | {
      status: 'done'
      seed: string
      tafsir: string
      hadith: string
      tafsirError: string | null
      hadithError: string | null
    }

/** Per-verse, on-demand trace into the tafsir and hadith corpora. Nothing is
    fetched until pressed; the seed and the disclaimer render beside every
    count so a hit is never read as commentary on the verse (T-13-14). */
function RelatedBlock({ arabicText }: { arabicText: string }) {
  const [state, setState] = useState<RelatedState>({ status: 'idle' })

  async function lookup() {
    const seed = seedQuery(arabicText)
    setState({ status: 'loading' })
    const [tafsirSettled, hadithSettled] = await Promise.allSettled([
      runTool('tafsir.search', { query: seed, limit: 5 }),
      runTool('hadith.search', { query: seed, limit: 5 }),
    ])
    setState({
      status: 'done',
      seed,
      tafsir:
        tafsirSettled.status === 'fulfilled' ? countHits(tafsirSettled.value.data) : '—',
      hadith:
        hadithSettled.status === 'fulfilled' ? countHits(hadithSettled.value.data) : '—',
      tafsirError: tafsirSettled.status === 'rejected' ? errorText(tafsirSettled.reason) : null,
      hadithError: hadithSettled.status === 'rejected' ? errorText(hadithSettled.reason) : null,
    })
  }

  if (state.status === 'idle') {
    return (
      <button type="button" className="seg px-1.5 text-[0.7rem]" onClick={lookup}>
        related
      </button>
    )
  }
  if (state.status === 'loading') {
    return (
      <button
        type="button"
        className="seg px-1.5 text-[0.7rem]"
        disabled
        aria-label="related loading"
      >
        related…
      </button>
    )
  }
  return (
    <div>
      <button
        type="button"
        className="seg px-1.5 text-[0.7rem]"
        onClick={lookup}
        aria-label="related — run again"
      >
        related
      </button>
      <div className="mt-2 rounded-md border px-3 py-2" dir="ltr">
        <p className="font-mono text-[0.6rem] uppercase tracking-[0.14em] text-muted-foreground">
          seeded query
        </p>
        <p className="mt-1 font-mono text-[0.75rem] text-foreground">{state.seed}</p>
        {state.tafsirError === null ? (
          <p className="mt-1.5 text-[0.75rem]">tafsir hits: {state.tafsir}</p>
        ) : (
          <p className="mt-1.5 text-[0.75rem] text-destructive">tafsir error: {state.tafsirError}</p>
        )}
        {state.hadithError === null ? (
          <p className="mt-1 text-[0.75rem]">hadith hits: {state.hadith}</p>
        ) : (
          <p className="mt-1 text-[0.75rem] text-destructive">hadith error: {state.hadithError}</p>
        )}
        <p className="margin-note mt-2 border-t pt-2">{RELATED_NOTE}</p>
      </div>
    </div>
  )
}

function guardedLoadPrefs() {
  if (typeof localStorage === 'undefined') {
    return { size: 'md' as ReaderSize, layout: 'stacked' as ReaderLayout, translationMode: 'inline' as TranslationMode, inspectorMode: 'popover' as InspectorMode }
  }
  return loadPrefs(localStorage)
}

export default function ReadingView({
  ayahs,
  onResolve,
}: {
  ayahs: AyahView[]
  onResolve: (reference: string) => void
}) {
  const [initial] = useState(guardedLoadPrefs)
  const [size, setSize] = useState<ReaderSize>(initial.size)
  const [layout, setLayout] = useState<ReaderLayout>(initial.layout)
  const [translationMode, setTranslationMode] = useState<TranslationMode>(initial.translationMode)
  const [inspectorMode, setInspectorMode] = useState<InspectorMode>(initial.inspectorMode)
  const [bookmarks, setBookmarks] = useState<Bookmark[]>(() =>
    typeof localStorage === 'undefined' ? [] : loadBookmarks(localStorage),
  )
  void onResolve

  useEffect(() => {
    if (typeof localStorage === 'undefined') return
    try {
      savePrefs(localStorage, { size, layout, translationMode, inspectorMode })
    } catch {
      // Private-mode storage throws — the reader still works for the session.
    }
  }, [size, layout, translationMode, inspectorMode])

  function toggleBookmark(reference: string) {
    setBookmarks((prev) => {
      const next = isBookmarked(prev, reference)
        ? removeBookmark(prev, reference)
        : addBookmark(prev, {
            reference,
            label: refShort(reference),
            addedAt: new Date().toISOString(),
          })
      try {
        if (typeof localStorage !== 'undefined') saveBookmarks(localStorage, next)
      } catch {
        // Session-only fallback, same as prefs.
      }
      return next
    })
  }

  // "quran:<slug@ver>:<s>:<a>" → "s:a" for a readable page title.
  const parts = ayahs[0]?.canonical.reference.split(':') ?? []
  const surah = parts.length >= 4 ? parts[2] : undefined
  const verse = parts.length >= 4 ? `${parts[2]}:${parts[3]}` : undefined

  return (
    <div
      data-translation-mode={translationMode}
      data-inspector-mode={inspectorMode}
      data-ayah-size={size}
      data-ayah-layout={layout}
    >
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
                  <button
                    type="button"
                    className="seg px-1.5 text-[0.7rem]"
                    aria-pressed={isBookmarked(bookmarks, ayah.canonical.reference)}
                    aria-label={
                      isBookmarked(bookmarks, ayah.canonical.reference)
                        ? 'bookmarked'
                        : 'bookmark'
                    }
                    title={
                      isBookmarked(bookmarks, ayah.canonical.reference)
                        ? 'bookmarked'
                        : 'bookmark'
                    }
                    onClick={() => toggleBookmark(ayah.canonical.reference)}
                  >
                    {isBookmarked(bookmarks, ayah.canonical.reference)
                      ? 'bookmarked'
                      : 'bookmark'}
                  </button>
                  <span className="ml-auto inline-flex items-center gap-1.5 font-mono text-[0.6rem] uppercase tracking-[0.14em] text-canon">
                    <span className="prodot bg-canon" aria-hidden="true" />
                    canonical
                  </span>
                  <span className="hidden font-mono text-[0.6rem] text-muted-foreground/50 sm:inline">
                    {String(index + 1).padStart(2, '0')}
                  </span>
                </div>

                <div className="mb-4">
                  <RelatedBlock arabicText={ayah.canonical.arabicText} />
                </div>

                {ayah.translations.length === 0 ? (
                  <>
                    <div className="rounded-md border bg-card px-3 py-4 sm:px-5 sm:py-5">
                      <AyahColumn
                        arabicText={ayah.canonical.arabicText}
                        reference={ayah.canonical.reference}
                        size={size}
                      />
                    </div>
                    <p
                      className="ayah-translation mt-4 rounded-md border border-dashed px-3 py-2 text-[0.8125rem] italic"
                      data-empty-translation="true"
                      dir="ltr"
                    >
                      no translation available for this reference
                    </p>
                  </>
                ) : layout === 'side-by-side' ? (
                  <div className="mt-5 grid gap-x-6 gap-y-3 md:grid-cols-2">
                    <div className="rounded-md border bg-card px-3 py-4 sm:px-5 sm:py-5">
                      <AyahColumn
                        arabicText={ayah.canonical.arabicText}
                        reference={ayah.canonical.reference}
                        size={size}
                      />
                    </div>
                    <div dir="ltr" className="min-w-0 space-y-2">
                      {ayah.translations.map((t) => (
                        <TranslationPanel key={t.translator} translator={t.translator} text={t.text} />
                      ))}
                    </div>
                  </div>
                ) : layout === 'interlinear' ? (
                  <div>
                    <div className="rounded-md border bg-card px-3 py-2 sm:px-5 sm:py-3">
                      <AyahColumn
                        arabicText={ayah.canonical.arabicText}
                        reference={ayah.canonical.reference}
                        size={size}
                      />
                    </div>
                    {ayah.translations.map((t) => (
                      <div key={t.translator} data-interlinear dir="ltr" className="mt-2 border-t pt-2">
                        <span className="font-mono text-[0.6rem] uppercase tracking-[0.14em] text-muted-foreground">
                          {t.translator}
                        </span>
                        <p className="mt-1 text-[0.8125rem] leading-relaxed">{t.text}</p>
                      </div>
                    ))}
                  </div>
                ) : (
                  <>
                    <div className="rounded-md border bg-card px-3 py-4 sm:px-5 sm:py-5">
                      <AyahColumn
                        arabicText={ayah.canonical.arabicText}
                        reference={ayah.canonical.reference}
                        size={size}
                      />
                    </div>

                    {translationMode === 'inline' ? (
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
                  </>
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
                layout={layout}
                onTranslationMode={setTranslationMode}
                onInspectorMode={setInspectorMode}
                onLayout={setLayout}
              />
            </div>
          </div>

          <div>
            <p className="kicker mb-2">Text size</p>
            <div className="grid grid-cols-3 gap-0.5 rounded-lg border bg-card p-2.5" role="group" aria-label="canonical text size">
              {SIZES.map((s) => (
                <button
                  key={s}
                  type="button"
                  className={`seg ${s === 'sm' ? 'text-xs' : s === 'md' ? 'text-base' : 'text-xl'}`}
                  aria-pressed={size === s}
                  aria-label={SIZE_LABELS[s]}
                  title={SIZE_LABELS[s]}
                  onClick={() => setSize(s)}
                >
                  A
                </button>
              ))}
            </div>
          </div>

          <div>
            <p className="kicker mb-2">Bookmarks</p>
            {bookmarks.length === 0 ? (
              <EmptyState title="no bookmarks">
                saved verses stay in this browser only — they never enter the corpus, the audit
                trail or an export.
              </EmptyState>
            ) : (
              <div className="rounded-lg border bg-card p-2.5">
                <p className="mb-2 font-mono text-[0.625rem] uppercase tracking-[0.14em] text-muted-foreground">
                  {bookmarks.length} saved
                </p>
                <ul className="space-y-1.5">
                  {bookmarks.map((entry) => {
                    const href = deepLinkFor(entry.reference)
                    return (
                      <li key={entry.reference} className="flex items-center gap-2 text-[0.75rem]">
                        {href === null ? (
                          <span className="font-mono text-foreground">{entry.label}</span>
                        ) : (
                          <a
                            href={href}
                            className="font-mono text-foreground underline decoration-dotted underline-offset-2"
                          >
                            {entry.label}
                          </a>
                        )}
                        <button
                          type="button"
                          className="seg ml-auto px-1.5 text-[0.7rem]"
                          aria-label={`remove bookmark ${entry.label}`}
                          title={`remove bookmark ${entry.label}`}
                          onClick={() => toggleBookmark(entry.reference)}
                        >
                          remove
                        </button>
                      </li>
                    )
                  })}
                </ul>
              </div>
            )}
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
            or re-metrics the canonical text. Bookmarks live in this browser and are not part of
            the corpus, the audit trail or any export.
          </p>
        </aside>
      </div>
    </div>
  )
}
