# Quran Normalization Profile Catalog

> **Audience:** operators and agents selecting a normalization profile for
> search or analysis.
> **Implementation:** `crates/quran-normalization/src/profile.rs`
> **Governing ADR:** ADR-0205 (Draft — pending linguist review)
> **Status:** Code-implemented; linguistic review pending (OD-12)

## 1. What Is a Profile?

A **profile** is an ordered, versioned rule list. Profiles — not individual
rules — are what users select, what indexes are built from, and what appears in
`reproducibility.normalization_rule_set`.

Profiles are **append-only**: changing a rule list requires a new version, a
full rebuild, and a drift report. Never edit a profile in place.

## 2. The L0–L8 Ladder

All profiles are at version `1.0.0`. Each rung extends the previous rule list
(cumulative), except L7 and L8 which branch from L5.

| Profile | ID String | Label | Rules | Indexed | Heuristic | Experimental |
|---------|-----------|-------|-------|---------|-----------|--------------|
| **L0** | `L0.exact` | Exact canonical | *(none)* | ✓ | — | — |
| **L1** | `L1.ws` | Exact after whitespace normalization | N01, N11, N16 | ✓ | — | — |
| **L2** | `L2.marks` | Ignore Quranic marks | L1 + N04, N14 | ✓ | — | — |
| **L3** | `L3.diacritics` | Ignore diacritics | L2 + N03, N05, N02 | ✓ | — | — |
| **L4** | `L4.hamza` | Normalize hamza/alif | L3 + N07, N06, N08 | ✓ | — | — |
| **L5** | `L5.codepoints` | Normalize Arabic/Persian code points | L4 + N10, N13, N09, N15 | ✓ | — | — |
| **L6** | `L6.skeleton` | Space-insensitive skeleton | L5 + N12, N17 | ✓ | — | — |
| **L7** | `L7.affix` | Morphological (heuristic affix) search | L5 + N18, N19, N20, N21 | ✓ | ✓ | — |
| **L8** | `L8.fuzzy` | Fuzzy spelling search | L5 | — | — | ✓ |

### 2.1 Rule Counts

| Profile | Rule Count | Cumulative |
|---------|-----------|------------|
| L0 | 0 | 0 |
| L1 | 3 | 3 |
| L2 | 5 | 5 |
| L3 | 8 | 8 |
| L4 | 11 | 11 |
| L5 | 15 | 15 |
| L6 | 17 | 17 |
| L7 | 19 | 15 + 4 heuristic |
| L8 | 15 | 15 (shares L5) |

## 3. Profile Selection Guide

### 3.1 For Exact Search

Use **L0** (`L0.exact`). No normalization is applied. The query must match
the canonical text exactly, including all diacritics and annotation marks.

### 3.2 For Normalized Search (Primary)

Use **L3** (`L3.diacritics`). This is the primary search field. It ignores
diacritics, tatweel, and Quranic marks while preserving the underlying
consonantal text. This is the profile most users should start with.

### 3.3 For Spaceless (Concatenated) Search

Use **L6** (`L6.skeleton`). This profile removes all spaces and punctuation,
producing a "skeleton" that supports concatenated-word search. The L6 path
uses a skeleton + trigram + exact verification strategy.

### 3.4 For Morphological Search

Use **L7** (`L7.affix`). This profile adds heuristic affix stripping
(N18–N21) for morphological analysis. Results must render the
heuristic-matched label (e.g., "Heuristic (pattern-based)").

### 3.5 For Fuzzy Search (Experimental)

Use **L8** (`L8.fuzzy`). This profile is experimental and off by default.
It uses query-time Levenshtein distance, not a rule. Not indexed.

## 4. Versioning Policy

Profiles are versioned with `SemVer`. The versioning policy is:

1. **Append-only:** A profile version, once registered, is immutable.
   Re-registering the same `(id, version)` with different rules is rejected
   with `NormalizationError::ProfileImmutable`.
2. **New version for changes:** To change a rule list, register a new version
   (e.g., `L3.diacritics@2.0.0`).
3. **Full rebuild required:** Changing a profile version requires a full
   index rebuild and a drift report.
4. **No silent fallback:** Callers must not silently fall back to another
   version. `get()` returns `UnknownProfile` for unregistered versions.

**Key file:** `crates/quran-normalization/src/profile.rs` — `ProfileRegistry`

## 5. Index Policy

Only profiles with `indexed: true` have an index field built from them. In v1:

- **Indexed:** L0, L1, L2, L3, L4, L5, L6, L7
- **Not indexed:** L8 (experimental, query-time only)

The index field names are stable API:
`text_exact`, `text_ws`, `text_marks`, `text_bare`, `text_hamza`,
`text_folded`, `text_affix`.

## 6. Heuristic Labeling

Profiles with `heuristic: true` (L7 only in v1) must render the
heuristic-matched label in results. This is enforced by the tool contract:
results from L7 carry a label like "Heuristic (pattern-based)" to distinguish
them from dataset-attested results.

## 7. Cost and Performance

| Profile | Relative Cost | Notes |
|---------|--------------|-------|
| L0 | Baseline | No transformation |
| L1–L3 | Low | Character deletions only |
| L4–L5 | Medium | Character replacements + expansions |
| L6 | Medium | Full skeletonization |
| L7 | Higher | Heuristic affix analysis |
| N8 | Highest | Query-time Levenshtein (not indexed) |

## 8. Governing ADR

- **ADR-0205** (Draft): Normalization Profile Ladder, Versioning, Immutability.
  Pending linguist review (OD-12).

**Important:** This ADR is Draft. The profile ladder is code-implemented but
not linguist-ratified. Do not present it as settled linguistic consensus.

## 9. Related Documents

- `docs/07-technical/quran-normalization-spec.md` — rule catalog and SpanMap
- `docs/07-technical/quran-search-cookbook.md` — search mode examples
- `docs/02-architecture/decisions/ADR-0205-profile-ladder.md` — governing ADR
