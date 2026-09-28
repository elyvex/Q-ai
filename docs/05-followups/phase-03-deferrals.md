# Phase 3 (Quran Search & Linguistics) — Deferral Ledger

> **Purpose.** Records, explicitly, everything Phase 3 deliberately did **not**
> do, with the rationale and the target phase. Items here are **deferred, not
> dropped** (D-02 / D-14). It uses the append-only deferral-ledger convention of
> `docs/03-plan/phases/phase-02-rag/done.md`.
>
> **Scope note.** This ledger records *scope* deferrals. The two owner gates
> (OD-11 dataset/license, OD-12/D-08 linguist) are **blocks**, not deferrals,
> and are recorded separately in
> `docs/05-followups/phase-03-owner-gates.md`. `decisions-needed.md` remains the
> source of truth for both.
>
> **Rule for agents:** adding an item here is the honest alternative to silently
> omitting it. Never move an item off this list except by implementing it under a
> plan that names it.

---

## 1. User-facing UI and the result-contract research checksum → Phase 5 (D-12)

| Deferred item | Rationale | Target |
|---|---|---|
| Interactive **TUI word inspector** | Phase 3's alpha is researcher/developer-facing (CLI + versioned HTTP API only, D-10); the interactive inspector is the roadmap Phase 5 experience. | Phase 5 |
| **Web GUI** | Same boundary — no GUI ships in Phase 3. | Phase 5 |
| **Word-inspector UI** | Follows from the TUI/Web inspector decision. | Phase 5 |
| `REQ-quran-result-contract` **research checksum** | The result contract is enforced at the CLI/API level in Phase 3 (I9/I10 trace + span on every hit); the *research checksum* surface belongs to the Phase-5 research tooling. | Phase 5 |

## 2. Counting / discovery tail beyond the SC-aligned core set → later phases (D-14)

The SC-aligned core set is exactly: exact / normalized / phrase / concatenated /
regex search; root + lemma search; morphology token inspection with
multi-analysis; word family; frequency + distribution + co-occurrence. Every
tool below is **outside** that set and is deferred to a later phase. Several are
already implemented in the codebase (`application::quran_counting`) but are not
part of the Phase-3 success-criteria surface, and none is ratified.

| Deferred item | Note | Target |
|---|---|---|
| `collocation` | Implemented; outside the SC-aligned core set. | later phase |
| `interval_analysis` | Implemented; outside the SC-aligned core set. | later phase |
| `first_last_occurrence` | Implemented; outside the SC-aligned core set. | later phase |
| `unusual_usage` (lexicon-gated) | Implemented; outside the SC-aligned core set. | later phase |
| `hapax_search` | Implemented; outside the SC-aligned core set. | later phase |
| `near_duplicate_passages` | Implemented; outside the SC-aligned core set. | later phase |
| `missing_expected_form` | Implemented; outside the SC-aligned core set. | later phase |
| numeric-report tool | Implemented; outside the SC-aligned core set. | later phase |

## 3. Legacy `phase-02-rag` tail → follow-ups (D-02)

The legacy board `docs/03-plan/phases/phase-02-rag/` is authoritative **evidence
and reference**, not a mandate to close every legacy task (D-04). The following
legacy tail is deferred to follow-ups, recorded here rather than silently
dropped:

| Deferred item | Rationale | Target |
|---|---|---|
| **Discovery depth** (remaining discovery tasks beyond the SC core) | Not required by Phase-3 success criteria; several counting tools are already implemented but unratified. | follow-ups |
| **Documentation depth** on the legacy board | The Phase-3 exit evidence is the `<verify>` blocks in the 03-01…03-08 PLANs, not legacy task-row closure. | follow-ups |
| **Eval-harness polish** (legacy evaluation harness) | Outside Phase-3's exit evidence; D-02 keeps the high-value hardening (goldens, doctor soak, performance budgets) and defers the polish. | follow-ups |

## 4. Transliteration and fuzzy quality → later phases

| Deferred item | Rationale | Target |
|---|---|---|
| Transliteration search (**ADR-0206**) | Reserved/deferred surface; not part of SC1–SC5. | later phase |
| Full-quality fuzzy **L8** (ADR-0216) | Heuristic L7/L8 matches are labeled when they occur; full-quality L8 policy is deferred. | later phase |

## 5. Graph nodes/edges → Phase 4

| Deferred item | Rationale | Target |
|---|---|---|
| Graph nodes/edges built from roots/lemmas | Roadmap Phase 4 owns the graph store, traversal, and annotations. | Phase 4 |

## 6. Additional editions / qira'at / multi-RAG / comparative scripture → their phases

| Deferred item | Rationale | Target |
|---|---|---|
| Additional Quran **qira'at / editions** | Multi-edition expansion is a later roadmap phase; Phase 3 is edition-relative but single-fixture. | later phases |
| **Multi-RAG / comparative-scripture** surfaces | Roadmap Phase 8/9. | their roadmap phases |

## 7. Validation-contract template state (recorded, not silently ignored)

`docs/05-followups/phase-03-deferrals.md` (this file) records that
`.planning/phases/03-quran-search-linguistics/03-VALIDATION.md` ships as a
**seeded template** (`status: draft`, `nyquist_compliant: false`). The
**authoritative per-task verification map for Phase 3 is the `<verify>` blocks in
`03-01…03-08-PLAN.md`** (each plan's tasks carry runnable automated checks, and
the phase-closing plan 03-08 runs the full-chain alpha plus the CLI surface).
The `03-VALIDATION.md` per-task map (sampling rate, per-task rows, manual-only
verifications) is **deferred to `/gsd-validate-phase`**, recorded here rather
than silently ignored.

---

*Phase: 3-Quran Search & Linguistics · Deferral ledger created 2026-09-28 by plan
03-08. Append-only; nothing here is silently dropped.*
