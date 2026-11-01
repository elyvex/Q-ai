// Words & Roots (13-11): the browser face of the morphology the engine
// holds — one input, the lexicon instruments, the counts, the shared
// ledger. Occurrence rows link to the reader only through the edition the
// server pinned; a count never prints without its rules. No Arabic verse
// body renders here, so the canonical-text rules do not apply.
import { useState } from 'react'
import { PageHeader } from '../components/layout/page-header'
import { EmptyState, ErrorNote, LoadingLine } from '../components/states'
import ToolLedger from '../components/ToolLedger'
import {
  cooccurrence,
  distribution,
  frequency,
  lemmaFrequency,
  rootFrequency,
  runTool,
  type ApiRequestError,
} from '../api/client'
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

type CountKind = 'root-frequency' | 'lemma-frequency' | 'frequency' | 'distribution' | 'cooccurrence'

const COUNT_INSTRUMENTS: Array<{ id: CountKind; label: string }> = [
  { id: 'root-frequency', label: 'root frequency' },
  { id: 'lemma-frequency', label: 'lemma frequency' },
  { id: 'frequency', label: 'token frequency' },
  { id: 'distribution', label: 'distribution' },
  { id: 'cooccurrence', label: 'cooccurrence' },
]

interface CountState {
  loading: boolean
  data: unknown
  failure: { code?: string; summary: string } | null
}

interface CountingRules {
  profile: string
  profile_version: string
  datasets: unknown
  multi_analysis_handling: unknown
  window: string | null
  exclusions: unknown
}

function ruleText(value: unknown): string {
  if (value === null || value === undefined) return 'none'
  if (typeof value === 'string') return value === '' ? '—' : value
  if (Array.isArray(value)) return value.length > 0 ? value.map((entry) => String(entry)).join(', ') : 'none'
  return JSON.stringify(value)
}

function isCountingRules(value: unknown): value is CountingRules {
  if (typeof value !== 'object' || value === null) return false
  const rules = value as Record<string, unknown>
  return (
    typeof rules.profile === 'string' &&
    typeof rules.profile_version === 'string' &&
    (typeof rules.window === 'string' || rules.window === null)
  )
}

function RulesBlock({ rules }: { rules: CountingRules }) {
  const rows: Array<[string, string]> = [
    ['profile', ruleText(rules.profile)],
    ['profile_version', ruleText(rules.profile_version)],
    ['datasets', ruleText(rules.datasets)],
    ['multi_analysis_handling', ruleText(rules.multi_analysis_handling)],
    ['window', rules.window === null ? 'none' : ruleText(rules.window)],
    ['exclusions', ruleText(rules.exclusions)],
  ]
  return (
    <dl className="mt-3 border-t">
      {rows.map(([key, value]) => (
        <div key={key} className="flex flex-wrap gap-x-3 py-1">
          <dt className="w-44 shrink-0 font-mono text-[0.65rem] text-muted-foreground">{key}</dt>
          <dd className="min-w-0 flex-1 font-mono text-[0.7rem] break-words">{value}</dd>
        </div>
      ))}
    </dl>
  )
}

interface FrequencyReportData {
  target: string
  rules: CountingRules
  count: number
  by_surah: Record<string, unknown>
  checksum: string
}

function isFrequencyReport(data: unknown): data is FrequencyReportData {
  if (typeof data !== 'object' || data === null) return false
  const report = data as Record<string, unknown>
  return (
    typeof report.count === 'number' &&
    typeof report.checksum === 'string' &&
    typeof report.by_surah === 'object' &&
    report.by_surah !== null &&
    !Array.isArray(report.by_surah) &&
    isCountingRules(report.rules)
  )
}

