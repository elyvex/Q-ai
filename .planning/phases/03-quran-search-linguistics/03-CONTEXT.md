# Phase 3: Quran Search & Linguistics - Context

**Gathered:** 2026-09-26
**Status:** Ready for planning

<domain>
## Phase Boundary

Turn the canonical corpus (roadmap Phase 2, complete) into a **searchable, linguistically explorable research engine** — without ever altering displayed canonical text. In scope: Arabic normalization (derived only) and user-controlled profiles; exact / normalized / phrase / **concatenated** search; root + lemma search; multi-analysis morphology inspection; word family; and frequency / distribution / co-occurrence — all reachable through the existing `qai` CLI and the versioned HTTP API with an explainability payload (rules applied, segmentation, canonical offsets).

This phase is **evidence-driven brownfield hardening + gap closure over the legacy board `docs/03-plan/phases/phase-02-rag/`** (note the numbering drift: that board is titled "Phase 2" but covers search/linguistics = roadmap Phase 3; currently ~58/114 tasks, 0/50 ACs, 0/14 ADRs). It preserves the substantial working code already present (`quran-normalization`, `quran-search`, `quran-morphology` types, CLI/HTTP search wiring) and creates work only for missing behavior or weak evidence.

**Out of scope:** Web GUI, TUI, word-inspector UI, and the `REQ-quran-result-contract` research checksum (roadmap Phase 5); graph nodes/edges (Phase 4); the counting/discovery tail beyond frequency/distribution/co-occurrence; transliteration and fuzzy L8 quality; any change to canonical rows, canonical tokenization, reference grammar, or the frozen v1 hash recipes.

**Boundary contract:** `.planning/ROADMAP.md` §Phase 3 (five success criteria) plus `REQ-quran-normalization` and `REQ-quran-linguistics`. The legacy board and its ADRs/specs are authoritative **evidence/reference**, not a mandate to close every legacy task.

</domain>

<decisions>
## Implementation Decisions

### Scope, Finish Line & Evidence
- **D-01:** Roadmap success criteria govern the boundary. Phase 3 completes when each of the five success criteria is satisfied by existing code plus at least one repeatable check, and both phase requirements are covered. Implement only the gaps those criteria need. — **Reversibility:** reversible — a planning-boundary choice.
- **D-02:** High-value legacy hardening is **in scope as exit evidence**: `qai doctor --indexes` drift checks, search/morphology golden sets, and explicit performance budgets. The legacy tail (discovery depth, documentation depth, eval-harness polish) is deferred to follow-ups, recorded explicitly rather than silently dropped. — **Reversibility:** reversible.
- **D-03:** Added **alpha-release bar** (owner directive): after Phase 3, Phases 1–3 together must form a coherent, runnable alpha — search/linguistics features reachable at real user surfaces, not service-only stubs. Scope stays confined to Phase 3's own surface; Phases 1–2 are not re-planned. — **Reversibility:** reversible.
- **D-04:** Preserve the legacy brownfield posture (mirrors Phase 1 D-01 / Phase 2 D-01): map each success criterion to current code + a repeatable check, preserve working implementations, and treat the legacy `phase-02-rag` board as authoritative evidence. — **Reversibility:** reversible.

