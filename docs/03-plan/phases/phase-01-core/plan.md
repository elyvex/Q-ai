# Q-ai — Phase 1 Development Plan: Canonical Quran Core

**Plan Version:** 1.0.0
**PRD Baseline:** Q-ai PRD v0.3.2
**Phase ID:** P1
**Phase Name:** Canonical Quran Corpus (structure, addressing, import, validation, reader)
**PRD Traceability:** §7, §12, §13.3, §19.1, §21.2, §22, §27.1, §34, §35.1, §35.3, §40, §43 (Phase 1), §46, §50, §55, §56, §88 (item 2)
**Depends On:** Phase 0 (all deliverables)
**Blocks:** Phase 2 (search/linguistics), Phase 3 (graph), Phase 4 (Web reader), Phase 5 (Quran↔hadith/tafsir links)
**Target Duration:** 6 calendar weeks (≈ 15–17 engineer-weeks, 3 engineers)
**Status:** Not Started

---

## 1. Phase Objective

Stand up the **canonical Quran engine**: a lossless, hash-verified, versioned, structurally
validated corpus with stable addressing and deterministic, LLM-free lookup — the foundation
every other Q-ai feature cites.

**The Phase-1 promise (PRD §7.3, §40, §46):**

> For any legal reference, Q-ai returns the exact canonical Arabic text of an identified
> edition version, byte-for-byte equal to the approved source, with a resolvable citation,
> in single-digit milliseconds, **without** an LLM, vector store, or network call — and no
> code path exists that can silently change that text.

### 1.1 Non-negotiable invariants introduced here

| # | Invariant | Enforcement |
|---|---|---|
| I1 | Canonical ayah text is immutable per edition version | insert-only tables + triggers + `CanonicalWriter`/`ApprovalToken` (Phase 0) |
| I2 | Canonical text is never produced by a model | canonical write path has no model dependency; architecture test forbids `llm` dep in `quran-corpus` |
| I3 | Different readings/editions are never merged into a synthetic text | edition_id is part of every canonical primary key and every returned record |
| I4 | Token order is stable and verifiable | `(edition_id, surah, ayah, position)` unique + order checksum per ayah |
| I5 | A partially imported edition can never become `Active` | staging tables + single-transaction pointer flip |
| I6 | Every returned quotation carries edition id + version + hash | `QuranQuotation` type has no constructor without them |
| I7 | Corrections create a new version with a difference report and human approval | `CanonicalChangeRequest` (Phase 0) + `quran_edition_v1` differ |

---

## 2. Scope

### 2.1 In Scope

| # | Item | PRD |
|---|------|-----|
| 1 | Quran edition model (all §7.2 fields) | §7.2 |
| 2 | Hierarchy: Edition → Surah → Ayah → Segment → Token (surface only) | §7.1 |
| 3 | Stable addressing + reference grammar + parser/serializer | §7.4 |
| 4 | Structural metadata: juz, hizb, rubʿ, manzil, page, sajdah, ruku, Makki/Madani, basmala policy | §7.4, §13.1 |
| 5 | Edition source format spec + `quran_edition_v1` structure validator | §22.4, §35.1 |
| 6 | Canonical importer job (quarantine → parse → validate → hash → stage → approve → activate) | §34 |
| 7 | Corpus validator (counts, identifiers, Unicode, token order, checksum, round-trip, reference-corpus comparison) | §35.1 |
| 8 | Atomic activation + rollback + edition version diff | §7.3, §85 |
| 9 | Exact lookup + context retrieval service with caching | §11.4, §40 |
| 10 | Translation model (attributed, verse-level and optional word-level) | §7.2, §13.1 |
| 11 | Quran read API v1 (`editions`, `surahs`, `ayahs/:ref`, `context/:ref`) | §27.1 |
| 12 | Tool result contract + first tools `quran.get_ayah`, `quran.get_context` | §11.4, §12 |
| 13 | Research checksum computation (deterministic parts) | §12.1 |
| 14 | Citation resolver v1 + deep-link format | §13.3, §21.2, §35.3 |
| 15 | CLI: `qai quran get/list/import/validate/activate/diff/rollback` | §26 |
| 16 | `qai doctor --quran` | §50 |
| 17 | Corpus integrity test suite + golden fixtures | §35.1, §46 |

### 2.2 Explicitly Out of Scope (Phase 1)

- **All search** (exact/normalized/phrase/concatenated/regex) → **Phase 2**.
- Normalization forms, root/lemma/stem/morphology, word families, frequency → **Phase 2**.
- Graph nodes/edges → **Phase 3**.
- Web reading UI → **Phase 4** (Phase 1 delivers the API + a minimal HTML debug reader only).
- Tafsir, hadith, other scriptures → **Phase 5/8**.
- Recitation audio, tajwid, qira'at variants → **Phase 4 / post-MVP** (schema fields reserved).
- Agents/LLM → **Phase 9**.

### 2.3 Data-source prerequisite (start Week 0, hard gate)

Phase 1 **cannot ship** without an approved edition dataset. This is a *legal + editorial*
task, not a coding task, and must start before Sprint 1.1.

**ADR-0101 (Initial Quran dataset & license)** must record:
- candidate datasets, their provenance, script (Uthmani), riwayah (Hafs ʿan ʿĀṣim),
  verse-numbering scheme, Unicode normalization form, and license;
- redistribution rights (§38) — whether the dataset may be bundled or must be user-supplied;
- the **reference corpus** used for independent comparison in validation (§35.1);
- editorial sign-off by a qualified reviewer that the text matches a recognized printed
  muṣḥaf, with the reviewer named in the `verified_by` field.

**Fallback if no dataset can be bundled:** Q-ai ships with the schema, validator, importer,
and a small **public-domain test fixture** (a handful of surahs) for tests, and requires the
user to run `qai quran import <manifest>` with their own approved dataset. This fallback must
be decided in ADR-0101, not improvised.

---

## 3. Domain Model

### 3.1 Edition (PRD §7.2)

```rust
// crates/quran-core/src/edition.rs
pub struct QuranEdition {
    pub id: EditionId,
    pub slug: String,                 // "hafs-uthmani"
    pub name: String,
    pub script: Script,               // Uthmani | ImlaeiSimple | Other(String)
    pub riwayah: Option<String>,      // "Hafs"
    pub qiraah: Option<String>,       // "Asim"
    pub publisher: Option<String>,
    pub source_url: Option<String>,
    pub license: LicenseRecord,        // from domain (Phase 0)
    pub language: Language,            // "ar"
    pub verse_numbering_scheme: NumberingScheme, // Hafs | Kufi | Custom(String)
    pub basmala_policy: BasmalaPolicy,
    pub unicode_normalization: UnicodeForm,      // as stored; NFC expected
    pub text_hash: ContentHash,        // over canonical ayah text stream
    pub structure_hash: ContentHash,   // over structural skeleton
    pub token_order_hash: ContentHash, // over token ids/positions
    pub manifest_hash: ContentHash,
    pub version: SemVer,
    pub source_version_id: SourceVersionId,
    pub imported_at: Timestamp,
    pub verified_at: Option<Timestamp>,
    pub verified_by: Option<String>,   // named human reviewer
    pub verification_method: Option<String>,
    pub status: EditionStatus,         // Staged | Approved | Active | Deprecated | Quarantined
    pub statistics: EditionStatistics, // counts, computed at import
}

/// §13.1 — basmala is edition-metadata-driven, never guessed.
pub enum BasmalaPolicy {
    /// Basmala is ayah 1 of the surah (as in Al-Fatihah).
    CountedAsFirstAyah,
    /// Basmala precedes the surah and is not numbered.
    UnnumberedHeader,
    /// Absent (e.g. At-Tawbah).
    Absent,
    /// Per-surah override table supplied by the edition.
    PerSurah,
}

pub struct EditionStatistics {
    pub surah_count: u16,
    pub ayah_count: u32,
    pub token_count: u64,
    pub distinct_surface_forms: u64,
    pub juz_count: u16,
    pub page_count: Option<u32>,
    pub sajdah_count: u16,
}
```

### 3.2 Structure (PRD §7.1)

