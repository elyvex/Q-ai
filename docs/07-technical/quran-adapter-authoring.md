# Quran Edition Adapter Authoring Guide (D1.14)

> **Audience:** anyone adding a new dataset shape for Quran editions.
> **Related:** `docs/07-technical/quran-corpus-architecture.md`,
> `docs/schemas/quran-edition-source.v1.schema.json`.

## 1. What an adapter is

An adapter turns a raw dataset string into the **intermediate format**
(`quran_corpus::format::EditionSource`) — a shape-independent description of an
edition: metadata, surahs, ayahs, optional segments, divisions, and tokens. The
importer, validator, tokenizer, and hasher all operate on that intermediate
format, so an adapter is the *only* thing that has to know a new file layout.

Trait (`crates/quran-corpus/src/adapters.rs`):

```rust
pub trait Adapter {
    fn name(&self) -> &'static str;
    fn parse(&self, input: &str) -> Result<EditionSource, CorpusError>;
}
```

`parse_with_adapter(name, input)` dispatches by name. Two adapters ship today,
chosen to prove the trait covers genuinely different shapes:

- `json` — document-oriented: one manifest with nested surahs/ayahs.
- `csv` — row-oriented: a flat `surah,ayah,text` table plus a sidecar metadata
  block. This is why JSON+CSV is enough evidence of extensibility (DEV-01).

## 2. The intermediate format contract

Your adapter must produce an `EditionSource` whose:

- `edition.slug` / `edition.version` are stable identifiers (a re-import of the
  same bytes must produce the same pair).
- `edition.script` and `edition.verse_numbering_scheme` are declared.
- Suwar are 1…N contiguous; ayah numbers are dense within each surah.
- Text is **NFC**, Uthmani-or-declared script, no presentation ligatures.

Do **not** normalize, strip diacritics, or "fix" the text in an adapter. The
tokenizer and Unicode auditor downstream are responsible for offsets and
findings; an adapter that silently edits text corrupts the corpus hash.

## 3. Adding an adapter — checklist

1. Create `crates/quran-corpus/src/adapters.rs`-style parser and implement
   `Adapter` (`name` must be unique and lowercase).
2. Register it in `parse_with_adapter`.
3. If it needs a JSON Schema for its manifest, add
   `docs/schemas/<name>.v1.schema.json` and wire `cargo xtask gen-schema`.
4. Add fixtures under `fixtures/quran/`:
   - one **benign** fixture that imports cleanly;
   - at least one **adversarial** fixture per QV concern you can trigger
     (missing surah, empty ayah, non-NFC text, duplicate ayah, …).
5. Add a test that the benign fixture imports to `Staged` with zero `Fatal`
   findings and that each adversarial fixture fails with the **specific** QV id.
6. Add the adapter name to the CLI's `--adapter` documentation and to this guide.

## 4. Manifest schema

The reference manifest shape is
`docs/schemas/quran-edition-source.v1.schema.json`; the fixture
`fixtures/quran/test-edition-min/manifest.json` is a minimal valid instance.
Validate any new manifest before importing:

```bash
cargo run -p xtask -- validate <manifest.json> docs/schemas/quran-edition-source.v1.schema.json
```

## 5. Failure modes to test

Adapters are the first line against malformed datasets. Ensure your adapter
returns a typed `CorpusError` (never panics) for:

- unreadable/undecodable bytes;
- missing required metadata fields;
- a surah with no ayahs, or an ayah with empty text;
- ayah numbers that are not dense/contiguous;
- duplicate `(surah, ayah)` pairs;
- text that is not NFC;
- a total count that disagrees with a declared count.

A panic in an adapter is a **corpus-integrity** failure: it can crash an import
mid-checkpoint. Prefer `Result` everywhere.

## 6. Do not

- Do not write canonical tables from an adapter.
- Do not add a dependency on storage, LLM, embeddings, retrieval, or a vector
  store — `quran-corpus` is checked by `xtask arch-check` (I2).
- Do not invent text. If the source is missing an ayah, fail with a coded error;
  filling gaps silently defeats the no-fabrication principle.
