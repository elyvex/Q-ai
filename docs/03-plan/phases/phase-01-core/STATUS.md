# Phase 1 — Implementation Status (as of 2026-09-15)

> **Purpose:** one page that answers *what is implemented, what remains, and what
> follows*. The authoritative, append-only ledgers are `done.md` (completion
> entries), `tasks.md` (task board) and `acceptance.md` (criteria); this file is
> a reconciled snapshot and is expected to be refreshed, not appended.
>
> **Caveat:** a concurrent Phase-2 writer was active in the same tree while this
> was written. Where the boards lag the code, this file says so explicitly.

## 0. Snapshot

| Dimension | State |
|---|---|
| Task rows (excluding 3 sequencing notes) | **50 ☑ / 2 ◐ / 13 ☐** of 65 |
| Acceptance criteria | **19 ◐ / 2 ☐** of 21 (none marked fully verified — rituals pending) |
| ADRs | **12 Accepted**, 2 Draft (0101, 0114) |
| Phase-1 migrations | **6 / 6** (`0007`–`0012`); workspace now at 16 (Phase-2 added `0013`–`0016`) |
| D1.14 documents | **5 / 5** published |
| Gate | `clippy -D warnings` clean · `cargo test --workspace` 135 suites ok · `arch-check` OK · `migrate-check` OK · `fmt` clean for Phase-1 files |

Remaining task IDs: `P1-X01..X05`, `P1-T01`, `P1-T02`, `P1-T03`, `P1-T26◐`,
`P1-T54`, `P1-T55`,
`P1-T56`, `P1-T58`, `P1-T60` (and `P1-T04◐`).

## 1. Implemented (by surface)

### 1.1 Canonical domain — `crates/quran-core` (Sprint 1.1, 9/9 ☑)
- Numbering newtypes, edition/enum vocabulary, `QuranEdition`/statistics,
  surah/ayah/segment/token structs.
- Reference grammar (parse/serialize) with a 331-case golden set and property
  tests (`roundtrip_parse_serialize`, `parser_never_panics_and_errors_are_coded`).
- `QuranQuotation` with a visibility-restricted constructor (invariant I6).
- Read views: `AyahView`, `AttributedTranslation`, `ContextView`, `ContextBoundary`.
- **Evidence:** `crates/quran-core/tests/reference_grammar.rs`,
  `fixtures/quran/golden/references.jsonl`.

### 1.2 Import & validation — `crates/quran-corpus` (Sprint 1.2, 16/17 ☑)
- Intermediate format + adapter trait with JSON and CSV adapters proving
  extensibility; JSON Schema at `docs/schemas/quran-edition-source.v1.schema.json`.
- Char-level tokenizer with separators and grapheme-mapped byte offsets
  (proptest: `reconstruct(tokens, separators) == text`, offsets in range).
- Frozen hashing recipes: `text_hash`, `structure_hash`, `token_order_hash`.
- Unicode auditor and validators **QV-001…028**; 16 adversarial fixtures reject
  with the specific rule id.
- 13-checkpoint importer (restart-is-resume, cancellation cleanup), edition differ.

### 1.3 Storage — `crates/storage`, `crates/storage-sqlite` (Sprint 1.2 ☑)
- Migrations `0007`–`0012`: editions, structure, divisions, translations,
  staging, validation. Canonical tables are insert-only (triggers).
- `QuranRepository` traits + SQLite impl: staging, atomic activation, reads,
  reports, citations, translations; approval rows; audit sequence queries.
- **Evidence:** `crates/storage-sqlite/tests/quran.rs`, `integrity_provenance.rs`.

### 1.4 Application services — `crates/application` (Sprints 1.2–1.3)
- `quran.import` job handler, approval-gated activation/rollback, hash-chained
  audit bridge.
- Deterministic `QuranReader` with a generation-keyed cache (no stale text after
  activation) and typed errors; context boundary/cap invariants now covered by a
  property test over every fixture ayah × spec matrix.
- Corpus doctor (`run_quran_checks`): 19 checks, read-only, `--deep` full scan;
  recomputed hashes asserted equal to import-time values.
- Translation import with structural attribution (principle 5) and structural
  alignment (aligned edition + per-passage ayah must exist; non-empty, unique;
  atomic on rejection).
- Word-gloss import (`import_glosses`, token-granularity alignment,
  `scholarly_annotation` provenance) with reader serving (`word_glosses` when
  requested, edition-scoped, dataset-attributed) and CLI (`quran gloss import`,
  `quran get --glosses`).

### 1.5 Surfaces — `crates/{tools,tool-registry,citations,cli,server}` (Sprints 1.3–1.4)
- **Tools:** `ToolResult`/`ReproducibilityData` contract + registry with
  `quran.get_ayah` and `quran.get_context`; typed no-fabrication errors.
