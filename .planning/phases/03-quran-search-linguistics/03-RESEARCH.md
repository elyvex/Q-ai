# Phase 3: Quran Search & Linguistics — Research

**Researched:** 2026-09-26
**Domain:** Arabic normalization, lexical/morphological search over a canonical Quran corpus (Rust workspace)
**Confidence:** HIGH for in-repo implementation state (read directly this session); MEDIUM for dataset/licensing posture (governed absences, owner gate open)

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Scope, Finish Line & Evidence**
- **D-01:** Roadmap success criteria govern the boundary. Phase 3 completes when each of the five success criteria is satisfied by existing code plus at least one repeatable check, and both phase requirements are covered. Implement only the gaps those criteria need. — **Reversibility:** reversible — a planning-boundary choice.
- **D-02:** High-value legacy hardening is **in scope as exit evidence**: `qai doctor --indexes` drift checks, search/morphology golden sets, and explicit performance budgets. The legacy tail (discovery depth, documentation depth, eval-harness polish) is deferred to follow-ups, recorded explicitly rather than silently dropped. — **Reversibility:** reversible.
- **D-03:** Added **alpha-release bar** (owner directive): after Phase 3, Phases 1–3 together must form a coherent, runnable alpha — search/linguistics features reachable at real user surfaces, not service-only stubs. Scope stays confined to Phase 3's own surface; Phases 1–2 are not re-planned. — **Reversibility:** reversible.
- **D-04:** Preserve the legacy brownfield posture (mirrors Phase 1 D-01 / Phase 2 D-01): map each success criterion to current code + a repeatable check, preserve working implementations, and treat the legacy `phase-02-rag` board as authoritative evidence. — **Reversibility:** reversible.

**Morphology Dataset, Licensing & Linguist Gates**
- **D-05:** **Verify-then-bundle.** Engineering builds the morphology importer, tokenization alignment, machine-readable source/license matrix, and the activation-rejection gate now. **Quranic Arabic Corpus (QAC)** is the documented **default provider** for morphological tags, lemmas, roots, and conventions. QAC is bundled into the repository **only if** its redistribution terms verify as compatible with the project's data-licensing posture; otherwise ship ADR-0203 **Option B** (user-supplied import via `qai quran morphology import`, with typed "no dataset active" errors — never guessed data). — **Reversibility:** costly — the license/attribution and alignment contract shape the morphology schema and activation gate; once morphology records are active, swapping provider requires new records + validation + a derived-index rebuild.
- **D-06:** **`fawazahmed0/quran-api` is an additional reference/validation source**, used locally (no new download) at `/Users/ali/dev/rust/Q-ai-side/references/01/quran-api/` (git HEAD `47ca096b0976443ba2eab2e45cdf0fb4096a2610`, dated 2026-09-12). Inspect `database/linebyline/` (490+ edition files), `database/chapterverse/`, `database/originals/`, and `editions.json` for Quran text, verse-level alignment, edition/cross-reference metadata, and validation. — **Reversibility:** reversible — a read-only reference source.
- **D-07:** **License discipline.** The quran-api repository's own license (Unlicense / public domain) is **separate** from the licenses of each bundled edition and translation it contains — the repo license does not blanket-cover every dataset. Record the exact upstream source, revision/version, dataset provenance, attribution string, and redistribution/license status in **ADR-0203** and the **machine-readable license matrix**. The activation gate **rejects** activation when required license/attribution evidence is absent (`metadata_only` / `pending_license_review`, never bundling and never guessed data). — **Reversibility:** costly — the license/attribution metadata and activation gate are consumed by provenance, derived-index manifests, and later export/redistribution paths.
- **D-08:** **Linguist sign-off remains a required gate** for ratifying the morphology tagset, root conventions, and alignment rules. If a qualified Arabic linguist is unavailable, record it **explicitly BLOCKED** (naming the missing capability) — never silently treat those conventions as finalized. Consistent with Phase 2 D-03's recorded-owner-gate posture. — **Reversibility:** reversible — a recorded gate can be closed later.
- **D-09:** Preserve the legacy morphology invariants unchanged: **I11** — zero/one/many analyses coexist with **no `is_correct` column**; preference is a per-request policy recorded in output. **I12** — machine-derived analyses are Layer D with algorithm/version/confidence/`verification_status`; **no LLM may generate and store roots/lemmas as dataset-supplied**. **I13** — family relations are typed (`root`/`lemma`/`stem`/`form`/`computational`/`verified`) with provenance. **Alignment (R3)** — a disagreeing tokenization is bridged by an explicit, auditable, hashed alignment table keyed to the exact source edition/version; canonical text is **never re-tokenized**. — **Reversibility:** one-way — these invariants are the correctness foundation of every downstream root/lemma/family/graph result; weakening them invalidates prior analyses and the attribution contract.

