// Client-side routes. `/read/{slug}@{version}/{surah}:{ayah}` parses to a
// reference string fetched through the typed client; unknown paths fall
// back to the reading hint (the server fallback serves index.html).
import { useEffect, useState } from 'react'
import { AppShell } from '@/components/layout/app-shell'
import { BookLock, Compass } from 'lucide-react'
import ReadingView from './views/ReadingView'
import ResearchView from './views/ResearchView'
import GraphView from './views/GraphView'
import RagDebugView from './views/RagDebugView'
import CitationView from './views/CitationView'
import ExplorerView from './views/ExplorerView'
import { fetchAyahs, type AyahView } from './api/client'

export function parseReadPath(path: string): string | null {
  const [pathname] = path.split('?', 1)
  const match = /^\/read\/([^/]+)\/(\d+:\d+)$/.exec(pathname)
  if (!match) return null
  return match[2]
}

/** Citation id carried on a deep link (`?citation=<id>`), if any. */
export function parseCitationId(path: string): string | null {
  const query = path.split('?', 2)[1]
  if (!query) return null
  const id = new URLSearchParams(query).get('citation')
  return id && id.length > 0 ? id : null
}

export const ROUTES = [
  { path: '/', label: 'Reader', crumb: 'Reader' },
  { path: '/research', label: 'Research', crumb: 'Research' },
  { path: '/graph', label: 'Graph', crumb: 'Graph' },
  { path: '/rag-debug', label: 'RAG Debug', crumb: 'RAG Debug' },
  { path: '/explorer', label: 'API Explorer', crumb: 'API Explorer' },
]

export default function Router({ path }: { path: string }) {
  const [ayahs, setAyahs] = useState<AyahView[] | null>(null)
  const [error, setError] = useState<string | null>(null)
  const reference = parseReadPath(path)

  useEffect(() => {
    if (reference === null) return
    let live = true
    fetchAyahs(reference)
      .then((envelope) => {
        if (live) setAyahs(envelope.data)
      })
      .catch((err: unknown) => {
        if (live) setError(err instanceof Error ? err.message : String(err))
      })
    return () => {
      live = false
    }
  }, [reference])

  let content: React.ReactNode

  if (path === '/research') {
    content = <ResearchView query="" />
  } else if (path === '/graph') {
    content = <GraphView lines={[]} truncated={false} />
  } else if (path === '/rag-debug') {
    content = <RagDebugView />
  } else if (path === '/explorer') {
    content = <ExplorerView />
  } else if (reference === null) {
    content = (
      <section className="mx-auto max-w-xl text-center" aria-label="reading hint">
        <p className="kicker mb-3 justify-center">Codex · Index</p>
        <h1 className="flex items-center justify-center gap-3 text-2xl font-semibold tracking-tight sm:text-3xl">
          <Compass className="size-5 text-gold" aria-hidden="true" />
          The reading room awaits a reference
        </h1>
        <p className="mx-auto mt-2 max-w-md text-[0.8125rem] leading-relaxed text-muted-foreground">
          Open a canonical deep link to unroll a page of the codex.
        </p>
        <code className="chip mt-5 font-mono">/read/test@0.1.0/2:1</code>
      </section>
    )
  } else if (error !== null) {
    content = (
      <div role="alert" className="rounded-xl border border-annot/40 bg-annot-soft p-6 sm:p-8">
        <div className="flex items-start gap-3">
          <BookLock className="mt-0.5 size-5 shrink-0 text-annot" aria-hidden="true" />
          <div>
            <h2 className="font-semibold text-annot">
              No edition is loaded for this reference
            </h2>
            <p className="mt-1 max-w-2xl text-[0.8125rem] leading-relaxed text-annot/85">
              The engine failed closed — canonical text is never fabricated to fill an empty
              reader. Register and activate a Quran edition, then reopen this deep link.
            </p>
          </div>
        </div>
        <code className="mt-4 block rounded-lg border border-annot/30 bg-background px-3.5 py-2.5 font-mono text-xs text-annot sm:w-fit">
          qai db migrate &amp;&amp; qai quran catalog &lt;editions.json&gt; &amp;&amp; qai quran import &lt;manifest&gt;
        </code>
      </div>
    )
  } else if (ayahs === null) {
    content = (
      <div className="py-16 text-center" role="status">
        <span className="prodot mx-auto animate-pulse bg-gold" />
        <p className="mt-3 text-[0.8125rem] text-muted-foreground">unrolling the page…</p>
      </div>
    )
  } else {
    const citationId = parseCitationId(path)
    content = (
      <>
        <ReadingView ayahs={ayahs} onResolve={() => {}} />
        {citationId !== null && <CitationView citationId={citationId} />}
      </>
    )
  }

  return <AppShell currentPath={path}>{content}</AppShell>
}
