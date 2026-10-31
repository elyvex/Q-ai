// Graph view: neighbors/paths from the frozen payload with verbatim
// truncation (ADR-0217). A truncated payload is never rendered as absence.
import { useState } from 'react'
import { Loader2, Network } from 'lucide-react'
import { PageHeader } from '../components/layout/page-header'
import { EmptyState, ErrorNote } from '../components/states'
import { graphNeighbors, graphPath } from '../api/client'

export default function GraphView({
  lines,
  truncated,
  incompleteReason,
}: {
  lines: string[]
  truncated: boolean
  incompleteReason?: string
}) {
  const [node, setNode] = useState('ayah:1:1')
  const [from, setFrom] = useState('ayah:1:1')
  const [to, setTo] = useState('ayah:1:2')
  const [live, setLive] = useState<string[] | null>(null)
  const [running, setRunning] = useState(false)
  const [error, setError] = useState<string | null>(null)

  async function openNeighbors() {
    setError(null)
    setRunning(true)
    try {
      const envelope = await graphNeighbors(node)
      const data = envelope.data as {
        results?: { edges?: Array<{ src?: string; edge?: string; dst?: string }> }
      }
      setLive(
        (data.results?.edges ?? []).map(
          (e) => `${e.src ?? '?'} -${e.edge ?? '?'}-> ${e.dst ?? '?'}`,
        ),
      )
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setRunning(false)
    }
  }

  async function openPath() {
    setError(null)
    setRunning(true)
    try {
      const envelope = await graphPath(from, to)
      const data = envelope.data as {
        results?: { edges?: Array<{ src?: string; edge?: string; dst?: string }> }
      }
      setLive(
        (data.results?.edges ?? []).map(
          (e) => `${e.src ?? '?'} -${e.edge ?? '?'}-> ${e.dst ?? '?'}`,
        ),
      )
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setRunning(false)
    }
  }

  const shown = live ?? lines

  return (
    <section>
      <PageHeader
        eyebrow="Research · Graph instruments"
        title="Knowledge Graph"
        lede="The frozen payload, verbatim. Edges keep engine order; truncation is announced, never disguised as absence (ADR-0217)."
        actions={<span className="chip">{shown.length} edges</span>}
      />

      <div className="mt-8 grid gap-x-10 gap-y-8 lg:grid-cols-[minmax(0,1fr)_16rem]">
        <div className="min-w-0">
          <div className="rounded-lg border bg-card">
            <div role="toolbar" aria-label="graph navigation" className="divide-y">
              <div className="flex flex-col gap-2 p-3 sm:flex-row sm:items-center">
                <span className="w-16 shrink-0 font-mono text-[0.6rem] uppercase tracking-[0.14em] text-muted-foreground">
                  Neighbors
                </span>
                <input
                  aria-label="node"
                  value={node}
                  onChange={(e) => setNode(e.target.value)}
                  className="field h-8 flex-1 font-mono text-xs"
                />
                <button
                  type="button"
                  className="btn justify-center"
                  onClick={() => void openNeighbors()}
                  disabled={running}
                >
                  {running ? (
                    <Loader2 className="size-3 animate-spin" aria-hidden="true" />
                  ) : (
                    <Network className="size-3.5" aria-hidden="true" />
                  )}
                  Open neighbors
                </button>
              </div>
              <div className="flex flex-col gap-2 p-3 sm:flex-row sm:items-center">
                <span className="w-16 shrink-0 font-mono text-[0.6rem] uppercase tracking-[0.14em] text-muted-foreground">
                  Path
                </span>
                <input
                  aria-label="path from"
                  value={from}
                  onChange={(e) => setFrom(e.target.value)}
                  className="field h-8 flex-1 font-mono text-xs"
                />
                <span className="hidden font-mono text-[0.7rem] text-muted-foreground sm:block" aria-hidden="true">
                  →
                </span>
                <input
                  aria-label="path to"
                  value={to}
                  onChange={(e) => setTo(e.target.value)}
                  className="field h-8 flex-1 font-mono text-xs"
                />
                <button
                  type="button"
                  className="btn btn-primary justify-center"
                  onClick={() => void openPath()}
                  disabled={running}
                >
                  Trace path
                </button>
              </div>
            </div>
          </div>

          {error !== null && <ErrorNote>{error}</ErrorNote>}

          {truncated && (
            <p className="graph-truncated mt-4 flex items-center gap-2 rounded-md border border-annot/40 bg-annot-soft px-3.5 py-2 font-mono text-[0.75rem] text-annot">
              truncated: {incompleteReason ?? 'reason withheld'}
            </p>
          )}

          <div className="mt-5">
            {shown.length === 0 ? (
              <EmptyState icon={Network} title="No graph lines">
                no graph lines selected — open neighbors or trace a path above
              </EmptyState>
            ) : (
              <ul className="graph-lines overflow-hidden rounded-lg border bg-card">
                {shown.map((line, i) => (
                  <li
                    key={line + ':' + i}
                    className="graph-line flex items-center gap-3 border-b px-4 py-2.5 font-mono text-xs last:border-b-0 hover:bg-muted/40"
                  >
                    <span className="w-6 shrink-0 text-right text-[0.6rem] tabular-nums text-muted-foreground/50">
                      {String(i + 1).padStart(2, '0')}
                    </span>
                    <span className="size-1 shrink-0 rounded-full bg-canon" aria-hidden="true" />
                    <span className="break-all text-foreground/85">{line}</span>
                  </li>
                ))}
              </ul>
            )}
          </div>
        </div>

        <aside className="lg:sticky lg:top-20 lg:h-fit lg:border-l lg:pl-6">
          <p className="kicker mb-2.5">Reading the ledger</p>
          <ul className="margin-note space-y-2.5 border-t pt-4">
            <li>Edges are printed exactly as the engine emitted them — no client-side sort.</li>
            <li>A truncated payload is labelled above the list; it is never shown as “no edges”.</li>
            <li>
              Node references follow the canonical form, e.g.{' '}
              <code className="font-mono text-[0.7rem] text-foreground">ayah:2:255</code>.
            </li>
          </ul>
        </aside>
      </div>
    </section>
  )
}
