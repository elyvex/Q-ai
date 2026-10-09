// Graph view: neighbors/paths from the frozen payload with verbatim
// truncation (ADR-0217). A truncated payload is never rendered as absence.
import { useState } from 'react'
import { graphNeighbors } from '../api/client'

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
  const [live, setLive] = useState<string[] | null>(null)
  const [error, setError] = useState<string | null>(null)

  async function openNeighbors() {
    setError(null)
    try {
      const envelope = await graphNeighbors(node)
      const data = envelope.data as { results?: { edges?: Array<{ src?: string; edge?: string; dst?: string }> } }
      setLive(
        (data.results?.edges ?? []).map((e) => `${e.src ?? '?'} -${e.edge ?? '?'}-> ${e.dst ?? '?'}`),
      )
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err))
    }
  }

  const shown = live ?? lines
  return (
    <section>
      <div role="toolbar" aria-label="graph navigation">
        <input aria-label="node" value={node} onChange={(e) => setNode(e.target.value)} />
        <button type="button" onClick={() => void openNeighbors()}>
          neighbors
        </button>
      </div>
      {error !== null && <p role="alert">{error}</p>}
      <ul>
        {shown.map((line) => (
          <li key={line} className="graph-line">
            {line}
          </li>
        ))}
      </ul>
      {truncated && (
        <p className="graph-truncated">
          truncated: {incompleteReason ?? 'reason withheld'}
        </p>
      )}
    </section>
  )
}
