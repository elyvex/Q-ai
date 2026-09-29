# Phase 4 (Quran Graph) — Owner Gates OD-11 / OD-12 / P4-X

> **Purpose.** This record closes no decision. It records, for the human-only
> gates that block *real* licensed morphology activation, *scholarly*
> ratification of the word-root projection, and *architectural* ratification of
> the graph ADRs, exactly what is open, what each gate blocks, and the precise
> action plus command/step that would close it. It exists so Phase 4 can be
> declared *engineered* while remaining unambiguous that the phase is **not** a
> licensed-dataset, linguist-ratified, or ADR-ratified release.
>
> **Agent-uncloseable.** No agent, model, or importer may close any gate below.
> No dataset slug, publisher, release, license, SPDX id, capture date, capturer,
> linguist name, sign-off, or ADR status flip may be invented, inferred, or
> substituted. Every such value stays `_unassigned_` / `pending_owner_verification`
> / `Proposed` until a human owner writes it in. See `.agent/coding-rules.md`,
> `licenses/README.md`, and ADR-0203/ADR-0204/ADR-0202/ADR-0702/ADR-0217/ADR-0218.
>
> **Status source of truth.** The canonical rows remain in
> `docs/05-followups/decisions-needed.md` (`OD-11` and `OD-12`, both 🔴) and
> `docs/03-plan/phases/phase-04-quran-graph/tasks.md` (swimlane X: `P4-X01`,
> `P4-X02` ☐; `P4-X03`, `P4-X05` ◐ draft-ready, ratification outstanding).
> This file is a supporting record; it never replaces those rows and never flips
> one to resolved. Owner intent recorded in `docs/05-followups/owner-decisions.md`
> is **unratified** — it does not close a gate.
>
> **Status key:** 🔴 BLOCKED (no agent may pass; a human close is recorded
> elsewhere) · 🟢 answered (with date + where recorded). Every gate below is 🔴
> at the time of writing.
>
> **Evidence base.** `docs/02-architecture/decisions/ADR-0203-quran-morphology-dataset.md`
> (Draft), `ADR-0204-normalization-rules.md` (Draft), `ADR-0202-graph-store.md`
> (Proposed), `ADR-0702-*` (Proposed), `ADR-0217-graph-query-limits.md`
> (Proposed), `ADR-0218-graph-export-formats.md` (Proposed),
> `fixtures/quran/morphology/license-matrix.json`,
> `fixtures/quran/graph/concept-seed-v1.json` (OD-11/OD-12 BLOCKED header),
> `fixtures/quran/graph/annotation-goldens.json` (BLOCKED header),
> `docs/05-followups/phase-03-owner-gates.md` (OD-11/OD-12 mechanism record),
> and `.planning/phases/04-quran-graph/04-01-SUMMARY.md` through
> `04-05-SUMMARY.md` (behavioral evidence on synthetic fixtures only).

---

## OD-11 — Morphology dataset selection & licensing (ADR-0203)

- **Status:** 🔴 BLOCKED (`decisions-needed.md` OD-11; re-confirmed 2026-09-29
  by plans 04-01 through 04-05).
- **Question:** Which morphology dataset ships as the default provider, under what
  license, and is the policy ADR-0203 **Option A** (bundle a licensed dataset) or
  **Option B** (ship no dataset; the operator imports one)?
- **What it blocks in Phase 4:** production acceptance of the **word-root
  projection** (`quran-wordroot-v1`). The builder, the `HAS_ROOT` / `HAS_LEMMA` /
  `SAME_ROOT_AS` / `SAME_LEMMA_AS` edges, and the root-family reads are
  *implemented and proven* but satisfied **behaviorally on the synthetic
  `synthetic_test_only` lexicon only** — never as a scholarly result on real
  data. Pending owner ratification `P4-X01`…`P4-X05` is tagged in the word-root
  CLI help and the `graph_build` test names; the concept-seed and
  annotation-golden fixtures carry explicit OD-11/OD-12 BLOCKED headers.
- **Current fallback in force:** ADR-0203 **Option B** (inherited from Phase 3).
  The word-root builder gates on the active attributed dataset
  (`require_active_dataset`); with no active dataset it returns the typed
  `UnavailableDataset` diagnostic (`QAI-MORPH-0004`, CLI exit 5, HTTP 404) naming
  the capability — never heuristics, never guessed roots.
- **Closing action (human owner only):** identical to the Phase-3 record
  (`phase-03-owner-gates.md` §OD-11): capture the license to
  `licenses/<dataset>/{LICENSE.txt,capture.json,attribution.txt}` per
  `licenses/README.md` (mandatory `source_url`, `capture_date`, `capturer`;
  plus `spdx_id`, `license_sha256`, redistribution/modification/attribution
  flags); ratify ADR-0203 **Option A** (permissive capture) or accept
  **Option B** (operator import); record in `owner-decisions.md` (append-only)
  and update the `decisions-needed.md` OD-11 row.
