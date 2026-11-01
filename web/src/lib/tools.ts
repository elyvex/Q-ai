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
]

const DURABLE_STATE_REASON =
  'This tool writes durable state, so the browser never calls it; the trusted operator surface is the local CLI. ' +
  'The HTTP route enables it only when the operator sets policy.tool_execution_default = "allow".'

export const DENIED: DeniedTool[] = [
  { name: 'rag.create_project', code: 'QAI-RAG-0326', reason: DURABLE_STATE_REASON },
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