```rust
pub struct Surah {
    pub edition_id: EditionId,
    pub number: SurahNumber,           // 1..=114 newtype, validated
    pub name_arabic: String,
    pub name_transliteration: Option<String>,
    pub name_translations: BTreeMap<Language, String>,
    pub ayah_count: u16,
    pub revelation_place: Option<RevelationPlace>, // Makki | Madani  (§6.2 metadata)
    pub revelation_order: Option<u16>,
    pub basmala: BasmalaPolicy,
    pub ruku_count: Option<u16>,
    pub metadata_provenance: ProvenanceId,          // revelation place is Layer B/C, not canonical
}

pub struct Ayah {
    pub edition_id: EditionId,
    pub surah: SurahNumber,
    pub number: AyahNumber,
    pub text: String,                  // CANONICAL. Exactly as in the approved source.
    pub text_hash: ContentHash,
    pub char_count: u32,               // grapheme clusters
    pub token_count: u16,
    pub juz: Option<u16>,
    pub hizb: Option<u16>,
    pub rub: Option<u16>,
    pub manzil: Option<u16>,
    pub ruku: Option<u16>,
    pub page: Option<u32>,
    pub sajdah: Option<SajdahKind>,    // Recommended | Obligatory
    pub global_ayah_index: u32,        // 1-based across the edition, for interval math (Phase 2)
}

/// §7.1 — a segment groups tokens for display/analysis without altering canonical order.
pub struct Segment {
    pub edition_id: EditionId,
    pub surah: SurahNumber,
    pub ayah: AyahNumber,
    pub index: u16,
    pub kind: SegmentKind,             // Clause | PauseMark | Basmala | Other
    pub token_range: (u16, u16),
    pub provenance: ProvenanceId,      // Layer B or C — never canonical
}

/// Phase 1 stores only the SURFACE token. Lemma/root/morphology arrive in Phase 2.
pub struct Token {
    pub edition_id: EditionId,
    pub surah: SurahNumber,
    pub ayah: AyahNumber,
    pub position: u16,                 // 1-based within the ayah
    pub surface: String,               // canonical Uthmani surface form
    pub surface_hash: ContentHash,
    pub char_start: u32,               // grapheme offset within ayah.text
    pub char_end: u32,
    pub byte_start: u32,
    pub byte_end: u32,
    pub is_pause_mark: bool,
    pub global_token_index: u64,
}
```

**Round-trip guarantee (tested):** for every ayah,
`join(tokens_by_position, separators_recorded) == ayah.text` byte-for-byte, and
`ayah.text[token.byte_start..token.byte_end] == token.surface`.
Separators are stored in `ayah_token_separators` so reconstruction is lossless even with
unusual whitespace or embedded marks.

### 3.3 Addressing (PRD §7.4)

```rust
pub enum QuranRef {
    Edition { edition: EditionSelector },
    Surah   { edition: EditionSelector, surah: SurahNumber },
    Ayah    { edition: EditionSelector, surah: SurahNumber, ayah: AyahNumber },
    AyahRange { edition: EditionSelector, start: (SurahNumber, AyahNumber),
                end: (SurahNumber, AyahNumber) },
    Token   { edition: EditionSelector, surah: SurahNumber, ayah: AyahNumber, position: u16 },
    Juz     { edition: EditionSelector, juz: u16 },
    Hizb    { edition: EditionSelector, hizb: u16 },
    Rub     { edition: EditionSelector, rub: u16 },
    Manzil  { edition: EditionSelector, manzil: u16 },
    Page    { edition: EditionSelector, page: u32 },
    Ruku    { edition: EditionSelector, surah: Option<SurahNumber>, ruku: u16 },
    Sajdah  { edition: EditionSelector, index: u16 },
}

pub enum EditionSelector {
    Active,                   // resolves via the active-edition pointer
    Slug(String),             // "hafs-uthmani"
    Pinned { slug: String, version: SemVer }, // reproducible research (§12.1)
}
```

**Reference grammar (EBNF, frozen by ADR-0102):**

```ebnf
reference     = [ "quran" ":" ] [ edition ":" ] locator ;
edition       = slug [ "@" semver ] ;
slug          = ALPHA , { ALPHA | DIGIT | "-" } ;
locator       = ayah_locator | range_locator | token_locator | division_locator ;

ayah_locator  = surah [ ":" ayah ] ;
range_locator = surah ":" ayah "-" ( ayah | surah ":" ayah ) ;
token_locator = surah ":" ayah ":" ( "token" ":" )? position ;
division_locator = ( "juz" | "hizb" | "rub" | "manzil" | "page" | "ruku" | "sajdah" )
                   ":" number ;

surah = number ; ayah = number ; position = number ;
```

Accepted, all resolving deterministically:

```text
quran:1:1
2:255
quran:2:255-257
quran:2:255-3:2
quran:2:255:token:3
2:255:3
quran:hafs-uthmani:36:1-12
quran:hafs-uthmani@1.0.0:112
quran:juz:30
quran:page:604
quran:sajdah:1
```

Canonical serialization is **always** the fully qualified pinned form for storage and
citations: `quran:hafs-uthmani@1.0.0:2:255`. The short forms are input conveniences only.

**Deep link format (§13.3):** `/read/{edition_slug}@{version}/{surah}:{ayah}?highlight=token:3`
plus an API-resolvable `qai://quran/...` URN stored in citations.

### 3.4 Translations (PRD §7.2, §13.1, principle 5)

```rust
pub struct TranslationEdition {
    pub id: EditionId,
    pub slug: String,
    pub name: String,
    pub translator: String,             // required, non-empty (principle 5 / §16)
    pub language: Language,
    pub source_version_id: SourceVersionId,
    pub license: LicenseRecord,
    pub aligned_to_edition: EditionId,  // which Arabic edition's numbering it follows
    pub numbering_scheme: NumberingScheme,
    pub trust_level: TrustLevel,
    pub version: SemVer,
    pub status: EditionStatus,
    pub text_hash: ContentHash,
}

pub struct TranslationPassage {
    pub translation_edition_id: EditionId,
    pub surah: SurahNumber,
    pub ayah: AyahNumber,
    pub text: String,
    pub footnotes: Vec<Footnote>,
    pub provenance: ProvenanceId,       // Layer B/C — never CanonicalSource
}

/// Optional word-by-word gloss (§13.1). Explicitly a separate, attributed dataset.
pub struct WordGloss {
    pub gloss_dataset_id: SourceId,
    pub edition_id: EditionId,          // Arabic edition it aligns to
    pub surah: SurahNumber, pub ayah: AyahNumber, pub position: u16,
    pub gloss: String,
    pub language: Language,
    pub provenance: ProvenanceId,
}
```

**Type-level guard for principle 5** ("translations must not be presented as the original"):
the API/tool response type is

```rust
pub struct AyahView {
    pub canonical: QuranQuotation,              // Arabic, edition-identified
    pub translations: Vec<AttributedTranslation>, // each REQUIRES translator + edition + license
    pub word_glosses: Option<Vec<AttributedGloss>>,
}
```

There is no variant of `AyahView` in which a translation can occupy the `canonical` field,
and `AttributedTranslation` has no constructor without `translator` and `edition_ref`.

---

## 4. Deliverables

### D1.1 — `quran-core` Crate (Pure Domain)

Types from §3, plus:
- `SurahNumber`, `AyahNumber`, `TokenPosition` newtypes with validated construction.
- `QuranRef` parser/serializer with exhaustive error variants (`QAI-QUR-01xx`).
- `QuranQuotation` (see D1.9) — the only type allowed to represent quoted canonical text.
- `Script`, `NumberingScheme`, `RevelationPlace`, `SajdahKind`, `UnicodeForm`.
- **No** async, no I/O, no `sqlx`. Architecture test: `quran-core` deps ⊆ `{domain, serde,
  thiserror}`.

### D1.2 — Edition Source Format & Manifest Extension

**PRD:** §22.4, §34

Defines what an importable Quran edition looks like, so any dataset can be adapted without
changing core code.

`docs/schemas/quran-edition-source.v1.schema.json` — the *normalized intermediate format*
that adapters produce:

