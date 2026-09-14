# ADR-0102 — Quran Addressing Scheme & Reference Grammar

- Status: Accepted
- Phase: 1 — Canonical Quran Core
- Date: 2026-09-14
- Related decisions: ADR-0101, ADR-0103, ADR-0108, ADR-0111
- Requirements: PRD §7.4, §12.1, §13.3, §21.2

## Context

Every stored citation, API reference, tool input, and deep link is a Quran
reference. The grammar must be total, deterministic, panic-free, and frozen
before any citation is stored — changing it later breaks every stored reference.

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| Hand-written recursive descent over borrowed slices | Total control; no parser dependency; panic-free by construction; testable | More code than a parser generator |
| `nom` / `winnow` / `pest` / `chumsky` | Less hand-written parsing | New dependency; error-code mapping is indirect |

## Decision

A hand-written, allocation-light parser and serializer in `quran-core::reference`.
Grammar (frozen):

```ebnf
reference     = [ "quran" ":" ] [ edition ":" ] locator ;
edition       = slug [ "@" semver ] ;
slug          = ALPHA , { ALPHA | DIGIT | "-" } ;   (* lowercase *)
locator       = ayah_locator | range_locator | token_locator | division_locator ;
ayah_locator  = surah [ ":" ayah ] ;
range_locator = surah ":" ayah "-" ( ayah | surah ":" ayah ) ;
token_locator = surah ":" ayah ":" ( "token" ":" )? position ;
division_locator = ( "juz" | "hizb" | "rub" | "manzil" | "page" | "ruku" | "sajdah" ) ":" number ;
```

- `serialize` emits the deterministic short form; `parse(serialize(r)) == r`.
- `canonical_form` emits the pinned storage form `quran:slug@version:locator`
  (ayah-level with a pinned edition only).
- Division keywords win over edition slugs only for exactly `keyword:number`.
- Errors are `QAI-QUR-0100…0112`; every malformed input returns a coded error and
  never panics (the parser is reachable from user input via `qai quran resolve`).

## Accuracy / Religious-source / Licensing implications

The grammar carries no text; it only addresses it. A malformed reference must
never resolve to a *different* ayah, so `edition_id` is part of every resolved
location (I3) and canonical serialization is fully qualified.

## Security / Operational implications

ASCII-only input, a 256-byte cap, and a fixed segment array bound the parser.
Canonical short forms are accepted for convenience; storage and citations use
the pinned form.

## Migration strategy / Reversal cost

The grammar is frozen once citations exist. Any change requires a new ADR and a
migration of stored references; the 331-case golden set and the round-trip
property must stay green. Reversal cost after citations exist: **high**.

## Consequences

- `quran-core::reference` ships parser, serializer, and `resolve`.
- AC-P1-12/13 are automated-green (`crates/quran-core/tests/reference_grammar.rs`).
