# Phase 1 — Canonical Quran Core

**Phase ID:** P1
**Phase name:** Canonical Quran Corpus — structure, addressing, import, validation, reader
**Plan version:** 1.0.0
**PRD baseline:** Q-ai PRD v0.3.2
**PRD traceability:** §7 · §12 · §13.3 · §19.1 · §21.2 · §22 · §27.1 · §34 · §35.1 · §35.3 ·
§40 · §43 (Phase 1) · §46 · §50 · §55 · §56 · §88 (item 2)
**Depends on:** Phase 0 — all deliverables (see `../phase-00-foundation/done.md` §6)
**Blocks:** Phase 2 (search/linguistics) · Phase 3 (graph) · Phase 4 (Web reader) ·
Phase 5 (Quran↔hadith/tafsir links)
**Target duration (plan):** 6 calendar weeks ≈ 15–17 engineer-weeks, 3 engineers
**Task-board estimate:** 131.0 ed across 60 tasks + 3 swimlane decisions → see §9
**Status:** In progress; source decisions and exit gates pending (reviewed 2026-09-17)

Canonical models, reference parsing, staged import/validation, activation/rollback,
reader services, translations/glosses, citations, CLI, API and a labelled debug reader
are implemented. See [STATUS.md](STATUS.md), [tasks.md](tasks.md) and [done.md](done.md)
for evidence rather than the original schedule below.

The bundled edition is synthetic, not Quran text. Dataset licensing/editorial approval,
independent reference-corpus verification, full-edition golden/soak runs, debug-reader
font licensing and the reviewer exit ritual remain open. Automated fixture evidence does
not establish phase acceptance. P2 implementation has begun while these gates remain open.

**Companion documents**

| File | Purpose |
|---|---|
| `plan.md` | Full Phase-1 plan (source of truth — do not edit without an ADR) |
| `tasks.md` | Work Breakdown Structure as a live task board (P1-T01…T60, swimlane X) |
| `acceptance.md` | Phase-1 exit gate: AC-P1-01…13 + supporting gates and required test suites |
| `done.md` | Append-only completion ledger (tasks, ACs, ADRs, deviations, deferrals) |

---

## 1. How To Read This Phase

- **D1.x** = Deliverable (a shippable, testable unit).
- **P1-Tnn** = Task (a work item on the board, `tasks.md`).
- **AC-P1-nn** = Acceptance Criterion (phase exit gate, `acceptance.md`).
- **ADR-01nn** = Architecture Decision Record required inside this phase.
- **I1…I7** = the non-negotiable invariants introduced here (§3).
- **QV-nnn** = a corpus-validation rule (§6, `plan.md` §4/D1.4).

Every task lists `Deliverable`, `Depends`, `Estimate (engineer-days)`, and `Owner role`.
Nothing in Phase 1 may weaken a Phase-0 guarantee; it may only build on them.

---

## 2. Phase Objective

Stand up the **canonical Quran engine**: a lossless, hash-verified, versioned, structurally
validated corpus with stable addressing and deterministic, LLM-free lookup — the foundation
every other Q-ai feature cites.

### The Phase-1 promise (PRD §7.3, §40, §46)

> For any legal reference, Q-ai returns the exact canonical Arabic text of an identified
> edition version, byte-for-byte equal to the approved source, with a resolvable citation,
> in single-digit milliseconds, **without** an LLM, vector store, or network call — and no
> code path exists that can silently change that text.

This is a load-bearing promise. If Phase 1 ships a corpus that *usually* returns the right
text, every later citation, graph edge, tafsir link, and research report inherits a silent
correctness bug that no amount of downstream tooling can detect. The gate is therefore
byte-equality, not plausibility.

---

## 3. Non-Negotiable Invariants Introduced Here

