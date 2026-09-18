# ADR-0101 — Quran Edition Model: Primary Default, Multiple Readings, and Licensing

- Status: **Draft — pending human sign-off (do not mark Accepted)**
- Phase: 1 — Canonical Quran Core
- Date: 2026-09-14 (created); 2026-09-18 (expanded for multi-edition architecture)
- Related decisions: ADR-0102 (addressing), ADR-0103 (numbering), ADR-0104 (Unicode),
  ADR-0105 (tokenization), ADR-0108 (hashing), ADR-0110 (basmala),
  ADR-0112 (translations), ADR-0114 (comparison), ADR-0203 (morphology)
- Requirements: PRD §7.2, §22.4, §34, §35.1, §38
- Owner: _unassigned_ (swimlane P1-X01/X02)
- Companion: `docs/02-architecture/upstream-sources.md`,
  `docs/05-followups/owner-decisions.md`

---

## Status note

This ADR is **Draft** and must not be marked `Accepted` merely because
implementation can proceed. Two distinct gates remain open and are tracked
separately below:

- **Owner Decision** — architecture direction the owner has now stated
  (recorded in this revision).
- **Source Verification Pending** — dataset identity, per-item licensing, and
  editorial sign-off that no agent may invent.

Where a field is not verified, this ADR writes `unknown`,
`pending_owner_verification`, or `source_verification_required`. Those literals
are intentional and must not be replaced with guesses.

---

## Context

Phase 1 cannot ship a *canonical* corpus without an approved edition dataset
whose redistribution is legally cleared and whose text a named qualified
reviewer has compared against a recognized printed muṣḥaf (PRD §38, §35.1).

Earlier revisions of this ADR treated the problem as "pick one Arabic edition".
The owner has clarified the architectural intent (2026-09-18):

> Q-ai's primary/default Quran representation is Uthmani script + Ḥafṣ ʿan
> ʿĀṣim, but the application must ultimately support multiple recognized Quran
> reading traditions (qiraʾāt/riwāyāt), multiple Arabic editions/scripts where
> available, and translations in many languages. The primary/default edition is
> an architectural intent; it does not authorize inventing any edition slug,
> publisher, release number, copyright status, or source identity.

This has two consequences:

1. The **domain model must be multi-edition from the start**. Hafs is the
   default selector, not the only representable edition.
2. The **selection of a concrete canonical dataset is still open** — the intent
   names *script + riwayah*, not a slug, file, publisher, or licence.

Until the concrete dataset is supplied, the pipeline runs end to end against a
**synthetic, clearly non-canonical** fixture (`fixtures/quran/test-edition-min/`),
per the documented fallback in `README.md` §8.

---

## Owner Decisions (recorded 2026-09-18)

### Decided

- Primary/default representation: **Uthmani script + Ḥafṣ ʿan ʿĀṣim**.
- Q-ai must support **multiple Quran readings/editions** concurrently; they are
  never merged (invariant I3).
- Q-ai must support **many languages and multiple translations per language**.
- `fawazahmed0/quran-api` is the preferred upstream **catalog/reference** for
  edition and translation inventory.
- `gaitco/quran-database` may be used as a **database/schema/reference** source.
- `spqrxi/quranchecksum` may be used as an **integrity/checksum reference** for
  compatible datasets only.
- Upstream identifiers are preserved exactly as `upstream_edition_slug`; any
  internal id is an explicit mapping `upstream_edition_slug -> qai_edition_id`.
- Per-source licensing is mandatory; repository licence ≠ data licence.
- Script, qiraʾah, riwayah, edition, and edition-version are **separate fields**.

### Pending Owner / Source Verification

- Exact primary edition/source identity and `upstream_edition_slug`.
- Publisher, publication/release metadata, and retrieval pin.
- Redistribution/licence status **per imported translation**.
- Named editorial reviewer and `verified_by` sign-off.
- Bundle-vs-user-supplied policy (Options A/B/C below).
- Whether any non-Hafs upstream edition has declared enough provenance to ship.

### Explicitly Unknown

- Canonical publisher, year, release number, source URL → `source_verification_required`.
- Q-ai's own SHA-256 manifest/root for the canonical edition →
  `pending_owner_verification`.
- Qiraʾah/riwayah of most upstream `ara-*` editions → `unknown` unless the
  upstream slug/name itself states the transmission (see
  `docs/02-architecture/upstream-sources.md` §1.3).

---

## Architecture: one logical corpus, many identified editions

Q-ai models **one logical Quran corpus** whose canonical representation is
composed from **explicitly identified editions/readings**. It does **not** model
"one Quran text".

