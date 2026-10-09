// Typed `/api/v1` client (05-06): mirrors the server `Envelope<T>`/`Meta`
// contract. Used by the reading view now; resolve/citation/tool methods
// serve plan 05-08.

export interface ResearchChecksum {
  algorithm: string
  hex: string
}

export interface Meta {
  edition: { slug: string; version: string }
  corpus_generation: number
  canonical_reference: string
  deep_link: string
  execution_time_ms: number
  reproducibility: unknown
  research_checksum: string
  warnings: string[]
}

export interface Envelope<T> {
  api_version: string
  data: T
  meta: Meta
}

export interface CanonicalText {
  reference: string
  arabicText: string
}

export interface TranslationText {
  translator: string
  text: string
  language?: string
}

export interface AyahView {
  canonical: CanonicalText
  translations: TranslationText[]
}

export interface ApiError {
  error: { code: string; summary: string; remedy?: string; next_command?: string }
}

async function get<T>(path: string): Promise<Envelope<T>> {
  const response = await fetch(path, { headers: { accept: 'application/json' } })
  if (!response.ok) {
    const body = (await response.json().catch(() => null)) as ApiError | null
    throw new Error(body?.error?.summary ?? `GET ${path}: ${response.status}`)
  }
  return (await response.json()) as Envelope<T>
}

async function post<T>(path: string, params: unknown): Promise<Envelope<T>> {
  const response = await fetch(path, {
    method: 'POST',
    headers: { 'content-type': 'application/json', accept: 'application/json' },
    body: JSON.stringify(params),
  })
  if (!response.ok) {
    const body = (await response.json().catch(() => null)) as ApiError | null
    throw new Error(body?.error?.summary ?? `POST ${path}: ${response.status}`)
  }
  return (await response.json()) as Envelope<T>
}

/** Fetch ayah views for a reference (`2:1`, `2`, …). */
export function fetchAyahs(reference: string): Promise<Envelope<AyahView[]>> {
  return get<AyahView[]>(`/api/v1/quran/ayahs/${encodeURIComponent(reference)}`)
}

/** Resolve a reference string through the canonical resolver. */
export function resolveReference(reference: string): Promise<Envelope<unknown>> {
  return get<unknown>(`/api/v1/quran/resolve?reference=${encodeURIComponent(reference)}`)
}

/** Re-verify a stored citation by id (plan 05-08 deep links). */
export function fetchCitation(id: string): Promise<Envelope<unknown>> {
  return get<unknown>(`/api/v1/quran/citations/${encodeURIComponent(id)}`)
}

/** Run any registered tool by canonical name with JSON params. */
export function runTool(name: string, params: unknown): Promise<Envelope<unknown>> {
  return post<unknown>(`/api/v1/quran/tool/${encodeURIComponent(name)}`, params)
}
