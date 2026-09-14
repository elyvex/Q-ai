# ADR-0108 — Corpus Hashing Scheme

- Status: Accepted
- Phase: 1 — Canonical Quran Core
- Date: 2026-09-14
- Related decisions: ADR-0104, ADR-0105, ADR-0106, PRD §12.1
- Requirements: PRD §7.3, §12.1, §35.1 (QV-013/014/024)

## Context

Every quotation, citation, reproducibility checksum, and doctor check depends
on hashes that must reproduce forever from stored rows. The recipe must be
frozen before the first import: changing it later invalidates every stored
hash, citation, and report.

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| SHA-256 over length-prefixed streams | Unambiguous (no boundary-shift collisions); matches Phase-0 `sha2` | Slower than BLAKE3 (irrelevant at corpus scale) |
| BLAKE3 | Faster | Second hash primitive in the workspace; no Phase-0 precedent |
| Plain concatenation without lengths | Simplest | `["ab","c"]` collides with `["a","bc"]` |

## Decision

SHA-256, length-prefixed (`len u64 LE || bytes`), domain-separated tags:

- `text_hash`: `feed("qai-text-hash-v1")`, `feed(slug)`, `feed(version)`, then
  `feed(ayah_text)` per ayah in `(surah, ayah)` order.
- `structure_hash`: SHA-256 over one canonical string
  `v1|{slug}|{version}|S{s}:{count},…|A{s}:{a}:{divisions};…`
  (`-` for absent divisions; sajdah `0|1|2`).
- `token_order_hash`: tag + slug + version, then per token in global order
  `surah u16 LE || ayah u32 LE || position u32 LE` plus `feed(surface)`.
- Per-ayah `text_hash` and per-token `surface_hash` are plain SHA-256 of the bytes.
- Storage form is `sha256:<hex>` (D0.6); `ContentHash` carries the algorithm tag
  so a future algorithm never reinterprets old digests.

Recomputation from stored rows (QV-014 half, QV-024) is part of the import
(round-trip checkpoint) and of `doctor --quran --deep`.

## Accuracy implications

The recipe is the reproducibility contract (§12.1): two systems with the same
bytes must produce the same hashes. Length-prefixing removes an entire class
of collision; domain tags remove cross-hash confusion.

## Religious-source implications

The hash binds a quotation to exact bytes; `verify_quotation` compares against
it. A changed byte is a `Mismatch`, never a silent edit.

## Licensing / Security implications

Hashes are content-derived, license-neutral. Manifest hashing (QV-013) detects
tampering between declaration and import.

## Operational / Migration implications

Implemented once in `quran_corpus::hashing` and reused by importer, doctor,
and tools. Changing the recipe = new ADR + re-import + re-verification of all
stored hashes; treated as breaking.

## Reversal cost

Effectively irreversible after the first activation without invalidating the
corpus. Frozen.

## Consequences

- AC-P1-07 (`doctor --deep` recomputation) rests on this recipe.
- Golden `edition_hashes.json` values (M4/M10) are computed with it.
