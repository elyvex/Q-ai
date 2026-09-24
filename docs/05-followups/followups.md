# Follow-ups

Dated, agent-actionable items that are not acceptance criteria, not
owner-only decisions (those live in `decisions-needed.md`), and not
phase-scoped technical questions (those live in `open-questions.md`).
Status key: 🔴 open · 🟢 done (with date + where recorded).

---

## FU-DOC-01 — `docs/03-plan/current-plan.md` is empty (0 bytes)
- **Status:** 🟢 done 2026-09-24 (active-phase pointer written; Phase 2 active,
  Phase-4 code-ahead-of-board + P0-T56 ◐ + owner-gate pointers recorded)
- **Found:** 2026-09-16 (repo discovery sweep)
- **Context:** `AGENTS.md` > "Before starting work" step 3 and `README.md`
  both instruct agents to read `docs/03-plan/current-plan.md` for the
  active phase. The file exists but is empty, so every new session must
  rediscover the active phase from `docs/06-progress/status.md` instead.
- **Next action:** fill it with a one-paragraph active-phase pointer
  (Phase 2, Sprint 2.3, per `status.md`) or delete it and remove the two
  references. Either is a 5-minute docs edit; do not leave it empty.
- **Scope:** any DOC pass. No code impact.

## FU-DOC-02 — `docs/04-tasks/` required by workflow does not exist
- **Status:** 🔴 open — partially addressed 2026-09-24: `docs/04-tasks/README.md`
  + `active/README.md` + `completed/README.md` created, declaring the phase
  boards the system of record and this directory an index.
- **Remaining:** reconcile the workflow text (`AGENTS.md` "Before starting work",
  `.agent/instructions.md` §3, `.agent/workflow.md`) to name the phase
  `tasks.md` boards canonical instead of implying `docs/04-tasks/` is primary.
- **Found:** 2026-09-16 (repo discovery sweep)
- **Context:** `AGENTS.md` > "Agent Workflow" §2 and `.agent/instructions.md`
  §3 require every work item to map to a `TASK-nnn-slug.md` under
  `docs/04-tasks/active/`, moved to `completed/` on completion.
  The directory does not exist; phase work is tracked in
  `docs/03-plan/phases/*/tasks.md` instead. No session is violating the
  rule on purpose — the location it names is absent.
- **Next action:** either create `docs/04-tasks/{active,completed}/` and
  migrate or mirror active items, or update `AGENTS.md` +
  `.agent/instructions.md` to name the phase `tasks.md` files as the
  system of record. Decide once; mixed conventions will rot.
- **Scope:** DOC + workflow decision. No code impact.

## FU-SPEC-01 — `specs/001-redaction-hardening/tasks.md`: 28/28 unchecked vs landed implementation
- **Status:** 🟢 done 2026-09-24 — rechecked: T001–T023, T025–T028 are `[x]`
  (the "28/28 unchecked" report of 2026-09-16 is stale); only T024 (full gate
  sweep) remains `[ ]` with partial evidence in its sync note S3. No action
  beyond T024's own gate run.
- **Found:** 2026-09-16 (spec reconciliation)
- **Context:** All T001–T028 checkboxes are `[ ]`, yet the implementation
  is committed (`domain::redaction`, `RedactingWriter`, `InitOptions`,
  diagnostic render-time scrubbing, CLI redact-then-print, allowlist line)
  and its 9 sentinel tests pass (`cargo test -p testkit --test secret_leak`
  9/9 green, verified 2026-09-16). Spec/code drift in the other direction:
  the code is ahead of its spec paperwork.
- **Next action (owning/redaction session):** flip each checkbox with the
  landing commit, or record a deviation note if any listed task was
  deliberately superseded. Then mark P0-T16 evidence complete in
  `docs/03-plan/phases/phase-00-foundation/tasks.md` (already ☑) and
  `done.md`.
- **Scope:** redaction session only. No code changes expected.

## FU-TEST-01 — DEBUG leftovers in `crates/testkit/tests/secret_leak.rs`
- **Status:** 🟢 done 2026-09-24 — the three `println!("DEBUG …")` lines are
  gone (only a `// DEBUG:` comment remained; rewritten as a plain comment),
  file ends with a trailing newline, `cargo fmt --check` clean,
  `cargo test -p testkit --test secret_leak` 9/9 green.
- **Found:** 2026-09-16 (working-tree review)
- **Context:** The uncommitted working tree contains three
  `println!("DEBUG …")` lines in
  `sentinel_key_value_pairs_scrubbed_from_free_text` plus a missing
  trailing newline at EOF. Harmless to test outcomes, but `cargo fmt --all
  -- --check` flags the EOF newline and the DEBUG lines must not land on
  `main`.
- **Next action (owning/redaction session):** delete the three DEBUG lines,
  run `cargo fmt -p testkit`, re-run the suite. Do not commit otherwise.
- **Scope:** redaction session only. Second sessions: do not touch this
  file while it is under live edit.

