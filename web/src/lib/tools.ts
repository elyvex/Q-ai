// Catalogue of every tool the generic dispatcher serves
// (`POST /api/v1/quran/tool/{name}` through the same registry the CLI
// `quran tool` verb uses), plus the three derivations the surface control
// prints: the parameter object, the equivalent CLI command, and the
// equivalent HTTP request. Forms are generated from `fields`, so a
// parameter outside the catalogue can never be entered.
export type ToolFieldKind = 'text' | 'number' | 'json' | 'bool'

export interface ToolField {
  key: string
  label: string
  kind: ToolFieldKind
  required: boolean
  placeholder?: string
  hint?: string
}

export type ToolDomain = 'quran' | 'graph' | 'hadith' | 'tafsir' | 'scripture' | 'isnad' | 'rag'

export interface ToolSpec {
  name: string
  label: string
  domain: ToolDomain
  summary: string
  fields: ToolField[]
}

/** A tool the HTTP route refuses by default, with the policy code it carries. */
export interface DeniedTool {
  name: string
  code: string
  reason: string
}

/** A typed parameter failure naming the offending field — never a bare `Error`. */
export class ToolParamError extends Error {
  readonly field: string

  constructor(field: string, message: string) {
    super(message)
    this.name = 'ToolParamError'
    this.field = field
  }
}

export const DOMAIN_ORDER: readonly ToolDomain[] = [
  'quran',
  'graph',
  'hadith',
  'tafsir',
  'scripture',
  'isnad',
  'rag',
] as const