- **Citations:** resolver with `QuotationVerdict` (ExactMatch / …Normalization /
  Mismatch / LocationNotFound / EditionNotFound / AccessDenied), persistence, and
  frozen deep-link/URN formats.
- **API v1** (`crates/server/src/api.rs`): envelope + `meta` + ETag +
  `Content-Language` + Diagnostic error body; `/api/v1/quran/…` routes; OpenAPI
  spec + route-coverage test; HTML debug reader at
  `/debug/read/{edition}/{surah}` (RTL, labelled, no persistence).
- **CLI** (`crates/cli`): every read verb (`get/context/surah/division/resolve`),
  lifecycle verbs (`import/validate/activate/rollback/diff/deprecate/edition/
  translation/gloss/hashes`), `--json` on reads, `--yes` on destructive verbs, Phase-0
  exit-code table; trycmd snapshot suite with RTL assertions and error exits.
- **Doctor:** `qai doctor --quran` (19 checks, read-only, `--deep`) and a single
  merged `--json` document that validates against `doctor.v1.schema.json`.

### 1.6 Docs (D1.14) — 5/5 published
`docs/07-technical/quran-corpus-architecture.md`, `quran-adapter-authoring.md`,
`quran-citation-spec.md`; `docs/10-operations/quran-import-runbook.md`,
`quran-rollback-runbook.md`. Handoff: `docs/plans/handoff-p1-to-p2.md`.

## 2. Acceptance criteria

**Partial — automated-green, ritual/live verification pending (19):**
AC-P1-02, 03, 04, 05, 06, 07, 08, 09, 10, 11, 12, 13, 14, 15, 16, 17, 19, 20, 21.
AC-P1-09 joined on 2026-09-15 via the activation-rejection state test.

**Not started (2):** AC-P1-01 (dataset/license sign-off), AC-P1-18
(debug-reader web font).

## 3. Remaining work

### 3.1 Blocked on owner/editorial decisions (cannot be done by an agent)
| Task | Blocked on |
|---|---|
| `P1-X01`, `P1-T01`, `P1-T02`, `P1-T04` (ADR-0101) | Licensed dataset + named editorial reviewer |
| `P1-X02`, `P1-T55` | Named reviewer signs sampled text (`verified_by`) |
| `P1-X03`, `P1-T03`, `P1-T26` (ADR-0114) | Reference corpus + comparison procedure + sign-off (QV-015 currently a recorded skip) |
| `P1-X04` (morphology data), `P1-X05` (normalization linguist) | Phase-2 upstream decisions |

### 3.2 Engineering work still open
| Task | What remains |
|---|---|
| `P1-T54` | Debug reader has RTL + label + CSS Arabic font stack + per-ayah markers + HTML escaping (tested); still ☐ pending a bundled-`@font-face` font/licensing choice, which needs an owner decision |
| `P1-T56` | Golden-set expansion to §5.2 edge cases — **blocked on a real dataset** (ADR-0101); must not be filled with fabricated scripture |
| `P1-T58` | Full-corpus soak (import → validate → activate → 10k lookups → `doctor --deep`) — needs a standard edition to be meaningful |
| `P1-T60` | Exit-gate review + handoff sign-off |

### 3.3 Board/ritual debt
- `acceptance.md`: AC-P1-02/03/05/06/07/10/11 are ◐ with evidence recorded;
  remaining ritual steps belong to the exit ritual (P1-T60), not board debt.
- `done.md` §4: ADR table still shows 0111/0112 as Draft (header: 10 Accepted);
  reality is 12 Accepted with 0101/0114 Draft pending owners — needs sync.
- `tasks.md`: T39/T40 reconciled (☑); T54 correctly ☐ pending the font-asset
  owner decision.

## 4. Follow-ups & deviations

### Recorded deviations (must be honoured by Phase 2)
- **DEV-01** — no XML adapter; JSON + CSV prove adapter extensibility.
- **DEV-02** — Phase-1 migrations are `0007`–`0012` (not plan `0010`–`0015`);
  `migrate-check` requires contiguity. Phase 2 continues from `0013`.
- **DEV-03** — division numbers are globally unique per kind (`ruku`/`rub`
  cumulative); per-surah ruku is available via the ayah `ruku` column.

### Open owner decisions
| ID | Item |
|---|---|
| **OWN-01** | ADR-0101 dataset/license/reviewer unresolved; engineering runs on the synthetic `test-edition-min` fixture |
| **OWN-02** | ADR-0114 reference corpus/procedure/sign-off unresolved; QV-015 skip-when-unconfigured |
| **OWN-03** | Estimate gap 82 ed vs 131.0 ed summed; no silent compression |
| **OWN-04** | axum + tower-http adopted provisionally for API v1; ADR to ratify |
| **OWN-05** | Phase-0 exit discrepancy between `status.md` and the build prompt |
| **OWN-06** | `server` reaches `storage` + `tools` directly; allowlisted to keep `arch-check` green, should be routed through `application` and tightened (Phase 3) |

