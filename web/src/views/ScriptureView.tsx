// Scripture view: editions, search, parallel columns, research.
import { useState } from 'react'
import { Loader2 } from 'lucide-react'
import { PageHeader } from '../components/layout/page-header'
import { EmptyState, ErrorNote } from '../components/states'

export default function ScriptureView() {
  const [edition, setEdition] = useState('')
  const [query, setQuery] = useState('')
  const [result, setResult] = useState<string>('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  async function call(url: string, init?: RequestInit) {
    setError(null)
    setLoading(true)
    setResult('')
    try {
      const res = await fetch(url, init)
      const text = await res.text()
      setResult(`${res.status} ${res.statusText}\n${text}`)
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setLoading(false)
    }
  }

  return (
    <section>
      <PageHeader
        eyebrow="Scripture · Comparative"
        title="Scripture"
        lede="Editions, search, parallel columns, and research. Responses are served verbatim."
        actions={<span className="chip">read-only</span>}
      />

      <div className="mt-8 grid gap-x-10 gap-y-8 lg:grid-cols-[13rem_minmax(0,1fr)]">
        <aside aria-label="Scripture actions">
          <p className="kicker mb-2.5">Actions</p>
          <div className="space-y-2">
            <button type="button" className="btn w-full justify-center" onClick={() => void call('/api/v1/scripture/editions')} disabled={loading}>
              List editions
            </button>
            <button
              type="button"
              className="btn w-full justify-center"
              onClick={() => void call(`/api/v1/scripture/search?query=${encodeURIComponent(query)}&limit=20`)}
              disabled={loading || query === ''}
            >
              Search
            </button>
            <button
              type="button"
              className="btn w-full justify-center"
              onClick={() =>
                void call('/api/v1/scripture/research', {
                  method: 'POST',
                  headers: { 'content-type': 'application/json', accept: 'application/json' },
                  body: JSON.stringify({ query, domains: 'scripture' }),
                })
              }
              disabled={loading || query === ''}
            >
              Research
            </button>
          </div>
          <label className="mt-4 block">
            <span className="kicker mb-1.5 block">Query / edition</span>
            <input value={query} onChange={(e) => setQuery(e.target.value)} className="field h-8" placeholder="genesis" />
          </label>
          <label className="mt-3 block">
            <span className="kicker mb-1.5 block">Edition (for verse)</span>
            <input value={edition} onChange={(e) => setEdition(e.target.value)} className="field h-8" placeholder="kjv" />
          </label>
        </aside>

        <div className="min-w-0 space-y-4">
          {error !== null && <ErrorNote>{error}</ErrorNote>}
          {result === '' ? (
            <EmptyState title="No transmission">Choose an action to see the transcript.</EmptyState>
          ) : (
            <pre className="carbon max-h-[32rem] overflow-auto rounded-lg border px-4 py-3 font-mono text-[0.72rem] leading-relaxed">
              {result}
            </pre>
          )}
          {loading && (
            <p role="status" className="flex items-center gap-2 text-[0.8125rem] text-muted-foreground">
              <Loader2 className="size-3.5 animate-spin" aria-hidden="true" />
              consulting…
            </p>
          )}
        </div>
      </div>
    </section>
  )
}
