// API Explorer: run any backend operation through the loopback API.
// Presets cover every domain; requests go out verbatim, status + body
// come back verbatim. Nothing is faked — failures print exactly as
// received.
import { useState } from 'react'
import { Loader2, Send } from 'lucide-react'
import { PageHeader } from '../components/layout/page-header'

const PRESETS: { label: string; method: 'GET' | 'POST'; path: string; body: string }[] = [
  { label: 'Quran ayahs 2:1', method: 'GET', path: '/api/v1/quran/ayahs/2:1', body: '' },
  { label: 'Search normalized', method: 'POST', path: '/api/v1/quran/search/normalized', body: '{"text":"الله"}' },
  { label: 'Graph neighbors', method: 'POST', path: '/api/v1/quran/graph/neighbors', body: '{"node":"quran:2:1"}' },
  { label: 'Scripture editions', method: 'GET', path: '/api/v1/scripture/editions', body: '' },
  { label: 'Scripture search', method: 'POST', path: '/api/v1/scripture/search', body: '{"query":"genesis","limit":5}' },
  { label: 'Cross-domain research', method: 'POST', path: '/api/v1/cross-domain/research', body: '{"query":"mercy","domains":"quran,scripture"}' },
  { label: 'RAG debug', method: 'GET', path: '/api/v1/rag/debug', body: '' },
]

export default function ExplorerView() {
  const [preset, setPreset] = useState(0)
  const [method, setMethod] = useState<'GET' | 'POST'>(PRESETS[0].method)
  const [path, setPath] = useState(PRESETS[0].path)
  const [body, setBody] = useState(PRESETS[0].body)
  const [result, setResult] = useState<string>('')
  const [loading, setLoading] = useState(false)

  function applyPreset(i: number) {
    setPreset(i)
    setMethod(PRESETS[i].method)
    setPath(PRESETS[i].path)
    setBody(PRESETS[i].body)
  }

  async function run() {
    setLoading(true)
    setResult('')
    try {
      const res = await fetch(path, {
        method,
        headers: { accept: 'application/json', 'content-type': 'application/json' },
        body: method === 'POST' ? body : undefined,
      })
      const text = await res.text()
      setResult(`${res.status} ${res.statusText}\n${text}`)
    } catch (e) {
      setResult(e instanceof Error ? e.message : String(e))
    } finally {
      setLoading(false)
    }
  }

  const statusLine = result.split('\n', 1)[0]
  const ok = statusLine.startsWith('2')

  return (
    <section>
      <PageHeader
        eyebrow="Instruments · Loopback console"
        title="API Explorer"
        lede="Dispatch any request against the local engine and read the raw transcript. Requests leave verbatim; responses return verbatim."
        actions={<span className="chip">loopback</span>}
      />

      <div className="mt-8 grid gap-x-10 gap-y-6 lg:grid-cols-[14rem_minmax(0,1fr)]">
        <aside aria-label="request presets">
          <p className="kicker mb-2.5">Presets</p>
          <div className="space-y-px">
            {PRESETS.map((p, i) => (
              <button
                key={i}
                type="button"
                aria-pressed={preset === i}
                onClick={() => applyPreset(i)}
                className={`w-full rounded-md px-2.5 py-2 text-left transition-colors ${
                  preset === i
                    ? 'bg-accent text-accent-foreground'
                    : 'text-muted-foreground hover:bg-muted hover:text-foreground'
                }`}
              >
                <span className="flex items-baseline gap-2 text-[0.8125rem] font-medium">
                  <span className="font-mono text-[0.6rem] opacity-50" aria-hidden="true">
                    {String(i + 1).padStart(2, '0')}
                  </span>
                  {p.label}
                </span>
                <span className="mt-0.5 block truncate pl-5 font-mono text-[0.625rem] opacity-60">
                  {p.method} {p.path}
                </span>
              </button>
            ))}
          </div>
        </aside>

        <div className="min-w-0 space-y-4">
          <div className="rounded-lg border bg-card">
            <div
              role="toolbar"
              aria-label="request method and path"
              className="flex flex-col gap-2 border-b p-3 sm:flex-row sm:items-center"
            >
              <div
                className="flex shrink-0 overflow-hidden rounded-md border"
                role="radiogroup"
                aria-label="method"
              >
                {(['GET', 'POST'] as const).map((m) => (
                  <button
                    key={m}
                    type="button"
                    role="radio"
                    aria-checked={method === m}
                    onClick={() => setMethod(m)}
                    className={`px-3 py-1.5 font-mono text-[0.7rem] font-medium transition-colors ${
                      method === m
                        ? 'bg-primary text-primary-foreground'
                        : 'bg-card text-muted-foreground hover:bg-muted hover:text-foreground'
                    }`}
                  >
                    {m}
                  </button>
                ))}
              </div>
              <input
                aria-label="request path"
                value={path}
                onChange={(e) => setPath(e.target.value)}
                className="field h-8 flex-1 font-mono text-xs"
              />
              <button
                type="button"
                className="btn btn-primary shrink-0 justify-center px-4"
                onClick={() => void run()}
                disabled={loading}
              >
                {loading ? (
                  <Loader2 className="size-3.5 animate-spin" aria-hidden="true" />
                ) : (
                  <Send className="size-3.5" aria-hidden="true" />
                )}
                Dispatch request
              </button>
            </div>

            {method === 'POST' && (
              <label className="block p-3">
                <span className="kicker mb-2 block">Request body · JSON</span>
                <textarea
                  aria-label="request body"
                  value={body}
                  onChange={(e) => setBody(e.target.value)}
                  rows={5}
                  spellCheck={false}
                  className="field resize-y font-mono text-xs"
                />
              </label>
            )}
          </div>

          <div className="overflow-hidden rounded-lg border">
            <div className="flex items-center justify-between gap-3 border-b bg-muted/50 px-3.5 py-2">
              <span className="kicker">Transcript</span>
              <span
                className={`font-mono text-[0.65rem] ${
                  result === '' ? 'text-muted-foreground' : ok ? 'text-canon' : 'text-destructive'
                }`}
              >
                {result === '' ? 'no transmission' : statusLine.slice(0, 24)}
              </span>
            </div>
            <pre
              className="explorer-out carbon max-h-[26rem] overflow-auto px-4 py-3 font-mono text-[0.72rem] leading-relaxed"
              aria-live="polite"
            >
              {result !== '' ? result : 'awaiting dispatch'}
            </pre>
          </div>
        </div>
      </div>
    </section>
  )
}