| # | Invariant | Enforcement | Where tested |
|---|---|---|---|
| **I1** | Canonical ayah text is immutable per edition version | insert-only tables + DB triggers + `CanonicalWriter`/`ApprovalToken` (Phase 0) | `acceptance.md` §3.2 immutability; AC-P1-08/09 |
| **I2** | Canonical text is never produced by a model | canonical write path has no model dependency; architecture test forbids `llm`/`embeddings`/`retrieval` deps in `quran-core`/`quran-corpus` | arch-check; D1.6 |
| **I3** | Different readings/editions are never merged into a synthetic text | `edition_id` is part of every canonical primary key and every returned record | QV-027; AC-P1-04 |
| **I4** | Token order is stable and verifiable | `(edition_id, surah, ayah, position)` unique + per-ayah order checksum | QV-024; AC-P1-07/13 |
| **I5** | A partially imported edition can never become `Active` | physically separate staging tables + single-transaction pointer flip | AC-P1-03/10/11 |
| **I6** | Every returned quotation carries edition id + version + hash | `QuranQuotation` has no constructor without them | AC-P1-05; `citations` tests |
| **I7** | Corrections create a new version with a difference report and human approval | `CanonicalChangeRequest` (Phase 0) + `quran_edition_v1` differ | AC-P1-03; D1.9 |

---

## 4. Scope Fence

### 4.1 In scope

| # | Item | PRD |
|---|------|-----|
| 1 | Quran edition model (all §7.2 fields) | §7.2 |
| 2 | Hierarchy: Edition → Surah → Ayah → Segment → Token (surface only) | §7.1 |
| 3 | Stable addressing + reference grammar + parser/serializer | §7.4 |
| 4 | Structural metadata: juz, hizb, rubʿ, manzil, page, sajdah, ruku, Makki/Madani, basmala policy | §7.4, §13.1 |
| 5 | Edition source format spec + `quran_edition_v1` structure validator | §22.4, §35.1 |
| 6 | Canonical importer job (quarantine → parse → validate → hash → stage → approve → activate) | §34 |
| 7 | Corpus validator (counts, identifiers, Unicode, token order, checksum, round-trip, reference-corpus comparison) | §35.1 |
| 8 | Atomic activation + rollback + edition version diff | §7.3, §85 |
| 9 | Exact lookup + context retrieval service with caching | §11.4, §40 |
| 10 | Translation model (attributed, verse-level and optional word-level) | §7.2, §13.1 |
| 11 | Quran read API v1 (`editions`, `surahs`, `ayahs/:ref`, `context/:ref`) | §27.1 |
| 12 | Tool result contract + first tools `quran.get_ayah`, `quran.get_context` | §11.4, §12 |
| 13 | Research checksum computation (deterministic parts) | §12.1 |
| 14 | Citation resolver v1 + deep-link format | §13.3, §21.2, §35.3 |
| 15 | CLI: `qai quran get/list/import/validate/activate/diff/rollback` | §26 |
| 16 | `qai doctor --quran` | §50 |
| 17 | Corpus integrity test suite + golden fixtures | §35.1, §46 |

### 4.2 Out of scope — do not build these in Phase 1

| Excluded | Lands in |
|---|---|
| **All search** (exact / normalized / phrase / concatenated / regex) | **Phase 2** |
| Normalization forms, root/lemma/stem/morphology, word families, frequency | **Phase 2** |
| Quran graph nodes/edges | **Phase 3** |
| Web reading UI (Phase 1 ships the API + a minimal labelled debug reader only) | **Phase 4** |
| Tafsir, hadith, other scriptures | **Phase 5 / 8** |
| Recitation audio, tajwid, qira'at variants (schema fields reserved only) | **Phase 4 / post-MVP** |
| Agents / LLM / tool sandbox | **Phase 9** |

### 4.3 Deliberate constraints adopted now

1. **The canonical write path has no model dependency.** Mechanized as an architecture test:
   `quran-core` deps ⊆ `{domain, serde, thiserror}` and neither `quran-core` nor
   `quran-corpus` may depend on `llm`, `embeddings`, `retrieval`, or a vector-store crate.
   This makes PRD §40's "canonical lookup must not require an LLM or vector database" a
   compile-time property, not a code-review promise.
2. **The importer cannot activate.** It holds no `ApprovalToken`; activation is a separate,
   human-gated command (§85, I5/I7).
3. **Staging is physically separate.** `quran_stg_*` mirrors share no triggers with canonical
   tables, and nothing but validation + the activation transaction reads staging.