export const TOOLS: ToolSpec[] = [
  {
    name: 'quran.get_ayah',
    label: 'Get ayah',
    domain: 'quran',
    summary: 'Fetch one canonical ayah with optional translations, glosses and tokens.',
    fields: [
      {
        key: 'reference',
        label: 'Reference',
        kind: 'text',
        required: true,
        placeholder: '2:1',
        hint: 'surah:ayah, for example 2:1',
      },
      {
        key: 'translations',
        label: 'Translations',
        kind: 'json',
        required: false,
        placeholder: '["en-sahih"]',
        hint: 'array of translation slugs',
      },
      { key: 'glosses', label: 'Glosses', kind: 'bool', required: false },
      { key: 'tokens', label: 'Tokens', kind: 'bool', required: false },
    ],
  },
  {
    name: 'quran.get_context',
    label: 'Get context',
    domain: 'quran',
    summary: 'Fetch the ayahs around a focal reference inside a structural boundary.',
    fields: [
      {
        key: 'reference',
        label: 'Reference',
        kind: 'text',
        required: true,
        placeholder: '2:1',
        hint: 'focal ayah-level reference',
      },
      {
        key: 'before',
        label: 'Before',
        kind: 'number',
        required: false,
        placeholder: '3',
        hint: 'ayahs before the focal reference (default 3)',
      },
      {
        key: 'after',
        label: 'After',
        kind: 'number',
        required: false,
        placeholder: '3',
        hint: 'ayahs after the focal reference (default 3)',
      },
      {
        key: 'max_ayahs',
        label: 'Max ayahs',
        kind: 'number',
        required: false,
        placeholder: '11',
        hint: 'hard cap on ayahs returned (default 11)',
      },
      {
        key: 'boundary',
        label: 'Boundary',
        kind: 'text',
        required: false,
        placeholder: 'none',
        hint: 'surah | juz | ruku | page | none',
      },
    ],
  },
  {
    name: 'quran.search',
    label: 'Search Quran',
    domain: 'quran',
    summary: 'Lexical search over the serving index; never an empty fabrication.',
    fields: [
      { key: 'text', label: 'Text', kind: 'text', required: true, placeholder: 'mercy' },
      {
        key: 'edition',
        label: 'Edition',
        kind: 'text',
        required: false,
        placeholder: 'test@0.1.0',
        hint: 'slug@version (defaults to the indexed edition)',
      },
      {
        key: 'limit',
        label: 'Limit',
        kind: 'number',
        required: false,
        placeholder: '20',
        hint: 'result cap (defaults to 20)',
      },
    ],
  },
  {
    name: 'quran.root',
    label: 'Root occurrences',
    domain: 'quran',
    summary: 'Group occurrences by a triliteral root surface.',
    fields: [{ key: 'root', label: 'Root', kind: 'text', required: true, placeholder: 'ktb' }],
  },
  {
    name: 'quran.lemma',
    label: 'Lemma occurrences',
    domain: 'quran',
    summary: 'Group occurrences by a lemma surface.',
    fields: [{ key: 'lemma', label: 'Lemma', kind: 'text', required: true }],
  },
  {
    name: 'quran.morphology',
    label: 'Morphology',
    domain: 'quran',
    summary: 'Analyse one token addressed by surah, ayah and position.',
    fields: [
      { key: 'surah', label: 'Surah', kind: 'number', required: true, placeholder: '2' },
      { key: 'ayah', label: 'Ayah', kind: 'number', required: true, placeholder: '1' },
      { key: 'position', label: 'Position', kind: 'number', required: true, placeholder: '3' },
    ],
  },
  {
    name: 'quran.family',
    label: 'Word family',
    domain: 'quran',
    summary: 'List the word-family relations for one member.',
    fields: [
      { key: 'kind', label: 'Kind', kind: 'text', required: true, placeholder: 'root' },
      { key: 'id', label: 'Id', kind: 'text', required: true, placeholder: 'ktb' },
    ],
  },
  {
    name: 'quran.graph_neighbors',
    label: 'Graph neighbors',
    domain: 'graph',
    summary: 'Open the bounded neighborhood around one stable node id.',
    fields: [
      {
        key: 'node',
        label: 'Node',
        kind: 'text',
        required: true,
        placeholder: 'quran:2:1',
        hint: 'stable node id to open around',
      },
      {
        key: 'projection',
        label: 'Projection',
        kind: 'text',
        required: false,
        placeholder: 'quran-structural-v1',
        hint: 'projection family to read (default quran-structural-v1)',
      },
      {
        key: 'direction',
        label: 'Direction',
        kind: 'text',
        required: false,
        placeholder: 'both',
        hint: 'both | outgoing | incoming',
      },
    ],
  },
  {
    name: 'quran.graph_path',
    label: 'Graph path',
    domain: 'graph',
    summary: 'Find reachability, the shortest path, or several paths between two nodes.',
    fields: [
      { key: 'from', label: 'From', kind: 'text', required: true, placeholder: 'quran:2:1' },
      { key: 'to', label: 'To', kind: 'text', required: true, placeholder: 'quran:2:5' },
      {
        key: 'mode',
        label: 'Mode',
        kind: 'text',
        required: false,
        placeholder: 'paths',
        hint: 'reachability | shortest | paths',
      },
      {
        key: 'paths',
        label: 'Paths',
        kind: 'number',
        required: false,
        placeholder: '5',
        hint: 'requested path count for paths mode',
      },
    ],
  },
  {
    name: 'quran.graph_pattern',
    label: 'Graph pattern',
    domain: 'graph',
    summary: 'Match an ordered typed edge pattern out from seed nodes.',
    fields: [
      {
        key: 'seeds',
        label: 'Seeds',
        kind: 'json',
        required: true,
        placeholder: '["quran:2:1"]',
        hint: 'seed stable ids, for example ["quran:2:1"]',
      },
      {
        key: 'steps',
        label: 'Steps',
        kind: 'json',
        required: true,
        placeholder: '[{"edge":"contains","direction":"out"}]',
        hint: 'ordered typed steps, for example [{"edge":"…","direction":"out"}]',
      },
    ],
  },
  {
    name: 'quran.graph_subgraph',
    label: 'Graph subgraph',
    domain: 'graph',
    summary: 'Collect the subgraph reachable from seed nodes.',
    fields: [
      {
        key: 'seeds',
        label: 'Seeds',
        kind: 'json',
        required: true,
        placeholder: '["quran:2:1"]',
        hint: 'seed stable ids, for example ["quran:2:1"]',
      },
      {
        key: 'direction',
        label: 'Direction',
        kind: 'text',
        required: false,
        placeholder: 'both',
        hint: 'both | outgoing | incoming',
      },
    ],
  },
  {
    name: 'quran.graph_root_family',
    label: 'Graph root family',
    domain: 'graph',
    summary: 'Collect the graph family of one normalized root spelling.',
    fields: [
      { key: 'root', label: 'Root', kind: 'text', required: true, placeholder: 'ktb' },
      {
        key: 'limit',
        label: 'Limit',
        kind: 'number',
        required: false,
        placeholder: '25',
        hint: 'result cap (default 25)',
      },
    ],
  },
  {
    name: 'hadith.get',
    label: 'Get hadith',
    domain: 'hadith',
    summary: 'Fetch one hadith by collection and number.',
    fields: [
      {
        key: 'reference',
        label: 'Reference',
        kind: 'text',
        required: true,
        placeholder: 'bukhari:1',
        hint: 'collection:number',
      },
    ],
  },
  {
    name: 'hadith.search',
    label: 'Search hadith',
    domain: 'hadith',
    summary: 'Search hadith matn across collections.',
    fields: [
      { key: 'query', label: 'Query', kind: 'text', required: true, placeholder: 'mercy' },
      {
        key: 'collection_slug',
        label: 'Collection',
        kind: 'text',
        required: false,
        hint: 'restrict to one collection slug',
      },
      {
        key: 'limit',
        label: 'Limit',
        kind: 'number',
        required: false,
        placeholder: '5',
        hint: 'result cap',
      },
    ],
  },
  {
    name: 'hadith.gradings',
    label: 'Hadith gradings',
    domain: 'hadith',
    summary: 'List the recorded gradings for one hadith id.',
    fields: [{ key: 'hadith_id', label: 'Hadith id', kind: 'text', required: true }],
  },
  {
    name: 'tafsir.get',
    label: 'Get tafsir',
    domain: 'tafsir',
    summary: 'Fetch a tafsir passage by work and reference.',
    fields: [
      {
        key: 'reference',
        label: 'Reference',
        kind: 'text',
        required: true,
        placeholder: 'tabari:2:1',
        hint: 'work:reference',
      },
    ],
  },
  {
    name: 'tafsir.search',
    label: 'Search tafsir',
    domain: 'tafsir',
    summary: 'Search tafsir text within or across works.',
    fields: [
      { key: 'query', label: 'Query', kind: 'text', required: true, placeholder: 'mercy' },
      {
        key: 'work_slug',
        label: 'Work',
        kind: 'text',
        required: false,
        hint: 'restrict to one work slug',
      },
      {
        key: 'limit',
        label: 'Limit',
        kind: 'number',
        required: false,
        placeholder: '5',
        hint: 'result cap',
      },
    ],
  },
  {
    name: 'scripture.editions',
    label: 'Scripture editions',
    domain: 'scripture',
    summary: 'List the scripture editions this backend serves.',
    fields: [],
  },
  {
    name: 'scripture.get',
    label: 'Get scripture',
    domain: 'scripture',
    summary: 'Fetch one scripture view by reference.',
    fields: [
      { key: 'reference', label: 'Reference', kind: 'text', required: true, placeholder: 'gen:1:1' },
    ],
  },
  {
    name: 'scripture.search',
    label: 'Search scripture',
    domain: 'scripture',
    summary: 'Search scripture text within an edition.',
    fields: [
      { key: 'query', label: 'Query', kind: 'text', required: true, placeholder: 'genesis' },
      {
        key: 'edition',
        label: 'Edition',
        kind: 'text',
        required: false,
        hint: 'edition slug (defaults to the served edition)',
      },
      {
        key: 'limit',
        label: 'Limit',
        kind: 'number',
        required: false,
        placeholder: '5',
        hint: 'result cap',
      },
    ],
  },
  {
    name: 'scripture.translations',
    label: 'Scripture translations',
    domain: 'scripture',
    summary: 'Fetch the translations of one scripture verse.',
    fields: [
      { key: 'verse_id', label: 'Verse id', kind: 'text', required: true },
      {
        key: 'language',
        label: 'Language',
        kind: 'text',
        required: false,
        hint: 'restrict to one language',
      },
    ],
  },
  {
    name: 'scripture.parallel_passages',
    label: 'Parallel passages',
    domain: 'scripture',
    summary: 'List the parallel passages aligned to one reference.',
    fields: [{ key: 'reference', label: 'Reference', kind: 'text', required: true }],
  },
  {
    name: 'isnad.trace',
    label: 'Trace isnad',
    domain: 'isnad',
    summary: 'Trace transmission paths between two narrators.',
    fields: [
      { key: 'from', label: 'From', kind: 'text', required: true },
      { key: 'to', label: 'To', kind: 'text', required: true },
      {
        key: 'k',
        label: 'K',
        kind: 'number',
        required: false,
        hint: 'maximum paths to return',
      },
    ],
  },
  {
    name: 'isnad.narrator',
    label: 'Get narrator',
    domain: 'isnad',
    summary: 'Fetch one narrator record by id.',
    fields: [{ key: 'id', label: 'Id', kind: 'text', required: true }],
  },
  {
    name: 'isnad.rijal',
    label: 'Rijal entry',
    domain: 'isnad',
    summary: 'Fetch the rijal entry for one narrator id.',
    fields: [{ key: 'id', label: 'Id', kind: 'text', required: true }],
  },
  {
    name: 'rag.query',
    label: 'RAG query',
    domain: 'rag',
    summary: 'Run read-only retrieval over one project.',
    fields: [
      { key: 'project', label: 'Project', kind: 'text', required: true, placeholder: 'default' },
      { key: 'query', label: 'Query', kind: 'text', required: true, placeholder: 'mercy' },
    ],
  },
  {
    name: 'rag.debug',
    label: 'RAG debug',
    domain: 'rag',
    summary: 'Run read-only retrieval with the route decision over one project.',
    fields: [
      { key: 'project', label: 'Project', kind: 'text', required: true, placeholder: 'default' },
      { key: 'query', label: 'Query', kind: 'text', required: true, placeholder: 'mercy' },
    ],
  },
]