### Known limitations / risks
- **Flaky test:** `jobs::worker::tests::retries_then_succeeds` passes in isolation
  but intermittently returns `Idle` under the fully-parallel workspace run
  (queue-timing race). De-flake before the exit ritual.
- **Standard-edition timings unverified:** `doctor --quran --deep` < 30 s and the
  full-corpus soak both need a real dataset (ADR-0101).
- **ADR-0111/0112 accepted:** deep-link and translation-alignment contracts were
  accepted 2026-09-15 on their own shipped-code conditions (P1-T46/T47 + citation
  endpoint; P1-T36/T37).
- **Board drift:** several tasks/criteria are more complete than `tasks.md` /
  `acceptance.md` indicate; sync before the exit gate walks the ledgers.

## 5. Verification

Last full green run (2026-09-15):
`cargo clippy --workspace --all-targets -- -D warnings` clean ·
`cargo test --workspace` 135 suites ok ·
`cargo xtask arch-check` OK · `cargo xtask migrate-check` OK · `fmt` clean for
Phase-1 files.

Re-run `cargo xtask ci` before relying on this snapshot: the tree was shared with
an active Phase-2 writer, and Phase-2 migrations (`0013`–`0014`) already extend
the schema version.

Targeted verification (2026-09-15, T36):
`cargo test -p application --test quran_translation` 9/9 green ·
`cargo test -p cli --test quran` 2/2 (trycmd) green ·
`rustfmt --check` clean on the T36 test file.
Full-workspace `clippy -D warnings` was not re-run because a concurrent Phase-2
writer had uncommitted `crates/application/src/quran_index.rs` changes in the
tree.

Targeted verification (2026-09-15, T57):
`cargo test -p application --test quran_reader context_invariants` 2/2 green
(exhaustive matrix + 512-case deterministic proptest) ·
`cargo clippy -p application --test quran_reader` shows no `quran_reader` lints ·
`rustfmt --check` clean on the T57 test file.

Targeted verification (2026-09-15, T53, docs-only):
ADR-0111 acceptance condition (P1-T46/T47 + citation endpoint) confirmed by
inspection — `crates/citations` resolver verdicts + `deep_link`/`citation_urn`,
`GET /api/v1/quran/citations/{id}` in `crates/server` with route coverage.

Targeted verification (2026-09-15, T42, docs-only):
ADR-0112 acceptance condition (P1-T36/T37) confirmed — `import_translations`
structural alignment suite 9/9 green, `AttributedTranslation` type guards in
`quran-core`, read-time alignment re-check in the reader.

Targeted verification (2026-09-15, T39):
`cargo test -p server` green (3 unit + 8 integration, incl. envelope/ETag/
diagnostic/route coverage).

Targeted verification (2026-09-15, T40):
`cargo test -p server --test api` 9/9 green in an isolated worktree at the last
green base (incl. the new `$ref`-resolvability + JSON-schema-coverage test);
spec JSON validated (26 refs resolve, 24 JSON responses carry schemas).
The shared tree could not run it directly because a concurrent Phase-2 writer
had broken `application` (E0004 in `quran_index.rs`) at HEAD.

Targeted verification (2026-09-15, T54 progress, not completion):
`cargo test -p server` green (incl. extended debug-reader structure asserts +
`escape_html` unit test); rustfmt clean. The reader now declares an Arabic
font stack, marks each ayah, and escapes text. T54 stays open: a bundled
`@font-face` needs a named font + licensing sign-off (owner decision).

Targeted verification (2026-09-15, T38):
storage `quran.rs` 9/9 · application `quran_gloss` 11/11 + `quran_reader`
12/12 (incl. gloss serving) + `quran_import`/`quran_tools`/`quran_translation`
green · CLI trycmd 2/2 (incl. gloss import + `get --glosses`) · rustfmt clean ·
clippy shows no lints in T38 files. Run in an isolated worktree at HEAD
because the shared tree's `quran-search` had concurrent uncommitted breakage.

Targeted verification (2026-09-15, AC-P1-09 hardening):
`cargo test -p application --test quran_import` 9/9 green in an isolated
worktree (incl. new `rejected_activations_leave_canonical_state_untouched`);
rustfmt clean. AC-P1-09 flipped to ◐ (code review + ritual pending).

Targeted verification (2026-09-16, gates):
`cargo xtask arch-check` OK · `cargo xtask migrate-check` OK (16 migrations
ordered, checksums stable). `cargo fmt --all -- --check` still shows a pending
diff in the concurrent writer's `crates/application/src/lib.rs` (module order;
not a Phase-1 file, left for its owner). Full `cargo test --workspace` not
re-run: shared tree has uncommitted Phase-2 churn.
