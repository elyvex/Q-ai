# Quran Counting Rules

> **Audience:** operators and agents performing frequency, distribution,
> and co-occurrence analysis.
> **Implementation:** `crates/application/src/quran_counting.rs`
> **Governing ADR:** ADR-0211 (Draft — pending linguist inputs)
> **Status:** Code-implemented; linguistic review pending (OD-12)

## 1. Purpose

Counting tools provide exact numeric analysis of Quranic text. Every numeric
output carries a complete `CountingRules` block so results are reproducible
and auditable.

**Anti-numerology discipline:** Counts are rule-relative. The `CountingRules`
block states exactly which rules produced the number, so a reader can
reproduce or challenge the count.

## 2. Multi-Analysis Modes

When a token has multiple competing analyses (e.g., multiple possible roots),
the counting mode determines how it is counted:

| Mode | Behavior | Use When |
|------|----------|----------|
| **SingleSource** | Count a token only when exactly one analysis matches the target. Exclude tokens with competing matching analyses. | Conservative counting; excludes ambiguity |
| **AllAnalyses** | Count every matching analysis as one occurrence. | Inclusive counting; each analysis is a vote |
| **OneVotePerToken** | Count every matching token once, regardless of how many analyses match. | Token-level counting; each token is a vote |

**Default:** `SingleSource` (most conservative).

**Exclusions:** In `SingleSource` mode, excluded tokens are recorded in
`CountingRules::exclusions`. Suppression is never silent.

## 3. CountingRules Block

Every numeric report carries a `CountingRules` block:

```json
{
  "profile": "L3.diacritics",
  "profile_version": "1.0.0",
  "datasets": ["stored-forms"],
  "multi_analysis_handling": "SingleSource",
  "window": null,
  "exclusions": []
}
```

**Fields:**
- `profile` — normalization profile id (e.g., `L3.diacritics`)
- `profile_version` — profile ladder version
- `datasets` — dataset slugs consulted (`["stored-forms"]` until a lexicon
  activates)
- `multi_analysis_handling` — one of `SingleSource`, `AllAnalyses`,
  `OneVotePerToken`
- `window` — co-occurrence window definition (e.g., `"token:5"` or
  `"ayah:1"`)
- `exclusions` — stop lists or exclusion rules applied (empty in v1)

**Canonical JSON:** `CountingRules::canonical_json()` produces the
checksum input. Field order is fixed by the struct.

## 4. Counting Services

### 4.1 Frequency

```bash
qai quran count frequency --target "بسم" --profile L3.diacritics
```

Returns the exact count of the target in the canonical text, normalized
to the specified profile.

### 4.2 Distribution

```bash
qai quran count distribution --target "بسم" --profile L3.diacritics
```

Returns per-surah counts with partition provenance and disagreement
warnings.

### 4.3 Co-occurrence

```bash
qai quran count cooccurrence --target "بسم" --context "الله" --window token:5
```

Returns co-occurrence counts within the specified window. Cross-ayah
flags indicate when co-occurrence spans an ayah boundary.

### 4.4 Collocation

```bash
qai quran count collocation --target "بسم" --context "الله" --min-count 2
```

Returns PMI, LLR, and t-score collocation scores with a minimum-count
floor.

### 4.5 First/Last Occurrence

```bash
qai quran count first-last-occurrence --target "بسم"
```

Returns the first and last occurrence positions of the target.

### 4.6 Interval Analysis

```bash
qai quran count interval-analysis --target "بسم"
```

Returns interval statistics between occurrences. Carries a mandatory
disclaimer verbatim.

### 4.7 Numeric Report

```bash
qai quran count numeric-report --target "بسم" --profile L3.diacritics
```

Returns a fixed-shape report with checksum and no interpretive commentary.

### 4.8 Root/Lemma Frequency

```bash
qai quran count root-frequency --root "بسم" --mode AllAnalyses
qai quran count lemma-frequency --lemma "بسم" --mode SingleSource
```

Returns exact SQL aggregation over the active lexicon's analyses. With
no active dataset, returns typed `CountingError::UnavailableDataset`
(`QAI-CNT-0005`) — never an empty report.

## 5. CountingError::UnavailableDataset

When no morphology dataset is active, root/lemma frequency returns:

```json
{
  "code": "QAI-CNT-0005",
  "message": "dataset unavailable for root frequency: import and activate a morphology dataset first"
}
```

This is a typed error, not an empty report. The capability is unavailable
until a dataset is imported and activated.

## 6. Determinism

Counting is deterministic: the same rules always produce the same number.
This is asserted by:
- `counting.rs` — `multi_analysis_modes` test
- `alpha_e2e.rs` — `assert_rules_complete` on every numeric report

## 7. Governing ADR

- **ADR-0211** (Draft): Counting Rules, Multi-Analysis Semantics,
  Numeric-Report Policy. Pending linguist inputs (OD-12).

**Important:** This ADR is Draft. The counting semantics are
code-implemented but not linguist-ratified. The `SingleSource`,
`AllAnalyses`, and `OneVotePerToken` modes are asserted behaviorally on
the synthetic lexicon only. Do not present them as settled linguistic
conventions.

## 8. Related Documents

- `docs/07-technical/quran-normalization-spec.md` — rule catalog
- `docs/07-technical/quran-profile-catalog.md` — L0–L8 profile table
- `docs/02-architecture/decisions/ADR-0211-counting-rules.md` — governing ADR
