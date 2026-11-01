// Words & Roots (13-11): the browser face of the morphology the engine
// holds — one input, one tool call, the shared ledger. Lexicon and count
// instruments attach to this screen in later tasks of the same plan. No
// Arabic verse body renders here, so the canonical-text rules do not apply.
import { useState } from 'react'
import { PageHeader } from '../components/layout/page-header'
import { EmptyState, ErrorNote, LoadingLine } from '../components/states'
import ToolLedger from '../components/ToolLedger'
import { runTool, type ApiRequestError } from '../api/client'
import type { ToolResultData } from '../lib/tool-result'
import { parseWordInput } from '../lib/words'

const EMPTY_REASON = 'enter a reference, root or lemma'
const SHAPE_REASON = 'use surah:ayah:position, surah:ayah, a root or a lemma'

function failureOf(error: unknown): { code?: string; summary: string } {
  if (error instanceof Error) {
    const code = (error as Partial<ApiRequestError>).code
    return code === undefined ? { summary: error.message } : { code, summary: error.message }
  }
  return { summary: String(error) }
}

export default function WordsView() {
  const [input, setInput] = useState('')
  const [loading, setLoading] = useState(false)
  const [result, setResult] = useState<ToolResultData | null>(null)
  const [failure, setFailure] = useState<{ code?: string; summary: string } | null>(null)
  const [notice, setNotice] = useState<string | null>(null)

  const trimmed = input.trim()
  const query = parseWordInput(input)
  const reason = trimmed === '' ? EMPTY_REASON : query === null ? SHAPE_REASON : null

  async function run() {
    if (query === null) return
    if (query.kind === 'text') {
      setNotice(`“${query.text}” reads as lexicon text — root, lemma and family lookups attach here next.`)
      return
    }
    setLoading(true)
    setResult(null)
    setFailure(null)
    setNotice(null)
    try {
      const envelope = await runTool('quran.morphology', {
        surah: query.surah,
        ayah: query.ayah,
        position: query.kind === 'token' ? query.position : 1,
      })
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
        eyebrow="Instruments · Lexicon"
        title="Words & Roots"
        lede="Every row below is served from the morphology the engine holds — analyses print as received, never invented."
        actions={<span className="chip">read-only</span>}
      />

      <form
        onSubmit={(event) => {
          event.preventDefault()
          void run()
        }}
        className="mt-8 max-w-2xl space-y-3"
      >
        <div>
          <label htmlFor="words-input" className="kicker mb-2 block">
            Reference, root or lemma
          </label>
          <div className="flex flex-col gap-2 sm:flex-row">
            <input
              id="words-input"
              value={input}
              onChange={(event) => setInput(event.target.value)}
              dir="auto"
              spellCheck={false}
              placeholder="2:1:3"
              className="field h-9 flex-1 font-mono text-xs"
            />
            <button
              type="submit"
              className="btn btn-primary h-9 justify-center px-4"
              disabled={reason !== null || loading}
              title={reason ?? undefined}
            >
              {loading ? 'consulting…' : 'Run'}
            </button>
          </div>
          <p className="margin-note mt-2">
            Shapes: <span className="font-mono">surah:ayah:position</span>,{' '}
            <span className="font-mono">surah:ayah</span>, a root, or a lemma.
          </p>
          {reason !== null && !loading && <p className="margin-note mt-1">{reason}</p>}
        </div>
      </form>

      <div className="mt-6 max-w-2xl">
        {loading && <LoadingLine label="consulting morphology…" />}

        {notice !== null && !loading && <p className="margin-note border-t pt-4">{notice}</p>}

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
          failure === null &&
          notice === null && (
            <EmptyState title="no word lookup has run">
              Enter a token reference and run. The morphology analyses print here verbatim.
            </EmptyState>
          )
        )}
      </div>
    </section>
  )
}
