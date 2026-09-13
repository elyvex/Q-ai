# Hashing Spec: SHA-256 Algorithm Tag Format `sha256:<hex>`, Canonical JSON Bytes Definition

## Algorithm Tag Format

All content hashes in Q-ai use the explicit format:
```
sha256:<hex>
```
where:
- `sha256` is the literal algorithm identifier (lowercase)
- `<hex>` is a lowercase hexadecimal string with even length (validated per byte)
- Together they form a `ContentHash` type (`crates/domain/src/hashing.rs`)

Example: `sha256:3a7bd3e2360a...`

## Canonical JSON Bytes Definition

The `canonical_json_bytes<T>(value: &T)` function in `crates/domain/src/hashing.rs` produces deterministic byte output by applying these rules to the input value before serialization:

1. **Key Ordering**: Object keys are sorted lexicographically by Unicode code point (using `BTreeMap` or equivalent sorted serialization)
2. **Whitespace**: Minimal formatting - no whitespace between tokens (e.g., `,` and `:` appear without surrounding spaces)
3. **Encoding**: UTF-8 without Byte Order Mark (BOM)
4. **Newlines**: Unix line feed (`\n`), never carriage return + line feed (`\r\n`)
5. **Unicode Normalization**: All string values are normalized to Unicode Normalization Form C (NFC) prior to serialization

This ensures that:
- Identical JSON documents always produce identical byte sequences regardless of insertion order, whitespace, or platform
- Cryptographic hashes of the output are reproducible forever
- The NFC normalization preserves Arabic diacritics while ensuring canonical equivalence

## Usage in the Codebase

- Stored in `provenance_records.quoted_text_hash` for quotation verification
- Used in `source_versions.content_hash` for source file integrity
- Required for all audit event chain hashes (ADR-0009)
- Used in manifest signing (ADR-0007) over canonical JSON bytes
- Basis for all `ContentHash` types throughout the domain layer

## Accuracy & Religious-Source Implications

Any alteration to Quranic text (whether accidental corruption or intentional tampering) changes the `sha256:<hex>` hash and immediately fails validation against the stored hash. This is the primary technical mechanism ensuring that canonical text remains mathematically immutable once approved.

The canonical JSON + NFC policy guarantees that two identical Quranic verses produce the exact same hash across all languages, operating systems, and serialization libraries.

## Licensing Implications

Relies on standard Rust crates: `serde_json` (MIT/Apache-2.0), `sha2` (MIT/Apache-2.0), `unicode-normalization` (MIT/Apache-2.0). No licensing concerns.

## Security Implications

- Prevents supply-chain attacks on imported scripture data (manifest signing over canonical JSON)
- Audit chains rely on `ContentHash` to form an unforgeable tamper-evident sequence
- The hash algorithm tag (`sha256`) enables future algorithm agility without breaking existing hashes

## Operational Implications

- Hashes generated in Phase 0 remain valid in all future phases
- Represented as lowercase hex strings, validated for even length on parsing
- To change the canonical serialization format or default hash algorithm would require a global re-hashing migration (high reversal cost)

## Migration Strategy

If a future hash algorithm is added:
- The `ContentHash` type retains its `algorithm: HashAlgorithm` field, allowing legacy `Sha256` hashes to coexist with new algorithms without breaking database constraints
- All validation code checks the algorithm field before comparing hex values