4. **A partially imported edition is invisible.** A crash mid-import leaves `Staged`, never
   `Active`. No reader can observe a half-corpus.
5. **Normalization never touches stored text.** Phases 2+ build indexes over *derived* forms;
   the canonical row is re-read and hash-checked after every index build (QV-028).

---

## 5. Deliverable Index

| ID | Deliverable | Primary crates | ACs |
|---|---|---|---|
| D1.1 | `quran-core` — pure domain: newtypes, enums, structs, reference grammar, `QuranQuotation` | `quran-core` | 12, 13 |
| D1.2 | Edition source format + manifest extension + adapters | `quran-corpus::adapters`, `docs/schemas/` | 04 |
| D1.3 | Canonical importer job (13 checkpoints, resumable, idempotent) | `quran-corpus`, `jobs` | 02, 03, 10, 11 |
| D1.4 | Corpus validator (QV-001…QV-028), Unicode auditor, reference comparator | `quran-corpus::validation` | 04 |
| D1.5 | Persistence migrations `0010`–`0015` + repository read paths | `storage-sqlite`, `migrations/sqlite/` | 07, 08, 09 |
| D1.6 | `QuranReader` deterministic lookup + context + caching + perf gates | `quran-corpus`, `application` | 05, 06, 07 |
| D1.7 | Quran read API v1 + response envelope + ETag/caching | `server`, `api` | 14† |
| D1.8 | Tool result contract + `quran.get_ayah` / `quran.get_context` | `tools`, `tool-registry` | 15† |
| D1.9 | Citation resolver v1 + `QuranQuotation` + deep links + `citations` table | `citations` | 05, 13 |
| D1.10 | CLI `qai quran …` surface | `cli`, `application` | 16† |
| D1.11 | `qai doctor --quran` (19 checks) + `--deep` + `--json` | `cli::doctor` | 17† |
| D1.12 | Minimal labelled debug reader (RTL, web font) — not the Phase-4 UI | `server` | 18† |
| D1.13 | Corpus integrity test suite: golden fixtures, adversarial corpora, property + immutability suites | `fixtures/quran/`, `testkit` | 04, 05, 06, 08, 09, 11, 12, 13 |
| D1.14 | Docs: corpus architecture, import runbook, rollback runbook, citation spec, adapter authoring guide | `docs/` | 20† |

† Supplementary criteria — see `acceptance.md` §1.6. The plan enumerates 13 exit criteria;
these gates cover deliverables the plan describes but does not assign an AC number.

**D1.13 note.** The test suite is a first-class deliverable, not "tests we'll add later". It
is the permanent regression net (PRD §35.1, §46, §58) and is scheduled across Sprints 1.0,
1.1, 1.2, and 1.5 (tasks P1-T05, T09, T14, T29, T30, T45, T50, T56, T57, T58).

**D1.14 note.** The five documents are named in task P1-T59. They are the port of record for
"how the corpus is imported, corrected, rolled back, and cited".

---

## 6. Architecture & Dependency Rule

```text
                    ┌────────────────┐
                    │  quran-core    │  pure domain: types + reference grammar +
                    │  (no I/O)      │  QuranQuotation. deps: domain, serde, thiserror
                    └───────┬────────┘
                            │
                    ┌───────▼─────────────┐
                    │   quran-corpus      │  intermediate format, adapters, tokenizer,
                    │ (parse/validate/hash)│  validator (QV-*), differ, importer logic
                    └───────┬─────────────┘
              ┌─────────────┼───────────────┬─────────────────┐
              │             │               │                 │
      ┌───────▼──────┐ ┌────▼────────┐ ┌────▼─────────┐ ┌─────▼──────────┐
      │  citations   │ │ storage-    │ │ application  │ │  server / api  │
      │ (resolver v1)│ │ sqlite      │ │ (QuranReader │ │ (Quran read v1)│
      └──────────────┘ │ (quran repo)│ │  + services) │ └─────┬──────────┘
                       └─────────────┘ └──────┬───────┘       │
                                            │               │
                                      ┌─────▼───────────────▼─────┐
                                      │           cli             │
                                      │  qai quran …  ·  doctor   │
                                      └───────────────────────────┘
```

