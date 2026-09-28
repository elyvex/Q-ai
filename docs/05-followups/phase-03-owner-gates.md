# Phase 3 (Quran Search & Linguistics) — Owner Gates OD-11 / OD-12 / D-08

> **Purpose.** This record closes no decision. It records, for the human-only
> gates that block *real* licensed morphology activation and *scholarly*
> ratification of the linguistics surfaces, exactly what is open, what each gate
> blocks, and the precise action plus command/file that would close it. It
> exists so Phase 3 can be declared *engineered* while remaining unambiguous
> that the phase is **not** a licensed-dataset, linguist-ratified release.
>
> **Agent-uncloseable.** No agent, model, or importer may close any gate below.
> No dataset slug, publisher, release, license, SPDX id, capture date, capturer,
> linguist name, or sign-off may be invented, inferred, or substituted. Every
> such value stays `_unassigned_` / `pending_owner_verification` until a human
> owner writes it in. See `.agent/coding-rules.md`, `licenses/README.md`, and
> ADR-0203/ADR-0204.
>
> **Status source of truth.** The canonical rows remain in
> `docs/05-followups/decisions-needed.md` (`OD-11` and `OD-12`, both 🔴). This
> file is a supporting record; it never replaces those rows and never flips one
> to resolved. Owner intent recorded in `docs/05-followups/owner-decisions.md`
> (QAC v0.4 gated candidate; identity-profile-only, linguist 🔴) is
> **unratified** — it does not close a gate.
>
> **Status key:** 🔴 unanswered (blocks licensed activation and/or scholarly
> ratification) · 🟢 answered (with date + where recorded). Both gates are 🔴 at
> the time of writing.
>
> **Evidence base.** `docs/02-architecture/decisions/ADR-0203-quran-morphology-dataset.md`
> (Draft), `ADR-0204-normalization-rules.md` (Draft),
> `ADR-0205-profile-ladder.md` (Draft), `ADR-0210-root-convention.md` (Draft),
> `ADR-0215-tagset.md` (Draft), `licenses/README.md`,
> `fixtures/quran/morphology/license-matrix.json`,
> `.planning/phases/03-quran-search-linguistics/03-RESEARCH.md` §Gap Register
> (BLOCKED table), and `docs/03-plan/phases/phase-02-rag/remaining-engineering-plan.md`
> §licensing boundary.

---

## OD-11 — Morphology dataset selection & licensing (ADR-0203)

- **Status:** 🔴 unanswered (`decisions-needed.md` OD-11; re-confirmed 2026-09-28
  by plan 03-06 and 03-08).
- **Question:** Which morphology dataset ships as the default provider, under what
  license, and is the policy ADR-0203 **Option A** (bundle a licensed dataset) or
  **Option B** (ship no dataset; the operator imports one)?
- **What it blocks:** real licensed morphology activation, and therefore the
  *scholarly* credibility of SC3 (lemma/root/analyses/word family) and SC4
  (frequency/distribution/co-occurrence). Until it closes, SC3/SC4 are satisfied
  **behaviorally on the synthetic `synthetic_test_only` lexicon only** — never as
  a scholarly result on real data.
- **Current fallback in force:** ADR-0203 **Option B**. The `qac` entry in
  `fixtures/quran/morphology/license-matrix.json` is
  `pending_license_review` / `redistribution_allowed: false`; the activation gate
  (gate 6, `QAI-MORPH-0006`) rejects any dataset whose license evidence is absent
  or forbids redistribution, so an unverified provider can never be silently
  bundled or activated.
- **Closing action (human owner only):**
  1. Capture the QAC license to `licenses/qac/{LICENSE.txt,capture.json,attribution.txt}`
     per `licenses/README.md`. The `capture.json` **mandatory** fields are
     `source_url`, `capture_date`, and `capturer`; also record `spdx_id` (if one
     exists), `license_sha256` of the raw `LICENSE.txt`, and the
     `redistribution_allowed` / `modification_allowed` / `attribution_required`
     flags.
  2. **If** `redistribution_allowed: true` with all fields present: ratify
     ADR-0203 **Option A** and set the `qac` entry in
     `fixtures/quran/morphology/license-matrix.json` to the captured permissive
     values (drop `pending_license_review`, set `redistribution_allowed: true`,
     fill `capture_date`/`capturer`/`spdx_id`/`license_sha256`).
  3. **Otherwise:** accept ADR-0203 **Option B** and keep the CLI user-supplied
     import (`qai quran morphology import … --license-status … --license-json …`)
     plus the typed "no dataset active" errors — never guessed or bundled data.
  4. Record the decision + date in `docs/05-followups/owner-decisions.md`
     (append-only) and update the `decisions-needed.md` OD-11 row.
