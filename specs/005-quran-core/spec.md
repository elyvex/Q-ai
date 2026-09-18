# Feature Specification: quran-core pure domain

**Feature Branch**: `005-quran-core`

**Created**: 2026-09-17

**Status**: Draft

**Input**: Module-level specification of the implemented `quran-core` crate (crates/quran-core/src/{lib,edition,enums,error,numbers,quotation,reference,structure,text,view}.rs), derived from code truth on 2026-09-17. Reverse specification of what exists today. Code is the source of truth over plan docs.

**Constitution compliance**: `.specify/memory/constitution.md` v1.0.2, Principles I (canonical immutability), I2 via VII (zero model/vector dependencies on the canonical path), III (I6 quotations), IV (translations never as originals).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Pure canonical vocabulary with zero model surface (Priority: P1)

Downstream crates (corpus, search, application, server, cli) share validated newtypes (`SurahNumber`, `AyahNumber`, `TokenPosition`), the edition/structure/view models, and the frozen reference grammar — with a compile-time guarantee that no LLM, embedding, or vector dependency can enter this path.

**Why this priority**: Invariant I2. If the canonical path could import a model crate, no-fabrication guarantees become unenforceable.

**Independent Test**: `cargo test -p quran-core --lib` green (31 passed on 2026-09-17); `cargo run -p xtask -- arch-check` enforces the allowlist (`{domain, serde, thiserror}` + unicode crates only; `storage`/`sqlx`/`tokio`/`llm`/`embeddings`/`retrieval`/vector stores forbidden).

**Acceptance Scenarios**:

1. **Given** a surah/ayah/token number outside range, **When** constructed, **Then** construction fails with a typed `QAI-QUR-0006…0008` error instead of panicking or wrapping.
2. **Given** any dependency audit of the crate, **When** inspected, **Then** no model, embedding, retrieval, vector, I/O, or async crate appears.

---

### User Story 2 - References that never panic (Priority: P1)

Users and tools address verses as strings (`2:255`, `quran:hafs-uthmani@1.0.0:2:255`, `quran:juz:3`, token and range forms). Every malformed input yields a typed `QAI-QUR-01xx` error; the grammar round-trips (parse → serialize → parse) and is property-tested.

**Why this priority**: Addressing is the stable contract for citations, deep links, and tool results. A panic on user input is a defect.

**Independent Test**: Parser matrix tests green (empty, bad surah/ayah/token, reversed ranges, bad slugs/versions, overlong input); golden `fixtures/quran/golden/references.jsonl` (331 cases) round-trips.

**Acceptance Scenarios**:

1. **Given** `quran:2:257-2:255` (reversed range), **When** parsed, **Then** the result is `QAI-QUR-0111` (range order), not a panic.
2. **Given** any valid reference, **When** serialized then re-parsed, **Then** the canonical form is identical.

---

### User Story 3 - Quotations that cannot forget their provenance (Priority: P1)

`QuranQuotation` is the only type allowed to carry quoted canonical text, and it cannot be constructed without edition identity + version + content hash (via mandatory `QuotationParts.edition`/`text_hash`). Translations ride alongside only, each requiring a non-empty named translator.

**Why this priority**: Invariants I6 (every quotation carries edition + version + hash) and Principle 5 (translations never presented as the original).

**Independent Test**: `view.rs` tests assert `QAI-QUR-0009`/`0011` paths; `TranslationRef::validate` rejects empty translators.

**Acceptance Scenarios**:

1. **Given** an attempt to build a quotation without edition or hash, **When** compiled/constructed, **Then** no such constructor exists — the type makes it unrepresentable.
2. **Given** a translation with an empty translator string, **When** validated, **Then** it fails (`MissingTranslator`, `QAI-QUR-0011` family).

### Edge Cases

