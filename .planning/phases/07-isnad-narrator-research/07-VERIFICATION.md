---
phase: 07-isnad-narrator-research
verified: 2026-10-17T20:20:00Z
status: passed
score: 16/17 must-haves verified
covered_files: [".planning/phases/07-isnad-narrator-research/07-01-PLAN.md", ".planning/phases/07-isnad-narrator-research/07-01-SUMMARY.md", ".planning/phases/07-isnad-narrator-research/07-02-PLAN.md", ".planning/phases/07-isnad-narrator-research/07-02-SUMMARY.md", ".planning/phases/07-isnad-narrator-research/07-03-PLAN.md", ".planning/phases/07-isnad-narrator-research/07-03-SUMMARY.md", ".planning/phases/07-isnad-narrator-research/07-04-PLAN.md", ".planning/phases/07-isnad-narrator-research/07-04-SUMMARY.md", ".planning/phases/07-isnad-narrator-research/07-05-PLAN.md", ".planning/phases/07-isnad-narrator-research/07-05-SUMMARY.md", "crates/application/src/isnad_graph_store.rs", "crates/application/src/isnad_reader.rs", "crates/application/src/isnad_tools.rs", "crates/application/tests/isnad_identity.rs", "crates/application/tests/isnad_reader.rs", "crates/cli/src/isnad.rs", "crates/cli/src/lib.rs", "crates/isnad-graph/src/lib.rs", "crates/isnad-graph/tests/conformance.rs", "crates/quran-graph/src/model.rs", "crates/server/src/api.rs", "crates/tool-registry/src/lib.rs", "crates/tui/src/app.rs", "crates/tui/src/palette.rs", "crates/tui/src/screens/mod.rs", "docs/05-followups/phase-07-deferrals.md", "docs/05-followups/phase-07-owner-gates.md", "migrations/sqlite/0031_narrators.up.sql", "migrations/sqlite/0032_narrator_rijal_sources.up.sql", "migrations/sqlite/0033_isnad_edges.up.sql", "migrations/sqlite/0034_narrator_identity_assertions.up.sql"]
covered_digest: "v3:sha256:e473b8335e279b70128e817b0ad58788ad5a8a99ec239999407af39a6b857476"
behavior_unverified: 0
overrides_applied: 0
human_verification:
  - test: "Open the TUI cockpit and select the `isnad-narrator`, `isnad-trace`, and `isnad-identity` palette commands."
    expected: "UI-SPEC §124/§152 promise a narrator Table (id/kind/decision/summary), a chain-path list with per-link uncertainty footer, and an identity-review table. The implemented screens instead render honest empty-state hints ('no narrator selected yet — use qai isnad narrator --id <slug>'). Decide whether the empty-state-only TUI satisfies Phase 7, or whether the data-rendering screens belong to a follow-up phase."
    why_human: "No narrator-selection / trace-input / identity-queue state exists in the cockpit, so the screens cannot render records. Whether that deviation is acceptable is a product/UX judgement, not a grep-checkable fact."
---

# Phase 7: Isnad & Narrator Research Verification Report

