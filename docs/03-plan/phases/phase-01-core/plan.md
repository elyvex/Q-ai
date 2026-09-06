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


# Q-ai — Phase 0 Development Plan: Foundations & Provenance

**Plan Version:** 1.0.0
**PRD Baseline:** Q-ai PRD v0.3.2
**Phase ID:** P0
**Phase Name:** Foundations and Provenance
**PRD Traceability:** §43 (Phase 0), §88 (item 1), §6, §22, §25.13, §32, §33, §37, §39, §41, §50, §55, §58, §82, §84, §87
**Depends On:** — (first phase)
**Blocks:** Phase 1 (Canonical Quran Core), and all later phases
**Target Duration:** 5 calendar weeks (≈ 12–14 engineer-weeks, 2–3 engineers)
**Status:** Not Started

---

## 0. How To Read This Plan

- **D0.x** = Deliverable (a shippable, testable unit).
- **P0-Tnn** = Task (a work item on the board).
- **AC-P0-nn** = Acceptance Criterion (phase exit gate).
- **ADR-nnn** = Architecture Decision Record required *inside* this phase.
- Every task lists: `Deliverable`, `Depends`, `Estimate (engineer-days)`, `Owner role`.

Nothing in Phase 0 touches religious corpora. Phase 0 builds the **chassis**: types,
storage, provenance, jobs, config, secrets, audit, CLI, and CI. If Phase 0 is weak,
every integrity guarantee in §46/§56/§93 becomes unenforceable later.

---

## 1. Phase Objective

Deliver a production-shaped Rust workspace that can:

1. Represent **all five trust layers** (§6: A canonical, B publisher metadata, C scholarly
   annotation, D computational annotation, E user/AI notes) as first-class, non-mergeable data.
2. Persist, migrate, and version data in SQLite behind a storage abstraction that can later
   become PostgreSQL (§32.1) without domain changes.
3. Record **provenance and audit** for every mutation, with source-version pinning (§39, §82).
4. Run **cancellable, resumable, idempotent background jobs** (§34, §41, §54).
5. Load and validate **configuration with correct precedence** and handle **secrets** without
   ever leaking them (§25.13).
6. Expose a `qai` CLI skeleton and a `qai doctor` diagnostic that never mutates data (§50).
7. Define the **source manifest schema** and source lifecycle state machine (§22).
8. Enforce a **security baseline** and a **deny-by-default** posture from day one (§37, §84).

**One-sentence goal:** *A trustworthy, observable, testable skeleton with immutable-canonical
and provenance semantics already enforced by types and the database — before any Quran byte
is imported.*

---

## 2. Scope

### 2.1 In Scope

| # | Item | PRD |
|---|------|-----|
| 1 | Cargo workspace, crate boundaries, MSRV, lint gates | §33, §87, §25.13 |
| 2 | Core domain model & newtype IDs | §33, §55 |
| 3 | Typed error model (`thiserror` + `anyhow` boundary) | §25.13 |
| 4 | Layered configuration (CLI > Env > File > Defaults) | §25.13 |
| 5 | Secret handling (env / OS keychain / encrypted file) | §25.13, §37 |
| 6 | Storage abstraction + SQLite implementation | §32.1 |
| 7 | Migration framework (forward + reversible where feasible) | §43 P0 |
| 8 | Structured logging, metrics, tracing spans | §39, §25.13 |
| 9 | Durable job system with leases, cancellation, checkpoints | §41, §54 |
| 10 | Source catalog tables + manifest schema + state machine | §22.1–22.6 |
| 11 | Provenance model (Layers A–E, annotation metadata) | §6 |
| 12 | Audit model (append-only, hash-chained, secret-free) | §82 |
| 13 | `qai` CLI skeleton, global flags, `--json`, error codes | §25.4, §26 |
| 14 | `qai doctor` core checks + `--json` + `--repair-preview` | §50 |
| 15 | Security baseline (localhost bind, SSRF guard, zip-slip guard, path guard) | §37, §84 |
| 16 | Test harness, fixtures, CI pipeline, coverage gates | §25.13, §58 |
| 17 | ADR process, docs skeleton, `.env.example`, Dockerfile stub | §25.13, §48, §60 |

### 2.2 Explicitly Out of Scope (Phase 0)

- Any Quran, hadith, tafsir, or scripture text or schema → Phase 1/5/8.
- Full-text index, vector store, embeddings, graph store → Phase 2/3/7.
- LLM / model provider adapters → Phase 9.
- Tool runtime, sandbox, agents → Phase 9/10.
- Web GUI and TUI screens → Phase 4/11 (only a `serve` stub returning health).
- Internet source **download** (the *schema* and *state machine* are in scope; the network
  fetcher lands in Phase 10) → Phase 10.
- Authentication/RBAC beyond a local single-principal model → Phase 11.

### 2.3 Deliberate Design Constraints Adopted Now

1. **Canonical immutability is a type-level property**, not a code convention: canonical rows
   are written only through a `CanonicalWriter` that requires an `ApprovalToken`.
2. **No domain crate may depend on** `api`, `server`, `tui`, `cli`, or any concrete provider (§33).
3. **Every derived artifact records the version of everything it was derived from** (§76).
4. **Deny-by-default**: permissions, network, filesystem, and side effects default to empty sets.
5. **Nothing that can modify data runs inside `doctor`** (§50).

---

## 3. Architecture: Crate Graph For Phase 0

```text
                     ┌────────────┐
                     │   config   │  (no deps on domain)
                     └─────┬──────┘
                           │
┌──────────┐      ┌────────▼────────┐      ┌────────────┐
│  domain  │◄─────│   application   │─────►│   audit    │
└────┬─────┘      └───┬────────┬────┘      └─────┬──────┘
     │                │        │                 │
     │          ┌─────▼──┐  ┌──▼─────┐     ┌─────▼──────┐
     │          │ jobs   │  │sources │     │ provenance │
     │          └─────┬──┘  └──┬─────┘     └─────┬──────┘
     │                │        │                 │
     │            ┌───▼────────▼─────────────────▼───┐
     └───────────►│            storage               │
                  │  (traits) + storage-sqlite       │
                  └──────────────┬───────────────────┘
                                 │
                          ┌──────▼──────┐
                          │  observability │
                          └──────┬──────┘
                                 │
                    ┌────────────▼────────────┐
                    │   cli   │  server(stub) │
                    └─────────────────────────┘
```

**Dependency rule (enforced in CI by `cargo-deny` + a custom `xtask arch-check`):**

```text
domain          -> (serde, thiserror, time, uuid) only. No I/O, no async runtime.
application     -> domain, storage(traits), provenance, audit, jobs, sources, config
storage         -> domain            (traits + errors only)
storage-sqlite  -> storage, domain, sqlx
provenance      -> domain, storage
audit           -> domain, storage
jobs            -> domain, storage, observability
sources         -> domain, storage, provenance
cli             -> application, config, observability
server          -> application, config, observability
observability   -> (tracing, metrics) only
```

### 3.1 Workspace Layout Created In Phase 0

Only the crates below are *created* in Phase 0. The remaining crates from PRD §87 are
declared as empty placeholders with a `//! Phase N` doc comment so the workspace graph is
visible from day one and reviewers can see where future code belongs.

```text
q-ai/
├── Cargo.toml                  # workspace, [workspace.dependencies], [workspace.lints]
├── rust-toolchain.toml         # pinned toolchain + components
├── deny.toml                   # cargo-deny: licenses, advisories, bans, sources
├── clippy.toml
├── rustfmt.toml
├── .env.example
├── xtask/                      # cargo xtask: arch-check, migrate-check, ci, gen-schema
├── crates/
│   ├── domain/                 # ★ Phase 0
│   ├── application/            # ★ Phase 0
│   ├── config/                 # ★ Phase 0
│   ├── observability/          # ★ Phase 0
│   ├── storage/                # ★ Phase 0 (traits)
│   ├── storage-sqlite/         # ★ Phase 0
│   ├── jobs/                   # ★ Phase 0
│   ├── provenance/             # ★ Phase 0
│   ├── audit/                  # ★ Phase 0
│   ├── sources/                # ★ Phase 0 (catalog + manifest; no network)
│   ├── cli/                    # ★ Phase 0
│   ├── server/                 # ★ Phase 0 (health + /readyz only)
│   ├── quran-core/             # placeholder — Phase 1
│   ├── quran-corpus/           # placeholder — Phase 1
│   ├── quran-normalization/    # placeholder — Phase 2
│   ├── quran-morphology/       # placeholder — Phase 2
│   ├── quran-search/           # placeholder — Phase 2
│   ├── quran-graph/            # placeholder — Phase 3
│   ├── hadith-core/            # placeholder — Phase 5
│   ├── ... (per PRD §87)
│   └── citations/              # placeholder — Phase 1 (resolver v1)
├── migrations/
│   └── sqlite/
├── fixtures/
├── docs/
│   ├── plans/
│   ├── architecture/
│   └── runbooks/
├── adr/
│   ├── 0000-adr-template.md
│   └── ...
├── examples/
└── tests/                      # workspace-level integration + e2e
```

---

## 4. Deliverables

### D0.1 — Workspace, Toolchain & Build Tooling

**PRD:** §25.13, §33, §58

**Contents**

- Workspace `Cargo.toml` with `resolver = "2"`, centralized `[workspace.dependencies]`,
  and `[workspace.lints]` (`unsafe_code = "forbid"` in all crates except an explicitly
  allowlisted set — Phase 0 has none).
- `rust-toolchain.toml`: pinned stable version, components `rustfmt, clippy`.
- `deny.toml`: license allowlist (MIT/Apache-2.0/BSD/ISC/Unicode-DFS), deny GPL/AGPL,
  advisory DB check, duplicate-version bans, registry allowlist.
- `xtask` binary providing:
  - `xtask arch-check` — parses `Cargo.toml` graph, fails if a forbidden edge exists.
  - `xtask ci` — runs fmt, clippy `-D warnings`, test, deny, arch-check, migrate-check.
  - `xtask migrate-check` — asserts migrations are append-only and checksums unchanged.
  - `xtask gen-schema` — emits JSON Schema for config + manifests into `docs/schemas/`.
