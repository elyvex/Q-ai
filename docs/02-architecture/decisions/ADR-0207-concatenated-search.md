# ADR-0207 — Concatenated-Search Architecture (Skeleton + Trigram + Verify)

- Status: Proposed (implementation complete; acceptance pending owner review)
- Phase: 2 — Quran Search and Linguistics
- Date: 2026-09-23
- Related decisions: ADR-0201 (FTS), ADR-0208 (SpanMap), ADR-0204 (rules)
- Requirements: plan §4.4; AC-P2-08; D2.4
- Implementation: `crates/quran-search/src/skeleton.rs`, `trigram.rs`;
  `application::quran_search::{search_concatenated, verify_concatenated,
  verify_concatenated_window}`; suites `index_lifecycle.rs` (parity),
  `search_goldens.rs`

## Context

Users type spaceless queries (`بسمالله`) expecting 1:1 with a segmentation
explanation. Naive substring search over the corpus is unbounded; FTS cannot
express spaceless matching across token boundaries and ayah boundaries.

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| Skeleton + trigram recall + exact verify (chosen) | Bounded recall; precision from exact substring verification; every hit carries segmentation + boundary flags | Posting index to build and retain; short queries bypass postings |
| Full scan per query | No index | Latency unbounded on full corpus |
| FTS n-grams | Reuses engine | Tokenizer-boundary semantics wrong for spaceless text; span fidelity loss |

## Decision

- Index time: L6 skeletons per ayah + per 3-ayah window (surah-scoped,
  joined raw texts normalized — never concatenated skeletons), stored in
  `quran_skeleton` rows; character-trigram postings in
  `<root>/gen-<N>/trigram.db` (T36).
- Query time: normalize the query through L6 (spaces vanish by design) →
  trigram recall (all query trigrams must occur; short queries and pre-T36
  generations fall back to the Rust scan — same recall, slower) → exact
  substring verification → re-normalization check → per-token segmentation.
- Cross-ayah: window matches split into one hit per overlapped ayah, each
  labeled `spans_ayah_boundary = true`; ayah-level hits always win dedup so
  no reference appears twice and no cross-verse fragment is presented as one
  verse. `allow_cross_ayah=false` and over-budget windows serve ayah-local
  only.
- Posting/query parity is test-locked: identical references through the
  posting index and the scan fallback (`index_lifecycle.rs`).

## Accuracy and Religious-Source Implications

A cross-verse fragment presented as one verse would misquote the mushaf.
Mitigations: `spans_ayah_boundary` labels, ayah-wins dedup, segmentation
explaining every query part against canonical tokens 1..2 (AC-P2-08 shape).

## Licensing Implications

None — mechanics over stored derived forms.

## Security Implications

Trigram recall is bounded (indexed lookups + intersection); verification is
linear in the recall set. No regex engine involved; I16 budgets do not apply.

## Operational Implications

`trigram.db` rides generation lifecycle (built during staging, verified by
sampled self-recall, removed by retention GC). Missing file = scan fallback,
never an error (rolling upgrades safe).

## Migration Strategy

Pre-T36 generations serve via fallback; next rebuild writes postings. No
dedicated migration (generation-scoped files, not catalog rows).

## Reversal Cost

Low. Delete the posting build stage and the reader uses the scan path
permanently (latency cost only).

## Acceptance Criteria

- `بسمالله` → 1:1 with token-1/token-2 segmentation (AC-P2-08 mechanics).
- Cross-ayah windows labeled, deduped, budgeted.
- Postings/scan parity suite green.
