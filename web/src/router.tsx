import { useEffect, useState } from 'react'
import { AppShell } from '@/components/layout/app-shell'
import { PageHeader } from '@/components/layout/page-header'
import { EmptyState, LoadingLine } from '@/components/states'
import { BookLock } from 'lucide-react'
import ReadingView from './views/ReadingView'
import ResearchView from './views/ResearchView'
import SearchView from './views/SearchView'
import HadithView from './views/HadithView'
import TafsirView from './views/TafsirView'
import ScriptureView from './views/ScriptureView'
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

// Rendered when the reader has no reference. It navigates with the same
// deep link the server already understands — no new API surface.
const DEEP_LINK = /^\/read\/[^/]+\/\d+:\d+(\?.*)?$/

function ReferenceForm() {
  const [value, setValue] = useState('/read/test@0.1.0/2:1')
  const [invalid, setInvalid] = useState(false)

  return (
    <form
      onSubmit={(event) => {
        event.preventDefault()
        if (!DEEP_LINK.test(value)) {
          setInvalid(true)
          return
        }
        window.location.assign(value)
      }}
    >
      <label htmlFor="deep-link" className="kicker mb-2 block">
        Deep link
      </label>
      <div className="flex gap-2">
        <input
          id="deep-link"
          value={value}
          onChange={(event) => {
            setValue(event.target.value)
            setInvalid(false)
          }}
          aria-invalid={invalid}
          spellCheck={false}
          className="field flex-1 font-mono text-xs"
        />
        <button type="submit" className="btn btn-primary shrink-0">
          Open reference
        </button>
      </div>
      {invalid && (
        <p role="alert" className="mt-1.5 text-[0.7rem] text-destructive">
          Use /read/&lt;edition&gt;/&lt;surah&gt;:&lt;ayah&gt;
        </p>
      )}
    </form>
  )
}

export const ROUTES = [
  { path: '/', label: 'Reader', crumb: 'Reader' },
  { path: '/search', label: 'Search', crumb: 'Search' },
  { path: '/research', label: 'Research', crumb: 'Research' },
  { path: '/hadith', label: 'Hadith', crumb: 'Hadith' },
  { path: '/tafsir', label: 'Tafsir', crumb: 'Tafsir' },
  { path: '/scripture', label: 'Scripture', crumb: 'Scripture' },
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

  if (path === '/search') {
    content = <SearchView />
  } else if (path === '/research') {
    content = <ResearchView query="" />
  } else if (path === '/hadith') {
    content = <HadithView />
  } else if (path === '/tafsir') {
    content = <TafsirView />
  } else if (path === '/scripture') {
    content = <ScriptureView />
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
          lede="The reading room opens on a canonical reference. Paste a deep link to unroll a page — the engine verifies every reference before it renders."
        />
        <div className="mt-8 grid gap-4 sm:grid-cols-[minmax(0,1fr)_minmax(0,20rem)]">
          <EmptyState title="No reference is open">
            A reader never invents a verse. Open a reference from a citation, the command menu, or
            the link field beside this note.
          </EmptyState>
          <div className="rounded-lg border bg-card p-4">
            <ReferenceForm />
            <p className="margin-note mt-3">
              The reference is parsed by the engine; this page never constructs a verse locally.
            </p>
          </div>
        </div>
        <nav aria-label="Workspace directory" className="mt-10">
          <p className="kicker mb-2">Directory</p>
          <ul className="divide-y overflow-hidden rounded-lg border bg-card">
            {[
              { path: '/search', label: 'Search', note: 'five lexical search modes over the typed engine' },
              { path: '/research', label: 'Research', note: 'nine lexical instruments over the typed engine' },
              { path: '/hadith', label: 'Hadith', note: 'fetch by collection and number, or search' },
              { path: '/tafsir', label: 'Tafsir', note: 'fetch a work by slug, or search across works' },
              { path: '/scripture', label: 'Scripture', note: 'editions, search, parallel, research' },
              { path: '/graph', label: 'Graph', note: 'neighbors and paths from the frozen payload' },
              { path: '/rag-debug', label: 'RAG Debug', note: 'sealed vault — retrieval lands in Phase 8' },
              { path: '/explorer', label: 'Explorer', note: 'dispatch any request against the local engine' },
            ].map((item) => (
              <li key={item.path}>
                <a
                  href={item.path}
                  className="flex items-baseline gap-4 px-4 py-3 transition-colors hover:bg-muted/40"
                >
                  <span className="text-[0.8125rem] font-medium">{item.label}</span>
                  <span className="truncate text-[0.75rem] text-muted-foreground">{item.note}</span>
                  <span className="ml-auto font-mono text-[0.65rem] text-muted-foreground/60" aria-hidden="true">
                    →
                  </span>
                </a>
              </li>
            ))}
          </ul>
        </nav>
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
