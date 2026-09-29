# Quran Search Cookbook

> **Audience:** operators and agents performing Quran searches.
> **Implementation:** `crates/quran-search/src/`
> **Governing ADR:** ADR-0207 (Proposed — implementation complete; acceptance
> pending owner review)
> **Status:** Code-implemented; owner acceptance pending

## 1. Search Modes

Q-ai provides five search modes. Each returns traced, spanned, cited hits
with a `NormalizationTrace` explaining the transformation.

| Mode | Tool Name | Profile | Description |
|------|-----------|---------|-------------|
| Exact | `quran.search_exact` | L0 | Exact canonical match |
| Normalized | `quran.search_normalized` | L3 (default) | Diacritic-free match |
| Phrase | `quran.search_phrase` | L3 | Ordered/near/unordered phrase |
| Concatenated | `quran.search_concatenated` | L6 | Spaceless (skeleton) match |
| Regex | `quran.search_regex` | L3 | DFA-safe regex |

## 2. Worked Examples

### 2.1 Exact Search

```bash
qai quran search exact --query "بِسْمِ اللَّهِ الرَّحْمَٰنِ الرَّحِيمِ"
```

**Returns:** Hits where the canonical text exactly matches the query,
including all diacritics. Zero results if the query has any normalization
differences from the canonical text.

**Zero-hit meaning:** The exact string does not exist in the canonical text.
Consider using normalized search instead.

### 2.2 Normalized Search

```bash
qai quran search normalized --query "بسم الله الرحمن الرحيم"
```

**Returns:** Hits where the L3-normalized canonical text matches the
L3-normalized query. Diacritics, tatweel, and Quranic marks are ignored.

**NormalizationTrace:** Each hit carries an ordered trace showing which rules
fired and how the query was transformed.

**Zero-hit meaning:** The normalized form does not exist in the index. This
usually means the query contains a spelling variant not covered by L3 rules.

### 2.3 Phrase Search

```bash
# Ordered phrase (default)
qai quran search phrase --query "الرحمن الرحIM"

# Unordered (near) with slop
qai quran search phrase --query "الرحمن الرحIM" --unordered --slop 3
```

**Returns:** Hits where the phrase appears in the canonical text. Ordered
mode enforces word order; unordered mode allows any order within the slop
window.

### 2.4 Concatenated (Spaceless) Search

```bash
qai quran search concatenated --query "سانبل"
```

**Returns:** Hits where the L6 skeleton matches. The query is normalized
to L6 (all spaces and punctuation removed), then matched against the L6
skeleton index.

**Segmentation:** Each hit carries a `segmentation` field explaining how
the query was tiled across ayah boundaries. Cross-ayah matches are split
per overlapped ayah and labeled with `spans_ayah_boundary`.

**Zero-hit meaning:** The skeleton does not exist in the index. This is
expected for queries that span word boundaries in ways the skeleton
cannot represent.

### 2.5 Regex Search

```bash
qai quran search regex --query "بسم.*الله"
```

**Returns:** Hits matching the DFA-safe regex pattern. The regex is
applied to the normalized field, never to raw canonical text.

**Guards (I16):**
- DFA engine only (no backtracking)
- 1 MiB pattern size limit, 4 MiB result limit
- 512-character pattern length limit
- 3-second timeout
- No leading `.*`
- Per-principal rate limits

## 3. Result Structure

Every search returns a list of `SearchHit` objects:

```json
{
  "reference": "quran:slug@version:surah:ayah",
  "quotation": "بِسْمِ اللَّهِ الرَّحْمَٰنِ الرَّحِيمِ",
  "span": { "start": 0, "end": 30 },
  "score": 1.0,
  "normalization_trace": [
    { "rule": "N03", "input": "بِسْمِ", "output": "بسم" }
  ],
  "segmentation": null,
  "spans_ayah_boundary": false
}
```

**Key fields:**
- `quotation` — the exact canonical text at the hit location (never
  normalized text)
- `span` — character offsets into the canonical text
- `normalization_trace` — ordered list of rules that fired
- `segmentation` — concatenated-search tiling explanation (null for
  other modes)
- `spans_ayah_boundary` — true if the hit crosses an ayah boundary

## 4. Explaining a Hit

Use the `NormalizationTrace` to explain why a hit matched:

```bash
qai quran search normalized --query "بسم" --explain
```

The trace shows:
1. The original query
2. Each rule applied in order
3. The final normalized form
4. The canonical text that matched

## 5. Filters

All search modes support filters:

```bash
# Surah filter
qai quran search normalized --query "بسم" --surah 1

# Juz filter
qai quran search normalized --query "بسم" --juz 1

# Page filter
qai quran search normalized --query "بسم" --page 1

# Global range filter
qai quran search normalized --query "بسم" --range 1-100
```

## 6. Zero-Hit Results

A zero-hit result is a valid result, not an error. It means:

1. **Exact mode:** The exact string does not exist in the canonical text.
2. **Normalized mode:** The normalized form does not exist in the index.
3. **Phrase mode:** The phrase does not appear in the canonical text.
4. **Concatenated mode:** The skeleton does not exist in the index.
5. **Regex mode:** The pattern does not match any normalized field.

**Never** return all matches or a fallback result for a zero-hit query.
The empty result is the correct answer.

## 7. Highlighting

Search results include canonical character ranges for display markers:

```bash
qai quran search normalized --query "بسم" --highlight
```

The highlight markers are placed on the canonical text, not on the
normalized text.

## 8. Result Cache

Search results are cached with generation invalidation:

- Cache key: `(edition, version, corpus_generation, query, profile, filters)`
- Invalidation: wholesale on `corpus_generation` change
- LRU cap: configurable, default 1000 entries

## 9. Governing ADR

- **ADR-0207** (Proposed): Concatenated-Search Architecture (Skeleton +
  Trigram + Verify). Implementation complete; acceptance pending owner
  review.

**Important:** This ADR is Proposed. The concatenated search architecture
is code-implemented but not owner-ratified.

## 10. Related Documents

- `docs/07-technical/quran-normalization-spec.md` — rule catalog
- `docs/07-technical/quran-profile-catalog.md` — L0–L8 profile table
- `docs/02-architecture/decisions/ADR-0207-concatenated-search.md` — governing ADR
