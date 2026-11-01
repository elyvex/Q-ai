// Tafsir view: fetch a tafsir work by slug, or search across works.
import { useState } from 'react'
import { Loader2 } from 'lucide-react'
import { PageHeader } from '../components/layout/page-header'
import { EmptyState, ErrorNote } from '../components/states'

export default function TafsirView() {
  const [work, setWork] = useState('')
  const [query, setQuery] = useState('')
  const [result, setResult] = useState<string>('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  async function fetchWork() {
    setError(null)
    setLoading(true)
    setResult('')
    try {
      const res = await fetch(`/api/v1/tafsir/${encodeURIComponent(work)}`)
      const text = await res.text()
      setResult(`${res.status} ${res.statusText}\n${text}`)
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setLoading(false)
    }
  }

  async function search() {
    setError(null)
    setLoading(true)
    setResult('')
    try {
      const res = await fetch('/api/v1/tafsir/search', {
        method: 'POST',
        headers: { 'content-type': 'application/json', accept: 'application/json' },
        body: JSON.stringify({ query, work_slug: work || null, limit: 20 }),
      })
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
        eyebrow="Tafsir · Works"
        title="Tafsir"
        lede="Fetch a tafsir work by slug, or search across works. Responses are served verbatim."
        actions={<span className="chip">read-only</span>}
      />

      <div className="mt-8 grid gap-x-10 gap-y-8 lg:grid-cols-[13rem_minmax(0,1fr)]">
        <aside aria-label="Tafsir parameters">
          <p className="kicker mb-2.5">Parameters</p>
          <div className="space-y-3">
            <label className="block">
              <span className="kicker mb-1.5 block">Work slug</span>
              <input value={work} onChange={(e) => setWork(e.target.value)} className="field h-8" placeholder="tafsir-al-tabari" />
            </label>
            <button type="button" className="btn btn-primary w-full justify-center" onClick={() => void fetchWork()} disabled={loading || work === ''}>
              {loading ? <Loader2 className="size-3.5 animate-spin" aria-hidden="true" /> : null}
              Fetch work
            </button>
          </div>
        </aside>

        <div className="min-w-0 space-y-4">
          <div className="rounded-lg border bg-card p-3">
            <p className="kicker mb-2">Search</p>
            <div className="flex gap-2">
              <input
                aria-label="tafsir search query"
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') void search()
                }}
                placeholder="search text"
                className="field h-8 flex-1"
              />
              <button type="button" className="btn btn-primary shrink-0" onClick={() => void search()} disabled={loading}>
                Search
              </button>
            </div>
          </div>

          {error !== null && <ErrorNote>{error}</ErrorNote>}

          {result === '' ? (
            <EmptyState title="No transmission">Fetch a work or run a search to see the transcript.</EmptyState>
          ) : (
            <pre className="carbon max-h-[32rem] overflow-auto rounded-lg border px-4 py-3 font-mono text-[0.72rem] leading-relaxed">
              {result}
            </pre>
          )}
        </div>
      </div>
    </section>
  )
}