- `Makefile` / `justfile` convenience wrappers.
- Dockerfile (multi-stage, distroless runtime) + `docker-compose.yml` stub (app only;
  Postgres/Qdrant services commented for Phase 12).

**Acceptance:** `cargo xtask ci` green on a clean checkout on Linux, macOS, Windows.

---

### D0.2 — Core Domain Model

**PRD:** §6, §33, §55

Pure, dependency-light types. No async, no I/O.

```rust
// crates/domain/src/ids.rs
macro_rules! typed_id { /* Uuid-backed newtype + Display + FromStr + serde */ }

typed_id!(SourceId);
typed_id!(SourceVersionId);
typed_id!(EditionId);
typed_id!(DocumentId);
typed_id!(JobId);
typed_id!(RunId);
typed_id!(WorkspaceId);
typed_id!(PrincipalId);
typed_id!(ProvenanceId);
typed_id!(AuditEventId);
typed_id!(ApprovalId);
```

```rust
// crates/domain/src/trust.rs  (PRD §6, §23)

/// PRD §6 — the five authoritative data layers. Non-mergeable by construction.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataLayer {
    /// §6.1 Immutable canonical source text for a specific source version.
    CanonicalSource,
    /// §6.2 Publisher / dataset supplied metadata (numbering, pagination, grading).
    PublisherMetadata,
    /// §6.3 Human scholarly annotation (tafsir claim, root analysis, evaluation).
    ScholarlyAnnotation,
    /// §6.4 Machine-generated annotation. Requires ComputationalMetadata.
    ComputationalAnnotation,
    /// §6.5 User note or AI note. Never rendered as canonical.
    UserOrAiNote,
}

/// PRD §23 — trust profile. Influences filtering, ranking, and whether a source
/// may support a definitive claim.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustLevel {
    Quarantined = 0,
    MachineGenerated = 1,
    UserProvided = 2,
    ImportedUnverified = 3,
    CommunityReviewed = 4,
    ScholarReviewed = 5,
    PublisherVerified = 6,
    CanonicalVerified = 7,
}

impl TrustLevel {
    /// PRD §23: popularity or vector similarity must never imply authority.
    pub fn may_support_definitive_claim(self) -> bool {
        self >= TrustLevel::ScholarReviewed
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    Unverified,
    NeedsReview,
    HumanVerified,
    Rejected,
    Superseded,
}
```

```rust
// crates/domain/src/hashing.rs
/// Content hash with an explicit algorithm tag so future algorithm changes are detectable.
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ContentHash {
    pub algorithm: HashAlgorithm, // Sha256 | Blake3
    pub hex: String,              // lowercase hex, validated length
}

/// Canonical serialization used for all checksums (stable key order, NFC-preserving,
/// LF newlines, no BOM). Defined once here so hashes are reproducible forever.
pub fn canonical_json_bytes<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, DomainError>;
```

```rust
// crates/domain/src/licensing.rs  (PRD §38)
pub enum LicenseStatus {
    PublicDomain, OpenLicense, PermissionGranted, UserOwned,
    MetadataOnly, Unknown, Restricted,
}
pub struct LicenseRecord {
    pub status: LicenseStatus,
    pub spdx_id: Option<String>,
    pub name: Option<String>,
    pub url: Option<String>,
    pub attribution_required: bool,
    pub redistribution_allowed: bool,
    pub export_allowed: bool,
    pub notes: Option<String>,
}
```

```rust
// crates/domain/src/version.rs
/// Records every version that a derived artifact depends on (PRD §76, §82).
pub struct DerivationVersions {
    pub source_version_id: SourceVersionId,
    pub parser_version: SemVer,
    pub normalizer_version: SemVer,
    pub chunker_version: Option<SemVer>,
    pub embedding_model_version: Option<String>,
    pub graph_builder_version: Option<SemVer>,
    pub schema_version: u32,
}
```

**Also in D0.2:** `SemVer`, `Timestamp` (UTC, RFC3339, microsecond precision),
`Language` (BCP-47 validated), `SideEffectClass` (§29), `Confidence(f32 in 0.0..=1.0)`.

**Tests:** property tests for ID round-trip, `canonical_json_bytes` stability across
map-insertion orders, `ContentHash` parsing rejection of bad hex/length.

---

### D0.3 — Typed Error Model

**PRD:** §25.13, §58

Requirement: every error answers *what / why / where / how to fix / next command*.

```rust
// crates/domain/src/error.rs
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ErrorCode(pub &'static str); // e.g. "QAI-CFG-0007"

/// Every library error type in the workspace implements this.
pub trait Diagnostic {
    fn code(&self) -> ErrorCode;
    fn summary(&self) -> String;         // what happened
    fn cause_chain(&self) -> Vec<String>;// why
    fn location(&self) -> Option<String>;// where (file/key/row/source id)
    fn remedy(&self) -> Option<String>;  // how to fix
    fn next_command(&self) -> Option<String>; // e.g. "qai doctor --sources"
    fn is_retryable(&self) -> bool;
    fn redacted(&self) -> bool;          // true if message already secret-scrubbed
}
```

**Code namespace registry** (documented in `docs/architecture/error-codes.md`,
validated by a unit test that all codes are unique):

```text
QAI-CFG-nnnn  configuration
QAI-SEC-nnnn  secrets / security
QAI-DB-nnnn   storage / migration
QAI-JOB-nnnn  job system
QAI-SRC-nnnn  source catalog / manifest
QAI-PROV-nnnn provenance
QAI-AUD-nnnn  audit
QAI-CLI-nnnn  CLI usage
QAI-QUR-nnnn  Quran corpus            (reserved, Phase 1)
QAI-NORM-nnnn normalization           (reserved, Phase 2)
QAI-IDX-nnnn  indexes                 (reserved, Phase 2)
```

CLI renders human form and `--json` form:

```json
{
  "error": {
    "code": "QAI-CFG-0007",
    "summary": "Configuration key `storage.sqlite.path` points to a non-writable directory",
    "location": "config file: /home/u/.config/qai/config.toml:14",
    "why": ["parent directory /var/lib/qai is not writable by uid 1000"],
    "remedy": "Choose a writable path or run: qai config set storage.sqlite.path ~/.local/share/qai/qai.db",
    "next_command": "qai doctor"
  }
}
```

**Tests:** unique-code test; a `Diagnostic` compliance test per error enum; snapshot tests
of rendered human + JSON output.

---

### D0.4 — Configuration System

**PRD:** §25.13

- Formats: TOML file + environment variables + CLI flags.
- Precedence (strict): **CLI > Env > Config file > Defaults**.
- Every field records its **provenance** (`ValueOrigin::{Default, File(path,line), Env(var), Cli(flag)}`)
  so `qai config show --explain` can print where each value came from.
- Validation at load **and** on every change (hot reload via `SIGHUP` / API is Phase 11;
  Phase 0 implements the `validate()` contract and a `reload()` seam).
- Redaction: `Secret<String>` fields serialize as `"***"` in every representation.

**Default config (`examples/config/default.toml`, also emitted by `qai config show --defaults`):**

```toml
[app]
mode = "local"                 # local | server
data_dir = "~/.local/share/qai"
locale = "en"

[server]
bind = "127.0.0.1"             # PRD §37: localhost by default
port = 8737
tls = "disabled"               # disabled | required   (required outside localhost, §84)
require_auth_outside_localhost = true
max_request_bytes = 10485760
request_timeout_ms = 30000
concurrency_limit = 128

[storage]
backend = "sqlite"             # sqlite | postgres (Phase 12)

[storage.sqlite]
path = "${app.data_dir}/qai.db"
journal_mode = "wal"
synchronous = "full"           # canonical integrity over write speed
busy_timeout_ms = 5000
foreign_keys = true
max_connections = 8
read_only_pool = true          # PRD §84 read-only credentials by default

[storage.objects]
backend = "filesystem"
root = "${app.data_dir}/objects"
max_file_bytes = 536870912

[secrets]
backend = "env"                # env | keychain | encrypted_file
encrypted_file_path = "${app.data_dir}/secrets.age"

[jobs]
workers = 4
poll_interval_ms = 250
lease_seconds = 60
max_attempts = 5
backoff_base_ms = 500
backoff_max_ms = 60000
backoff_jitter = 0.2

[logging]
level = "info"                 # trace|debug|info|warn|error
format = "text"                # text | json
file = ""                      # empty = stderr only
redact_secrets = true

[telemetry]
enabled = false                # PRD §39: no external telemetry without opt-in
otlp_endpoint = ""
metrics_enabled = true

[security]
allow_network_egress = false   # deny-by-default (§84)
domain_allowlist = []
ssrf_block_private_ranges = true
max_download_bytes = 268435456
max_archive_entries = 20000
max_archive_expansion_ratio = 100
follow_symlinks = false

[policy]
tool_execution_default = "deny"        # §29, §84
command_execution_enabled = false
canonical_write_requires_approval = true
```

**Env mapping:** `QAI__SERVER__PORT=9000`, `QAI__STORAGE__SQLITE__PATH=/tmp/x.db`
(double underscore = nesting; documented and tested).

**Tests:** precedence matrix test (16 cases), `${...}` interpolation cycle detection,
rejection of `bind = "0.0.0.0"` when `tls = "disabled"` and `require_auth_outside_localhost`
cannot be satisfied, secret redaction snapshot, JSON-Schema generation matches struct.

---

### D0.5 — Secret Management

**PRD:** §25.13, §37, §84

```rust
pub trait SecretStore: Send + Sync {
    async fn get(&self, r: &SecretRef) -> Result<Secret<String>, SecretError>;
    async fn put(&self, r: &SecretRef, v: Secret<String>) -> Result<(), SecretError>;
    async fn delete(&self, r: &SecretRef) -> Result<(), SecretError>;
    async fn list_refs(&self) -> Result<Vec<SecretRef>, SecretError>; // refs only, never values
    fn backend_name(&self) -> &'static str;
}
```