## FU-TEST-02 — `lookup_performance_smoke` is load-sensitive and aborts workspace runs
- **Status:** 🔴 open (reproduced 2026-09-24; not a regression)
- **Evidence:** `cargo test --workspace` (2026-09-24) failed
  `application::quran_reader::lookup_performance_smoke` with
  `warm average 6.009 ms over budget` — a 5.0 ms average over 2,000 sequential
  warm `get_ayah` calls (`crates/application/tests/quran_reader.rs:366`).
  Two control runs both pass: in isolation (`cargo test -p application --test
  quran_reader lookup_performance_smoke` → ok, 2.21 s) and in a full
  `cargo test --workspace --no-fail-fast` (zero failures workspace-wide, same
  test included). The failing run was executed while a second cargo build and
  another live agent session were competing for CPU, i.e. wall-clock latency
  was measuring machine load, not a code regression. No reader/lookup/query code
  changed in that window.
- **Impact:** because the gate aborts the crate, one perf miss hides the rest
  of that test binary; a workspace run should use `--no-fail-fast` and report
  the failing test separately.
- **Next action (owning/search session):** make the gate robust rather than
  noisy — measure a median or p95 instead of a mean, take the minimum of a few
  warm repetitions, and/or gate on a dedicated benchmark job with a stated
  machine class, keeping the strict 5 ms mean only for isolated runs. Record the
  decision (ADR-0212 already owns regex limits; a new ADR is not required for a
  test-harness statistic).
- **Scope:** test harness only. No reader behaviour may be relaxed.

## FU-COORD-01 — Parallel sessions editing the same files (observed 2026-09-16)
- **Status:** 🟢 resolved for T45 (2026-09-16, recorded here as precedent)
- **Context:** During P2-T45 verification, 7+ live agent sessions were
  active; `crates/application/src/quran_search.rs` was rewritten mid-read
  (2146 → 2052 lines), the staged set churned twice, and the failing
  assertion in `concatenated_window_verify_tiles_across_ayahs` was repaired
  by the owning session while a second session was root-causing it. Both
  sessions independently reached the same conclusion (tight-hull span
  `0..6` correct; strict `== 7` expectation wrong — proven by executing
  the T44 ayah-level path on identical content). No work was lost, but one
  `edit` call failed safely on a stale read, proving the collision is real.
- **Precedent set:** implementation owner edits; second session verifies
  read-only (targeted `cargo test`, no source writes) and syncs only
  task-ledger docs (`tasks.md` flip + `done.md` entry) on lines the owner
  is not touching. T45 closed this way: `search_tools` 14/14, application
  lib 28/28, `quran-search` 18+5+3, rustfmt clean.
- **Next action:** keep this discipline for T51/T52 (API + CLI surfaces),
  where the cache-wiring both sessions will touch `quran_search.rs` again.
  If collisions recur, assign file-level ownership in the phase
   execution-plan before starting.

## FU-P1-01 — P1-T58: extend fixture soak to an approved standard edition
- **Status:** open; full-corpus run blocked on approved data.
  Re-verified 2026-09-24: `cargo test -p application --test quran_doctor`
  3/3 green (fixture 10k-lookup soak + hash recomputation + deep scan, ~14 s
  for the binary). Fixture evidence only — T58 stays ◐.
- **Found:** 2026-09-17 (Phase-1 implementation review).
- **Evidence:** `crates/application/tests/quran_doctor.rs::fixture_soak_ten_thousand_lookups_preserves_corpus_integrity` passes: fixture import/validation/activation, 10,000 seeded lookups, exact text and token offsets, then deep hashes and round-trip checks.
- **Next action:** once ADR-0101 supplies an approved edition, adapt the fixture-bound harness to accept it and run the complete pipeline; record corpus identity, hashes, lookup results, and deep-scan duration. The current fixture run is not evidence of standard-edition performance or editorial correctness.
- **Task status:** P1-T58 stays partial; do not close it on synthetic-fixture results.

## FU-P1-02 — P1-T60: complete automated exit-gate evidence
- **Status:** open; engineering verification can proceed now. Partial refresh
  2026-09-24: `cargo fmt --all -- --check` clean, `cargo clippy --workspace
  --all-targets -- -D warnings` clean, `cargo xtask arch-check` OK, `cargo xtask
  migrate-check` OK (19 migrations). Coverage + `deny` gates and the recorded
  walkthrough are still outstanding.
- **Found:** 2026-09-17 (Phase-1 gate preparation).
- **Verified:** `cargo test -p application -- --test-threads=1` passed (110 tests); workspace fmt, clippy with warnings denied, build check, architecture check, and migration check passed.
- **Next action:** run workspace-wide tests, required coverage and dependency-audit gates; record commands and failures without assuming unrelated worktree edits are broken. An initial parallel application run timed out during doctor tests, while isolated and serial reruns passed; investigate if reproducible rather than claiming it fixed.
- **Targeted gates 2026-09-24 (P1 scope only):** `cargo clippy -p quran-corpus
  -p storage -p storage-sqlite -p application -p cli --all-targets -- -D warnings`
  clean; `cargo xtask arch-check` OK; `cargo xtask migrate-check` OK
  (19 migrations); `cargo fmt --check` clean on the touched crates;
  `quran-corpus` 49+6+7, `storage` 32, `storage-sqlite::quran` 10/10,
  `application::quran_import` 14/14, `application::quran_verification` 3/3
  (new), `application::quran_doctor` 3/3, `cli::quran` 5/5 — all green.
  Workspace-wide `cargo test` was not run (15-minute timeout is a known
  pre-existing suite-size limit; a concurrent Phase-2 writer was active in
  the tree). Coverage + `deny` + recorded walkthrough still outstanding.