**Phase Goal:** Users can trace narration chains and judge narrator identity with explicit uncertainty.
**Verified:** 2026-10-17
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| #   | Truth   | Status     | Evidence       |
| --- | ------- | ---------- | -------------- |
| 1 | **SC1** User can view a narrator record with name variants and rijal sources | ✓ VERIFIED | `qai --data-dir <t> isnad narrator --id narrator:abu-hurayra` → `narrator:abu-hurayra — Abu Hurayra` / `variants: Abu Huraira, Abu Hurairah` / 2 per-scholar rijal rows. Tests `narrator_round_trip_with_variants`, `rijal_sources_per_scholar_not_merged`, `empty_rijal_returns_zero` green. |
| 2 | **SC2** User can trace chain paths between narrators with uncertainty marked on each link | ✓ VERIFIED | `qai isnad trace --from narrator:abu-hurayra --to narrator:malik` → 2 rows, each with edge, provenance layer, confidence and `(uncertain)`. After flipping one edge to `label='confirmed'`, that link's suffix disappears — the label is read per-link, not hardcoded. Tests `budget_exhaustion_truncated`, `empty_path_complete`, `non_narrator_edge_endpoint_is_skipped_not_fatal`. |
| 3 | **SC3** Reviewer can approve or reject a narrator-identity proposal in a recorded workflow | ✓ VERIFIED | e2e: `identity propose` → `pending`; `identity accept --reviewer rev-9` → DB `accepted / rev-9 / decided_at set`; `identity reject` refuses without `--yes` (exit USAGE), records `rejected / rev-10` with `--yes`. DB CHECK `decision='pending' OR (reviewer IS NOT NULL AND reviewer != '' AND decided_at IS NOT NULL)` enforced by `db_check_rejects_identity_accepted_without_decided_at`. |
| 4 | Narrator record with stable id `narrator:{slug}` round-trips with variants and rijal sources | ✓ VERIFIED | `NarratorId::new`/`FromStr` validated (`narrator_id_rejects_invalid_slugs`); round-trip test green. |
| 5 | Transmission edges (TRANSMITS, NARRATES) stage into the graph vocabulary without rejection | ✓ VERIFIED | `EDGE_VOCABULARY` contains TRANSMITS/NARRATES (`is_allowed_edge_transmits_and_narrates`); e2e edges staged and traversed. |
| 6 | Identity proposals are assertions with Pending/Accepted/Rejected/Superseded/Disputed states | ✓ VERIFIED | `propose_pending_reviewer_null`, `accept_sets_reviewer_and_decided_at`, `reject_retained_not_deleted`, `dispute_stays_effective`, `supersession_chain_resolves_latest` green. |
| 7 | Chain traversal reuses quran-graph traversal with uncertainty on each link | ✓ VERIFIED | `IsnadReaderService::trace_paths` → `SqliteIsnadGraphStore::bounded_paths` → `MemGraphStore`; edge `attrs.label` survives to the CLI. e2e above. |
| 8 | Typed tools (isnad.narrator/trace/identity) return ToolResult with research_checksum | ✓ VERIFIED | `isnad_tools_dispatch_through_the_shared_registry` asserts `research_checksum.hex` is a string; every backend returns a checksum via typed `ToolError` on failure. |
| 9 | `qai isnad narrator/rijal/trace/identity …` verbs work from the CLI | ✓ VERIFIED | All verbs exercised against the built binary (see Behavioral Spot-Checks). |
| 10 | CLI output follows `CommandOutput { human, json }` with research_checksum in JSON | ✓ VERIFIED | `handle_isnad` renders human text on default, full `ToolResult` JSON with `--json`; code read + e2e human leg. |
| 11 | HTTP exposes isnad tools via the generic typed-tool route | ✓ VERIFIED | `run_isnad_dispatch`/`AppState.isnad` deleted (WR-07); serve arm wires `tools.set_isnad_backend(...)` so `isnad.*` resolves through the shared name→tool table. Server test `isnad_identity_mutations_are_forbidden_on_http_by_default` green; shared-dispatcher test green. Mutating identity tools denied by default with typed 403 (WR-08). |
| 12 | TUI exposes narrator record and identity review screens | ⚠️ PRESENT_BEHAVIOR_UNVERIFIED | Palette entries (`isnad-narrator|trace|identity`) and `Screen` variants exist and are wired, but each renders an honest empty-state hint, not the record/table UI-SPEC §124/§152 describe (WR-09). Reachable ≠ rendering records. See Human Verification. |
| 13 | DB CHECK rejects confirmed/accepted without reviewer + decided_at | ✓ VERIFIED | `db_check_rejects_rijal_accepted_without_reviewer`, `db_check_rejects_isnad_edge_confirmed_without_reviewer`, `db_check_rejects_identity_accepted_without_decided_at` green. |
| 14 | Disputed stays effective; rejected/superseded retained; supersession chain ordered | ✓ VERIFIED | `dispute_stays_effective`, `traversal_excludes_tombstoned_disputed_effective`, `supersede_creates_new_referencing_old_old_retained`, `supersede_binds_supersedes_id_in_the_insert`, `failed_supersede_leaves_no_orphan_tombstone` green. |
| 15 | Budget exhaustion returns truncated + incomplete_reason; confidence bands high/medium/low | ✓ VERIFIED | `budget_exhaustion_truncated` green; `confidence_bands_thresholds` green at both crate and graph levels. |
| 16 | Open owner decisions recorded with exact closing commands; deferred scope recorded | ✓ VERIFIED | `docs/05-followups/phase-07-owner-gates.md` (OD-07-01..04, all BLOCKED, exact closing command each) and `docs/05-followups/phase-07-deferrals.md` (D-07-01..04) exist. |
| 17 | Full workspace test suite green | ✓ VERIFIED (with caveat) | In-session: `isnad_reader` 11/11, `isnad_identity` 12/12, `isnad-graph` 12/12, `server --test api isnad` 1/1, `cli --test parity_cross_surface` 5/5 — all green. Independent `cargo test --workspace` → **1 failure**: `cli --test quran :: serve_hosts_the_worker_and_shuts_down_joined` (timeout, job stayed `Queued` after 90s under parallel load). Re-run in isolation → **passes in 7.5s**. `git log` shows that file was last touched in Phase 5/3.5 and **never by a Phase 7 commit** — pre-existing flake, not a Phase 7 gap (same test flagged as known-flaky in 07-REVIEW-FIX). |