Intended dependency edges (to be added to `xtask/allowlist.toml` — currently Phase-0 only,
so new path edges would fail `arch-check` until registered):

```text
quran-core      -> domain                              (serde, thiserror, unicode-* only; NO async, NO sqlx)
quran-corpus    -> quran-core, domain, sources, storage (parser/validator/importer; NO llm/embeddings/retrieval)
citations       -> quran-core, domain
storage-sqlite  -> storage, domain, quran-core         (quran row types + repository)
application     -> quran-core, quran-corpus, citations, storage, storage-sqlite, sources, jobs, …
cli             -> application, config, observability, server
server          -> application, config, observability
```

**Hard rule (I2):** neither `quran-core` nor `quran-corpus` may depend on `llm`,
`embeddings`, `retrieval`, or any vector-store crate. `cargo xtask arch-check` enforces this
once the allowlist is extended; until then, adding those edges is a gate failure waiting to
happen and must not be merged.

`quran-core`, `quran-corpus` and `citations` are implemented. The dependency table above
is the design baseline; actual allowed edges are maintained in `xtask/allowlist.toml`.

---

## 7. Migrations Delivered

The table retains the plan's numbering. Actual files are `0007_quran_editions`,
`0008_quran_structure`, `0009_quran_divisions`, `0010_quran_translations`,
`0011_quran_staging` and `0012_quran_validation` under `migrations/sqlite/`, all with
`.up.sql` suffixes. See `done.md` DEV-02; do not create duplicate `0010`–`0015` migrations.

| Planned file | Contents |
|---|---|
| `0010_quran_editions.up.sql` | `quran_editions` (+ identity/hash immutability trigger), `quran_active_edition` singleton pointer with `corpus_generation` |
| `0011_quran_structure.up.sql` | `quran_surahs`, `quran_ayahs`, `quran_tokens` (+ insert-only triggers `QAI-QUR-0001…0005`), `quran_token_separators`, `quran_segments` |
| `0012_quran_divisions.up.sql` | `quran_divisions` (juz/hizb/rub/manzil/ruku/page/sajdah) + range index |
| `0013_quran_translations.up.sql` | `translation_editions` (`translator` required by CHECK — principle 5), `translation_passages`, `word_glosses` |
| `0014_quran_staging.up.sql` | `quran_stg_*` mirrors, each with `import_run_id`, **no immutability triggers**, `ON DELETE CASCADE` from `import_runs` |
| `0015_quran_validation.up.sql` | `validation_reports`, `difference_reports`, `citations` |

Migrations are **append-only and checksummed** (Phase 0 D0.7). Editing an applied migration
is a hard, coded failure (`QAI-DB-0003`). Canonical tables are **forward-only**: deactivation
is used instead of deletion (§76). Physical removal of a `Removed` edition is an audited,
offline maintenance command documented as a runbook, out of normal operation.

**Note on `quran_active_edition`:** exactly one row (`singleton = 1`), pointing at the active
Arabic edition with the current `corpus_generation`. Activation is a single transaction that
ends in this pointer flip; every cache and derived index records the generation it was built
from, so staleness is detectable rather than assumed (§76).

---

## 8. Data-Source Prerequisite — The Hard Gate (ADR-0101)

Phase 1 **cannot ship** without an approved edition dataset. This is a *legal + editorial*
task, not a coding task, and **must start before Sprint 1.1** (in practice it overlaps Phase 0
Sprint 0.5). It is tracked as swimlane **P1-X01…X03** in `tasks.md` §1, deliberately outside
the sprint line so it cannot be silently dropped.

**ADR-0101 (Initial Quran dataset & license)** must record:

- candidate datasets — provenance, script (Uthmani), riwayah (Hafs ʿan ʿĀṣim), verse-numbering
  scheme, Unicode normalization form, license;
- redistribution rights (PRD §38) — whether the dataset may be *bundled* or must be
  *user-supplied*;
- the **reference corpus** used for independent comparison in validation (§35.1);
- editorial sign-off by a **named** qualified reviewer that the text matches a recognized
  printed muṣḥaf (`verified_by` field on the edition).

