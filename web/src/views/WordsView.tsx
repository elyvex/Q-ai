// Words & Roots (13-11): the browser face of the morphology the engine
// holds — one input, the lexicon instruments, the counts, the shared
// ledger. Occurrence rows link to the reader only through the edition the
// server pinned; a count never prints without its rules. No Arabic verse
// body renders here, so the canonical-text rules do not apply.
import { useState } from 'react'
import { PageHeader } from '../components/layout/page-header'
import { EmptyState, ErrorNote, LoadingLine } from '../components/states'
import ToolLedger from '../components/ToolLedger'
import { runTool, type ApiRequestError } from '../api/client'
import {
  deepLinkFor,
  editionOf,
  rowsOf,
  type ToolResultData,
} from '../lib/tool-result'
import { familyKindFor, parseWordInput, tokenId } from '../lib/words'

const EMPTY_REASON = 'enter a reference, root or lemma'
const SHAPE_REASON = 'use surah:ayah:position, surah:ayah, a root or a lemma'

function failureOf(error: unknown): { code?: string; summary: string } {
  if (error instanceof Error) {
    const code = (error as Partial<ApiRequestError>).code
    return code === undefined ? { summary: error.message } : { code, summary: error.message }
  }
  return { summary: String(error) }
}

type LexiconMode = 'root' | 'lemma' | 'family'

interface OccurrenceRow {
  surah: number
  ayah: number
  position: number
  dataset: string | null
}

function occurrenceOf(row: Record<string, unknown>): OccurrenceRow | null {
  if (typeof row.surah !== 'number' || typeof row.ayah !== 'number' || typeof row.position !== 'number') {
    return null
  }
  return {
    surah: row.surah,
    ayah: row.ayah,
    position: row.position,
    dataset: typeof row.dataset === 'string' ? row.dataset : null,
  }
}

interface FamilyRow {
  id: string
  kind: string
  relation: string
  explanation: string
  dataset: string | null
}

function familyOf(row: Record<string, unknown>): FamilyRow | null {
  if (
    typeof row.id !== 'string' ||
    typeof row.kind !== 'string' ||
    typeof row.relation !== 'string' ||
    typeof row.explanation !== 'string'
  ) {
    return null
  }
  return {
    id: row.id,
    kind: row.kind,
    relation: row.relation,
    explanation: row.explanation,
    dataset: typeof row.dataset === 'string' ? row.dataset : null,
  }
}

/** The single edition the server pinned, or null when no deep link can be built. */
function pinnedEdition(result: ToolResultData): { slug: string; version: string } | null {
  const slugs = Object.keys(result.reproducibility.edition_versions)
  if (slugs.length !== 1) return null
  return editionOf(result.reproducibility, slugs[0])
}

function DatasetAttribution({ datasets }: { datasets: string[] }) {
  return (
    <div className="mt-4">
      <p className="kicker mb-2">dataset attribution</p>
      {datasets.length === 0 ? (
        <p className="margin-note">No dataset stated these rows.</p>
      ) : (
        <ul className="space-y-1">
          {datasets.map((dataset) => (
            <li key={dataset} className="font-mono text-[0.7rem] text-muted-foreground">
              {dataset}
            </li>
          ))}
        </ul>
      )}
      {datasets.length > 1 && (
        <p className="margin-note mt-2">
          Rows span {datasets.length} datasets; every row is shown as served.
        </p>
      )}
    </div>
  )
}

