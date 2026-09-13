# ADR-0007 — Source Manifest Format & Signing (ed25519 Detached over Canonical JSON)

- Status: Accepted
- Phase: 0 — Foundations
- Date: 2026-09-11
- Related decisions: ADR-0006, ADR-0008
- Requirements: PRD §§22, 38, 76

## Context

Q-ai imports scripture editions, hadith collections, and tafsir corpora via a
`source-manifest.v1.json` file. Different publishers may supply the same text
at different quality; the system must cryptographically verify that a manifest
has not been tampered with since its author signed it. Local imports must work
without network access, so the signing scheme must be offline-verifiable.

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| **ed25519 detached over canonical JSON** | Fast, small signatures, deterministic, offline-verifiable, ties signing to the canonical representation (ADR-0006) | Key management overhead for publishers |
| Ed25519 over raw bytes | Simpler for publishers | Byte representation is not canonical; two byte-identical manifests can hash differently if re-serialized |
| RSA-4096 over JSON | Familiar, wide library support | Slower keys, larger signatures, no canonical-JSON tie |
| JWS (RFC 7515) | Standardized | JWS payload encoding is not canonical; tooling multiplies implementation surface |
| No signing (local files only) | Zero friction | Only acceptable for local files with `allow_unsigned_manifests = true` (default `true` locally, `false` remotely) |

## Decision

Use **ed25519 detached signatures** computed over the canonical JSON bytes
(ADR-0006) of the manifest with the `signature` block removed and fields
sorted. The manifest embeds:

- `publisher.signing_key_id` (e.g. `ed25519:9f2c...`)
- `signature.algorithm = "ed25519"`, `key_id`, `value` (base64), `signed_fields`
  (a pointer to which canonical serialization was signed).

Verification (`sources::signature`):

- Canonicalize the manifest by removing the `signature` object, computing
  `canonical_json_bytes`, and verifying the detached ed25519 signature with the
  publisher's published public key (out-of-band key discovery for Phase 0;
  signed key registry from Phase 1).
- `signature = null` is allowed **only** when
  `security.allow_unsigned_manifests = true` (default `true` for local file
  imports, `false` for remote/network-sourced manifests). Unsigned manifests
  can never become `Active` in Phase 0 regardless of policy (source state
  machine still requires a verified `ContentHash`).

## Accuracy and Religious-Source Implications

- Tampering with a single ayah in an imported edition is detected by the hash
  and signature verification; importing a tampered edition is rejected with a
  `QAI-SRC-…` code (AC-P0-09).
- Publishers of Islamic content are encouraged (not required in Phase 0) to sign;
  unsigned local test fixtures remain importable.

## Licensing Implications

ed25519 signing via `ed25519-dalek` (MIT/Apache-2.0). No licensing impact.

## Security Implications

- Signing protects against supply-chain attacks on imported scripture data.
- Publisher signing keys must be kept offline; only public keys are distributed.
- A future revocation mechanism (key rotation list) is reserved; the `signing_key_id`
  field supports key rotation without manifest rewrite.

## Operational Implications

- `qai source import <manifest>` signs during verification and emits an audit event
  (`source.imported` with manifest hash).
- `qai source validate <staged-version>` re-verifies signature + hash on demand.

## Migration Strategy

Manifest schema version `1.0.0` is fixed in Phase 0. Future schema versions bump
`manifest_version` and are validated against the JSON Schema in
`docs/schemas/source-manifest.vN.schema.json`.

## Reversal Cost

Medium. Removing the signing requirement would silently weaken trust in all
`Active` sources; a controlled deprecation would require re-validating every
source that became `Active` under a now-untrusted key.

## Acceptance Criteria

- Signed local manifest imports to `Staged`; tampered manifest rejected with `QAI-SRC-…`.
- Unsigned remote manifest rejected under default policy.
- `qai source validate` re-runs signature verification end-to-end.