**Fallback if no dataset can be bundled.** Q-ai ships the schema, validator, importer, and a
small **public-domain test fixture** (a handful of surahs) for tests, and requires the user to
run `qai quran import <manifest>` with their own approved dataset. This fallback must be
decided **in ADR-0101**, not improvised on the deadline. The engineering work (D1.1–D1.12) is
identical either way; only the shipped data differs.

> ⚠️ Engineering delay is recoverable (add people, extend a sprint). A late **licensing or
> editorial** decision is not: it stalls the whole phase, and a qualified reviewer books out
> weeks ahead. ADR-0101 and ADR-0114 are the two longest-lead items in this phase.

---

## 9. Scheduling Reality Check — Read Before Committing To Dates

Two things in `plan.md` need an owner decision before Phase 1 is scheduled.

### 9.1 The task rows do not reconcile with the stated total

The plan's WBS header says **"Total ≈ 82 ed ⇒ ~6 weeks with 3 engineers."** Summing the
individual task estimates gives **131.0 ed**, a **~49 ed gap** (~1.6×). At a realistic
15 ed/week for a 3-engineer team that is **~8.7 weeks**, not 6.

| Sprint | Scope | Σ task estimates |
|---|---|---|
| 1.0 | Data & Decisions (runs in parallel with Phase 0 Sprint 0.5) | 11.5 ed |
| 1.1 | Domain & Addressing | 19.5 ed |
| 1.2 | Import & Validation | **42.0 ed** ⚠️ |
| 1.3 | Reader, Translations, API | 21.5 ed |
| 1.4 | Tools, Citations, CLI, Doctor | 21.5 ed |
| 1.5 | Debug Reader, Hardening, Exit | 15.0 ed |
| **Total** | **60 tasks + 3 swimlane** | **131.0 ed** |

By role: **BE ≈ 81.0** · **QA ≈ 23.0** · **DOC ≈ 10.0** · **DATA ≈ 9.0** · **EDIT ≈ 6.5** ·
shared/all 1.5.

Pick one before Sprint 1.1 starts:

- **(a)** Accept ~8.5–9 weeks and revise the target duration; or
- **(b)** Hold 6 weeks and cut scope explicitly — the first cut should be D1.12 (debug reader,
  1.5 ed), then D1.7's OpenAPI/contract depth (P1-T40, 2.0 ed), then D1.14 documentation depth
  (P1-T59, 3.0 ed); or
- **(c)** Add a fourth engineer for Sprints 1.1–1.4 (the heavy validation/reader/tool work).

> **Ratified 2026-09-22 (OD-05 + A4, owner series): option (a)** — 131.0 ed /
> ~8.7 weeks with the 1.2a/1.2b split, no fourth engineer in Phase-1, no scope
> cut, sprint cap 24 ed, plus a 1-week contingency buffer after Sprint 1.2b
> (~9.7 weeks with buffer). D1.1/D1.4/D1.5/D1.6/D1.13 protected. Absolute dates
> stamped at the Sprint-1.1 kickoff.

Do **not** resolve this by silently compressing estimates. **D1.1 (`quran-core`), D1.4
(validator), D1.5 (migrations + triggers), D1.6 (`QuranReader`), and D1.13 (integrity suite)
are retrofit-impossible** and must not absorb the cut: once a canonical row can be written
without a token, or text can be served without a hash, the guarantee is gone.

### 9.2 Sprint 1.2 is overloaded

Import & Validation is **42.0 ed**, roughly **3×** a normal 15 ed/week sprint, and it contains
the two highest-risk items in the phase (the importer's 13-checkpoint state machine and the
full QV-001…QV-028 rule set). Recommended split, to be agreed before the sprint opens:

- **1.2a — Format, adapters, tokenizer, hashing** (P1-T15…T18, T21…T24; ≈ 19.0 ed)
- **1.2b — Validator rules, importer, activation, differ** (P1-T19…T20, T25…T31; ≈ 23.0 ed)

The QA tasks (P1-T29, T30) follow 1.2b and may run as a dedicated hardening window.

### 9.3 Cross-phase decisions with external lead times

