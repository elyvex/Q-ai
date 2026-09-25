# Decisions Needed From the Owner

> **Purpose:** every question in this file can only be answered by a human
> owner/editor — not by an agent. Each item blocks the listed tasks or
> acceptance criteria. Nothing here is invented: owners, dates, datasets,
> reviewers, and licenses are `_unassigned_` / `TBD` until a human writes them
> in.
>
> **How to answer:** reply with the item ID (e.g. `OD-01`) plus the answer.
> The answering agent will record it in the ADR / task / criterion named in
> "Recorded in" and flip the blocked rows. Do not answer by editing this file
> directly unless that is the agreed workflow.
>
> **Scope:** Phase 1 (Canonical Quran Core). Phase-0 items live in
> `docs/05-followups/phase-0-remaining-work.md`; Phase-2+ technical questions
> live in `docs/05-followups/open-questions.md`.
>
> **Status key:** 🔴 unanswered · 🟢 answered (with date + where recorded)

> **Architecture direction (2026-09-18):** the owner's multi-edition /
> multi-riwayah / multilingual decisions and the source-verification backlog are
> recorded in `docs/05-followups/owner-decisions.md` and
> `docs/02-architecture/upstream-sources.md`. Items there complement (do not
> replace) the OD rows below.

---

## OD-01 — Licensed Quran dataset + bundle policy (blocks AC-P1-01, T58)

- **Status:** 🔴 unanswered
- **Question:** Which Arabic edition is the canonical dataset (script,
  riwayah, numbering, normalization), is redistribution licensed, and do we
  bundle it or ship the fixture with user-supplied import?
- **Options (ADR-0101):** A. bundle a licensed edition · B. ship
  `test-edition-min`, user imports their own approved edition (current
  fallback) · C. defer Phase 1 until licensed.
- **Needed:** dataset identity + license evidence + policy choice (A/B/C).
- **Blocks:** `P1-X01`, `P1-T01`, `P1-T02`, `P1-T56`, `P1-T58`, `AC-P1-01`.
- **Current fallback:** engineering runs on synthetic `test-edition-min`;
  candidates surveyed in ADR-0101 are all unverified.
- **Partial answer (2026-09-18, architectural intent only):** primary/default
  representation = Uthmani script + Ḥafṣ ʿan ʿĀṣim; model must be
  multi-edition/multi-riwayah/multilingual; `quran-api` is the preferred
  catalog, `quran-database` a schema reference, `quranchecksum` an integrity
  reference for compatible datasets. Recorded in
  `docs/05-followups/owner-decisions.md` and ADR-0101 (still Draft).
  Still pending: exact `upstream_edition_slug`, publisher/release metadata,
  per-edition license evidence, numbering/normalization sign-off, and policy
  choice (A/B/C). Status stays 🔴 until those land.
- **Recorded in:** `docs/02-architecture/decisions/ADR-0101-initial-quran-dataset.md`
  (Draft), `tasks.md` §1/`§2`, `acceptance.md` §1.1.
- **Phase 2 record (2026-09-25, plan 02-07):** `docs/05-followups/phase-02-owner-gates.md` §OD-01.
  Blocked and agent-uncloseable; closing action + import command recorded there. **Status stays 🔴.**

## OD-02 — Named editorial reviewer + `verified_by` sign-off

- **Status:** 🔴 unanswered
- **Question:** Who is the qualified reviewer that compares the text against a
  recognized printed muṣḥaf, and what sample + method did they sign?
- **Needed:** reviewer name + sample + comparison method, recorded in the
  edition's `verified_by` field.
- **Blocks:** `P1-X02`, `P1-T55`, `AC-P1-01` (with OD-01).
- **Recorded in:** ADR-0101, `tasks.md` §1/`§7`.
- **Phase 2 record (2026-09-25, plan 02-07):** `docs/05-followups/phase-02-owner-gates.md` §OD-02.
  Blocked and agent-uncloseable; reviewer/sample/method and the `qai quran edition verify` command are recorded
  there; any in-tree reviewer string is a test-only placeholder. **Status stays 🔴.**

## OD-03 — Reference corpus + comparison procedure + sign-off

- **Status:** 🔴 unanswered
- **Question:** Which independent reference corpus, which comparison procedure,
  and who signs off?
- **Needed:** corpus identity + procedure + signer name. Until then QV-015
  stays a recorded skip (Info, never silent pass).
- **Blocks:** `P1-X03`, `P1-T03`, `P1-T26◐`, QV-015.
- **Recorded in:** `docs/02-architecture/decisions/ADR-0114-reference-corpus-comparison.md`
  (Draft), `tasks.md` §1/`§4.2`.
