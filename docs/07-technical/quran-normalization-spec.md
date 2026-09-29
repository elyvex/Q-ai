# Quran Normalization Specification

> **Audience:** operators, agents, and developers who need to understand how
> Quranic text is normalized for search and analysis.
> **Implementation:** `crates/quran-normalization/src/`
> **Governing ADR:** ADR-0204 (Draft — pending linguist review), ADR-0205
> (Draft — pending linguist review)
> **Status:** Code-implemented; linguistic review pending (OD-12)

## 1. Purpose

Normalization transforms canonical Quranic text into derived forms for search
and analysis. The canonical text is **never modified** — normalization produces
separate derived forms that map back to the canonical text through a `SpanMap`.

This separation is the core guarantee: displayed canonical text is always the
exact canonical text, never a normalized variant.

## 2. Architecture

The normalization pipeline is a sequence of ordered rules applied to canonical
text. Each rule is a pure, total, order-significant character transformation
that emits its own `SpanMap`. The pipeline composes these maps so every derived
character traces back to its canonical source.

```
Canonical Text → [Rule N01] → [Rule N02] → ... → [Rule Nn] → Derived Text
                   ↓            ↓                ↓
                SpanMap      SpanMap          SpanMap
                   └──────────┴────────────────┘
                              ↓
                    Composed SpanMap
                    (derived → canonical)
```

**Key files:**
- `crates/quran-normalization/src/rules/mod.rs` — rule catalog and `transform`
- `crates/quran-normalization/src/span.rs` — `SpanMap` implementation
- `crates/quran-normalization/src/profile.rs` — profile ladder L0–L8
- `crates/quran-normalization/src/pipeline.rs` — `NormalizationPipeline`

## 3. Rule Catalog

### 3.1 Deterministic Rules (N01–N17)

These rules are pure character transformations with published mapping tables.
They are always safe to apply and idempotent.

| ID | Name | Description | Mapping |
|----|------|-------------|---------|
| N01 | WhitespaceCollapse | Collapse whitespace runs to single space | `␣+` → `␣` |
| N02 | StripTatweel | Remove tatweel (kashida) | `ـ` → `` |
| N03 | StripHarakat | Remove diacritical marks | `ً-ْ` → `` |
| N04 | StripQuranicMarks | Remove Quranic annotation signs | `ۖ-ۛ` → `` |
| N05 | StripSuperscriptAlef | Remove superscript alef | `ٰ` → `` |
| N06 | NormalizeHamzaForms | Normalize hamza variants | `أإآٱ` → `ا` |
| N07 | NormalizeWasla | Normalize wasla | `ٱ` → `ا` |
| N08 | NormalizeAlifMaqsura | Normalize alif maqsura | `ى` → `ي` |
| N09 | NormalizeTaMarbuta | Normalize ta marbuta | `ة` → `ه` |
| N10 | NormalizePersianCodepoints | Fold Persian variants | `کیپ` → `كيپ` |
| N11 | StripZeroWidthAndBidi | Remove zero-width and bidi chars | `​-‏` → `` |
| N12 | StripPunctuation | Remove punctuation | `۔؟،؛` → `` |
| N13 | FoldDigits | Fold Arabic-Indic digits | `٠-٩` → `0-9` |
| N14 | StripPauseMarks | Remove pause marks | `۝` → `` |
| N15 | ExpandPresentationForms | Expand presentation forms | `ﷲ` → `الله` |
| N16 | NfcCompose | NFC composition | NFC normalize |
| N17 | RemoveSpaces | Remove all spaces | `␣` → `` |

### 3.2 Heuristic Rules (N18–N22)

These rules use linguistic heuristics and are tagged `RuleKind::Heuristic`.
They are **not** applied to canonical text — only to search queries and
derived forms. Results must render the heuristic-matched label.

| ID | Name | Description | Risk |
|----|------|-------------|------|
| N18 | StripDefiniteArticle | Strip `ال` prefix | May strip genuine morpheme |
| N19 | StripConjunctionPrefix | Strip `و` prefix | May strip genuine morpheme |
| N20 | StripPrepositionPrefix | Strip `ب`/`ك`/`ل` prefix | May strip genuine morpheme |
| N21 | StripPronounSuffix | Strip pronoun suffixes | May strip genuine morpheme |
| N22 | DedupeRepeatedLetters | Deduplicate repeated letters | May over-normalize |

### 3.3 Reserved Rules (N23–NN24)

Reserved for future use. `by_id()` returns `None` for these ids.

## 4. SpanMap Contract

The `SpanMap` is the mechanism that maps normalized offsets back to canonical
text. It is the guarantee that displayed canonical text is never modified.

**Properties:**
1. **Total:** Every derived character maps to at least one canonical character.
2. **Composable:** `to_canonical(to_derived(r)) ⊇ r` for any range `r`.
3. **Associative:** Composition of span maps is associative.
4. **L0 identity:** The identity profile (L0) has an empty rule list and an
   identity span map.
5. **Re-normalization containment:** Re-normalizing a normalized text produces
   a subset of the original normalization.

**Key file:** `crates/quran-normalization/src/span.rs`

## 5. SC5 Guarantee

**Displayed canonical text is never modified by normalization.**

This is enforced by:
1. The `SpanMap` contract above — every derived character traces back to
   canonical text.
2. The `SearchHit` validating constructor (`crates/quran-search/src/hit.rs`) —
   a hit cannot exist without a verified quotation + span + trace.
3. The per-hit test (`crates/application/tests/canonical_display_identity.rs`) —
   asserts byte-identity between displayed quotation and canonical row across
   all five search modes.

## 6. Profile Ladder

Profiles are ordered, versioned rule lists. See `quran-profile-catalog.md` for
the full L0–L8 table.

**Key file:** `crates/quran-normalization/src/profile.rs`

## 7. Idempotency and Safety

Every rule is idempotent: applying a rule twice produces the same result as
applying it once. This is asserted by the `check` function in
`crates/quran-normalization/src/rules/mod.rs` (test support module).

The fuzz battery (`crates/quran-normalization/tests/fuzz.rs`) asserts no panic
on any Unicode input.

## 8. CLI Surface

```bash
# Normalize text with a profile
qai quran normalize --profile L3.diacritics --text "بِسْمِ اللَّهِ"

# Explain the normalization (show trace)
qai quran normalize --profile L3.diacritics --text "بِسْمِ" --explain

# List all profiles
qai quran normalize --list-profiles

# Show a specific rule
qai quran normalize --show-rule N03
```

## 9. HTTP Surface

```
POST /api/v1/normalization/preview
GET /api/v1/normalization/profiles
```

## 10. Governing ADRs

- **ADR-0204** (Draft): Arabic Normalization Rule Catalog, Mapping Tables,
  Rule Ordering. Pending linguist review (OD-12).
- **ADR-0205** (Draft): Normalization Profile Ladder, Versioning, Immutability.
  Pending linguist review (OD-12).

**Important:** These ADRs are Draft. The rule catalog and profile ladder are
code-implemented but not linguist-ratified. Do not present them as settled
linguistic consensus.

## 11. Related Documents

- `docs/07-technical/quran-profile-catalog.md` — L0–L8 profile table
- `docs/07-technical/quran-search-cookbook.md` — search mode examples
- `docs/07-technical/quran-adapter-authoring.md` — adapter guide
- `docs/02-architecture/decisions/ADR-0204-normalization-rules.md` — governing ADR
- `docs/02-architecture/decisions/ADR-0205-profile-ladder.md` — governing ADR
