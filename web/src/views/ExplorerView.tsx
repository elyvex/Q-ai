// API Explorer: run any backend operation through the loopback API.
// Presets cover every domain; requests go out verbatim, status + body
// come back verbatim. Nothing is faked — failures print exactly as
// received.
import { useState } from 'react'
import { Loader2, PlugZap, Send } from 'lucide-react'

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
  const bodyLine = result.length > 0 && result.includes('\n') ? result.slice(result.indexOf('\n') + 1) : ''

  return (
    <section>
      <div className="mb-6 flex flex-wrap items-end justify-between gap-x-6 gap-y-4 border-b pb-6">
        <div>
          <p className="kicker mb-2">Instruments · Loopback console</p>
          <h1 className="text-2xl font-semibold tracking-tight sm:text-3xl">API Explorer</h1>
          <p className="mt-1 max-w-xl text-[0.8125rem] leading-relaxed text-muted-foreground">
            A quill for the wire: dispatch any request against the local engine and read the raw
            transcript. Requests leave verbatim; transcripts return verbatim.
          </p>
        </div>
        <PlugZap className="size-5 text-gold/60" aria-hidden="true" />
      </div>

      <div className="grid gap-4 lg:grid-cols-[minmax(0,17rem)_minmax(0,1fr)]">
        {/* presets ledger */}
        <aside className="plate overflow-hidden p-1.5" aria-label="request presets">
          <p className="kicker px-2.5 py-2.5 !justify-start">Bound forms</p>
          <div className="space-y-0.5">
            {PRESETS.map((p, i) => (
              <button
                key={i}
                type="button"
                aria-pressed={preset === i}
                onClick={() => applyPreset(i)}
                className={`w-full rounded-md px-2.5 py-2 text-left transition-colors ${
                  preset === i
                    ? 'bg-secondary text-foreground shadow-[inset_2px_0_0_0_var(--gold)]'
                    : 'text-muted-foreground hover:bg-accent hover:text-foreground'
                }`}
              >
                <span className="block text-xs font-medium">{p.label}</span>
                <span className="mt-0.5 block truncate font-mono text-[0.65rem] opacity-60">
                  {p.method} {p.path}
                </span>
              </button>
            ))}
          </div>
        </aside>

        {/* request bench */}
        <div className="min-w-0 space-y-3">
          <div className="plate p-3 sm:p-4">
            <div role="toolbar" aria-label="request method and path" className="flex flex-col gap-2 sm:flex-row sm:items-center">
              <div className="flex shrink-0 overflow-hidden rounded-md border border-input" role="radiogroup" aria-label="method">
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
                        : 'bg-card text-muted-foreground hover:bg-accent hover:text-foreground'
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
                className="min-w-0 flex-1 rounded-md border border-input bg-background px-3 py-1.5 font-mono text-xs outline-none focus:border-ring/60 focus:ring-2 focus:ring-ring/25"
              />
              <button
                type="button"
                className="ibtn ibtn-primary shrink-0 justify-center px-4"
                onClick={() => void run()}
                disabled={loading}
                aria-live="polite"
              >
                {loading ? <Loader2 className="size-3.5 animate-spin" /> : <Send className="size-3.5" />}
                Dispatch
              </button>
            </div>

            {method === 'POST' && (
              <label className="mt-3 block">
                <span className="kicker mb-1.5 !justify-start">Request body · JSON</span>
                <textarea
                  aria-label="request body"
                  value={body}
                  onChange={(e) => setBody(e.target.value)}
                  rows={6}
                  spellCheck={false}
                  className="w-full resize-y rounded-md border border-input bg-background px-3 py-2.5 font-mono text-xs leading-relaxed outline-none focus:border-ring/60 focus:ring-2 focus:ring-ring/25"
                />
              </label>
            )}
          </div>

          {/* transcript */}
          <div className="rounded-xl border overflow-hidden">
            <div className="flex items-center justify-between border-b bg-muted/40 px-4 py-2">
              <span className="kicker !justify-start">Transcript</span>
              <span className={`font-mono text-[0.68rem] ${statusLine.startsWith('2') ? 'text-canon' : 'text-destructive'}`}>
                {result === '' ? '— no transmission yet —' : statusLine.slice(0, 9) + '…'}
              </span>
            </div>
            <pre className="explorer-out max-h-[26rem] overflow-auto px-4 py-3 font-mono text-[0.72rem] leading-relaxed text-foreground/85" aria-live="polite">
              {result === '' && bodyLine === '' ? '⏳ awaiting dispatch' : (result !== '' ? result : '')}
            </pre>
          </div>
        </div>
      </div>
    </section>
  )
}