```json
{
  "format": "qai.quran.edition",
  "format_version": 1,
  "edition": {
    "slug": "hafs-uthmani",
    "name": "Quran — Hafs, Uthmani script",
    "script": "uthmani",
    "riwayah": "Hafs",
    "qiraah": "Asim",
    "language": "ar",
    "verse_numbering_scheme": "hafs",
    "unicode_normalization": "nfc",
    "basmala_policy": "per_surah",
    "publisher": "…",
    "version": "1.0.0"
  },
  "expected": {
    "surah_count": 114,
    "ayah_count": 6236,
    "reference_corpus_id": "reference-hafs-2024",
    "reference_text_hash": "sha256:…"
  },
  "surahs": [
    {
      "number": 1,
      "name_arabic": "الفاتحة",
      "name_transliteration": "Al-Fatihah",
      "ayah_count": 7,
      "revelation_place": "makki",
      "revelation_order": 5,
      "basmala": "counted_as_first_ayah",
      "ruku_count": 1
    }
  ],
  "ayahs": [
    {
      "surah": 1,
      "ayah": 1,
      "text": "بِسْمِ ٱللَّهِ ٱلرَّحْمَٰنِ ٱلرَّحِيمِ",
      "juz": 1, "hizb": 1, "rub": 1, "manzil": 1,
      "ruku": 1, "page": 1, "sajdah": null
    }
  ],
  "tokenization": {
    "strategy": "whitespace_preserving",
    "separator_policy": "record_exact"
  }
}
```

**Adapters** (`quran-corpus::adapters`) convert real-world dataset shapes (JSON per-ayah,
XML, SQL dump, CSV) into this format. Each adapter is a small, independently tested module
declaring the `source_id` pattern it handles. Adding a new dataset = new adapter + manifest,
never a core change (§56 Extensibility).

`quran_edition_v1` is registered in Phase 0's `StructureValidator` registry.

### D1.3 — Canonical Importer

**PRD:** §34, §7.3, §85

Implemented as the job `quran.import` with resumable checkpoints, following §34 exactly:

```text
 1  claim source version (must be Downloaded|Staged)          [checkpoint: claimed]
 2  verify declared vs observed file hashes                    [checkpoint: hashed]
 3  detect format, select adapter                              [checkpoint: adapter]
 4  parse → intermediate format (Untrusted<T> unwrapped here)  [checkpoint: parsed]
 5  Unicode audit (form, forbidden ranges, normalization)      [checkpoint: unicode]
 6  structural validation (D1.4)                               [checkpoint: validated]
 7  tokenize + build separators + offsets                      [checkpoint: tokenized]
 8  compute text_hash / structure_hash / token_order_hash      [checkpoint: hashed2]
 9  write to STAGING tables under a staging edition id         [checkpoint: staged]
10  round-trip verification against staged rows                [checkpoint: roundtrip]
11  reference-corpus comparison (if configured)                [checkpoint: compared]
12  difference report vs current Active version                 [checkpoint: diffed]
13  → state Staged; emit approval request                      [terminal for import job]
```

Then, separately and only by a human:

```text
qai quran activate <edition>@<version>   -> requires approval; ONE transaction:
    INSERT active pointer row (or UPDATE) ; set old Active -> Deprecated ;
    set new -> Active ; write audit event ; bump corpus generation counter
```

**Guarantees**

- Staging tables (`quran_stg_*`) are physically separate from canonical tables. Nothing
  reads staging except validation and the activation transaction.
- Activation is a single transaction ending in a pointer flip (§85). A crash mid-import
  leaves `Staged`, never `Active`.
- The importer holds no `ApprovalToken`; it literally cannot activate (I5, I7).
- `quran.import` is idempotent on `(source_version_id, adapter_version, parser_version)`.
- Cancellation drops staging rows for that run and records the cancellation (§54).

### D1.4 — Corpus Validator

**PRD:** §35.1, §46

`quran-corpus::validation` produces a machine-readable `ValidationReport` with one entry per
rule. All rules run; the report lists every failure (no fail-fast), because partial reports
are useless for editorial review.

| Rule ID | Rule | Severity |
|---|---|---|
| QV-001 | Surah count equals manifest `expected.surah_count` (114 for standard editions) | Fatal |
| QV-002 | Surah numbers are exactly `1..=N`, no gaps, no duplicates | Fatal |
| QV-003 | Each surah's ayah count equals its declared count | Fatal |
| QV-004 | Total ayah count equals manifest `expected.ayah_count` | Fatal |
| QV-005 | Ayah numbers within each surah are exactly `1..=n`, no gaps/duplicates | Fatal |
| QV-006 | No ayah text is empty or whitespace-only | Fatal |
| QV-007 | All text is valid UTF-8 and in the declared Unicode normalization form | Fatal |
| QV-008 | No forbidden code points (C0/C1 controls except none, BOM, bidi overrides, PUA, unassigned) | Fatal |
| QV-009 | Arabic code points are within expected blocks; any out-of-block point is reported with location | Error |
| QV-010 | Token positions per ayah are exactly `1..=k`, monotonic | Fatal |
| QV-011 | Token round-trip: reconstructed ayah == stored ayah text, byte-identical | Fatal |
| QV-012 | Token offsets are valid grapheme/byte boundaries inside the ayah text | Fatal |
| QV-013 | Declared file hashes match observed hashes | Fatal |
| QV-014 | Serialization round-trip: intermediate → DB → intermediate is hash-stable | Fatal |
| QV-015 | Reference-corpus comparison: zero ayah-text differences, or all differences explicitly acknowledged in the manifest | Fatal (if reference configured) |
| QV-016 | Juz values, where present, are contiguous and cover the corpus | Error |
| QV-017 | Hizb/rubʿ/manzil consistency with juz where all are present | Warning |
| QV-018 | Page numbers, where present, are monotonic non-decreasing by global ayah index | Error |
| QV-019 | Sajdah markers count matches manifest expectation | Warning |
| QV-020 | Basmala policy is stated for every surah and consistent with the text | Error |
| QV-021 | Revelation place present for every surah (Layer B metadata) | Warning |
| QV-022 | No duplicate ayah text within a surah unless whitelisted (legitimate repetition exists, e.g. Ar-Rahman refrain) | Info |
| QV-023 | `global_ayah_index` and `global_token_index` are dense and monotonic | Fatal |
| QV-024 | Token-order hash is reproducible from stored rows | Fatal |
| QV-025 | Every ayah has provenance pointing to the importing source version | Fatal |
| QV-026 | Every structural-metadata field is attributed to Layer B provenance, not canonical | Error |
| QV-027 | Translation editions, if imported, align to an existing Arabic edition and numbering scheme | Error |
| QV-028 | Search normalization (Phase 2 hook) does not alter stored canonical text — verified by re-reading after any index build | Fatal |

`Fatal` ⇒ import cannot reach `Staged`. `Error` ⇒ can stage, cannot be approved without an
explicit override recorded in the approval record. `Warning`/`Info` ⇒ shown in the report.

**Corpus generation counter:** `corpus_generation` increments on every activation/rollback.
Every cache entry and derived index (Phase 2+) records the generation it was built from, so
staleness is detectable rather than assumed (§76).

### D1.5 — Persistence: Migrations

See §6 for full DDL. Summary:

```text
0010_quran_editions.up.sql     editions, active pointer, edition statistics
0011_quran_structure.up.sql    surahs, ayahs, segments, tokens, separators (+triggers)
0012_quran_divisions.up.sql    juz/hizb/rub/manzil/page/ruku/sajdah tables
0013_quran_translations.up.sql translation editions, passages, footnotes, word glosses
0014_quran_staging.up.sql      quran_stg_* mirrors for import staging
0015_quran_validation.up.sql   validation reports, difference reports
```

### D1.6 — Lookup Service (Deterministic, LLM-Free)

**PRD:** §11.4, §40, §56

```rust
#[async_trait]
pub trait QuranReader: Send + Sync {
    async fn list_editions(&self, f: EditionFilter) -> Result<Vec<QuranEdition>>;
    async fn get_edition(&self, sel: &EditionSelector) -> Result<QuranEdition>;
    async fn list_surahs(&self, sel: &EditionSelector) -> Result<Vec<Surah>>;
    async fn get_surah(&self, sel: &EditionSelector, s: SurahNumber) -> Result<SurahWithAyahs>;

    async fn get_ayah(&self, r: &QuranRef, opts: AyahOptions) -> Result<AyahView>;
    async fn get_ayahs(&self, r: &QuranRef, opts: AyahOptions) -> Result<Vec<AyahView>>;

    /// §11.4 — context by CANONICAL structure, never arbitrary chunking.
    async fn get_context(&self, r: &QuranRef, c: ContextSpec) -> Result<ContextView>;

    async fn get_tokens(&self, r: &QuranRef) -> Result<Vec<Token>>;
    async fn resolve(&self, text: &str) -> Result<ResolvedRef>;  // parse + bounds-check
}

pub struct ContextSpec {
    pub before: u16,                 // ayahs before
    pub after: u16,                  // ayahs after
    pub boundary: ContextBoundary,   // Surah | Juz | Ruku | Page | None
    pub include_surah_header: bool,
    pub max_ayahs: u16,              // hard cap (§40 memory-bounded results)
}
```

