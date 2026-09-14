# ADR-0111 — Citation Identity and Deep-Link Format (Draft)

- Status: **Draft** (accepted when the resolver ships, P1-T46/T47)
- Phase: 1 — Canonical Quran Core
- Date: 2026-09-14
- Related decisions: ADR-0102, ADR-0108
- Requirements: PRD §13.3, §21.2, §35.3

## Context

Citations must be stable, resolvable, and re-verifiable years later. The
identity format must be fixed before the first citation is stored.

## Decision (proposed)

- Canonical reference: `quran:{slug}@{version}:{surah}:{ayah}` (ADR-0102).
- Deep link: `/read/{slug}@{version}/{surah}:{ayah}[?highlight=token:{n}]`.
- URN for storage: `qai://quran/{slug}@{version}/{surah}:{ayah}`.
- Every quotation carries edition id + version + hash (I6, enforced by the
  `QuranQuotation` constructor); `verify_quotation` returns
  `ExactMatch | MatchAfterWhitespaceNormalization |
  MatchAfterDeclaredNormalization{rules} | Mismatch{…} | LocationNotFound |
  EditionNotFound | AccessDenied`, and mismatch is a hard failure on answer paths.

## Consequences

- `citations` crate + `citations` table (`content_hash` + `ingestion_version`)
  land in M9 (P1-T46/T47); this ADR is accepted then.
