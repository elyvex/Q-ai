// Citations workbench (distinct from the inline `CitationView` under a
// reader page): an id in, the server's verification evidence out — verdict,
// corpus `text_hash`, URN and deep link — plus a re-verify control and a
// byte-exact comparison against the text the deep link names. The verdict
// panel renders only server data and is never computed here; the local
// comparison below it cannot forge or repair a verdict. No hashing code
// lives in this file: the only hash shown is the string the endpoint
// returned, labelled as the server's corpus fingerprint.
import { useState } from 'react'
import { fetchAyahs, fetchCitation } from '../api/client'
import { PageHeader } from '../components/layout/page-header'
import { EmptyState, ErrorNote, LoadingLine } from '../components/states'
import { byteDiff, describeDiff } from '../lib/byte-diff'

/** The endpoint serializes unit verdicts as a plain string and struct
 * verdicts as a single-key object (`{"Mismatch": {...}}`); both read as
 * their variant name. Anything else prints as-is, never as a verdict. */
function verdictLabel(verdict: unknown): string {
  if (typeof verdict === 'string') return verdict
  if (verdict !== null && typeof verdict === 'object' && !Array.isArray(verdict)) {
    const keys = Object.keys(verdict)
    if (keys.length === 1) return keys[0] as string
  }
  return String(verdict)
}

interface CitationData {
  citation_id?: string
  verdict?: unknown
  text_hash?: string
  urn?: string
  deep_link?: string | null
}

type VerdictState =
  | { status: 'idle' }
  | { status: 'loading' }
  | { status: 'failure'; code?: string; summary: string; remedy?: string }
  | {
      status: 'ready'
      citationId: string
      verdict: string
      textHash: string
      urn: string
      deepLink: string | null
    }

type TextState =
  | { status: 'idle' }
  | { status: 'loading' }
  | { status: 'none' }
  | {
      status: 'ready'
      canonical: string
      editionSlug: string
      editionVersion: string
      linkSlug: string | null
    }

/** Trailing `surah:ayah` of a reader deep link (`/read/<slug>@<v>/<ref>`). */
function referenceOf(deepLink: string): string {
  const last = deepLink.split('/').pop() ?? deepLink
  return last
}

/** Edition slug carried by a reader deep link, if it parses. */
function slugOf(deepLink: string): string | null {
  const match = /^\/read\/([^@/\s]+)@[^/]+\//.exec(deepLink)
  return match?.[1] ?? null
}

/** Up to eight code points either side of `index` — the context the
 * mismatch readout prints beside the index and the two labels. */
function contextOf(value: string, index: number): string {
  return Array.from(value)
    .slice(Math.max(0, index - 8), index + 8)
    .join('')
}

function failureOf(error: unknown): { code?: string; summary: string; remedy?: string } {
  if (error instanceof Error) {
    const code = (error as { code?: unknown }).code
    const remedy = (error as { remedy?: unknown }).remedy
    const failure: { code?: string; summary: string; remedy?: string } = {
      summary: error.message,
    }
    if (typeof code === 'string' && code.length > 0) failure.code = code
    if (typeof remedy === 'string' && remedy.length > 0) failure.remedy = remedy
    return failure
  }
  return { summary: String(error) }
}

