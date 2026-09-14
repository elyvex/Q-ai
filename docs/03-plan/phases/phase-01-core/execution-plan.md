# Phase 1 — Execution Plan (Incremental Build Order)

**Phase:** P1 — Canonical Quran Core
**Status:** Active — generated per `AGENT-PROMPT.md` Step 1
**Source of truth:** `plan.md` (never edited). This file sequences the work and names the
increments; it does not replace the plan.
**Constraint reminder:** nothing in this file may weaken a Phase-0 guarantee; `plan.md`,
`acceptance.md`, `tasks.md` statuses, and `done.md` remain the controlling artifacts.

---

## 0. Owner decisions we are blocked on, and how we proceed without them

| # | Decision | Owner | Why an agent cannot make it | Interim fallback (implemented here) |
|---|---|---|---|---|
| B1 | **ADR-0101** — initial Quran dataset, script, riwayah, license, editorial reviewer (P1-X01/X02) | human / editorial + legal | Licensing and religious-source sign-off are legal/editorial acts | Build and verify the full pipeline end to end on `fixtures/quran/test-edition-min/` (synthetic, ~5 surahs, public-domain-safe) + the 16 adversarial corpora. ADR-0101 written **DRAFT**, never `Accepted`. |
| B2 | **ADR-0114** — reference corpus + comparison procedure + sign-off (P1-X03) | human / editorial | Requires an independent approved corpus and a named signer | Comparator is implemented; **QV-015 is skipped (never silently passed)** when no reference corpus is configured. ADR-0114 stays DRAFT. |
| B3 | **Estimate reconciliation** — plan states 82 ed, task rows sum to 131.0 ed (README §9.1) | owner | Capacity/scope call | Work proceeds in the smallest independently verifiable increments; the **README §9.2 split (1.2a / 1.2b)** is adopted. The 82-vs-131 gap is surfaced in `done.md` §7 with an owner — never silently compressed. |
| B4 | **HTTP framework for API v1** (technology-stack.md §4) | owner | Architecture choice | **axum + tower-http** (both cached locally) adopted provisionally; a short ADR is written **before P1-T39**; Phase-0 hand-rolled health endpoints keep working. |
| B5 | **Phase-0 exit discrepancy** — `status.md` lists outstanding Phase-0 items while the prompt declares Phase 0 complete | owner | Cannot re-scope Phase 0 | Phase-1 work does not touch those items; the discrepancy is recorded in `done.md` §5. |

Everything else — the canonical write path, staging/activation split, hashing recipe,
reference grammar, validator rules, migrations — is engineering and is decided here.

---

## 1. Technology decisions taken now

- **New workspace crates already exist as empty placeholders** (`quran-core`, `quran-corpus`,
  `citations`); they are filled in, not renamed or relocated.
- **New external deps are added incrementally**, each already present in the local registry
  cache so the workspace keeps resolving offline:

  | Increment | Crate | Version | Used by |
  |---|---|---|---|
  | M1 | `unicode-segmentation` | 1.12 | `quran-core`, `quran-corpus` |
  | M3 | `csv` | 1.3 | `quran-corpus::adapters` |
  | M3 | `quick-xml` | 0.37 | `quran-corpus::adapters` (optional second shape) |
  | M4 | `unicode-normalization` | 0.1.25 | `quran-corpus::validation`/`unicode` |
  | M7 | `similar` | 2.7 | `quran-corpus::differ` |
  | M8 | `lru` | 0.12 | `application` reader cache |
  | M9a | `axum` / `tower-http` | 0.8 / 0.6 | `server` (API v1) |
  | M9f | `unicode-bidi` | 0.3 | `server` debug reader (only if CSS RTL is insufficient) |

- **Explicitly never added** in Phase 1: any `llm`, `embeddings`, `retrieval`, vector-store,
  graph, or search index crate (invariant I2).

### 1.1 `xtask/allowlist.toml` entries added before any new edge

```text
[quran-core]      workspace.allow = ["domain"]
[quran-corpus]    workspace.allow = ["quran-core", "domain", "sources", "storage"]
[citations]       workspace.allow = ["quran-core", "domain"]
[storage-sqlite]  workspace.allow = ["storage", "domain", "quran-core"]
[application]     workspace.allow = [... existing ..., "quran-core", "quran-corpus",
                                       "citations", "tools", "tool-registry"]
[tools]           workspace.allow = ["quran-core", "domain"]
[tool-registry]   workspace.allow = ["tools"]
```

`arch-check` is fail-closed: it treats an unlisted crate as "may depend on nothing", so these
entries land in **M0 before** the crates gain dependencies.