```text
Quran (logical corpus)
 ├── Edition A  (e.g. primary default)
 │    ├── script          uthmani
 │    ├── qiraah          [verified | unknown]
 │    ├── riwayah         [verified | unknown]
 │    ├── edition_version SemVer
 │    ├── source          repo + revision + path + retrieved_at
 │    ├── provenance      chain + quality flags
 │    ├── license         per-source record
 │    └── integrity       own manifest (verse/surah/quran scope)
 ├── Edition B  (alternate reading, e.g. Warsh ʿan Nāfiʿ)
 │    └── ...
 └── ...
```

### Separation of concerns (do not collapse these fields)

| Field | Meaning | Example / allowed value |
|---|---|---|
| `script` | orthography | `uthmani`, `imlaei_simple`, `indopak`, `other` |
| `qiraah` | the reading (imam-level) | `Asim`, `Nafi`, `AbuAmr`, … / `unknown` |
| `riwayah` | the transmission | `Hafs`, `Warsh`, `Qalun`, `AlSusi`, … / `unknown` |
| `edition` | identified dataset | `upstream_edition_slug` + `qai_edition_id` |
| `edition_version` | pinned revision | SemVer for Q-ai; upstream version string where declared |
| `source` | where the bytes came from | repo + revision + path + retrieved_at |
| `publisher` | publishing body | verified value or `source_verification_required` |
| `language` | content language (BCP-47) | `ar`, `en`, … |
| `translation` | non-canonical interpretation | separate first-class record |
| `license` | per-family rights | status + expression + source_url + flags |
| `attribution` | required credit | translator/publisher string |
| `provenance` | chain of custody | ordered hops + quality flags |
| `checksum` | integrity | own manifest per edition, per scope |

**Explicitly forbidden inferences:** `Uthmani ⇒ Hafs`, `Uthman Taha ⇒ Hafs`,
"contains several Arabic variants ⇒ all qiraʾat". The upstream catalog carries
no `qiraah`/`riwayah` field (`upstream-sources.md` §1.1); any assignment is
recorded as inferred-from-name with its evidence, or `unknown`.

### Identifier mapping

```yaml
edition:
  upstream_edition_slug: "<exact upstream name, never renamed>"
  upstream_catalog_key: "<exact upstream object key>"
  qai_edition_id: "<internal stable id>"
  script: "uthmani"
  qiraah: "unknown"          # verified value or unknown
  riwayah: "unknown"         # verified value or unknown
  edition_version: "<pinned>"
  language: "ar"
  type: "quran_text"          # quran_text | translation | tafsir | transliteration
```

Upstream ids are **external** ids. Q-ai primary keys are its own; the mapping is
explicit and auditable.

---

## Candidate datasets and upstream sources

All concrete candidates remain **unselected**; see
`docs/02-architecture/upstream-sources.md` for verified facts and revisions.
Summary:

| Upstream | Role | Repository licence | Data-text licence | Q-ai use |
|---|---|---|---|---|
| `fawazahmed0/quran-api` `@47ca096b` | Edition/translation catalog (492 entries, 98 languages, 33 `ara-*`) | Unlicense | per-edition `unknown` | catalog + adapter target |
| `gaitco/quran-database` `@4e0cb341` | Relational/schema + divisions + verse manifest | MIT | translations own terms; rukus source-terms restricted | reference/importer |
| `spqrxi/quranchecksum` `@954244e6` | Integrity manifest for Uthmani/Hafs | MIT | hash-only, no text | integrity-reference design |
| Local candidates (`/Users/ali/dev/python/misc/q/quran/`) | survey only | unverified | unverified | none until cleared |
| `test-edition-min` fixture | pipeline exercise | synthetic | n/a | current fallback |

Engineering does not choose among these. Selection requires the owner to
confirm at minimum: (1) redistribution rights, (2) script, (3) qiraʾah and
riwayah as separate facts, (4) verse-numbering scheme, (5) Unicode normalization
form, (6) an independent reference corpus for QV-015, and (7) editorial sign-off
with a named reviewer.

---

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| **A. Bundle a licensed edition** | Zero-config canonical corpus for users | Requires a dataset with explicit redistribution rights; largest legal exposure |
| **B. Ship schema + synthetic fixture; user supplies the real edition via `qai quran import`** | No licensing exposure in the shipped artifact; pipeline fully testable offline | Users must obtain and import their own verified edition |
| **C. Defer Phase 1 entirely until a dataset is licensed** | Avoids any interim corpus | Stalls all downstream phases on a non-engineering dependency |

## Decision

**Pending for the concrete dataset; decided for the model.**

