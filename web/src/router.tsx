// Client-side routes. `/read/{slug}@{version}/{surah}:{ayah}` parses to a
// reference string fetched through the typed client; unknown paths fall
// back to the reading hint (the server fallback serves index.html).
import { useEffect, useState } from 'react'
import { AppShell } from '@/components/layout/app-shell'
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
      <div className="rounded-xl border border-dashed p-10 text-center text-muted-foreground">
        Reading view — open a{' '}
        <code className="rounded bg-muted px-1.5 py-0.5 font-mono text-sm">/read/…</code> deep link
      </div>
    )
  } else if (error !== null) {
    content = (
      <div role="alert" className="rounded-2xl border border-amber-300 bg-amber-50 p-6 dark:border-amber-800 dark:bg-amber-950/30">
        <h2 className="font-medium text-amber-800 dark:text-amber-200">No edition is loaded for this reference</h2>
        <p className="mt-1 max-w-2xl text-sm text-amber-800/80 dark:text-amber-200/80">
          The engine failed closed — canonical text is never fabricated to fill an empty reader.
          Register and activate a Quran edition, then reopen this deep link.
        </p>
        <code className="mt-3 block rounded-lg bg-amber-100/60 px-3 py-2 font-mono text-xs text-amber-900 dark:bg-amber-900/40 dark:text-amber-100">
          qai db migrate && qai quran catalog &lt;editions.json&gt; && qai quran import &lt;manifest&gt;
        </code>
      </div>
    )
  } else if (ayahs === null) {
    content = <p className="text-muted-foreground">loading…</p>
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