---

## 2. Increment sequence

Each increment is independently verifiable. No increment starts until the previous one is green:

```
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p xtask -- arch-check
cargo run -p xtask -- migrate-check
```

### M0 — Dependency graph & decisions registry
- **Tasks:** setup for all of Phase 1
- **Files:** `xtask/allowlist.toml`; root `Cargo.toml` `[workspace.dependencies]`
- **Turned green:** `arch-check` still passes with the new (initially unused) entries.
- **Blockers:** none.

### M1 — `quran-core` skeleton (P1-T06, T07, T10 · D1.1, D1.9 partial)
- **Files:** `crates/quran-core/{Cargo.toml,src/lib.rs,src/error.rs,src/numbers.rs,src/enums.rs,`
  `src/edition.rs,src/structure.rs,src/quotation.rs}`
- **Deps:** `domain`, `serde`, `thiserror`, `unicode-segmentation`
- **Allowlist:** `quran-core -> domain`
- **Tests:** in-crate unit tests + `Diagnostic` conformance (unique `QAI-QUR` codes, remedy +
  next command).
- **Notes:** `QuranQuotation` has no constructor without edition id + version + hash (I6).
  A Phase-1 `Diagnostic` trait is defined locally because invariant I2 forbids
  `quran-core -> storage`, where Phase-0's `Diagnostic` trait lives.

### M2 — Reference grammar + golden set (P1-T08, T09 · D1.1, D1.13 · ADR-0102 draft)
- **Files:** `crates/quran-core/src/reference/{mod,parser,serializer}.rs`;
  `fixtures/quran/golden/references.jsonl` (~300 cases); `tests/quran/reference_grammar.rs`
- **Tests green:** `tests/quran/reference_grammar.rs` → **AC-P1-12, AC-P1-13**
- **Freeze rule:** grammar is frozen only after the property `parse(serialize(ref)) == ref`
  holds for every variant.

### M3 — Intermediate format, adapters, fixtures (P1-T15, T16, T17, T05 · D1.2, D1.13)
- **Files:** `crates/quran-corpus/{Cargo.toml,src/lib.rs,src/error.rs,src/format/*,src/adapters/*}`;
  `docs/schemas/quran-edition-source.v1.schema.json`; `fixtures/quran/test-edition-min/`;
  `fixtures/quran/adversarial/*` (16); `fixtures/quran/golden/ayah_texts.jsonl`
- **Deps:** `quran-core`, `domain`, `sources`, `storage`, `serde`, `serde_json`, `csv`,
  `quick-xml`, `schemars` (+ `jsonschema` dev)
- **Allowlist:** `quran-corpus -> {quran-core, domain, sources, storage}`
- **Blockers:** B1 — ADR-0101 stays DRAFT; the JSON + CSV adapters prove extensibility on the
  synthetic edition.

### M4 — Tokenizer, separators, offsets, hashing (P1-T21, T22, T23 · D1.3 · ADR-0104/0105/0108 drafts)
- **Files:** `crates/quran-corpus/src/tokenize/*`, `src/hashing/*`, `src/unicode/*`
- **Deps:** `unicode-segmentation`, `unicode-normalization`, `sha2`
- **Tests:** reconstruction byte-equality; token-offset property; hash determinism.
  Feeds **AC-P1-05/06/07**.

### M5 — Corpus validator QV-001…QV-028 (P1-T18, T19, T20, T30 · D1.4, D1.13)
- **Files:** `crates/quran-corpus/src/validation/{mod,rules,report}.rs`; registers
  `quran_edition_v1` in the Phase-0 `StructureValidator` registry; `tests/quran/adversarial.rs`
- **Tests green:** `tests/quran/adversarial.rs` → **AC-P1-04** (each fixture fails with the
  specific rule id, no fail-fast).

### M6 — Migrations `0007`–`0012` + repositories (P1-T12, T13, T14, T24 · D1.5, D1.13)
- **Files:** `migrations/sqlite/0007_quran_editions.up.sql` (plan 0010),
  `0008_quran_structure.up.sql` (plan 0011), `0009_quran_divisions.up.sql` (plan 0012),
  `0010_quran_translations.up.sql` (plan 0013), `0011_quran_staging.up.sql` (plan 0014),
  `0012_quran_validation.up.sql` (plan 0015); `migrations/sqlite/checksums.json`;
  `crates/storage/src/quran.rs` (traits); `crates/storage-sqlite/src/quran.rs` (impls);
  `crates/storage-sqlite/tests/quran.rs`