**Performance requirements**

| Operation | p50 | p99 | Method |
|---|---|---|---|
| `get_ayah` (warm cache) | < 0.2 ms | < 1 ms | in-process LRU |
| `get_ayah` (cold, SQLite) | < 2 ms | < 8 ms | covering index |
| `get_context(±5)` | < 3 ms | < 12 ms | single range query |
| `get_surah` (longest surah) | < 25 ms | < 80 ms | range scan + streaming |
| `resolve` reference parse | < 20 µs | < 100 µs | zero-alloc parser |

**Caching (§40):** `moka`-style LRU keyed by
`(edition_id, version, corpus_generation, ref, options_hash)`. Invalidated wholesale when
`corpus_generation` changes. A cache-consistency test asserts that after activating a new
version, no stale text is ever served.

**Architecture test:** `quran-corpus` and `quran-core` must not depend on `llm`,
`embeddings`, `retrieval`, or any vector store crate — mechanizing PRD §40's
"canonical lookup must not require an LLM or vector database."

### D1.7 — Quran Read API v1

**PRD:** §27.1

```text
GET  /api/v1/quran/editions
       ?status=active&language=ar
GET  /api/v1/quran/editions/:slug
GET  /api/v1/quran/surahs?edition=hafs-uthmani@1.0.0
GET  /api/v1/quran/surahs/:number?edition=...
GET  /api/v1/quran/ayahs/:reference
       ?edition=...&translations=en-x,fa-y&glosses=true&tokens=true
GET  /api/v1/quran/context/:reference
       ?before=3&after=3&boundary=surah
GET  /api/v1/quran/divisions/:kind/:number     # juz|hizb|rub|manzil|page|ruku|sajdah
GET  /api/v1/quran/tokens/:reference
GET  /api/v1/quran/resolve?ref=<text>          # parse + validate a user reference
GET  /api/v1/quran/citations/:citation_id      # citation resolver (D1.9)
```

Response envelope (stable, versioned, reused by Phase 2+):

```json
{
  "api_version": "v1",
  "data": { "...": "..." },
  "meta": {
    "edition": { "slug": "hafs-uthmani", "version": "1.0.0",
                 "text_hash": "sha256:…", "script": "uthmani",
                 "riwayah": "Hafs", "numbering_scheme": "hafs" },
    "corpus_generation": 7,
    "canonical_reference": "quran:hafs-uthmani@1.0.0:2:255",
    "deep_link": "/read/hafs-uthmani@1.0.0/2:255",
    "execution_time_ms": 1.4,
    "reproducibility": { "checksum": "sha256:…", "inputs_hash": "sha256:…" },
    "warnings": []
  }
}
```

Cross-cutting API requirements met in Phase 1: request-size limit, timeout, concurrency limit,
`ETag` based on `(text_hash, corpus_generation)`, `Cache-Control` for canonical resources,
consistent error body from Phase 0's `Diagnostic`, and RFC-compliant content negotiation for
Arabic text (always UTF-8, `Content-Language`).

### D1.8 — Tool Contract & First Tools

**PRD:** §11.4, §12, §12.1, §29

Phase 1 defines the **tool result contract** (§12) once, so every later tool conforms.

```rust
pub struct ToolResult<T> {
    pub tool_name: String,
    pub tool_version: SemVer,
    pub query: serde_json::Value,          // exact input, redacted
    pub normalization_rules: Vec<RuleId>,  // empty in Phase 1
    pub edition_id: Option<EditionId>,
    pub edition_version: Option<SemVer>,
    pub results: T,
    pub canonical_references: Vec<String>, // fully-qualified refs
    pub analysis_sources: Vec<AnalysisSource>,
    pub confidence: Option<Confidence>,
    pub warnings: Vec<Warning>,
    pub execution_time_ms: f64,
    pub reproducibility: ReproducibilityData, // §12.1
}

/// §12.1 — deterministic checksum. Phase 1 covers the deterministic inputs;
/// Phase 2 adds normalization rules; Phase 9 adds model/prompt fields.
pub struct ReproducibilityData {
    pub checksum: ContentHash,
    pub query_hash: ContentHash,
    pub tool_plan: Vec<ToolInvocationRef>,   // name@version
    pub source_versions: BTreeMap<SourceId, SemVer>,
    pub edition_versions: BTreeMap<String, SemVer>,
    pub normalization_rule_set: Option<SemVer>,
    pub retrieval_config_hash: Option<ContentHash>,
    pub model_provider: Option<String>,
    pub model_name: Option<String>,
    pub prompt_version: Option<SemVer>,
    pub corpus_generation: u64,
    pub deterministic: bool,
}
```

Tools shipped in Phase 1, both `side_effect_class = ReadOnly` (§29):

| Tool | Input | Output |
|---|---|---|
| `quran.get_ayah` | `{reference, edition?, translations?, glosses?, tokens?}` | `ToolResult<Vec<AyahView>>` |
| `quran.get_context` | `{reference, before, after, boundary, edition?}` | `ToolResult<ContextView>` |

Both are registered in a minimal, local tool registry (no sandbox, no agents yet — that is
Phase 9/10) and are exercised by the CLI and API to prove the contract is usable from all
interfaces before agents exist.

**Test:** a "no fabrication" test asserts that requesting a non-existent reference returns a
typed error, never a synthesized ayah, and that no code path constructs
`QuranQuotation` from a string literal outside the corpus repository (enforced by a
`#[deny]`-style visibility rule: `QuranQuotation::new` is `pub(crate)` in `quran-corpus`).

### D1.9 — Citation Resolver v1

**PRD:** §13.3, §21.2, §35.3, §12

```rust
pub struct QuranQuotation {
    pub reference: String,           // "quran:hafs-uthmani@1.0.0:2:255"
    pub surah_number: SurahNumber,
    pub surah_name_arabic: String,
    pub surah_name_translit: Option<String>,
    pub ayah_range: (AyahNumber, AyahNumber),
    pub arabic_text: String,
    pub text_hash: ContentHash,
    pub edition: EditionRef,          // slug + version + script + riwayah
    pub translation: Option<TranslationRef>,
    pub deep_link: String,
    pub page: Option<u32>,
    pub juz: Option<u16>,
}
```

`Citation` + `CitationResolver` (crate `citations`, first real implementation):

```rust
pub trait CitationResolver: Send + Sync {
    /// §35.3: source exists, location resolves, quotation matches,
    /// edition exists, permission ok, hash+version available.
    async fn resolve(&self, c: &Citation) -> Result<ResolvedCitation, CitationError>;
    async fn verify_quotation(&self, c: &Citation, quoted: &str)
        -> Result<QuotationVerdict, CitationError>;
}

pub enum QuotationVerdict {
    ExactMatch,
    MatchAfterWhitespaceNormalization,
    /// Only ever produced with an explicit, displayed normalization rule list.
    MatchAfterDeclaredNormalization { rules: Vec<RuleId> },
    Mismatch { first_difference_at: u32, expected_hash: ContentHash },
    LocationNotFound,
    EditionNotFound,
    AccessDenied,
}
```

Phase 1 wires `Mismatch`/`LocationNotFound` into a hard failure for any answer path — the
mechanism that Phase 9's citation verifier will reuse. A stored `citations` table persists
resolved citations with `content_hash` + `ingestion_version` so a research report can be
re-verified later (§12.1, §21.2).

### D1.10 — CLI Surface

