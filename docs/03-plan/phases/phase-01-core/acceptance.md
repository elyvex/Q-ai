# Phase 1 — Acceptance Criteria & Exit Gate

**Phase:** P1 — Canonical Quran Core
**Source:** `plan.md` §9 (Acceptance Criteria), §5 (Corpus Integrity Test Suite), §4 (Deliverables)
**Plan criteria:** 13 (AC-P1-01 … AC-P1-13)
**Supporting criteria:** 8 (AC-P1-14 … AC-P1-21, derived from D1.6–D1.14)
**Gate owner:** task P1-T60
**Status:** 🔴 0 / 21 verified

**Status legend:** ☐ Not verified · ◐ Partially verified · ✗ Failed · ☑ Verified

> **Verification rule:** an AC is ☑ only when an **automated test or scripted check** proves it
> and the evidence artifact (CI run URL, test path, or recording) is recorded in the Evidence
> column and appended to `done.md`. "It works on my machine" is not verification. The
> exit-gate ritual (§6) additionally requires a **recorded live walkthrough** by a reviewer who
> is not the implementer.

---

## 1. Acceptance Criteria

### 1.1 Data & decisions (blocking everything)

| ID | Criterion | Verification | Blocks | Evidence | Status |
|---|---|---|---|---|---|
| **AC-P1-01** | ADR-0101 is `Accepted` with a licensed, editorially signed-off edition (or a documented user-supplied policy that names the fallback fixture) | ADR review + `verified_by` recorded | D1.3, all data | | ☐ |

> AC-P1-01 is the Phase-1 hard gate. If no dataset can be bundled, the fallback **must be
> written in ADR-0101** (ship `test-edition-min`; user imports their own approved edition) — an
> undocumented absence of data is a gate failure, not an acceptable ambiguity.

### 1.2 Import & validation (the integrity core)

| ID | Criterion | Verification | Blocks | Evidence | Status |
|---|---|---|---|---|---|
| **AC-P1-02** | `qai quran import <manifest>` imports the edition to `Staged` and produces a `ValidationReport` with **zero `Fatal` findings** | Scripted run against `test-edition-min` + the licensed edition | D1.3, D1.4 | | ☐ |
| **AC-P1-03** | Import is **not** activatable by the importer; activation requires `qai quran activate` with a human approval, producing an `approvals` row **and** an audit event | Negative + positive test | D1.3, D1.9 | | ☐ |
| **AC-P1-04** | Validation rules QV-001…QV-028 are all implemented, and **each of the 16 adversarial fixtures is rejected with the specific expected rule ID** | Adversarial suite (`fixtures/quran/adversarial/`) | D1.4, D1.13 | `crates/quran-corpus/tests/adversarial.rs` 6/6 green; ritual pending | ◐ |
| **AC-P1-05** | For **every ayah** in the corpus, `reconstruct(tokens, separators)` equals the stored text **byte-for-byte** | Full-corpus test (property + explicit) | D1.3, D1.13 | | ☐ |
| **AC-P1-06** | For **every token**, `ayah.text[byte_start..byte_end] == surface` | Full-corpus test | D1.3, D1.13 | | ☐ |
| **AC-P1-07** | Recomputed `text_hash`, `structure_hash`, `token_order_hash` from DB rows match the values stored at import | `qai doctor --quran --deep` | D1.4, D1.11 | | ☐ |

> AC-P1-04 is deliberately specific. "The import failed" does not satisfy it: each adversarial
> fixture must fail with the **documented rule id** (e.g. `nfd_text` → QV-007, `bidi_override` →
> QV-008, `shuffled_tokens` → QV-010/QV-024). A validator that rejects everything with `QV-000`
> passes AC-P1-02 and fails AC-P1-04.
>
> AC-P1-05/06 are the losslessness guarantees. If either fails for a single ayah, the corpus is
> not canonical and Phase 1 does not exit — no partial pass, no whitelist without an ADR.

### 1.3 Immutability & atomicity

