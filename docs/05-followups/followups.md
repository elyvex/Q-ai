# Follow-ups

Dated, agent-actionable items that are not acceptance criteria, not
owner-only decisions (those live in `decisions-needed.md`), and not
phase-scoped technical questions (those live in `open-questions.md`).
Status key: 🔴 open · 🟢 done (with date + where recorded).

---

## FU-DOC-01 — `docs/03-plan/current-plan.md` is empty (0 bytes)
- **Status:** 🔴 open
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
- **Status:** 🔴 open
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
- **Status:** 🔴 open
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
- **Status:** 🔴 open
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
- **Found:** 2026-09-17 (Phase-1 implementation review).
- **Evidence:** `crates/application/tests/quran_doctor.rs::fixture_soak_ten_thousand_lookups_preserves_corpus_integrity` passes: fixture import/validation/activation, 10,000 seeded lookups, exact text and token offsets, then deep hashes and round-trip checks.
- **Next action:** once ADR-0101 supplies an approved edition, adapt the fixture-bound harness to accept it and run the complete pipeline; record corpus identity, hashes, lookup results, and deep-scan duration. The current fixture run is not evidence of standard-edition performance or editorial correctness.
- **Task status:** P1-T58 stays partial; do not close it on synthetic-fixture results.

## FU-P1-02 — P1-T60: complete automated exit-gate evidence
- **Status:** open; engineering verification can proceed now.
- **Found:** 2026-09-17 (Phase-1 gate preparation).
- **Verified:** `cargo test -p application -- --test-threads=1` passed (110 tests); workspace fmt, clippy with warnings denied, build check, architecture check, and migration check passed.
- **Next action:** run workspace-wide tests, required coverage and dependency-audit gates; record commands and failures without assuming unrelated worktree edits are broken. An initial parallel application run timed out during doctor tests, while isolated and serial reruns passed; investigate if reproducible rather than claiming it fixed.
- **Closure boundary:** automated checks do not replace the independent recorded walkthrough, editorial approval, or handoff sign-off. P1-T60 remains open.

## FU-P1-03 — Resume owner-gated Phase-1 implementation
- **Status:** open; engineering actions depend on existing owner decisions.
- **Found:** 2026-09-17 (remaining-task review).
- **P1-T26:** comparator unchanged; QV-015 currently records a skip. After ADR-0114 defines the reference corpus and comparison procedure, implement configured comparison with match/mismatch/missing-reference tests and preserve explicit unconfigured behavior.
- **P1-T54:** after the font asset and license are approved, bundle the web font, wire `@font-face`, and verify rendering and route behavior. Existing RTL, labels, markers, and escaping are not completion of the web-font requirement.
- **P1-T56:** after approved data and P1-T55 review, expand golden fixtures to every required edge case with traceable expected text/hashes; never substitute fabricated scripture.
- **Decision tracking:** dataset/license/reviewer and reference-corpus choices remain the existing ADR-0101/ADR-0114 owner decisions, not approvals supplied by this follow-up.
