// Client-side routes. `/read/{slug}@{version}/{surah}:{ayah}` parses to a
// reference string fetched through the typed client; unknown paths fall
// back to the reading view (the server fallback serves index.html).
import { useEffect, useState } from 'react'
import ReadingView from './views/ReadingView'
import { fetchAyahs, type AyahView } from './api/client'

export function parseReadPath(path: string): string | null {
  const match = /^\/read\/([^/]+)\/(\d+:\d+)$/.exec(path)
  if (!match) return null
  void match[1]
  return match[2]
}

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

  if (reference === null) {
    return <p>reading view — open a /read/… deep link</p>
  }
  if (error !== null) return <p role="alert">{error}</p>
  if (ayahs === null) return <p>loading…</p>
  return <ReadingView ayahs={ayahs} onResolve={() => {}} />
}
