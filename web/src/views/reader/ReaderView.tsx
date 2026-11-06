import * as React from 'react'
import { BookOpen, Link2, RefreshCw } from 'lucide-react'
import { fetchAyahs, type AyahView } from '../../api/client'
import { Badge } from '../../components/ui/Badge'
import { Button } from '../../components/ui/Button'
import { EmptyState } from '../../components/ui/EmptyState'
import { ErrorState } from '../../components/ui/ErrorState'
import { Input } from '../../components/ui/Input'
import { Skeleton } from '../../components/ui/Skeleton'

export interface ParsedDeepLink {
  edition: string
  reference: string
  deepLink: string
}

/**
 * Parse a canonical reader deep link (`/read/{slug}@{version}/{surah}:{ayah}`)
 * or a plain `{surah}:{ayah}` reference.
 */
export function parseReaderDeepLink(input: string): ParsedDeepLink | null {
  const trimmed = input.trim()
  const [pathname] = trimmed.split('?', 1)
  const deepMatch = /^\/read\/([^/]+)\/(\d+:\d+)$/.exec(pathname ?? '')
  if (deepMatch && deepMatch[1] && deepMatch[2]) {
    return {
      edition: deepMatch[1],
      reference: deepMatch[2],
      deepLink: `/read/${deepMatch[1]}/${deepMatch[2]}`,
    }
  }
  const refMatch = /^(\d+:\d+)$/.exec(trimmed)
  if (refMatch && refMatch[1]) {
    return {
      edition: 'uthmani@1.0.0',
      reference: refMatch[1],
      deepLink: `/read/uthmani@1.0.0/${refMatch[1]}`,
    }
  }
  return null
}

/**
 * Build `/read/{slug}@{version}/{surah}:{ayah}` from a canonical URN
 * (`quran:slug@version:surah:ayah`) or fallback edition + reference.
 */
export function formatAyahDeepLink(canonicalReference: string, fallbackEdition = 'uthmani@1.0.0'): string {
  const parts = canonicalReference.split(':')
  if (parts.length >= 4 && parts[0] === 'quran') {
    const edition = parts[1] || fallbackEdition
    const surah = parts[2]
    const ayah = parts[3]
    return `/read/${edition}/${surah}:${ayah}`
  }
  return `/read/${fallbackEdition}/${canonicalReference}`
}

export interface ReaderAnnotation {
  id: string
  reference: string
  label: string
  note: string
}

export interface ReaderViewProps {
  initialReference?: string
  deepLinkPath?: string
  initialAyahs?: AyahView[]
  annotations?: ReaderAnnotation[]
}

