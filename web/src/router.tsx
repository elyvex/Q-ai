import { useEffect, useState } from 'react'
import { AppShell } from '@/components/layout/app-shell'
import { PageHeader } from '@/components/layout/page-header'
import { EmptyState, LoadingLine } from '@/components/states'
import { BookLock, CornerDownRight } from 'lucide-react'
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
      <section>
        <PageHeader
          eyebrow="Qurʾān · Canonical edition"
          title="Reader"
          lede="The reading room opens on a canonical reference. Copy a deep link from a citation or the command menu to unroll a page."
        />
        <div className="mt-8 grid gap-4 sm:grid-cols-[minmax(0,1fr)_minmax(0,18rem)]">
          <EmptyState title="No reference is open">
            <span className="inline-flex items-center gap-1.5">
              Open a canonical deep link
              <CornerDownRight className="size-3.5" aria-hidden="true" />
              and the reader will verify it against the corpus.
            </span>
          </EmptyState>
          <div className="rounded-lg border bg-card p-4">
            <p className="kicker mb-2">Deep link</p>
            <code className="block break-all rounded border bg-muted px-2.5 py-2 font-mono text-[0.7rem]">
              /read/test@0.1.0/2:1
            </code>
            <p className="margin-note mt-3">
              The reference is parsed by the engine; this page never constructs a verse locally.
            </p>
          </div>
        </div>
      </section>
    )
  } else if (error !== null) {
    content = (
      <section>
        <PageHeader eyebrow="Qurʾān · Canonical edition" title="No edition is loaded" />
        <div role="alert" className="mt-8 rounded-lg border border-annot/40 bg-annot-soft p-6 sm:p-8">
          <div className="flex items-start gap-3">
            <BookLock className="mt-0.5 size-5 shrink-0 text-annot" aria-hidden="true" />
            <div>
              <h2 className="text-[0.9375rem] font-medium text-foreground">
                The engine failed closed for this reference
              </h2>
              <p className="mt-1 max-w-2xl text-[0.8125rem] leading-relaxed text-muted-foreground">
                Canonical text is never fabricated to fill an empty reader. Register and activate a
                Quran edition, then reopen this deep link.
              </p>
            </div>
          </div>
          <code className="mt-4 block rounded border bg-background px-3.5 py-2.5 font-mono text-[0.7rem] text-foreground/80 sm:w-fit">
            qai db migrate &amp;&amp; qai quran catalog &lt;editions.json&gt; &amp;&amp; qai quran import
            &lt;manifest&gt;
          </code>
        </div>
      </section>
    )
  } else if (ayahs === null) {
    content = (
      <div className="py-24">
        <LoadingLine label="unrolling the page…" />
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