```bash
# Reading
qai quran get 2:255
qai quran get quran:2:255-257 --translations en-sahih --json
qai quran get 1:1 --tokens --glosses
qai quran context 18:60 --before 3 --after 3 --boundary surah
qai quran surah 36 [--metadata]
qai quran division juz 30
qai quran resolve "بقره ۲۵۵"          # reference parsing diagnostics (not text search)

# Editions
qai quran edition list [--json]
qai quran edition show hafs-uthmani [--statistics] [--hashes]
qai quran edition active
qai quran edition set-active hafs-uthmani@1.0.0

# Corpus lifecycle
qai quran import <manifest-path> [--dry-run] [--adapter <name>]
qai quran validate <edition>@<version> [--report <path>] [--json]
qai quran diff <edition> --from 1.0.0 --to 1.1.0 [--format text|json|unified]
qai quran activate <edition>@<version>          # prompts; requires approval
qai quran rollback <edition> --to 1.0.0 --yes
qai quran deprecate <edition>@<version>
qai quran hashes <edition>@<version>            # recompute and compare

# Translations
qai quran translation list
qai quran translation import <manifest-path>
qai quran translation show <slug>

# Diagnostics
qai doctor --quran [--json]
```

`--json` output for `qai quran get` includes the full `meta` envelope so scripts get the
edition version and reproducibility checksum for free.

### D1.11 — `qai doctor --quran`

**PRD:** §50

```text
QURAN
  quran.edition_active                 Exactly one active Arabic edition
  quran.edition_checksum               Stored text_hash matches recomputed hash
  quran.structure_hash                 Structural skeleton hash matches
  quran.token_order_hash               Token order hash matches
  quran.surah_count                    114 (or manifest expectation)
  quran.ayah_count                     Matches edition statistics + manifest
  quran.ayah_identifiers               No gaps/duplicates in any surah
  quran.token_roundtrip                Sampled + full-scan mode (--deep)
  quran.unicode_form                   Stored form matches declared form
  quran.division_coverage              Juz/hizb/rub/page coverage complete
  quran.basmala_policy                 Declared for every surah
  quran.provenance_complete            Every ayah has canonical provenance
  quran.metadata_layering              No Layer-B metadata stored as canonical
  quran.translations_aligned           Every translation aligns to an existing edition+scheme
  quran.translation_attribution        Every translation has a non-empty translator
  quran.staging_orphans                No leftover quran_stg_* rows
  quran.corpus_generation              Caches/derived indexes match current generation
  quran.reference_corpus               Comparison result recorded and clean
  quran.license_status                 Active edition license is not Unknown/Restricted-without-note
```

`--deep` runs full-corpus hash recomputation and full token round-trip
(target: complete in under 30 seconds for a standard edition).
`doctor` remains strictly read-only (Phase 0 AC-P0-14).

### D1.12 — Minimal Debug Reader (not the Phase-4 UI)

A single server-rendered page at `/debug/read/{edition}/{surah}` that renders ayahs with
correct RTL and a web font, purely to let engineers and editorial reviewers eyeball the
imported text before Phase 4 exists. Explicitly labeled "debug view", no features, no
persistence, excluded from the product navigation.

Rationale: catching a mangled dataset in Week 3 instead of Week 20 is worth 1.5 engineer-days.

---

## 5. Corpus Integrity Test Suite

The permanent regression net (PRD §35.1, §46, §58).

### 5.1 Golden fixtures

`fixtures/quran/`

| Fixture | Purpose |
|---|---|
| `golden/ayah_texts.jsonl` | A curated set of ~200 ayahs (including all edge cases below) with expected exact text + hash |
| `golden/edition_hashes.json` | Expected `text_hash`, `structure_hash`, `token_order_hash` per fixture edition |
| `golden/references.jsonl` | ~300 reference strings → expected parse result or expected error code |
| `golden/tokens.jsonl` | Expected token surfaces + offsets for selected ayahs |
| `golden/divisions.json` | Expected juz/hizb/page boundaries for the fixture edition |
| `test-edition-min/` | A small public-domain-safe synthetic edition (structurally valid, 5 surahs) for fast tests |
| `adversarial/` | Deliberately broken editions (see 5.3) |

### 5.2 Edge cases that must be in the golden set

- Al-Fatihah 1:1 (basmala counted as an ayah)
- At-Tawbah 9:1 (no basmala)
- An-Naml 27:30 (basmala inside an ayah)
- Al-Baqarah 2:255 (long ayah)
- Al-Kawthar 108 (short surah)
- Al-Baqarah 2:1 (muqattaʿat / disjointed letters)
- Ar-Rahman refrain (legitimately repeated ayah text)
- Ayahs containing sajdah markers
- Ayahs spanning a page boundary
- The first and last ayah of the corpus
- An ayah containing a pause mark (waqf) sign
- An ayah with a superscript alef (U+0670) and small high signs
- The longest and shortest tokens in the edition
- An ayah where two tokens are separated by a mark rather than a plain space

### 5.3 Adversarial import corpora (each must be rejected with the correct rule ID)

| Fixture | Injected fault | Expected |
|---|---|---|
| `missing_ayah` | one ayah removed | QV-005 Fatal |
| `duplicate_ayah_id` | duplicated identifier | QV-005 Fatal |
| `extra_surah` | 115 surahs | QV-001 Fatal |
| `wrong_ayah_count` | surah count mismatch | QV-003 Fatal |
| `nfd_text` | text in NFD while declaring NFC | QV-007 Fatal |
| `bom_prefix` | BOM inside a text field | QV-008 Fatal |
| `bidi_override` | U+202E injected | QV-008 Fatal |
| `zero_width` | ZWJ/ZWNJ inserted mid-token | QV-008/QV-011 Fatal |
| `latin_homoglyph` | Latin `ا`-lookalike | QV-009 Error |
| `bad_offsets` | token offsets off by one | QV-012 Fatal |
| `shuffled_tokens` | token positions permuted | QV-010/QV-024 Fatal |
| `truncated_ayah` | ayah text silently truncated | QV-011/QV-015 Fatal |
| `hash_mismatch` | manifest hash altered | QV-013 Fatal |
| `page_regression` | page numbers decrease | QV-018 Error |
| `missing_juz` | juz gap | QV-016 Error |
| `translation_as_canonical` | translation submitted as an Arabic edition | QV-027 Error + type-level rejection |

### 5.4 Property tests

- For every ayah: `reconstruct(tokens, separators) == text` (byte equality).
- For every token: `text[byte_start..byte_end] == surface`.
- For every valid reference: `parse(serialize(ref)) == ref` (round trip).
- For random `(before, after, boundary)`: context never crosses the declared boundary and
  never exceeds `max_ayahs`.
- For random reference strings: parser never panics and always returns a coded error.
- Hash stability: recomputing `text_hash` from DB rows equals the import-time value.

### 5.5 Immutability tests

- `UPDATE quran_ayahs SET text = …` → aborts via trigger with `QAI-QUR-0001`.
- `DELETE FROM quran_ayahs` → aborts.
- Writing an ayah without a `CanonicalChangeSession` → compile error (no public API) plus a
  runtime repository test proving the guard.
- A `CanonicalChangeRequest` missing a difference report, checksum result, or approver → rejected.
- Activating a `Staged` version whose validation report has any `Fatal` → rejected.
- Simulated crash (process kill) at each of the 13 importer checkpoints → the active edition
  is unchanged in all 13 cases, and resume completes correctly.

---

## 6. Database Schema (Phase 1 Migrations)

### `0010_quran_editions.up.sql`

