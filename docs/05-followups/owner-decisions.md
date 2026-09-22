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

---

## Orchestrator recommendations — 2026-09-22 (NOT human sign-off)

> Source: orchestrator-model answer series received 2026-09-22. Per
> `decisions-needed.md` header, only a human owner/editor can close an OD row.
> Everything below is a **recommendation pending human ratification** — no OD
> row flips to 🟢, no ADR flips to `Accepted` on this basis.
> Rule for agents: keep `unknown` / `pending_owner_verification` /
> `source_verification_required` literals; never invent license/reviewer facts.

### Ratified-direction recommendations (engineering may proceed on these patterns)

- **OD-01 → B now, A-track in parallel, C rejected.** Chassis stays on synthetic
  `test-edition-min` (`synthetic=true`, hidden from user surfaces). Bundling
  stays blocked until Tanzil license text + URL + capture date lands in
  `licenses/tanzil/` (ODV-02/ODV-09). Numbering intent: standard Kufan/Hafs
  counting, 6236 verses; canonical column = identity profile `verbatim-v1`.
  Publisher/release/pin/license stay `source_verification_required` /
  `pending_owner_verification`. Record in ADR-0101 (stays Draft).
- **OD-02 → method ratified, human 🔴.** L1 100% codepoint diff vs pinned
  artifact + L2 quranchecksum cross-check + L3 visual sample (al-Fatihah,
  al-Baqarah 1–5, first page of each juz, last 3 surahs, every differ-flagged
  boundary). Reviewer constraints: not a quran-core/adapter implementer,
  mushaf-literate. `verified_by` format proposed; name stays
  `pending_owner_verification`. Record in ADR-0101.
- **OD-03 → two-tier + conditional QV-015 skip authorized (recommendation).**
  Tier-1 now (pinned-artifact diff + quranchecksum digests); Tier-2 independent
  second corpus identity `pending_owner_verification` (same license gate as
  OD-01). If no licensed Tier-2 by Sprint-1.2b end, QV-015 closes as
  `passed_tier1_only` with explicit exit-gate flag — never silent pass.
  Record in ADR-0114 (stays Draft).
- **OD-05 → accept 8.7 weeks (131.0 ed), ratify 1.2a/1.2b split, no 4th
  engineer in Phase-1, no scope cut.** Sprint cap 24 ed henceforth;
  D1.1/D1.4/D1.5/D1.6/D1.13 protected. Staffing revisited at Phase-2 kickoff.
- **OD-06 → ratify axum 0.8.x + tower-http 0.6.x** (workspace-pinned, loopback
  only, per-install bearer token, tower timeouts/body-limits). Record in
  `technology-stack.md` §4 / `done.md` OWN-04.
- **OD-07 → ratify floors as proposed** (quran-core 90, validation/tokenize/
  hashing 90, adapters/differ 80, citations 85, storage-sqlite 80, surfaces
  smoke-only; per-crate cargo-llvm-cov CI gate, no lowering without new OD).
- **OD-04 → system-stack only for Phase-1 debug reader, no bundled font.**
  CSS stack with locally installed fonts only. Amiri (Quran variant, SIL OFL
  1.1) as Phase-2 candidate, `pending_owner_verification` until vendored.
  P1-T54 = "system-stack debug reader".
- **OD-08/09/10 →** presenter = project owner; every row gets verifier ≠
  evidence producer; recording local-only
  (`~/q-ai-archive/reviews/phase-1-exit-<date>.mp4` + SHA-256 + duration
  logged); all 21 rows close at Sprint-1.2b end + 2 working days (absolute date
  stamped at Sprint-1.1 kickoff); P1-X01/X02/X03 verifier = OD-02 reviewer,
  P1-X04/X05 = chief architect.
- **OD-13 → Phase-0 complete per engineering evidence (267 green tests);
  reconcile `status.md` within 1 working day** into exit-gate artifacts vs
  Phase-1 backlog; standing rule: `status.md` is single source of truth,
  updated in same PR as state change, every claim cites evidence.
- **OD-11 → QAC morphology (corpus.quran.com, Dukes et al.) v0.4 as gated
  candidate only;** license = `source_verification_required`; incompatible →
  user-import adapter only (OD-01-B pattern). Record in ADR-0203 (stays Draft).
- **OD-12 → catalog structure ratified; Phase-1 ships identity profile only**
  (`verbatim-v1`); candidate `uthmani-search-min-v1` (tatweel strip +
  diacritic-insensitive only, no hamza/rasm rewriting) waits for named
  linguist (`pending_owner_verification`). No ADR-0204 exists yet — do not
  cite it as existing; create it only via DOC task.
- **OD-14 → accept server→storage/tools direct under enforced allowlist**
  (arch-test: `server` may depend only on agreed crates; trait seams
  required); formal routing evaluation at Phase-3 kickoff.

### Conflicts / corrections (must resolve before recording as final)

1. **Custom slug `uthmani-hafs-tanzil-1.1` rejected as `upstream_edition_slug`.**
   Violates ADR-0101 §identifier-mapping + `upstream-sources.md` §1.1/§1.4:
   upstream ids preserved exactly (e.g. `ara-quranuthmanihaf`); Q-ai id is a
   separate explicit mapping. Use the proposed string only as a
   `qai_edition_id` candidate, never as the upstream slug.
2. **ADR numbers ADR-0102…0106 in orchestrator table are wrong.** Those numbers
   already exist (addressing, numbering, unicode, tokenization, storage
   layout). OD-02 method → ADR-0101; OD-03 → ADR-0114; OD-06/OD-04/OD-14 need
   new or existing correct homes (`technology-stack.md`, `acceptance.md`,
   or a new ADR number ≥0115 — DOC to assign). Do not overwrite existing ADRs.
3. **Paths `docs/02-plan/phase-1-plan.md` and `docs/07-acceptance/*` do not
   exist.** Real homes: `docs/03-plan/phases/phase-01-core/{tasks,acceptance,
   STATUS,done}.md`. Do not create the `02-plan`/`07-acceptance` tree without
   a DOC decision (see FU-DOC-01/FU-DOC-02).
4. **Tanzil CC BY 3.0 / publisher / release remain unverified.** Keep
   `pending_owner_verification` / `source_verification_required` until license
   text is captured. No bundling, no vendoring, no manifest adoption before
   that.
5. **QAC v0.4 / Amiri OFL / KFGQPC-second-corpus candidates are unevaluated.**
   Keep `unknown` / `source_verification_required` until evaluated per
   ADR-0203 §5.

### Still needs a human (unchanged, with orchestrator wording attached)

1. OD-01 bundling gate: license text + URL + capture date in `licenses/tanzil/`
   (or an edition already held).
2. OD-02: one named reviewer (name + qualification).
3. OD-03: Tier-2 corpus identity + license (else tier-1-only close at 1.2b end).
4. OD-09/OD-10: Sprint-1.1 kickoff calendar date for absolute deadlines.
5. OD-11: QAC license verification / written permission before vendoring.
6. OD-12: named linguist before any lossy profile ships.