### Morphology Dataset, Licensing & Linguist Gates
- **D-05:** **Verify-then-bundle.** Engineering builds the morphology importer, tokenization alignment, machine-readable source/license matrix, and the activation-rejection gate now. **Quranic Arabic Corpus (QAC)** is the documented **default provider** for morphological tags, lemmas, roots, and conventions. QAC is bundled into the repository **only if** its redistribution terms verify as compatible with the project's data-licensing posture; otherwise ship ADR-0203 **Option B** (user-supplied import via `qai quran morphology import`, with typed "no dataset active" errors — never guessed data). — **Reversibility:** costly — the license/attribution and alignment contract shape the morphology schema and activation gate; once morphology records are active, swapping provider requires new records + validation + a derived-index rebuild.
- **D-06:** **`fawazahmed0/quran-api` is an additional reference/validation source**, used locally (no new download) at `/Users/ali/dev/rust/Q-ai-side/references/01/quran-api/` (git HEAD `47ca096b0976443ba2eab2e45cdf0fb4096a2610`, dated 2026-09-12). Inspect `database/linebyline/` (490+ edition files), `database/chapterverse/`, `database/originals/`, and `editions.json` for Quran text, verse-level alignment, edition/cross-reference metadata, and validation. — **Reversibility:** reversible — a read-only reference source.
- **D-07:** **License discipline.** The quran-api repository's own license (Unlicense / public domain) is **separate** from the licenses of each bundled edition and translation it contains — the repo license does not blanket-cover every dataset. Record the exact upstream source, revision/version, dataset provenance, attribution string, and redistribution/license status in **ADR-0203** and the **machine-readable license matrix**. The activation gate **rejects** activation when required license/attribution evidence is absent (`metadata_only` / `pending_license_review`, never bundling and never guessed data). — **Reversibility:** costly — the license/attribution metadata and activation gate are consumed by provenance, derived-index manifests, and later export/redistribution paths.
- **D-08:** **Linguist sign-off remains a required gate** for ratifying the morphology tagset, root conventions, and alignment rules. If a qualified Arabic linguist is unavailable, record it **explicitly BLOCKED** (naming the missing capability) — never silently treat those conventions as finalized. Consistent with Phase 2 D-03's recorded-owner-gate posture. — **Reversibility:** reversible — a recorded gate can be closed later.
- **D-09:** Preserve the legacy morphology invariants unchanged: **I11** — zero/one/many analyses coexist with **no `is_correct` column**; preference is a per-request policy recorded in output. **I12** — machine-derived analyses are Layer D with algorithm/version/confidence/`verification_status`; **no LLM may generate and store roots/lemmas as dataset-supplied**. **I13** — family relations are typed (`root`/`lemma`/`stem`/`form`/`computational`/`verified`) with provenance. **Alignment (R3)** — a disagreeing tokenization is bridged by an explicit, auditable, hashed alignment table keyed to the exact source edition/version; canonical text is **never re-tokenized**. — **Reversibility:** one-way — these invariants are the correctness foundation of every downstream root/lemma/family/graph result; weakening them invalidates prior analyses and the attribution contract.

