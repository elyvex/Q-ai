// Research Tools: the whole callable tool surface as one catalogue. A
// tool is picked from the left column, its typed form renders from the
// catalogue `fields` (so no out-of-catalogue parameter can be entered),
// and a surface control shows the same parameter object as the browser
// run (`ui`), the equivalent CLI command (`cli`), or the equivalent HTTP
// request (`http`). The `cli` and `http` panels derive from the params
// object kept beside the last successful ledger — never recomputed from
// the live form — so the checksum beside them is the checksum of the
// params shown. Tools the route refuses render as disabled rows with
// their policy code; nothing on this page can issue one of those calls.
import { useState } from 'react'
import { Copy, Loader2, Send } from 'lucide-react'
import { PageHeader } from '../components/layout/page-header'
import { EmptyState, ErrorNote, LoadingLine } from '../components/states'
import ToolLedger from '../components/ToolLedger'
import { runTool, type ApiRequestError } from '../api/client'
import type { ToolResultData } from '../lib/tool-result'
import {
  DENIED,
  DOMAIN_ORDER,
  TOOLS,
  ToolParamError,
  cliCommand,
  httpRequest,
  paramsJson,
  type ToolField,
  type ToolSpec,
} from '../lib/tools'

type Surface = 'ui' | 'cli' | 'http'

interface Failure {
  code?: string
  summary: string
  remedy?: string
}

interface SentRun {
  name: string
  params: Record<string, unknown>
}

function toolOf(name: string): ToolSpec {
  const found = TOOLS.find((tool) => tool.name === name)
  if (found !== undefined) return found
  const first = TOOLS[0]
  if (first === undefined) throw new Error('tool catalogue is empty')
  return first
}

function failureOf(error: unknown): Failure {
  if (error instanceof Error) {
    const code = (error as Partial<ApiRequestError>).code
    const remedy = (error as Partial<Failure>).remedy
    const failure: Failure = { summary: error.message }
    if (code !== undefined) failure.code = code
    if (typeof remedy === 'string' && remedy.length > 0) failure.remedy = remedy
    return failure
  }
  return { summary: String(error) }
}

/** A live JSON.parse check so a malformed `json` field blocks `Run` with its message. */
function jsonErrorFor(field: ToolField, raw: string): string | null {
  if (field.kind !== 'json' || raw === '') return null
  try {
    JSON.parse(raw)
    return null
  } catch (error: unknown) {
    return error instanceof Error ? error.message : `${field.label} is not valid JSON`
  }
}