- `SecretRef` is what is stored in the database/config (`secret://keychain/openai/default`).
  **Secret values are never stored in SQLite** in Phase 0.
- Backends: `EnvSecretStore`, `KeychainSecretStore` (Secret Service / macOS Keychain /
  Windows Credential Manager), `EncryptedFileSecretStore` (age/XChaCha20-Poly1305,
  passphrase from env or interactive prompt).
- `Secret<T>` wrapper: `Debug`/`Display`/`Serialize` all emit `***`; `zeroize` on drop.
- **Global redaction layer** in the tracing subscriber: regex + known-secret-value matching,
  applied to logs, spans, audit records, and CLI output (§37).

**Tests:** a "secret never escapes" test suite that (a) puts a known sentinel value into every
backend, (b) exercises log emission, error formatting, `config show`, `doctor --json`, and
audit-event serialization, then (c) asserts the sentinel appears in **zero** output bytes.
This test is a permanent CI gate reused by all future phases.

---

### D0.6 — Storage Abstraction + SQLite Implementation

**PRD:** §32.1, §55, §56

```rust
// crates/storage/src/lib.rs
pub trait UnitOfWork: Send {
    fn sources(&mut self) -> &mut dyn SourceRepository;
    fn provenance(&mut self) -> &mut dyn ProvenanceRepository;
    fn audit(&mut self) -> &mut dyn AuditRepository;
    fn jobs(&mut self) -> &mut dyn JobRepository;
    fn settings(&mut self) -> &mut dyn SettingsRepository;
    async fn commit(self: Box<Self>) -> Result<(), StorageError>;
    async fn rollback(self: Box<Self>) -> Result<(), StorageError>;
}

pub trait Database: Send + Sync {
    /// Read-only handle. Uses the read-only pool (§84).
    async fn read(&self) -> Result<Box<dyn ReadTx>, StorageError>;
    /// Writable unit of work. Serialized transaction semantics.
    async fn write(&self) -> Result<Box<dyn UnitOfWork>, StorageError>;
    async fn health(&self) -> Result<DbHealth, StorageError>;
    fn schema_version(&self) -> u32;
    fn backend(&self) -> DbBackend;
}
```

**SQLite specifics**

- `sqlx` with compile-time-checked queries; WAL; `synchronous=FULL`; `foreign_keys=ON`;
  `busy_timeout`.
- Two pools: a write pool (`max_connections = 1`, serialized) and a read pool (N connections,
  `query_only = ON`).
- All timestamps stored as `TEXT` RFC3339 UTC **and** an `INTEGER` epoch-microseconds column
  where indexing/sorting matters.
- All hashes stored as `TEXT` `"<alg>:<hex>"`.
- `CHECK` constraints encode domain invariants (see §6 below) so a bug in Rust cannot
  produce an illegal row.

**Portability rule:** no SQLite-only SQL in repository code paths shared with Postgres;
dialect-specific SQL lives behind `#[cfg]`-free trait impls per backend crate.

---

### D0.7 — Migration Framework

**PRD:** §43 P0, §56, §76

- Migrations are **append-only, numbered, checksummed** SQL files:
  `migrations/sqlite/0001_core.up.sql` (+ optional `.down.sql`).
- `schema_migrations(version, name, checksum, applied_at, applied_by, duration_ms)`.
- On startup: verify every applied migration's checksum matches the file. Mismatch =
  hard fail with `QAI-DB-0003` (protects against silently edited history).
- `qai db migrate` (apply), `qai db status`, `qai db verify`, `qai db plan` (dry-run diff).
- A `xtask migrate-check` CI gate: new migrations must be additive; editing an existing
  migration fails CI.
- **Reversibility policy:** `.down.sql` is required for all non-canonical tables. Canonical
  tables (created in Phase 1) may be forward-only; deactivation is used instead of deletion (§76).

---

### D0.8 — Observability

**PRD:** §39, §25.13

- `tracing` + `tracing-subscriber` with an env-filter and two formatters (text, JSON).
- Span conventions (documented + macro-enforced):
  `qai.component`, `qai.operation`, `qai.job_id`, `qai.run_id`, `qai.source_id`,
  `qai.source_version`, `qai.principal_id`, `qai.workspace_id`, `qai.duration_ms`,
  `qai.outcome`.
- Metrics registry (`metrics` crate) with a **named** metric catalog so Phase 12 dashboards
  are stable:

```text
qai_jobs_enqueued_total{kind}
qai_jobs_completed_total{kind,outcome}
qai_job_duration_seconds{kind}            (histogram)
qai_db_query_duration_seconds{op}         (histogram)
qai_db_pool_in_use{pool}
qai_audit_events_total{action}
qai_config_reloads_total{outcome}
qai_errors_total{code}
qai_doctor_checks_total{check,status}
```

- OpenTelemetry OTLP exporter behind `telemetry.enabled = false` (opt-in only, §39).
- **Privacy gate:** a compile-time-tested list of fields forbidden from telemetry export
  (query text, prompt text, document content, research questions, model responses).

---

### D0.9 — Job System

**PRD:** §41, §54, §76, §85

Durable, DB-backed queue (no external broker in local mode).

```rust
pub struct JobRecord {
    pub id: JobId,
    pub kind: JobKind,              // string-typed, registry-validated
    pub payload: serde_json::Value, // schema-validated per kind
    pub idempotency_key: Option<String>, // UNIQUE with kind
    pub state: JobState,
    pub priority: i32,
    pub attempts: u32,
    pub max_attempts: u32,
    pub available_at: Timestamp,
    pub lease_owner: Option<String>,
    pub lease_expires_at: Option<Timestamp>,
    pub checkpoint: Option<serde_json::Value>, // resume point (§54)
    pub cancel_requested: bool,
    pub progress: Option<JobProgress>,          // {stage, done, total, message}
    pub parent_job_id: Option<JobId>,
    pub created_by: PrincipalId,
    pub versions: DerivationVersions,           // §76
}

pub enum JobState {
    Queued, Leased, Running, Checkpointed,
    Succeeded, Failed, Cancelled, Interrupted, DeadLettered,
}

#[async_trait]
pub trait JobHandler: Send + Sync {
    fn kind(&self) -> JobKind;
    fn payload_schema(&self) -> &'static str;   // JSON Schema
    fn is_idempotent(&self) -> bool;
    async fn run(&self, ctx: JobContext, payload: serde_json::Value)
        -> Result<JobOutcome, JobError>;
}
```

**`JobContext` provides:** `CancellationToken`, `progress(stage, done, total)`,
`checkpoint(value)`, `deadline`, `tracing::Span`, `UnitOfWork` factory, and a
`heartbeat()` that renews the lease.

**Guarantees implemented and tested in Phase 0**

| Guarantee | Mechanism | Test |
|---|---|---|
| At-least-once with idempotency | `UNIQUE(kind, idempotency_key)` + handler idempotency flag | duplicate-enqueue test |
| Crash recovery | expired lease reclaim; `Running` → `Interrupted` on startup scan (§85) | kill-worker test |
| Cancellation propagates | `cancel_requested` polled + `CancellationToken` | cancel-mid-run test |
| Resume from checkpoint | `checkpoint` JSON replayed to handler | resume test |
| No partial activation | handlers must write to staging and flip a pointer in one tx (§85) | atomicity test |
| Bounded retries + jitter | `attempts`, exponential backoff, `DeadLettered` | backoff distribution test |
| Non-idempotent ops never auto-retry | handler flag gate | policy test |

**Phase 0 job kinds (real, useful):**
`system.integrity_scan`, `system.vacuum`, `source.validate_manifest`,
`source.compute_hashes`, `audit.verify_chain`, `system.noop_test`.

---

### D0.10 — Source Catalog & Manifest Schema

**PRD:** §22 (all), §34, §38

Phase 0 delivers the **catalog data model, manifest format, validators, and state machine**.
Network discovery/download is Phase 10; Phase 0 supports **local manifest import** which is
exactly what Phase 1 needs to import a Quran edition.

**Source state machine (§22.2) — legal transitions only, enforced in SQL + Rust:**

```text
Discovered ──► PendingReview ──► Downloading ──► Downloaded ──► Validating
                    │                 │              │              │
                    │                 ▼              ▼              ├─► ValidationFailed ─► Quarantined
                    │             Quarantined    Quarantined        │
                    ▼                                              ▼
                Quarantined                                      Staged
                                                                    │
                                                       ┌────────────┴──────────┐
                                                       ▼                       ▼
                                                   Approved               Quarantined
                                                       │
                                                       ▼
                                                   Indexing ──► Active ──► Deprecated ──► Removed
                                                       │            │
                                                       └──► ValidationFailed
                                                                    └──► Quarantined
```

Rules encoded now (used from Phase 1 onward):
- `Approved` requires: license status ≠ `Unknown`, a verified `ContentHash`, a completed
  structural validation report, and a human `ApprovalRecord` (§22.3: *no internet source may
  become active solely because an LLM recommended it*).
- `Active` is set only by an atomic pointer flip (§85).
- Exactly one `Active` version per `(source_id, role)`; enforced by a partial unique index.

**Manifest schema v1** (`docs/schemas/source-manifest.v1.schema.json`, also see §22.4):

