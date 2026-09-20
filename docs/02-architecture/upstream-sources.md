# Upstream Source Reference — Quran Data & Integrity

- **Recorded:** 2026-09-18
- **Status:** Reference facts verified by direct inspection of locally cloned
  repositories on 2026-09-18. All revisions are exact `HEAD` commits at that
  time; re-verify before relying on them (upstreams move).
- **Companion:** `ADR-0101` (initial dataset), `ADR-0114` (comparison),
  `ADR-0203` (morphology), `owner-decisions.md`.

This document records **verified upstream facts only**. Where a fact is not
verifiable from the source, it is written `unknown` / `source_verification_required`.
It is the reference for the ingestion/adapter layer; it does not authorize
redistribution of any individual dataset.

> Licence rule (non-negotiable): *repository open source ≠ contained data
> redistributable.* Each data family carries its own licence record; if
> redistribution is not verified, import as `metadata_only` or
> `pending_license_review`, never as vendored text.

---

## 1. `fawazahmed0/quran-api` — edition & translation catalog

| Field | Value |
|---|---|
| Repository | `fawazahmed0/quran-api` |
| Reference URL | https://github.com/fawazahmed0/quran-api |
| Verified revision | `47ca096b0976443ba2eab2e45cdf0fb4096a2610` (branch `1`, 2026-09-12) |
| Role in Q-ai | Preferred upstream catalog/reference for edition identifiers and translation inventory |
| Repository licence | Unlicense (public domain), `LICENSE` — applies to the **repository software/compilation** |
| Data licence | **Per-edition `unknown`** — upstream does not declare a per-translation licence; README redirects donations to translators/publishers |
| Redistribution | `redistribution_allowed: unknown` for every imported translation until cleared individually |
| Modification | `unknown` |

### 1.1 Catalog file shape (verified)

`editions.json` is a JSON **object** keyed by underscore ids (e.g.
`ara_quranuthmanihaf`). Each value has exactly these keys:

```json
{
  "name": "ara-quranuthmanihaf",
  "author": "Quran Uthmani Hafs",
  "language": "Arabic",
  "direction": "rtl",
  "source": "https://qurancomplex.gov.sa/",
  "comments": "Version 13, ...",
  "link": "https://cdn.jsdelivr.net/gh/fawazahmed0/quran-api@1/editions/ara-quranuthmanihaf.json",
  "linkmin": ".../ara-quranuthmanihaf.min.json"
}
```

Key facts:

- There is **no explicit `qiraah`, `riwayah`, `script`, or `recitation`
  field**. Any qiraʾah/riwayah assignment derived from an edition must be
  recorded as *inferred from the upstream slug/name*, not as upstream-declared.
- Two identifiers exist per edition: the **object key** (`ara_quranuthmanihaf`,
  underscore) and the **`name`** (`ara-quranuthmanihaf`, hyphen). Both are
  upstream identifiers. Q-ai preserves the hyphenated `name` as
  `upstream_edition_slug` and records the object key as
  `upstream_catalog_key`.
- `-la` / `-lad` suffixes denote Latin (roman) transliterations and Latin with
  diacritics respectively — these are **not** Arabic-script editions.
- `comments` carries provenance/quality notes, including OCR provenance, a
  KFGQPC "Version N" note, and explicit character substitutions.

### 1.2 Measured inventory at the verified revision

| Measure | Value |
|---|---|
| Total catalog entries | **492** |
| Distinct `language` values | **98** |
| Arabic (`ara-*`) entries | **33** |
| `direction: rtl` entries | **62** |

The README's "90+ languages & 440+ translations" is the upstream's own
headline; the measured catalog at the pinned revision is above. Neither figure
should be restated as "all qiraʾat".

### 1.3 Arabic (`ara-*`) editions — observed, with declared transmission

Grouped by what the upstream slug/name **states**. `inferred` means the
transmission is read from the upstream name, not from an explicit field.

**Named transmissions (riwayah stated by upstream slug):**

