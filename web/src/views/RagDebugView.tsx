// RAG debug view: one read-only path — the form, the typed client, the
// ToolResult shape, and the shared ledger. The screen observes retrieval;
// it never creates projects or imports documents.
import { useState } from 'react'
import { Loader2, Send } from 'lucide-react'
import { PageHeader } from '../components/layout/page-header'
import { EmptyState, ErrorNote, LoadingLine } from '../components/states'
import ToolLedger from '../components/ToolLedger'
import { ragDebug, type ApiRequestError } from '../api/client'
import type { ToolResultData } from '../lib/tool-result'

function failureOf(error: unknown): { code?: string; summary: string } {
  if (error instanceof Error) {
    const code = (error as Partial<ApiRequestError>).code
    return code === undefined ? { summary: error.message } : { code, summary: error.message }
  }
  return { summary: String(error) }
}

export default function RagDebugView() {
  const [project, setProject] = useState('default')
  const [query, setQuery] = useState('')
  const [loading, setLoading] = useState(false)
  const [result, setResult] = useState<ToolResultData | null>(null)
  const [failure, setFailure] = useState<{ code?: string; summary: string } | null>(null)

  async function run() {
    setLoading(true)
    setResult(null)
    setFailure(null)
    try {
      const envelope = await ragDebug(project, query)
      setResult(envelope.data)
    } catch (error: unknown) {
      setFailure(failureOf(error))
    } finally {
      setLoading(false)
    }
  }

  return (
    <section>
      <PageHeader
        eyebrow="Instruments · Observation"
        title="RAG Debug"
        lede="This screen observes retrieval over one project: it runs a read-only debug query and prints the tool result verbatim. It creates no projects and imports no documents."
        actions={<span className="chip">read-only</span>}
      />

      <div className="mt-8 grid gap-x-10 gap-y-8 lg:grid-cols-[minmax(0,1fr)_16rem]">
        <div className="min-w-0 space-y-4">
          <form
            onSubmit={(event) => {
              event.preventDefault()
              void run()
            }}
            className="space-y-3"
          >
            <div>
              <label htmlFor="rag-project" className="kicker mb-2 block">
                Project
              </label>
              <input
                id="rag-project"
                value={project}
                onChange={(event) => setProject(event.target.value)}
                spellCheck={false}
                className="field font-mono text-xs"
              />
            </div>
            <div>
              <label htmlFor="rag-query" className="kicker mb-2 block">
                Query
              </label>
              <input
                id="rag-query"
                value={query}
                onChange={(event) => setQuery(event.target.value)}
                spellCheck={false}
                placeholder="mercy"
                className="field text-sm"
              />
            </div>
            <button type="submit" className="btn btn-primary" disabled={loading}>
              {loading ? (
                <Loader2 className="size-3.5 animate-spin" aria-hidden="true" />
              ) : (
                <Send className="size-3.5" aria-hidden="true" />
              )}
              Run debug query
            </button>
          </form>

          {loading && <LoadingLine label="running rag.debug…" />}

          {failure !== null && (
            <ErrorNote>
              {failure.code !== undefined && (
                <span className="font-mono text-[0.75rem]">{failure.code} — </span>
              )}
              {failure.summary}
            </ErrorNote>
          )}

          {result !== null ? (
            <div className="border-t pt-4">
              <ToolLedger result={result} />
            </div>
          ) : (
            !loading &&
            failure === null && (
              <EmptyState title="no debug query has run">
                Enter a project and a query, then run. The tool result prints here verbatim.
              </EmptyState>
            )
          )}
        </div>

        <aside className="lg:sticky lg:top-20 lg:h-fit">
          <p className="kicker mb-2.5">Reading the ledger</p>
          <ul className="margin-note space-y-2.5 border-t pt-4">
            <li>The header names the tool, its version, and the server checksum.</li>
            <li>Rows print as the server sent them — never reformatted.</li>
            <li>The footer lists the tool plan and the edition versions read.</li>
          </ul>
        </aside>
      </div>
    </section>
  )
}