```json
{
  "manifest_version": "1.0.0",
  "catalog_version": "1.4.0",
  "generated_at": "2026-01-15T10:00:00Z",
  "publisher": {
    "name": "Example Corpus Project",
    "url": "https://example.org",
    "signing_key_id": "ed25519:9f2c..."
  },
  "signature": {
    "algorithm": "ed25519",
    "key_id": "ed25519:9f2c...",
    "value": "base64...",
    "signed_fields": "canonical_json_without_signature"
  },
  "sources": [
    {
      "id": "hafs-uthmani",
      "title": "Quran — Hafs, Uthmani script",
      "alternate_titles": [],
      "content_type": "quran_edition",
      "version": "1.0.0",
      "schema_version": 1,
      "language": "ar",
      "tradition": null,
      "school": null,
      "authors": [],
      "publisher": "…",
      "edition": "…",
      "publication_date": null,
      "license": {
        "status": "PublicDomain",
        "spdx_id": null,
        "name": "…",
        "url": "…",
        "attribution_required": true,
        "redistribution_allowed": true,
        "export_allowed": true
      },
      "trust_level": "PublisherVerified",
      "genealogy": {
        "derives_from_source_id": null,
        "derivation_type": "original",
        "derivation_language": null,
        "derivation_date": null,
        "derivation_note": null
      },
      "files": [
        {
          "role": "primary",
          "path": "quran-uthmani.json",
          "url": "https://example.org/quran-uthmani.json",
          "format": "json",
          "media_type": "application/json",
          "bytes": 1234567,
          "sha256": "…",
          "encoding": "utf-8",
          "unicode_normalization": "nfc"
        }
      ],
      "expected_structure": {
        "validator": "quran_edition_v1",
        "parameters": { "surah_count": 114, "numbering_scheme": "hafs" }
      },
      "notes": ""
    }
  ]
}
```

**Deliverables in code**

- `sources::manifest` — parse, JSON-Schema validate, semantic validate, canonical-JSON
  re-serialize, signature verify (ed25519; `signature = null` allowed only when
  `security.allow_unsigned_manifests = true`, default `false` for remote / `true` for local files).
- `sources::genealogy` — transitive lineage resolution + rendering (§22.6):
  *"English translation (2024) of an Arabic summary (1990) of the original manuscript (1200)."*
  Includes cycle detection.
- `sources::catalog` — CRUD, state machine, version listing, license reporting.
- `sources::validation` — pluggable `StructureValidator` registry (Phase 1 registers
  `quran_edition_v1`).
- `sources::diff` — version-diff **framework** (line/record-level, produces a
  `DifferenceReport`); corpus-specific differs register later.
- Local ingest: `qai source import <manifest-path>` → Discovered → Validating → Staged.

---

### D0.11 — Provenance Model

**PRD:** §6, §6.4, §10.3, §39, §82

One universal provenance record referenced by every non-canonical assertion in the system.

```rust
pub struct ProvenanceRecord {
    pub id: ProvenanceId,
    pub layer: DataLayer,                      // §6
    pub subject: SubjectRef,                   // what this describes (typed URN)
    pub attribution: Attribution,
    pub source_version_id: Option<SourceVersionId>,
    pub source_location: Option<SourceLocation>,// exact location (§21.2)
    pub trust_level: TrustLevel,
    pub verification: VerificationStatus,
    pub confidence: Option<Confidence>,
    pub versions: DerivationVersions,          // §76
    pub created_at: Timestamp,
    pub created_by: PrincipalId,
    pub reviewed_by: Option<PrincipalId>,
    pub reviewed_at: Option<Timestamp>,
    pub review_note: Option<String>,
    pub superseded_by: Option<ProvenanceId>,
}

pub enum Attribution {
    /// §6.1/§6.2 — comes straight from the source dataset.
    Dataset { source_id: SourceId, dataset_name: String, dataset_version: String },
    /// §6.3 — a named human scholar / work.
    Scholar { name: String, school: Option<String>, work: Option<String>, edition: Option<String> },
    /// §6.4 — an algorithm or model.
    Computational { algorithm: String, version: SemVer, model: Option<String>, parameters_hash: ContentHash },
    /// §6.5 — a user of this Q-ai instance.
    User { principal_id: PrincipalId },
}

pub struct SourceLocation {
    pub canonical_reference: Option<String>, // "quran:2:255" (Phase 1 fills this)
    pub volume: Option<String>,
    pub book: Option<String>,
    pub chapter: Option<String>,
    pub page: Option<String>,
    pub record_number: Option<String>,
    pub char_range: Option<(u32, u32)>,
    pub quoted_text_hash: Option<ContentHash>, // enables quotation verification (§35.3)
}
```

**Type-level invariants (compile-time + DB `CHECK`):**

1. `layer == ComputationalAnnotation` ⇒ `Attribution::Computational` **and**
   `confidence.is_some()` **and** `verification != HumanVerified` unless
   `reviewed_by.is_some()` (§10.6: no computational suggestion becomes verified without an
   explicit human decision).
2. `layer == CanonicalSource` ⇒ `source_version_id.is_some()` and rows are insert-only.
3. `layer == ScholarlyAnnotation` ⇒ `Attribution::Scholar` with a non-empty name.
4. Any record whose subject is a canonical row **cannot** be written through the normal
   repository; requires `CanonicalWriter` + `ApprovalToken`.

**Canonical write guard (the core integrity mechanism, PRD §7.3, §84, §92:43–46):**

```rust
/// Only obtainable from an ApprovalRecord persisted by a human principal with the
/// `canonical:approve` permission. Not constructible by agents, tools, or importers.
pub struct ApprovalToken { /* private fields */ }

pub trait CanonicalWriter {
    fn begin_canonical_change(
        &self,
        token: &ApprovalToken,
        change: CanonicalChangeRequest, // includes DifferenceReport + checksums
    ) -> Result<CanonicalChangeSession, ProvenanceError>;
}
```

`CanonicalChangeRequest` **must** contain: new source version, checksum validation result,
structural validation result, difference report, and approver identity — mirroring §7.3
exactly. A unit test asserts each missing field is rejected.

---

### D0.12 — Audit Model

**PRD:** §82, §84, §91

- Append-only `audit_events` table. `INSERT` only; `UPDATE`/`DELETE` blocked by SQLite triggers.
- **Hash chain**: `chain_hash = H(prev_chain_hash || canonical_json(event_without_chain_hash))`.
  `qai audit verify` walks the chain; `audit.verify_chain` job runs it nightly.
- Audited actions in Phase 0 (superset grows per §82): config change, secret-ref change,
  migration applied, source discovered/imported/validated/staged/approved/activated/rolled back,
  canonical change session opened/committed/aborted, approval granted/denied,
  principal/role change, job dead-lettered, doctor repair executed.
- **Secret-free by construction:** the audit writer runs values through the redaction layer and
  a schema that forbids free-form payload keys outside an allowlist.

```rust
pub struct AuditEvent {
    pub id: AuditEventId,
    pub sequence: u64,                 // monotonic, gapless
    pub occurred_at: Timestamp,
    pub actor: Actor,                  // Principal | System | Job | Agent(future)
    pub action: AuditAction,           // closed enum -> stable strings
    pub subject: SubjectRef,
    pub outcome: AuditOutcome,         // Allowed | Denied | Failed
    pub reason: Option<String>,
    pub before: Option<serde_json::Value>, // redacted, allowlisted keys
    pub after: Option<serde_json::Value>,
    pub request_id: Option<String>,
    pub prev_chain_hash: ContentHash,
    pub chain_hash: ContentHash,
}
```

---

### D0.13 — CLI Skeleton

**PRD:** §25.4, §26

Phase 0 implements the framework + the commands that exist now. All Phase 1+ commands are
registered as stubs that print a clear "available in Phase N" diagnostic (so `--help` shows
the final shape and shell completions are stable).

```bash
qai                                  # short status + hints
qai version [--json]
qai doctor [--json] [--core] [--sources] [--indexes] [--models] [--tools] [--repair-preview]
qai serve                            # health-only server in Phase 0

qai config show [--explain] [--defaults] [--json]
qai config get <key>
qai config set <key> <value>
qai config validate [--file <path>]

qai db migrate | status | verify | plan
qai db backup <path>                 # consistent SQLite backup
qai db restore <path> --yes

qai secret list                      # refs only
qai secret set <ref>                 # reads value from stdin/prompt, never argv
qai secret delete <ref> --yes
qai secret backend

qai source list [--state <s>] [--json]
qai source show <source-id> [--versions] [--genealogy]
qai source import <manifest-path>    # local manifest -> Staged
qai source validate <staged-version>
qai source approve <staged-version>  # requires confirmation; writes ApprovalRecord + audit
qai source rollback <source-id> --to <version> --yes
qai source diff <source-id> --from <v1> --to <v2>
qai source quarantine <version> --reason <text>

qai job list [--state <s>] | show <job-id> | cancel <job-id> | retry <job-id>
qai job logs <job-id> [--follow]

qai audit list [--since <ts>] [--action <a>] | verify

qai completions <bash|zsh|fish|powershell>
```

**Global flags (§25.4):** `--config`, `--data-dir`, `--log-level`, `--no-color`, `--json`,
`--quiet`, `--yes`.

**Conventions enforced by a CLI conformance test:**
verbs after nouns as specified; `--json` supported on every read command; destructive commands
confirm unless `--yes`; exit codes: `0` ok, `1` generic, `2` usage, `3` validation failed,
`4` denied by policy, `5` not found, `6` conflict/state, `7` cancelled, `70` internal.

---

### D0.14 — `qai doctor` (Core)

**PRD:** §50

Phase 0 checks (each returns `Pass | Warn | Fail | Skipped` + remedy + next command):

```text
CORE
  configuration.valid
  configuration.file_permissions          (0600 recommended for files with secret refs)
  data_dir.writable
  database.reachable
  database.migration_current
  database.integrity_check                (PRAGMA integrity_check, read-only)
  database.foreign_keys_enabled
  secrets.backend_available
  secrets.refs_resolvable                 (existence only; never prints values)
  jobs.worker_health
  jobs.interrupted_runs                   (§85)
  jobs.dead_letter_count
  audit.chain_valid
  sources.manifest_schema_valid
  sources.orphaned_versions               (§50)
  sources.missing_files
  sources.license_unknown_count
  sources.multiple_active_versions        (must be 0)
  filesystem.object_store_writable
  security.bind_address_safe
  security.tls_policy_consistent
  observability.subscriber_installed
  local_only_fallbacks                    (§25.13: report local-only fallback paths)
```