| upstream slug | upstream author | upstream `source` | quality / comments |
|---|---|---|---|
| `ara-quranuthmanihaf` | Quran Uthmani Hafs | qurancomplex.gov.sa | "Version 13"; KFGQPC Uthmani fonts; character substitutions for Unicode |
| `ara-quranuthmanihaf1` | Quran Uthmani Hafs No Diacritics | qurancomplex.gov.sa | no diacritics |
| `ara-quranwarsh` | Quran Warsh | qurancomplex.gov.sa | "Version 8"; verse numbering changed to Uthmani |
| `ara-quranqaloon` | Quran Qaloon | qurancomplex.gov.sa | "Version 8"; Uthmani numbering |
| `ara-quransoosi` | Quran Soosi | qurancomplex.gov.sa | "Version 8"; Uthmani numbering |
| `ara-quransoosinonun` | Quran Soosi Non Unicode | qurancomplex.gov.sa | non-Unicode; needs KFGQPC fonts |
| `ara-qurandoori` | Quran Doori | qurancomplex.gov.sa | "Version 8"; Uthmani numbering |
| `ara-qurandoorinonun` | Quran Doori Non Unicode | qurancomplex.gov.sa | non-Unicode |
| `ara-quranbazzi` | Quran Bazzi | qurancomplex.gov.sa | "Version 7"; Uthmani numbering |
| `ara-quranqumbul` | Quran Qumbul | qurancomplex.gov.sa | "Version 7"; Uthmani numbering |
| `ara-quranshouba` | Quran Shouba | qurancomplex.gov.sa | "Version 7"; Uthmani numbering |

**Other `ara-*` entries (transmission not stated by upstream):**

| upstream slug | upstream author | upstream `source` | notes |
|---|---|---|---|
| `ara-kingfahadquranc` | King Fahad Quran Complex | tanzil.net | Arabic; transmission not declared |
| `ara-quranacademy` | Quran Academy | github.com/quranacademy/quran-text | |
| `ara-quranuthmanienc` | Quran Uthmani Enc | — | comments: "copy of ara-quranacademy"; transmission not declared |
| `ara-qurankhaledhosn` | Quran Khaled Hosney | github.com/khaledhosny/quran-data | |
| `ara-quransimple` | Quran Simple | api.alquran.cloud `quran-simple` | simplified orthography; transmission not declared |
| `ara-quranspelled` | Quran Spelled | ar.wikisource (Imlāʾī rasm) | |
| `ara-quranspellednod` | Quran Spelled No Diacritics | ar.wikisource (Imlāʾī rasm) | |
| `ara-quranindopak` | Quran Indopak | fonts.qurancomplex.gov.sa | IndoPak/Nastaleeq; converted to Unicode |
| `ara-qurannastaleeqn` | Quran Nastaleeq Non Unicode | fonts.qurancomplex.gov.sa | "Version 10"; non-Unicode; also known as IndoPak |
| `ara-jalaladdinalmah` | Jalal Ad Din Al Mahalli & As Suyuti | tanzil.net | author names are those of Tafsīr al-Jalālayn; upstream has **no `type` field**, so classification is inferred and needs verification |
| `ara-sirajtafseer` | Siraj Tafseer | quranenc.com | slug declares tafseer |
| `ara-sirajtafseernod` | Siraj Tafseer No Diacritics | quranenc.com | slug declares tafseer; diacritics removed for search |

**Transliterations / phonetic (not Arabic script):**

`ara-quran-la`, `ara-quran-la1..la5`, `ara-quran-lad4`,
`ara-quranphoneticst`, `ara-quranphoneticst-la`,
`ara-kingfahadquranc-la`.

**Coverage/gap conclusions.**

- Upstream exposes **named riwayat** for Hafs, Warsh, Qalun, Soosi, Doori,
  Bazzi, Qumbul, Shouba — but **only as slug names**, with no explicit
  `riwayah` field and no independent provenance statement for the underlying
  reading. This is not "all qiraʾat"; it is the set this catalog happens to
  carry.
- **Nothing in the catalog establishes that `Uthmani ⇒ Hafs`.** Uthmani is an
  orthography; Hafs is a transmission. They must stay separate fields.
- Non-Unicode editions (`*nonun`, `ara-qurannastaleeqn`) are **not directly
  importable as canonical text** without a declared conversion; they are
  reference-only.
- Commentary entries (`ara-jalaladdinalmah`, `ara-sirajtafseer*`) are **not bare
  Quran text**; `sirajtafseer` declares this in its slug, while for
  `jalaladdinalmah` it is inferred from the author names and must be confirmed
  before classification. None may enter the canonical corpus.

### 1.4 Adapter requirements (implemented as `quran_corpus::upstream_catalog`)