```sql
CREATE TABLE quran_editions (
  id                     TEXT PRIMARY KEY,
  slug                   TEXT NOT NULL,
  version                TEXT NOT NULL,
  name                   TEXT NOT NULL,
  script                 TEXT NOT NULL,
  riwayah                TEXT,
  qiraah                 TEXT,
  publisher              TEXT,
  source_url             TEXT,
  language               TEXT NOT NULL,
  verse_numbering_scheme TEXT NOT NULL,
  basmala_policy         TEXT NOT NULL CHECK (basmala_policy IN
                            ('counted_as_first_ayah','unnumbered_header','absent','per_surah')),
  unicode_normalization  TEXT NOT NULL CHECK (unicode_normalization IN ('nfc','nfd','nfkc','nfkd')),
  license_json           TEXT NOT NULL,
  text_hash              TEXT NOT NULL,
  structure_hash         TEXT NOT NULL,
  token_order_hash       TEXT NOT NULL,
  manifest_hash          TEXT NOT NULL,
  source_version_id      TEXT NOT NULL REFERENCES source_versions(id),
  statistics_json        TEXT NOT NULL,
  status                 TEXT NOT NULL CHECK (status IN
                            ('Staged','Approved','Active','Deprecated','Quarantined')),
  imported_at            TEXT NOT NULL,
  verified_at            TEXT,
  verified_by            TEXT,
  verification_method    TEXT,
  activated_at           TEXT,
  deprecated_at          TEXT,
  UNIQUE (slug, version)
);

-- Exactly one active Arabic edition pointer; row id fixed at 1 (§7.3, §85).
CREATE TABLE quran_active_edition (
  singleton          INTEGER PRIMARY KEY CHECK (singleton = 1),
  edition_id         TEXT NOT NULL REFERENCES quran_editions(id),
  corpus_generation  INTEGER NOT NULL,
  activated_at       TEXT NOT NULL,
  activated_by       TEXT NOT NULL REFERENCES principals(id),
  approval_id        TEXT NOT NULL REFERENCES approvals(id)
);

CREATE TRIGGER trg_edition_immutable_hashes
BEFORE UPDATE OF text_hash, structure_hash, token_order_hash, slug, version
ON quran_editions
BEGIN
  SELECT RAISE(ABORT, 'QAI-QUR-0002: edition identity and hashes are immutable');
END;
```

### `0011_quran_structure.up.sql`

```sql
CREATE TABLE quran_surahs (
  edition_id            TEXT NOT NULL REFERENCES quran_editions(id) ON DELETE RESTRICT,
  number                INTEGER NOT NULL CHECK (number BETWEEN 1 AND 200),
  name_arabic           TEXT NOT NULL,
  name_transliteration  TEXT,
  name_translations_json TEXT NOT NULL DEFAULT '{}',
  ayah_count            INTEGER NOT NULL CHECK (ayah_count > 0),
  revelation_place      TEXT CHECK (revelation_place IN ('makki','madani')),
  revelation_order      INTEGER,
  basmala               TEXT NOT NULL,
  ruku_count            INTEGER,
  metadata_provenance_id TEXT REFERENCES provenance_records(id),
  PRIMARY KEY (edition_id, number)
);

CREATE TABLE quran_ayahs (
  edition_id        TEXT NOT NULL,
  surah             INTEGER NOT NULL,
  ayah              INTEGER NOT NULL CHECK (ayah > 0),
  text              TEXT NOT NULL CHECK (length(trim(text)) > 0),
  text_hash         TEXT NOT NULL,
  char_count        INTEGER NOT NULL,
  token_count       INTEGER NOT NULL,
  global_ayah_index INTEGER NOT NULL,
  juz               INTEGER, hizb INTEGER, rub INTEGER, manzil INTEGER,
  ruku              INTEGER, page INTEGER,
  sajdah            TEXT CHECK (sajdah IS NULL OR sajdah IN ('recommended','obligatory')),
  provenance_id     TEXT NOT NULL REFERENCES provenance_records(id),
  PRIMARY KEY (edition_id, surah, ayah),
  FOREIGN KEY (edition_id, surah) REFERENCES quran_surahs(edition_id, number),
  UNIQUE (edition_id, global_ayah_index)
);

CREATE INDEX ix_ayah_global ON quran_ayahs(edition_id, global_ayah_index);
CREATE INDEX ix_ayah_juz    ON quran_ayahs(edition_id, juz, global_ayah_index);
CREATE INDEX ix_ayah_page   ON quran_ayahs(edition_id, page, global_ayah_index);

CREATE TABLE quran_tokens (
  edition_id         TEXT NOT NULL,
  surah              INTEGER NOT NULL,
  ayah               INTEGER NOT NULL,
  position           INTEGER NOT NULL CHECK (position > 0),
  surface            TEXT NOT NULL,
  surface_hash       TEXT NOT NULL,
  char_start         INTEGER NOT NULL,
  char_end           INTEGER NOT NULL,
  byte_start         INTEGER NOT NULL,
  byte_end           INTEGER NOT NULL,
  is_pause_mark      INTEGER NOT NULL DEFAULT 0 CHECK (is_pause_mark IN (0,1)),
  global_token_index INTEGER NOT NULL,
  PRIMARY KEY (edition_id, surah, ayah, position),
  FOREIGN KEY (edition_id, surah, ayah) REFERENCES quran_ayahs(edition_id, surah, ayah),
  UNIQUE (edition_id, global_token_index),
  CHECK (byte_end > byte_start AND char_end > char_start)
);

CREATE INDEX ix_token_global  ON quran_tokens(edition_id, global_token_index);
CREATE INDEX ix_token_surface ON quran_tokens(edition_id, surface);

-- Lossless reconstruction: exact separator between token n and n+1 (and leading/trailing).
CREATE TABLE quran_token_separators (
  edition_id TEXT NOT NULL,
  surah      INTEGER NOT NULL,
  ayah       INTEGER NOT NULL,
  after_position INTEGER NOT NULL,   -- 0 = leading text before token 1
  separator  TEXT NOT NULL,
  PRIMARY KEY (edition_id, surah, ayah, after_position)
);

CREATE TABLE quran_segments (
  edition_id    TEXT NOT NULL,
  surah         INTEGER NOT NULL,
  ayah          INTEGER NOT NULL,
  seg_index     INTEGER NOT NULL,
  kind          TEXT NOT NULL,
  token_start   INTEGER NOT NULL,
  token_end     INTEGER NOT NULL,
  provenance_id TEXT NOT NULL REFERENCES provenance_records(id),
  PRIMARY KEY (edition_id, surah, ayah, seg_index),
  CHECK (token_end >= token_start)
);

-- I1: canonical rows are insert-only.
CREATE TRIGGER trg_ayah_no_update BEFORE UPDATE ON quran_ayahs
BEGIN SELECT RAISE(ABORT, 'QAI-QUR-0001: canonical ayah text is immutable'); END;
CREATE TRIGGER trg_ayah_no_delete BEFORE DELETE ON quran_ayahs
BEGIN SELECT RAISE(ABORT, 'QAI-QUR-0003: canonical ayah rows cannot be deleted'); END;
CREATE TRIGGER trg_token_no_update BEFORE UPDATE ON quran_tokens
BEGIN SELECT RAISE(ABORT, 'QAI-QUR-0004: canonical tokens are immutable'); END;
CREATE TRIGGER trg_token_no_delete BEFORE DELETE ON quran_tokens
BEGIN SELECT RAISE(ABORT, 'QAI-QUR-0005: canonical tokens cannot be deleted'); END;
```

> **Note on deletion:** deprecating an edition never deletes rows (§76). Physical removal of a
> `Removed` edition is a separate, audited, offline maintenance command that drops triggers in
> a dedicated migration-style transaction; it is documented in a runbook and out of scope for
> normal operation.

### `0012_quran_divisions.up.sql`

```sql
CREATE TABLE quran_divisions (
  edition_id     TEXT NOT NULL REFERENCES quran_editions(id),
  kind           TEXT NOT NULL CHECK (kind IN ('juz','hizb','rub','manzil','ruku','page','sajdah')),
  number         INTEGER NOT NULL,
  start_surah    INTEGER NOT NULL,
  start_ayah     INTEGER NOT NULL,
  end_surah      INTEGER NOT NULL,
  end_ayah       INTEGER NOT NULL,
  start_global   INTEGER NOT NULL,
  end_global     INTEGER NOT NULL,
  label          TEXT,
  provenance_id  TEXT NOT NULL REFERENCES provenance_records(id),
  PRIMARY KEY (edition_id, kind, number),
  CHECK (end_global >= start_global)
);

CREATE INDEX ix_div_range ON quran_divisions(edition_id, kind, start_global, end_global);
```

### `0013_quran_translations.up.sql`

