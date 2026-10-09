// Citation open view (D-17/ADR-0111): opens a deep link against
// `/api/v1/quran/citations/{id}` — the endpoint that re-verifies
// server-side and hard-fails on mismatch. Verification never runs
// client-side; `/api/v1/quran/resolve` only parses references (always 200)
// and must not be the verdict path.
import { useEffect, useState } from 'react'
import { fetchCitation } from '../api/client'
import CitationCopy from '../components/CitationCopy'

export default function CitationView({ citationId }: { citationId: string }) {
  const [state, setState] = useState<
    | { status: 'loading' }
    | { status: 'verified'; urn: string; deepLink: string }
    | { status: 'mismatch'; detail: string }
  >({ status: 'loading' })

  useEffect(() => {
    let live = true
    fetchCitation(citationId)
      .then((envelope) => {
        if (!live) return
        const data = envelope.data as {
          citation_id?: string
          urn?: string
          deep_link?: string
          verdict?: string | { Mismatch?: unknown }
        }
        setState({
          status: 'verified',
          urn: data.urn ?? '',
          deepLink: data.deep_link ?? '',
        })
      })
      .catch((err: unknown) => {
        if (!live) return
        // Hard failure from the server (typed error, never a 200 envelope).
        setState({ status: 'mismatch', detail: err instanceof Error ? err.message : String(err) })
      })
    return () => {
      live = false
    }
  }, [citationId])

  if (state.status === 'loading') return <p>verifying citation…</p>
  if (state.status === 'mismatch') {
    return (
      <p role="alert">
        citation mismatch — the stored quote does not match the canonical text: {state.detail}
      </p>
    )
  }
  return (
    <section>
      <p>verified at the exact source location</p>
      <CitationCopy urn={state.urn} deepLink={state.deepLink} />
    </section>
  )
}