**Score:** 16/17 truths verified (1 present, behavior/surface-unverified — TUI)

### Required Artifacts

| Artifact | Expected    | Status | Details |
| -------- | ----------- | ------ | ------- |
| `migrations/sqlite/0031_narrators.up.sql` | narrators table | ✓ VERIFIED | Applied by `qai db migrate` → "schema version 34"; PK is `narrator:{slug}` |
| `migrations/sqlite/0032_narrator_rijal_sources.up.sql` | per-scholar rijal sources, no consensus column | ✓ VERIFIED | CHECK owner gate + `provenance_layer IN (B,C,D)`; `dropped consensus` asserted in test |
| `migrations/sqlite/0033_isnad_edges.up.sql` | transmission edges w/ owner gate | ✓ VERIFIED | label default `uncertain`; confirmed requires reviewer+decided_at |
| `migrations/sqlite/0034_narrator_identity_assertions.up.sql` | identity assertions | ✓ VERIFIED | decision CHECK + D-layer gate |
| `crates/isnad-graph/src/lib.rs` | port crate domain types | ✓ VERIFIED | 216 lines; NarratorId/RijalSource/IsnadLinkLabel/ConfidenceBand; slug validation (WR-11) |
| `crates/application/src/isnad_graph_store.rs` | SQLite→GraphStore adapter | ✓ VERIFIED | 265 lines; loads narrators+edges, skipped-edge accounting (CR-01); wired into `trace_paths` |
| `crates/application/src/isnad_reader.rs` | IsnadReader trait + service | ✓ VERIFIED | 416 lines; transactional supersede (WR-03), confidence validation (WR-06) |
| `crates/application/src/isnad_tools.rs` | IsnadBackend implementation | ✓ VERIFIED | 381 lines; checksums as typed errors (WR-04), real decision labels (WR-05), rijal warning (WR-02) |
| `crates/cli/src/isnad.rs` | `qai isnad` command tree | ✓ VERIFIED | 521 lines; dispatched from `Commands::Isnad`; e2e exercised |
| `crates/application/tests/isnad_reader.rs` | reader/CHECK tests | ✓ VERIFIED | 11 tests, all pass |
| `crates/application/tests/isnad_identity.rs` | identity/supersession tests | ✓ VERIFIED | 12 tests, all pass |
| `crates/isnad-graph/tests/conformance.rs` | conformance tests | ✓ VERIFIED | 7 tests, all pass |
| `docs/05-followups/phase-07-owner-gates.md` | owner-gate ledger | ✓ VERIFIED | 4 BLOCKED gates with closing commands |
| `docs/05-followups/phase-07-deferrals.md` | deferral ledger | ✓ VERIFIED | 4 deferred items with rationale |

