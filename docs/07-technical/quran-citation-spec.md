# Quran Citation Specification (D1.14)

> **Status:** frozen in Phase 1 (ADR-0111). Later phases reuse this identity.
> **Related:** `crates/citations/src/lib.rs`, `crates/quran-core/src/reference/`.

## 1. Identifiers

A canonical reference addresses one ayah (or a range) inside one edition:

| Form | Example | Use |
|---|---|---|
| Canonical reference | `quran:{slug}@{version}:{surah}:{ayah}` | Machine identity; `resolved.reference` |
| Human shorthand | `2:255`, `quran:2:255-257` | CLI/API input; resolves against the active edition |
| Deep link | `/read/{slug}@{version}/{surah}:{ayah}` | Web reader |
| URN | `qai://quran/{slug}@{version}/{surah}:{ayah}` | App-internal links |

Rules:

- `slug` is lowercase; `version` is a `SemVer`.
- A **pinned** reference names a version explicitly. A **shorthand** reference
  resolves against the currently active edition and is expanded to the pinned
  form before being stored or returned.
- Ayah ranges use an ASCII hyphen: `2:255-257`.

## 2. Citation record

A stored citation pins everything needed to re-verify it later:

| Field | Meaning |
|---|---|
| `id` | Stable citation id |
| `canonical_reference` | Pinned `quran:…` reference |
| `edition_ref` | `slug@version` |
| `location_json` | `{ "surah": …, "ayah": … }` (and range end where relevant) |
| `quoted_text_hash` | Hash of the quoted text at creation |
| `content_hash` | Canonical `text_hash` at creation |
| `ingestion_version` | Corpus generation at creation |

Persisting these is what makes re-verification possible after a re-import or an
activation: the resolver re-runs the verdict against the *current* canonical
rows and reports drift rather than trusting the stored hash.

## 3. Verdicts

`CitationResolver::resolve` (and `resolve_stored`) returns exactly one of
(`QuotationVerdict`, `crates/citations/src/lib.rs`):

| Verdict | Meaning | Answer-path handling |
|---|---|---|
| `ExactMatch` | Byte-identical | OK |
| `MatchAfterWhitespaceNormalization` | Equal modulo whitespace | OK, note the transform |
| `MatchAfterDeclaredNormalization { rules }` | Equal after listed rules | OK, list the rules |
| `Mismatch { first_difference_at, expected_hash }` | Different text | **Hard failure** |
| `LocationNotFound` | Location does not resolve | **Hard failure** |
| `EditionNotFound` | Edition unknown/inactive | **Hard failure** |
| `AccessDenied` | Reserved (v1 never denies single-user) | Deny |

`Mismatch` and `LocationNotFound` are **hard failures on any answer path** — the
mechanism Phase 9's citation verifier reuses. They must never be downgraded to a
warning or softened into a paraphrase.

## 4. Resolution algorithm

1. Parse the reference via the `quran-core` grammar → `(surah, ayah, range)`.
2. Resolve the edition (explicit version, or the active one).
3. Confirm the location exists (ayah in range for that surah).
4. Compare the quoted text against the canonical text:
   - exact → `ExactMatch`;
   - whitespace-only differences → `MatchAfterWhitespaceNormalization`;
   - differences removed by the declared `normalization_rules` →
     `MatchAfterDeclaredNormalization`;
   - otherwise → `Mismatch` with `first_difference_at` + `expected_hash`.
5. Return the resolved citation with `text_hash` and `deep_link`.

v1 verifies single ayahs. Any non-ayah locator (whole surah, page, …) is a
**documented `LocationNotFound`** — safe, never a silent pass.

## 5. CLI / API surface

- `qai quran resolve <reference>` — parse + bounds-check; prints the canonical
  reference and the active edition.
- `qai quran get <reference>` — prints text and the provenance citation line.
- `GET /api/v1/quran/citations/{id}` — resolves a persisted citation.

## 6. Invariants

- Quoted canonical text only ever travels inside a `QuranQuotation`
  (invariant I6); a raw string literal is not a citation.
- The deep-link/URN formats are frozen; changing them is an ADR-level decision
  because stored citations reference them.
- A citation is re-verified on use; the stored hash is a comparison baseline,
  never a substitute for re-reading canonical rows.