- **Closure boundary:** automated checks do not replace the independent recorded walkthrough, editorial approval, or handoff sign-off. P1-T60 remains open.

## FU-P1-03 — Resume owner-gated Phase-1 implementation
- **Status:** open; engineering groundwork landed 2026-09-17, owner decisions still pending.
- **Found:** 2026-09-17 (remaining-task review; updated same day after T26/T54/T56 groundwork).
- **P1-T26:** `quran_corpus::import::compare_reference` now implements exact, order-independent comparison (Fatal QV-015 on any text difference, missing/duplicate ayahs, empty corpus, or incompatible reading); `compared()` fails the import closed when the manifest requests a reference corpus that is unavailable (`requested_reference_cannot_be_silently_skipped` regression guards it). Remaining: a real reference corpus + procedure + sign-off (ADR-0114 owner) before any configured comparison runs against approved data.
- **P1-T54:** `server::api::router_with_debug_font` serves a caller-supplied woff2 in memory at `/debug/assets/reader.woff2` (font/woff2, no-store, nosniff) and the debug page emits `@font-face` only when a font is wired; without it, the system Arabic stack remains and no `@font-face` is emitted. Remaining: an owner-chosen font asset + license before any release bundles one.
- **P1-T56:** `golden_ayah_texts_match_the_imported_fixture` ties every fixture ayah to `fixtures/quran/golden/ayah_texts.jsonl` byte-for-byte; the 331-case reference golden set remains green. Remaining: expand fixtures only after approved data + P1-T55 review; never fabricate scripture.
- **Decision tracking:** dataset/license/reviewer and reference-corpus choices remain the existing ADR-0101/ADR-0114 owner decisions, not approvals supplied by this follow-up.
- **Re-verified 2026-09-24 (no code change):** OD-01…OD-14 all still 🔴 in
  `decisions-needed.md`; the three groundwork items above are still exactly as
  described (compare_reference, debug-font route, golden 331-case tie-out) and
  none can advance without the named owner inputs (approved dataset, reference
  corpus + signer, font license). An agent verification pass was appended to the
  `decisions-needed.md` answer log; no row was closed.
- **Engineering 2026-09-24 (code landed, owner gates unchanged):**
  - **P1-T26 Tier-2:** `quran-corpus::differ` now carries the ADR-0114
    comparison taxonomy — `ComparisonKind`
    (integrity/version/readings/translation/reference/checksum),
    `DifferenceClass` (the nine ADR literals, serialized verbatim),
    `classify_difference` (presence → `edition_difference`,
    whitespace-only → `normalization_only`, else `unknown_difference` —
    never forced benign), `diff_ayahs_typed`, `ComparisonOperands`, and
    `CLASSIFICATION_VOCABULARY = "ADR-0114-v1"`. `diff_ayahs` keeps its
    signature and Tier-1 exact behavior; new fields are `#[serde(default)]`
    so legacy reports deserialize. QV-015 `compared()` evidence now records
    `comparison_kind: "reference"`, `classification_vocabulary`, and
    `normalization_applied: []`. Evidence: 6 new `differ` unit tests;
    `cargo test -p quran-corpus` 49+6+7 green; `quran_import` 14/14 green.
    ADR-0114 stays Draft; T26 stays ◐ on the Tier-2 corpus + sign-off.
  - **P1-T55 plumbing:** `QuranRepository::set_edition_verification`
    (trait + SQLite impl; only the `verified_*` metadata columns — the
    edition-identity trigger cannot fire, proven by test) plus the
    approval-gated `application::quran::record_edition_verification`
    service (`EditionVerification{reviewer, method}` recorded verbatim,
    empty values → `QAI-QUR-0306`, missing/denied/mismatched approvals
    rejected like activation, `SourceApproved` audit in the same
    transaction) plus `qai quran edition verify --reviewer --method`.
    Evidence: `application --test quran_verification` 3/3 (new),
    `storage-sqlite --test quran` 10/10 (new stamp test), `cli --test
    quran` 5/5 (snapshots intact). T55 stays ☐ — the mechanism records a
    reviewer; only OD-02 can name one.
  - **P1-T02/T03/X01…X05:** no status change available to an agent. Legal
    review (bundle vs user-supplied), reference-corpus selection, dataset
    choice, reviewer naming, morphology dataset/license, and linguist
    engagement remain `_unassigned_` owner acts (OD-01/02/03/10/11/12).
