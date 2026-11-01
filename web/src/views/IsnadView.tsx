// Isnad reading room: the three read-only isnad instruments — trace,
// narrator, rijal — each rendered through the shared ledger. The identity
// mutations below have no form and no control: they write durable identity
// state, the HTTP route refuses them by default, and only the local CLI
// (`qai isnad …`) is the trusted operator surface.
import { useState } from 'react'
import { PageHeader } from '../components/layout/page-header'
import { EmptyState, ErrorNote, LoadingLine } from '../components/states'
import ToolLedger from '../components/ToolLedger'
import { runTool } from '../api/client'
import type { ToolResultData } from '../lib/tool-result'
import { DENIED } from '../lib/tools'

type Instrument = 'trace' | 'narrator' | 'rijal'

const INSTRUMENTS: readonly Instrument[] = ['trace', 'narrator', 'rijal']

const IDENTITY_DENIED = DENIED.filter((tool) => tool.name.startsWith('isnad.identity'))

interface Failure {
  code?: string
  summary: string
}

function failureOf(error: unknown): Failure {
  if (error instanceof Error) {
    const code = (error as { code?: unknown }).code
    if (typeof code === 'string' && code.length > 0) return { code, summary: error.message }
    return { summary: error.message }
  }
  return { summary: String(error) }
}

export default function IsnadView() {
  const [instrument, setInstrument] = useState<Instrument>('trace')
  const [from, setFrom] = useState('')
  const [to, setTo] = useState('')
  const [k, setK] = useState('')
  const [id, setId] = useState('')
  const [loading, setLoading] = useState(false)
  const [result, setResult] = useState<ToolResultData | null>(null)
  const [failure, setFailure] = useState<Failure | null>(null)

  const runReason =
    instrument === 'trace'
      ? from.trim() === ''
        ? 'required: From'
        : to.trim() === ''
          ? 'required: To'
          : null
      : id.trim() === ''
        ? 'required: Id'
        : null
  const runDisabled = loading || runReason !== null

  async function run() {
    if (runReason !== null) return
    setLoading(true)
    setResult(null)
    setFailure(null)
    try {
      const name = `isnad.${instrument}`
      const params =
        instrument === 'trace'
          ? {
              from: from.trim(),
              to: to.trim(),
              ...(k.trim() === '' ? {} : { k: Number(k.trim()) }),
            }
          : { id: id.trim() }
      const envelope = await runTool(name, params)
      setResult(envelope.data)
    } catch (error: unknown) {
      setFailure(failureOf(error))
    } finally {
      setLoading(false)
    }
  }

  function select(candidate: Instrument) {
    setInstrument(candidate)
    setResult(null)
    setFailure(null)
  }

  return (
    <section>
      <PageHeader
        eyebrow="Instruments · Transmission"
        title="Isnad"
        lede="This screen reads the narrator graph. It never proposes, accepts, rejects, disputes or supersedes an identity — those tools have no control here."
        actions={<span className="chip">read-only</span>}
      />

      <div className="mt-6">
        <div className="flex gap-0.5" role="group" aria-label="isnad instrument">
          {INSTRUMENTS.map((candidate) => (
            <button
              key={candidate}
              type="button"
              className="seg px-3 py-1 text-[0.75rem]"
              aria-pressed={instrument === candidate}
              onClick={() => select(candidate)}
            >
              {candidate}
            </button>
          ))}
        </div>

        <form
          onSubmit={(event) => {
            event.preventDefault()
            void run()
          }}
          className="mt-4 space-y-3"
        >
          {instrument === 'trace' && (
            <>
              <div>
                <label htmlFor="isnad-from" className="kicker mb-2 block">
                  From
                </label>
                <input
                  id="isnad-from"
                  value={from}
                  onChange={(event) => setFrom(event.target.value)}
                  spellCheck={false}
                  className="field font-mono text-xs"
                />
              </div>
              <div>
                <label htmlFor="isnad-to" className="kicker mb-2 block">
                  To
                </label>
                <input
                  id="isnad-to"
                  value={to}
                  onChange={(event) => setTo(event.target.value)}
                  spellCheck={false}
                  className="field font-mono text-xs"
                />
              </div>
              <div>
                <label htmlFor="isnad-k" className="kicker mb-2 block">
                  K
                  <span className="ml-2 normal-case tracking-normal text-muted-foreground/70">
                    optional
                  </span>
                </label>
                <input
                  id="isnad-k"
                  value={k}
                  onChange={(event) => setK(event.target.value)}
                  spellCheck={false}
                  inputMode="numeric"
                  className="field font-mono text-xs"
                />
              </div>
            </>
          )}

          {instrument !== 'trace' && (
            <div>
              <label htmlFor="isnad-id" className="kicker mb-2 block">
                Id
              </label>
              <input
                id="isnad-id"
                value={id}
                onChange={(event) => setId(event.target.value)}
                spellCheck={false}
                className="field font-mono text-xs"
              />
            </div>
          )}

          <div className="flex flex-wrap items-center gap-3">
            <button type="submit" className="btn btn-primary" disabled={runDisabled}>
              Run isnad.{instrument}
            </button>
            {runReason !== null && (
              <p className="text-[0.75rem] text-muted-foreground">{runReason}</p>
            )}
          </div>
        </form>

        <div className="mt-4">
          {loading && <LoadingLine label={`running isnad.${instrument}…`} />}

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
              <EmptyState title={`no isnad.${instrument} run yet`}>
                Fill the fields above and run. The tool result prints here verbatim, with its
                checksum and reproducibility.
              </EmptyState>
            )
          )}
        </div>

        <div className="mt-6">
          <p className="kicker mb-2">Unavailable from the browser</p>
          <ul className="divide-y border-t">
            {IDENTITY_DENIED.map((tool) => (
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
                  This tool writes durable identity state, so the HTTP route refuses it by
                  default; the trusted operator surface is `qai isnad …` on the local CLI.
                </span>
              </li>
            ))}
          </ul>
        </div>
      </div>
    </section>
  )
}