export default function ToolsView() {
  const [selectedName, setSelectedName] = useState<string>(() => toolOf('quran.get_ayah').name)
  const [values, setValues] = useState<Record<string, string>>({})
  const [surface, setSurface] = useState<Surface>('ui')
  const [loading, setLoading] = useState(false)
  const [result, setResult] = useState<ToolResultData | null>(null)
  const [failure, setFailure] = useState<Failure | null>(null)
  const [sent, setSent] = useState<SentRun | null>(null)
  const [copyError, setCopyError] = useState<string | null>(null)

  const spec = toolOf(selectedName)

  function select(name: string) {
    setSelectedName(name)
    setValues({})
    setResult(null)
    setFailure(null)
    setCopyError(null)
  }

  const missingRequired = spec.fields.find(
    (field) => field.required && (values[field.key] ?? '') === '',
  )
  const fieldErrors = new Map<string, string>()
  for (const field of spec.fields) {
    const message = jsonErrorFor(field, values[field.key] ?? '')
    if (message !== null) fieldErrors.set(field.key, message)
  }
  const runReason =
    missingRequired !== undefined
      ? `required: ${missingRequired.label}`
      : fieldErrors.size > 0
        ? 'fix the field error below'
        : null
  const runDisabled = loading || runReason !== null

  async function run() {
    let params: Record<string, unknown>
    try {
      params = paramsJson(spec, values)
    } catch (error: unknown) {
      // Backstop only: `Run` stays disabled while the form is invalid.
      setFailure(
        error instanceof ToolParamError
          ? { summary: error.message }
          : { summary: String(error) },
      )
      return
    }
    setLoading(true)
    setResult(null)
    setFailure(null)
    setCopyError(null)
    try {
      const envelope = await runTool(spec.name, params)
      setResult(envelope.data)
      setSent({ name: spec.name, params })
    } catch (error: unknown) {
      setFailure(failureOf(error))
    } finally {
      setLoading(false)
    }
  }

  async function copyText(text: string) {
    try {
      await navigator.clipboard.writeText(text)
      setCopyError(null)
    } catch {
      setCopyError(
        'Copy failed: the clipboard is unavailable in this browser context. Select the text manually.',
      )
    }
  }

  const cliText = sent !== null ? cliCommand(sent.name, sent.params) : null
  const http = sent !== null ? httpRequest(sent.name, sent.params) : null

  return (
    <section>
      <PageHeader
        eyebrow="Instruments · Tools"
        title="Research Tools"
        lede="Every tool here is dispatched through the same registry as the CLI, so the checksum is identical across surfaces: run it here, paste the CLI line there, and both print the same research_checksum."
        actions={<span className="chip">same registry · same checksum</span>}
      />

      <div className="mt-8 grid gap-x-10 gap-y-8 lg:grid-cols-[minmax(0,18rem)_minmax(0,1fr)]">
        <div className="min-w-0">
          <p className="kicker mb-2">Catalogue</p>
          <div className="divide-y divide-border/60 overflow-hidden rounded-lg border bg-card">
            {DOMAIN_ORDER.map((domain) => {
              const group = TOOLS.filter((tool) => tool.domain === domain)
              if (group.length === 0) return null
              return (
                <div key={domain}>
                  <p className="border-b bg-muted/30 px-4 py-1.5 font-mono text-[0.625rem] uppercase tracking-[0.14em] text-muted-foreground">
                    {domain}
                  </p>
                  <ul className="divide-y">
                    {group.map((tool) => {
                      const active = tool.name === spec.name
                      return (
                        <li key={tool.name}>
                          <button
                            type="button"
                            onClick={() => select(tool.name)}
                            aria-current={active ? 'true' : undefined}
                            className={`flex w-full flex-col gap-0.5 px-4 py-2.5 text-left transition-colors ${
                              active ? 'bg-accent/60' : 'hover:bg-muted/40'
                            }`}
                          >
                            <span className="font-mono text-[0.72rem] font-medium">
                              {tool.name}
                            </span>
                            <span className="text-[0.75rem] leading-relaxed text-muted-foreground">
                              {tool.summary}
                            </span>
                          </button>
                        </li>
                      )
                    })}
                  </ul>
                </div>
              )
            })}
          </div>

          <div className="mt-6">
            <p className="kicker mb-2">Unavailable from the browser</p>
            <ul className="divide-y border-t">
              {DENIED.map((tool) => (
                <li
                  key={tool.name}
                  aria-disabled="true"
                  className="flex flex-wrap items-baseline gap-x-3 py-1.5 opacity-70"
                >
                  <span className="font-mono text-[0.7rem]">{tool.name}</span>
                  <span className="font-mono text-[0.65rem] text-muted-foreground">
                    {tool.code}
                  </span>
                  <span className="w-full text-[0.75rem] text-muted-foreground">
                    {tool.reason}
                  </span>
                </li>
              ))}
            </ul>
          </div>
        </div>

        <div className="min-w-0">
          <div className="flex flex-wrap items-center justify-between gap-3">
            <p className="kicker">{spec.name}</p>
            <div className="flex gap-0.5" role="group" aria-label="invocation surface">
              {(['ui', 'cli', 'http'] as const).map((candidate) => (
                <button
                  key={candidate}
                  type="button"
                  className="seg px-3 py-1 text-[0.75rem]"
                  aria-pressed={surface === candidate}
                  onClick={() => setSurface(candidate)}
                >
                  {candidate}
                </button>
              ))}
            </div>
          </div>
          <p className="margin-note mt-2">
            The research_checksum is computed by the server over the tool name, version,
            query and payload, so the same params through <span className="font-mono">ui</span>,{' '}
            <span className="font-mono">cli</span> and <span className="font-mono">http</span>{' '}
            print the same hex. The CLI line on this page is copyable, not executed here,
            because the browser cannot spawn a process.
          </p>

          {surface === 'ui' && (
            <form
              onSubmit={(event) => {
                event.preventDefault()
                void run()
              }}
              className="mt-4 space-y-3"
            >
              <p className="text-[0.8125rem] leading-relaxed text-muted-foreground">
                {spec.summary}
              </p>
              {spec.fields.map((field) => {
                const id = `tool-${spec.name}-${field.key}`
                const raw = values[field.key] ?? ''
                const error = fieldErrors.get(field.key)
                return (
                  <div key={field.key}>
                    <label htmlFor={id} className="kicker mb-2 block">
                      {field.label}
                      {!field.required && (
                        <span className="ml-2 normal-case tracking-normal text-muted-foreground/70">
                          optional
                        </span>
                      )}
                    </label>
                    {field.kind === 'bool' ? (
                      <button
                        type="button"
                        id={id}
                        role="checkbox"
                        aria-checked={raw === 'true'}
                        aria-label={field.label}
                        onClick={() =>
                          setValues((prev) => ({
                            ...prev,
                            [field.key]: raw === 'true' ? '' : 'true',
                          }))
                        }
                        className="seg px-3 py-1 text-[0.75rem]"
                      >
                        {raw === 'true' ? 'true' : 'false'}
                      </button>
                    ) : field.kind === 'json' ? (
                      <textarea
                        id={id}
                        value={raw}
                        rows={2}
                        onChange={(event) =>
                          setValues((prev) => ({ ...prev, [field.key]: event.target.value }))
                        }
                        spellCheck={false}
                        placeholder={field.placeholder}
                        aria-invalid={error !== undefined}
                        className="field font-mono text-xs"
                      />
                    ) : (
                      <input
                        id={id}
                        value={raw}
                        onChange={(event) =>
                          setValues((prev) => ({ ...prev, [field.key]: event.target.value }))
                        }
                        spellCheck={false}
                        placeholder={field.placeholder}
                        inputMode={field.kind === 'number' ? 'numeric' : undefined}
                        className="field font-mono text-xs"
                      />
                    )}
                    {field.hint !== undefined && (
                      <p className="margin-note mt-1">{field.hint}</p>
                    )}
                    {error !== undefined && (
                      <p role="alert" className="mt-1.5 text-[0.7rem] text-destructive">
                        {error}
                      </p>
                    )}
                  </div>
                )
              })}
              <div className="flex flex-wrap items-center gap-3">
                <button type="submit" className="btn btn-primary" disabled={runDisabled}>
                  {loading ? (
                    <Loader2 className="size-3.5 animate-spin" aria-hidden="true" />
                  ) : (
                    <Send className="size-3.5" aria-hidden="true" />
                  )}
                  Run {spec.name}
                </button>
                {runReason !== null && (
                  <span className="text-[0.75rem] text-muted-foreground">{runReason}</span>
                )}
              </div>
            </form>
          )}

          {surface !== 'ui' &&
            (sent === null || cliText === null || http === null ? (
              <div className="mt-4">
                <EmptyState title={`no ${spec.name} run to mirror`}>
                  Run the tool on the <span className="font-mono">ui</span> surface first —
                  the {surface} panel prints the exact invocation of the params that produced
                  the ledger checksum.
                </EmptyState>
              </div>
            ) : (
              <div className="mt-4 space-y-3">
                <p className="text-[0.8125rem] leading-relaxed text-muted-foreground">
                  {surface === 'cli'
                    ? `The checksum shown on the ui surface is the one the CLI returns for the same --json invocation.`
                    : `The checksum shown on the ui surface is the one this request returns for the same JSON body.`}
                </p>
                {surface === 'cli' ? (
                  <pre className="carbon overflow-x-auto rounded-lg border px-3.5 py-3 font-mono text-[0.72rem] leading-relaxed">
                    {cliText}
                  </pre>
                ) : (
                  <div className="overflow-hidden rounded-lg border">
                    <p className="border-b bg-muted/30 px-3.5 py-2 font-mono text-[0.72rem]">
                      {http.method} {http.path}
                    </p>
                    <pre className="carbon overflow-x-auto px-3.5 py-3 font-mono text-[0.72rem] leading-relaxed">
                      {http.body}
                    </pre>
                  </div>
                )}
                <div className="flex flex-wrap items-center gap-3">
                  <button
                    type="button"
                    onClick={() =>
                      void copyText(surface === 'cli' ? cliText : http.body)
                    }
                    className="btn btn-sm"
                  >
                    <Copy className="size-3" aria-hidden="true" />
                    copy {surface} invocation
                  </button>
                  {copyError !== null && (
                    <span role="alert" className="text-[0.75rem] text-destructive">
                      {copyError}
                    </span>
                  )}
                </div>
              </div>
            ))}

          {surface === 'ui' && (
            <div className="mt-4">
              {loading && <LoadingLine label={`running ${spec.name}…`} />}

              {failure !== null && (
                <ErrorNote>
                  {failure.code !== undefined && (
                    <span className="font-mono text-[0.75rem]">{failure.code} — </span>
                  )}
                  {failure.summary}
                  {failure.remedy !== undefined && (
                    <span className="mt-1 block text-foreground/80">{failure.remedy}</span>
                  )}
                </ErrorNote>
              )}

              {result !== null ? (
                <div className="space-y-6 border-t pt-4">
                  <ToolLedger result={result} />
                </div>
              ) : (
                !loading &&
                failure === null && (
                  <EmptyState title="no tool has run">
                    Fill the form and run. The tool result prints here verbatim with its
                    server checksum.
                  </EmptyState>
                )
              )}
            </div>
          )}
        </div>
      </div>
    </section>
  )
}
