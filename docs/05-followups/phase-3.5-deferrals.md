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

## 7. Phase-1 hardening residue (W9): WR-01…WR-13 + IN-01…IN-09 dispositions → Phase 12

Per D-3.5-10, the `01-VERIFICATION.md` review findings were split into **fixed
here** (real precision or fail-closed defects) and **deferred** (genuine future
capability, where a partial implementation would be a claim this phase cannot
support). Every identifier the verification listed is accounted for below; none
was silently dropped. The capability-scope items target **Phase 12 (Production
Hardening)**.

| Item | Finding | Disposition (plan 03.5-04) | Target |
|---|---|---|---|
| **WR-01** | Worker-host liveness under injected faults | Deferred — hardening scope | Phase 12 |
| **WR-02** | Shutdown deadline enforcement | Deferred — hardening scope | Phase 12 |
| **WR-03** | Reschedule owner guard | Deferred — hardening scope | Phase 12 |
| **WR-04** | Dead-letter boolean precision | Deferred — hardening scope | Phase 12 |
| **WR-05** | Gap report named the survivor, not the missing rows | **Fixed** — names the missing sequence span; a pure deletion no longer taints survivors | — |
| **WR-06** | Doctor conflated symlink / non-directory cases | **Fixed** — symlink disclosed separately from target type | — |
| **WR-07** | Doctor/readiness conflated permission failure with missing path | **Fixed** — only `NotFound` names `qai db migrate`; other IO errors surface as themselves | — |
| **WR-08** | Audit-verify JSON envelope shape differed from human fields | **Fixed** — one stable envelope for every outcome; inapplicable fields are `null`, never absent | — |
| **WR-09** | In-memory queue dropped result/progress vs SQLite | **Fixed** at the doc boundary — typed persistence contract on the trait + backend; in-memory is lossy by contract (test-only) | — |
| **WR-10** | `record_approval` defaulted a non-principal actor (fail-open) | **Fixed** — unparseable `decided_by` is a typed error before any staging | — |
| **WR-11** | Doctor read/permission error conflation | **Fixed** — `Unreadable{kind}` reported with its own remedy | — |
| **WR-12** | Record checker matched tokens as substrings (`C10` satisfied `C1`) | **Fixed** — whole-word fixed-string matching | — |
| **WR-13** | Preservation checker accepted a base-line deletion on a clean merge | **Fixed** — base-line retention required when ours == base | — |
| **IN-01** | Read-only verifier holds a write-flavored UoW (`db.write()` on a read-only open) | Deferred — needs an explicit read-snapshot accessor or a documented reason the write flavor is required | Phase 12 |
| **IN-02** | Success results/dispositions stored under the `error_json` column (no result column in schema) | Deferred — needs a `result_json` schema migration plus envelope-key aliasing | Phase 12 |
| **IN-03** | `ChainVerificationFailed { sequence: 0 }` sentinel leaks a never-real sequence into operator output | Deferred — needs `Option<u64>` plumbing through the diagnosis rendering | Phase 12 |
| **IN-04** | Audit-composition enforcement is documentary (`AuditedMutation` bypassable by direct callers) | Deferred — needs a minter-token gate or a softened claim plus grep CI check | Phase 12 |
| **IN-05** | External-policy gate classifies `sparse+https` sources as `Unknown` (forward-compat brittleness) | Deferred — needs `sparse+` accepted as `Registry` with a live-metadata proof | Phase 12 |
| **IN-06** | Record checker over-permissive frontmatter / bare-`## ` section / index counting | **Fixed** — delimiter-validated frontmatter, Phase-1-anchored section, entry-count index check | — |
| **IN-07** | Preservation checker path-encoding collisions, untracked diff noise, tree-wide whitespace gate | **Fixed** — injective `%`-encoding, tracked-only diffs, owned-path whitespace scope | — |
| **IN-08, IN-09** | Documented design warts, explicitly-deferred scope | Deferred — as the verification characterized them | Phase 12 / their owning phases |

**Manual gate (not deferred, not automated here).** The observability
live-endpoint smoke (`otlp.rs::builds_a_provider_for_a_valid_endpoint`) stays
`#[ignore]`d: it needs a live OTLP collector. Un-ignoring a test that cannot run
in CI would turn a manual gate into a false pass; the exporter-boundary scrubbing
it guards is asserted instead by the per-field scrubbing tests.

**WR-09 boundary note.** Aligning the in-memory backend would mean building a
second persistence path; SQLite is the production authority and the in-memory
backend is test-only, so the divergence is documented at the type level and
pinned by a contract test.

---

*Phase: 3.5-Residual Closure · Deferral ledger created 2026-10-02 by plan
03.5-03. §7 (WR/IN dispositions) appended 2026-10-02 by plan 03.5-04.
Append-only; nothing here is silently dropped.*