| ID | Criterion | Verification | Blocks | Evidence | Status |
|---|---|---|---|---|---|
| **AC-P1-08** | `UPDATE`/`DELETE` on `quran_ayahs` / `quran_tokens` aborts with the documented error codes (`QAI-QUR-0001`…`0005`) | Raw-SQL trigger tests | D1.5, D1.13 | `crates/storage-sqlite/tests/quran.rs` trigger/schema tests green; ritual pending | ◐ |
| **AC-P1-09** | No public API exists to write canonical ayah rows without a `CanonicalChangeSession` derived from an `ApprovalToken` | API-surface test + code review | D1.1, D1.3 | | ☐ |
| **AC-P1-10** | Killing the import process at each of the **13 checkpoints** leaves the active edition **unchanged in all 13 cases**, and `qai job retry` completes the import correctly | Crash matrix | D1.3 | | ☐ |
| **AC-P1-11** | Cancelling an import removes all `quran_stg_*` rows for that run **and** records the cancellation | Cancellation test | D1.3 | | ☐ |

> AC-P1-08/09 together implement I1. The DB trigger is the last line of defence, but it is not
> sufficient: AC-P1-09 closes the API path so a future developer cannot write canonical rows
> "temporarily" through a repository without a token. Both must hold.
>
> AC-P1-10 is the §85 guarantee: a crash mid-import can never expose a partial corpus. The
> active pointer is unchanged in every one of the 13 cases. This is a *negative* criterion — a
> green import after crash+retry does not satisfy it unless the intermediate state was also
> proven `Staged`, never `Active`.

### 1.4 Addressing (frozen public surface)

| ID | Criterion | Verification | Blocks | Evidence | Status |
|---|---|---|---|---|---|
| **AC-P1-12** | The reference grammar parses all **300 golden references** correctly and returns **coded errors (never panics)** for all malformed inputs | Golden set + fuzz/property test | D1.1 | `crates/quran-core/tests/reference_grammar.rs` (331 cases green; ritual pending) | ◐ |
| **AC-P1-13** | `parse(serialize(ref)) == ref` for **all** reference variants | Property test | D1.1 | `roundtrip_parse_serialize` proptest green | ◐ |

> AC-P1-12/13 freeze ADR-0102. After the grammar is frozen, changing it breaks every stored
> citation, so the golden set and round-trip property must be green **before** the freeze is
> declared. The malformed-input half of AC-P1-12 is a panic-safety guarantee: the parser is
> reachable from user input (`qai quran resolve "…"`) and must always return a `QAI-QUR-01xx`
> error.

### 1.5 Non-negotiable integrity (restated from Phase 0)

| ID | Criterion | Verification | Blocks | Evidence | Status |
|---|---|---|---|---|---|
| — | I2: no canonical lookup requires an LLM/vector store | `cargo xtask arch-check` forbids `llm`/`embeddings`/`retrieval` edges into `quran-core`/`quran-corpus` | D1.1, D1.6 | | ☐ |
| — | I6: every quotation carries edition + version + hash | `QuranQuotation` constructor visibility test; `citations` suite | D1.1, D1.9 | | ☐ |

### 1.6 Supporting exit criteria (derived from D1.6–D1.14)

These are not enumerated in `plan.md` §9, but each corresponds to a deliverable the plan
describes and must ship for the phase to be usable. They are separated so their provenance is
unambiguous.

