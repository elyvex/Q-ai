import { useState } from 'react'

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

  return (
    <div>
      <h2>API Explorer</h2>
      <p>Run any backend operation. Presets cover every domain.</p>
      <div>
        <label>
          preset{' '}
          <select value={preset} onChange={(e) => applyPreset(Number(e.target.value))}>
            {PRESETS.map((p, i) => (
              <option key={i} value={i}>
                {p.label}
              </option>
            ))}
          </select>
        </label>
      </div>
      <div>
        <label>
          method{' '}
          <select value={method} onChange={(e) => setMethod(e.target.value as 'GET' | 'POST')}>
            <option>GET</option>
            <option>POST</option>
          </select>
        </label>
      </div>
      <div>
        <label>
          path{' '}
          <input value={path} onChange={(e) => setPath(e.target.value)} style={{ width: '60%' }} />
        </label>
      </div>
      {method === 'POST' && (
        <div>
          <label>
            body (JSON){' '}
            <textarea value={body} onChange={(e) => setBody(e.target.value)} rows={6} cols={60} />
          </label>
        </div>
      )}
      <button onClick={run} disabled={loading}>
        {loading ? 'running…' : 'run'}
      </button>
      <pre style={{ whiteSpace: 'pre-wrap', marginTop: '1rem' }}>{result}</pre>
    </div>
  )
}
