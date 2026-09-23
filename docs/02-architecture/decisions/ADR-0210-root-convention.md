# ADR-0210 — Root Convention + Cross-Dataset Unification as Suggestions

- Status: **Draft — pending dataset + linguist inputs (do not mark Accepted)**
- Phase: 2 — Quran Search and Linguistics
- Date: 2026-09-23
- Owner: _unassigned_ (P2-X01/X02/X03 swimlane)
- Related decisions: ADR-0203 (dataset), ADR-0215 (tagset), ADR-0209 (multi-analysis)
- Requirements: plan §6.2; AC-P2-22
- Implementation: pending morphology import (Sprint 2.4); public-domain test
  lexicon ships as fallback per ADR-0203

## Context

Different morphology datasets spell the same root differently (hamza
variants, alif forms, weakly-attested radicals). Merging across datasets
would fabricate scholarly consensus; keeping every spelling isolated makes
cross-dataset root search useless.

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| Hub topology + suggestion queue, never merge (chosen direction) | Ayahs reach roots one-hop via tokens (no ayah×ayah edges); competing spellings stay separate with dataset identity; unification is a reviewable suggestion | Review queue needs scholar attention to converge |
| Canonical root merge at import | Simple queries | Silent consensus fabrication; breaks I11 |

## Decision (draft)

Roots/lemmas keep dataset identity and alignment forever. Same-root
relations across datasets are `review_queue` suggestions with evidence,
promoted to `ScholarVerified` only through the review flow with a recorded
reviewer, timestamp, and displayed evidence (AC-P2-22, AC-P2-27). Graph
projection (Phase 4, TASK-405) uses hub topology: no ayah×ayah edges.

## Open inputs (blocking acceptance)

- Licensed dataset + its native root inventory (P2-X01/ADR-0203).
- Linguist-authored root convention + unification policy (P2-X03).
- Curated family set review (120 families, P2-T92).

## Accuracy and Religious-Source Implications

SUBSTANTIVE: root claims are scholarly claims. Never generate roots
(plan §2.3); never merge without review; computational suggestions carry the
mandatory "not verified scholarship" label and are off by default
(AC-P2-26).

## Licensing Implications

Root inventories derive from the licensed dataset — attribution string
required (ADR-0203); test lexicon is public-domain and labeled as such.

## Security Implications

Review queue is the only promotion path (no auto-verify; suggestions never
become `ScholarVerified` computationally).

## Operational Implications

Dataset version change trips root/lemma drift checks (`doctor`); generation
stamps on every linguistic row.

## Migration Strategy

Lexicon migrations `0017`/`0018` carry roots/lemmas/staging; convention
change = new dataset version + reimport + drift report.

## Reversal Cost

Medium. Convention change re-opens the review queue.

## Acceptance Criteria

- AC-P2-22/23/27: suggestions never merged; 300 root + 200 lemma cases at
  gated precision/recall; promotion only via review queue.