| ID | Criterion | Verification | Blocks | Evidence | Status |
|---|---|---|---|---|---|
| **AC-P1-14** | Quran read API v1 returns the stable response envelope; `meta` carries edition slug/version/`text_hash`/script/riwayah/numbering + `corpus_generation` + `canonical_reference` + `deep_link` + reproducibility checksum; `ETag` derives from `(text_hash, corpus_generation)`; Arabic responses are UTF-8 with `Content-Language`; errors use the Phase-0 `Diagnostic` body | Handler contract + snapshot tests | D1.7 | `server/tests/api.rs` envelope + ETag + OpenAPI-coverage green; ritual pending | ◐ |
| **AC-P1-15** | `quran.get_ayah` and `quran.get_context` conform to the §12 `ToolResult` contract (`tool_name`, `tool_version`, `query`, `normalization_rules`, `edition_id`, `edition_version`, `results`, `canonical_references`, `analysis_sources`, `warnings`, `execution_time_ms`, `reproducibility`); the reproducibility checksum is deterministic; **no-fabrication** test returns a typed error for a non-existent reference | Tool conformance + no-fabrication suite | D1.8, D1.13 | | `quran_tools.rs` green; ritual pending | ◐ |
| **AC-P1-16** | CLI follows the Phase-0 conventions (verbs after nouns; `--json` on every read command; destructive commands confirm unless `--yes`; documented exit-code table); snapshots cover `qai quran get/context/surah/division/resolve/edition/import/validate/activate/diff/rollback` incl. RTL/Arabic terminal output sanity | CLI snapshot suite (trycmd) | D1.10, D1.13 | `cli/tests/quran/read_flow.trycmd` green (RTL + provenance + translations + exit 5/6); `diff`/`rollback`/`context`/`surah` snapshots pending | ◐ |
| **AC-P1-17** | `qai doctor --quran` runs all 19 checks (each emitting `Pass/Warn/Fail/Skipped` + remedy + next command), opens the DB **read-only**, `--deep` completes full-corpus hash + token round-trip in **< 30 s** for a standard edition, and `--json` validates against `docs/schemas/doctor.v1.schema.json` | Read-only + schema + timing test | D1.11 | 19 checks + read-only open live-verified; JSON matches schema by shape; `--deep` timing + schema-validation test pending | ◐ |
| **AC-P1-18** | Debug reader at `/debug/read/{edition}/{surah}` renders ayahs with correct RTL and a web font, is labelled "debug view", has no persistence, and is excluded from product navigation | Visual + route test | D1.12 | | ☐ |
| **AC-P1-19** | After activating a new edition version, no stale text is ever served: cache keyed by `(edition_id, version, corpus_generation, ref, options_hash)` and invalidated wholesale on generation change | Cache-consistency test | D1.6 | `cache_serves_no_stale_text_after_activation` green; ritual pending | ◐ |
| **AC-P1-20** | ADR-0101…ADR-0114 exist, are `Accepted`, and each contains **all §48 fields** including **religious-source** and **licensing** implications; the five D1.14 docs are published | ADR lint + doc review checklist | D1.14 | | ☐ |
| **AC-P1-21** | `CitationResolver::resolve` checks source exists, location resolves, quotation matches, edition exists, permission ok, and hash+version available; `verify_quotation` returns `ExactMatch` / `MatchAfterWhitespaceNormalization` / `MatchAfterDeclaredNormalization{rules}` / `Mismatch{first_difference_at, expected_hash}` / `LocationNotFound` / `EditionNotFound` / `AccessDenied`; resolved citations persist with `content_hash` + `ingestion_version` and re-verify later | Resolver suite + persistence test | D1.9 | | resolver + persistence suites green; ritual pending | ◐ |

> A `Mismatch` or `LocationNotFound` verdict must be wired into a **hard failure** on any answer
> path. This is the mechanism Phase 9's citation verifier reuses; Phase 1 must not leave it
> advisory.

---

## 2. Corpus Integrity Test Suite (D1.13)

The permanent regression net (PRD §35.1, §46, §58). Every subsection below must exist and be
green; a missing suite fails the gate even if the mapped AC appears satisfied by other means.

### 2.1 Golden fixtures (`fixtures/quran/`)

| Fixture | Purpose |
|---|---|
| `golden/ayah_texts.jsonl` | ~200 curated ayahs (incl. every §2.2 edge case) with expected exact text + hash |
| `golden/edition_hashes.json` | Expected `text_hash`, `structure_hash`, `token_order_hash` per fixture edition |
| `golden/references.jsonl` | ~300 reference strings → expected parse result or expected error code (AC-P1-12) |
| `golden/tokens.jsonl` | Expected token surfaces + offsets for selected ayahs |
| `golden/divisions.json` | Expected juz/hizb/page boundaries for the fixture edition |
| `test-edition-min/` | Small public-domain-safe synthetic edition (structurally valid, 5 surahs) for fast tests |
| `adversarial/` | Deliberately broken editions (§2.3) |

### 2.2 Edge cases that must be in the golden set

- Al-Fatihah 1:1 (basmala counted as an ayah)
- At-Tawbah 9:1 (no basmala)
- An-Naml 27:30 (basmala inside an ayah)
- Al-Baqarah 2:255 (long ayah)
- Al-Kawthar 108 (short surah)
- Al-Baqarah 2:1 (muqattaʿat / disjointed letters)
- Ar-Rahman refrain (legitimately repeated ayah text)
- Ayahs containing sajdah markers
- Ayahs spanning a page boundary
- The first and last ayah of the corpus
- An ayah containing a pause mark (waqf) sign
- An ayah with a superscript alef (U+0670) and small high signs
- The longest and shortest tokens in the edition
- An ayah where two tokens are separated by a mark rather than a plain space