```sql
CREATE TABLE translation_editions (
  id                 TEXT PRIMARY KEY,
  slug               TEXT NOT NULL,
  version            TEXT NOT NULL,
  name               TEXT NOT NULL,
  translator         TEXT NOT NULL CHECK (length(trim(translator)) > 0),  -- principle 5
  language           TEXT NOT NULL,
  aligned_edition_id TEXT NOT NULL REFERENCES quran_editions(id),
  numbering_scheme   TEXT NOT NULL,
  license_json       TEXT NOT NULL,
  trust_level        TEXT NOT NULL,
  source_version_id  TEXT NOT NULL REFERENCES source_versions(id),
  text_hash          TEXT NOT NULL,
  status             TEXT NOT NULL,
  imported_at        TEXT NOT NULL,
  UNIQUE (slug, version)
);

CREATE TABLE translation_passages (
  translation_edition_id TEXT NOT NULL REFERENCES translation_editions(id) ON DELETE RESTRICT,
  surah                  INTEGER NOT NULL,
  ayah                   INTEGER NOT NULL,
  text                   TEXT NOT NULL,
  footnotes_json         TEXT NOT NULL DEFAULT '[]',
  provenance_id          TEXT NOT NULL REFERENCES provenance_records(id),
  PRIMARY KEY (translation_edition_id, surah, ayah)
);

CREATE TABLE word_glosses (
  gloss_dataset_id TEXT NOT NULL REFERENCES sources(id),
  edition_id       TEXT NOT NULL,
  surah            INTEGER NOT NULL,
  ayah             INTEGER NOT NULL,
  position         INTEGER NOT NULL,
  language         TEXT NOT NULL,
  gloss            TEXT NOT NULL,
  provenance_id    TEXT NOT NULL REFERENCES provenance_records(id),
  PRIMARY KEY (gloss_dataset_id, edition_id, surah, ayah, position, language)
);
```

### `0014_quran_staging.up.sql`

Mirrors of `quran_editions`, `quran_surahs`, `quran_ayahs`, `quran_tokens`,
`quran_token_separators`, `quran_segments`, `quran_divisions` prefixed `quran_stg_`,
each with an extra `import_run_id TEXT NOT NULL` column, **no immutability triggers**, and an
`ON DELETE CASCADE` from an `import_runs` table so cancellation cleanup is one statement.

### `0015_quran_validation.up.sql`

```sql
CREATE TABLE validation_reports (
  id                TEXT PRIMARY KEY,
  subject_urn       TEXT NOT NULL,          -- edition or source version
  validator         TEXT NOT NULL,          -- 'quran_edition_v1'
  validator_version TEXT NOT NULL,
  outcome           TEXT NOT NULL CHECK (outcome IN ('pass','pass_with_warnings','fail')),
  fatal_count       INTEGER NOT NULL,
  error_count       INTEGER NOT NULL,
  warning_count     INTEGER NOT NULL,
  findings_json     TEXT NOT NULL,          -- [{rule_id, severity, location, message, detail}]
  created_at        TEXT NOT NULL
);

CREATE TABLE difference_reports (
  id             TEXT PRIMARY KEY,
  subject_urn    TEXT NOT NULL,
  from_version   TEXT NOT NULL,
  to_version     TEXT NOT NULL,
  differ         TEXT NOT NULL,
  differ_version TEXT NOT NULL,
  summary_json   TEXT NOT NULL,   -- {ayahs_added, ayahs_removed, ayahs_changed, metadata_changed}
  details_json   TEXT NOT NULL,   -- per-ayah diffs with char-level ranges
  created_at     TEXT NOT NULL
);

CREATE TABLE citations (
  id                    TEXT PRIMARY KEY,
  kind                  TEXT NOT NULL,          -- quran|translation|(hadith/tafsir later)
  canonical_reference   TEXT NOT NULL,
  source_id             TEXT REFERENCES sources(id),
  source_version_id     TEXT REFERENCES source_versions(id),
  edition_ref           TEXT,
  location_json         TEXT NOT NULL,
  quoted_text_hash      TEXT,
  ingestion_version     TEXT NOT NULL,
  resolved_at           TEXT NOT NULL,
  verdict               TEXT NOT NULL
);

CREATE INDEX ix_citations_ref ON citations(canonical_reference);
```

---

## 7. ADRs Required In Phase 1

| ADR | Title | Blocking | Notes |
|---|---|---|---|
| ADR-0101 | Initial Quran dataset, script, riwayah, and license | Everything | Must include editorial sign-off and the bundle-vs-user-supplied decision |
| ADR-0102 | Quran addressing scheme & reference grammar | D1.1 | Frozen grammar; changing it later breaks every stored citation |
| ADR-0103 | Verse-numbering scheme handling and alternate-numbering strategy | D1.1, D1.5 | How to represent editions with different numbering without merging |
| ADR-0104 | Unicode policy: normalization form, allowed blocks, forbidden code points, grapheme handling | D1.4 | Directly determines whether text is "corrupt" |
| ADR-0105 | Canonical tokenization rule for Phase 1 (whitespace-preserving surface tokenization) | D1.3 | Must be lossless; morphological segmentation is Phase 2 and separate |
| ADR-0106 | Canonical text storage layout (row-per-ayah in SQLite vs. blob+index) | D1.5 | Trade-off: query flexibility vs. hash simplicity |
| ADR-0107 | Atomic activation & rollback mechanism (staging + pointer flip + generation counter) | D1.3 | §85 compliance |
| ADR-0108 | Corpus hashing scheme: what exactly enters `text_hash` / `structure_hash` / `token_order_hash` | D1.3, D1.4 | Frozen forever; determines reproducibility |
| ADR-0109 | Edition difference algorithm for canonical text | D1.3 | Char-level vs token-level; reviewer usability |
| ADR-0110 | Basmala representation policy | D1.1 | Affects ayah counts and display |
| ADR-0111 | Citation identity and deep-link URL/URN format | D1.9 | Public, stable surface |
| ADR-0112 | Translation alignment and attribution model | D1.4 | Enforcing principle 5 |
| ADR-0113 | Canonical lookup caching & invalidation | D1.6 | Correctness over hit rate |
| ADR-0114 | Reference-corpus comparison procedure and who signs off | D1.4 | Editorial process, not just code |

---

## 8. Work Breakdown Structure

Total ≈ **82 ed** ⇒ ~6 weeks with 3 engineers. Roles: **BE**, **DATA** (corpus/data
engineering), **QA**, **DOC**, **EDIT** (editorial/legal reviewer, part-time).

### Sprint 1.0 — Data & Decisions (runs in parallel with Phase 0 Sprint 0.5)

| ID | Task | Deliverable | Depends | Est | Role |
|---|---|---|---|---|---|
| P1-T01 | Survey candidate Quran datasets: provenance, script, license, numbering | ADR-0101 | — | 3.0 | DATA |
| P1-T02 | Legal review of redistribution rights; decide bundle vs user-supplied | ADR-0101 | T01 | 2.0 | EDIT |
| P1-T03 | Select reference corpus + define comparison procedure and sign-off | ADR-0114 | T01 | 1.5 | EDIT |
| P1-T04 | Write ADR-0101/0104/0110 | ADR | T01–T03 | 2.0 | DOC |
| P1-T05 | Build `test-edition-min` + `adversarial/*` fixtures | D1.13 | T04 | 3.0 | QA |

### Sprint 1.1 — Domain & Addressing (Week 1–2)

| ID | Task | Deliverable | Depends | Est | Role |
|---|---|---|---|---|---|
| P1-T06 | `quran-core`: newtypes, `Script`, `NumberingScheme`, enums | D1.1 | P0 done | 1.5 | BE |
| P1-T07 | `quran-core`: edition/surah/ayah/segment/token structs | D1.1 | T06 | 2.0 | BE |
| P1-T08 | Reference grammar implementation: parser (zero-alloc) + serializer | D1.1 | T06 | 3.0 | BE |
| P1-T09 | Reference golden-set tests (300 cases incl. malformed) | D1.13 | T08 | 2.0 | QA |
| P1-T10 | `QuranQuotation` + constructor visibility guard + tests | D1.9 | T07 | 1.5 | BE |
| P1-T11 | ADR-0102/0103/0105 | ADR | T08 | 1.5 | DOC |
| P1-T12 | Migrations `0010`–`0012` + triggers + constraint tests | D1.5 | T07 | 3.0 | BE |
| P1-T13 | Repository layer: editions, surahs, ayahs, tokens, divisions (read paths) | D1.5 | T12 | 3.0 | BE |
| P1-T14 | Immutability test suite (trigger + API-level guards) | D1.13 | T12 | 2.0 | QA |

### Sprint 1.2 — Import & Validation (Week 2–3)