const DURABLE_STATE_REASON =
  'This tool writes durable state, so the browser never calls it; the trusted operator surface is the local CLI. ' +
  'The HTTP route enables it only when the operator sets policy.tool_execution_default = "allow".'

export const DENIED: DeniedTool[] = [
  { name: 'rag.create_project', code: 'QAI-RAG-0326', reason: DURABLE_STATE_REASON },
  { name: 'rag.create', code: 'QAI-RAG-0326', reason: DURABLE_STATE_REASON },
  { name: 'rag.import', code: 'QAI-RAG-0326', reason: DURABLE_STATE_REASON },
  { name: 'rag.source_status', code: 'QAI-RAG-0326', reason: DURABLE_STATE_REASON },
  { name: 'rag.source_activate', code: 'QAI-RAG-0326', reason: DURABLE_STATE_REASON },
  { name: 'rag.source_deactivate', code: 'QAI-RAG-0326', reason: DURABLE_STATE_REASON },
  { name: 'rag.source_tombstone', code: 'QAI-RAG-0326', reason: DURABLE_STATE_REASON },
  { name: 'isnad.identity_propose', code: 'QAI-ISNAD-0326', reason: DURABLE_STATE_REASON },
  { name: 'isnad.identity_accept', code: 'QAI-ISNAD-0326', reason: DURABLE_STATE_REASON },
  { name: 'isnad.identity_reject', code: 'QAI-ISNAD-0326', reason: DURABLE_STATE_REASON },
  { name: 'isnad.identity_dispute', code: 'QAI-ISNAD-0326', reason: DURABLE_STATE_REASON },
  { name: 'isnad.identity_supersede', code: 'QAI-ISNAD-0326', reason: DURABLE_STATE_REASON },
]

