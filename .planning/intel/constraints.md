# Constraints (synthesized from 2 SPECs)

## Hashing Spec: SHA-256 algorithm tag format and canonical JSON bytes
- source: docs/architecture/hashing-spec.md
- type: schema
- content: All content hashes use the explicit format `sha256:<hex>` (lowercase algorithm identifier, lowercase even-length hex) forming the `ContentHash` type in `crates/domain/src/hashing.rs`; `canonical_json_bytes` is deterministic via lexicographically sorted keys, minimal whitespace, UTF-8 without BOM, LF newlines, and NFC normalization of strings; used for `quoted_text_hash`, `source_versions.content_hash`, audit chain hashes, and manifest signing; hashes from Phase 0 remain valid in all future phases, and changing the format or default algorithm requires a global re-hashing migration.

## Quran Citation Specification (D1.14)
- source: docs/07-technical/quran-citation-spec.md
- type: api-contract
- content: Identifier forms — canonical `quran:{slug}@{version}:{surah}:{ayah}`, human shorthand (`2:255`), deep link `/read/{slug}@{version}/{surah}:{ayah}`, URN `qai://quran/{slug}@{version}/{surah}:{ayah}`; slug lowercase, version SemVer, shorthand resolved against the active edition and expanded to pinned form before storage; stored citation pins id, canonical reference, edition_ref, location_json, quoted_text_hash, content_hash, ingestion_version; `CitationResolver::resolve` returns exactly one QuotationVerdict (ExactMatch, MatchAfterWhitespaceNormalization, MatchAfterDeclaredNormalization{rules}, Mismatch{first_difference_at, expected_hash}, LocationNotFound, EditionNotFound, AccessDenied) with Mismatch and LocationNotFound as hard failures on any answer path; resolution algorithm parse → resolve edition → confirm location → compare text → return citation with text_hash and deep_link; v1 verifies single ayahs only and non-ayah locators are documented LocationNotFound; surfaces `qai quran resolve`, `qai quran get`, `GET /api/v1/quran/citations/{id}`; invariants that quoted canonical text travels only inside `QuranQuotation`, deep-link/URN formats are frozen, and citations are re-verified on use.