### 2.3 Adversarial import corpora — each must be rejected with the correct rule ID (AC-P1-04)

| Fixture | Injected fault | Expected |
|---|---|---|
| `missing_ayah` | one ayah removed | QV-005 Fatal |
| `duplicate_ayah_id` | duplicated identifier | QV-005 Fatal |
| `extra_surah` | 115 surahs | QV-001 Fatal |
| `wrong_ayah_count` | surah count mismatch | QV-003 Fatal |
| `nfd_text` | text in NFD while declaring NFC | QV-007 Fatal |
| `bom_prefix` | BOM inside a text field | QV-008 Fatal |
| `bidi_override` | U+202E injected | QV-008 Fatal |
| `zero_width` | ZWJ/ZWNJ inserted mid-token | QV-008 / QV-011 Fatal |
| `latin_homoglyph` | Latin `ا`-lookalike | QV-009 Error |
| `bad_offsets` | token offsets off by one | QV-012 Fatal |
| `shuffled_tokens` | token positions permuted | QV-010 / QV-024 Fatal |
| `truncated_ayah` | ayah text silently truncated | QV-011 / QV-015 Fatal |
| `hash_mismatch` | manifest hash altered | QV-013 Fatal |
| `page_regression` | page numbers decrease | QV-018 Error |
| `missing_juz` | juz gap | QV-016 Error |
| `translation_as_canonical` | translation submitted as an Arabic edition | QV-027 Error + type-level rejection |

**Severity semantics.** `Fatal` ⇒ import cannot reach `Staged`. `Error` ⇒ can stage, cannot be
approved without an explicit override recorded in the approval record. `Warning`/`Info` ⇒ shown
in the report. The validator runs **all** rules and lists every failure (no fail-fast), because
partial reports are useless for editorial review.

### 2.4 Property tests

- For every ayah: `reconstruct(tokens, separators) == text` (byte equality). → AC-P1-05
- For every token: `text[byte_start..byte_end] == surface`. → AC-P1-06
- For every valid reference: `parse(serialize(ref)) == ref` (round trip). → AC-P1-13
- For random `(before, after, boundary)`: context never crosses the declared boundary and
  never exceeds `max_ayahs`.
- For random reference strings: the parser never panics and always returns a coded error.
  → AC-P1-12
- Hash stability: recomputing `text_hash` from DB rows equals the import-time value. → AC-P1-07

### 2.5 Immutability tests

- `UPDATE quran_ayahs SET text = …` → aborts via trigger with `QAI-QUR-0001`.
- `DELETE FROM quran_ayahs` → aborts (`QAI-QUR-0003`).
- `UPDATE`/`DELETE` on `quran_tokens` → aborts (`QAI-QUR-0004`/`0005`).
- Writing an ayah without a `CanonicalChangeSession` → compile error (no public API) **plus** a
  runtime repository test proving the guard.
- A `CanonicalChangeRequest` missing a difference report, checksum result, or approver → rejected.
- Activating a `Staged` version whose validation report has any `Fatal` → rejected.
- Simulated crash (process kill) at each of the 13 importer checkpoints → the active edition is
  unchanged in all 13 cases, and resume completes correctly.

---

## 3. Definition of Done

### 3.1 Per-deliverable checklist (PRD §58, §93)

Every Phase-1 deliverable must satisfy **all** of the following. This is the PR template.

- [ ] Implemented behind an explicit interface; no cross-layer coupling (`arch-check` green)
- [ ] Unit tests + property tests where the state space warrants
- [ ] Integration tests against **real** SQLite
- [ ] Typed errors implementing `Diagnostic` with remedy + next command (codes `QAI-QUR-*`)
- [ ] Observable: tracing spans + metrics registered in the catalog
- [ ] Documented in `docs/architecture/` and referenced from the crate's `//!` docs
- [ ] Configuration validated at load **and** on change
- [ ] Cancellation and timeouts implemented where the operation is long-running (the importer)
- [ ] Secrets redacted everywhere (Phase-0 leak suite still green)
- [ ] Provenance and audit events recorded for **every** mutation
- [ ] Schema versions recorded; migration reversible or explicitly forward-only **with rationale**
- [ ] Access/policy checks present (deny-by-default) even in single-user mode
- [ ] Failure recovery tested (crash, retry, cancelled import, interrupted job)
- [ ] `cargo fmt`, `cargo clippy -D warnings`, `cargo test`, `cargo deny` all green

