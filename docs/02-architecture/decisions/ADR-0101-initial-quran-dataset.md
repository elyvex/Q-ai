# ADR-0101 — Initial Quran Dataset, Script, Riwayah, and License

- Status: **Draft — pending human sign-off (do not mark Accepted)**
- Phase: 1 — Canonical Quran Core
- Date: 2026-09-14
- Related decisions: ADR-0104 (Unicode), ADR-0105 (tokenization), ADR-0108 (hashing), ADR-0110 (basmala), ADR-0114 (reference corpus)
- Requirements: PRD §7.2, §22.4, §34, §35.1, §38
- Owner: _unassigned_ (swimlane P1-X01/X02)

## Context

Phase 1 cannot ship a *canonical* corpus without an approved edition dataset whose
redistribution is legally cleared and whose text a named qualified reviewer has
compared against a recognized printed muṣḥaf (PRD §38, §35.1). This is a
legal + editorial decision, not an engineering one. Until it is made, the
pipeline is built and verified end to end against a **synthetic, clearly
non-canonical** fixture (`fixtures/quran/test-edition-min/`), per the documented
fallback in `README.md` §8.

This ADR records the decision while it is open. It is **Draft**: it must not be
marked `Accepted` until a human supplies (a) a licensed dataset, (b) the
bundle-vs-user-supplied policy, and (c) the named reviewer.

## Candidate datasets (surveyed; NOT selected)

Candidate sources available to the project. All are **unverified** for license,
script, riwayah, numbering, and normalization; each must be cleared by the
owner before use. None is bundled or imported by this phase.

### Local candidates (`/Users/ali/dev/python/misc/q/quran/`)

| Candidate | Source | Notes / open questions |
|---|---|---|
| `quran-json` (semitica / risan lineage) | local `quran/quran-json-main/` | Nested JSON per surah/ayah; translations included. **License unverified.** Script/riwayah/normalization undeclared. |
| `quran-text` | local `quran/quran-text-main/` | Plain-text Quran. **License unverified.** Silent whitespace and mark handling must be checked against ADR-0105. |
| Quran-Truth-Edition | local `quran/Quran-Truth-Edition-master/` | English plain text primarily. Not an ArabicUthmani canonical source. |
| NoorUlHuda | local `quran/NoorUlHuda-master/` | Android app source; assets may embed an edition. **License and provenance unverified.** |
| `alvahy.com` scrape | local `quran/alvahy.com/` | HTML per-verse + hadith; Persian translations. Not a clean Arabic-only edition. |
| `quran-with-hadiths.json` | local `quran/quran-with-hadiths.json` | Combined corpus; not a canonical Arabic edition. |

Public reference projects (user-supplied 2026-09-14; unverified from here).
The owner supplied the following public projects as further candidates (see
`/Users/ali/Documents/Chats/quran_github_projects.md`). Claims below are the
supplier's, **not independently verified** — this environment has no network
access to confirm licenses, contents, or activity.

| # | Repository | Claimed focus | Offline shape (claimed) |
|---|---|---|---|
| 1 | `quran/quran-android` | Official Android app, offline packs | SQLite DB + downloadable packs |
| 2 | `quran/quran-ios` | Official iOS app | Offline DB + audio |
| 3 | `GlobalQuran/Quran-Data` | Quran dataset (JSON, XML, CSV, SQL) | Multiple formats |
| 4 | `fawazahmed0/quran-api` | Static-JSON "API", no key | Cloneable JSON files |
| 5 | `semitica/quran-json` | Complete Quran, clean JSON | Standalone JSON |
| 6 | `risan/quran-json` | Simple JSON + translations | Lightweight JSON |
| 7 | `quran/quran.com-api` | quran.com backend | Core text metadata |
| 8 | `sharfuddin/Al-Quran-Database` | Full Quran SQLite + translations | Ready `.sqlite` file |
| 9 | `alquran/alquran.github.io` | Offline-first PWA | Cached web assets |
| 10 | `quran/quran-php` | PHP library + SQLite | Bundled SQLite |
| 11 | `quran/quran-api` | Python package | Verified text (claimed) |
| 12 | `tushar177/Quran-Offline` | Flutter offline app | Built-in data |
| 13 | `sajjadjawadi/quran-json` | Urdu + Arabic JSON | Offline-ready JSON |
| 14 | `qurancode/quran-code` | Offline-capable web app | PWA text + audio |
| 15 | `mohammad-hassan/quran-data` | Minimal text + transliteration | Small JSON |