### User-Facing Surface Boundary
- **D-10:** Satisfy SC1–SC4 at the **CLI + versioned HTTP API** level only; the alpha is researcher/developer-facing. CLI: `qai quran search/root/lemma/morphology/family/freq`. API: `/api/v1/quran/search/{exact,normalized,phrase,concatenated,regex}` plus `/normalization/{preview,profiles}`. — **Reversibility:** reversible — the UI is deferred, not removed.
- **D-11:** Every search/linguistics result carries the **explainability payload** — the ordered normalization rule set applied, segmentation, and canonical character offsets (**I9**/**I10**). This is a result contract downstream agents and later UI consume. — **Reversibility:** costly — removing explainability from results reverses the published "every match reports why it matched" guarantee and breaks highlight/citation fidelity.
- **D-12:** Defer the interactive TUI word inspector, Web GUI, word-inspector UI, and the `REQ-quran-result-contract` research checksum to **Phase 5**, consistent with the existing legacy scope fence and roadmap phase split. — **Reversibility:** reversible.

### Linguistics Tool Depth
- **D-13:** Implement the **SC-aligned core set only**: exact / normalized / phrase / concatenated / regex search; root + lemma search; morphology token inspection with multi-analysis; word family; frequency + distribution + co-occurrence. Every tool returns provenance/attribution, and every numeric output carries **`CountingRules`** (**I15** — state the counting rules; never assert numerological significance). — **Reversibility:** costly for the tool/numeric-result contract (published output schema consumed by later surfaces); reversible for the deferral itself.
- **D-14:** Defer collocation, interval, first/last-occurrence, `unusual_usage`, `hapax_search`, `near_duplicate_passages`, `missing_expected_form`, and the numeric-report tool to later phases. Record the deferral explicitly; do not silently drop them from the legacy board. — **Reversibility:** reversible.
- **D-15:** Derive all search/linguistics data read-only from canonical rows into **derived** tables/indexes; canonical bytes are never mutated. Preserve the frozen `qai-text-hash-v1` / `structure_hash` / `token_order_hash` recipes (Phase 2 D-12); additive, domain-separated recipes only. Re-assert **I8** (normalization never mutates canonical text) with a post-build canonical-unchanged check. — **Reversibility:** one-way — changing a frozen recipe invalidates every stored hash and requires a global re-hash migration.

### the agent's Discretion
- Exact normalization rule ordering, profile ladder versions, SpanMap storage format, index manifest/hash function, checkpoint payload schemas, CLI flag naming, golden-set composition, and operator-facing wording may follow existing project conventions (Phase 1 D-16 convention) so long as the locked behavior and invariants above are preserved.
- The precise QAC revision/URL, adapter shape, and tag-mapping details are for research to confirm; the owner ratifies the license/attribution entry.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Contract
- `.planning/ROADMAP.md` §Phase 3 — goal, five success criteria, dependency on Phase 2, and phase boundary.
- `.planning/REQUIREMENTS.md` — `REQ-quran-normalization` (PRD §8), `REQ-quran-linguistics` (PRD §9); note `REQ-quran-research-tools` / `REQ-quran-result-contract` / `REQ-tui-cli` belong to Phase 5.
- `.planning/PROJECT.md` — 25 locked ADRs, constraints, out-of-scope, local-first direction; proposed ADRs incl. ADR-0203/0204/0205/0210/0211/0215.
- `.planning/STATE.md` — current position, blockers (OD-11, OD-12), and the Phase-2→3 handoff context.
- `docs/01-requirements/requirements.md` — authoritative living PRD: §8 normalization/search, §9 linguistics/morphology/families, §11.1–§11.3 tools, §12 result fields, §46 completeness rule.

### Legacy Phase Board (authoritative evidence — numbering drift: "Phase 2" = roadmap Phase 3)
- `docs/03-plan/phases/phase-02-rag/README.md` — invariants I8–I16, scope fence, deliverable index, ADR list, top risks.
- `docs/03-plan/phases/phase-02-rag/plan.md` — full plan (source of truth for the legacy scope).
- `docs/03-plan/phases/phase-02-rag/tasks.md` — live task board P2-T01…T114 + swimlane X.
- `docs/03-plan/phases/phase-02-rag/acceptance.md` — AC-P2-01…50 exit gate and required suites.
- `docs/03-plan/phases/phase-02-rag/done.md` — append-only completion ledger (deviations/deferrals).
- `docs/03-plan/phases/phase-02-rag/remaining-engineering-plan.md` — remaining waves, licensing boundary, P0 contract repairs, wave DoD.

### Locked / Core Architecture Decisions
- `docs/02-architecture/decisions/ADR-0104-unicode-policy.md` — NFC storage, forbidden code points, grapheme counts with byte offsets.
- `docs/02-architecture/decisions/ADR-0105-canonical-tokenization.md` — whitespace-preserving surface tokens; morphology alignment target.
- `docs/02-architecture/decisions/ADR-0106-canonical-storage-layout.md` — canonical vs staging vs derived table layout.
- `docs/02-architecture/decisions/ADR-0108-corpus-hashing-scheme.md` — frozen `qai-text-hash-v1` recipes and `sha256:<hex>` form.
- `docs/02-architecture/decisions/ADR-0111-citation-identity-and-deep-links.md` — citation identity/URNs and `verify_quotation` verdicts (search hits link to pinned canonical refs).
- `docs/02-architecture/decisions/ADR-0012-workspace-boundaries.md` — machine-enforced crate dependency boundaries.

### Phase-3 Architecture Decisions
- `docs/02-architecture/decisions/ADR-0201-full-text-engine.md` — FTS engine choice (SQLite FTS5 implemented; Tantivy was the original target).
- `docs/02-architecture/decisions/ADR-0203-quran-morphology-dataset.md` — **Draft**; morphology provider, alignment guarantee, license rule, Option B fallback (D-05/D-07/D-08).
- `docs/02-architecture/decisions/ADR-0204-normalization-rules.md` — rule catalog N01–N24, mapping tables, ordering (OD-12).
- `docs/02-architecture/decisions/ADR-0205-profile-ladder.md` — profile ladder L0–L8, versioning, immutability.
- `docs/02-architecture/decisions/ADR-0207-concatenated-search.md` — skeleton + trigram candidates + verification (SC2).
- `docs/02-architecture/decisions/ADR-0208-span-map.md` — offset-mapping representation and storage (I10).
- `docs/02-architecture/decisions/ADR-0209-multi-analysis.md` — multi-analysis representation, no authoritative flag (I11).
- `docs/02-architecture/decisions/ADR-0210-root-convention.md` — root normalization and cross-dataset unification as reviewable suggestions (I13).
- `docs/02-architecture/decisions/ADR-0211-counting-rules.md` — counting rules, multi-analysis counting semantics, numeric-report policy (I15).
- `docs/02-architecture/decisions/ADR-0213-index-generations.md` — generation stamping, atomic activation, retention, drift (I14).
- `docs/02-architecture/decisions/ADR-0214-search-cache.md` — result caching and invalidation keying.
- `docs/02-architecture/decisions/ADR-0215-tagset.md` — unified morphological tagset and native-tag mapping.
- `docs/02-architecture/decisions/ADR-0206-transliteration.md`, `ADR-0212-regex-limits.md`, `ADR-0216-fuzzy-policy.md` — reserved/deferred surfaces still relevant to search bounds.

### Open / Owner-Gated Decisions
- `docs/05-followups/decisions-needed.md` — **OD-11** (morphology dataset/license) and **OD-12** (normalization rule catalog + linguist), both human-only.
- `docs/05-followups/owner-decisions.md` — owner's recorded architectural intent.
- `docs/02-architecture/upstream-sources.md` — verified upstream facts/revisions.

### Data Sources (D-05/D-06/D-07)
- **Quranic Arabic Corpus (QAC)** — primary morphology reference; exact revision/URL to be recorded in ADR-0203.
- `/Users/ali/dev/rust/Q-ai-side/references/01/quran-api/` — local external reference repo (git HEAD `47ca096b0976443ba2eab2e45cdf0fb4096a2610`, 2026-09-12; `LICENSE` = Unlicense). Read `database/linebyline/`, `database/chapterverse/`, `database/originals/`, `editions.json`; treat per-edition licenses separately from the repo license.

### Specs & Existing Reconciliation (evidence)
- `specs/007-quran-normalization/`, `specs/008-quran-search/`, `specs/020-morphology-import-alignment/`, `specs/030-search-service-wiring/` — normalization, search, morphology import/alignment, and search-service wiring specs.
- `docs/07-technical/quran-citation-spec.md` — citation identity and resolution (search-hit linking).
- `docs/06-progress/task-done-rollup.md` — reconciliation record.

### Codebase Maps & Fixtures
- `.planning/codebase/ARCHITECTURE.md`, `.planning/codebase/STACK.md`, `.planning/codebase/INTEGRATIONS.md`, `.planning/codebase/CONVENTIONS.md`, `.planning/codebase/TESTING.md`.
- `fixtures/quran/lexicon/` — synthetic, labeled test lexicon (`synthetic_test_only`); `fixtures/quran/test-edition-min/`, `fixtures/quran/adversarial/`, `fixtures/quran/golden/`.

### Prior Phase
- `.planning/phases/02-canonical-quran-core/02-CONTEXT.md` — Phase 2 decisions this phase inherits (D-05/D-06/D-12, owner-gate posture).
- `.planning/phases/01-foundations/01-CONTEXT.md` — Phase 1 brownfield gap-closure pattern.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/quran-normalization` — rules **N01–N22**, profiles **L0–L8**, `NormalizationPipeline`, bidirectional `SpanMap`, `NormalizationTrace`; pure logic, no I/O (I9/I10).
- `crates/quran-search` — `FullTextIndex` port + **FTS5** backend, `ar_*` tokenizers, `skeleton.rs`, `trigram.rs`, `regex.rs` (DFA-only, I16), `highlight.rs`, `hit.rs`, `model.rs`, search cache.
- `crates/quran-morphology` — pure types + serde: `adapter_csv`/`adapter_json`, `align`, `validate`, `policy`, `compare`, `family`, `tagset`, `dataset`; bundles **no dataset**, performs no I/O (ADR-0203 Option B shape).
- `crates/application` — services `quran_forms.rs`, `quran_index.rs`, `quran_normalize.rs`, `quran_search.rs` (+ `search_exact/normalized/phrase/concatenated/regex`, `segment_concatenated`, `search_cache`), `quran_morphology.rs`, `quran_counting.rs`.
- `crates/cli` — `qai quran {normalize, forms, index, search, root, morphology}` with `QuranSearchArgs` and search subcommands `Root`/`Lemma`/`RootFamily`.
- `crates/server` — `/api/v1/quran/search/{exact,normalized,phrase,concatenated,regex}` (POST) and `/normalization/{preview,profiles}`.
- Migrations `0013_quran_normalization` … `0018_morphology_staging` (incl. `0017_quran_lexicon`); SQLite FTS5 per-generation index dirs.

### Established Patterns
- Derived/search layer is **read-only over canonical**; derived tables/indexes are generation-stamped and rebuildable (I8/I14).
- One shared `NormalizationPipeline` instance for query and index paths; the 5,000-substring parity test is a hard gate (R6).
- Typed `Diagnostic` errors with unique `QAI-*` codes mapped to stable CLI exit codes; `QAI-NORM-*` (normalization) and `QAI-IDX-*` (index drift, e.g. `QAI-IDX-0101` stale warning).
- Canonical writes are approval-gated; the importer and required-token fences from Phase 2 (migration `0021_canonical_write_fence`) remain in force.
- Purity fence: `quran-normalization` / `quran-search` / `quran-morphology` must never gain `llm`/`embeddings`/`retrieval`/vector deps (`xtask/allowlist.toml`).

### Integration Points
- `crates/application/src/quran_search.rs` (+ `quran_search_cache.rs`) — query services to complete/harden.
- `crates/application/src/quran_morphology.rs` — morphology import/activation wiring (importer + activation/reindex on approval).
- `crates/application/src/quran_counting.rs` — frequency/distribution/co-occurrence with `CountingRules`.
- `crates/application/src/quran_tools.rs` + `crates/tool-registry/src/lib.rs` — tool surface where root/lemma/morphology/family must be registered with attribution.
- `crates/cli/src/quran.rs` — CLI dispatch to complete/harden.
- `crates/server/src/api.rs` — search/normalization routes; `crates/cli/src/doctor.rs` + `crates/application/src/quran_doctor.rs` — `doctor --indexes` drift checks.
- `migrations/sqlite/0013…0018` (+ forward-only additions) — derived normalization/forms/lexicon/index/cache/staging tables.

</code_context>

<specifics>
## Specific Ideas

- Begin planning with an **evidence matrix** (criterion → existing implementation → repeatable evidence → gap → planned task if needed), mirroring Phase 1/2.
- Owner gates (**OD-11** dataset/license, **OD-12** linguist) must appear in every relevant plan as explicit **BLOCKED** items with the exact command/step needed to close them — never a silent pass.
- Enforce invariants as **tests, not comments**: canonical-unchanged after every build (I8), SpanMap round-trip properties (I10), no `is_correct` column / multi-analysis policy (I11), typed family relations (I13), `CountingRules` present on every numeric output (I15), regex/pattern resource bounds (I16).
- Concatenated search (SC2) must produce an **explainable segmentation** and link each hit to a pinned canonical reference.
- The alpha must be runnable end-to-end from CLI/API on the synthetic fixture: normalize → forms rebuild → index rebuild → search (all five modes) → root/lemma → morphology token → family → frequency/distribution/co-occurrence.
- Label **L7/L8 heuristic** matches explicitly (e.g. "matched using heuristic affix stripping"); synthetic data is labeled synthetic, never scholarly ground truth.

</specifics>

<deferred>
## Deferred Ideas

- Interactive **TUI word inspector**, **Web GUI**, word-inspector UI, and `REQ-quran-result-contract` research checksum → roadmap Phase 5.
- Collocation, interval, first/last-occurrence, `unusual_usage`, `hapax_search`, `near_duplicate_passages`, `missing_expected_form`, and the numeric-report tool → later phases.
- Legacy `phase-02-rag` tail: discovery depth, documentation depth, eval-harness polish → follow-ups (recorded, not dropped).
- Transliteration search (ADR-0206) and full-quality fuzzy L8 (ADR-0216) → later phases.
- Graph nodes/edges built from roots/lemmas → roadmap Phase 4.
- Additional Quran qira'at / editions and multi-RAG/comparative-scripture surfaces → their roadmap phases.

</deferred>

---

*Phase: 3-Quran Search & Linguistics*
*Context gathered: 2026-09-26*