export default function CitationsWorkbenchView({ initialId }: { initialId: string }) {
  const [id, setId] = useState(initialId)
  const [verdict, setVerdict] = useState<VerdictState>({ status: 'idle' })
  const [text, setText] = useState<TextState>({ status: 'idle' })
  const [localCopy, setLocalCopy] = useState<string | null>(null)
  const [verifications, setVerifications] = useState(0)

  const openDisabled = id.trim() === ''

  async function open(citationId: string) {
    const trimmed = citationId.trim()
    if (trimmed === '') return
    setVerdict({ status: 'loading' })
    setText({ status: 'idle' })
    setLocalCopy(null)
    let data: CitationData
    try {
      const envelope = await fetchCitation(trimmed)
      data = envelope.data as CitationData
    } catch (error: unknown) {
      // Hard failure from the server: typed code/summary/remedy, never a
      // softened success.
      setVerdict({ status: 'failure', ...failureOf(error) })
      return
    }
    setVerifications((count) => count + 1)
    const deepLink =
      typeof data.deep_link === 'string' && data.deep_link.length > 0 ? data.deep_link : null
    setVerdict({
      status: 'ready',
      citationId: data.citation_id ?? trimmed,
      verdict: verdictLabel(data.verdict),
      textHash: data.text_hash ?? '',
      urn: data.urn ?? '',
      deepLink,
    })
    if (deepLink === null) {
      setText({ status: 'none' })
      return
    }
    setText({ status: 'loading' })
    try {
      const envelope = await fetchAyahs(referenceOf(deepLink))
      const first = envelope.data[0]
      const canonical = first?.canonical.arabicText ?? ''
      setLocalCopy(canonical)
      setText({
        status: 'ready',
        canonical,
        editionSlug: envelope.meta.edition.slug,
        editionVersion: envelope.meta.edition.version,
        linkSlug: slugOf(deepLink),
      })
    } catch (error: unknown) {
      setText({ status: 'idle' })
      setVerdict({ status: 'failure', ...failureOf(error) })
    }
  }

  const baseline =
    text.status === 'ready' && localCopy !== null ? byteDiff(text.canonical, localCopy) : null

  return (
    <section>
      <PageHeader
        eyebrow="Instruments · Provenance"
        title="Citations"
        lede="The verdict below comes from the server and is never computed here. This screen shows the verification evidence for one stored citation — and what a changed character does to a local copy of its text."
      />

      <form
        className="mt-6 flex flex-wrap items-end gap-2"
        onSubmit={(event) => {
          event.preventDefault()
          void open(id)
        }}
      >
        <div className="min-w-0 grow basis-64">
          <label htmlFor="citation-id" className="kicker mb-2 block">
            Citation id
          </label>
          <input
            id="citation-id"
            value={id}
            onChange={(event) => setId(event.target.value)}
            spellCheck={false}
            className="field font-mono text-xs"
          />
        </div>
        <button type="submit" className="btn btn-primary" disabled={openDisabled}>
          Open
        </button>
        {openDisabled && <p className="w-full text-[0.75rem] text-muted-foreground">enter a citation id</p>}
      </form>

      {verdict.status === 'loading' && (
        <div className="mt-6">
          <LoadingLine label="verifying citation — against the server, never locally…" />
        </div>
      )}

      {verdict.status === 'failure' && (
        <ErrorNote>
          {verdict.code !== undefined && (
            <>
              <span className="font-mono">{verdict.code}</span>
              {' — '}
            </>
          )}
          {verdict.summary}
          {verdict.remedy !== undefined && (
            <>
              {' — '}
              {verdict.remedy}
            </>
          )}
        </ErrorNote>
      )}

      {verdict.status === 'ready' && (
        <div className="mt-6 overflow-hidden rounded-lg border">
          <div className="flex flex-wrap items-center gap-x-4 gap-y-1 border-b bg-card px-4 py-2.5">
            <p className="font-mono text-[0.7rem] font-semibold">{verdict.citationId}</p>
            <p className="font-mono text-[0.7rem] text-muted-foreground">
              verdict {verdict.verdict}
            </p>
            <button
              type="button"
              className="btn btn-quiet ml-auto h-7 px-2 text-[0.75rem]"
              onClick={() => void open(id)}
            >
              Re-verify
            </button>
          </div>
          <dl className="space-y-2 px-4 py-3 text-[0.8125rem]">
            <div className="flex flex-wrap gap-x-2">
              <dt className="kicker">text hash</dt>
              <dd className="break-all font-mono text-[0.72rem]">{verdict.textHash}</dd>
            </div>
            <div className="flex flex-wrap gap-x-2">
              <dt className="kicker">urn</dt>
              <dd className="break-all font-mono text-[0.72rem]">{verdict.urn}</dd>
            </div>
            {verdict.deepLink !== null && (
              <div className="flex flex-wrap gap-x-2">
                <dt className="kicker">deep link</dt>
                <dd>
                  <a
                    href={verdict.deepLink}
                    className="break-all font-mono text-[0.72rem] text-primary underline-offset-4 hover:underline"
                  >
                    {verdict.deepLink}
                  </a>
                </dd>
              </div>
            )}
          </dl>
          <p className="margin-note border-t px-4 py-2.5">
            {verifications === 1
              ? 'verified once against the server in this session'
              : `verified ${verifications} times against the server in this session`}
          </p>
        </div>
      )}

      {text.status === 'loading' && (
        <div className="mt-6">
          <LoadingLine label="fetching the text the deep link names…" />
        </div>
      )}

      {text.status === 'none' && (
        <div className="mt-6">
          <EmptyState title="no deep link on this citation">
            This citation carries no reader location, so there is no text to compare.
          </EmptyState>
        </div>
      )}

      {text.status === 'ready' && (
        <div className="mt-6 overflow-hidden rounded-lg border">
          <div className="flex flex-wrap items-center gap-2 border-b bg-card px-4 py-2.5">
            <p className="font-mono text-[0.7rem] font-semibold">Tamper simulation</p>
            <p className="font-mono text-[0.7rem] text-muted-foreground">
              text · {text.editionSlug}@{text.editionVersion}
            </p>
            <button
              type="button"
              className="btn btn-quiet ml-auto h-7 px-2 text-[0.75rem]"
              onClick={() => setLocalCopy(text.canonical)}
            >
              Restore
            </button>
          </div>
          <div className="space-y-3 px-4 py-3">
            {text.linkSlug !== null && text.linkSlug !== text.editionSlug && (
              <p className="margin-note">
                This citation points at edition {text.linkSlug}, which differs from the active
                edition {text.editionSlug} — the comparison below is against the active
                edition&apos;s text.
              </p>
            )}
            <div>
              <label htmlFor="tamper-copy" className="kicker mb-2 block">
                Local copy — edit to see what changes
              </label>
              <textarea
                id="tamper-copy"
                value={localCopy ?? ''}
                onChange={(event) => setLocalCopy(event.target.value)}
                dir="rtl"
                lang="ar"
                rows={3}
                spellCheck={false}
                className="field font-arabic text-base leading-loose"
              />
            </div>
            <div dir="ltr">
              <p className="font-mono text-[0.72rem] text-muted-foreground">
                {baseline !== null ? describeDiff(baseline) : 'comparing…'}
              </p>
              {baseline !== null && baseline.firstMismatch !== null && (
                <div className="mt-1.5 space-y-0.5 font-mono text-[0.72rem] text-muted-foreground">
                  <p>expected: {contextOf(text.canonical, baseline.firstMismatch.index)}</p>
                  <p>found: {contextOf(localCopy ?? '', baseline.firstMismatch.index)}</p>
                </div>
              )}
            </div>
            <p className="margin-note border-t pt-2.5">
              This edits a copy held in the browser. It does not write to the corpus, it does
              not change the stored verdict, and it does not recompute text_hash — the hash is
              a corpus fingerprint over the edition slug, its version and the ordered texts,
              not a plain digest of one verse.
            </p>
          </div>
        </div>
      )}
    </section>
  )
}
