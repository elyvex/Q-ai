# ADR-0006 — Hashing & Canonical Serialization: SHA-256, Canonical JSON, NFC Policy

- Status: Accepted
- Phase: 0 — Foundations
- Date: 2026-09-11
- Related decisions: ADR-0001, ADR-0008, ADR-0009
- Requirements: PRD §§6–7, 8, 12.1, 22.4, 38, 73, 82

## Context

Q-ai requires deterministic, reproducible content hashing and canonical serialization for all scriptural text, source manifests, provenance records, and audit events.
Because Islamic scripture, hadith texts, and scholarly commentaries are sensitive to orthographic, character encoding, and structural alterations, hashing must be cryptographically secure, invariant to serialization details, and resistant to platform/compiler variations.

Specifically, we must define:
1. The standard cryptographic hashing algorithms supported by the platform.
2. A canonical JSON serialization algorithm (`canonical_json_bytes`) that guarantees identical byte output across all languages, OSs, and map insertion orders.
3. Unicode normalization policy for Arabic and comparative scriptural text before hashing.

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| **SHA-256 + Canonical JSON (RFC 8785) + Unicode NFC** | Standardized, deterministic, cryptographically strong, hardware-accelerated | Slightly higher byte overhead than binary protocols (e.g., Protobuf/Cap'n Proto) |
| **BLAKE3 + BSON** | Fast hashing | BSON has non-standard implementations across platforms; less widespread canonical standardization |
| **Unconstrained JSON (`serde_json::to_vec`)** | Simple, standard library | Key ordering is non-deterministic in standard maps; white space and formatting differences break hash reproducibility |
| **Protobuf / MessagePack** | Binary efficiency | Binary serialization can suffer from field ordering and default value handling quirks |

## Decision

We adopt **SHA-256 as the primary hash algorithm**, wrapped in an explicit `ContentHash` type tagged with `HashAlgorithm::Sha256` (and `Blake3` reserved for performance-critical local caches).

All canonical JSON serialization uses `canonical_json_bytes<T>(value: &T)` in `crates/domain/src/hashing.rs`, conforming to the following rules:
1. **Key Ordering:** Object keys are sorted lexicographically by code point (enforced via `BTreeMap` or sorted serialization).
2. **Whitespace:** Minimal formatting (no whitespace between tokens, e.g., `,` and `:` without surrounding spaces).
3. **Encoding:** UTF-8 without BOM (Byte Order Mark).
4. **Newlines:** Unix LF (`\n`), never CRLF (`\r\n`).
5. **Unicode Normalization:** All text strings in canonical structures are normalized to **Unicode Normalization Form C (NFC)** prior to serialization.

## Accuracy and Religious-Source Implications

- Hashing canonical text guarantees that any modification (whether accidental corruption or deliberate tampering) changes the `ContentHash` and immediately fails validation.
- Canonical JSON and NFC normalization ensure that two identical Quranic verses produce the exact same `ContentHash` across all platforms and environments.
- Displayed Arabic canonical text preserves exact diacritics and Uthmani orthography; NFC normalization is applied strictly at the Unicode code point level without modifying character identities or harakat.

## Licensing Implications

- The implementation relies on standard Rust crates (`serde_json`, `sha2`, `unicode-normalization`), all licensed under MIT/Apache-2.0.

## Security Implications

- Cryptographic hashes prevent tampered source manifests or modified canonical text from entering the system.
- Audit chains rely on `ContentHash` to form an unforgeable tamper-evident chain (ADR-0009).

## Operational Implications

- Hashes generated in Phase 0 remain valid in all future phases.
- `ContentHash` is represented in hex format (`[a-f0-9]`), lowercase, validated for even length.

## Migration Strategy

If a future hash algorithm is added:
- The `ContentHash` type retains its `algorithm: HashAlgorithm` field, allowing legacy `Sha256` hashes to coexist with new algorithms without breaking database constraints.

## Reversal Cost

High.
Changing the canonical serialization format or default hash algorithm would invalidate all stored `ContentHash` values across the database, requiring a global re-hashing migration.

## Acceptance Criteria

- `canonical_json_bytes` produces byte-identical output for maps regardless of key insertion order.
- `ContentHash::try_new` rejects odd-length or non-hex strings.
- Property tests in `crates/domain/src/hashing.rs` pass cleanly.
