// Shared `ToolResult` contract: mirrors what `crates/tools/src/lib.rs`
// serializes — `tool_version` (a domain `SemVer`) arrives as a
// `MAJOR.MINOR.PATCH` string, never an object. The only place a tool
// result is turned into rows.
export interface ContentHashValue {
  algorithm: string
  hex: string
}

export interface AnalysisSourceValue {
  kind: string
  reference: string
}

export interface ReproducibilityValue {
  checksum: ContentHashValue
  query_hash: ContentHashValue
  tool_plan: string[]
  source_versions: Record<string, string>
  edition_versions: Record<string, string>
}

export interface ToolResultData {
  tool_name: string
  tool_version: string
  query: unknown
  normalization_rules: string[]
  edition_id: string | null
  edition_version: string | null
  results: unknown
  canonical_references: string[]
  analysis_sources: AnalysisSourceValue[]
  confidence: unknown
  warnings: string[]
  execution_time_ms: number
  reproducibility: ReproducibilityValue
  research_checksum: ContentHashValue
}

/** `Sha256` + hex → `sha256:<hex>`; nullish in, empty string out. */
export function checksumLabel(hash: ContentHashValue | null | undefined): string {
  if (hash == null) return ''
  return `${hash.algorithm.toLowerCase()}:${hash.hex}`
}

/**
 * The array when `results` is an array of plain objects, otherwise an
 * empty array. Never throws on a string, number, null, or nested object.
 */
export function rowsOf(results: unknown): Record<string, unknown>[] {
  if (!Array.isArray(results)) return []
  return results.filter(
    (row): row is Record<string, unknown> =>
      typeof row === 'object' && row !== null && !Array.isArray(row),
  )
}

const QURAN_REF = /^quran:([^@/:]+)@([^/:]+):(\d+:\d+)$/

/**
 * A pinned `quran:<slug>@<version>:<surah>:<ayah>` reference becomes its
 * deep link (`/read/<slug>@<version>/<surah>:<ayah>`); anything else is
 * not linkable, so null. This is a URL — Latin stays unmirrored.
 */
export function deepLinkFor(canonicalReference: string): string | null {
  const match = QURAN_REF.exec(canonicalReference)
  if (!match) return null
  return `/read/${match[1]}@${match[2]}/${match[3]}`
}

/**
 * Look the caller-supplied slug up in `edition_versions`. A result that
 * carries more than one edition has no single answer, so a caller with
 * no explicit slug renders no deep link — the first entry is never
 * good enough.
 */
export function editionOf(
  reproducibility: { edition_versions: Record<string, string> },
  slug: string,
): { slug: string; version: string } | null {
  const version = reproducibility.edition_versions[slug]
  if (version === undefined) return null
  return { slug, version }
}
