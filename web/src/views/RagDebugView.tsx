// RAG debug view: one read-only path — the form, the typed client, the
// ToolResult shape, and the shared ledger — plus the route-decision block
// and the honest note about the mutating tools the browser cannot call.
// The screen observes retrieval; it never creates projects or imports
// documents.
import { useState } from 'react'
import { Loader2, Send } from 'lucide-react'
import { PageHeader } from '../components/layout/page-header'
import { EmptyState, ErrorNote, LoadingLine } from '../components/states'
import ToolLedger from '../components/ToolLedger'
import { ragDebug, ragQuery, type ApiRequestError } from '../api/client'
import type { ToolResultData } from '../lib/tool-result'

type RagMode = 'debug' | 'query'

const ROUTE_DECISION_FIELDS = [
  'router',
  'tools',
  'sources',
  'scores',
  'final_ranking',
  'candidates',
  'merged',
  'reranked',
  'excluded',
  'checksum',
] as const

/** Tools the HTTP route refuses with `QAI-RAG-0326` unless the operator opts in. */
const DENIED_RAG_TOOLS = [
  'rag.create_project',
  'rag.create',
  'rag.import',
  'rag.source_status',
  'rag.source_activate',
  'rag.source_deactivate',
  'rag.source_tombstone',
] as const

function failureOf(error: unknown): { code?: string; summary: string } {
  if (error instanceof Error) {
    const code = (error as Partial<ApiRequestError>).code
    return code === undefined ? { summary: error.message } : { code, summary: error.message }
  }
  return { summary: String(error) }
}

function routeDecisionOf(results: unknown): Record<string, unknown> | null {
  if (typeof results !== 'object' || results === null || Array.isArray(results)) return null
  const decision = (results as Record<string, unknown>).route_decision
  if (typeof decision !== 'object' || decision === null || Array.isArray(decision)) return null
  return decision as Record<string, unknown>
}

function decisionText(value: unknown): string | null {
  if (value === undefined || value === null) return null
  if (Array.isArray(value)) return value.map((entry) => decisionText(entry) ?? '').join(', ')
  if (typeof value === 'string') return value
  if (typeof value === 'number') return String(value)
  return JSON.stringify(value)
}

export default function RagDebugView() {
  const [mode, setMode] = useState<RagMode>('debug')
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
      const envelope = mode === 'debug' ? await ragDebug(project, query) : await ragQuery(project, query)
      setResult(envelope.data)
    } catch (error: unknown) {
      setFailure(failureOf(error))
    } finally {
      setLoading(false)
    }
  }

  const routeDecision = result !== null ? routeDecisionOf(result.results) : null

  return (
    <section>
      <PageHeader
        eyebrow="Instruments · Observation"
        title="RAG Debug"
        lede="This screen observes retrieval over one project: it runs a read-only retrieval query and prints the tool result verbatim. It creates no projects and imports no documents."
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
              <p className="kicker mb-2">
                debug returns the same payload as query in this build
              </p>
              <div className="flex gap-0.5" role="group" aria-label="retrieval mode">
                {(['debug', 'query'] as const).map((candidate) => (
                  <button
                    key={candidate}
                    type="button"
                    className="seg px-3 py-1 text-[0.75rem]"
                    aria-pressed={mode === candidate}
                    onClick={() => setMode(candidate)}
                  >
                    {candidate}
                  </button>
                ))}
              </div>
            </div>
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
              Run {mode} query
            </button>
          </form>

          {loading && <LoadingLine label={`running rag.${mode}…`} />}

          {failure !== null && (
            <ErrorNote>
              {failure.code !== undefined && (
                <span className="font-mono text-[0.75rem]">{failure.code} — </span>
              )}
              {failure.summary}
            </ErrorNote>
          )}

          {result !== null ? (
            <div className="space-y-6 border-t pt-4">
              <ToolLedger result={result} />

              {routeDecision !== null && (
                <div>
                  <p className="kicker mb-2">Route decision</p>
                  <dl className="divide-y border-t">
                    {ROUTE_DECISION_FIELDS.map((field) => {
                      const text = decisionText(routeDecision[field])
                      return (
                        <div key={field} className="flex flex-wrap gap-x-3 py-1.5">
                          <dt className="w-28 shrink-0 font-mono text-[0.65rem] text-muted-foreground">
                            {field}
                          </dt>
                          {text !== null ? (
                            <dd className="min-w-0 flex-1 font-mono text-[0.7rem] break-words">
                              {text === '' ? '—' : text}
                            </dd>
                          ) : (
                            <dd className="min-w-0 flex-1 text-[0.7rem] text-muted-foreground">
                              absent
                            </dd>
                          )}
                        </div>
                      )
                    })}
                  </dl>
                </div>
              )}

            </div>
          ) : (
            !loading &&
            failure === null && (
              <EmptyState title="no retrieval query has run">
                Enter a project and a query, then run. The tool result prints here verbatim.
              </EmptyState>
            )
          )}

          <div>
            <p className="kicker mb-2">Unavailable from the browser</p>
            <ul className="divide-y border-t">
              {DENIED_RAG_TOOLS.map((name) => (
                <li
                  key={name}
                  aria-disabled="true"
                  className="flex flex-wrap items-baseline gap-x-3 py-1.5 opacity-70"
                >
                  <span className="font-mono text-[0.7rem]">{name}</span>
                  <span className="font-mono text-[0.65rem] text-muted-foreground">
                    QAI-RAG-0326
                  </span>
                  <span className="w-full text-[0.75rem] text-muted-foreground">
                    This tool writes durable state, so the browser cannot call it; the trusted
                    surface for it is the local CLI (`qai rag …`).
                  </span>
                </li>
              ))}
            </ul>
          </div>
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