Implemented and unit-tested (`quran-corpus/src/upstream_catalog.rs`, 11 tests;
verified read-only against the pinned file: 492 entries → 20 QuranText /
3 Tafsir / 285 Translation / 184 Transliteration, 11 named transmissions).
Surfaced in the CLI as `qai quran catalog <editions.json> [--revision R] [--report P]`
(`application::quran_cli::cmd_catalog`; metadata-only, no database, licences stay
unknown; malformed catalog → exit 3, unreadable path → exit 2).
The adapter MUST (and does):

1. Read `editions.json` and preserve the exact `name` as `upstream_edition_slug`
   and the object key as `upstream_catalog_key`.
2. Record `author`, `language`, `direction`, `source`, `comments`, `link`,
   `linkmin` verbatim.
3. Classify `type` as `quran_text` vs `translation` vs `tafsir` — **not**
   purely from the `ara-` prefix (tafsir entries are `ara-*`).
4. Attach quality flags (`ocr`, `non_unicode`, `transliteration`,
   `no_diacritics`, `machine_transcription`) parsed from `comments`; do not
   discard `comments`.
5. Record repository, revision, file path, retrieval date, and a source hash.
6. Default `license.status = unknown` unless per-edition terms are supplied;
   never inherit the repository Unlicense onto the data text.
7. Run verse-count/structure validation before any promotion past staging.

---

## 2. `gaitco/quran-database` — relational/schema reference

| Field | Value |
|---|---|
| Repository | `gaitco/quran-database` |
| Reference URL | https://github.com/gaitco/quran-database |
| Verified revision | `4e0cb3414fa3993666a1bf910de45a585ab314d2` (branch `main`, 2026-08-22) |
| Role in Q-ai | Reference for relational modeling, divisions metadata, SQLite shape, and verse-manifest provenance — **not** a schema to copy |
| Repository licence | MIT (code/packaging) |
| Data licence | Arabic text is Tanzil Uthmani via alquran.cloud; the 134 translation/tafsir editions carry their own provenance — MIT does **not** cover them |
| Redistribution | `unknown` for the translation/tafsir rows |

### 2.1 Verified facts

- Three exported schemas: MySQL (source dump), SQLite, PostgreSQL (converter-
  enriched). Core tables: `surahs`, `ayahs`, `editions`, `ayah_edition`, plus
  converter-added `juzs`, `hizbs`, `pages`; views `surah_stats`,
  `ayah_with_translation`.
- `ayahs` holds **6,236** rows. `editions` holds **134** entries.
  `ayah_edition` holds 6,236 × 134 = **835,624** rows.
- `editions.identifier` is an **alquran.cloud edition id** (e.g.
  `quran-uthmani`, `en.sahih`), not a Q-ai id.
- `ayah_edition` has no per-row licence or attribution; the repository MIT
  licence does not extend to that text.
- Divisions: `juzs` (30), `hizbs` (60), `pages` (604). `ayahs.rub_id` is the
  **rubʿ al-hizb quarter, 1–240**, and does **not** join to `hizbs`; a
  self-documented breaking-rename from `hizb_id` (MySQL still uses `hizb_id`).
- Page mapping is derived from the dump's `ayahs.page` field, not independently
  verified against print.
- Ships `manifest/quran-arabic.manifest.json`: verse-level SHA-256, NFC,
  per-surah and whole-mushaf rollups. Its hashing is deliberately identical to
  `spqrxi/quranchecksum`, so the two manifests are directly comparable.
