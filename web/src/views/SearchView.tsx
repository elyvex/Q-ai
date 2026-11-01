// Search view: all five lexical search modes over the typed client.
// Hits render in service-returned order; the client never re-sorts.
import { useState } from 'react'
import { Loader2 } from "lucide-react"
import { PageHeader } from '../components/layout/page-header'
import { EmptyState, ErrorNote } from '../components/states'
import { search, type SearchMode } from '../api/client'

const MODES: { id: SearchMode; label: string; hint: string }[] = [
  { id: 'exact', label: 'Exact', hint: 'byte-exact token match' },
  { id: 'normalized', label: 'Normalized', hint: 'diacritic-insensitive normalized match' },
  { id: 'phrase', label: 'Phrase', hint: 'contiguous phrase match' },
  { id: 'concatenated', label: 'Concatenated', hint: 'concatenated-form match' },
  { id: 'regex', label: 'Regex', hint: 'regular-expression match' },
]

interface Hit {
  reference: string
  snippet: string
}

function hitsOf(envelope: { data: unknown }): Hit[] {
  const data = envelope.data as {
    results?: { hits?: Array<{ reference?: string; quotation?: { arabic_text?: string } }> }
  }
  const hits = data.results?.hits
  if (!Array.isArray(hits)) return []
  return hits.map((h) => ({ reference: h.reference ?? '?', snippet: h.quotation?.arabic_text ?? '' }))
}

export default function SearchView() {
  const [mode, setMode] = useState<SearchMode>('normalized')
  const [term, setTerm] = useState('')
  const [hits, setHits] = useState<Hit[] | null>(null)
  const [running, setRunning] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const current = MODES.find((m) => m.id === mode)!

  async function run() {
    setError(null)
    setRunning(true)
    try {
      setHits(hitsOf(await search(mode, term)))
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setRunning(false)
    }
  }

  return (
    <section>
      <PageHeader
        eyebrow="Qurʾān · Search"
        title="Search"
        lede="Five modes over the typed engine. Hits keep service order; the client never re-sorts."
        actions={hits === null ? null : <span className="chip">{hits.length} hits</span>}
      />

      <div className="mt-8 grid gap-x-10 gap-y-8 lg:grid-cols-[13rem_minmax(0,1fr)]">
        <nav aria-label="Search modes">
          <p className="kicker mb-2.5">Modes</p>
          <div className="grid grid-cols-2 gap-0.5 sm:grid-cols-3 lg:grid-cols-1">
            {MODES.map((m) => (
              <button
                key={m.id}
                type="button"
                aria-pressed={mode === m.id}
                title={m.hint}
                onClick={() => setMode(m.id)}
                className={`flex items-center gap-2.5 rounded-md px-2.5 py-1.5 text-left text-[0.8125rem] transition-colors ${
                  mode === m.id
                    ? 'bg-accent font-medium text-accent-foreground'
                    : 'text-muted-foreground hover:bg-muted hover:text-foreground'
                }`}
              >
                {m.label}
              </button>
            ))}
          </div>
          <p className="margin-note mt-4 hidden border-t pt-3 lg:block">{current.hint}.</p>
        </nav>

        <div className="min-w-0">
          <div className="flex flex-col gap-2 sm:flex-row">
            <input
              aria-label="search query"
              value={term}
              onChange={(e) => setTerm(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') void run()
              }}
              dir="auto"
              placeholder={current.hint}
              className="field h-9 flex-1"
            />
            <button
              type="button"
              className="btn btn-primary h-9 justify-center px-4"
              onClick={() => void run()}
              disabled={running}
            >
              {running && <Loader2 className="size-3.5 animate-spin" aria-hidden="true" />}
              {running ? 'searching…' : 'Search'}
            </button>
          </div>

          {error !== null && <ErrorNote>{error}</ErrorNote>}

          {hits !== null && (
            <div className="mt-6">
              {hits.length === 0 ? (
                <EmptyState title="No matches">no results for “{term}” in {current.label} mode.</EmptyState>
              ) : (
                <ol className="divide-y overflow-hidden rounded-lg border bg-card">
                  {hits.map((h, i) => (
                    <li
                      key={h.reference + ':' + i}
                      className="flex flex-col gap-1.5 px-4 py-3.5 transition-colors hover:bg-muted/40 sm:flex-row sm:items-baseline sm:gap-5"
                    >
                      <span className="chip shrink-0 self-start sm:self-auto">{h.reference}</span>
                      <span className="font-arabic text-lg leading-[1.95] text-foreground/90" dir="rtl" lang="ar">
                        {h.snippet}
                      </span>
                    </li>
                  ))}
                </ol>
              )}
            </div>
          )}

          {hits === null && (
            <div className="mt-6">
              <EmptyState title={current.label}>{current.hint}. Enter a term and search.</EmptyState>
            </div>
          )}
        </div>
      </div>
    </section>
  )
}