/**
 * Build the parameter object for `spec` from raw form strings. An empty
 * string skips an optional field (the server fills its default); a
 * required field is always included, throwing a `ToolParamError` naming
 * the field when it is missing. Numbers coerce with `Number()`, `json`
 * parses with `JSON.parse`, and `bool` maps `'true'`/`'false'` — each
 * rejecting with a typed per-field failure.
 */
export function paramsJson(
  spec: ToolSpec,
  values: Record<string, string>,
): Record<string, unknown> {
  const params: Record<string, unknown> = {}
  for (const field of spec.fields) {
    const raw = values[field.key] ?? ''
    if (raw === '') {
      if (field.required) throw new ToolParamError(field.key, `required: ${field.label}`)
      continue
    }
    switch (field.kind) {
      case 'number': {
        const parsed = Number(raw)
        if (!Number.isFinite(parsed)) {
          throw new ToolParamError(field.key, `${field.label} must be a number`)
        }
        params[field.key] = parsed
        break
      }
      case 'json': {
        try {
          params[field.key] = JSON.parse(raw) as unknown
        } catch (error: unknown) {
          throw new ToolParamError(
            field.key,
            error instanceof Error ? error.message : `${field.label} is not valid JSON`,
          )
        }
        break
      }
      case 'bool': {
        if (raw !== 'true' && raw !== 'false') {
          throw new ToolParamError(field.key, `${field.label} must be true or false`)
        }
        params[field.key] = raw === 'true'
        break
      }
      default: {
        params[field.key] = raw
      }
    }
  }
  return params
}

/**
 * The equivalent CLI invocation for an exact parameter object. Embedded
 * single quotes escape as the shell sequence quote-backslash-quote-quote,
 * so the printed line pastes verbatim. The `--json` flag is global on the
 * CLI. The line is displayed and copied, never executed here.
 */
export function cliCommand(name: string, params: Record<string, unknown>): string {
  const json = JSON.stringify(params).replace(/'/g, "'\\''")
  return `qai quran tool ${name} --params '${json}' --json`
}

/** The equivalent HTTP request for an exact parameter object. */
export function httpRequest(
  name: string,
  params: Record<string, unknown>,
): { method: 'POST'; path: string; body: string } {
  return {
    method: 'POST',
    path: `/api/v1/quran/tool/${name}`,
    body: JSON.stringify(params, null, 2),
  }
}