**Engineering does not choose among these.** Selection requires the owner to
confirm at minimum: (1) redistribution rights, (2) Uthmani script,
(3) riwayah (expected Hafs ʿan ʿĀṣim), (4) verse-numbering scheme,
(5) Unicode normalization form, (6) an independent reference corpus for QV-015,
and (7) editorial sign-off with a named reviewer.

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| **A. Bundle a licensed edition** | Zero-config canonical corpus for users | Requires a dataset with explicit redistribution rights; largest legal exposure |
| **B. Ship schema + synthetic fixture; user supplies the real edition via `qai quran import`** | No licensing exposure in the shipped artifact; pipeline fully testable offline | Users must obtain and import their own verified edition |
| **C. Defer Phase 1 entirely until a dataset is licensed** | Avoids any interim corpus | Stalls all downstream phases on a non-engineering dependency |

## Decision

**Pending.** The engineering deliverable is identical under A and B: the
importer/validator/reader operate on the `qai.quran.edition` intermediate
format, and the shipped data differs only in the bundled fixture. The documented
fallback is **Option B** (ADR-0101 fallback, `README.md` §8), which is what the
current build verifies against. This ADR remains **Draft** until a human records
the chosen option and the dataset details.

## Accuracy implications

A wrong dataset is a *silent* correctness bug downstream: every citation, graph
edge, and tafsir link inherits it. Selection must include the QV-001…QV-028
validation run and a reference-corpus comparison (QV-015, ADR-0114) before any
activation. The hash recipe (ADR-0108) is frozen so the approved bytes are the
bytes every later phase cites.

## Religious-source implications

The dataset determines the canonical Arabic text, numbering, and basmala
handling. No model may produce or alter it (invariant I2). Editorial sign-off by
a named qualified reviewer is required and recorded in the edition's
`verified_by` field; it is a religious-source judgment, not an engineering one.

## Licensing implications

Redistribution rights are decisive. If rights cannot be confirmed, the fallback
is B (ship no text; user imports). License status flows into the edition's
license record and the `quran.license_status` doctor check.

## Security implications

Imported bytes are untrusted until validated; adapters run under the Phase-0
deny-by-default posture, and the synthetic fixture contains no real text.

## Operational implications

`qai quran import <manifest>` is the single entry point either way; a bundled
edition merely adds a first-run import step.

## Migration strategy

If the dataset changes after activation, a new edition *version* is imported and
activated; the prior version is deprecated, never deleted (I7, §76). Stored
hashes make the change auditable via `qai quran diff`.

## Reversal cost

Low while Draft (no corpus shipped). Once the first real edition is activated,
changing the dataset requires a new version + difference report + human
approval; changing the *hashing recipe* would invalidate every stored hash and
is treated as a breaking change (ADR-0108).

## Consequences

- Engineering proceeds unblocked on the synthetic fixture.
- `qai quran import` and the whole pipeline are exercised against
  `test-edition-min` and the 16 adversarial corpora.
- The phase cannot be declared complete (AC-P1-01) until this ADR is Accepted.

## Follow-ups

- P1-X01: open vendor/license correspondence; assign an owner and decision-open date.
- P1-X02: engage and name the editorial reviewer.
- P1-T55: record the reviewer and sample in `verified_by`.