- **Exact command after the owner edit (dataset half):**
  `qai quran morphology import <dataset.json> --dataset <slug@version> --edition
  <slug@version> --license-status <status> --license-json <capture.json>` then
  `qai quran morphology activate <batch> --approval <id> --yes`, followed by
  `qai quran graph build` to republish the word-root projection from the
  licensed dataset (or `qai quran graph doctor-repair rebuild-projection
  --projection quran-wordroot-v1 --yes` once a stale build exists).
- **Blocks (legacy ids):** word-root production acceptance; `P4-X04` (Phase-2
  morphology landing, gates M5 and full phase exit); the scholarly half of any
  root/lemma claim served from the graph.
- **What Phase 4 did NOT do:** it selected no dataset, asserted no license,
  flipped no matrix entry and no ADR, and served no real-data morphology result;
  the `UnavailableDataset` gate and the synthetic-only labeling are the proof.
- **Recorded in / evidence:** `docs/05-followups/decisions-needed.md` (OD-11,
  🔴), `fixtures/quran/morphology/license-matrix.json` (`qac` pending),
  `crates/application/tests/graph_build.rs` (OD-11 BLOCKED pins), the
  `Review`/`Doctor` CLI help text, and Section §OD-11 of this file.

---

## OD-12 — Normalization catalog + named linguist (ADR-0204/0205, D-08 tagset)

- **Status:** 🔴 BLOCKED (`decisions-needed.md` OD-12; re-confirmed 2026-09-29
  by plans 04-01 through 04-05).
- **Question:** Which normalization rule catalog is ratified, and who is the
  qualified Arabic linguist that signs off on the catalog, the morphology
  tagset, the root conventions, and the alignment rules?
- **What it blocks in Phase 4:** linguist ratification of every
  human-attributed graph assertion. The review lifecycle (propose / suggest /
  accept / reject / dispute / correct with reviewer + timestamp) is *implemented
  and proven*, but reviewer names on real scholarship are unsigned until a
  linguist exists; `layer-D` suggestions stay `pending` and labeled, never
  promoted, until a recorded human decision accepts them.
- **Current fallback in force:** seed and golden fixtures label attribution
  `pending-scholar` / synthetic scope; ADR-0204/0205/0210/0211/0215 remain
  Draft/Proposed (not Accepted). No agent may flip any of them.
- **Closing action (human/editorial only):** identical to the Phase-3 record
  (`phase-03-owner-gates.md` §OD-12/D-08): name the linguist, record the
  sign-off in `docs/reviews/`, flip ADR-0204/0205 (OD-12) and ADR-0210/0215
  (D-08) to **Accepted**, re-label fixture headers from `pending-scholar` to
  the named reviewer + date, record in `owner-decisions.md` (append-only), and
  update the `decisions-needed.md` OD-12 row.
- **Exact step after the owner edit:** the concept-seed and annotation-golden
  header lines change to `"reviewer": "<named linguist>", "reviewed_at":
  "<YYYY-MM-DD>"` — the seed validators assert the current labeling, so a real
  signature is a deliberate, reviewable edit, never a silent pass.
- **Blocks (legacy ids):** scholarly acceptance of annotated assertions;
  `AC-P4` review-lifecycle ratification rows that name a reviewer.
- **What Phase 4 did NOT do:** it named no linguist, signed no tagset or root
  convention, and flipped no ADR; suggestions never self-promote (pinned by
  `graph_review` pending-label cases).
- **Recorded in / evidence:** `docs/05-followups/decisions-needed.md` (OD-12,
  🔴), `docs/05-followups/owner-decisions.md` (linguist 🔴), the Draft ADRs,
  `crates/application/tests/graph_review.rs` (suggestion-labeling pins), and
  Section §OD-12 of this file.

---

## P4-X01 — Ratify ADR-0202 graph store (Proposed → Accepted)

- **Status:** 🔴 BLOCKED (`tasks.md` swimlane X, ☐ `_unassigned_`, due before M1).
- **Question:** Accept the decided graph architecture (relational adjacency +
  bounded traversal first, `GraphStore` port, authority-vs-projection split, no
  query-language leakage, SQLite single source of truth) as the project
  direction?
- **What it blocks:** architectural finality of everything Phase 4 built: the
  `SqliteGraphStore` snapshot adapter, the staged-batch fenced builder, and the
  CLI/HTTP/tool surface split. The code is implemented and conformance-proven;
  the *decision* is unratified.
- **Closing action (human owner only):** flip
  `docs/02-architecture/decisions/ADR-0202-graph-store.md` to **Accepted**,
  record the date + rationale in `docs/05-followups/owner-decisions.md`
  (append-only), and check the `P4-X01` row in `tasks.md`.
- **Exact step:** no command — an ADR status edit plus the owner-decisions log
  entry. Verification afterward: `cargo xtask arch-check` (the port direction
  the ADR blesses stays enforced in CI).