function OccurrenceTable({ result }: { result: ToolResultData }) {
  const rows = rowsOf(result.results)
    .map(occurrenceOf)
    .filter((row): row is OccurrenceRow => row !== null)
  const pinned = pinnedEdition(result)
  const datasets = [
    ...new Set(
      rows.map((row) => row.dataset).filter((dataset): dataset is string => dataset !== null),
    ),
  ]

  return (
    <div>
      {pinned === null && (
        <p className="margin-note mb-3 border-t pt-3">
          No single edition can be pinned, so no deep link can be built.
        </p>
      )}
      <div className="overflow-x-auto border-t">
        <table className="w-full border-collapse text-left">
          <thead>
            <tr className="border-b">
              {['reference', 'position', 'dataset'].map((column) => (
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
            {rows.map((row, i) => {
              const reference = `${row.surah}:${row.ayah}`
              const href =
                pinned !== null
                  ? deepLinkFor(`quran:${pinned.slug}@${pinned.version}:${reference}`)
                  : null
              return (
                <tr key={`${reference}:${row.position}:${i}`}>
                  <td className="px-3 py-2 font-mono text-[0.8125rem]">
                    {href !== null ? (
                      <a href={href} className="text-primary underline-offset-4 hover:underline">
                        {reference}
                      </a>
                    ) : (
                      <span>{reference}</span>
                    )}
                  </td>
                  <td className="px-3 py-2 font-mono text-[0.8125rem]">{row.position}</td>
                  <td className="px-3 py-2 font-mono text-[0.7rem] text-muted-foreground">
                    {row.dataset ?? '—'}
                  </td>
                </tr>
              )
            })}
          </tbody>
        </table>
      </div>
      <DatasetAttribution datasets={datasets} />
    </div>
  )
}

const TOKEN_MEMBER = /^token:(\d+):(\d+):(\d+)$/

function FamilyList({ result }: { result: ToolResultData }) {
  const members = rowsOf(result.results)
    .map(familyOf)
    .filter((row): row is FamilyRow => row !== null)
  const pinned = pinnedEdition(result)
  const datasets = [
    ...new Set(
      members.map((member) => member.dataset).filter((dataset): dataset is string => dataset !== null),
    ),
  ]

  return (
    <div>
      <ul className="divide-y border-t">
        {members.map((member, i) => {
          const token = TOKEN_MEMBER.exec(member.id)
          const href =
            token !== null && pinned !== null
              ? deepLinkFor(`quran:${pinned.slug}@${pinned.version}:${token[1]}:${token[2]}`)
              : null
          return (
            <li key={`${member.id}:${i}`} className="py-3">
              <p className="flex flex-wrap items-baseline gap-x-3">
                {href !== null ? (
                  <a href={href} className="text-[0.8125rem] font-medium text-primary underline-offset-4 hover:underline">
                    {member.relation}
                  </a>
                ) : (
                  <span className="text-[0.8125rem] font-medium">{member.relation}</span>
                )}
                <span className="font-mono text-[0.65rem] text-muted-foreground">{member.kind}</span>
                {member.dataset !== null && (
                  <span className="font-mono text-[0.65rem] text-muted-foreground">
                    {member.dataset}
                  </span>
                )}
              </p>
              <p className="mt-1 max-w-2xl text-[0.8125rem] leading-relaxed text-foreground/90">
                {member.explanation}
              </p>
            </li>
          )
        })}
      </ul>
      <DatasetAttribution datasets={datasets} />
    </div>
  )
}

export default function WordsView() {
  const [input, setInput] = useState('')
  const [loading, setLoading] = useState(false)
  const [result, setResult] = useState<ToolResultData | null>(null)
  const [failure, setFailure] = useState<{ code?: string; summary: string } | null>(null)
  const [notice, setNotice] = useState<string | null>(null)

  const [modeOverride, setModeOverride] = useState<LexiconMode | null>(null)
  const [kindOverride, setKindOverride] = useState<string | null>(null)
  const [idOverride, setIdOverride] = useState<string | null>(null)
  const [lexLoading, setLexLoading] = useState(false)
  const [lexResult, setLexResult] = useState<{ mode: LexiconMode; result: ToolResultData } | null>(null)
  const [lexFailure, setLexFailure] = useState<{ code?: string; summary: string } | null>(null)

  const trimmed = input.trim()
  const query = parseWordInput(input)
  const reason = trimmed === '' ? EMPTY_REASON : query === null ? SHAPE_REASON : null

  const lexMode: LexiconMode = modeOverride ?? (query?.kind === 'token' ? 'family' : 'root')
  const queryText = query?.kind === 'text' ? query.text : ''
  const familyKind = kindOverride ?? (query ? familyKindFor(query) : 'root')
  const familyId = idOverride ?? (query?.kind === 'token' ? tokenId(query) : '')
  const lexReason =
    lexMode === 'family'
      ? familyId.trim() === ''
        ? 'enter a family member id'
        : null
      : queryText === ''
        ? lexMode === 'root'
          ? 'enter a root in the field above'
          : 'enter a lemma in the field above'
        : null

  function resetLexiconOverrides() {
    setModeOverride(null)
    setKindOverride(null)
    setIdOverride(null)
  }

  async function run() {
    if (query === null) return
    if (query.kind === 'text') {
      setNotice(`“${query.text}” reads as lexicon text — run the lexicon lookup below.`)
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

  async function runLexicon() {
    if (lexReason !== null) return
    setLexLoading(true)
    setLexResult(null)
    setLexFailure(null)
    try {
      const name = lexMode === 'root' ? 'quran.root' : lexMode === 'lemma' ? 'quran.lemma' : 'quran.family'
      const params =
        lexMode === 'root'
          ? { root: queryText }
          : lexMode === 'lemma'
            ? { lemma: queryText }
            : { kind: familyKind, id: familyId.trim() }
      const envelope = await runTool(name, params)
      setLexResult({ mode: lexMode, result: envelope.data })
    } catch (error: unknown) {
      setLexFailure(failureOf(error))
    } finally {
      setLexLoading(false)
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
              onChange={(event) => {
                setInput(event.target.value)
                resetLexiconOverrides()
              }}
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

      <div className="mt-10 max-w-2xl border-t pt-6">
        <p className="kicker mb-2">Lexicon</p>
        <div className="flex gap-0.5" role="group" aria-label="lexicon instrument">
          {(['root', 'lemma', 'family'] as const).map((candidate) => (
            <button
              key={candidate}
              type="button"
              className="seg px-3 py-1 text-[0.75rem]"
              aria-pressed={lexMode === candidate}
              onClick={() => setModeOverride(candidate)}
            >
              {candidate}
            </button>
          ))}
        </div>
        <p className="margin-note mt-2">
          {lexMode === 'family'
            ? 'Family reads the kind and member id; a token reference fills the id.'
            : `The ${lexMode} lookup reads the text in the field above.`}
        </p>

        {lexMode === 'family' && (
          <div className="mt-3 grid gap-3 sm:grid-cols-[10rem_minmax(0,1fr)]">
            <div>
              <label htmlFor="words-kind" className="kicker mb-2 block">
                Kind
              </label>
              <select
                id="words-kind"
                value={familyKind}
                onChange={(event) => setKindOverride(event.target.value)}
                className="field h-9 font-mono text-xs"
              >
                {['token', 'root', 'lemma'].map((kind) => (
                  <option key={kind} value={kind}>
                    {kind}
                  </option>
                ))}
              </select>
            </div>
            <div>
              <label htmlFor="words-member-id" className="kicker mb-2 block">
                Member id
              </label>
              <input
                id="words-member-id"
                value={familyId}
                onChange={(event) => setIdOverride(event.target.value)}
                dir="auto"
                spellCheck={false}
                placeholder="token:2:1:3"
                className="field h-9 font-mono text-xs"
              />
            </div>
          </div>
        )}

        <div className="mt-3">
          <button
            type="button"
            className="btn btn-primary h-9 justify-center px-4"
            onClick={() => void runLexicon()}
            disabled={lexReason !== null || lexLoading}
            title={lexReason ?? undefined}
          >
            {lexLoading ? 'consulting…' : 'Look up'}
          </button>
          {lexReason !== null && !lexLoading && <p className="margin-note mt-1">{lexReason}</p>}
        </div>

        <div className="mt-6">
          {lexLoading && <LoadingLine label={`consulting ${lexMode}…`} />}

          {lexFailure !== null && (
            <ErrorNote>
              {lexFailure.code !== undefined && (
                <span className="font-mono text-[0.75rem]">{lexFailure.code} — </span>
              )}
              {lexFailure.summary}
            </ErrorNote>
          )}

          {lexResult !== null &&
            (lexResult.mode === 'family' ? (
              <FamilyList result={lexResult.result} />
            ) : (
              <OccurrenceTable result={lexResult.result} />
            ))}
        </div>
      </div>
    </section>
  )
}