Tracked as swimlane **P1-X01…X03**. These are **not** on Phase 1's own critical path, which is
exactly why they get deprioritized — and each has a lead time engineering cannot compress.

| Decision | ADR | Needed by | Bottleneck |
|---|---|---|---|
| Quran text dataset selection & licensing | ADR-0101 | **Before Sprint 1.1** | Licensing review, source verification, attribution terms |
| Reference corpus + comparison procedure + sign-off | ADR-0114 | Before P1-T26 (Sprint 1.2) | Editorial process; independent text comparison |
| Morphology dataset selection & licensing (carry from P0) | ADR-0203 | Start of Phase 2 | Dataset-shape evaluation, licensing |
| Normalization rule catalog + linguist engagement (carry from P0) | ADR-0204 | Start of Phase 2 | **Sourcing/booking a qualified Arabic linguist**, not the writing |

**Action:** in the **first standup**, assign an owner and a "decision-open" date to each row,
independent of sprint task assignments and independent of whether that owner has other Phase-1
work.

---

## 10. Quick Start

From the repository root, build the CLI and isolate the synthetic fixture in a fresh
data directory. These commands activate test data, not an approved Quran edition.

```bash
cargo build -p cli --bin qai
export QAI_DATA_DIR="$(mktemp -d)"
./target/debug/qai db migrate
./target/debug/qai quran import fixtures/quran/test-edition-min/manifest.json
./target/debug/qai quran validate test-edition-min@0.1.0
./target/debug/qai quran activate test-edition-min@0.1.0 --yes
./target/debug/qai quran get 1:1 --json
./target/debug/qai quran context 2:1 --before 1 --after 1 --boundary surah
./target/debug/qai doctor --quran --deep
```