- **Allowlist:** extend `storage-sqlite -> quran-core`
- **Tests green:** `crates/storage-sqlite/tests/quran.rs` → **AC-P1-08** (raw-SQL trigger aborts).

### M7 — Importer, activation, differ (P1-T25–T29, T31 · D1.3 · ADR-0106/0107/0109 drafts)
- **Files:** `crates/quran-corpus/src/import/*` (13 checkpoints, `JobHandler`),
  `src/activate/*` (one-transaction pointer flip + `corpus_generation` + rollback),
  `src/differ/*`; `tests/quran/checkpoints.rs`
- **Deps:** `similar`
- **Tests green:** `tests/quran/checkpoints.rs` → **AC-P1-10/11**
- **Invariants:** importer holds no `ApprovalToken` (I5/I7); crash leaves `Staged`, never
  `Active`. QV-015 skips when the reference corpus is unconfigured (B2).

### M8 — `QuranReader` + cache (P1-T32–T35, T41 · D1.6 · ADR-0113 draft)
- **Files:** `crates/application/src/quran/{mod,reader,context,cache}.rs`;
  `tests/quran/context.rs`; `tests/quran/cache.rs`
- **Deps:** `lru`
- **Tests green:** context boundary/cap property; cache → **AC-P1-19** (no stale text after
  activation).

### M9 — Surfaces (P1-T36–T54, T59 · D1.7–D1.12, D1.14)
- **M9a API v1:** `crates/server/src/quran/*`; axum + tower-http; framework ADR **before**
  P1-T39; envelope/meta/ETag/`Content-Language`/error body → **AC-P1-14**.
- **M9b Tools:** `crates/tools` (`ToolResult`, `ReproducibilityData`), `crates/tool-registry`
  (minimal registry, `quran.get_ayah`/`quran.get_context`) → **AC-P1-15** incl. no-fabrication.
- **M9c Citations:** `crates/citations/*` + `citations` persistence → **AC-P1-21**; ADR-0111.
- **M9d CLI:** `crates/cli/src/commands/quran/*` + `tests/cli/quran.trycmd` → **AC-P1-16**.
- **M9e Doctor:** `qai doctor --quran` (19 checks, `--deep` < 30 s, `--json`) → **AC-P1-17**.
- **M9f Debug reader + docs:** `/debug/read/{edition}/{surah}` → **AC-P1-18**; D1.14 docs.

### M10 — Hardening, soak, exit (P1-T56–T58, T60 · D1.13)
- Golden-set expansion to all §5.2 edge cases; full property + hash-stability suites; full
  soak (import → validate → activate → 10k lookups → `doctor --deep`); exit-gate ritual;
  `docs/plans/handoff-p1-to-p2.md`.

---

## 3. Risk register (mirrors README §13) and the mitigation this plan implements

| # | Risk | Mitigation |
|---|---|---|
| R1 | ADR-0101 slips | Swimlane X opened; engineering built on the synthetic fixture; DRAFT fallback recorded (B1). |
| R2 | Silent text corruption | M5 runs all QV rules, no fail-fast; 16 adversarial fixtures fail with specific ids; QV-015 is Fatal when configured. |
| R3 | Non-lossless tokenization | M4 whitespace-preserving tokenizer + `quran_token_separators`; byte-equality is Fatal (QV-011). |
| R4 | Accidental activation | Staging physically separate; importer holds no token; one-transaction pointer flip; 13-checkpoint crash matrix. |
| R5 | Hashing recipe changes later | ADR-0108 frozen before any real import; algorithm tag in `ContentHash`; rehash procedure documented. |
| R6 | Estimate gap becomes crunch | B3 surfaced; cut candidates are D1.12, D1.7 OpenAPI depth, D1.14 depth — never M1–M7/D1.13. |
| R7 | Sprint 1.2 overload | Split 1.2a (M3–M4) / 1.2b (M5–M7); QA rejection suites scheduled. |
| R8 | Grammar frozen wrong | 300-case golden set + round-trip property land in M2 before freeze. |
| R9 | Translation attribution bypassed | NOT NULL + non-empty CHECK; type-level `AyahView` guard; QV-027. |
| R10 | Cache serves stale text | Generation-keyed LRU, wholesale invalidation, explicit consistency test. |

---

## 4. Definition of done per increment

Run and make green after every increment: `cargo fmt --all`,
`cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`,
`cargo run -p xtask -- arch-check`, `cargo run -p xtask -- migrate-check`.
Persistence work is tested against real SQLite in a tempdir, never a mock. Every acceptance
criterion claimed points at the automated test named in `acceptance.md` §3.2.
