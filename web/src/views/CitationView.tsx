// Citation open view (D-17/ADR-0111): opens a deep link against
// `/api/v1/quran/citations/{id}` — the endpoint that re-verifies
// server-side and hard-fails on mismatch. Verification never runs
// client-side; `/api/v1/quran/resolve` only parses references (always 200)
// and must not be the verdict path.
import { useEffect, useState } from 'react'
import { ShieldCheck, XCircle } from 'lucide-react'
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

  if (state.status === 'loading')
    return (
      <div className="mt-6 flex items-center gap-2 rounded-md border px-3.5 py-2.5" role="status">
        <span className="prodot animate-pulse bg-primary" aria-hidden="true" />
        <p className="text-[0.8125rem] text-muted-foreground">
          verifying citation — against the server, never locally…
        </p>
      </div>
    )

  if (state.status === 'mismatch') {
    return (
      <div
        role="alert"
        className="mt-6 flex items-start gap-3 rounded-lg border border-destructive/35 bg-destructive/[0.06] px-4 py-3.5"
      >
        <XCircle className="mt-0.5 size-4 shrink-0 text-destructive" aria-hidden="true" />
        <div>
          <p className="text-[0.8125rem] font-medium text-destructive">citation mismatch</p>
          <p className="mt-0.5 text-[0.8125rem] text-destructive/85">
            the stored quote does not match the canonical text — {state.detail}
          </p>
        </div>
      </div>
    )
  }

  return (
    <aside className="mt-6 overflow-hidden rounded-lg border">
      <div className="flex items-center gap-2 border-b px-4 py-2.5">
        <ShieldCheck className="size-3.5 text-canon" aria-hidden="true" />
        <p className="font-mono text-[0.625rem] uppercase tracking-[0.14em] text-canon">
          verified at the exact source location
        </p>
      </div>
      <div className="bg-card px-4 py-3">
        <CitationCopy urn={state.urn} deepLink={state.deepLink} />
      </div>
    </aside>
  )
}