Hard rules:
- `doctor` opens the database **read-only**. Enforced by using the read-only pool and asserted
  by a test that runs `doctor` against a file-permission-restricted DB.
- Repairs are never automatic. `--repair-preview` prints the plan; `qai repair <check-id>`
  (Phase 1+) executes with confirmation and an audit event.

Output shape mirrors the PRD §50 example; `--json` output is schema-stable and CI-consumable.

---

### D0.15 — Security Baseline

**PRD:** §37, §84, §74

Implemented as reusable, tested guard libraries used by every later phase:

| Guard | Module | Behaviour |
|---|---|---|
| Path containment | `security::path` | Canonicalize, reject `..`, reject symlink escape, reject absolute outside root |
| Archive safety | `security::archive` | Zip-slip block, entry-count cap, expansion-ratio cap, nested-archive depth cap, no symlink/device entries |
| SSRF guard | `security::net` | DNS-resolve-then-check, block loopback/link-local/private/CGNAT/multicast/IPv6-ULA, block redirects to blocked ranges, domain allowlist, port allowlist, max redirects |
| Size/time limits | `security::limits` | Download cap, request cap, per-op timeout, decompression cap |
| Input validation | `security::input` | UTF-8 validation, control-char policy, max field lengths, JSON depth cap |
| HTML/text sanitization | `security::sanitize` | Strip scripts/handlers/`javascript:`; used by importers from Phase 7 |
| Untrusted-content marking | `security::taint` | `Untrusted<T>` wrapper; retrieved/imported content must be unwrapped explicitly (foundation for §74) |
| Deny-by-default policy | `policy::baseline` | Empty permission sets; `PolicyDecision::{Allow, Deny, RequireApproval}` with reason strings |

**Fail-closed rule (§37):** if a guard cannot be enforced (e.g., cannot canonicalize a path,
cannot resolve DNS to check ranges), the operation is denied.

`Untrusted<T>` is introduced now, in Phase 0, specifically because §74 and §92:36 are
unenforceable if added later.

---

### D0.16 — Test Harness & CI

**PRD:** §25.13, §58

**Test tiers**

| Tier | Location | Runs |
|---|---|---|
| Unit | in-crate `#[cfg(test)]` | every PR |
| Property | `proptest` in-crate | every PR |
| Repository/integration | `crates/*/tests/` with temp SQLite | every PR |
| Workspace integration | `/tests/` | every PR |
| CLI snapshot | `trycmd` / `insta` in `/tests/cli/` | every PR |
| Chaos/recovery | `/tests/recovery/` (kill workers, corrupt WAL, expire leases) | nightly |
| Security | `/tests/security/` (guards, secret-leak sentinel) | every PR |
| Performance smoke | `criterion` benches, thresholds only | nightly |

**Shared fixtures crate** (`crates/testkit`, dev-only): temp data dir, migrated in-memory and
on-disk DBs, deterministic clock, deterministic UUID generator, fake `SecretStore`,
in-process job runner, manifest builders, audit-chain assertions.

**CI pipeline (GitHub Actions or equivalent)**

```text
job: check          -> cargo fmt --check; cargo clippy --all-targets -- -D warnings
job: arch           -> cargo xtask arch-check; cargo xtask migrate-check
job: test-linux     -> cargo test --workspace --all-features
job: test-macos     -> cargo test --workspace
job: test-windows   -> cargo test --workspace
job: deny           -> cargo deny check advisories bans licenses sources
job: schemas        -> cargo xtask gen-schema && git diff --exit-code docs/schemas/
job: doctor         -> qai doctor --json | validate against docs/schemas/doctor.v1.schema.json
job: coverage       -> cargo llvm-cov; gate: domain/provenance/audit/security >= 85% lines
job: msrv           -> build with pinned MSRV
```

---

### D0.17 — Documentation, ADRs & Developer Experience

**PRD:** §25.13, §48, §60, §94

- `adr/0000-adr-template.md` containing **every** field PRD §48 requires: Context, Options,
  Decision, **Accuracy implications**, **Religious-source implications**, **Licensing
  implications**, Security implications, Operational implications, Migration strategy,
  Reversal cost.
- `docs/architecture/`: crate map, dependency rules, data layers, source lifecycle, job
  semantics, error codes, hashing/canonical-JSON spec, security guards.
- `docs/runbooks/`: backup & restore, interrupted-job recovery, audit-chain break,
  migration checksum mismatch, quarantine handling.
- `CONTRIBUTING.md` with the Definition of Done checklist (§58) as a PR template.
- `.env.example`, `examples/config/*.toml`, `docker/` assets.

---

## 5. Database Schema (Phase 0 Migrations)

### `0001_core.up.sql`

```sql
PRAGMA foreign_keys = ON;

CREATE TABLE schema_migrations (
  version      INTEGER PRIMARY KEY,
  name         TEXT    NOT NULL,
  checksum     TEXT    NOT NULL,
  applied_at   TEXT    NOT NULL,
  applied_by   TEXT    NOT NULL,
  duration_ms  INTEGER NOT NULL
);

-- Minimal principal model. Phase 11 extends with roles/OIDC; the shape stays.
CREATE TABLE principals (
  id           TEXT PRIMARY KEY,
  kind         TEXT NOT NULL CHECK (kind IN ('local_user','service','system')),
  display_name TEXT NOT NULL,
  created_at   TEXT NOT NULL,
  disabled_at  TEXT
);

CREATE TABLE workspaces (
  id          TEXT PRIMARY KEY,
  name        TEXT NOT NULL UNIQUE,
  owner_id    TEXT NOT NULL REFERENCES principals(id),
  created_at  TEXT NOT NULL,
  archived_at TEXT
);

CREATE TABLE settings (
  key         TEXT PRIMARY KEY,
  value_json  TEXT NOT NULL,
  origin      TEXT NOT NULL,      -- default|file|env|cli|api
  updated_at  TEXT NOT NULL,
  updated_by  TEXT REFERENCES principals(id)
);

-- Object store index: every file Q-ai owns on disk.
CREATE TABLE blobs (
  id            TEXT PRIMARY KEY,
  hash          TEXT NOT NULL,           -- "sha256:<hex>"
  bytes         INTEGER NOT NULL CHECK (bytes >= 0),
  media_type    TEXT,
  relative_path TEXT NOT NULL UNIQUE,
  created_at    TEXT NOT NULL,
  UNIQUE (hash, bytes)
);
```

### `0002_sources.up.sql`

```sql
CREATE TABLE sources (
  id                TEXT PRIMARY KEY,      -- stable slug, e.g. 'hafs-uthmani'
  title             TEXT NOT NULL,
  alternate_titles  TEXT NOT NULL DEFAULT '[]',
  content_type      TEXT NOT NULL,         -- quran_edition|translation|tafsir|hadith_collection|scripture|book|dataset
  authors           TEXT NOT NULL DEFAULT '[]',
  compiler          TEXT,
  translator        TEXT,
  editor            TEXT,
  publisher         TEXT,
  language          TEXT,                  -- BCP-47
  tradition         TEXT,
  school            TEXT,
  identifiers       TEXT NOT NULL DEFAULT '{}', -- ISBN, OCLC, ...
  created_at        TEXT NOT NULL,
  updated_at        TEXT NOT NULL
);

CREATE TABLE source_versions (
  id                   TEXT PRIMARY KEY,
  source_id            TEXT NOT NULL REFERENCES sources(id) ON DELETE RESTRICT,
  version              TEXT NOT NULL,
  schema_version       INTEGER NOT NULL,
  state                TEXT NOT NULL CHECK (state IN (
                          'Discovered','PendingReview','Downloading','Downloaded',
                          'Validating','ValidationFailed','Staged','Approved',
                          'Indexing','Active','Deprecated','Quarantined','Removed')),
  trust_level          TEXT NOT NULL CHECK (trust_level IN (
                          'Quarantined','MachineGenerated','UserProvided','ImportedUnverified',
                          'CommunityReviewed','ScholarReviewed','PublisherVerified','CanonicalVerified')),
  license_status       TEXT NOT NULL CHECK (license_status IN (
                          'PublicDomain','OpenLicense','PermissionGranted','UserOwned',
                          'MetadataOnly','Unknown','Restricted')),
  license_json         TEXT NOT NULL,
  manifest_blob_id     TEXT REFERENCES blobs(id),
  manifest_hash        TEXT,
  content_hash         TEXT,
  source_urls          TEXT NOT NULL DEFAULT '[]',
  publication_date     TEXT,
  imported_at          TEXT,
  validated_at         TEXT,
  approved_at          TEXT,
  approved_by          TEXT REFERENCES principals(id),
  activated_at         TEXT,
  deprecated_at        TEXT,
  quarantine_reason    TEXT,
  validation_report    TEXT,               -- JSON structural validation report
  notes                TEXT,
  created_at           TEXT NOT NULL,
  UNIQUE (source_id, version),
  -- Approval preconditions (PRD §22.3, §22.5)
  CHECK (state NOT IN ('Approved','Indexing','Active')
         OR (content_hash IS NOT NULL
             AND license_status <> 'Unknown'
             AND approved_by IS NOT NULL
             AND validation_report IS NOT NULL))
);

-- Exactly one Active version per source (PRD §85 atomic activation).
CREATE UNIQUE INDEX ux_source_active
  ON source_versions(source_id) WHERE state = 'Active';

CREATE TABLE source_files (
  id                TEXT PRIMARY KEY,
  source_version_id TEXT NOT NULL REFERENCES source_versions(id) ON DELETE CASCADE,
  role              TEXT NOT NULL,          -- primary|metadata|scan|audio|aux
  relative_path     TEXT NOT NULL,
  format            TEXT NOT NULL,
  media_type        TEXT,
  bytes             INTEGER,
  declared_hash     TEXT NOT NULL,
  observed_hash     TEXT,
  encoding          TEXT,
  unicode_form      TEXT CHECK (unicode_form IN ('nfc','nfd','nfkc','nfkd','unknown')),
  blob_id           TEXT REFERENCES blobs(id),
  verified_at       TEXT,
  UNIQUE (source_version_id, relative_path)
);

-- PRD §22.6 genealogy
CREATE TABLE source_genealogy (
  child_source_id    TEXT NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
  parent_source_id   TEXT NOT NULL REFERENCES sources(id) ON DELETE RESTRICT,
  derivation_type    TEXT NOT NULL CHECK (derivation_type IN
                        ('translation','summary','edition','abridgment','commentary','original')),
  derivation_language TEXT,
  derivation_date    TEXT,
  derivation_note    TEXT,
  PRIMARY KEY (child_source_id, parent_source_id),
  CHECK (child_source_id <> parent_source_id)
);

CREATE TABLE source_state_transitions (
  id                TEXT PRIMARY KEY,
  source_version_id TEXT NOT NULL REFERENCES source_versions(id) ON DELETE CASCADE,
  from_state        TEXT,
  to_state          TEXT NOT NULL,
  actor_id          TEXT REFERENCES principals(id),
  reason            TEXT,
  occurred_at       TEXT NOT NULL
);

CREATE TABLE approvals (
  id                TEXT PRIMARY KEY,
  subject_urn       TEXT NOT NULL,
  kind              TEXT NOT NULL,   -- source_activation|canonical_change|rollback|tool_publish(future)
  requested_by      TEXT REFERENCES principals(id),
  decided_by        TEXT REFERENCES principals(id),
  decision          TEXT CHECK (decision IN ('approved','denied')),
  request_payload   TEXT NOT NULL,   -- includes DifferenceReport + checksums
  decision_note     TEXT,
  requested_at      TEXT NOT NULL,
  decided_at        TEXT
);
```