- **Exact command after the owner edit (bundling half):**
  `qai quran morphology import <dataset.json> --dataset <slug@version> --edition
  <slug@version> --license-status <status> --license-json <capture.json>` then
  `qai quran morphology activate <batch> --approval <id> --yes` (the activation
  path re-validates the captured evidence; nothing is promoted without it).
- **Blocks (legacy ids):** `P2-T56`/`P2-T64`/`P2-T68`/`P2-T74`, and the scholarly
  half of `AC-P2-01`.
- **What plan 03-08 did NOT do:** it recorded this gate and proved the alpha on
  the synthetic fixture; it introduced no dataset slug, license value, SPDX id,
  capture date, or capturer, and it flipped no matrix entry and no ADR.
- **Recorded in / evidence:** `docs/05-followups/decisions-needed.md` (OD-11,
  🔴), `docs/05-followups/owner-decisions.md` (gated candidate, unratified),
  `fixtures/quran/morphology/license-matrix.json` (`qac` pending), plan 03-06
  SUMMARY, and Section §OD-11 of this file.

---

## OD-12 / D-08 — Normalization catalog + named linguist

- **Status:** 🔴 unanswered (`decisions-needed.md` OD-12; D-08 linguist sign-off
  also open; re-confirmed 2026-09-28).
- **Question:** Which normalization rule catalog is ratified, and who is the
  qualified Arabic linguist that signs off on the catalog, the morphology
  tagset, the root conventions, and the alignment rules?
- **What it blocks:**
  - **OD-12 (catalog + linguist):** acceptance of ADR-0204 (normalization rules
    N01–N24) and ADR-0205 (profile ladder L0–L8). The catalog is *implemented in
    code* but not linguist-ratified; the golden/curated sets stay
    `reviewed_by: pending-linguist`.
  - **D-08 (linguist sign-off):** acceptance of ADR-0210 (root convention and
    cross-dataset unification) and ADR-0215 (unified tagset). Word-family and
    root/lemma results cannot be ratified as **linguistically correct** until a
    linguist signs the tagset, root conventions, and alignment rules.
- **Current fallback in force:** every phase-3 golden/curated header reads
  `reviewed_by: pending-linguist` / `reviewed_at: null`; the 154-family curated
  golden set has every row `synthetic_test_only`; ADR-0204/0205/0210/0211/0215
  remain Draft/Proposed (not Accepted). No agent may flip any of them.
- **Closing action (human/editorial only):**
  1. Name a qualified Arabic linguist (≈0.4 FTE) and record the sign-off in
     `docs/reviews/` (create the directory if it does not exist) covering the
     normalization catalog, the profile ladder, the morphology tagset, the root
     conventions, and the alignment rules.
  2. Then flip ADR-0204 and ADR-0205 (OD-12) and ADR-0210 and ADR-0215 (D-08) to
     **Accepted**, and update the golden/curated headers from
     `reviewed_by: pending-linguist` to the named reviewer + date.
  3. Record the decision + date in `docs/05-followups/owner-decisions.md`
     (append-only) and update the `decisions-needed.md` OD-12 row.
- **Exact step after the owner edit:** the golden/curated header line changes to
  `"reviewed_by": "<named linguist>", "reviewed_at": "<YYYY-MM-DD>"` — the golden
  runners assert this labeling, so a real signature is a deliberate, reviewable
  edit, never a silent pass.
- **Blocks (legacy ids):** `AC-P2-02`, `AC-P2-46` (golden ratification), and the
  linguistic-correctness half of SC3/SC4.
- **What plan 03-08 did NOT do:** it named no linguist and flipped no ADR; the
  owner-gates record and the deferral ledger keep OD-12/D-08 explicitly open.
- **Recorded in / evidence:** `docs/05-followups/decisions-needed.md` (OD-12,
  🔴), `docs/05-followups/owner-decisions.md` (linguist 🔴), the Draft ADRs
  0204/0205/0210/0211/0215, and Section §OD-12 of this file.

---

## No agent may close OD-11 or OD-12/D-08

Both gates are human-only. An agent may:

- prepare the generic shape (importer, activation gate, machine-readable license
  matrix, normalization catalog and profiles, typed family relations and
  counting rules, CLI/HTTP surfaces);
- run the mechanism against **synthetic** fixtures (labeled
  `synthetic_test_only`);
- record the gate, its evidence, and its closing action (this file).

An agent may **not**: choose or bundle a dataset, assert a license, supply a
SPDX id / capture date / capturer, name a linguist, sign off a tagset or root
convention, or mark any gate resolved. `docs/05-followups/decisions-needed.md`
remains the source of truth; this file closes nothing.

---

*Phase: 3-Quran Search & Linguistics · Owner-gate record created 2026-09-28 by
plan 03-08. No gate is resolved by this file.*