### Key Link Verification

| From | To  | Via | Status | Details |
| ---- | --- | --- | ------ | ------- |
| `crates/cli/src/isnad.rs` | `IsnadReaderService` + `IsnadToolBackend` | direct construction in `handle_isnad_async` | WIRED | e2e produced real rows |
| CLI output | `CommandOutput { human, json }` | `print!` human vs `serde_json` ToolResult | WIRED | code read + e2e human leg |
| HTTP generic route | shared `dispatch_registered_tool` name→tool table | serve arm `set_isnad_backend` | WIRED | server + shared-dispatcher tests green (WR-07) |
| `SqliteIsnadGraphStore` | `quran_graph::GraphStore` (MemGraphStore) | trait impl delegation | WIRED | trace e2e traverses staged narrators/edges |
| isnad edges | `EDGE_VOCABULARY` (TRANSMITS/NARRATES) | `is_allowed_edge` | WIRED | conformance test green |
| `tool_registry::IsnadBackend` | `ToolResult` with `research_checksum` | `IsnadToolBackend` | WIRED | checksum asserted in test |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| `qai isnad narrator` | `results.narrator.name_variants`, `results.rijal_sources` | `SELECT … FROM narrators` / `narrator_rijal_sources` | Yes (seeded rows rendered verbatim) | ✓ FLOWING |
| `qai isnad trace` | `results.paths[].edges[].attrs.label/confidence` | `isnad_edges` → MemGraphStore → `bounded_paths` | Yes (per-link label flipped confirmed/uncertain changes output) | ✓ FLOWING |
| `qai isnad identity *` | assertion decision/reviewer/decided_at | `narrator_identity_assertions` | Yes (DB row reflects CLI verb) | ✓ FLOWING |
| TUI isnad screens | — | none (no selection/queue state) | No | ⚠️ HOLLOW — deliberate empty state (WR-09) |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| Migration set applies | `qai --data-dir /tmp/qai-verify db migrate` | `migrations applied; schema version 34` | ✓ PASS |
| Narrator record + rijal render | `qai isnad narrator --id narrator:abu-hurayra` | name + 2 variants + 2 scholar rows | ✓ PASS |
| Chain trace marks uncertainty per link | `qai isnad trace --from narrator:abu-hurayra --to narrator:malik --k 5` | 2 links, both `(uncertain)` with confidence | ✓ PASS |
| Confirmed link drops the suffix | (edge flipped to `confirmed` + reviewer/decided_at) → re-run trace | second link prints without `(uncertain)` | ✓ PASS |
| Identity propose/accept recorded | `identity propose` … `identity accept --reviewer rev-9` | DB row `accepted / rev-9 / decided_at` | ✓ PASS |
| Reject needs `--yes`; records decision | `identity reject` (no flag) → exit 2; with `--yes` → `rejected / rev-10` | recorded | ✓ PASS |
| Phase test suites | `cargo test -p application --test isnad_reader --test isnad_identity` / `-p isnad-graph` | 23 + 12 passed, 0 failed | ✓ PASS |
| Full workspace | `cargo test --workspace` | 1 failed: `cli::quran::serve_hosts_the_worker_and_shuts_down_joined` (pre-existing flake); re-run alone → PASS in 7.5s | ⚠️ FLAKE |

### Probe Execution