### `0003_provenance.up.sql`

```sql
CREATE TABLE provenance_records (
  id                  TEXT PRIMARY KEY,
  layer               TEXT NOT NULL CHECK (layer IN (
                        'canonical_source','publisher_metadata','scholarly_annotation',
                        'computational_annotation','user_or_ai_note')),
  subject_urn         TEXT NOT NULL,
  attribution_kind    TEXT NOT NULL CHECK (attribution_kind IN ('dataset','scholar','computational','user')),
  attribution_json    TEXT NOT NULL,
  source_version_id   TEXT REFERENCES source_versions(id),
  location_json       TEXT,
  quoted_text_hash    TEXT,
  trust_level         TEXT NOT NULL,
  verification_status TEXT NOT NULL CHECK (verification_status IN
                        ('unverified','needs_review','human_verified','rejected','superseded')),
  confidence          REAL CHECK (confidence IS NULL OR (confidence >= 0.0 AND confidence <= 1.0)),
  versions_json       TEXT NOT NULL,
  created_at          TEXT NOT NULL,
  created_by          TEXT REFERENCES principals(id),
  reviewed_by         TEXT REFERENCES principals(id),
  reviewed_at         TEXT,
  review_note         TEXT,
  superseded_by       TEXT REFERENCES provenance_records(id),

  -- §6.4: computational annotations must carry algorithm + confidence
  CHECK (layer <> 'computational_annotation'
         OR (attribution_kind = 'computational' AND confidence IS NOT NULL)),
  -- §10.6: no computational suggestion becomes verified without a human decision
  CHECK (verification_status <> 'human_verified' OR reviewed_by IS NOT NULL),
  -- §6.1: canonical rows are always tied to a source version
  CHECK (layer <> 'canonical_source' OR source_version_id IS NOT NULL)
);

CREATE INDEX ix_prov_subject ON provenance_records(subject_urn);
CREATE INDEX ix_prov_layer   ON provenance_records(layer, verification_status);
CREATE INDEX ix_prov_srcver  ON provenance_records(source_version_id);

-- Canonical-layer provenance is insert-only.
CREATE TRIGGER trg_prov_canonical_no_update
BEFORE UPDATE ON provenance_records
WHEN OLD.layer = 'canonical_source'
BEGIN
  SELECT RAISE(ABORT, 'QAI-PROV-0001: canonical provenance is immutable');
END;

CREATE TRIGGER trg_prov_canonical_no_delete
BEFORE DELETE ON provenance_records
WHEN OLD.layer = 'canonical_source'
BEGIN
  SELECT RAISE(ABORT, 'QAI-PROV-0002: canonical provenance cannot be deleted');
END;

-- Review queue for computational suggestions (§10.6 foundation).
CREATE TABLE review_queue (
  id             TEXT PRIMARY KEY,
  provenance_id  TEXT NOT NULL REFERENCES provenance_records(id) ON DELETE CASCADE,
  queue          TEXT NOT NULL,           -- graph_edge|morphology|narrator_identity|cross_reference
  priority       INTEGER NOT NULL DEFAULT 0,
  evidence_json  TEXT NOT NULL,           -- must be shown before acceptance (§10.6)
  state          TEXT NOT NULL CHECK (state IN ('pending','accepted','rejected','corrected')),
  decided_by     TEXT REFERENCES principals(id),
  decided_at     TEXT,
  decision_note  TEXT,
  created_at     TEXT NOT NULL
);
```

### `0004_jobs.up.sql`

```sql
CREATE TABLE jobs (
  id                TEXT PRIMARY KEY,
  kind              TEXT NOT NULL,
  payload_json      TEXT NOT NULL,
  idempotency_key   TEXT,
  state             TEXT NOT NULL CHECK (state IN (
                      'Queued','Leased','Running','Checkpointed','Succeeded',
                      'Failed','Cancelled','Interrupted','DeadLettered')),
  priority          INTEGER NOT NULL DEFAULT 0,
  attempts          INTEGER NOT NULL DEFAULT 0,
  max_attempts      INTEGER NOT NULL DEFAULT 5,
  available_at      TEXT NOT NULL,
  lease_owner       TEXT,
  lease_expires_at  TEXT,
  checkpoint_json   TEXT,
  progress_json     TEXT,
  cancel_requested  INTEGER NOT NULL DEFAULT 0 CHECK (cancel_requested IN (0,1)),
  parent_job_id     TEXT REFERENCES jobs(id),
  versions_json     TEXT NOT NULL DEFAULT '{}',
  error_code        TEXT,
  error_json        TEXT,
  created_at        TEXT NOT NULL,
  started_at        TEXT,
  finished_at       TEXT,
  created_by        TEXT REFERENCES principals(id),
  UNIQUE (kind, idempotency_key)
);

CREATE INDEX ix_jobs_claim ON jobs(state, available_at, priority DESC);
CREATE INDEX ix_jobs_kind  ON jobs(kind, state);

CREATE TABLE job_events (
  id          TEXT PRIMARY KEY,
  job_id      TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
  sequence    INTEGER NOT NULL,
  level       TEXT NOT NULL CHECK (level IN ('trace','debug','info','warn','error')),
  stage       TEXT,
  message     TEXT NOT NULL,
  data_json   TEXT,
  occurred_at TEXT NOT NULL,
  UNIQUE (job_id, sequence)
);
```

### `0005_audit.up.sql`

```sql
CREATE TABLE audit_events (
  id                TEXT PRIMARY KEY,
  sequence          INTEGER NOT NULL UNIQUE,
  occurred_at       TEXT NOT NULL,
  actor_kind        TEXT NOT NULL CHECK (actor_kind IN ('principal','system','job','agent')),
  actor_id          TEXT,
  action            TEXT NOT NULL,
  subject_urn       TEXT NOT NULL,
  outcome           TEXT NOT NULL CHECK (outcome IN ('allowed','denied','failed')),
  reason            TEXT,
  before_json       TEXT,
  after_json        TEXT,
  request_id        TEXT,
  prev_chain_hash   TEXT NOT NULL,
  chain_hash        TEXT NOT NULL
);

CREATE INDEX ix_audit_time    ON audit_events(occurred_at);
CREATE INDEX ix_audit_action  ON audit_events(action, occurred_at);
CREATE INDEX ix_audit_subject ON audit_events(subject_urn);

CREATE TRIGGER trg_audit_no_update BEFORE UPDATE ON audit_events
BEGIN SELECT RAISE(ABORT, 'QAI-AUD-0001: audit log is append-only'); END;

CREATE TRIGGER trg_audit_no_delete BEFORE DELETE ON audit_events
BEGIN SELECT RAISE(ABORT, 'QAI-AUD-0002: audit log is append-only'); END;
```

---

## 6. ADRs Required In Phase 0

| ADR | Title | Blocking For | Key trade-off |
|---|---|---|---|
| ADR-0001 | Relational store & access layer: SQLite + `sqlx`, Postgres-portable SQL | D0.6 | compile-time query checks vs. dialect portability |
| ADR-0002 | Migration strategy: append-only checksummed SQL, forward-only for canonical tables | D0.7 | reversibility vs. canonical immutability |
| ADR-0003 | Durable job system: DB-backed leased queue (no external broker) | D0.9 | operational simplicity vs. throughput ceiling |
| ADR-0004 | Configuration & precedence model, env mapping, interpolation | D0.4 | flexibility vs. predictability |
| ADR-0005 | Secret storage per OS (env / keychain / age-encrypted file) | D0.5 | portability vs. OS-native security |
| ADR-0006 | Hashing & canonical serialization (SHA-256 + canonical JSON + NFC policy) | D0.2 | reproducibility forever vs. algorithm agility |
| ADR-0007 | Source manifest format & signing (ed25519 detached over canonical JSON) | D0.10 | trust model vs. publisher friction |
| ADR-0008 | Provenance representation: single universal record + typed attribution | D0.11 | one table vs. per-domain tables |
| ADR-0009 | Audit integrity: hash chain in SQLite, append-only triggers | D0.12 | tamper evidence vs. tamper prevention |
| ADR-0010 | Error taxonomy & CLI exit codes | D0.3, D0.13 | granularity vs. stability |
| ADR-0011 | Observability stack: `tracing` + `metrics` + optional OTLP; telemetry opt-in | D0.8 | insight vs. privacy (§39) |
| ADR-0012 | Workspace/crate boundaries & dependency-direction enforcement | D0.1 | many small crates vs. build time |

