# Quran Corpus Architecture (D1.14)

> **Audience:** maintainers and reviewers of the canonical Quran core.
> **Status:** Phase 1. Canonical text is stored structurally; interpretations are
> never mixed into it.
> **Related:** ADR-0101…0114, `docs/02-architecture/decisions/`.

## 1. Layers and dependency direction

```
domain ──▶ quran-core ──▶ quran-corpus ──▶ storage ──▶ application ──▶ {cli, server, tools}
                                              ▲
                              storage-sqlite ─┘
```

- **`crates/domain`** — shared value types (`ContentHash`, `SemVer`,
  `Timestamp`, `DataLayer`, `TrustLevel`, `VerificationStatus`). No I/O.
- **`crates/quran-core`** — pure canonical domain: numbering newtypes
  (`SurahNumber`, `AyahNumber`, `TokenPosition`), edition/structure types,
  the reference grammar (`reference/{parser,serializer}.rs`), `QuranQuotation`,
  and the read surfaces (`view.rs`: `AyahView`, `AttributedTranslation`,
  `ContextBoundary`). **No storage, no LLM, no embeddings, no retrieval.**
- **`crates/quran-corpus`** — the ingest pipeline: intermediate format,
  adapters, tokenizer, hashing, Unicode audit, validators, differ, importer.
- **`crates/storage` / `storage-sqlite`** — repository traits and the SQLite
  implementation. Canonical tables are insert-only (DB triggers).
- **`crates/application`** — services that compose the above: import job,
  approval-gated activation/rollback, reader + cache, doctor, CLI workhorse.
- **`crates/{tools,tool-registry,citations}`** — the tool contract, the two
  read-only tools, and citation resolution.
- **`crates/{cli,server}`** — the two Phase-1 interfaces.

Dependency direction is enforced by `cargo xtask arch-check` against
`xtask/allowlist.toml` (name-keyed, fail-closed). `quran-core` and
`quran-corpus` may **not** depend on `storage`, `llm`, `embeddings`,
`retrieval`, or any vector store (invariant I2).

## 2. Data model (migrations `0007`–`0012`)

| Migration | Tables |
|---|---|
| `0007_quran_editions` | `quran_editions`, `quran_surahs`, `quran_ayahs` |
| `0008_quran_structure` | `quran_segments`, `quran_tokens`, `quran_token_separators` |
| `0009_quran_divisions` | `quran_divisions`, `quran_ayah_divisions` |
| `0010_quran_translations` | `translation_editions`, `translation_passages`, `word_glosses` |
| `0011_quran_staging` | `quran_stg_*` staging tables |
| `0012_quran_validation` | `quran_validation_reports`, `quran_import_runs`, citations |

Key rules:

- **I1 — insert-only.** `UPDATE`/`DELETE` on canonical tables aborts with a
  coded error via triggers; deactivation uses tombstones.
- **I3 — `edition_id` in every key.** No canonical row is addressable without
  its edition.
- **Provenance is structural.** Every ayah and every translation passage carries
  a `provenance_id` FK into `provenance_records`; a translation is
  `NOT NULL` attributed with a non-empty `translator` CHECK (principle 5).
- **Numbering.** Ayah numbers are dense within a surah. Divisions are numbered
  **globally per kind** (`(kind, number)` is the PK); `ruku`/`rub` are cumulative
  — see DEV-03.

## 3. Ingest pipeline

`quran-corpus::import::run_import` drives 13 checkpoints, each a resume point
(`ImportCheckpoint` in `crates/quran-corpus/src/import.rs`):

1. `Claimed`, 2. `ManifestHashed`, 3. `AdapterSelected`, 4. `Parsed`,
5. `UnicodeAudited`, 6. `Validated`, 7. `Tokenized`, 8. `HashesComputed`,
9. `Staged`, 10. `RoundtripVerified`, 11. `ReferenceCompared`,
plus terminal success/cancellation bookkeeping.

Guarantees:

- **Restart is resume.** A re-run re-derives completed checkpoints from the
  staged rows and continues; it never double-writes.
- **Staging is physically separate.** The importer writes only `quran_stg_*`;
  it holds no `ApprovalToken` and cannot touch canonical rows (I5).
- **Cancellation is clean.** Cancelling removes staging rows and records the
  cancellation.
- **Validation never fails fast.** All QV-001…028 run; the report counts
  `Fatal`/`Error`/`Warning`. A non-zero `Fatal` count blocks activation.

## 4. Hashing (frozen, ADR-0108)

Three hashes are computed once at import and re-verified on demand
(`crates/quran-corpus/src/hashing.rs`):

| Hash | Over |
|---|---|
| `text_hash` | the canonical text, NFC, in ayah order |
| `structure_hash` | the surah/ayah/segment/token skeleton |
| `token_order_hash` | the token order + separators |

They are stored as `ContentHash` (`algorithm` tag + lowercase hex) so the recipe
can be versioned without ambiguity. **Changing a recipe is a
retrofit-impossible decision** — it requires an ADR and a rehash procedure.

## 5. Read path

`application::quran_reader` is the only supported read surface:

- Resolves references through the `quran-core` grammar.
- Expands ayahs, ranges, surahs, divisions, and tokens.
- Builds `AyahView` with attributed translations (rendered only when
  explicitly requested — quotation and translation are never conflated).
- Caches results keyed on
  `(edition_id, version, corpus_generation, reference, options_hash)`; a
  generation change invalidates wholesale, so no stale text is served after an
  activation (AC-P1-19).

`crates/tools` and `crates/server` both consume this reader; no interface reads
canonical tables directly except the debug reader and the reader's own SQL.

## 6. Corpus generation

Activation flips a pointer and bumps `corpus_generation` in one transaction
(I7). Consumers (cache, and in later phases indexes) treat the generation as the
freshness key. Rollback to a prior version also bumps the generation, so caches
and projections re-derive rather than trust a stale bag of rows.

## 7. Where to look

| Concern | File |
|---|---|
| Reference grammar | `crates/quran-core/src/reference/{parser,serializer}.rs` |
| Quotation guard (I6) | `crates/quran-core/src/quotation.rs` |
| Tokenizer | `crates/quran-corpus/src/tokenize.rs` |
| Hashing | `crates/quran-corpus/src/hashing.rs` |
| Unicode audit | `crates/quran-corpus/src/unicode.rs` |
| Validators QV-001…028 | `crates/quran-corpus/src/validation.rs` |
| Importer / differ | `crates/quran-corpus/src/{import,differ}.rs` |
| Repository traits | `crates/storage/src/quran.rs` |
| SQLite implementation | `crates/storage-sqlite/src/quran.rs` |
| Reader + cache | `crates/application/src/quran_reader.rs` |
| Import / activation services | `crates/application/src/quran.rs` |
| Doctor checks | `crates/application/src/quran_doctor.rs` |