- Provenance chain (upstream's own statement):
  `KFGQPC Madinah Mushaf → Tanzil → alquran.cloud (edition quran-uthmani) → this repository (dump 2018-06-07)`.
  Against a fresh Tanzil download (Aug 2026), 5,927/6,236 verses are
  byte-identical under NFC; the 309 differences are character carriers and word
  spacing only — no letter or diacritic differs.
- Supplemental `data/rukus.json`: 558 Ruku boundaries from the **Quran
  Foundation** convention. The repository itself states the MIT licence does
  **not** override Quran Foundation terms and that redistribution permission
  must be confirmed. Treat as `metadata_only` / `pending_license_review`.

### 2.2 Use in Q-ai (concepts to reuse, not schema)

- Surah/Ayah identity and the `surah:ayah` key convention (already Q-ai's).
- Division metadata shapes: page/juz/hizb/rubʿ/ruku, with the `rubʿ` vs `hizb`
  naming trap called out.
- Verse-manifest format and rollup design (cross-check against `quranchecksum`).
- Provenance/verification framing (`docs/provenance.md`).

Q-ai maps upstream schema → its own normalized schema in a documented importer,
and treats upstream ids as **external ids**, never as primary keys.

---

## 3. `spqrxi/quranchecksum` — integrity / checksum reference

| Field | Value |
|---|---|
| Repository | `spqrxi/quranchecksum` |
| Reference URL | https://github.com/spqrxi/quranchecksum |
| Verified revision | `954244e634e7b9d0bfd9f501a7eb859ffff68db1` (branch `main`, 2026-07-04) |
| Role in Q-ai | Integrity-reference implementation for the **Uthmani/Ḥafṣ ʿan ʿĀṣim** baseline only |
| Repository licence | MIT |
| Data licence | Manifest is hash-only over Tanzil Uthmani (no Quran text stored); translation manifests are hash-only, never text |

### 3.1 Verified facts

- Published manifest `manifest/quran-uthmani.manifest.json`:
  - algorithm **SHA-256**, normalization **NFC**, strip leading/trailing
    whitespace, no other transformation;
  - granularity **verse**, **6,236** verses, **114** surahs;
  - rollups: surah = `sha256(concat(verse hashes in ayah order))`,
    whole-Quran = `sha256(concat(surah hashes in surah order))`;
  - `meta.source = tanzil-uthmani`;
  - `meta.orthographic_standard = Madinah Mushaf (KFGQPC, Medina)`;
  - whole-Quran root at the pinned revision:
    `5b8bb60d84ad9fbbc3abbca112234abf77d9e14ce52076ff397656f7f84ed73c`.
- Provenance chain (upstream's own statement, `docs/PROVENANCE.md`):
  KFGQPC Madinah Mushaf → Tanzil Uthmani transcription (manually verified) →
  this manifest. It explicitly states it is **not** a direct KFGQPC text
  export and does not re-verify Tanzil's manual check.
- The SPEC explicitly warns: a manifest match is **not** cross-edition
  equivalence, **not** tajwīd/qirāʾah validation, and a difference between two
  legitimate editions is not evidence either is wrong.
- Translation manifests (`manifest/translations/`) are **hash-only**, with
  `meta.kind = "translation"` and a distinct root key `translation_root` so a
  translation root can never be read as a Quran root. At the verified revision
  `manifest/translations/` contains only `.gitkeep` — no translation manifests
  are committed yet.
- Translation manifests carry `translation_id`, `translator`, `language`,
  `distributor`, `distributor_edition_id`, `edition_version` (or
  `"unversioned"`), `retrieved_at`, `based_on_arabic`, `verse_scheme`,
  `canonicity: none`, and versioned history via `supersedes`.

### 3.2 Use in Q-ai

- Adopt the **design** (verse hash + surah root + Quran root; explicit
  normalization; algorithm tag), not the manifest as Q-ai's own.
- **One manifest per edition.** The Hafs/Uthmani checksum is never reused for
  Warsh, Qalun, or any other reading. Non-Hafs editions get independent
  manifests generated from their own source.
- A hash mismatch between different editions/riwayat is classified (see
  `ADR-0114`), never reported as corruption.
- Translation integrity uses hash-only manifests with a distinct root key;
  translation text is never stored in an integrity manifest.

---

## 4. Cross-source relationship

```text
KFGQPC Madinah Mushaf (Uthmani, Ḥafṣ ʿan ʿĀṣim orthographic standard)
        |                                   (print/visual reference)
        +--> Tanzil Uthmani transcription
        |         |
        |         +--> quranchecksum  : hash-only integrity manifest
        |         +--> quran-database : text + manifest (via alquran.cloud)
        |
        +--> quran-api : edition/translation catalog (many scripts/readings)

Q-ai canonical corpus (owner-selected edition)  <-- upstream_edition_slug mapping
Q-ai alternate editions / riwayat               <-- their own manifests
Q-ai translations                               <-- per-edition licence + attribution
Q-ai reference corpora                          <-- policy pending (ODV-08)
Q-ai checksum corpus                            <-- per-edition manifests
```

These layers stay separate. A checksum corpus never substitutes for the text
source; a translation never substitutes for canonical text; one riwayah's
checksum never certifies another.
