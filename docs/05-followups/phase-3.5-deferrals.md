# Phase 3.5 (Residual Closure) — Deferral Ledger

> **Purpose.** Records, explicitly, everything Phase 3.5 deliberately did **not**
> do, with the rationale and the target phase. Items here are **deferred, not
> dropped**. It mirrors the ledger shape of
> `docs/05-followups/phase-03-deferrals.md` and
> `docs/05-followups/phase-04-deferrals.md`.
>
> **Scope note.** This ledger records *scope* deferrals. The owner gates
> (OD-11 dataset/license, OD-12 linguist, and the rest of OD-01…OD-14) are
> **blocks**, not deferrals, and are recorded separately in
> `docs/05-followups/decisions-needed.md`, which remains the source of truth
> for those. No row in that file is closed by Phase 3.5 (D-3.5-04).
>
> **Rule for agents:** adding an item here is the honest alternative to silently
> omitting it. Never move an item off this list except by implementing it under a
> plan that names it.

---

## 1. Full-corpus soak and full-corpus budget rows → Phase 3.5 exit (unblocked by OD-11)

| Deferred item | Rationale | Target |
|---|---|---|
| **Full-corpus 50k soak** (P2-T111 second half) | The 50,000-query soak in `crates/application/tests/soak_full.rs` runs against the 14-ayah synthetic fixture (`scale: synthetic_fixture`). The same harness is parameterized to accept a real edition, but no licensed corpus is active. | Phase 3.5 exit — execute once OD-11 closes |
| **`applies_to: full-corpus` budget rows** (`search.*.p50/p99`, `normalize.explain.*`, `cold_rebuild_total_ms`) | Recorded in `fixtures/quran/performance/budgets.json` with rationale per row; never executed against the fixture. The fixture harness gates only against `fixture_bound_ms`. | Phase 3.5 exit — measure once OD-11 closes |
| **Full-corpus concatenated p99 ≤ 150 ms** (ADR-0207) | The ADR-0207 target is codified, not measured: nothing at fixture scale can stand in for a full-corpus p99. | OD-11, then a measurement plan |

## 2. Linguist sign-off on every golden set → OD-12

| Deferred item | Rationale | Target |
|---|---|---|
| **Root/lemma golden review** (500 cases) | Every row stays `reviewed_by: pending-linguist` / `reviewed_at: null` / `synthetic_test_only`; root conventions await ADR-0210/ADR-0215 acceptance. | OD-12 (named linguist) |
| **Family golden review** (154 curated families) | Same labeling; relation-kind semantics await the same acceptance. | OD-12 (named linguist) |
| **Evaluation-lexicon review** (`lexicon-v1.jsonl`) | Synthetic rows from the deterministic test scheme; pending the same review. | OD-12 (named linguist) |

## 3. Linguistic quality gates in the evaluation harness → OD-12 (ADRs Draft)

| Deferred item | Rationale | Target |
|---|---|---|
| **Any linguistic quality gate** | The harness (`crates/evaluation`) gates mechanical regressions only — empty results where goldens exist, missing attribution, determinism failures, unresolvable dataset versions. It asserts nothing about linguistic correctness and labels its own output as not a linguistic verdict, because ADR-0204, ADR-0209, and ADR-0211 are Draft. | OD-12 + ADR acceptance |

## 4. Transliteration surface and full-quality L8 fuzzy policy → later phases

| Deferred item | Rationale | Target |
|---|---|---|
| **Transliteration search surface** (ADR-0206) | Reserved/deferred since Phase 3; not part of SC1–SC5 or the 3.5 workstreams. | later phase |
| **Full-quality L8 fuzzy policy** (ADR-0216) | Heuristic L7/L8 matches stay labeled when they occur; the full-quality policy is deferred since Phase 3. | later phase |

## 5. Counting tail outside the SC-aligned core → OD-12

| Deferred item | Rationale | Target |
|---|---|---|
| **Counting-rule ratification** (ADR-0211) | The counting tail is implemented to the same bar as the SC-aligned core (typed errors, `CountingRules` on every numeric report, dataset attribution, CLI + HTTP parity), but ratifying the *rules* stays owner-gated. | OD-12 (named linguist + ADR-0211 acceptance) |
| **`unusual_usage` capability** | Implemented as lexicon-gated: it returns typed `UnavailableDataset` until a lexicon capability exists. Mining itself is not built. | later phase, after OD-11/OD-12 |

## 6. Everything Phases 3 and 4 already deferred → unchanged

| Deferred item | Rationale | Target |
|---|---|---|
| **Phase-03 deferrals §1–7** (UI, result-contract checksum, counting tail, legacy phase-02-rag tail, transliteration/L8, graph, editions/multi-RAG) | Out of scope for Phase 3.5 per 03.5-CONTEXT; none implemented here. | their recorded phases |
| **Phase-04 deferrals §1–6** (tafsir/hadith/isnad, GUI/TUI, adapter spikes, GraphML, production auth, additional editions) | Same boundary; none implemented here. | their recorded phases |

---

*Phase: 3.5-Residual Closure · Deferral ledger created 2026-10-02 by plan
03.5-03. Append-only; nothing here is silently dropped.*