### 3.2 Required test suites

| Suite | Proves | AC |
|---|---|---|
| `tests/quran/reference_grammar.rs` | 300 golden refs parse; malformed → coded error; round-trip | 12, 13 |
| `tests/quran/roundtrip.rs` | Full-corpus token/separator reconstruction byte-equality; token offsets match | 05, 06 |
| `tests/quran/immutability.rs` | Raw-SQL triggers abort with `QAI-QUR-0001…0005`; API guard rejects token-less writes | 08, 09 |
| `tests/quran/adversarial.rs` | All 16 fixtures rejected with the correct rule id | 04 |
| `tests/quran/checkpoints.rs` | 13-case crash matrix; cancellation cleans staging; resume | 10, 11 |
| `tests/quran/hashes.rs` | Recomputed hashes equal import-time hashes; hash stability | 07 |
| `tests/quran/divisions.rs` | juz/hizb/rub/manzil/page/ruku/sajdah lookups correct; QV-016/018 consistency | 07 |
| `tests/quran/context.rs` | Boundary + `max_ayahs` never violated (property); context by canonical structure | 14 |
| `tests/quran/cache.rs` | No stale text after activation; generation-keyed invalidation | 19 |
| `tests/quran/translations.rs` | Attribution required; translation never occupies `canonical`; alignment validated | 21 |
| `tests/quran/citations.rs` | Resolver verdicts + persisted citation re-verification | 21 |
| `tests/quran/tools.rs` | §12 contract conformance; deterministic checksum; no fabrication | 15 |
| `tests/cli/quran.trycmd` | Human + `--json` snapshots for every `qai quran` command; RTL sanity | 16 |
| `tests/doctor/quran.rs` | 19 checks, read-only enforcement, `--deep` timing, JSON schema | 17 |
| `tests/api/quran_v1.rs` | Envelope, meta, ETag, content-language, error body | 14 |

### 3.3 CI jobs that must be green

```text
check        -> cargo fmt --check; cargo clippy --all-targets -- -D warnings
arch         -> cargo xtask arch-check; cargo xtask migrate-check   (allowlist extended for quran-core/corpus/citations)
test-linux   -> cargo test --workspace --all-features
test-macos   -> cargo test --workspace
test-windows -> cargo test --workspace
deny         -> cargo deny check advisories bans licenses sources
schemas      -> cargo xtask gen-schema && git diff --exit-code docs/schemas/
doctor       -> qai doctor --quran --json | validate against docs/schemas/doctor.v1.schema.json
corpus       -> corpus integrity suite (golden + adversarial + property + immutability)
coverage     -> cargo llvm-cov; gates per §4
msrv         -> build with pinned MSRV
```

---

## 4. Coverage Gates

Proposed floors (the plan does not state numeric coverage gates; these are derived from the
Phase-0 policy to keep the standard consistent). Owner to ratify before Sprint 1.2.

| Crates / modules | Line coverage | Status |
|---|---|---|
| `quran-core` (incl. reference grammar) | **≥ 90%** | ☐ |
| `quran-corpus::validation`, `::tokenize`, `::hashing` | **≥ 90%** | ☐ |
| `quran-corpus::adapters`, `::differ` | **≥ 80%** | ☐ |
| `citations` | **≥ 85%** | ☐ |
| `storage-sqlite` (quran repositories) | **≥ 80%** | ☐ |
| `cli`, `server` (quran surfaces) | Smoke + snapshot; **no numeric gate** | ☐ |

Coverage is a floor, not a target — it does not substitute for the named suites in §3.2.
The canonical integrity suites (round-trip, immutability, adversarial, hash stability) must be
green even if coverage is above the floor.

---

## 5. Exit-Gate Ritual

A **recorded walkthrough** in which a reviewer — not the implementer — performs the following
**live on a clean machine**:

| Order | AC | Demonstration |
|---|---|---|
| 1 | AC-P1-02 | Import `test-edition-min` (and the licensed edition) to `Staged`; show the `ValidationReport` with zero `Fatal` |
| 2 | AC-P1-03 | Attempt to activate from the importer/job path (must fail); run `qai quran activate` with approval and show the `approvals` row + audit event |
| 3 | AC-P1-05/06 | Run the full-corpus round-trip check; show byte-equality for a deliberately awkward ayah (waqf mark, page-spanning) |
| 4 | AC-P1-07 | Run `qai doctor --quran --deep`; show recomputed `text_hash`/`structure_hash`/`token_order_hash` match import-time values |
| 5 | AC-P1-08 | From raw SQL, attempt `UPDATE`/`DELETE` on `quran_ayahs`/`quran_tokens`; show the coded abort |
| 6 | AC-P1-04 | Run the adversarial suite; show three fixtures failing with the **specific** expected rule id |
| 7 | AC-P1-10 | `SIGKILL` the import at each of the 13 checkpoints; show the active edition is unchanged every time and `qai job retry` completes |
| 8 | AC-P1-11 | Cancel a running import; show all `quran_stg_*` rows for the run are gone and the cancellation is recorded |
| 9 | AC-P1-12 | Feed malformed references to `qai quran resolve`; show coded errors and no panic |

Recording is archived and linked from `done.md`. The gate is **not** closed by a green CI run
alone — these are the criteria most likely to pass in CI while being wrong in practice.

---

## 6. Sign-Off

| Gate | Requirement | Owner | Date | Status |
|---|---|---|---|---|
| All 13 plan criteria verified | §1.1–1.4 fully ☑ | | | ☐ |
| All 8 supporting criteria verified | §1.6 fully ☑ | | | ☐ |
| Coverage gates met | §4 | | | ☐ |
| DoD satisfied per deliverable | §3.1 × 14 deliverables | | | ☐ |
| All required suites green | §3.2 | | | ☐ |
| 14 ADRs accepted, §48-complete | ADR-0101…0114 | | | ☐ |
| 6 migrations applied & checksummed | `migrations/sqlite/` | | | ☐ |
| Editorial sign-off recorded | P1-T55 (`verified_by`) | | | ☐ |
| Exit-gate ritual recorded | §5 | | | ☐ |
| Handoff doc published | `docs/plans/handoff-p1-to-p2.md` (P1-T60) | | | ☐ |
| Swimlane X decisions owned & open | `tasks.md` §1 — P1-X01…X05 | | | ☐ |
| **Phase 1 accepted → Phase 2 unblocked** | All rows above ☑ | | | ☐ |

---

## 7. Handoff Assets Phase 2 Must Not Re-Invent

Verified as part of P1-T60. Phase 2 inherits and must build on these; re-deriving any of them
invalidates the Phase-1 hashes.

| Asset | Location | Phase-2 usage |
|---|---|---|
| Canonical ayah/token rows + insert-only triggers | `quran_ayahs`, `quran_tokens`, `quran_token_separators` | Search indexes are built **over** these; QV-028 re-reads and re-hashes after every index build |
| `text_hash` / `structure_hash` / `token_order_hash` recipe (ADR-0108) | `quran-corpus::hashing` | Every derived index records the corpus generation + hashes it was built from |
| `corpus_generation` counter | `quran_active_edition` | Index staleness detection — never re-derive freshness by timestamps |
| Surface tokens + offsets | `quran_tokens` | Normalization/morphology attach to token positions; never re-tokenize the canonical text |
| Reference grammar + serializer (ADR-0102) | `quran-core::reference` | Search results and citations use the same frozen addressing |
| `QuranQuotation` + `Attribution` types | `quran-core`, `citations` | Search hits are returned as quotations, not bare strings |
| `AyahView` / `AttributedTranslation` + principle-5 guard | `quran-core` | Translation comparison tools reuse the type-level separation |
| `ToolResult` contract + reproducibility checksum (§12) | `tools` | Phase-2 search tools conform to the same envelope and checksum inputs |
| Validator + `ValidationReport` | `quran-corpus::validation` | Phase-2 derived annotations are validated by the same machinery |
| Import/rollback runbooks + D1.14 docs | `docs/` | New adapters and re-index procedures follow the documented process |
| Error namespace `QAI-QUR-*` | `domain::diagnostic` registry | Search/normalization codes use their reserved namespaces (`QAI-NORM`, `QAI-IDX`) |

**Handoff document:** `docs/plans/handoff-p1-to-p2.md`, produced by task P1-T60, listing the
above plus known limitations and any deferred items **with owners**.

> Phase 2 must **not** invent a second tokenizer, a second reference grammar, or a second
> hashing scheme. Any proposal to do so requires an ADR that explains why the Phase-1 artifact
> cannot be reused and how existing citations/hashes survive the change.