- Grapheme counting follows ADR-0104 (`char_count` in grapheme clusters via `text::grapheme_count`), not byte or scalar counts.
- `AyahView` has no translation-as-canonical variant (enforced by type shape, mirrored by the DB `translator NOT NULL + CHECK`).
- Division kinds (juz/hizb/rub/manzil/ruku/page/sajdah) parse through the same never-panic grammar (`QAI-QUR-0107/0108` on misuse).
- `Diagnostic::render_json` escapes and stays well-formed (property-tested); every error has summary + remedy (test-enforced).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Crate MUST expose validated `SurahNumber`/`AyahNumber`/`TokenPosition` newtypes plus `Script`, `NumberingScheme`, `BasmalaPolicy`, `RevelationPlace`, `EditionStatus`, `ContextBoundary`, `SegmentKind`, `UnicodeForm`, `SajdahKind` enums and `is_valid_slug` (`numbers.rs`, `enums.rs`).
- **FR-002**: Crate MUST expose `QuranEdition` + `EditionStatistics`, `Surah`/`Ayah`/`Segment`/`Token` structure types, and `AyahView`/`ContextView`/`ContextSpec` read views with attributed translations/glosses (`edition.rs`, `structure.rs`, `view.rs`).
- **FR-003**: Crate MUST implement the frozen reference grammar (ADR-0102): `parse`/`serialize`/`resolve`/`canonical_form` over `QuranRef`/`ResolvedRef`/`DivisionKind`, never panicking on user input (`reference/{mod,parser,serializer}.rs`).
- **FR-004**: Crate MUST provide `QuranQuotation` constructible only via `QuotationParts` with mandatory `edition: EditionRef{slug,version,script,riwayah?}` and `text_hash: ContentHash`; translations only as `TranslationRef{slug,translator(non-empty),language}` (`quotation.rs` L1–53 contract).
- **FR-005**: Crate MUST implement the Phase-1 `Diagnostic` contract (`code`, `summary`, `cause_chain`, `location`, `remedy`, `next_command`, `is_retryable`, `redacted`, `render_human`, `render_json`) on `QuranError`, with uniqueness + summary/remedy coverage test-enforced (`error.rs` L38–389).
- **FR-006**: Crate MUST depend only on `{domain, serde, thiserror}` + unicode crates; never `storage`, `sqlx`, `tokio`, `llm`, `embeddings`, `retrieval`, or vector stores (doc comment L21–25 + `arch-check`).

### Key Entities

- **SurahNumber / AyahNumber / TokenPosition**: Range-validated numeric newtypes; invalid values are typed errors.
- **QuranRef / ResolvedRef**: Parsed vs. edition-resolved addresses (surah, ayah, ranges, tokens, divisions).
- **QuranEdition / EditionStatistics**: Edition identity (slug, version, script, riwayah/qiraah) plus corpus counts.
- **QuranQuotation / QuotationParts / EditionRef / TranslationRef**: The sole quotation type with mandatory provenance.
- **AyahView / ContextView**: Read models with structure-bounded context; translations attributed, never canonical.

### Error Surface

`QAI-QUR-0001` canonical ayah immutable · `0002` edition identity immutable · `0003` no delete · `0004/0005` token immutable/no-delete · `0006` invalid surah number · `0007` invalid ayah number · `0008` invalid token position · `0009` empty field · `0010` invalid slug (`QAI-QUR-0101` family at parse) · `0011` missing translator · `0012/0013` quotation missing edition/hash · `0100` empty · `0101` invalid edition · `0102` invalid edition version · `0103` invalid surah · `0104` invalid ayah · `0105` invalid position · `0106` invalid range · `0107/0108` invalid division/number · `0109` unexpected input · `0110` too long · `0111` range order · `0112` missing locator (`error.rs codes` + parser matrix).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `cargo test -p quran-core --lib` passes with zero failures (31 passed on 2026-09-17).
- **SC-002**: `arch-check` shows zero model/vector/storage/async edges into the crate.
- **SC-003**: 331-case golden reference set round-trips byte-identically; the full malformed-input matrix maps to documented codes with zero panics.
- **SC-004**: No constructible `QuranQuotation` lacks edition + version + hash (constructor-enforced; I6 holds by type shape).

## Assumptions

- Different readings/editions are never merged (`edition_id` in every key; QV-027 downstream); this crate provides the types that make merging unrepresentable.
- Token order stability (I4) is enforced downstream by unique `(edition,surah,ayah,position)` + order checksum; this crate provides `TokenPosition`.
- Basmala handling follows edition metadata per ADR-0110; numbering per ADR-0103; Unicode policy per ADR-0104.
- No I/O, no async, no Tantivy, no embeddings here — lexical search over derived text lives in `quran-search` (FTS5), morphology is a placeholder crate.