Each uses the §48 template, including the *Religious-source implications* section
(for Phase 0 ADRs this is usually "none directly, but constrains Phase 1 canonical integrity" —
and that reasoning must be written down, not omitted).

---

## 7. Work Breakdown Structure

Estimates are engineer-days (ed). Roles: **BE** backend/Rust, **INF** infra/CI, **SEC** security,
**DOC** docs. Total ≈ **68 ed** ⇒ ~5 weeks with 3 engineers including review and slack.

### Sprint 0.1 — Skeleton & Contracts (Week 1)

| ID | Task | Deliverable | Depends | Est | Role |
|---|---|---|---|---|---|
| P0-T01 | Create workspace, toolchain pin, lint config, workspace deps | D0.1 | — | 1.5 | INF |
| P0-T02 | `xtask` with `arch-check`, `ci`, `migrate-check`, `gen-schema` | D0.1 | T01 | 2.0 | INF |
| P0-T03 | `deny.toml` license/advisory policy + CI job | D0.1 | T01 | 0.5 | INF |
| P0-T04 | Placeholder crates for all PRD §87 crates with phase doc comments | D0.1 | T01 | 0.5 | BE |
| P0-T05 | `domain`: typed IDs, `SemVer`, `Timestamp`, `Language`, `Confidence` | D0.2 | T01 | 1.5 | BE |
| P0-T06 | `domain`: `DataLayer`, `TrustLevel`, `VerificationStatus`, `SideEffectClass` | D0.2 | T05 | 1.0 | BE |
| P0-T07 | `domain`: `ContentHash`, `canonical_json_bytes`, hashing spec + prop tests | D0.2 | T05 | 1.5 | BE |
| P0-T08 | `domain`: `LicenseRecord`, `DerivationVersions`, `SubjectRef` URN grammar | D0.2 | T05 | 1.0 | BE |
| P0-T09 | `Diagnostic` trait, error-code registry, uniqueness test | D0.3 | T05 | 1.5 | BE |
| P0-T10 | ADR-0001/0002/0006/0012 written and reviewed | ADR | T01 | 1.5 | DOC |
| P0-T11 | CI pipeline (check/arch/test-3os/deny/schemas/coverage) | D0.16 | T02 | 2.0 | INF |

### Sprint 0.2 — Config, Secrets, Storage, Migrations (Week 2)

| ID | Task | Deliverable | Depends | Est | Role |
|---|---|---|---|---|---|
| P0-T12 | `config` crate: layered loader + `ValueOrigin` + interpolation | D0.4 | T09 | 2.5 | BE |
| P0-T13 | Config validation rules + precedence matrix tests | D0.4 | T12 | 1.5 | BE |
| P0-T14 | `Secret<T>`, `SecretRef`, `SecretStore` trait | D0.5 | T09 | 1.0 | SEC |
| P0-T15 | Env + keychain + age-encrypted-file backends | D0.5 | T14 | 2.5 | SEC |
| P0-T16 | Global redaction tracing layer + secret-leak sentinel suite | D0.5, D0.16 | T14 | 2.0 | SEC |
| P0-T17 | `storage` traits: `Database`, `ReadTx`, `UnitOfWork`, repo traits, `StorageError` | D0.6 | T09 | 2.0 | BE |
| P0-T18 | `storage-sqlite`: dual pools, pragmas, tx semantics, health | D0.6 | T17 | 2.5 | BE |
| P0-T19 | Migration runner: apply, checksum verify, status, plan, backup/restore | D0.7 | T18 | 2.5 | BE |
| P0-T20 | Migrations `0001_core`, `0005_audit` | D0.6, D0.12 | T19 | 1.5 | BE |
| P0-T21 | ADR-0004/0005 | ADR | T12,T15 | 1.0 | DOC |

### Sprint 0.3 — Provenance, Audit, Sources (Week 3)

| ID | Task | Deliverable | Depends | Est | Role |
|---|---|---|---|---|---|
| P0-T22 | Migration `0003_provenance` + triggers + CHECK constraints | D0.11 | T20 | 1.5 | BE |
| P0-T23 | `provenance` crate: record model, repository, invariant tests | D0.11 | T22 | 2.5 | BE |
| P0-T24 | `ApprovalToken`, `CanonicalWriter`, `CanonicalChangeSession` | D0.11 | T23 | 2.0 | BE |
| P0-T25 | `review_queue` model + accept/reject/correct API + evidence requirement | D0.11 | T23 | 1.5 | BE |
| P0-T26 | `audit` crate: event model, hash chain writer, verifier, `AuditAction` enum | D0.12 | T20 | 2.5 | BE |
| P0-T27 | Audit redaction + allowlisted payload schema + leak tests | D0.12 | T26,T16 | 1.0 | SEC |
| P0-T28 | Migration `0002_sources` | D0.10 | T20 | 1.0 | BE |
| P0-T29 | `sources`: manifest parse + JSON-Schema + semantic validation | D0.10 | T28,T07 | 2.5 | BE |
| P0-T30 | `sources`: ed25519 signature verification + unsigned policy | D0.10 | T29 | 1.5 | SEC |
| P0-T31 | `sources`: state machine + transition log + approval preconditions | D0.10 | T28,T24 | 2.5 | BE |
| P0-T32 | `sources`: genealogy resolver, cycle detection, lineage rendering | D0.10 | T28 | 1.5 | BE |
| P0-T33 | `sources`: `StructureValidator` registry + `DifferenceReport` framework | D0.10 | T31 | 1.5 | BE |
| P0-T34 | ADR-0007/0008/0009 | ADR | T30,T23,T26 | 1.5 | DOC |

### Sprint 0.4 — Jobs, Security Guards, Observability (Week 4)

| ID | Task | Deliverable | Depends | Est | Role |
|---|---|---|---|---|---|
| P0-T35 | Migration `0004_jobs` | D0.9 | T20 | 0.5 | BE |
| P0-T36 | Job repository: enqueue, claim-with-lease, heartbeat, finish, cancel | D0.9 | T35 | 2.5 | BE |
| P0-T37 | Worker pool, handler registry, payload-schema validation | D0.9 | T36 | 2.0 | BE |
| P0-T38 | Cancellation, deadlines, checkpoint/resume, progress reporting | D0.9 | T37 | 2.0 | BE |
| P0-T39 | Retry/backoff/jitter, dead-lettering, interrupted-run recovery scan | D0.9 | T37 | 1.5 | BE |
| P0-T40 | Job chaos tests (kill worker, expire lease, duplicate enqueue, resume) | D0.16 | T39 | 2.0 | BE |
| P0-T41 | `observability`: subscriber, span conventions, metric catalog | D0.8 | T12 | 2.0 | BE |
| P0-T42 | OTLP exporter (opt-in) + telemetry field-denylist test | D0.8 | T41 | 1.5 | BE |
| P0-T43 | `security::path`, `security::archive` + attack-corpus tests | D0.15 | T09 | 2.0 | SEC |
| P0-T44 | `security::net` SSRF guard (resolve-then-check, redirect policy) | D0.15 | T09 | 2.0 | SEC |
| P0-T45 | `security::limits`, `security::input`, `security::sanitize`, `Untrusted<T>` | D0.15 | T09 | 2.0 | SEC |
| P0-T46 | `policy::baseline` deny-by-default decision engine | D0.15 | T09 | 1.0 | SEC |
| P0-T47 | ADR-0003/0011 | ADR | T37,T41 | 1.0 | DOC |

### Sprint 0.5 — CLI, Doctor, Hardening, Docs (Week 5)

| ID | Task | Deliverable | Depends | Est | Role |
|---|---|---|---|---|---|
| P0-T48 | CLI framework: `clap` tree, global flags, `--json` renderer, exit codes | D0.13 | T09,T12 | 2.5 | BE |
| P0-T49 | `config`, `db`, `secret` command groups | D0.13 | T48 | 2.0 | BE |
| P0-T50 | `source`, `job`, `audit` command groups | D0.13 | T48,T31 | 2.5 | BE |
| P0-T51 | Phase-N stub commands + shell completions + CLI conformance test | D0.13 | T48 | 1.5 | BE |
| P0-T52 | `doctor` engine: check registry, read-only enforcement, severity, remedies | D0.14 | T48 | 2.5 | BE |
| P0-T53 | Phase-0 doctor checks (all listed in D0.14) + JSON schema | D0.14 | T52 | 2.5 | BE |
| P0-T54 | `--repair-preview` planner (no mutation) | D0.14 | T52 | 1.0 | BE |
| P0-T55 | `serve` stub: `/healthz`, `/readyz`, `/api/v1/meta`, localhost bind guard | D0.1 | T48 | 1.5 | BE |
| P0-T56 | Dockerfile + compose stub + non-root runtime | D0.1 | T55 | 1.5 | INF |
| P0-T57 | `testkit` crate finalization + fixtures + deterministic clock/UUID | D0.16 | T18 | 2.0 | BE |
| P0-T58 | Architecture docs, runbooks, CONTRIBUTING/DoD PR template | D0.17 | all | 2.5 | DOC |
| P0-T59 | ADR-0010 + ADR index + template lint (all §48 fields present) | ADR | T09 | 1.0 | DOC |
| P0-T60 | Phase-0 exit-gate review, AC verification, Phase-1 handoff doc | — | all | 1.5 | all |

---

## 8. Testing Strategy (Phase 0 Specific)

### 8.1 Must-have test suites

