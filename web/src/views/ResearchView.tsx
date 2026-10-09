// Research view: search + root/lemma/family/frequency/distribution/
// co-occurrence over the typed client. Hits render in service-returned
// order — the client never re-sorts. Zero results render an explicit empty
// state, never a blank panel. Dataset text renders as text.
import { useState } from 'react'
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

const KINDS: ResearchKind[] = [
  'search',
  'root',
  'lemma',
  'family',
  'root-frequency',
  'lemma-frequency',
  'frequency',
  'distribution',
  'cooccurrence',
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
      <p className="research-empty">
        no results for “{query}” — try a different spelling or normalization
      </p>
    )
  }
  return (
    <ul>
      {hits.map((hit) => (
        <li key={hit.reference} className="research-hit">
          <span>{hit.reference}</span> — <span>{hit.snippet}</span>
        </li>
      ))}
    </ul>
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
  const [live, setLive] = useState<ResearchHit[] | null>(hits ?? null)
  const [error, setError] = useState<string | null>(null)

  async function run() {
    setError(null)
    try {
      setLive(await runKind(kind, query))
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err))
    }
  }

  return (
    <section>
      <div role="toolbar" aria-label="research kind">
        {KINDS.map((k) => (
          <button key={k} type="button" aria-pressed={kind === k} onClick={() => setKind(k)}>
            {k}
          </button>
        ))}
        <button type="button" onClick={() => void run()}>
          run
        </button>
      </div>
      {error !== null && <p role="alert">{error}</p>}
      <ResearchResults query={query} hits={live ?? []} />
    </section>
  )
}