- The engineering deliverable is identical under A and B: the
  importer/validator/reader operate on the Q-ai edition source format, and the
  shipped data differs only in the bundled fixture.
- The documented fallback remains **Option B** (ADR-0101 fallback, `README.md`
  §8), which the current build verifies against.
- The **multi-edition, multi-riwayah, multilingual model in this ADR is the
  accepted direction** and is implemented incrementally (see
  `quran-core::catalog`). Implementing the model does **not** make this ADR
  Accepted.

This ADR remains **Draft** until a human records the chosen option, the dataset
identity, its licence evidence, and the editorial reviewer.

---

## Accuracy implications

A wrong dataset is a *silent* correctness bug downstream: every citation, graph
edge, and tafsir link inherits it. Selection must include the QV-001…QV-028
validation run and a reference-corpus comparison (QV-015, ADR-0114) before any
activation. The hash recipe (ADR-0108) is frozen so the approved bytes are the
bytes every later phase cites. Because Q-ai now represents several editions, the
differ and hasher must always compare **compatible representations**
(ADR-0114): a Hafs-vs-Warsh difference is an edition/readings comparison, never
a corruption diff, and one riwayah's checksum never certifies another.

## Religious-source implications

The dataset determines the canonical Arabic text, numbering, and basmala
handling. No model may produce or alter it (invariant I2). Editorial sign-off by
a named qualified reviewer is required and recorded in the edition's
`verified_by` field; it is a religious-source judgment, not an engineering one.
Preserving `qiraah`/`riwayah` explicitly is required so that no edition is
silently presented as another's text.

## Licensing implications

Redistribution rights are decisive **per data family**, not per repository. The
repository licence (Unlicense / MIT) governs the software/compilation; each
translation and each edition text carries its own terms. Every imported family
records:

```yaml
license:
  status: verified | unknown | restricted | metadata_only
  expression: "<SPDX or source wording>"
  source_url: "<license URL>"
  redistribution_allowed: true | false | unknown
  modification_allowed: true | false | unknown
  attribution_required: true | false | unknown
```

If redistribution is not verified: do not vendor the text, do not label it open,
mark it `metadata_only` / `pending_license_review`, and continue on all tooling
that does not redistribute it. If rights cannot be confirmed at all, the
fallback is B (ship no text; user imports). License status flows into the
edition's license record and the `quran.license_status` doctor check.

## Security implications

Imported bytes are untrusted until validated. Ingestion is staged:
`source → adapter → immutable raw artifact → metadata extraction → validation →
normalized dataset → storage`, with provenance recorded at every hop (repo,
revision, path, retrieval date, upstream id, source hash, transformation,
validation result, licence state, attribution). Adapters run under the Phase-0
deny-by-default posture, and the synthetic fixture contains no real text.

## Operational implications

`qai quran import <manifest>` is the single entry point either way; a bundled
edition merely adds a first-run import step. The upstream catalog/adapter layer
(pending) adds offline metadata ingestion that never activates a corpus by
itself. Each edition's integrity manifest is generated and verified
independently.

## Migration Strategy

If the dataset changes after activation, a new edition *version* is imported and
activated; the prior version is deprecated, never deleted (I7, §76). Stored
hashes make the change auditable via `qai quran diff`. Adding a new reading or
translation is additive: a new edition/translation record, its own manifest,
and no change to existing keys beyond `edition_id` (already present in every
canonical key, I3).

## Reversal cost

Low while Draft (no corpus shipped). Once the first real edition is activated,
changing the dataset requires a new version + difference report + human
approval; changing the *hashing recipe* would invalidate every stored hash and
is treated as a breaking change (ADR-0108). Adding editions/translations is
reversible by deactivation; removing a shipped default is not.

## Consequences

- Engineering proceeds unblocked on the synthetic fixture, now against an
  explicitly multi-edition model.
- `qai quran import` and the whole pipeline are exercised against
  `test-edition-min` and the 16 adversarial corpora.
- The upstream catalog/adapter layer can be developed and tested against
  metadata without redistributing restricted text.
- The phase cannot be declared complete (AC-P1-01) until this ADR is Accepted.
- No invented slug/publisher/year/licence/hash is introduced; unknowns stay
  unknown.

## Follow-ups

- P1-X01: open vendor/license correspondence; assign an owner and decision-open date.
- P1-X02: engage and name the editorial reviewer.
- P1-T55: record the reviewer and sample in `verified_by`.
- Implement/document the `quran-api` catalog adapter and per-edition licence
  records (`upstream-sources.md` §1.4).
- Generate independent integrity manifests for any non-Hafs edition before it
  can be activated.
- Resolve QV-015 reference corpus per ADR-0114 before comparing real data.