- **Phase 2 record (2026-09-25, plan 02-07):** `docs/05-followups/phase-02-owner-gates.md` §OD-03.
  Blocked and agent-uncloseable; corpus/procedure/signer and the `qai quran import --reference` command are
  recorded there, including the hash-only limitation of the `spqrxi/quranchecksum` candidate. **Status stays 🔴.**

## OD-04 — Debug-reader web font: name + license (blocks T54 / AC-P1-18)

- **Status:** 🔴 unanswered
- **Question:** Which web font ships with the debug reader (bundled
  `@font-face`), and is its license cleared for redistribution?
- **Needed:** font name + license sign-off. The reader works today on a CSS
  system stack (Amiri → Noto Naskh Arabic → Scheherazade New → platform
  fonts); only the bundled binary needs this decision.
- **Blocks:** `P1-T54`, `AC-P1-18`.
- **Recorded in:** `tasks.md` §7, `acceptance.md` §1.6, `STATUS.md` §3.2.

## OD-05 — Estimate / schedule gap: 82 ed stated vs 131.0 ed summed

- **Status:** 🔴 unanswered
- **Question:** Accept ~8.7 weeks, cut scope explicitly, or add a fourth
  engineer? (Includes the Sprint 1.2 42-ed overload and the 1.2a/1.2b split.)
- **Needed:** one of the three options, recorded; retrofit-impossible
  deliverables (D1.1, D1.4, D1.5, D1.6, D1.13) must not absorb a cut.
- **Blocks:** Phase-1 scheduling (`tasks.md` §9, `README.md` §9.1).
- **Recorded in:** `tasks.md` §9.1/`§9.2`, `done.md` §7 (OWN-03).

## OD-06 — Provisional HTTP stack: ratify or redirect axum + tower-http

- **Status:** 🔴 unanswered
- **Question:** Ratify axum + tower-http for API v1, or redirect to another
  framework?
- **Blocks:** `AC-P1-14` final wording; health endpoints unchanged either way.
- **Recorded in:** `done.md` §7 (OWN-04), `technology-stack.md` §4.

## OD-07 — Coverage gates: ratify the proposed floors

- **Status:** 🔴 unanswered
- **Question:** Ratify the `acceptance.md` §4 floors (quran-core ≥ 90%,
  validation/tokenize/hashing ≥ 90%, adapters/differ ≥ 80%, citations ≥ 85%,
  storage-sqlite ≥ 80%, surfaces smoke-only)?
- **Blocks:** `acceptance.md` §4, exit gate coverage row.
- **Recorded in:** `acceptance.md` §4 ("Owner to ratify before Sprint 1.2").

## OD-08 — Exit rituals: reviewer identity + recording archive

- **Status:** 🔴 unanswered
- **Question:** Who (not the implementer) performs the §5 live walkthrough,
  and where is the recording archived?
- **Needed:** reviewer name + recording link, then per-criterion sign-off.
- **Blocks:** all 19 ◐ criteria (ritual half), `P1-T60`, `acceptance.md` §5/`§6`.
- **Recorded in:** `acceptance.md` §5/`§6`, `done.md` §3 (all "—").

## OD-09 — Exit-gate sign-off rows: owners + dates

- **Status:** 🔴 unanswered
- **Question:** Fill every `acceptance.md` §6 row (13 plan + 8 supporting
  criteria, coverage, DoD, suites, 14 ADRs, 6 migrations, editorial sign-off,
  ritual, handoff, swimlane X, Phase-1-accepted).
- **Needed:** owner + date per row.
- **Blocks:** `P1-T60`, Phase-2 unblocking.
- **Recorded in:** `acceptance.md` §6 (all Owner/Date cells empty).

## OD-10 — Swimlane X ownership: assign every row

- **Status:** 🔴 unanswered
- **Question:** Owner + decision-open date for `P1-X01`…`P1-X05` (all
  `_unassigned_` / `TBD`).
- **Blocks:** tracking of OD-01/OD-03 and Phase-2 inputs (OD-11, OD-12).
- **Recorded in:** `tasks.md` §1.

## OD-11 — Morphology dataset selection & licensing (Phase-2 input)

- **Status:** 🔴 unanswered
- **Question:** Which morphology dataset, under what license (ADR-0203)?
- **Needed:** dataset + license, before Phase 2 starts.
- **Blocks:** `P1-X04`.
- **Recorded in:** `tasks.md` §1.