| ID | Task | Deliverable | Depends | Est | Role |
|---|---|---|---|---|---|
| P1-T15 | Intermediate format schema + JSON Schema + serde types | D1.2 | T07 | 2.0 | BE |
| P1-T16 | Adapter trait + adapter for the chosen dataset (per ADR-0101) | D1.2 | T15 | 3.0 | DATA |
| P1-T17 | Second adapter (different shape) to prove extensibility | D1.2 | T16 | 1.5 | DATA |
| P1-T18 | Register `quran_edition_v1` in the Phase-0 validator registry | D1.4 | T15 | 1.0 | BE |
| P1-T19 | Validator rules QV-001…QV-012 | D1.4 | T18 | 3.5 | BE |
| P1-T20 | Validator rules QV-013…QV-028 | D1.4 | T19 | 3.5 | BE |
| P1-T21 | Unicode auditor (blocks, forbidden points, normalization check, grapheme utils) | D1.4 | T18 | 2.5 | BE |
| P1-T22 | Tokenizer (whitespace-preserving) + separator capture + offset computation | D1.3 | T15 | 3.0 | BE |
| P1-T23 | Hashing: `text_hash`, `structure_hash`, `token_order_hash` per ADR-0108 | D1.3 | T22 | 2.0 | BE |
| P1-T24 | Migrations `0014_staging`, `0015_validation` | D1.5 | T12 | 1.5 | BE |
| P1-T25 | `quran.import` job with 13 checkpoints, cancellation, resume | D1.3 | T22–T24 | 4.0 | BE |
| P1-T26 | Round-trip verifier + reference-corpus comparator | D1.4 | T23 | 2.5 | BE |
| P1-T27 | Edition differ (char-level, per ADR-0109) + `DifferenceReport` | D1.3 | T23 | 2.5 | BE |
| P1-T28 | Activation transaction + rollback + `corpus_generation` | D1.3 | T25,T27 | 2.5 | BE |
| P1-T29 | Crash-at-each-checkpoint test matrix (13 cases) | D1.13 | T25,T28 | 2.5 | QA |
| P1-T30 | Adversarial-corpus rejection tests (16 fixtures × correct rule id) | D1.13 | T20,T25 | 2.5 | QA |
| P1-T31 | ADR-0106/0107/0108/0109 | ADR | T23,T28 | 2.0 | DOC |

### Sprint 1.3 — Reader, Translations, API (Week 4)

| ID | Task | Deliverable | Depends | Est | Role |
|---|---|---|---|---|---|
| P1-T32 | `QuranReader` implementation: get_ayah/get_ayahs/get_tokens | D1.6 | T13 | 2.5 | BE |
| P1-T33 | `get_context` with boundary logic + caps | D1.6 | T32 | 2.0 | BE |
| P1-T34 | Division lookups (juz/hizb/rub/manzil/page/ruku/sajdah) | D1.6 | T13 | 1.5 | BE |
| P1-T35 | Caching layer keyed by corpus generation + consistency tests | D1.6 | T32 | 2.0 | BE |
| P1-T36 | Migration `0013` + translation import job + alignment validation | D1.4 | T12 | 2.5 | BE |
| P1-T37 | `AyahView` / `AttributedTranslation` types + principle-5 guard tests | D1.6 | T36 | 1.5 | BE |
| P1-T38 | Word-gloss dataset import (optional path) | D1.4 | T36 | 1.5 | DATA |
| P1-T39 | API v1 handlers + response envelope + ETag/caching | D1.7 | T32–T37 | 3.0 | BE |
| P1-T40 | API OpenAPI spec + contract tests + error-body conformance | D1.7 | T39 | 2.0 | BE |
| P1-T41 | Performance benchmarks + threshold gates (table in D1.6) | D1.6 | T35 | 2.0 | QA |
| P1-T42 | ADR-0112/0113 | ADR | T35,T37 | 1.0 | DOC |

### Sprint 1.4 — Tools, Citations, CLI, Doctor (Week 5)

| ID | Task | Deliverable | Depends | Est | Role |
|---|---|---|---|---|---|
| P1-T43 | `ToolResult` contract + `ReproducibilityData` + checksum computation | D1.8 | T32 | 2.5 | BE |
| P1-T44 | Minimal tool registry + `quran.get_ayah` + `quran.get_context` | D1.8 | T43 | 2.0 | BE |
| P1-T45 | Tool contract conformance tests + no-fabrication tests | D1.13 | T44 | 1.5 | QA |
| P1-T46 | `citations` crate: `Citation`, resolver, `verify_quotation`, persistence | D1.9 | T32,T15 | 3.0 | BE |
| P1-T47 | Deep-link format + resolver endpoint + round-trip tests | D1.9 | T46 | 1.5 | BE |
| P1-T48 | CLI `quran get/context/surah/division/resolve` + `--json` | D1.10 | T32 | 2.5 | BE |
| P1-T49 | CLI `quran edition/import/validate/diff/activate/rollback` | D1.10 | T28 | 2.5 | BE |
| P1-T50 | CLI snapshot tests incl. RTL/Arabic terminal output sanity | D1.13 | T48,T49 | 1.5 | QA |
| P1-T51 | `doctor --quran` checks (19 checks) + `--deep` mode | D1.11 | T23,T13 | 3.0 | BE |
| P1-T52 | `doctor --quran --json` schema + CI consumption | D1.11 | T51 | 1.0 | BE |
| P1-T53 | ADR-0111 | ADR | T47 | 0.5 | DOC |

### Sprint 1.5 — Debug Reader, Hardening, Exit (Week 6)

| ID | Task | Deliverable | Depends | Est | Role |
|---|---|---|---|---|---|
| P1-T54 | Debug reader page (RTL, web font, ayah markers) | D1.12 | T39 | 1.5 | BE |
| P1-T55 | Editorial review pass: reviewer verifies sampled text against printed muṣḥaf; record `verified_by` | AC | T54 | 3.0 | EDIT |
| P1-T56 | Golden-set expansion to all §5.2 edge cases | D1.13 | T55 | 2.0 | QA |
| P1-T57 | Property-test suite (§5.4) | D1.13 | T32 | 2.0 | QA |
| P1-T58 | Full-corpus soak: import → validate → activate → 10k random lookups → doctor --deep | D1.13 | all | 2.0 | QA |
| P1-T59 | Docs: corpus architecture, import runbook, rollback runbook, citation spec, adapter authoring guide | D1.14 | all | 3.0 | DOC |
| P1-T60 | Phase-1 exit gate review + handoff to Phase 2 | — | all | 1.5 | all |

---

## 9. Acceptance Criteria (Phase 1 Exit Gate)

| ID | Criterion | Verification |
|---|---|---|
| AC-P1-01 | ADR-0101 is `Accepted` with a licensed, editorially signed-off edition (or a documented user-supplied policy) | ADR review |
| AC-P1-02 | `qai quran import <manifest>` imports the edition to `Staged` and produces a `ValidationReport` with zero `Fatal` findings | scripted run |
| AC-P1-03 | Import is **not** activatable by the importer; activation requires `qai quran activate` with a human approval, producing an `approvals` row and an audit event | negative + positive test |
| AC-P1-04 | Validation rules QV-001…QV-028 are all implemented, and each of the 16 adversarial fixtures is rejected with the **specific** expected rule ID | adversarial suite |
| AC-P1-05 | For every ayah in the corpus, `reconstruct(tokens, separators)` equals the stored text byte-for-byte | full-corpus test |
| AC-P1-06 | For every token, `ayah.text[byte_start..byte_end] == surface` | full-corpus test |
| AC-P1-07 | Recomputed `text_hash`, `structure_hash`, `token_order_hash` from DB rows match the values stored at import | `doctor --quran --deep` |
| AC-P1-08 | `UPDATE`/`DELETE` on `quran_ayahs` / `quran_tokens` aborts with the documented error codes | trigger tests |
| AC-P1-09 | No public API exists to write canonical ayah rows without a `CanonicalChangeSession` derived from an `ApprovalToken` | API-surface test + code review |
| AC-P1-10 | Killing the import process at each of the 13 checkpoints leaves the active edition unchanged in all 13 cases, and `qai job retry` completes the import correctly | crash matrix |
| AC-P1-11 | Cancelling an import removes all `quran_stg_*` rows for that run and records the cancellation | cancellation test |
| AC-P1-12 | The reference grammar parses all 300 golden references correctly and returns coded errors (never panics) for all malformed inputs | golden set + fuzz |
| AC-P1-13 | `parse(serialize(ref)) == ref` for all reference variants | property test |
| AC-P1