function BySurahTable({ bySurah }: { bySurah: Record<string, unknown> }) {
  const entries = Object.entries(bySurah)
    .map(([surah, count]) => ({ surah: Number(surah), count: typeof count === 'number' ? count : null }))
    .filter((entry) => Number.isInteger(entry.surah) && entry.count !== null)
    .sort((a, b) => a.surah - b.surah)
  if (entries.length === 0) return null
  return (
    <div className="mt-3 overflow-x-auto border-t">
      <table className="w-full border-collapse text-left">
        <thead>
          <tr className="border-b">
            {['surah', 'count'].map((column) => (
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
          {entries.map((entry) => (
            <tr key={entry.surah}>
              <td className="px-3 py-2 font-mono text-[0.8125rem]">{entry.surah}</td>
              <td className="px-3 py-2 font-mono text-[0.8125rem] tabular-nums">{entry.count}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}

function FrequencyOutput({ label, data }: { label: string; data: unknown }) {
  if (!isFrequencyReport(data)) {
    return <p className="margin-note mt-3">The server returned an unrecognized {label} shape.</p>
  }
  return (
    <div className="mt-3">
      <p className="kicker mb-2">{label}</p>
      {data.count === 0 ? (
        <EmptyState title="zero under these rules">
          No tokens matched under these rules — a proven zero, not an absence.
        </EmptyState>
      ) : (
        <p className="font-mono text-[0.9375rem] tabular-nums">{data.count}</p>
      )}
      <RulesBlock rules={data.rules} />
      <BySurahTable bySurah={data.by_surah} />
      <p className="mt-2 font-mono text-[0.7rem] text-muted-foreground">{data.checksum}</p>
    </div>
  )
}

function DistributionOutput({ data }: { data: unknown }) {
  if (typeof data !== 'object' || data === null) {
    return <p className="margin-note mt-3">The server returned an unrecognized distribution shape.</p>
  }
  const report = data as Record<string, unknown>
  if (!isFrequencyReport(report.frequency)) {
    return <p className="margin-note mt-3">The server returned an unrecognized distribution shape.</p>
  }
  const frequencyReport = report.frequency
  const warnings = Array.isArray(report.warnings)
    ? report.warnings.filter((warning): warning is string => typeof warning === 'string')
    : []
  return (
    <div className="mt-3">
      <p className="kicker mb-2">distribution</p>
      {frequencyReport.count === 0 ? (
        <EmptyState title="zero under these rules">
          No tokens matched under these rules — a proven zero, not an absence.
        </EmptyState>
      ) : (
        <p className="font-mono text-[0.9375rem] tabular-nums">{frequencyReport.count}</p>
      )}
      <RulesBlock rules={frequencyReport.rules} />
      {typeof report.partition_provenance === 'string' && (
        <p className="margin-note mt-3">{report.partition_provenance}</p>
      )}
      {warnings.length > 0 && (
        <ul className="mt-2 space-y-1 border-t pt-2">
          {warnings.map((warning, i) => (
            <li key={i} className="text-[0.8125rem] text-muted-foreground">
              {warning}
            </li>
          ))}
        </ul>
      )}
      <BySurahTable bySurah={frequencyReport.by_surah} />
      <p className="mt-2 font-mono text-[0.7rem] text-muted-foreground">{frequencyReport.checksum}</p>
    </div>
  )
}

interface CooccurrenceHit {
  form: string
  count: number
  spans_ayah_boundary: boolean
}

function cooccurrenceHitOf(hit: unknown): CooccurrenceHit | null {
  if (typeof hit !== 'object' || hit === null) return null
  const row = hit as Record<string, unknown>
  if (typeof row.form !== 'string' || typeof row.count !== 'number' || typeof row.spans_ayah_boundary !== 'boolean') {
    return null
  }
  return { form: row.form, count: row.count, spans_ayah_boundary: row.spans_ayah_boundary }
}

function CooccurrenceOutput({ data }: { data: unknown }) {
  if (typeof data !== 'object' || data === null) {
    return <p className="margin-note mt-3">The server returned an unrecognized cooccurrence shape.</p>
  }
  const report = data as Record<string, unknown>
  if (!isCountingRules(report.rules) || !Array.isArray(report.hits)) {
    return <p className="margin-note mt-3">The server returned an unrecognized cooccurrence shape.</p>
  }
  const hits = report.hits
    .map(cooccurrenceHitOf)
    .filter((hit): hit is CooccurrenceHit => hit !== null)
  return (
    <div className="mt-3">
      <p className="kicker mb-2">cooccurrence</p>
      {hits.length === 0 ? (
        <EmptyState title="zero under these rules">
          No co-occurring forms under these rules — a proven zero, not an absence.
        </EmptyState>
      ) : (
        <div className="overflow-x-auto border-t">
          <table className="w-full border-collapse text-left">
            <thead>
              <tr className="border-b">
                {['form', 'count', 'spans_ayah_boundary'].map((column) => (
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
              {hits.map((hit, i) => (
                <tr key={`${hit.form}:${i}`}>
                  <td className="px-3 py-2 font-mono text-[0.8125rem]">{hit.form}</td>
                  <td className="px-3 py-2 font-mono text-[0.8125rem] tabular-nums">{hit.count}</td>
                  <td className="px-3 py-2 font-mono text-[0.8125rem]">
                    {String(hit.spans_ayah_boundary)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
      <RulesBlock rules={report.rules} />
    </div>
  )
}

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
  const [targetOverride, setTargetOverride] = useState<string | null>(null)
  const [profileInput, setProfileInput] = useState('L3.diacritics')
  const [windowInput, setWindowInput] = useState('5')
  const [lexLoading, setLexLoading] = useState(false)
  const [lexResult, setLexResult] = useState<{ mode: LexiconMode; result: ToolResultData } | null>(null)
  const [lexFailure, setLexFailure] = useState<{ code?: string; summary: string } | null>(null)
  const [counts, setCounts] = useState<Record<CountKind, CountState>>({
    'root-frequency': { loading: false, data: null, failure: null },
    'lemma-frequency': { loading: false, data: null, failure: null },
    frequency: { loading: false, data: null, failure: null },
    distribution: { loading: false, data: null, failure: null },
    cooccurrence: { loading: false, data: null, failure: null },
  })

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

  const countTarget = targetOverride ?? queryText
  const windowNumber = Number(windowInput)
  const windowValid = Number.isInteger(windowNumber) && windowNumber > 0

  function countReason(kind: CountKind): string | null {
    if (kind === 'cooccurrence' && !windowValid) return 'enter a positive window'
    if (countTarget.trim() === '') return 'enter a count target above'
    return null
  }

  function resetLexiconOverrides() {
    setModeOverride(null)
    setKindOverride(null)
    setIdOverride(null)
    setTargetOverride(null)
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

  async function runCount(kind: CountKind) {
    if (countReason(kind) !== null) return
    setCounts((prev) => ({ ...prev, [kind]: { loading: true, data: null, failure: null } }))
    try {
      let data: unknown
      if (kind === 'root-frequency') data = (await rootFrequency(countTarget)).data
      else if (kind === 'lemma-frequency') data = (await lemmaFrequency(countTarget)).data
      else if (kind === 'frequency') data = (await frequency(countTarget, profileInput)).data
      else if (kind === 'distribution') data = (await distribution(countTarget, profileInput)).data
      else data = (await cooccurrence(countTarget, profileInput, windowNumber)).data
      setCounts((prev) => ({ ...prev, [kind]: { loading: false, data, failure: null } }))
    } catch (error: unknown) {
      setCounts((prev) => ({ ...prev, [kind]: { loading: false, data: null, failure: failureOf(error) } }))
    }
  }

  function renderCountOutput(kind: CountKind, data: unknown) {
    switch (kind) {
      case 'root-frequency':
        return <FrequencyOutput label="root frequency" data={data} />
      case 'lemma-frequency':
        return <FrequencyOutput label="lemma frequency" data={data} />
      case 'frequency':
        return <FrequencyOutput label="token frequency" data={data} />
      case 'distribution':
        return <DistributionOutput data={data} />
      case 'cooccurrence':
        return <CooccurrenceOutput data={data} />
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

      <div className="mt-10 max-w-2xl border-t pt-6">
        <p className="kicker mb-2">Counts</p>
        <p className="margin-note">Every count prints beside the rules that produced it.</p>

        <div className="mt-3 grid gap-3 sm:grid-cols-[minmax(0,1fr)_10rem_6rem]">
          <div>
            <label htmlFor="words-count-target" className="kicker mb-2 block">
              Count target
            </label>
            <input
              id="words-count-target"
              value={countTarget}
              onChange={(event) => setTargetOverride(event.target.value)}
              dir="auto"
              spellCheck={false}
              placeholder="a surface form"
              className="field h-9 font-mono text-xs"
            />
          </div>
          <div>
            <label htmlFor="words-profile" className="kicker mb-2 block">
              Profile
            </label>
            <input
              id="words-profile"
              value={profileInput}
              onChange={(event) => setProfileInput(event.target.value)}
              dir="auto"
              spellCheck={false}
              className="field h-9 font-mono text-xs"
            />
          </div>
          <div>
            <label htmlFor="words-window" className="kicker mb-2 block">
              Window
            </label>
            <input
              id="words-window"
              value={windowInput}
              onChange={(event) => setWindowInput(event.target.value)}
              dir="auto"
              spellCheck={false}
              inputMode="numeric"
              className="field h-9 font-mono text-xs"
            />
          </div>
        </div>

        {COUNT_INSTRUMENTS.map(({ id, label }) => {
          const state = counts[id]
          const reason = countReason(id)
          return (
            <div key={id} className="mt-4 border-t pt-4">
              <div className="flex flex-wrap items-center gap-3">
                <button
                  type="button"
                  className="btn btn-primary h-9 justify-center px-4"
                  onClick={() => void runCount(id)}
                  disabled={reason !== null || state.loading}
                  title={reason ?? undefined}
                >
                  {state.loading ? 'consulting…' : `Run ${label}`}
                </button>
              </div>
              {reason !== null && !state.loading && <p className="margin-note mt-1">{reason}</p>}
              {state.loading && (
                <div className="mt-3">
                  <LoadingLine label={`consulting ${label}…`} />
                </div>
              )}
              {state.failure !== null && (
                <ErrorNote>
                  {state.failure.code !== undefined && (
                    <span className="font-mono text-[0.75rem]">{state.failure.code} — </span>
                  )}
                  {state.failure.summary}
                </ErrorNote>
              )}
              {state.data !== null && renderCountOutput(id, state.data)}
            </div>
          )
        })}
      </div>
    </section>
  )
}