## OD-12 — Normalization rule catalog + linguist engagement (Phase-2 input)

- **Status:** 🔴 unanswered
- **Question:** Which normalization rule catalog, and who is the engaged
  linguist (ADR-0204)?
- **Needed:** catalog + linguist name, before Phase 2 starts.
- **Blocks:** `P1-X05`.
- **Recorded in:** `tasks.md` §1.

## OD-13 — Phase-0 exit discrepancy reconciliation

- **Status:** 🔴 unanswered
- **Question:** Reconcile `status.md`'s outstanding Phase-0 items against the
  build prompt's Phase-0-complete declaration.
- **Blocks:** Phase-0 exit (not Phase-1 engineering).
- **Recorded in:** `done.md` §7 (OWN-05).

## OD-14 — `server` layering: accept allowlist or schedule the routing fix

- **Status:** 🔴 unanswered (deferred, not blocking Phase 1)
- **Question:** Accept `server` reaching `storage` + `tools` directly
  (allowlisted), or schedule routing through `application` + tightening the
  allowlist (suggested Phase 3)?
- **Blocks:** nothing in Phase 1; Phase-3/server-hardening scope.
- **Recorded in:** `done.md` §7 (OWN-06).

---

### Answer log

| Date | Item | Answer | Recorded in |
|---|---|---|---|
| 2026-09-24 | Agent verification pass (no owner answers received) | All 14 rows re-read against the live tree: OD-01…OD-14 stay 🔴. No agent may close a human/editor decision. Engineering progress that reduces (but does not close) the gap is recorded elsewhere: Phase-4 board reconciliation + M5 blocker correction (`phase-04-quran-graph/done.md` §2/§3), T54/T56 groundwork status (FU-P1-03), server layering still deferred (OD-14 → phase-03-server placeholder notice). | this file (unchanged status), `task-done-rollup.md` |
|---|---|---|---|
| 2026-09-18 | OD-01 (partial) | Primary/default = Uthmani script + Ḥafṣ ʿan ʿĀṣim (architectural intent, not a dataset selection); model must be multi-edition / multi-riwayah / multilingual. Exact slug, publisher/release, license evidence, numbering/normalization, and A/B/C policy still pending — OD-01 stays 🔴 | `owner-decisions.md`, ADR-0101 |
| 2026-09-18 | Upstream roles (no OD closed) | `quran-api` = preferred edition/translation catalog; `quran-database` = schema/reference source; `quranchecksum` = integrity reference for compatible datasets only. Repository licence ≠ data licence; per-edition terms stay `unknown` until cleared | `owner-decisions.md`, `upstream-sources.md`, ADR-0101 |
| 2026-09-22 | Orchestrator series OD-01…OD-14 (recommendation, no row closed) | B-now/A-track-parallel/C-rejected; OD-02 L1L2L3 method (name 🔴); OD-03 two-tier + conditional tier-1-only close; OD-05 accept 8.7 wks + 1.2a/1.2b split; OD-06 ratify axum+tower-http; OD-07 ratify floors; OD-04 system-stack only (Amiri Phase-2 candidate); OD-08/09/10 owner-present + verifier≠producer + 1.2b+2d close; OD-13 Phase-0 complete per evidence, reconcile status.md ≤1d; OD-11 QAC v0.4 gated candidate; OD-12 identity-profile-only, linguist 🔴; OD-14 allowlist-accept + Phase-3 review. All 🔴 until human ratifies. Conflicts flagged: custom slug ≠ upstream slug; ADR-0102…0106 numbers already taken; `docs/02-plan/*` + `docs/07-acceptance/*` paths do not exist | `owner-decisions.md` § Orchestrator recommendations 2026-09-22 |
| 2026-09-22 | Owner addendum A0…A10 + clarifications (append-only, no row closed) | A0 append-only log; C1 full-Uthmani `qai_edition_id` candidate; C2 ranked fallback chain (bundling-only gate); A1 reviewer≠signer independence (governs OD-03 signer + OD-10 X02/X03 mapping); A2 Tier-2 shortlist; A3 license-capture standard; A4 +1wk buffer (~9.7 wks); A5 CI arch-test (extends `xtask arch-check`); A6 recording redaction; A7 vector path (new, needs task); A8 auto-escalation; A9 owner-authored standing rule for low-risk technicals only (second-pass effective 2026-09-25 barring objection); A10 per-OD DoD checklist. Six 🔴 inputs unchanged | `owner-decisions.md` §§ Addendum + Clarifications 2026-09-22 |