Not applicable — no probes declared in any Phase 7 PLAN and no `scripts/*/tests/probe-*.sh` in the repo.

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ----------- | ----------- | ------ | -------- |
| REQ-isnad-narrator-graph | 07-01..07-05 (all four declare it in frontmatter) | Isnad/narrator graph with node/edge types and identity-uncertainty handling (PRD §15) | ✓ SATISFIED | Truths 1–11, 13–15; model declares `NodeKind::Narrator`, `AssertionKind::NarratorIdentity`, TRANSMITS/NARRATES. No ORPHANED requirements: REQUIREMENTS.md maps only REQ-isnad-narrator-graph to Phase 7 and every plan claims it. |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| — | — | No `TBD`/`FIXME`/`XXX`/`TODO`/`HACK`/`PLACEHOLDER` in any phase-modified file | — | Clean debt-marker scan |
| `crates/tool-registry/src/lib.rs` | 1839 | `ISNAD_TOOL_NAMES` (6 entries) is dead — declared, never referenced, and omits `identity_dispute`/`identity_supersede` | ℹ️ Info | Cosmetic; dispatch uses the real name→tool table. Carry from REVIEW IN-01. |
| `crates/application/src/isnad_reader.rs` | 137 | `get_rijal_sources` has no `ORDER BY` | ℹ️ Info | Row order (and thus the tool's `research_checksum`) is not stably reproducible across runs. Carry from REVIEW IN-06. |
| `docs/05-followups/phase-07-owner-gates.md` | 12 | Claims canonical rows live in `docs/05-followups/decisions-needed.md`, but that file's `OD-07` is an unrelated Phase-3.5 coverage-gate row — the Phase 7 gates (`OD-07-01..04`) are not in it | ⚠️ Warning | The ledger's cross-reference is inaccurate (ID namespace collision). Closing actions themselves are correct and actionable; no owner decision is falsely recorded. |
| `crates/cli/tests/quran.rs` | 840 | `serve_hosts_the_worker_and_shuts_down_joined` failed once in the full workspace run (90s timeout, state `Queued`); passes in isolation (7.5s). File last touched in Phase 5/3.5, never by a Phase 7 commit | ℹ️ Info | Pre-existing load-sensitive flake, unrelated to Phase 7. Flagged as known-flaky in 07-REVIEW-FIX. |
| `docs/06-progress/status.md`, `docs/06-progress/task-done-rollup.md`, `.planning/ROADMAP.md`, `.planning/STATE.md` | — | Not updated for Phase 7 (status.md still says "Active phase: Phase 5"; ROADMAP shows `Plans: TBD` and 07-05 unchecked; STATE `stopped_at: Completed 07-03-PLAN.md`) | ℹ️ Info | Project-level bookkeeping lag (Phase 6 is also absent from status.md); does not affect delivered behaviour. |

### Human Verification Required

#### 1. TUI isnad screens render records, not just empty states

**Test:** Launch `qai tui`, open the palette, run `isnad-narrator`, `isnad-trace`, `isnad-identity`.
**Expected:** Either (a) the screens render the narrator Table / chain-path list / identity-review table that 07-UI-SPEC §124/§152 promise, or (b) the empty-state-only implementation is accepted as the intended Phase 7 TUI deliverable (with the data-rendering screens scheduled for a later phase).
**Why human:** The screens are present and reachable but render "no narrator selected yet — use `qai isnad …`"; no cockpit state exists to feed them. Whether that satisfies the phase is a product/UX decision, not a programmatic check. Truth 12 is left ⚠️ PRESENT_BEHAVIOR_UNVERIFIED for this reason.

### Gaps Summary

No blockers. The three roadmap success criteria are all observably true in the codebase and reproduce end-to-end through the built `qai` binary against a freshly migrated SQLite database:

1. **View a narrator record with name variants and rijal sources** — works (CLI + HTTP tool + tests).
2. **Trace chain paths with uncertainty marked on each link** — works; the per-link `label` is genuinely read (confirmed vs uncertain changes the output).
3. **Reviewer approve/reject a narrator-identity proposal in a recorded workflow** — works; decisions persist with reviewer + `decided_at`, and the DB CHECK enforces it.

The only soft item is the TUI surface: the three isnad screens exist and are wired but intentionally render empty-state hints rather than records (07-REVIEW-FIX WR-09). That is a UI-SPEC deviation, not a broken roadmap capability — hence `human_needed`, not `gaps_found`.

The independent full-workspace run failed exactly one test, `cli::quran::serve_hosts_the_worker_and_shuts_down_joined`, which is a pre-existing load-sensitive flake outside Phase 7's file set (last touched Phase 5/3.5; passes in isolation in 7.5s) — so it is **not** counted as a Phase 7 gap. The phase's own suites (23 + 12 + server gate + CLI parity) are green.

---

_Verified: 2026-10-17_
_Verifier: the agent (gsd-verifier)_
