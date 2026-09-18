# Owner Decisions — Quran Edition Architecture

- **Recorded:** 2026-09-18
- **Source:** project owner, via the architecture-continuation brief of 2026-09-18.
- **Companion docs:** `docs/02-architecture/upstream-sources.md`,
  `docs/02-architecture/decisions/ADR-0101-initial-quran-dataset.md`,
  `ADR-0203` (morphology), `ADR-0114` (comparison),
  `docs/05-followups/decisions-needed.md` (per-task OD items).

This file separates **what the owner has decided**, **what still requires
owner/source verification**, and **what is currently unknown**. It exists so
engineering can proceed on the decided items while the pending items stay
explicitly open. Nothing here overrides a verified upstream fact or an ADR.

> Rule for agents: never replace `unknown` / `pending_owner_verification` /
> `source_verification_required` with a guessed value, and never flip an ADR to
> `Accepted` merely because implementation can proceed.

---

## Decided

- **Primary/default Quran representation:** Uthmani script + Ḥafṣ ʿan ʿĀṣim.
  This is the intended *application default*, not an assertion about any
  specific upstream slug, publisher, or print edition.
- **Q-ai must support multiple Quran readings/editions.** The domain model must
  represent many Arabic editions/readings simultaneously; Hafs is not the only
  possible representation. Different editions are never merged.
- **Q-ai must support many languages and multiple translations per language.**
  A translation is never identified by language code alone, and a translation
  is never the canonical Quran text.
- **`fawazahmed0/quran-api` is the preferred upstream catalog/reference** for
  edition identifiers and translation inventory.
- **`gaitco/quran-database` may be used as a database/schema/reference source**
  (Surah/Ayah identity, page/juz/hizb/rubʿ metadata, SQLite shape, checksum
  provenance). It does not replace Q-ai's domain model.
- **`spqrxi/quranchecksum` may be used as an integrity/checksum reference** for
  *compatible* datasets only (currently the Uthmani/ḤafṣʿanʿĀṣim baseline).
- **Upstream identifiers are preserved exactly** as `upstream_edition_slug`;
  any Q-ai-internal id is an explicit mapping
  `upstream_edition_slug -> qai_edition_id`.
- **Per-source licensing is mandatory.** A repository being open source never
  implies its contained translation/data text is redistributable.
- **Script, qiraʾah, riwayah, edition, and edition-version are separate
  fields** and are not collapsed (no `Uthmani ⇒ Hafs` inference).
- **Upstream ingestion is a staged adapter pipeline**, not a direct insert of
  external JSON into canonical tables.

---

## Pending Verification (owner or source action required)

| # | Item | Blocks | Recorded in |
|---|---|---|---|
| ODV-01 | Exact **primary edition/source identity** (which upstream slug is canonical) | `ADR-0101`, `AC-P1-01`, P1-T01/T02/T56/T58 | ADR-0101 |
| ODV-02 | **Redistribution/license status per imported translation** (per-edition, not per-repository) | translation import beyond metadata; `quran.license_status` | ADR-0101, upstream-sources |
| ODV-03 | **Primary source publication/version metadata** (publisher, release, retrieval pin) | edition `version`, manifest pin | ADR-0101 |
| ODV-04 | **Named editorial reviewer + `verified_by` sign-off** | `AC-P1-01`, P1-T55 | ADR-0101, `decisions-needed.md` OD-02 |
| ODV-05 | **Independent reference corpus + procedure + signer** for QV-015 | QV-015, P1-T03/T26 | ADR-0114, OD-03 |
| ODV-06 | **Final morphology dataset** (provider unselected) | P2 sprints 2.4–2.5, `AC-P2-01` | ADR-0203 |
| ODV-07 | **Morphology dataset license/attribution** | morphology import | ADR-0203 |
| ODV-08 | **Final reference-corpus policy** (which corpora may be bundled) | comparison scope | ADR-0114 |
| ODV-09 | **Bundle-vs-user-supplied policy** for the canonical corpus (ADR-0101 A/B/C) | `AC-P1-01` | ADR-0101, OD-01 |
| ODV-10 | Whether any upstream **non-Hafs edition** has enough declared provenance to ship | multi-riwayah activation | ADR-0101 |

---

## Explicitly Unknown

Values that are **not verified** and must not be invented. Use these literals
verbatim where a field is required:

- The canonical edition's **publisher**, **publication year**, **release
  number**, and **source URL**: `source_verification_required`.
- The canonical edition's **SHA-256 manifest / root hash** in Q-ai terms:
  `pending_owner_verification` (the upstream checksum manifest exists, but Q-ai
  has not adopted it as its own).
- The **qiraʾah/riwayah** of every upstream `ara-*` edition except where the
  upstream slug/name states a transmission; others are `unknown` (see
  upstream-sources inventory).
- The **license** of every individual translation in `quran-api`: `unknown`
  unless the specific translator/publisher terms are cleared.
- The **morphology provider**, its version, and its license: `unknown`.
- The **reference corpus** for QV-015: `pending_owner_verification`.

---

## How this interacts with `decisions-needed.md`

`docs/05-followups/decisions-needed.md` remains the per-task owner-question
ledger (OD-01 … OD-14) for Phase 1. This file records the *architectural*
decisions taken on 2026-09-18 and the verification backlog they create. When an
item here is answered, record the answer in the ADR named under "Recorded in"
and update the corresponding OD row — do not delete history.
