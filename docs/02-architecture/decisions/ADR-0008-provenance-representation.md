# ADR-0008 — Provenance Representation: Single Universal Record + Typed Attribution

- Status: Accepted
- Phase: 0 — Foundations
- Date: 2026-09-11
- Related decisions: ADR-0001, ADR-0006, ADR-0007
- Requirements: PRD §§6, 76, 82

## Context

Every non-canonical assertion in Q-ai (scholarly note, AI annotation,
machine-generated gloss, user bookmark) must carry its origin: where it came
from, who/what produced it, under what trust level, and what versions of source
it depends on. A per-domain table design proliferates quickly; a universal
record is flexible but harder to query efficiently.

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| **Single universal `provenance_records` table** | One insert path; easy to query all assertions for a subject; extensible via `attribution_json` | Requires careful indexing; JSONB columns less queryable than typed columns |
| Per-domain tables (`scholarly_provenance`, `ai_provenance`, …) | Typed, queryable columns per domain | Schema proliferates; cross-domain queries require UNION; new domains need new tables |
| Graph-based provenance (triples) | Natural lineage queries | Heavy for a local-first install; joins become expensive |

## Decision

Use a **single universal provenance record** (`provenance_records` table) with:

- A closed `layer` enum: `canonical_source`, `publisher_metadata`,
  `scholarly_annotation`, `computational_annotation`, `user_or_ai_note` (PRD §6).
- A closed `attribution_kind` enum (`dataset`, `scholar`, `computational`, `user`)
  plus a typed `attribution_json` blob holding the per-kind fields.
- Typed `VerificationStatus` and `Confidence` columns with DB `CHECK` constraints.
- `versions: DerivationVersions` (PRD §76) stored as JSON — every derived artifact
  records the version of everything it was derived from.
- `source_version_id`, `source_location` (including `quoted_text_hash` for
  quotation verification §35.3) for traceability.

**Type-level invariants (also DB `CHECK` constraints):**

1. `layer = computational_annotation` ⇒ `Attribution::Computational` and
   `confidence.is_some()` and `verification != HumanVerified` unless
   `reviewed_by.is_some()`.
2. `layer = scholarly_annotation` ⇒ `Attribution::Scholar` with non-empty name.
3. `layer = canonical_source` ⇒ `source_version_id.is_some()`; rows insert-only
   (enforced by triggers).
4. Records whose subject is a canonical row require `ApprovalToken` to write
   (enforced at the repository layer, not in SQL).

## Accuracy and Religious-Source Implications

- Provenance is the primary mechanism by which Q-ai distinguishes **quotation**
  (verifiable hash), **source summary** (dataset attribution), and **AI analysis**
  (computational attribution with confidence). Getting this wrong makes every
  downstream answer untrustworthy.
- Hadith and tafsir gradings in Phase 5/6 will extend `attribution_json`; the
  universal record format must stay extensible.

## Licensing Implications

`serde_json` (MIT/Apache-2.0). No impact.

## Security Implications

- `provenance_records` is append-only + update/delete-blocked by triggers
  (ADR-0009).
- `review_note` and `reviewer` fields are user-controlled text — routed through
  the global redaction layer if they accidentally contain a secret.

## Operational Implications

- Queries by `(subject_urn, layer)` need a composite index; the table is expected
  to grow large (every annotation carries a record).
- Provenance is written inside the same `UnitOfWork` as the authoritative change
  (atomicity).

## Migration Strategy

Created by `0003_provenance.up.sql`. Schema version bumps only if the `layer` or
`attribution_kind` enum changes — which is a breaking change requiring a
`0003_provenance` rewrite migration (forward-only + audit event per ADR-0002).

## Reversal Cost

High. Removing the universal record would require migrating every annotation
row to a per-domain table — a multi-day data migration with a maintenance window.

## Acceptance Criteria

- `layer = computational_annotation` rows without `confidence` are rejected by DB `CHECK`.
- `layer = computational_annotation` rows reaching `human_verified` without `reviewed_by` are rejected.
- Canonical provenance rows cannot be updated or deleted (trigger-tested).