Doctor may report unresolved configuration/source checks. See the
[project README](../../../../README.md#development-and-verification) for repository gates.

**Error-code namespace:** `QAI-QUR-nnnn` (reserved in Phase 0 D0.3; first used here).
Reference-grammar errors are `QAI-QUR-01xx`; canonical-table trigger codes are
`QAI-QUR-0001…0005`; edition immutability is `QAI-QUR-0002`. Codes are public API.

**CLI exit codes** are inherited from Phase 0 (D0.13) and must not diverge: `0` ok · `1`
generic · `2` usage · `3` validation failed · `4` denied by policy · `5` not found · `6`
conflict/state · `7` cancelled · `70` internal.

---

## 11. ADRs Required In This Phase

| ADR | Title | Blocking | Notes |
|---|---|---|---|
| ADR-0101 | Initial Quran dataset, script, riwayah, and license | **Everything** | Must include editorial sign-off and the bundle-vs-user-supplied decision |
| ADR-0102 | Quran addressing scheme & reference grammar | D1.1 | Frozen grammar; changing it later breaks every stored citation |
| ADR-0103 | Verse-numbering scheme handling and alternate-numbering strategy | D1.1, D1.5 | How to represent editions with different numbering without merging |
| ADR-0104 | Unicode policy: normalization form, allowed blocks, forbidden code points, grapheme handling | D1.4 | Directly determines whether text is "corrupt" |
| ADR-0105 | Canonical tokenization rule for Phase 1 (whitespace-preserving surface tokenization) | D1.3 | Must be lossless; morphological segmentation is Phase 2 and separate |
| ADR-0106 | Canonical text storage layout (row-per-ayah in SQLite vs. blob+index) | D1.5 | Query flexibility vs. hash simplicity |
| ADR-0107 | Atomic activation & rollback mechanism (staging + pointer flip + generation counter) | D1.3 | §85 compliance |
| ADR-0108 | Corpus hashing scheme: what exactly enters `text_hash` / `structure_hash` / `token_order_hash` | D1.3, D1.4 | Frozen forever; determines reproducibility |
| ADR-0109 | Edition difference algorithm for canonical text | D1.3 | Char-level vs token-level; reviewer usability |
| ADR-0110 | Basmala representation policy | D1.1 | Affects ayah counts and display |
| ADR-0111 | Citation identity and deep-link URL/URN format | D1.9 | Public, stable surface |
| ADR-0112 | Translation alignment and attribution model | D1.4 | Enforcing principle 5 |
| ADR-0113 | Canonical lookup caching & invalidation | D1.6 | Correctness over hit rate |
| ADR-0114 | Reference-corpus comparison procedure and who signs off | D1.4 | Editorial process, not just code |

Every ADR uses the Phase-0 §48 template, including **Accuracy implications**,
**Religious-source implications**, and **Licensing implications** (enforced by the ADR lint
introduced in Phase 0 AC-P0-19). For Phase-1 ADRs the religious-source section is often
substantive — ADR-0101, 0104, 0105, 0110, and 0114 directly determine whether text or
metadata can be corrupted, mis-numbered, or presented as something it is not.

ADR numbering follows the phase-coded scheme adopted in Phase 0 (`ADR-01nn` = Phase 1).
Writing an ADR with a bare `ADR-000n` id is a lint failure.

---

## 12. Exit Gate

Phase 1 is complete when **all criteria in `acceptance.md` pass**: the 13 plan-defined ACs
(AC-P1-01…13), the supporting gates (§1.6), the required corpus-integrity suites (§2), and
the coverage floors (§4). The exit-gate ritual (§6 of `acceptance.md`) must be recorded live
on a clean machine by a reviewer who is not the implementer.

**Handoff artifact:** `docs/plans/handoff-p1-to-p2.md` (task P1-T60), listing every asset
Phase 2 inherits and must not re-invent, plus known limitations and deferred items **with
owners**. In particular, Phase 2 must inherit the canonical token rows, the hash scheme, and
the corpus-generation counter — not re-derive them.

---

## 13. Top Risks

| # | Risk | L | I | Mitigation |
|---|---|---|---|---|
| R1 | **ADR-0101 slips** — no licensed, editorially-approved dataset by Sprint 1.2 | Med | **Critical** | Swimlane X, weekly standup check; ADR-0101 fallback (test fixture + user-supplied import) decided in advance so engineering never stalls |
| R2 | **Silent text corruption** — a dataset that is *almost* right (NFD vs NFC, truncated ayah, homoglyph) enters the corpus | Med | **Critical** | QV-001…QV-028 all run (no fail-fast); 16 adversarial fixtures must each fail with the **specific** rule id (AC-P1-04); reference-corpus comparison is a Fatal gate |
| R3 | **Non-lossless tokenization** — whitespace/waqf marks lost, so tokens can't rebuild the ayah | Med | High | Whitespace-preserving tokenizer + `quran_token_separators`; full-corpus round-trip is Fatal (QV-011) and an exit AC (AC-P1-05) |
| R4 | **Accidental activation** — importer or a bug flips the active pointer | Low | **Critical** | Staging physically separate; importer holds no token; activation is one transaction; crash matrix at all 13 checkpoints (AC-P1-10) |
| R5 | **Hashing scheme changes later**, invalidating every stored hash | Low | High | ADR-0108 freezes the recipe; algorithm tag inside `ContentHash`; rehash-migration procedure documented before first import |
| R6 | **Estimate gap** (§9.1) turns into an unplanned crunch that cuts D1.4/D1.13 | High | High | Owner decision before Sprint 1.1; cut D1.12/T40/T59 depth first, never the integrity work |
| R7 | **Sprint 1.2 overload** (§9.2) delays the importer and validator together | High | Med | Split into 1.2a/1.2b; QA rejection suites scheduled as a dedicated window |
| R8 | **Reference grammar frozen wrong** (ADR-0102) — stored citations can't round-trip | Low | High | 300-case golden set + `parse(serialize(ref)) == ref` property test (AC-P1-12/13) before the grammar is declared frozen |
| R9 | **Translation attribution bypassed** — a translation presented as the Quran | Low | High | `translation_editions.translator` NOT NULL + non-empty CHECK; `AyahView` has no variant putting a translation in `canonical`; QV-027 |
| R10 | **Caching serves stale text** after an activation | Med | High | Cache keyed by `corpus_generation`; wholesale invalidation on generation change; explicit cache-consistency test (D1.6) |

Full risk context: `plan.md` §1 (invariants), §2.3 (data prerequisite), §9 (acceptance).
