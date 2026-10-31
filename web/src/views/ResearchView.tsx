// Research view: search + root/lemma/family/frequency/distribution/
// co-occurrence over the typed client. Hits render in service-returned
// order — the client never re-sorts. Zero results render an explicit empty
// state, never a blank panel. Dataset text renders as text.
import { useState } from 'react'
import { Loader2, SearchX } from 'lucide-react'
import { PageHeader } from '../components/layout/page-header'
import { EmptyState, ErrorNote } from '../components/states'
import {
  cooccurrence,
  distribution,
  family,
  frequency,
  lemmaFrequency,
  rootFrequency,
  runTool,
  search,
  type SearchMode,
} from '../api/client'

export interface ResearchHit {
  reference: string
  snippet: string
}

type ResearchKind =
  | 'search'
  | 'root'
  | 'lemma'
  | 'family'
  | 'root-frequency'
  | 'lemma-frequency'
  | 'frequency'
  | 'distribution'
  | 'cooccurrence'

const KINDS: { id: ResearchKind; label: string; hint: string }[] = [
  { id: 'search', label: 'Search', hint: 'normalized lexical search over the serving index' },
  { id: 'root', label: 'Root', hint: 'all forms sharing one triliteral root' },
  { id: 'lemma', label: 'Lemma', hint: 'all derived forms of one lemma' },
  { id: 'family', label: 'Family', hint: 'word-family relations for a token' },
  { id: 'root-frequency', label: 'Root freq', hint: 'rules-blocked root frequency' },
  { id: 'lemma-frequency', label: 'Lemma freq', hint: 'exact lemma frequency' },
  { id: 'frequency', label: 'Frequency', hint: 'exact token frequency under a profile' },
  { id: 'distribution', label: 'Distribution', hint: 'the same, partitioned by surah' },
  { id: 'cooccurrence', label: 'Co-occurrence', hint: 'windowed co-occurrence' },
]

function hitsOf(envelope: { data: unknown }): ResearchHit[] {
  const data = envelope.data as {
    results?: { hits?: Array<{ reference?: string; quotation?: { arabic_text?: string } }> }
  }
  const hits = data.results?.hits
  if (!Array.isArray(hits)) return []
  return hits.map((hit) => ({
    reference: hit.reference ?? '?',
    snippet: hit.quotation?.arabic_text ?? '',
  }))
}

async function runKind(kind: ResearchKind, query: string): Promise<ResearchHit[]> {
  switch (kind) {
    case 'search': {
      const mode: SearchMode = 'normalized'
      return hitsOf(await search(mode, query))
    }
    case 'root':
      return hitsOf(await runTool('quran.root', { root: query }))
    case 'lemma':
      return hitsOf(await runTool('quran.lemma', { lemma: query }))
    case 'family':
      return hitsOf(await family('token', query))
    case 'root-frequency':
      return hitsOf(await rootFrequency(query))
    case 'lemma-frequency':
      return hitsOf(await lemmaFrequency(query))
    case 'frequency':
      return hitsOf(await frequency(query, 'default'))
    case 'distribution':
      return hitsOf(await distribution(query, 'default'))
    case 'cooccurrence':
      return hitsOf(await cooccurrence(query, 'default', 5))
  }
}

export function ResearchResults({ query, hits }: { query: string; hits: ResearchHit[] }) {
  if (hits.length === 0) {
    return (
      <EmptyState icon={SearchX} title="No matches">
        <span className="research-empty">
          no results for “{query}” — try a different spelling or normalization
        </span>
      </EmptyState>
    )
  }
  return (
    <ol className="divide-y overflow-hidden rounded-lg border bg-card">
      {hits.map((hit, i) => (
        <li
          key={hit.reference + ':' + i}
          className="research-hit flex flex-col gap-1.5 px-4 py-3.5 transition-colors hover:bg-muted/40 sm:flex-row sm:items-baseline sm:gap-5"
        >
          <span className="chip shrink-0 self-start sm:self-auto">{hit.reference}</span>
          <span
            className="font-arabic text-lg leading-[1.95] text-foreground/90"
            dir="rtl"
            lang="ar"
          >
            {hit.snippet}
          </span>
        </li>
      ))}
    </ol>
  )
}

export default function ResearchView({
  query,
  hits,
}: {
  query: string
  hits?: ResearchHit[]
}) {
  const [kind, setKind] = useState<ResearchKind>('search')
  const [term, setTerm] = useState(query)
  const [live, setLive] = useState<ResearchHit[] | null>(hits ?? null)
  const [running, setRunning] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const current = KINDS.find((k) => k.id === kind)!

  async function run() {
    setError(null)
    setRunning(true)
    try {
      setLive(await runKind(kind, term))
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setRunning(false)
    }
  }

  return (
    <section>
      <PageHeader
        eyebrow="Research · Lexical instruments"
        title="Research Console"
        lede="Nine instruments over one typed engine. Hits keep the order the service returned — the client never re-sorts and never invents."
        actions={
          <span className="chip">
            {live === null ? 'idle' : `${live.length} hits`}
          </span>
        }
      />

      <div className="mt-8 grid gap-x-10 gap-y-8 lg:grid-cols-[13rem_minmax(0,1fr)]">
        <nav aria-label="Research instruments">
          <p className="kicker mb-2.5">Instruments</p>
          <div
            role="toolbar"
            aria-label="research instrument"
            className="grid grid-cols-2 gap-0.5 sm:grid-cols-3 lg:grid-cols-1"
          >
            {KINDS.map((k, i) => (
              <button
                key={k.id}
                type="button"
                aria-pressed={kind === k.id}
                title={k.hint}
                onClick={() => setKind(k.id)}
                className={`flex items-center gap-2.5 rounded-md px-2.5 py-1.5 text-left text-[0.8125rem] transition-colors ${
                  kind === k.id
                    ? 'bg-accent font-medium text-accent-foreground'
                    : 'text-muted-foreground hover:bg-muted hover:text-foreground'
                }`}
              >
                <span className="font-mono text-[0.6rem] tabular-nums opacity-50" aria-hidden="true">
                  {String(i + 1).padStart(2, '0')}
                </span>
                {k.label}
              </button>
            ))}
          </div>
          <p className="margin-note mt-4 hidden border-t pt-3 lg:block">
            Enter runs the instrument · findings keep service order.
          </p>
        </nav>

        <div className="min-w-0">
          <div className="flex flex-col gap-2 sm:flex-row">
            <input
              aria-label="research query"
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
              {running ? 'consulting…' : 'Run'}
            </button>
          </div>

          {error !== null && <ErrorNote>{error}</ErrorNote>}

          {live !== null && (
            <div className="mt-6">
              <div className="mb-2.5 flex items-baseline gap-3">
                <h2 className="kicker">Findings</h2>
                <span className="h-px flex-1 bg-border" aria-hidden="true" />
                <span className="font-mono text-[0.625rem] tabular-nums text-muted-foreground">
                  {String(live.length).padStart(2, '0')} · service order
                </span>
              </div>
              <ResearchResults query={term} hits={live} />
            </div>
          )}

          {live === null && (
            <div className="mt-6">
              <EmptyState title={current.label}>
                {current.hint}. Enter a term and run the instrument.
              </EmptyState>
            </div>
          )}
        </div>
      </div>
    </section>
  )
}