- **What Phase 4 did NOT do:** it implemented *toward* the Proposed ADR and
  recorded the ☐ as BLOCKED in plan summaries; it flipped no ADR status.

---

## P4-X02 — Accept Phase-3 subset of ADR-0702 (snapshot/publication/fencing)

- **Status:** 🔴 BLOCKED (`tasks.md` swimlane X, ☐ `_unassigned_`, due before M1).
- **Question:** Accept the snapshot/publication/fencing contract subset of
  ADR-0702 that the graph builders rely on (staged-batch build, single-tx
  fenced publish, generation stamp, single-step rollback retention)?
- **What it blocks:** architectural finality of the build pipeline
  (`quran_graph_build.rs` reserve → stage → verify → flip) and the
  delete-plus-rebuild authority-preservation contract (AC-P4-03).
- **Closing action (human owner only):** accept the ADR-0702 subset in the ADR
  file, record in `owner-decisions.md` (append-only), check the `P4-X02` row.
- **Exact step:** no command — an ADR status edit plus the owner-decisions log
  entry. Verification afterward: `cargo test -p application --test graph_build`
  (the AC-P4-03 preservation proof the contract requires).
- **What Phase 4 did NOT do:** it built the fence without a ratified contract
  and recorded the ☐ as BLOCKED; it accepted no ADR subset.

---

## P4-X03 — Graph query safety limits ADR-0217 (Proposed → Accepted)

- **Status:** 🔴 BLOCKED (`tasks.md` swimlane X, ◐ draft ready 2026-09-24;
  ratification is owner work, due before M3).
- **Question:** Ratify the budget defaults (6 hops / 500 nodes / 10 paths /
  2000 edges / 128 fanout / 5 s), the ranges, and the pre-flight-vs-in-flight
  exhaustion semantics drafted in ADR-0217?
- **What it blocks:** finality of every budget number the graph surfaces
  enforce. The limits are implemented in `crates/quran-graph/src/model.rs` with
  backend-generic conformance pins; the *numbers* are unratified.
- **Closing action (human owner only):** flip
  `docs/02-architecture/decisions/ADR-0217-graph-query-limits.md` to
  **Accepted**, record in `owner-decisions.md` (append-only), check `P4-X03`.
- **Exact step:** no command — an ADR status edit plus the owner-decisions log
  entry. Verification afterward: `cargo test -p quran-graph` (budget pins) plus
  `cargo test -p application --test graph_paths` (exhaustion pins).
- **What Phase 4 did NOT do:** it encoded the draft numbers and proved
  exhaustion behavior on both backends; it ratified nothing.

---

## P4-X05 — Export format decision ADR-0218 (Proposed → Accepted)

- **Status:** 🔴 BLOCKED (`tasks.md` swimlane X, ◐ draft ready 2026-09-24;
  ratification is owner work, due before M6).
- **Question:** Confirm Graph JSON v1 as the mandatory lossless envelope
  (assertions-travel-with-edges, explicit truncation, policy-on-export) and
  GraphML as a deferred lossy view only?
- **What it blocks:** finality of the export contract (`quran_graph_export.rs`,
  `export_document`, DOT/SVG static rendering). The envelope is implemented
  with byte-absence pins for restricted/tombstoned material; the *format
  decision* is unratified.
- **Closing action (human owner only):** flip
  `docs/02-architecture/decisions/ADR-0218-graph-export-formats.md` to
  **Accepted**, record in `owner-decisions.md` (append-only), check `P4-X05`.
- **Exact step:** no command — an ADR status edit plus the owner-decisions log
  entry. Verification afterward: `cargo test -p application --test graph_export`
  (travel-with-edges + byte-absence pins).
- **What Phase 4 did NOT do:** it shipped the Graph JSON exporter and deferred
  GraphML as lossy-only per the draft; it ratified nothing.

---

## No agent may close OD-11, OD-12, or P4-X01/X02/X03/X05

All six gates are human-only. An agent may:

- build the mechanism against **synthetic** fixtures (labeled
  `synthetic_test_only` / `pending-scholar`);
- gate production paths with typed refusals (`QAI-MORPH-0004`, pending-labeled
  suggestions, Proposed ADR headers);
- record the gate, its evidence, and its closing action (this file).

An agent may **not**: choose or bundle a dataset, assert a license, supply a
SPDX id / capture date / capturer, name a linguist, sign off a tagset or root
convention, flip an ADR to Accepted, or mark any gate resolved.
`docs/05-followups/decisions-needed.md` (OD-11/OD-12) and
`docs/03-plan/phases/phase-04-quran-graph/tasks.md` (swimlane X) remain the
sources of truth; this file closes nothing. (`P4-X04` — Phase-2 morphology
landing — is covered by OD-11 above, not by a separate row here.)

---

*Phase: 4-Quran Graph · Owner-gate record created 2026-09-29 by plan 04-05
(D-01 closure). No gate is resolved by this file.*