| Suite | What it proves | PRD |
|---|---|---|
| `tests/security/secret_leak.rs` | A sentinel secret appears in zero bytes of logs, errors, CLI output, doctor JSON, audit rows | §25.13, §37 |
| `tests/security/path_guard.rs` | 40+ traversal/symlink payloads rejected | §37, §84 |
| `tests/security/archive_guard.rs` | zip-slip, bomb, symlink-entry, nested-depth all rejected | §22.5, §84 |
| `tests/security/ssrf_guard.rs` | loopback/private/link-local/DNS-rebind/redirect-to-private rejected | §22.5, §84 |
| `tests/integrity/canonical_guard.rs` | Canonical provenance cannot be updated/deleted; writes require `ApprovalToken`; a `CanonicalChangeRequest` missing any §7.3 field is rejected | §7.3, §92:43 |
| `tests/integrity/audit_chain.rs` | Chain verifies; a tampered row is detected; update/delete aborts | §82 |
| `tests/sources/state_machine.rs` | All illegal transitions rejected; `Approved` requires hash+license+report+approver; only one `Active` | §22.2, §22.3 |
| `tests/sources/manifest.rs` | Schema validation, signature verify/reject, unsigned policy, genealogy cycles | §22.4, §22.6 |
| `tests/recovery/jobs.rs` | Lease expiry, crash → `Interrupted`, resume from checkpoint, no double side effect | §41, §85 |
| `tests/config/precedence.rs` | Full CLI>Env>File>Defaults matrix; invalid configs rejected with actionable errors | §25.13 |
| `tests/cli/*.trycmd` | Human + `--json` output snapshots for every Phase-0 command | §25.4 |
| `tests/db/migrations.rs` | Fresh migrate, idempotent re-run, checksum mismatch fails, down-migrations restore schema | §43 P0 |
| `tests/observability/telemetry_privacy.rs` | Denylisted fields never exported | §39 |

### 8.2 Coverage gates

- `domain`, `provenance`, `audit`, `sources`, `security`: **≥ 85%** line coverage.
- `config`, `jobs`, `storage-sqlite`: **≥ 75%**.
- CLI/server: smoke + snapshot coverage, no numeric gate.

---

## 9. Acceptance Criteria (Phase 0 Exit Gate)

| ID | Criterion | Verification |
|---|---|---|
| AC-P0-01 | `cargo xtask ci` passes on Linux, macOS, Windows from a clean clone | CI matrix green |
| AC-P0-02 | `cargo xtask arch-check` fails when a forbidden dependency edge is introduced | Mutation test: add `domain -> cli` dep, CI must fail |
| AC-P0-03 | `qai db migrate` on an empty dir produces a valid schema; re-run is a no-op; editing an applied migration file causes a hard, coded failure | scripted test |
| AC-P0-04 | Config precedence CLI>Env>File>Defaults holds for all typed field kinds, and `qai config show --explain` prints the origin of every value | precedence matrix test + snapshot |
| AC-P0-05 | A sentinel secret set through each backend never appears in logs, errors, CLI output, doctor JSON, or audit rows | secret-leak suite |
| AC-P0-06 | Canonical provenance rows cannot be updated or deleted, and canonical writes are impossible without an `ApprovalToken` derived from a persisted human approval | integrity suite + trigger tests |
| AC-P0-07 | A computational annotation cannot be stored without algorithm+version+confidence, and cannot reach `human_verified` without a reviewer | DB CHECK tests |
| AC-P0-08 | Source lifecycle rejects every illegal transition; `Approved` is impossible without hash, known license, validation report, and approver; at most one `Active` version per source | state-machine suite |
| AC-P0-09 | A signed local manifest imports to `Staged`; a tampered manifest is rejected with `QAI-SRC-…`; an unsigned remote manifest is rejected under default policy | manifest suite |
| AC-P0-10 | Source genealogy renders a 3-level lineage sentence and rejects cycles | unit + snapshot |
| AC-P0-11 | Jobs: duplicate enqueue with the same idempotency key yields one execution; `SIGKILL`-ing a worker marks the run `Interrupted` and it resumes from checkpoint without repeating completed stages | chaos suite |
| AC-P0-12 | `qai job cancel` stops a long-running job within 2 seconds and records cancellation | timed test |
| AC-P0-13 | Audit chain verifies end-to-end; `qai audit verify` detects a manually tampered row | integrity suite |
| AC-P0-14 | `qai doctor` runs against a read-only database file and never issues a write; all Phase-0 checks emit a remedy and next command; `--json` validates against the published schema | read-only + schema test |
| AC-P0-15 | Security guards reject the full attack corpus (path, archive, SSRF, size, depth) and **fail closed** when a guard cannot be evaluated | security suites |
| AC-P0-16 | `qai serve` binds `127.0.0.1` by default; configuring a non-loopback bind with `tls = "disabled"` fails config validation with an actionable error | config + integration test |
| AC-P0-17 | Every error type implements `Diagnostic`; every error code is unique; human and JSON renderings are snapshot-stable | conformance test |
| AC-P0-18 | Telemetry is off by default; enabling it never exports denylisted content fields | privacy test |
| AC-P0-19 | ADR-0001…ADR-0012 exist, are `Accepted`, and each contains all §48 fields (including religious-source and licensing implications) | ADR lint |
| AC-P0-20 | Docs complete: crate map, data-layer spec, source lifecycle, hashing spec, error codes, 5 runbooks, `.env.example`, example configs | doc review checklist |
| AC-P0-21 | `qai db backup` / `qai db restore` round-trip a populated database with byte-identical audit chain verification afterwards | scripted test |
| AC-P0-22 | Coverage gates met (§8.2) | CI coverage report |

**Exit gate ritual:** a recorded walkthrough where a reviewer performs `AC-P0-03`, `05`, `06`,
`08`, `11`, `14`, `16` live on a clean machine.

---

## 10. Risks & Mitigations

| # | Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | Over-engineering the chassis; Phase 1 slips | High | High | Hard scope fence in §2.2; placeholder crates stay empty; timebox each sprint; cut D0.17 depth before cutting D0.11/D0.12 |
| R2 | Provenance model proves too rigid for hadith gradings / narrator uncertainty (Phase 5/6) | Medium | High | Review the model against three concrete future cases (tafsir claim, hadith grading, narrator possible-identity) during T23; keep `attribution_json` extensible; version `provenance` schema |
| R3 | SQLite single-writer becomes a job-throughput bottleneck | Medium | Medium | Write pool = 1 with short transactions; batch writes; ADR-0001 documents the Postgres migration path; benchmark in nightly perf smoke |
| R4 | Canonical-JSON/hashing decision changes later, invalidating stored hashes | Low | High | Algorithm tag inside `ContentHash`; spec frozen in ADR-0006 with an explicit rehash-migration procedure |
| R5 | Keychain backends behave inconsistently across OSes/CI | High | Low | Env backend is the default; keychain tests are `#[ignore]`-gated in CI and run in a manual matrix |
| R6 | Audit hash chain gaps under concurrent writers | Medium | High | Chain writes serialized through the single write pool inside the same transaction as the audited change; gapless-sequence test under concurrency |
| R7 | Migration checksum enforcement blocks legitimate hotfixes | Medium | Low | Documented `qai db repair-checksum --yes` behind an explicit, audited command; runbook written |
| R8 | Dependency license drift (GPL pulled in transitively) | Medium | Medium | `cargo deny` in CI on every PR; license allowlist in ADR-0001 |
| R9 | Error-code sprawl / instability breaks scripts | Medium | Low | Central registry + uniqueness test + "codes are public API" rule in CONTRIBUTING |
| R10 | Team treats `Untrusted<T>` as ceremony and bypasses it | Medium | High | Clippy lint/`xtask` grep for `.into_inner()` outside allowlisted sanitizer modules; PR checklist item |

---

## 11. Definition of Done (Phase 0)

Per PRD §58 and §93, every Phase-0 deliverable must satisfy:

- [ ] Implemented behind an explicit interface; no cross-layer coupling (`arch-check` green)
- [ ] Unit tests + property tests where state space warrants
- [ ] Integration tests against real SQLite
- [ ] Typed errors implementing `Diagnostic` with remedy + next command
- [ ] Observable: tracing spans + metrics registered in the catalog
- [ ] Documented in `docs/architecture/` and referenced from the crate's `//!` docs
- [ ] Configuration validated at load and on change
- [ ] Cancellation and timeouts implemented where the operation is long-running
- [ ] Secrets redacted everywhere (leak suite green)
- [ ] Provenance and audit events recorded for every mutation
- [ ] Schema versions recorded; migration reversible or explicitly forward-only with rationale
- [ ] Access/policy checks present (deny-by-default) even in single-user mode
- [ ] Failure recovery tested (crash, retry, interrupted job)
- [ ] `cargo fmt`, `cargo clippy -D warnings`, `cargo test`, `cargo deny` all green

---

## 12. Handoff To Phase 1

Phase 1 receives and must not re-invent:

| Asset | Location | Phase-1 usage |
|---|---|---|
| `DataLayer`, `TrustLevel`, `VerificationStatus` | `domain` | Quran text = `CanonicalSource`; numbering = `PublisherMetadata` |
| `ContentHash` + canonical JSON | `domain::hashing` | `text_hash`, `manifest_hash` on the Quran edition (§7.2) |
| `SourceId`/`SourceVersionId` + state machine | `sources` | Quran edition import → `Staged` → `Approved` → `Active` |
| Manifest schema + `StructureValidator` registry | `sources` | register `quran_edition_v1` validator |
| `DifferenceReport` framework | `sources::diff` | Quran edition version diff (§7.3) |
| `ApprovalToken` + `CanonicalWriter` | `provenance` | the only path that may write canonical ayah rows |
| Job system with checkpoints/cancel | `jobs` | `quran.import`, `quran.validate`, `quran.reindex` |
| Audit actions + chain | `audit` | edition activation/rollback events (§82) |
| Doctor check registry | `cli::doctor` | add `--quran` checks |
| `Untrusted<T>`, path/archive/SSRF guards | `security` | importing a downloaded edition archive |
| `testkit` fixtures | `crates/testkit` | Quran corpus golden-file harness |
| Error-code namespace `QAI-QUR-*` | `domain::error` | reserved and ready |

**Handoff document:** `docs/plans/handoff-p0-to-p1.md`, produced by task P0-T60, listing the
above plus known limitations and any deferred items with owners.
