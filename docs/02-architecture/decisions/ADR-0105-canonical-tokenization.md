# ADR-0105 — Canonical Tokenization Rule (Whitespace-Preserving Surface Tokens)

- Status: Accepted
- Phase: 1 — Canonical Quran Core
- Date: 2026-09-14
- Related decisions: ADR-0104, ADR-0108
- Requirements: PRD §7.1, §35.1 (QV-010…QV-012)

## Context

Phase 1 stores surface tokens only; morphology arrives in Phase 2 and must
attach to token positions without re-tokenizing (acceptance §7). The rule must
be lossless: `reconstruct(tokens, separators) == text` byte-for-byte, or the
corpus is not canonical.

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| Whitespace split, separators recorded exactly | Lossless by construction; trivially reviewable | Waqf marks need an explicit rule |
| Morphological segmentation now | Richer tokens immediately | Conflates canonical storage with analysis; Phase-2 rework |
| Split waqf marks always / never | Simple rule | Combining marks share grapheme clusters; either choice corrupts offsets somewhere |

## Decision

Split per character, map offsets back to grapheme clusters:

- Unicode whitespace runs are separators (recorded exactly, always ending the
  open token — each separator row sits `after_position` of a token).
- Quranic annotation signs U+06D6…U+06ED form their own tokens
  (`is_pause_mark`), **except** a mark following a word character attaches to
  that word (combining marks share the cluster; splitting them would create
  overlapping spans).
- Everything else accumulates into word tokens.
- Positions are dense `1..=k` in row order (QV-010 is order-sensitive, which is
  what catches permuted positions).

Adapter-supplied tokens are **verified, never trusted**: they are checked
against recomputed separators and offsets (QV-010…012) and the importer stores
its own tokenization.

## Accuracy / Religious-source implications

The tokenizer never alters order or content; separators preserve even unusual
whitespace and mark-separated tokens. A lossy tokenizer would silently rewrite
revelation — hence the byte-equality gate (AC-P1-05) with no whitelist.

## Licensing / Security implications

None beyond input hygiene (untrusted bytes until validated).

## Operational / Migration implications

`quran_corpus::{tokenize, reconstruct}` plus a losslessness proptest over
arbitrary strings. Phase 2 attaches lemmas/roots to `(surah, ayah, position)`.

## Reversal cost

High after import: token rows, offsets, and `token_order_hash` all assume this
rule. Changing it requires re-import as a new edition version.

## Consequences

- QV-010/011/012 enforce positions, round-trip, and offsets.
- `quran_token_separators` stores the exact separators (`after_position`
  `0..=k`, where `0` is leading text).
