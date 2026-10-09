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

// ─── Research methods (05-09): one typed method per surface route ───

export type SearchMode = 'exact' | 'normalized' | 'phrase' | 'concatenated' | 'regex'

/** POST /api/v1/quran/search/{mode} — lexical search over the serving index. */
export function search(mode: SearchMode, query: string): Promise<Envelope<unknown>> {
  return post<unknown>(`/api/v1/quran/search/${mode}`, { text: query })
}

/** POST /api/v1/quran/family — word-family relations for one member. */
export function family(kind: string, id: string): Promise<Envelope<unknown>> {
  return post<unknown>('/api/v1/quran/family', { kind, id })
}

/** POST /api/v1/quran/count/root-frequency — exact rules-blocked root count. */
export function rootFrequency(root: string): Promise<Envelope<unknown>> {
  return post<unknown>('/api/v1/quran/count/root-frequency', { root })
}

/** POST /api/v1/quran/count/lemma-frequency — the lemma analogue. */
export function lemmaFrequency(lemma: string): Promise<Envelope<unknown>> {
  return post<unknown>('/api/v1/quran/count/lemma-frequency', { lemma })
}

/** POST /api/v1/quran/count/frequency — exact token frequency under a profile. */
export function frequency(target: string, profile: string): Promise<Envelope<unknown>> {
  return post<unknown>('/api/v1/quran/count/frequency', { target, profile })
}

/** POST /api/v1/quran/count/distribution — frequency partitioned by surah. */
export function distribution(target: string, profile: string): Promise<Envelope<unknown>> {
  return post<unknown>('/api/v1/quran/count/distribution', { target, profile })
}

/** POST /api/v1/quran/count/cooccurrence — windowed co-occurrence. */
export function cooccurrence(
  target: string,
  profile: string,
  window: number,
): Promise<Envelope<unknown>> {
  return post<unknown>('/api/v1/quran/count/cooccurrence', { target, profile, window })
}

// ─── Graph methods (05-09): neighbors + paths over the frozen payload ───

/** POST /api/v1/quran/graph/neighbors — bounded neighbors with provenance. */
export function graphNeighbors(node: string): Promise<Envelope<unknown>> {
  return post<unknown>('/api/v1/quran/graph/neighbors', { node })
}

/** POST /api/v1/quran/graph/path — reachability/shortest/paths between nodes. */
export function graphPath(from: string, to: string): Promise<Envelope<unknown>> {
  return post<unknown>('/api/v1/quran/graph/path', { from, to })
}
