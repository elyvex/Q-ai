// The single renderer for any ToolResultData: header identity, warnings,
// canonical references, a ruled row table, and the reproducibility footer.
// Values print as received — numbers are never reformatted, an absent
// field never prints as zero — and every cell renders as React text,
// never as markup.
import { EmptyState } from './states'
import { checksumLabel, deepLinkFor, rowsOf, type ToolResultData } from '../lib/tool-result'

function cellText(value: unknown): string {
  if (value === undefined) return ''
  if (typeof value === 'string') return value
  if (typeof value === 'object' && value !== null) return JSON.stringify(value)
  return String(value)
}

export default function ToolLedger({ result }: { result: ToolResultData }) {
  const rows = rowsOf(result.results)
  const columns = rows.length > 0 ? Object.keys(rows[0]) : []

  return (
    <div>
      <p className="flex flex-wrap items-baseline gap-x-3 gap-y-1 font-mono text-[0.7rem]">
        <span className="font-semibold text-foreground">{result.tool_name}</span>
        <span className="text-muted-foreground">{result.tool_version}</span>
        <span className="text-muted-foreground">{result.execution_time_ms} ms</span>
        <span className="text-muted-foreground">
          research_checksum (server) {checksumLabel(result.research_checksum)}
        </span>
      </p>

      {result.warnings.length > 0 && (
        <ul className="mt-3 space-y-1 border-t pt-3">
          {result.warnings.map((warning, i) => (
            <li key={i} className="text-[0.8125rem] text-muted-foreground">
              {warning}
            </li>
          ))}
        </ul>
      )}

      {result.canonical_references.length > 0 && (
        <ul className="mt-3 space-y-1 border-t pt-3">
          {result.canonical_references.map((reference) => {
            const href = deepLinkFor(reference)
            return (
              <li key={reference} className="font-mono text-[0.72rem]">
                {href !== null ? (
                  <a href={href} className="text-primary underline-offset-4 hover:underline">
                    {reference}
                  </a>
                ) : (
                  <span className="text-foreground">{reference}</span>
                )}
              </li>
            )
          })}
        </ul>
      )}

      {rows.length > 0 ? (
        <div className="mt-3 overflow-x-auto border-t">
          <table className="w-full border-collapse text-left">
            <thead>
              <tr className="border-b">
                {columns.map((column) => (
                  <th
                    key={column}
                    scope="col"
                    className="px-3 py-2 font-mono text-[0.625rem] font-medium uppercase tracking-[0.14em] text-muted-foreground"
                  >
                    {column}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody className="divide-y">
              {rows.map((row, i) => (
                <tr key={i}>
                  {columns.map((column) => (
                    <td key={column} className="px-3 py-2 align-top text-[0.8125rem]">
                      {cellText(row[column])}
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : result.results !== null && result.results !== undefined ? (
        <pre className="carbon mt-3 overflow-x-auto border-t px-0 py-3 font-mono text-[0.72rem] leading-relaxed">
          {JSON.stringify(result.results, null, 2)}
        </pre>
      ) : (
        <div className="mt-3 border-t pt-3">
          <EmptyState title="the tool returned no rows">
            The tool ran and reported no rows. Nothing was invented to fill the table.
          </EmptyState>
        </div>
      )}

      <dl className="mt-3 border-t pt-3">
        <div className="flex flex-wrap gap-x-2 py-1 text-[0.75rem]">
          <dt className="kicker">tool plan</dt>
          <dd className="font-mono text-[0.7rem] text-muted-foreground">
            {result.reproducibility.tool_plan.join(', ') || '—'}
          </dd>
        </div>
        {Object.entries(result.reproducibility.edition_versions).map(([slug, version]) => (
          <div key={slug} className="flex flex-wrap gap-x-2 py-1 text-[0.75rem]">
            <dt className="kicker">edition</dt>
            <dd className="font-mono text-[0.7rem] text-muted-foreground">
              {slug}@{version}
            </dd>
          </div>
        ))}
        <div className="flex flex-wrap gap-x-2 py-1 text-[0.75rem]">
          <dt className="kicker">query hash</dt>
          <dd className="font-mono text-[0.7rem] text-muted-foreground">
            {checksumLabel(result.reproducibility.query_hash)}
          </dd>
        </div>
      </dl>
    </div>
  )
}