export function ReaderView({
  initialReference = '1:1',
  deepLinkPath,
  initialAyahs,
  annotations = [],
}: ReaderViewProps) {
  const parsedFromPath = deepLinkPath ? parseReaderDeepLink(deepLinkPath) : null
  const [referenceInput, setReferenceInput] = React.useState<string>(
    parsedFromPath ? parsedFromPath.deepLink : initialReference,
  )
  const [activeRef, setActiveRef] = React.useState<string>(
    parsedFromPath ? parsedFromPath.reference : initialReference,
  )
  const [activeEdition, setActiveEdition] = React.useState<string>(
    parsedFromPath ? parsedFromPath.edition : 'uthmani@1.0.0',
  )
  const [ayahs, setAyahs] = React.useState<AyahView[] | null>(initialAyahs ?? null)
  const [loading, setLoading] = React.useState<boolean>(initialAyahs === undefined)
  const [error, setError] = React.useState<string | null>(null)
  const [reloadTick, setReloadTick] = React.useState<number>(0)

  React.useEffect(() => {
    if (initialAyahs !== undefined && reloadTick === 0) {
      setAyahs(initialAyahs)
      setLoading(false)
      return
    }
    if (!activeRef) {
      setAyahs([])
      setLoading(false)
      return
    }

    let cancelled = false
    setLoading(true)
    setError(null)

    fetchAyahs(activeRef)
      .then((envelope) => {
        if (cancelled) return
        setAyahs(envelope.data)
        if (envelope.meta?.edition?.slug && envelope.meta?.edition?.version) {
          setActiveEdition(`${envelope.meta.edition.slug}@${envelope.meta.edition.version}`)
        }
        setLoading(false)
      })
      .catch((err: unknown) => {
        if (cancelled) return
        setError(err instanceof Error ? err.message : String(err))
        setLoading(false)
      })

    return () => {
      cancelled = true
    }
  }, [activeRef, initialAyahs, reloadTick])

  function handleNavigate(event: React.FormEvent) {
    event.preventDefault()
    const parsed = parseReaderDeepLink(referenceInput)
    if (parsed) {
      setActiveEdition(parsed.edition)
      setActiveRef(parsed.reference)
      setReloadTick((t) => t + 1)
    } else if (referenceInput.trim().length > 0) {
      setActiveRef(referenceInput.trim())
      setReloadTick((t) => t + 1)
    }
  }

  return (
    <section
      aria-label="Quran Reader"
      data-slot="reader-view"
      className="mx-auto max-w-4xl px-6 py-10"
    >
      <header className="mb-8 flex flex-col gap-4 border-b pb-6 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <div className="mb-2 flex items-center gap-2">
            <Badge variant="canonical" dot>
              Canonical Layer
            </Badge>
            <Badge variant="neutral">{activeEdition}</Badge>
          </div>
          <h1 className="text-2xl font-semibold tracking-tight text-foreground">
            Quran Reader
          </h1>
          <p className="mt-1 text-xs text-muted-foreground">
            Exact canonical Arabic text separated structurally from translations and annotations.
          </p>
        </div>

        <form onSubmit={handleNavigate} className="flex items-center gap-2" aria-label="Reference navigation">
          <Input
            aria-label="Reference or deep link"
            value={referenceInput}
            onChange={(e) => setReferenceInput(e.target.value)}
            placeholder="/read/uthmani@1.0.0/1:1"
            className="w-56 font-mono text-xs"
          />
          <Button type="submit" variant="primary" size="sm">
            <BookOpen className="mr-1 size-3.5" aria-hidden="true" />
            Open
          </Button>
        </form>
      </header>

      {loading && (
        <div data-testid="reader-loading" className="space-y-6 py-6">
          <Skeleton lines={3} label="Loading canonical ayahs" />
          <Skeleton lines={2} label="Loading translations" />
        </div>
      )}

      {!loading && error !== null && (
        <ErrorState
          title="Failed to load canonical passage"
          message={error}
          onRetry={() => setReloadTick((t) => t + 1)}
        />
      )}

      {!loading && error === null && (ayahs === null || ayahs.length === 0) && (
        <EmptyState
          title="No ayahs loaded for this reference"
          description="Select a valid surah:ayah reference or deep link (/read/{slug}@{version}/{surah}:{ayah}) to inspect canonical scripture."
          action={
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={() => {
                setReferenceInput('/read/uthmani@1.0.0/1:1')
                setActiveRef('1:1')
                setReloadTick((t) => t + 1)
              }}
            >
              <RefreshCw className="mr-1 size-3.5" aria-hidden="true" />
              Load 1:1
            </Button>
          }
        />
      )}

      {!loading && error === null && ayahs !== null && ayahs.length > 0 && (
        <div className="space-y-10" data-testid="reader-ayah-list">
          {ayahs.map((item) => {
            const deepLink = formatAyahDeepLink(item.canonical.reference, activeEdition)
            const matchingAnnotations = annotations.filter(
              (ann) =>
                ann.reference === item.canonical.reference || ann.reference === activeRef,
            )

            return (
              <article
                key={item.canonical.reference}
                data-slot="reader-ayah-card"
                data-reference={item.canonical.reference}
                className="rounded-lg border border-border bg-card p-6 shadow-xs"
              >
                <div className="mb-4 flex flex-wrap items-center justify-between gap-2 border-b pb-3">
                  <div className="flex items-center gap-2">
                    <Badge variant="canonical" dot>
                      Canonical
                    </Badge>
                    <span className="font-mono text-xs text-muted-foreground">
                      {item.canonical.reference}
                    </span>
                  </div>
                  <a
                    href={deepLink}
                    data-slot="ayah-deep-link"
                    className="inline-flex items-center gap-1 font-mono text-xs text-muted-foreground hover:text-foreground hover:underline"
                  >
                    <Link2 className="size-3" aria-hidden="true" />
                    {deepLink}
                  </a>
                </div>

                {/* Canonical Arabic layer: strictly isolated in --layer-canonical-* with RTL + unicode-bidi: isolate */}
                <div className="py-4">
                  <p
                    dir="rtl"
                    lang="ar"
                    data-layer="canonical"
                    style={{
                      unicodeBidi: 'isolate',
                      color: 'var(--layer-canonical-fg)',
                      backgroundColor: 'var(--layer-canonical-bg)',
                      fontFamily: 'var(--layer-font-arabic)',
                    }}
                    className="ayah-canonical text-[2rem] leading-[2.15]"
                  >
                    {item.canonical.arabicText}
                  </p>
                </div>

                {/* Translation layer: strictly separated under --layer-translation-* in neutral sans */}
                <div className="mt-4 space-y-3 border-t pt-4" dir="ltr" data-layer="translation">
                  <div className="flex items-center gap-2">
                    <Badge variant="translation">Translation</Badge>
                  </div>
                  {item.translations.length === 0 ? (
                    <p
                      className="ayah-translation text-xs italic"
                      style={{
                        color: 'var(--layer-translation-fg)',
                        backgroundColor: 'var(--layer-translation-bg)',
                      }}
                    >
                      No translation available for this reference.
                    </p>
                  ) : (
                    item.translations.map((tr) => (
                      <div
                        key={tr.translator}
                        data-translator={tr.translator}
                        style={{
                          color: 'var(--layer-translation-fg)',
                          backgroundColor: 'var(--layer-translation-bg)',
                        }}
                        className="ayah-translation border-l-2 border-dashed pl-3 font-sans"
                      >
                        <span className="block font-mono text-[0.7rem] uppercase tracking-wider text-muted-foreground">
                          {tr.translator}
                        </span>
                        <p className="mt-1 text-sm leading-relaxed">{tr.text}</p>
                      </div>
                    ))
                  )}
                </div>

                {/* Annotation layer: strictly separated under --layer-annotation-* with left rule */}
                <div className="mt-4 border-t pt-4" dir="ltr" data-layer="annotation">
                  <div className="mb-2 flex items-center gap-2">
                    <Badge variant="annotation">Annotation</Badge>
                  </div>
                  {matchingAnnotations.length === 0 ? (
                    <div
                      data-slot="annotation-item"
                      style={{
                        color: 'var(--layer-annotation-fg)',
                        backgroundColor: 'var(--layer-annotation-bg)',
                        borderLeft: 'var(--layer-annotation-border, 3px double currentColor)',
                      }}
                      className="ayah-annotation rounded-r-md pl-3 py-1.5 text-xs"
                    >
                      Lexical &amp; manuscript annotations attach per token when morphology is active.
                    </div>
                  ) : (
                    <div className="space-y-2">
                      {matchingAnnotations.map((ann) => (
                        <div
                          key={ann.id}
                          data-slot="annotation-item"
                          style={{
                            color: 'var(--layer-annotation-fg)',
                            backgroundColor: 'var(--layer-annotation-bg)',
                            borderLeft: 'var(--layer-annotation-border, 3px double currentColor)',
                          }}
                          className="ayah-annotation rounded-r-md pl-3 py-1.5 text-xs"
                        >
                          <span className="font-mono font-medium">{ann.label}: </span>
                          <span>{ann.note}</span>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              </article>
            )
          })}
        </div>
      )}
    </section>
  )
}

export default ReaderView