**User-Facing Surface Boundary**
- **D-10:** Satisfy SC1–SC4 at the **CLI + versioned HTTP API** level only; the alpha is researcher/developer-facing. CLI: `qai quran search/root/lemma/morphology/family/freq`. API: `/api/v1/quran/search/{exact,normalized,phrase,concatenated,regex}` plus `/normalization/{preview,profiles}`. — **Reversibility:** reversible — the UI is deferred, not removed.
- **D-11:** Every search/linguistics result carries the **explainability payload** — the ordered normalization rule set applied, segmentation, and canonical character offsets (**I9**/**I10**). This is a result contract downstream agents and later UI consume. — **Reversibility:** costly — removing explainability from results reverses the published "every match reports why it matched" guarantee and breaks highlight/citation fidelity.
- **D-12:** Defer the interactive TUI word inspector, Web GUI, word-inspector UI, and the `REQ-quran-result-contract` research checksum to **Phase 5**, consistent with the existing legacy scope fence and roadmap phase split. — **Reversibility:** reversible.

**Linguistics Tool Depth**
- **D-13:** Implement the **SC-aligned core set only**: exact / normalized / phrase / concatenated / regex search; root + lemma search; morphology token inspection with multi-analysis; word family; frequency + distribution + co-occurrence. Every tool returns provenance/attribution, and every numeric output carries **`CountingRules`** (**I15** — state the counting rules; never assert numerological significance). — **Reversibility:** costly for the tool/numeric-result contract (published output schema consumed by later surfaces); reversible for the deferral itself.
- **D-14:** Defer collocation, interval, first/last-occurrence, `unusual_usage`, `hapax_search`, `near_duplicate_passages`, `missing_expected_form`, and the numeric-report tool to later phases. Record the deferral explicitly; do not silently drop them from the legacy board. — **Reversibility:** reversible.
- **D-15:** Derive all search/linguistics data read-only from canonical rows into **derived** tables/indexes; canonical bytes are never mutated. Preserve the frozen `qai-text-hash-v1` / `structure_hash` / `token_order_hash` recipes (Phase 2 D-12); additive, domain-separated recipes only. Re-assert **I8** (normalization never mutates canonical text) with a post-build canonical-unchanged check. — **Reversibility:** one-way — changing a frozen recipe invalidates every stored hash and requires a global re-hash migration.

### the agent's Discretion
- Exact normalization rule ordering, profile ladder versions, SpanMap storage format, index manifest/hash function, checkpoint payload schemas, CLI flag naming, golden-set composition, and operator-facing wording may follow existing project conventions (Phase 1 D-16 convention) so long as the locked behavior and invariants above are preserved.
- The precise QAC revision/URL, adapter shape, and tag-mapping details are for research to confirm; the owner ratifies the license/attribution entry.

### Deferred Ideas (OUT OF SCOPE)
- Interactive **TUI word inspector**, **Web GUI**, word-inspector UI, and `REQ-quran-result-contract` research checksum → roadmap Phase 5.
- Collocation, interval, first/last-occurrence, `unusual_usage`, `hapax_search`, `near_duplicate_passages`, `missing_expected_form`, and the numeric-report tool → later phases.
- Legacy `phase-02-rag` tail: discovery depth, documentation depth, eval-harness polish → follow-ups (recorded, not dropped).
- Transliteration search (ADR-0206) and full-quality fuzzy L8 (ADR-0216) → later phases.
- Graph nodes/edges built from roots/lemmas → roadmap Phase 4.
- Additional Quran qira'at / editions and multi-RAG/comparative-scripture surfaces → their roadmap phases.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| REQ-quran-normalization | Search normalization that never replaces displayed canonical text; indexed search forms, user-controlled normalization, concatenated-word search (PRD §8) | Normalization engine `quran-normalization` implements N01–N22 rules and L0–L8 profiles [VERIFIED: crates/quran-normalization/src/profile.rs:48-56; crates/quran-normalization/src/rules/mod.rs:59-98] with `NormalizationTrace` (I9) and `SpanMap` (I10). Search services `search_exact`/`search_normalized`/`search_phrase`/`search_concatenated`/`search_regex` exist [VERIFIED: crates/application/src/quran_search.rs:802,897,1217,1480,1992]. Derived-only writes + MV-018 canonical-unchanged check exist [VERIFIED: crates/application/src/quran_forms.rs:222,238]. Gaps: human-reviewed goldens (OD-12 linguist), 120-case concatenated golden set absent, CLI/API user-controlled profile surface incomplete. |
| REQ-quran-linguistics | Token linguistic data with multiple analyses and Arabic word families (PRD §9) | Morphology types + adapters + alignment + validation + family exist, datasetless [VERIFIED: crates/quran-morphology/src/lib.rs:17-44]. Application services for token analyses, root/lemma search, browse, family relations exist [VERIFIED: crates/application/src/quran_morphology.rs:1140,1505,1531,1746,1808]. Migration 0017 encodes no-`is_correct` discipline [VERIFIED: migrations/sqlite/0017_quran_lexicon.up.sql:4-5]. Gaps: no active licensed dataset (OD-11 BLOCKED), license/attribution activation gate missing, family builders incomplete, `word_family` has no CLI/API surface, root/lemma frequency stubs. |
</phase_requirements>

## Summary

Phase 3 is an **evidence-driven brownfield gap-closure** phase. The legacy board `docs/03-plan/phases/phase-02-rag/` (numbering drift: titled "Phase 2", covers roadmap Phase 3) reports roughly **58 ☑ / 42 ◐ / 30 ☐** task rows and **0/50 ACs, 0/14 ADRs** (CONTEXT.md records "~58/114 tasks"; a raw symbol scan this session found 64/42/30 lines, inflated by legend/summary rows). Substantial, tested code already exists for every success criterion's *mechanics*: the normalization engine (N01–N22, L0–L8), FTS5-backed exact/normalized/phrase search, skeleton+trigram concatenated search with segmentation, multi-analysis morphology types, root/lemma search, counting with mandatory `CountingRules`, 19 `doctor --indexes` checks, and a broad Rust test harness. This session verified `cargo xtask migrate-check` (21 migrations, checksums stable), `cargo xtask arch-check` (no forbidden edges), and `cargo test -p quran-search --test search_parity` (5,000-substring tokenizer-parity gate) all pass on the current checkout.

The phase does **not** need a greenfield build. It needs (a) closure of a small number of genuine behavioral gaps that block the five success criteria, and (b) evidence that satisfies each criterion repeatably. The most load-bearing gaps are: **root/lemma frequency are hard stubs** (`CountingRules::UnavailableDataset`, so SC4 "for any root or lemma" is not satisfiable today even with an active dataset); **`quran.word_family` has no CLI command and no HTTP route** (SC3's "word family" is service-only); **no concatenated golden-set fixture exists** despite ADR-0207/AC-P2-08 requiring a 120-case set; **the license/attribution activation-rejection gate required by D-07 is not implemented** (`activate_morphology` gates on approval + findings + alignment but never on license evidence; the CLI hardcodes `license_status: "Unspecified"`); and the **tool-registry registers only `quran.get_ayah`/`quran.get_context`**, not search/root/lemma/morphology/family.

Two owner gates are open and must be recorded as explicit **BLOCKED** items in every relevant plan: **OD-11** (morphology dataset selection & licensing; owner-decisions records "QAC v0.4 gated candidate" but unratified) and **OD-12** (normalization rule catalog + named linguist; catalog is implemented in code, linguist is unassigned). The reference repo `fawazahmed0/quran-api` is present at the pinned HEAD; inspection this session confirmed its `LICENSE` is the Unlicense **and** that `editions.json` carries **no per-edition license field at all** — so the per-edition license matrix D-07 requires cannot be derived from that repo and must be captured separately. The reference repo contains translations and Arabic text, **not morphology** — QAC remains the morphology default provider per D-05.

**Primary recommendation:** Plan Phase 3 as a criterion-by-criterion closure with an explicit evidence matrix: for each of SC1–SC5, one plan to (i) confirm the existing implementation via a named automated command, (ii) close the specific gap(s) listed in the Gap Register, and (iii) add the missing repeatable check (golden set / gate). Add one plan for the licensing/license-matrix + activation-rejection gate (D-07), one plan for the CLI/API surface completion (D-10/D-13), and one plan for owner-gate recording (OD-11/OD-12) as BLOCKED with exact closing commands.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Arabic normalization rules + profiles (N01–N22, L0–L8) | Pure domain crate (`quran-normalization`) | — | No I/O, no async, no DB; enforced by `xtask arch-check` [VERIFIED: xtask/allowlist.toml:82-90] |
| Derived search forms + skeleton/trigram stores | Domain crate (`quran-search`) + derived tables/index files | `storage-sqlite` | Derived from canonical; generation-stamped; rebuildable (I14) [VERIFIED: migrations/sqlite/0014_quran_forms.up.sql] |
| Query/index tokenizer parity | `quran-search` (`TokenizerFamily`) | — | One shared pipeline for query and index paths (R6) [VERIFIED: crates/quran-search/tests/search_parity.rs:1-8] |
| Exact / normalized / phrase / concatenated / regex search services | Application (`crates/application/src/quran_search.rs`) | `quran-search` port | Service layer assembles `SearchHit` through one validating constructor (I9/I10) [VERIFIED: crates/quran-search/src/hit.rs:97-145] |
| Search HTTP endpoints | `server` (`crates/server/src/api.rs`) | `application::quran_search_api::SearchBackend` | Routing only; no new arch edges [VERIFIED: crates/server/src/api.rs:1323-1327] |
| Morphology dataset import + alignment + validation | Domain crate `quran-morphology` + application job | `storage-sqlite` staging | Staging → validation → approval-gated activation; canonical never re-tokenized (D-09/R3) [VERIFIED: crates/application/src/quran_morphology.rs:550-615] |
| Morphology token inspection / root / lemma / family | Application (`quran_morphology.rs`) + `quran-morphology` types | CLI/API surface (missing for family) | Multi-analysis policy, typed family relations (I11/I13) |
| Frequency / distribution / co-occurrence | Application (`quran_counting.rs`) | Derived token-form tables | Exact SQL aggregation, never FTS frequencies; `CountingRules` mandatory (I15) [VERIFIED: crates/application/src/quran_counting.rs:37-50] |
| Explainability payload (rules, segmentation, canonical offsets) | `quran-search` (`SearchHit`/`Segmentation`) + `quran-normalization` (`NormalizationTrace`/`SpanMap`) | Application assembly | Type-level mandatory, not optional (I9/I10/D-11) [VERIFIED: crates/quran-search/src/hit.rs:124-127] |
| License/attribution evidence + activation rejection | Application activation gate + machine-readable matrix (missing) | `licenses/` capture dir (docs only) | D-07; consumed by provenance/derived manifests |
| Derived-index drift diagnostics | `application::quran_doctor_indexes` + CLI `doctor --indexes` | — | 19 read-only checks [VERIFIED: crates/application/src/quran_doctor_indexes.rs:25-45] |
| Typed tool registration (agents/API) | `tool-registry` | `application::quran_tools` | Only 2 reader tools registered today [VERIFIED: crates/tool-registry/src/lib.rs:154] |

## Project Constraints (from AGENTS.md)

Actionable directives extracted from `./AGENTS.md` (treat with the authority of locked decisions; research must not recommend approaches that contradict them):

- **Language:** Rust is the primary language; all new code is Rust. Primary interfaces are Web GUI, TUI, CLI, API; deployment is local-first + server.
- **Docs tree is authoritative and ordered:** `docs/00-overview` → `01-requirements` → `02-architecture` (ADRs in `decisions/`) → `03-plan` (phases + backlog) → `04-tasks` → `05-followups` → `06-progress` → `07-technical` → `08-api` → `09-testing` → `10-operations`. Read the active phase plan and `docs/06-progress/status.md` before work.
- **Before starting work:** read `AGENTS.md`, `docs/00-overview/project-overview.md`, `docs/03-plan/current-plan.md`, the active phase plan, `docs/06-progress/status.md`, and `docs/05-followups/open-questions.md`.
- **Naming conventions:** tasks `TASK-nnn-slug.md`; phases `phase-NN-slug/`; ADRs `ADR-nnnn-title.md`; deliverables `Dn.n`; acceptance criteria `AC-Pn-nn`; follow-ups `YYYY-MM-DD-session-nnn.md`.
- **Planning rules:** never modify the master plan without justification; every implementation task must have a TASK-ID; update task status after implementation; record important architectural decisions as ADRs; never mark a task complete until its acceptance criteria are satisfied.
- **Completion rules:** implement → run tests → run lint/checks → update the task document → update phase progress → update `docs/06-progress/task-done-rollup.md` → add important follow-ups to `docs/05-followups/` → update `CHANGELOG.md` when appropriate.
- **Key principles (PRD-derived):** exact text before generated interpretation; every factual claim traceable; Quran stored structurally, not only as RAG chunks; canonical text separated from translations/annotations; **no model may fabricate a verse, hadith, chain, grading, or citation**; answers must distinguish quotation / source summary / AI analysis; research reproducible; local data stays local unless the user enables a remote provider; agents and tools use deny-by-default permissions; religious conclusions must not be falsely presented as scholarly consensus.
- **From `.agent/coding-rules.md`:** no domain code depends on `api`/`server`/`tui`/`cli`/concrete providers; canonical rows written only through `CanonicalWriter` requiring an `ApprovalToken`; every derived artifact records the version of everything it was derived from; deny-by-default permissions/network/filesystem; nothing that can modify data runs inside `doctor`; secrets never logged or embedded; imported internet content is untrusted until validated.
- **Repo tooling convention (root `AGENTS.md`):** this repo is indexed by the `graft/` code-context graph — prefer `graft ask`/`graft skeleton`/`graft find` over raw grepping; RTK CLI proxy is preferred for token-efficient commands.

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Rust | 1.97.1 (pinned) | Whole workspace | [VERIFIED: rust-toolchain.toml] |
| SQLite + FTS5 | sqlite3 3.51.0 (system CLI); bundled via `libsqlite3-sys` in the workspace | Canonical + derived tables + FTS5 index | ADR-0001 (SQLite default), ADR-0201 (FTS engine = FTS5, DEV-05 supersedes Tantivy) [VERIFIED: crates/storage-sqlite/tests/fts5_available.rs] |
| `sqlx` | workspace-pinned | SQLite access behind `storage` ports | ADR-0001 [VERIFIED: crates/storage-sqlite] |
| `regex-automata` | workspace | DFA-only regex for `search_regex` | ADR-0212 (no backtracking) [VERIFIED: crates/quran-search/src/regex.rs:40-41] |
| `serde` / `serde_json` | workspace | All result/explainability payloads | `SearchHit`, `CountingRules`, `IndexManifest` all serde [VERIFIED: crates/quran-search/src/hit.rs] |
| `thiserror` | workspace | Typed `Diagnostic` errors with `QAI-*` codes | ADR-0010 [VERIFIED: crates/quran-search/src/error.rs:90-106] |
| `clap` | workspace | `qai` CLI command tree | [VERIFIED: crates/cli/src/lib.rs:43-122] |
| `axum` | workspace | Versioned HTTP search/normalization API | ADR-0201/owner ratification [VERIFIED: crates/server/src/api.rs:1291-1328] |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `unicode-normalization` | workspace | NFC composition (rule N16, ADR-0104) | Normalization pipeline only |
| `lru` | workspace | Lookup cache (ADR-0113) | Canonical reader cache |
| `tempfile` | dev | Test databases/index dirs | All integration tests |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| SQLite FTS5 | Tantivy | Legacy plan targeted Tantivy; ADR-0201 records FTS5 as the implemented engine (DEV-05). Do **not** re-open the engine choice (D-04 preserves working implementations). |
| DFA-only regex (`regex-automata`) | Backtracking `regex` with timeouts | ADR-0212 rejects backtracking (racy timeouts, DoS-adjacent). Locked. |
| Skeleton+trigram concatenated search | Full scan / FTS n-grams | ADR-0207 rejects both (latency / tokenizer-boundary semantic loss). Locked. |

**Installation:** No new packages are required for the core gaps — every gap closes inside existing crates. If Phase 3 adds any dependency, it must pass `xtask arch-check` (the purity fence forbids `llm`/`embeddings`/`retrieval`/vector deps in `quran-normalization`, `quran-search`, `quran-morphology`) [VERIFIED: xtask/allowlist.toml:82-96].

**Version verification:** `rustc 1.97.1`, `cargo 1.97.1`, `sqlite3 3.51.0` confirmed installed this session. Crate versions are workspace-pinned in `Cargo.toml`/`Cargo.lock`; no new registry packages are proposed, so no `npm view`/`pip index` checks are applicable to this phase.

## Package Legitimacy Audit

**No external packages are proposed by this research.** Every recommended change closes a gap inside existing workspace crates (`quran-normalization`, `quran-search`, `quran-morphology`, `application`, `cli`, `server`) using dependencies already pinned in `Cargo.lock`. Therefore the Package Legitimacy Gate has no new package to check, and no `[ASSUMED]` package installs are proposed.

| Package | Registry | Age | Downloads | Source Repo | Verdict | Disposition |
|---------|----------|-----|-----------|-------------|---------|-------------|
| *(none — no new packages proposed)* | — | — | — | — | — | — |

**Packages removed due to [SLOP] verdict:** none.
**Packages flagged as suspicious [SUS]:** none.

*If planning introduces a new crate (e.g. a streaming reader or a checksum helper), its install must be gated behind a `checkpoint:human-verify` task and validated with `gsd_run query package-legitimacy check --ecosystem crates <pkg>` before use.*

## Architecture Patterns

### System Architecture Diagram

```text
                         ┌───────────────────────────────────────────────┐
                         │  CANONICAL (immutable, Phase 2)               │
                         │  quran_editions / quran_surahs / quran_ayahs  │
                         │  quran_tokens   (ApprovalToken-gated writes)  │
                         └───────────────┬───────────────────────────────┘
                                         │ read-only derive
                                         ▼
              ┌──────────────────────────────────────────────────────────┐
              │  DERIVED REBUILD (generation-stamped, I14)               │
              │  forms rebuild ──► quran_token_forms / quran_ayah_forms  │
              │      │            quran_skeletons (ayah + 3-ayah windows)│
              │      │            index gen-N/ : FTS5 db + trigram.db    │
              │      └── MV-018 post-check: canonical hashes unchanged   │
              │  morphology import ─► staging ─► validate/findings ─►   │
              │      align ─► approval+license gate ─► lexicon tables    │
              └───────────────┬──────────────────────────────────────────┘
                              │ query
        ┌─────────────────────┼──────────────────────────┐
        │                     │                          │
        ▼                     ▼                          ▼
 ┌────────────┐      ┌────────────────┐        ┌──────────────────┐
 │ qai CLI    │      │ HTTP /api/v1   │        │ tool-registry    │
 │ quran      │      │ quran/search/* │        │ (agents/API)     │
 │ search/    │      │ normalization/*│        │ currently only   │
 │ root/count/│      │ (morphology/   │        │ get_ayah/get_ctx │
 │ morphology │      │  count/family  │        └──────────────────┘
 │ /graph     │      │  routes ABSENT)│
 └─────┬──────┘      └───────┬────────┘
       └──────────┬──────────┘
                  ▼
        ┌───────────────────────────────────────────────────────────┐
        │ application services                                      │
        │ quran_search (exact/normalized/phrase/concat/regex)       │
        │ quran_morphology (token/root/lemma/family/browse)         │
        │ quran_counting (frequency/distribution/cooccurrence)      │
        │ quran_search_cache (generation-keyed)                     │
        └───────────────────────────────────────────────────────────┘
                  │ every hit → SearchHit { trace (I9), span (I10),   │
                  │              quotation, segmentation, warnings } │
                  └───────────────────────────────────────────────────┘
```

**Trace the primary use case:** operator runs `qai quran forms rebuild <edition>` then `qai quran index rebuild` (MV-018 re-checks canonical) → user searches a diacritic-free phrase (`qai quran search 'الرحمن' --profile L3.diacritics`) → the shared normalization pipeline produces the derived and the `SpanMap` → FTS5 or Rust scan returns candidates → the service re-verifies per ayah and assembles a `SearchHit` with canonical span + ordered rule trace + verified quotation → CLI/HTTP renders it. Concatenated search substitutes the L6 skeleton store and trigram postings for recall, and attaches `Segmentation` per canonical token.

### Recommended Project Structure (extend existing; do not restructure)

```text
crates/
├── quran-normalization/   # rules n01..n22, profiles L0-L8, SpanMap, trace (no I/O)
├── quran-search/          # FullTextIndex port, FTS5, skeleton, trigram, regex, hit
├── quran-morphology/      # dataset, adapters, align, validate, tagset, family, policy
├── application/src/       # quran_search.rs, quran_morphology.rs, quran_counting.rs,
│                          # quran_search_api.rs, quran_cli.rs, quran_doctor_indexes.rs
├── cli/src/quran.rs       # command tree + dispatch
├── server/src/api.rs      # /api/v1/quran/* routes
└── tool-registry/src/     # tool registration (currently under-populated)
migrations/sqlite/         # 0013..0018 (+ forward-only additions, never renumber)
fixtures/quran/            # search/, normalization/, lexicon/, adversarial/, golden/
licenses/                  # capture dir (docs only today)
```

### Pattern 1: One normalizing path, query and index (R6)
**What:** Both the query path and the document/index path use the same `NormalizationPipeline` instance/profile so a query and a document can never disagree.
**When to use:** Every search mode and every derived build.
**Example:**
```rust
// Source: crates/application/src/quran_search.rs:897 + crates/quran-search/tests/search_parity.rs
let profile = quran_normalization::ProfileId::L3;
let pipeline = quran_normalization::NormalizationPipeline::for_profile(
    &registry, profile, latest_version(&registry, profile)?)?;
let trace = empty_trace_for(&pipeline);
let normalized_query = normalize_token(&pipeline, &params.text);
```
The 5,000-substring parity gate is the enforced proof: `end_to_end_parity_both_directions` + `family_total_and_stable_over_5000_substrings` (verified passing this session).

### Pattern 2: Type-level explainability (I9/I10/D-11)
**What:** A `SearchHit` cannot be constructed without a `NormalizationTrace` and a canonical span; segmentation is carried for concatenated matches.
**When to use:** Any new search or linguistics result type — mirror this discipline.
**Example:**
```rust
// Source: crates/quran-search/src/hit.rs:124-127
/// Exact ordered rule set applied (I9 — mandatory, never optional).
pub explanation: NormalizationTrace,
/// Concatenated-match segmentation (empty for all other tools).
pub segmentation: Vec<Segmentation>,
```

### Pattern 3: Typed capability-unavailable, never guessed data
**What:** Morphology-gated tools return a typed error naming the missing capability instead of empty/guessed results.
**When to use:** Every root/lemma/family/pattern/affix tool and every lexicon-gated count.
**Example:**
```rust
// Source: crates/application/src/quran_counting.rs:718-731
pub async fn root_frequency(
    _db: &SqliteDatabase,
    _root: &str,
) -> Result<FrequencyReport, CountingError> {
    Err(CountingError::UnavailableDataset { capability: "root frequency".to_string() })
}
```

### Pattern 4: Generation-stamped derived artifacts + atomic activation (I14)
**What:** Derived tables/index dirs carry `corpus_generation`; index generations activate atomically; rollback is a pointer flip; GC enforces retention; cache is generation-keyed.
**When to use:** Any new derived table or index addition in Phase 3.
**Example:** `IndexManifest` records `corpus_generation`, `rule_set_versions`, `morphology_dataset_versions` [VERIFIED: crates/quran-search/src/model.rs:226-244]; cache key is `v1:{generation}:{sha256(canonical_json)}` [VERIFIED: crates/application/src/quran_search_cache.rs:38-45].

### Anti-Patterns to Avoid
- **Re-tokenizing canonical text to match a morphology dataset:** forbidden by D-09/R3; bridge with an explicit hashed alignment table.
- **Merging competing analyses / adding an `is_correct` or preferred column:** forbidden by I11; migration 0017 deliberately has none [VERIFIED: migrations/sqlite/0017_quran_lexicon.up.sql:4-5].
- **Generating roots/lemmas with an LLM and storing them as dataset-supplied:** forbidden by I12/ADR-0203.
- **Persisting SpanMaps:** ADR-0208 says span maps are recomputed from canonical text through the shared pipeline, never persisted [VERIFIED: docs/02-architecture/decisions/ADR-0208-span-map.md].
- **Auto-repairing drift in `doctor`:** forbidden; `doctor` is read-only and only suggests commands (AGENTS.md + AC-P2-34).
- **Presenting a cross-verse concatenated fragment as one verse:** `spans_ayah_boundary` must stay true and ayah-level hits win dedup (ADR-0207).
- **Bare numeric counts:** every numeric output must carry `CountingRules`; no interpretive commentary (I15/ADR-0211).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Arabic normalization rules | New ad-hoc char maps | `quran-normalization` N01–N22 + profile registry | SpanMap composition, idempotency, append-only profile versions already proven |
| FTS / lexical search | Custom inverted index | `quran-search::FullTextIndex` + `Fts5Index` | Tokenizer parity, BM25 explain, generation lifecycle |
| Concatenated (spaceless) search | Naive substring scan | `search_concatenated` (skeleton + trigram + verify) | Bounded recall, segmentation, cross-ayah labels (ADR-0207) |
| Regex search safety | Backtracking regex + timeout | `regex-automata` dense DFA + guards | ADR-0212; `MAX_PATTERN_LEN`/DFA/NFA caps [VERIFIED: crates/quran-search/src/regex.rs:13-17] |
| Offset mapping | Manual byte arithmetic | `quran_normalization::SpanMap` | Total per-char function, composition, char indices (ADR-0208) |
| Numeric counting | Ad-hoc `COUNT(*)` with defaults | `quran_counting` + `CountingRules` | Determinism, rule-relativity, anti-numerology disclaimers (ADR-0211) |
| Morphology alignment | Re-tokenize canonical / fuzzy match | `quran-morphology::align` (`DirectKey`/`AlignmentTable`) | D-09/R3; auditable, hashed, edition-keyed |
| Index generations / retention | Manual file management | `quran_index` build/gc/rollback + `IndexManifest` | Atomic activation, drift, cache invalidation (ADR-0213/0214) |
| Diagnostic error codes | Free-form strings | `Diagnostic` `QAI-*` codes | ADR-0010; stable CLI exit codes + HTTP status mapping |

**Key insight:** In this domain, the expensive and dangerous work is *offset fidelity and attribution*, not feature surface. Every hand-rolled shortcut in normalization/morphology/alignment silently breaks the "every match reports why it matched, and canonical text is never altered" guarantee that later phases (graph, answer contract, agents) depend on.

## Runtime State Inventory

Phase 3 is **not** a rename/refactor of source symbols, but it **does** build and activate runtime state that outlives a source edit. The canonical question — *after every file in the repo is updated, what runtime systems still hold derived state?* — applies to derived artifacts. Every category is answered explicitly.

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data (SQLite) | Derived tables from migrations 0013–0018: `normalization_rules`/`normalization_profiles` (append-only triggers), `quran_token_forms`, `quran_ayah_forms`, `quran_skeletons`, `index_pointers`, `index_build_runs`, `search_result_cache` (128 MiB cap), `quran_datasets`/`quran_roots`/`quran_lemmas`/`quran_token_analyses`/`quran_morphemes`/`word_family_relations`/`morphology_review_queue`, `morphology_staging_*`/`morphology_alignment`/`morphology_findings` [VERIFIED: migrations/sqlite/0013–0018 up.sql]. | **Code edit + rebuild**: adding new rule/profile versions or new family builders requires `qai quran forms rebuild` then `qai quran index rebuild`; profiles are append-only (new version, never in-place edit). Lexicon rows require a new activation if the dataset changes. |
| Live service config (in DB, not git) | `index_pointers` rows name the **serving generation** per index id; `quran_datasets.state` names the **active** dataset; `search_result_cache` rows are generation-keyed [VERIFIED: migrations/sqlite/0015_quran_indexes.up.sql:12-20, 0017:26, 0016:13]. | **Rebuild/activation, not migration**: generation bump atomically flips the pointer; cache entries for other generations are invalidated wholesale. |
| OS-registered state | None — no launchd/systemd/pm2/Task Scheduler registrations for Quran search. Verified by repo scan for service registration artifacts. | None. |
| Secrets / env vars | `QAI_DATA_DIR` selects the database + sibling `index/` root; no secret key names reference search/linguistics state. | None. |
| Build artifacts / installed packages | Index generation directories `<data_dir>/index/<index_id>/gen-<N>/` containing the FTS5 db and `trigram.db`; these are **not** in git and do not auto-update [VERIFIED: crates/application/src/quran_index.rs:186 `index_root_for_db`; docs/02-architecture/decisions/ADR-0207-concatenated-search.md]. The morphology CLI today writes only staging, never canonical. | **Rebuild**: `qai quran index rebuild` after any forms/profile/lexicon change; `doctor --indexes` reports drift (`QAI-IDX-0101`). |

**Canonical safety note:** the frozen `qai-text-hash-v1`/`structure_hash`/`token_order_hash` recipes must not change (D-15); MV-018 re-verifies canonical hashes after every derived build [VERIFIED: crates/application/src/quran_forms.rs:222-255].

## Evidence Matrix: Success Criteria → Current State → Gap

> Evidence commands are the Phase-2-proven patterns. Run from the repo root.

### SC1 — "User can search an Arabic phrase without diacritics and get exact-location hits"

| Layer | Finding | Evidence |
|-------|---------|----------|
| Implementation EXISTS | `search_exact` (L0/L1 fields) and `search_normalized` (registry profile, default L3.diacritics, or adhoc rule list) both return hits through the validating `SearchHit` constructor with canonical span + `NormalizationTrace`. | [VERIFIED: crates/application/src/quran_search.rs:802-880, 897-920] |
| Implementation EXISTS | Phrase search with three modes (`ordered_exact`/`ordered_near`/`unordered_near`) and slop. | [VERIFIED: crates/application/src/quran_search.rs:1091, 1217] |
| Surface EXISTS (CLI) | `qai quran search '<text>' [--profile L3.diacritics] [--rules …] [--phrase …] [--match-mode …]`; exact never silently folds and emits a Persian-codepoint hint. | [VERIFIED: crates/cli/src/quran.rs:203-217, 245-303; crates/application/src/quran_search.rs:872-878] |
| Surface EXISTS (HTTP) | `POST /api/v1/quran/search/{exact,normalized,phrase}`; error → HTTP status mapping. | [VERIFIED: crates/server/src/api.rs:1323-1326, 1015-1106, 828-844] |
| Repeatable check EXISTS | 400-query synthetic golden suite (`200 exact / 100 normalized / 60 phrase / 40 regex`) with `must_not_contain` precision anchors + recall monotonicity assertion. | [VERIFIED: crates/application/tests/search_goldens.rs; fixtures/quran/search/queries.jsonl (401 lines incl. header)] |
| Repeatable check EXISTS | Tokenizer parity 5,000-substring gate (verified **passing** this session). | [VERIFIED: `cargo test -p quran-search --test search_parity` → 3 passed] |
| **GAP** | Golden sets are **synthetic** (`reviewed_by: pending-linguist`); licensed/curated mushaf goldens absent. | [VERIFIED: fixtures/quran/search/queries.jsonl header `"reviewed_by": "pending-linguist"`] |
| **GAP** | No explicit repeatable CLI-level criterion check asserting "UI-display text unchanged while match location is exact" per hit (the storage-level MV-018 check exists; a per-hit assertion that the displayed canonical slice equals the stored canonical slice is implied by `SearchHit` but not asserted as a criterion test). | [ASSUMED — inferred from test inventory] |

### SC2 — "User can search a concatenated (spaceless) phrase and get exact-location hits"

| Layer | Finding | Evidence |
|-------|---------|----------|
| Implementation EXISTS | `search_concatenated` normalizes the query through **L6.skeleton**, uses trigram recall then exact-substring verification, then re-normalization of the sliced canonical text (the slice must reproduce the matched derived substring). | [VERIFIED: crates/application/src/quran_search.rs:1468-1516] |
| Implementation EXISTS | `segment_concatenated` maps each query part to canonical token position/surface; `verify_concatenated`/`verify_concatenated_window`; window matches split per overlapped ayah with `spans_ayah_boundary = true`; ayah-level hits win dedup. | [VERIFIED: crates/application/src/quran_search.rs:1405-1466, 1684-1926] |
| Implementation EXISTS | Skeleton store: `quran_skeletons` with ayah + 3-ayah windows (stride 1, surah-scoped, `ayah_end - ayah_start <= 2`). | [VERIFIED: migrations/sqlite/0014_quran_forms.up.sql:60-75, line 66 CHECK] |
| Implementation EXISTS | Trigram postings at `<root>/gen-<N>/trigram.db`; missing file → scan fallback, never an error. | [VERIFIED: docs/02-architecture/decisions/ADR-0207-concatenated-search.md] |
| Surface EXISTS (CLI) | `qai quran search '<spaceless>' --concatenated [--cross-ayah] [--max-ayah-span N]` (mockup exercised in trycmd). | [VERIFIED: crates/cli/src/quran.rs:220-243; crates/cli/tests/quran/search.trycmd:80] |
| Surface EXISTS (HTTP) | `POST /api/v1/quran/search/concatenated`. | [VERIFIED: crates/server/src/api.rs:1327, 1107-1138] |
| Repeatable check EXISTS | Postings/scan parity + concatenated-with-segmentation tests. | [VERIFIED: crates/application/tests/index_lifecycle.rs; crates/application/tests/search_tools.rs:464-547] |
| **GAP (critical)** | **No concatenated golden-set fixture exists.** The 400-query `queries.jsonl` contains zero `concatenated` tool rows; AC-P2-08 requires a `بسمالله` → 1:1 segmentation case, and acceptance.md specifies a 120-case `search/concatenated.jsonl` set that is absent. | [VERIFIED: `grep -c concatenated fixtures/quran/search/queries.jsonl` → 0; docs/03-plan/phases/phase-02-rag/acceptance.md:151, 274] |
| **GAP** | Cross-ayah and Persian-codepoint concatenated cases specified by acceptance.md are not covered by any fixture. | [VERIFIED: acceptance.md:151] |

### SC3 — "User can select a word and inspect its lemma, root, morphological analyses, and word family"

| Layer | Finding | Evidence |
|-------|---------|----------|
| Implementation EXISTS | `morphology_for_token` returns all analyses with dataset attribution, lemma/root/stem/POS/provenance (multi-analysis, no winner). | [VERIFIED: crates/application/src/quran_morphology.rs:1116-1196] |
| Implementation EXISTS | `root_search`, `lemma_search`, `browse_roots`, `browse_lemmas`; `morphology_compare` (verdicts only, no resolution field). | [VERIFIED: crates/application/src/quran_morphology.rs:1294-1342, 1505-1554, 1199-1250] |
| Implementation EXISTS | Family types: `FamilyRelation` closed enum (8 variants), mandatory explanation, `SUGGESTION_LABEL`, opt-in + confidence floor, `ReviewPromotion` requiring reviewer/timestamp/evidence. | [VERIFIED: crates/quran-morphology/src/family.rs:19-35, 143-210] |
| Implementation EXISTS | `word_family` read path + `build_same_root_relations` (same_root only). | [VERIFIED: crates/application/src/quran_morphology.rs:1746-1805, 1808-1830] |
| Implementation EXISTS | Schema has **no** `is_correct`/`is_primary`/`selected` column; `AnalysisPolicy` counts suppression never silently. | [VERIFIED: migrations/sqlite/0017_quran_lexicon.up.sql:4-5; crates/quran-morphology/src/policy.rs] |
| Surface EXISTS (CLI) | `qai quran morphology {token,compare,datasets,root,lemma,affix,diff}`; `qai quran root list`. | [VERIFIED: crates/cli/src/quran.rs:419-431, 525-620] |
| **GAP (critical)** | **`quran.word_family` has no CLI command and no HTTP route.** D-10 names `qai quran family`; no such variant exists in `QuranAction` or `MorphologyAction`; the only family-ish CLI path is `qai quran graph root-family` (lexicon-gated ranked ayahs, not relations). P2-T89 (API endpoints) is ☐ and T90 (family command) is ◐. | [VERIFIED: crates/cli/src/quran.rs:222-238 (action list lacks `Family`); acceptance.md:87, P2-T89 ☐] |
| **GAP** | Family builders incomplete: only `same_root` is built; `same_form`/`same_lemma`/`same_stem`/`derived`/`inflectional`/`affix` are absent (P2-T85 ◐). `build_same_root_relations` pairs only adjacent `windows(2)` members. | [VERIFIED: crates/application/src/quran_morphology.rs:1746-1805; docs/03-plan/phases/phase-02-rag/tasks.md P2-T85] |
| **GAP** | No family golden set (P2-T92 ☐, 120 curated families + linguist review). | [VERIFIED: acceptance.md:86, 156] |
| **GAP** | Entirely dataset-gated: without an active dataset, every SC3 tool returns a typed `UnavailableDataset` error. Synthetic lexicon fixture exists and can activate end-to-end, but is **`synthetic_test_only`** and never scholarly ground truth. | [VERIFIED: fixtures/quran/lexicon/{sample-a.json,sample-b.csv,intermediate-schema.json}; crates/application/tests/morphology_import.rs] |
| **GAP** | `affix_search` dataset backend returns an **empty vector** when a dataset is active ("Full morpheme index arrives with T73 field population") — a silent empty, not a typed capability error. | [VERIFIED: crates/application/src/quran_morphology.rs:1690-1695] |
| **GAP** | Tool-registry registers only `quran.get_ayah`/`quran.get_context`; D-13 requires search/root/lemma/morphology/family tools registered with attribution. | [VERIFIED: crates/tool-registry/src/lib.rs:154] |

### SC4 — "User can view frequency, distribution, and co-occurrence for any root or lemma"

| Layer | Finding | Evidence |
|-------|---------|----------|
| Implementation EXISTS (surface forms) | `frequency` (exact SQL `COUNT(*)` over the profile column + per-surah), `distribution` (surah partitions + provenance + single-source warning), `cooccurrence` (token windows + cross-ayah flags). All carry a complete `CountingRules` block. | [VERIFIED: crates/application/src/quran_counting.rs:210-286, 451-511] |
| Implementation EXISTS | `CountingRules` has 6 mandatory fields incl. `profile`, `profile_version`, `datasets`, `multi_analysis_handling`, `window`, `exclusions`. | [VERIFIED: crates/application/src/quran_counting.rs:37-50] |
| Surface EXISTS (CLI) | `qai quran count {frequency,distribution,cooccurrence,...}`. | [VERIFIED: crates/cli/src/quran.rs:433-522, 846+] |
| Repeatable check EXISTS | `counting.rs` tests: `frequency_exact_and_deterministic`, `distribution_and_numeric_report`, `cooccurrence_and_collocation`, and `morphology_gated_targets_unavailable`. | [VERIFIED: crates/application/tests/counting.rs:131-260] |
| **GAP (critical)** | **`root_frequency` and `lemma_frequency` are hard stubs returning `UnavailableDataset`** — even with an active lexicon. SC4 says "for any root or lemma", so the criterion is not satisfiable today. | [VERIFIED: crates/application/src/quran_counting.rs:718-731] |
| **GAP** | `MultiAnalysisHandling` only ever uses `SingleSource`; `AllAnalyses`/`OneVotePerToken` are declared but never produced (`rules_for` always sets `SingleSource`, `datasets: ["stored-forms"]`). | [VERIFIED: crates/application/src/quran_counting.rs:194-201; MultiAnalysisHandling at :26-33] |
| **GAP** | Counting has **no HTTP endpoints** (P2-T104 ◐); D-10/D-13 expect CLI + API reachability. `qai quran count` exists but `/api/v1/quran/...count/*` does not. | [VERIFIED: crates/server/src/api.rs:1291-1328 route list contains no count/root/lemma/morphology/family route] |

### SC5 — "Displayed canonical text is never modified by normalization in any result"

| Layer | Finding | Evidence |
|-------|---------|----------|
| Implementation EXISTS | `verify_canonical_unchanged` / `verify_canonical_unchanged_in` compute `text_hash` and fail with `IndexError::CanonicalChanged` on drift; the forms rebuild stages + post-checks MV-018 inside the write transaction. | [VERIFIED: crates/application/src/quran_forms.rs:222-255] |
| Implementation EXISTS | Morphology import reports `mv018_unchanged`; activation promotes staged rows without touching canonical. | [VERIFIED: crates/application/src/quran_morphology.rs:189-190, 550-615] |
| Implementation EXISTS | `doctor --indexes` runs `quran.canonical_unchanged` as one of its 19 checks. | [VERIFIED: crates/application/src/quran_doctor_indexes.rs:25-45, 189] |
| Implementation EXISTS | Derived-only table layout: `quran_token_forms` FK → `quran_tokens`, no canonical write path; frozen hash recipes unchanged. | [VERIFIED: migrations/sqlite/0014_quran_forms.up.sql:1-32] |
| Repeatable check EXISTS | trycmd asserts the `MV-018 canonical-unchanged: pass (sha256:…)` line after forms rebuild; storage tests cover canonical-unchanged. | [VERIFIED: crates/cli/tests/quran/search.trycmd:26-32] |
| **GAP (low)** | SC5 lacks a dedicated criterion-level assertion that **the text passed to display** (not just stored canonical) is byte-identical to canonical for every hit — the guarantee is structural (`SearchHit.quotation` is a `QuranQuotation` from canonical) but should be pinned by a test to lock the contract for Phase 5 consumers. | [ASSUMED — inferred from test inventory] |
| **GAP (low)** | The `doctor --indexes` `quran.search.smoke` check and the drift warning `QAI-IDX-0101` need a Phase-3 evidence run across the *new* derived artifacts (lexicon/trigram) once morphology activation is exercised end-to-end. | [ASSUMED] |

## Gap Register (consolidated, prioritized)

> Priority: **P0** = blocks a success criterion; **P1** = required exit evidence (D-02) or D-07 gate; **P2** = surface/parity completion; **BLOCKED** = owner/linguist gate.

### BLOCKED — Owner-gated items (must appear in every relevant plan; never a silent pass)

| ID | Item | Why it blocks | Exact command/step to close |
|----|------|---------------|------------------------------|
| **OD-11** | Morphology dataset selection & licensing (ADR-0203 Option A vs Option B). Status 🔴 unanswered; owner-decisions records "QAC v0.4 gated candidate" but **unratified**. | Blocks bundling QAC; without a bundled dataset SC3/SC4 run only on the synthetic fixture (never scholarly). | Owner: (1) capture QAC license to `licenses/qac/{LICENSE.txt,capture.json,attribution.txt}` per `licenses/README.md` (source_url + capture_date + capturer mandatory); (2) if `redistribution_allowed: true` with all fields, ratify ADR-0203 Option A and set the machine-readable matrix entry; else accept Option B and keep the CLI/typed-error fallback. Record the decision + date in `docs/05-followups/owner-decisions.md`. |
| **OD-12** | Normalization rule catalog + named linguist (ADR-0204). Catalog is **implemented in code** (N01–N22, L0–L8) but ADR-0204/0205 are Draft pending linguist review; owner-decisions records "identity-profile-only, linguist 🔴". | Blocks ADR acceptance and golden-set ratification (AC-P2-02/46); masks any linguistically wrong fold (R2). | Owner/editorial: name a qualified Arabic linguist (0.4 FTE) and record the sign-off in `docs/reviews/`; then flip ADR-0204/0205 to Accepted. Until then every plan must carry OD-12 BLOCKED. |
| **D-08** | Linguist sign-off on tagset, root conventions, alignment rules (ADR-0210/0215). | Family/root search cannot be ratified as linguistically correct. | Same linguist engagement as OD-12; add ADR-0210/0215 acceptance to the sign-off record. |

### P0 — Blocks a success criterion

| # | Gap | Criterion | Recommended closure | Evidence of closure |
|---|-----|-----------|---------------------|---------------------|
| G-01 | `root_frequency`/`lemma_frequency` are hard stubs (`UnavailableDataset`). | SC4 | Implement exact SQL frequency/distribution/co-occurrence over lexicon joins (`quran_token_analyses` root_id/lemma_id), carrying `CountingRules` with real `datasets: [<dataset>@<version>]` and a chosen `multi_analysis_handling`. Wire CLI (`qai quran count root-frequency`/`lemma-frequency` or `qai quran freq`) and HTTP. | New tests extending `crates/application/tests/counting.rs`: root/lemma frequency deterministic + rule-relativity; still `UnavailableDataset` when no dataset active. |
| G-02 | `quran.word_family` has no CLI command and no HTTP route. | SC3 | Add `qai quran family <kind> <id>` (D-10 CLI shape) dispatching `word_family`; add `POST/GET /api/v1/quran/family` (P2-T89). | trycmd + server contract test; typed `UnavailableDataset` when no dataset. |
| G-03 | No concatenated golden-set fixture (AC-P2-08 / acceptance.md 120-case `search/concatenated.jsonl`). | SC2 | Author `fixtures/quran/search/concatenated.jsonl` with ≥1 `بسمالله`→1:1-with-segmentation case, cross-ayah cases, and Persian-codepoint cases; extend `search_goldens.rs` to run them. | Golden report green with `reviewed_by: pending-linguist` (synthetic) until linguist ratifies. |
| G-04 | License/attribution activation-rejection gate not implemented (D-07). | D-07 gate (feeds SC3/SC4 credibility) | Implement a machine-readable license matrix (per-artifact) + an activation check that **rejects** when required license/attribution evidence is absent (`metadata_only`/`pending_license_review`), and add license fields to `MorphologyImportParams` (CLI currently hardcodes `license_status: "Unspecified"`, `license_json: "{}"`). | New test: activation fails closed without license evidence; passes with captured evidence. |

### P1 — Required exit evidence (D-02) / hardening

| # | Gap | Recommended closure | Evidence of closure |
|---|-----|---------------------|---------------------|
| G-05 | Family builders incomplete (only `same_root`). | Implement `same_form`/`same_lemma`/`same_stem`/`derived`/`inflectional`/`affix` builders per ADR-0210, all typed + explained; keep computational suggestions opt-in. | Extended `family_relations_explained` + non-merge test. |
| G-06 | No performance budgets codified for full corpus. | Codify ADR-0207's `p99 ≤ 150 ms` (concatenated) and the plan §17.1 p50/p99 table as CI benchmark artifacts; today only a 2000 ms fixture bound exists. | Machine-readable benchmark report + threshold check. |
| G-07 | `MultiAnalysisHandling::AllAnalyses`/`OneVotePerToken` declared but never produced. | Implement selectable multi-analysis counting semantics so changing the mode visibly changes count + rules block (AC-P2-28). | Count-rule matrix test. |
| G-08 | `doctor --indexes` drift/lexicon checks need an end-to-end run on activated morphology + trigram. | Run `qai doctor --indexes` after a full rebuild→activate→search soak; assert 19 checks green and `QAI-IDX-0101` on injected drift. | `crates/application/tests/doctor_indexes.rs` extension. |
| G-09 | `affix_search` dataset backend returns a silent empty. | Return a typed capability error (or implement the morpheme index) instead of `Ok(Vec::new())` when a dataset is active. | Contract test. |

### P2 — Surface / parity completion

| # | Gap | Recommended closure | Evidence of closure |
|---|-----|---------------------|---------------------|
| G-10 | Counting has no HTTP endpoints (P2-T104). | Add `/api/v1/quran/count/*` (or document D-10's API list as the accepted boundary — see Open Question Q1). | Server contract test. |
| G-11 | Tool-registry registers only `quran.get_ayah`/`quran.get_context`. | Register search/root/lemma/morphology/family tools with provenance/attribution per D-13. | Tool-conformance test. |
| G-12 | `quran-normalization/src/rules/mod.rs` module doc says "Rules N18–N24 … are not implemented here" while `heuristic_rules()` implements N18–N22 and `by_id` returns `None` for N23/N24 only. | Correct the doc comment to match `heuristic_rules()` and `by_id()` (doc/comment drift, not behavior). | fmt/clippy clean. |
| G-13 | CLI surface named by D-10 (`qai quran search/root/lemma/morphology/family/freq`) is only partially realized (`root` = browse list; root/lemma search live under `morphology`; no `family`, no `freq`). | Either add thin top-level aliases (`lemma`, `family`, `freq`) or record the naming divergence explicitly in the plan (Discretion area). | trycmd snapshot. |

### Deferred (recorded, not dropped — D-02/D-14)

Collocation, interval, first/last-occurrence, `unusual_usage`, `hapax_search`, `near_duplicate_passages`, `missing_expected_form`, numeric-report tool → later phases (note: several already implemented but out of the SC-aligned core set). Transliteration (ADR-0206) and L8 fuzzy quality → later. Graph nodes/edges → Phase 4. These must appear as an explicit deferral list in the plan, not be silently omitted.

## Common Pitfalls

### Pitfall 1: Treating "service exists" as "criterion satisfied"
**What goes wrong:** SC3/SC4 appear met because `morphology_for_token`/`frequency` return data — but the *criterion* names CLI/API reachability and "for any root or lemma". `word_family` has no surface, and root/lemma frequency are stubs.
**Why it happens:** The legacy board counts implementation tasks, not roadmap success criteria; 42 rows are ◐ (partial).
**How to avoid:** Drive the plan from the Evidence Matrix; each criterion needs a named, repeatable check at the criterion's own surface (CLI **and** API where D-10 names it).
**Warning signs:** A plan task says "morphology exists" without a command line a reviewer can paste.

### Pitfall 2: Letting the license gate be documentation-only
**What goes wrong:** `licenses/README.md` reads like a gate, but no code path consults it; `activate_morphology` promotes rows on approval+findings+alignment alone, and the CLI records `license_status: "Unspecified"`.
**Why it happens:** D-07 describes a gate; the implementation stopped at the capture-doc template.
**How to avoid:** Implement the rejection in the activation path and test it fail-closed; make the matrix machine-readable.
**Warning signs:** A dataset with `license_status = "Unspecified"` reaching `state = 'active'`.

### Pitfall 3: Re-tokenizing canonical text for alignment
**What goes wrong:** The tempting fix for a dataset whose tokenization disagrees with ADR-0105 surface tokens is to re-split canonical text — which mutates the canonical key space and breaks every stored hash.
**Why it happens:** Alignment tables are more work than a re-split.
**How to avoid:** Only `quran-morphology::align` (`DirectKey`/`AlignmentTable`); record unmatched ratios; gate approval above threshold (D-09/R3).
**Warning signs:** Any code that writes to `quran_tokens` from an import path — structurally impossible today; keep it that way.

### Pitfall 4: Presenting synthetic data as scholarly ground truth
**What goes wrong:** The synthetic lexicon activates end-to-end and every tool appears "done", but the analyses are `synthetic_test_only`.
**Why it happens:** Synthetic fixtures are the only dataset available while OD-11 is open.
**How to avoid:** Label synthetic fixtures prominently; never mark ADR/goldens Accepted on synthetic data alone (D-08, remaining-engineering-plan §5.5).
**Warning signs:** A verification report claiming root/lemma correctness without a licensed dataset.

### Pitfall 5: Numeric output without CountingRules or with implied significance
**What goes wrong:** Anti-numerology discipline erodes once root/lemma counts are added.
**Why it happens:** New numeric tooling often reuses a raw `COUNT(*)`.
**How to avoid:** Every numeric output (including new root/lemma frequency) carries `CountingRules`; no interpretive commentary (I15/ADR-0211).
**Warning signs:** A count report whose `datasets` field still says `["stored-forms"]` after a lexicon activates.

### Pitfall 6: Generation/cache staleness after adding a derived artifact
**What goes wrong:** Adding lexicon/trigram fields without a generation bump leaves stale cached results or a serving generation built from old inputs.
**Why it happens:** `SearchOutput` is cached by `v1:{generation}:{sha256}`, and the FTS pointer serves a generation.
**How to avoid:** Any derived-input change → new generation via rebuild; cache invalidates wholesale on bump (ADR-0213/0214); `doctor` reports `QAI-IDX-0101` drift.
**Warning signs:** Search results that don't reflect a newly activated dataset until manual rebuild, with no warning.

### Pitfall 7: Regex recall gap with interior diacritics
**What goes wrong:** ADR-0212 records a known limitation: FTS5's `unicode61` tokenizer splits terms at harakat, so anchored regex patterns with interior diacritics have no recall.
**Why it happens:** Term-dictionary expansion never contains harakat-bearing terms.
**How to avoid:** Keep the documented limitation visible; if SC1/SC2 testing surfaces it, prefer the scan fallback rather than a tokenizer change (a tokenizer change is an I14/index-rebuild event).
**Warning signs:** A user reports a regex that "should" match but returns zero with no typed reason.

## Code Examples

Verified patterns from this codebase (use these as the task-action reference; do not re-invent).

### Normalize a query through the shared pipeline (query path)
```rust
// Source: crates/application/src/quran_search.rs:897-920 (search_normalized)
let registry = db_registry(db).await?;
let profile = quran_normalization::ProfileId::L3; // L3.diacritics default
let pipeline = quran_normalization::NormalizationPipeline::for_profile(
    &registry, profile, latest_version(&registry, profile)?)?;
let trace = empty_trace_for(&pipeline);
let normalized_query = normalize_token(&pipeline, &params.text);
```

### Assemble a hit with mandatory trace + span (I9/I10)
```rust
// Source: crates/quran-search/src/hit.rs:97-145 — SearchHitParts → SearchHit::new
// Fields are private; the only constructor validates and requires:
//   quotation: QuranQuotation, canonical_span: CanonicalSpan,
//   explanation: NormalizationTrace (I9 — mandatory, never optional),
//   segmentation: Vec<Segmentation> (concatenated only).
```

### Concatenated search: candidate recall → exact verify → segment
```rust
// Source: crates/application/src/quran_search.rs:1480-1516
// 1. Normalize query through L6.skeleton (spaces vanish by design).
// 2. Trigram recall over stored skeletons; Rust scan fallback if no posting file.
// 3. Exact substring verification + re-normalization of the sliced canonical text.
// 4. segment_concatenated(...) → Vec<Segmentation> { query_part, canonical_token, canonical_surface }.
// 5. Window matches split per overlapped ayah, spans_ayah_boundary = true, ayah-level wins dedup.
```

### Counting output shape (I15)
```rust
// Source: crates/application/src/quran_counting.rs:37-50
pub struct CountingRules {
    pub profile: String,                                  // e.g. "L3.diacritics"
    pub profile_version: String,
    pub datasets: Vec<String>,                            // ["stored-forms"] until a lexicon activates
    pub multi_analysis_handling: MultiAnalysisHandling,   // SingleSource | AllAnalyses | OneVotePerToken
    pub window: Option<String>,                           // "token:N" for co-occurrence
    pub exclusions: Vec<String>,
}
// Every report (FrequencyReport, DistributionReport, CooccurrenceHit set) embeds `rules`.
```

### Typed capability-unavailable (never guessed data)
```rust
// Source: crates/application/src/quran_counting.rs:718-740
Err(CountingError::UnavailableDataset { capability: "root frequency".to_string() })
// Codes: QAI-CNT-5 (UnavailableDataset), QAI-CNT-2 (UnknownProfile), QAI-CNT-3 (ProfileNotCountable).
```

### Regex bounded execution (I16)
```rust
// Source: crates/quran-search/src/regex.rs:13-41
pub const NFA_SIZE_LIMIT: usize = 1024 * 1024;   // 1 MiB
pub const DFA_SIZE_LIMIT: usize = 4 * 1024 * 1024; // 4 MiB
pub const MAX_PATTERN_LEN: usize = 512;
let dfa = dense::Builder::new()
    .dense(dense::Config::new().minimize(true).dfa_size_limit(Some(DFA_SIZE_LIMIT)))
    .thompson(thompson::Config::new().nfa_size_limit(Some(NFA_SIZE_LIMIT)))
    .build(pattern)?;
// Execution budget default 3000 ms, ceiling 10000; per-principal rate limit (QAI-IDX-0007).
```

### Index manifest (I14 drift inputs)
```rust
// Source: crates/quran-search/src/model.rs:226-244
pub struct IndexManifest {
    pub index_id: String,                                  // "quran.ayah.v1"
    pub schema_version: u32,
    pub corpus_generation: u64,
    pub edition_id: String,
    pub edition_version: SemVer,
    pub rule_set_versions: BTreeMap<String, SemVer>,
    pub tokenizer_version: SemVer,
    pub morphology_dataset_versions: BTreeMap<String, SemVer>,
    pub doc_count: u64,
    pub trigram_postings: u64,
    pub content_hash: String,
    // ... built_at
}
// manifest_content_hash clears content_hash + built_at then sha256s canonical JSON.
```

### Morphology import/activation sequence (D-07 gate goes here)
```rust
// Source: crates/application/src/quran_morphology.rs:550-615 (activate_morphology)
// Current gates, in order:
//   1. batch.state == "staged"
//   2. approval.decision == Some("approved") AND approval.subject_urn == "morphology-dataset:{slug}@{version}"
//   3. zero fatal AND zero error findings
//   4. zero unmatched alignment rows
// MISSING (D-07): a license/attribution evidence gate before promotion.
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Tantivy FTS backend (original plan) | SQLite FTS5 backend | Recorded as DEV-05 in the legacy board | Engine choice is settled; do not re-open (D-04). |
| Single-edition morphology assumption | Edition-relative morphology + explicit alignment table | ADR-0203 (Draft) | Roots/lemmas must never be presented as analysis of a different edition. |
| Mutable profiles / in-place rule edits | Append-only profile versions (`QAI-NORM-0003` on rewrite) | ADR-0205, migration 0013 triggers | Any rule change = new version + full rebuild + drift report. |
| Bare counts with global defaults | Mandatory `CountingRules` on every numeric output | ADR-0211 | Counts are rule-relative and reproducible; anti-numerology policy. |
| Backtracking regex | DFA-only with construction/execution budgets | ADR-0212 | ReDoS structurally impossible; some patterns rejected with typed reason. |

**Deprecated/outdated:**
- The legacy board's original migration numbers `0020`–`0025` were renumbered to implemented `0013`–`0018` (without `.down.sql`); future numbers allocate from the actual directory (never renumber an applied migration).
- `rules/mod.rs` module doc claiming N18–N24 are unimplemented is stale relative to `heuristic_rules()` (N18–N22 implemented) — a G-12 doc fix.
- `qai quran search/root/morphology/family` wording in the legacy README (line ~387) predates the actual CLI tree; the current tree is `search/root/count/morphology/graph`.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | A per-hit assertion that the displayed canonical slice equals stored canonical is not yet pinned by a criterion test (SC5). | SC5 Evidence Matrix | Low — the guarantee is structural via `QuranQuotation`; if a test already exists that I did not locate, the task is a no-op, not a hazard. |
| A2 | SC1 lacks an explicit display-text-unchanged criterion test distinct from storage MV-018. | SC1 Evidence Matrix | Low — same reasoning as A1. |
| A3 | The `doctor --indexes` smoke check has not been exercised end-to-end against an activated morphology dataset + trigram postings. | G-08 | Medium — if it already passes, the task is smaller; if it fails, it is a real gap. |
| A4 | D-10's API list (`search/*` + `normalization/*`) is intended as the complete API boundary for Phase 3, so counting/morphology/family HTTP endpoints are optional. | G-10, Q1 | Medium — if the boundary is meant to include all linguistics tools, G-10 becomes P0 and the plan grows. Needs owner/planner confirmation. |
| A5 | The phase can close SC3/SC4 on the synthetic lexicon fixture while OD-11 stays BLOCKED, provided synthetic data is labeled and no ADR/golden is marked Accepted. | Gap Register, OD-11 | High — if the owner requires a licensed dataset for SC3/SC4, those criteria stay BLOCKED and the phase cannot complete on synthetic alone. This is a decision the owner must ratify. |
| A6 | No new external packages are needed; all gaps close inside existing crates. | Standard Stack / Package Legitimacy | Low — if a gap needs a new crate, the package-legitimacy gate applies. |
| A7 | The `fawazahmed0/quran-api` reference repo contains no morphology data and no per-edition license metadata, so it is a validation/alignment reference only (consistent with D-06). | Summary, OD-11 | Medium — if a downstream task expects morphology from this repo, it will find none. Observed by directory inspection this session. |

## Open Questions

1. **Q1 — Is D-10's API list exhaustive or illustrative?**
   - What we know: D-10 lists CLI `search/root/lemma/morphology/family/freq` and API `search/*` + `normalization/*`. The codebase has search + normalization HTTP routes but no morphology/count/family routes; P2-T89 (morphology/family API) and P2-T104 (counting API) are legacy tasks (☐/◐).
   - What's unclear: whether "satisfy SC1–SC4 at the CLI + versioned HTTP API level" requires HTTP endpoints for SC3/SC4 or only CLI.
   - Recommendation: ask the planner/owner. Default (recommended): implement CLI for SC3/SC4 now and add HTTP for search + normalization only, recording the morphology/count/family API as a *documented deferral* unless the owner says otherwise. If HTTP is required, add `POST /api/v1/quran/{morphology,root,lemma,family,count/*}`.

2. **Q2 — Can SC3/SC4 be marked satisfied on the synthetic lexicon (OD-11 BLOCKED)?**
   - What we know: The synthetic fixture activates end-to-end (`crates/application/tests/morphology_import.rs`); `synthetic_test_only` marks it non-scholarly; D-08 says conventions must not be treated as finalized without a linguist.
   - What's unclear: whether the owner accepts "criterion satisfied with synthetic dataset + explicit BLOCKED license gate" as Phase-3 exit, or requires a licensed dataset first.
   - Recommendation: get an explicit owner ruling. Proposed default: satisfy SC3/SC4 **behaviorally** on synthetic data with labels, keep ADR-0203/0210/0215 + goldens **not Accepted**, and record OD-11 BLOCKED as the remaining gate (mirrors Phase 2 D-03 posture).

3. **Q3 — Which QAC revision and adapter shape?**
   - What we know: D-05 names QAC as the documented default provider; owner-decisions recommends "QAC v0.4 gated candidate"; ADR-0203 §5 names candidate families but no chosen provider. The adapter interfaces (`parse_flat_csv`, `parse_array_shape`) and the intermediate schema already exist.
   - What's unclear: the exact QAC revision/URL, coverage/depth (POS/features/lemma/root/stem/pattern), and whether it supplies one or several analyses per token.
   - Recommendation: research the exact QAC revision **only if** the owner pursues Option A; otherwise ship Option B (user-supplied import) and record the exact URL/revision for the owner to ratify. Do not download data during planning.

4. **Q4 — Performance budgets on the synthetic fixture vs full corpus?**
   - What we know: `search_latency.rs` uses `FIXTURE_BOUND = 2000 ms` and explicitly notes the p50/p99 targets gate full-corpus runs; ADR-0207 names `p99 ≤ 150 ms` for concatenated search.
   - What's unclear: whether Phase 3 is expected to run a full-corpus benchmark (no licensed corpus yet) or record budgets as harness shape only.
   - Recommendation: codify the budget table as a machine-readable artifact and gate the fixture harness; record full-corpus benchmark execution as OD-11-dependent.

5. **Q5 — Exact "displayed text unchanged" criterion test shape for SC5.**
   - What we know: storage-level MV-018 exists and is re-run in every derived build; `SearchHit` carries a `QuranQuotation` from canonical.
   - What's unclear: whether the owner wants a new per-hit property test or accepts existing structural + storage evidence.
   - Recommendation: add one small per-hit test (slice canonical at `canonical_span`, assert byte-identity with the hit's quotation text) to pin the Phase-5 consumer contract; cheap and high-value.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain (stable, pinned) | Entire workspace | ✓ | `rustc 1.97.1` / `cargo 1.97.1` [VERIFIED: `rustc --version`; rust-toolchain.toml] | — |
| SQLite CLI | Manual DB inspection, doctor sanity | ✓ | `sqlite3 3.51.0` (64-bit) [VERIFIED: `sqlite3 --version`] | sqlx-embedded engine inside the binary |
| SQLite FTS5 | Full-text index, search engine | ✓ | FTS5 compiled in (proven by `search_parity`/`fts5_backend` tests passing; `fts5_available.rs` test exists) [VERIFIED: `cargo test -p quran-search --test search_parity` → 3 passed] | none needed; FTS5 is the chosen engine (ADR-0201) |
| `cargo` workspace + `xtask` | Build/test/migrate/arch gates | ✓ | `migrate-check` and `arch-check` verified passing this session | — |
| Local reference repo `fawazahmed0/quran-api` | D-06 validation/alignment reference (read-only, no download) | ✓ | git HEAD `47ca096b0976443ba2eab2e45cdf0fb4096a2610` (2026-09-12); `LICENSE` = Unlicense; `database/{linebyline,chapterverse,originals}/` present (492/492/433 files) | No network download permitted; treat as local-only |
| Licensed Quranic Arabic Corpus (QAC) morphology dataset | D-05 default provider; SC3/SC4 scholarly ratification | ✗ | — | **ADR-0203 Option B**: `qai quran morphology import` with a user-supplied dataset; synthetic lexicon for tests; typed "no dataset active" errors (OD-11 BLOCKED) |
| Qualified Arabic linguist (0.4 FTE) | OD-12 / D-08 sign-off on catalog, tagset, roots, goldens | ✗ | — | **Recorded BLOCKED**; goldens keep `reviewed_by: pending-linguist`; ADRs stay Draft (OD-12 BLOCKED) |
| Licensed mushaf corpus | Human-reviewed (non-synthetic) goldens | ✗ | — | Synthetic goldens (`fixtures/quran/test-edition-min`, `test-edition-rich`) only; record the limit |

**Missing dependencies with no fallback:**
- None that block *engineering*. The two ✗ items that are genuinely owner/legal-gated (QAC dataset, linguist) are recorded as BLOCKED with exact closing steps (see Gap Register), not silently assumed.

**Missing dependencies with fallback:**
- QAC dataset → ADR-0203 Option B (user-supplied import + synthetic test lexicon).
- Linguist → keep `pending-linguist` markers; do not mark ADR-0204/0205/0210/0215 Accepted.
- Licensed mushaf corpus → synthetic goldens labeled as such.

## Validation Architecture

> `workflow.nyquist_validation` is absent from `.planning/config.json`, so Nyquist validation is ENABLED (absent = enabled). This section is included.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]`/`#[tokio::test]` (+ `trycmd` for CLI acceptance snapshots) |
| Config file | Per-crate `Cargo.toml` (workspace root `Cargo.toml`); no separate test config. trycmd files live in `crates/cli/tests/quran/*.trycmd` |
| Quick run command | `cargo test -p quran-normalization -p quran-search -p quran-morphology --lib` then `cargo test -p application --test quran_identity` |
| Full suite command | `cargo test --workspace` |
| Gate commands (Phase-2-proven) | `cargo run -q -p xtask -- migrate-check` · `cargo run -q -p xtask -- arch-check` · `cargo test -p storage-sqlite --test quran` · `cargo test -p quran-corpus --test fixtures` · `cargo test -p cli --test quran -- quran_verify_snapshots` · `cargo test -p provenance --lib` |

### Phase Requirements → Test Map

| Req ID | Behavior (criterion) | Test Type | Automated Command | File Exists? |
|--------|----------------------|-----------|-------------------|--------------|
| REQ-quran-normalization / SC1 | Diacritic-free phrase search returns exact-location hits | integration (golden) | `cargo test -p application --test search_goldens` | ✅ `crates/application/tests/search_goldens.rs` |
| REQ-quran-normalization / SC1 | Query/index tokenizer parity (5,000 substrings) | unit/property | `cargo test -p quran-search --test search_parity` | ✅ (verified passing) |
| REQ-quran-normalization / SC1 | Persian-codepoint hint, never silent fold | integration | `cargo test -p application --test search_tools -- persian_query_gets_hint_not_silence` | ✅ |
| REQ-quran-normalization / SC2 | Concatenated (spaceless) match with segmentation | integration | `cargo test -p application --test search_tools -- concatenated_matches_with_segmentation` | ✅ (single case) |
| REQ-quran-normalization / SC2 | Concatenated golden set (120 cases incl. cross-ayah/Persian) | integration (golden) | `cargo test -p application --test search_goldens -- concatenated` | ❌ **Wave 0** (G-03) |
| REQ-quran-normalization / SC2 | Postings/scan parity; skeleton index | integration | `cargo test -p application --test index_lifecycle` | ✅ |
| REQ-quran-normalization / SC5 | Canonical unchanged after every derived build (MV-018) | integration | `cargo test -p application --test forms_rebuild`; `cargo test -p cli --test quran -- quran_verify_snapshots` | ✅ |
| REQ-quran-normalization / SC5 | Doctor canonical-unchanged check | integration | `cargo test -p application --test doctor_indexes` | ✅ |
| REQ-quran-linguistics / SC3 | Token analyses (multi-analysis, attribution, no winner) | integration | `cargo test -p application --test morphology_import -- compare_and_unavailable_tools` | ✅ |
| REQ-quran-linguistics / SC3 | Root + lemma search / browse | integration | `cargo test -p application --test morphology_import -- browse_roots_and_lemmas_are_attributed_and_bounded` | ✅ |
| REQ-quran-linguistics / SC3 | Word family (typed relations + explanation + promotion) | integration | `cargo test -p application --test morphology_import -- family_relations_explained` | ✅ (same_root only) |
| REQ-quran-linguistics / SC3 | `qai quran family` CLI + `/api/v1/quran/family` | CLI snapshot + server contract | `cargo test -p cli --test quran -- family`; `cargo test -p server` | ❌ **Wave 0** (G-02) |
| REQ-quran-linguistics / SC3 | Family builders (all relation types) + 120-family golden | integration (golden) | `cargo test -p application --test family_goldens` | ❌ **Wave 0** (G-05) |
| REQ-quran-linguistics / SC4 | Frequency/distribution/co-occurrence for surface forms + CountingRules | integration | `cargo test -p application --test counting` | ✅ |
| REQ-quran-linguistics / SC4 | Root/lemma frequency over an active lexicon | integration | `cargo test -p application --test counting -- root_lemma_frequency` | ❌ **Wave 0** (G-01) |
| REQ-quran-linguistics / SC4 | Multi-analysis counting modes | integration (matrix) | `cargo test -p application --test counting -- multi_analysis_modes` | ❌ **Wave 0** (G-07) |
| D-07 license gate | Activation rejects absent license/attribution evidence | integration (fail-closed) | `cargo test -p application --test morphology_import -- license_gate` | ❌ **Wave 0** (G-04) |
| ADR-0203 Option B | Import → activate → query surfaces end-to-end | integration | `cargo test -p application --test morphology_import` | ✅ (synthetic) |
| Derived-index health | 19 `doctor --indexes` checks green + drift warning | integration | `cargo test -p application --test doctor_indexes` | ✅ |

### Sampling Rate

- **Per task commit:** `cargo test -p <crate>` for the touched crate + `cargo fmt --check` + `cargo clippy -p <crate> -- -D warnings`.
- **Per wave merge:** full suite `cargo test --workspace` + `cargo run -q -p xtask -- migrate-check` + `cargo run -q -p xtask -- arch-check`.
- **Phase gate:** Full suite green, both requirements' criteria each backed by a named automated command, and the owner-gate list (OD-11/OD-12) recorded as BLOCKED before `/gsd-verify-work`.

### Wave 0 Gaps (test infrastructure to add before/with implementation)

- [ ] `fixtures/quran/search/concatenated.jsonl` — 120-case concatenated golden set incl. `بسمالله`→1:1 segmentation, cross-ayah, Persian-codepoint (covers SC2, G-03).
- [ ] `crates/application/tests/search_goldens.rs` — extend to run the concatenated golden set (currently only exact/normalized/phrase/regex).
- [ ] `crates/application/tests/counting.rs` — add `root_lemma_frequency` + `multi_analysis_modes` (covers SC4, G-01/G-07).
- [ ] `crates/application/tests/morphology_import.rs` — add `license_gate` fail-closed test + full family-builder test (covers D-07 G-04, SC3 G-05).
- [ ] `fixtures/quran/lexicon/families/curated.jsonl` — 120-family curated set (may stay `reviewed_by: pending-linguist` until OD-12 closes).
- [ ] `crates/cli/tests/quran/family.trycmd` — CLI snapshot for `qai quran family` (covers SC3 surface, G-02).
- [ ] Per-hit canonical-slice byte-identity test (covers SC5, Q5).
- [ ] No framework install needed — Rust test harness + trycmd already present.

## Security Domain

> Security enforcement is ENABLED (ASVS level 1, block on high). This section is included.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No (this phase) | CLI is local single-user; HTTP search routes carry no auth yet (auth lands Phase 11). No new auth surface is added. |
| V3 Session Management | No | No sessions introduced. |
| V4 Access Control | Partial | Agent/tool policy gating is deferred to Phase 7 (ADR-0212 notes "Agent-policy gating is NOT enforced in the service"); the tool-registry gate is the only grant path. Phase 3 must not weaken it. |
| V5 Input Validation | **Yes** | Query text→`SearchParams` (limit clamped `1..=1000`), filter parsing rejects malformed values with `QAI-IDX-0002` [VERIFIED: crates/application/src/quran_search_api.rs:103-161, 227-247]; regex pattern guard `MAX_PATTERN_LEN = 512` + anchored-only + DFA/NFA size caps [VERIFIED: crates/quran-search/src/regex.rs:13-41]; morphology import parses untrusted manifests under the deny-by-default posture and validates with MV-* rules + schema [VERIFIED: crates/quran-morphology/src/validate.rs; fixtures/quran/adversarial/morphology/*.json]. |
| V6 Cryptography | No new crypto | Existing SHA-256 `ContentHash`/`qai-text-hash-v1` recipes are frozen (D-15); never hand-roll or change them. |

### Known Threat Patterns for {Rust + SQLite/FTS5 + CLI/HTTP search}

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| FTS/MATCH syntax injection via query text | Tampering | Query text is normalized then bound as an FTS term/substring; structured filters are typed (`Filter::Surah/JuzRange/Page/RevelationPlace/GlobalRange`), not string-concatenated SQL. Preserve this discipline in any new endpoint. |
| Regex denial-of-service (ReDoS) | Denial of Service | DFA-only engine (no backtracking), `MAX_PATTERN_LEN=512`, anchored patterns only, 1 MiB NFA / 4 MiB DFA caps, execution budget (default 3000 ms, ceiling 10000), per-principal rate limit (`QAI-IDX-0007`) [VERIFIED: crates/quran-search/src/regex.rs; ADR-0212]. |
| Resource exhaustion via unbounded result sets | Denial of Service | `limit` clamped to `1..=1000`; exact totals computed separately; concatenated recall bounded by trigram postings + scan fallback; index generation retention caps disk (ADR-0213). |
| Panic on hostile Unicode input | Denial of Service | Normalization rules are total over Unicode (fuzz-pinned), `transform`/`transform_mask` are total by construction; 5,000-substring parity test over a hostile sample [VERIFIED: crates/quran-normalization/src/rules/mod.rs:104-150; crates/quran-search/tests/search_parity.rs]. |
| Untrusted morphology manifest import | Tampering / Elevation | Import writes only staging; activation requires a granted approval whose `subject_urn` matches the dataset URN; fatal/error findings block activation; unmatched alignment blocks activation; no auto-activation; no network fetch in the deterministic path [VERIFIED: crates/application/src/quran_morphology.rs:550-615]. |
| Silent stale cached search results | Tampering (integrity) | Cache key binds the corpus generation; reads verify the stored generation and drop mismatches; generation bump invalidates wholesale [VERIFIED: crates/application/src/quran_search_cache.rs:1-60]. |
| Path traversal / arbitrary file read via import path | Tampering | CLI reads the operator-specified file; no path is derived from untrusted input. Keep new CLI surfaces free of user-controlled path joining. |
| Registry poisoning of tool results | Spoofing | `SearchHit` cannot exist without a `QuranQuotation` verified against canonical; direct-read paths are structurally exempt because they serve canonical. Do not add a search result type that bypasses verification. |

**Phase-specific security TODO:** the D-07 license/attribution gate (G-04) is a *safety* control as well as a legal one — it prevents activating a dataset whose provenance is unknown. Implement it fail-closed and test that an activation attempt with `license_status = "Unspecified"`/absent matrix entry is rejected.

## Sources

### Primary (HIGH confidence — read directly this session)
- `crates/application/src/quran_search.rs`, `quran_morphology.rs`, `quran_counting.rs`, `quran_search_api.rs`, `quran_search_cache.rs`, `quran_forms.rs`, `quran_cli.rs`, `quran_doctor_indexes.rs` — services, gates, codes.
- `crates/quran-normalization/src/{profile.rs,rules/mod.rs,span.rs}` — L0–L8 profile ids, N01–N22 catalog, SpanMap.
- `crates/quran-search/src/{lib.rs,hit.rs,regex.rs,model.rs,error.rs}` — port, explainability types, regex caps, manifest, `QAI-IDX-*` codes.
- `crates/quran-morphology/src/{lib.rs,family.rs,policy.rs,dataset.rs}` — types, family relation enum, no-winner policy, adapters.
- `crates/cli/src/{lib.rs,quran.rs,doctor.rs}`, `crates/server/src/api.rs`, `crates/tool-registry/src/lib.rs` — surfaces.
- `migrations/sqlite/0013…0018` + `0021` — derived schema; `licenses/README.md` — capture standard.
- `docs/03-plan/phases/phase-02-rag/{README.md,remaining-engineering-plan.md,tasks.md,acceptance.md}` — invariants I8–I16, AC-P2-01…50, gap status.
- `docs/02-architecture/decisions/ADR-0203/0204/0205/0207/0208/0209/0210/0211/0212/0213/0214/0215` — locked/draft decisions.
- `.planning/{ROADMAP.md,REQUIREMENTS.md,PROJECT.md,STATE.md,phases/03-…/03-CONTEXT.md}` — phase contract.
- Local reference repo `/Users/ali/dev/rust/Q-ai-side/references/01/quran-api/` at HEAD `47ca096b0976443ba2eab2e45cdf0fb4096a2610` — `LICENSE` (Unlicense) + `editions.json` structure (no per-edition license field).
- Tool runs this session: `cargo run -q -p xtask -- migrate-check` (OK, 21 migrations); `cargo run -q -p xtask -- arch-check` (OK); `cargo test -p quran-search --test search_parity` (3 passed); `rustc/cargo/sqlite3 --version`.

### Secondary (MEDIUM confidence)
- `docs/05-followups/decisions-needed.md` (OD-11/OD-12 status 🔴) and `docs/05-followups/owner-decisions.md` (QAC v0.4 gated candidate; linguist 🔴) — owner intent, unratified.
- `docs/04-tasks/` and `docs/06-progress/task-done-rollup.md` — reconciliation context (not read line-by-line this session).

### Tertiary (LOW confidence)
- None required — no web-only claims drive this research; all phase-critical facts are in-repo and were read directly.

## Metadata

**Confidence breakdown:**
- Standard stack: **HIGH** — every library/version is in-repo and pinned; no new packages proposed.
- Architecture: **HIGH** — services, types, schema, and surfaces were read directly; test commands executed this session.
- Implementation state (gaps): **HIGH** for absence of `root_frequency`/`lemma_frequency` implementations, absence of `qai quran family`, absence of morphology/count HTTP routes, absence of a concatenated golden fixture, and absence of a license gate in activation (all grep/read-confirmed).
- Dataset/licensing posture: **MEDIUM** — governed absences (no per-edition license in `editions.json`; no QAC license capture; no linguist) are recorded as BLOCKED, not resolved.
- Pitfalls: **HIGH** for in-repo invariants; **MEDIUM** for inferred test-coverage gaps (see Assumptions Log A1–A3).

**Research date:** 2026-09-26
**Valid until:** ~2026-10-26 for stable in-repo facts; the OD-11/OD-12 owner decisions and QAC license status can change at any time and should be re-checked before planning the licensing plan.

