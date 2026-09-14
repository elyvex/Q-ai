# ADR-0104 — Unicode Policy: Normalization, Blocks, Forbidden Points, Graphemes

- Status: Accepted
- Phase: 1 — Canonical Quran Core
- Date: 2026-09-14
- Related decisions: ADR-0101, ADR-0105, ADR-0108
- Requirements: PRD §35.1 (QV-007…QV-009)

## Context

A dataset that is *almost* right (NFD instead of NFC, a BOM, a bidi override,
a Latin homoglyph) is the most dangerous input: it looks correct and hashes
differently. The policy must decide the stored form, the forbidden set, and
the counting unit before any byte is imported.

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| Store NFC, reject everything else (QV-007 Fatal) | One canonical byte stream; hashes comparable | Requires the source to actually be NFC |
| Store whatever the source declares | No conversion disputes | Hashes incomparable across sources; normalization bugs hide |
| Grapheme counting vs scalar counting | User-perceived characters; correct offsets for combining marks | Needs a segmentation crate |

## Decision

- **Stored form: NFC** (declared per edition; QV-007 is Fatal on mismatch).
- **Forbidden (QV-008 Fatal):** C0/C1 controls, BOM, bidi controls
  (U+200E/200F, U+202A–U+202E, U+2066–U+2069), ZWJ/ZWNJ, private-use areas,
  noncharacters. Unassigned points cannot be detected without tables and are
  documented as not-checked.
- **Expected blocks (QV-009 Error):** Arabic 0600–06FF, Supplement 0750–077F,
  Extended-A/B 0870–089F/08A0–08FF, Extended-C 10EFD–10EFF, plus plain space.
  Presentation forms are excluded (compatibility decompositions).
- **Counting unit: grapheme clusters** (`char_count`, token `char_start/end`);
  byte offsets are carried alongside for slicing.

## Accuracy / Religious-source implications

Combining marks (fatha, superscript alef) join their base letter's cluster; a
scalar-counting scheme would split user-perceived characters and corrupt
offsets. The forbidden set blocks the classic homoglyph and bidi attacks that
make two different texts render identically.

## Licensing implications

None.

## Security implications

Bidi overrides and homoglyphs are a spoofing vector; QV-008/009 close it at
import. Input bytes are untrusted until the validator passes.

## Operational / Migration implications

Implemented in `quran_corpus::unicode` (`normalization_form`,
`find_forbidden`, `is_expected_code_point`). Changing the stored form later
re-hashes the corpus (ADR-0108) — effectively a new edition.

## Reversal cost

Medium: the recipe is code, but every stored hash assumes NFC.

## Consequences

- QV-007 (Fatal), QV-008 (Fatal), QV-009 (Error) enforce this policy.
- `unicode-segmentation` + `unicode-normalization` are pinned workspace deps.
