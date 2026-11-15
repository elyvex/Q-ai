---
phase: 07-isnad-narrator-research
verified: 2026-11-15T15:15:39Z
status: passed
score: 17/17 must-haves verified
covered_files: [".planning/phases/07-isnad-narrator-research/07-01-PLAN.md", ".planning/phases/07-isnad-narrator-research/07-01-SUMMARY.md", ".planning/phases/07-isnad-narrator-research/07-02-PLAN.md", ".planning/phases/07-isnad-narrator-research/07-02-SUMMARY.md", ".planning/phases/07-isnad-narrator-research/07-03-PLAN.md", ".planning/phases/07-isnad-narrator-research/07-03-SUMMARY.md", ".planning/phases/07-isnad-narrator-research/07-04-PLAN.md", ".planning/phases/07-isnad-narrator-research/07-04-SUMMARY.md", ".planning/phases/07-isnad-narrator-research/07-05-PLAN.md", ".planning/phases/07-isnad-narrator-research/07-05-SUMMARY.md", "crates/application/src/isnad_graph_store.rs", "crates/application/src/isnad_reader.rs", "crates/application/src/isnad_tools.rs", "crates/application/src/quran_tools.rs", "crates/application/tests/isnad_identity.rs", "crates/application/tests/isnad_reader.rs", "crates/cli/src/isnad.rs", "crates/cli/src/lib.rs", "crates/isnad-graph/src/lib.rs", "crates/isnad-graph/tests/conformance.rs", "crates/quran-graph/src/model.rs", "crates/server/src/api.rs", "crates/tool-registry/src/lib.rs", "crates/tui/src/app.rs", "crates/tui/src/palette.rs", "crates/tui/src/screens/mod.rs", "docs/05-followups/phase-07-deferrals.md", "docs/05-followups/phase-07-owner-gates.md", "migrations/sqlite/0031_narrators.up.sql", "migrations/sqlite/0032_narrator_rijal_sources.up.sql", "migrations/sqlite/0033_isnad_edges.up.sql", "migrations/sqlite/0034_narrator_identity_assertions.up.sql"]
covered_digest: "v3:sha256:0182ba576d0a4fd62a1cbdf2e23809d0a63badb6ee1f05709664c1da36d7c948"
behavior_unverified: 0
overrides_applied: 0
re_verification:
  previous_status: passed
  previous_score: 17/17
  gaps_closed: []
  gaps_remaining: []
  regressions: []
deferred:
  - truth: "Full narrator database ingestion (real licensed data)"
    addressed_in: "Phase 9 (OD-07-01)"
    evidence: "Owner gate BLOCKED in docs/05-followups/phase-07-owner-gates.md; Phase 7 delivers schema + tools + review workflow only"
  - truth: "Comparative isnad across collections"
    addressed_in: "Phase 9 (OD-07-02)"
    evidence: "Owner gate BLOCKED; deferral D-07-02 in docs/05-followups/phase-07-deferrals.md"
  - truth: "Backfill hadith_records.narrator_names_json to narrator nodes"
    addressed_in: "After OD-07-01 (D-07-03)"
    evidence: "Deferral D-07-03; skip-with-count path in SqliteIsnadGraphStore already tolerates hadith:* endpoints"
  - truth: "Narrator identity auto-resolution (never presented as certainty)"
    addressed_in: "V2-04 (D-07-04)"
    evidence: "Deferral D-07-04; uncertain-until-owner-confirmed policy locked"
---

# Phase 7: Isnad & Narrator Research Verification Report

**Phase Goal:** Users can trace narration chains and judge narrator identity with explicit uncertainty.
**Verified:** 2026-11-15 (HEAD `67b47f6`)
**Status:** passed
**Re-verification:** Yes — fingerprint refresh. The prior report (2026-10-17) was `passed` but its covered-input digest was stale after Phases 8–12 changed covered files (`crates/server/src/api.rs`, `crates/cli/src/lib.rs`, `crates/application/src/quran_tools.rs`, `crates/tool-registry/src/lib.rs`, `crates/tui/*`). Every claim below was re-checked against the current tree; nothing was carried over on trust.

## Goal Achievement

### Observable Truths

| #   | Truth   | Status     | Evidence       |
| --- | ------- | ---------- | -------------- |
| 1 | **SC1** User can view a narrator record with name variants and rijal sources | ✓ VERIFIED | Live at HEAD against a freshly migrated DB (schema 44): `qai isnad narrator --id narrator:abu-hurayra` → `narrator:abu-hurayra — Abu Hurayra` / `variants: Abu Huraira, Abu Hurairah` / 2 per-scholar rows (`Ibn Hajar — Tahdhib al-Tahdhib — thiqa (pending)`, `al-Dhahabi — Siyar — no grade (pending)`). Same payload over HTTP (see spot-checks). Tests `narrator_round_trip_with_variants`, `rijal_sources_per_scholar_not_merged`, `empty_rijal_returns_zero` green. |
| 2 | **SC2** User can trace chain paths between narrators with uncertainty marked on each link | ✓ VERIFIED | Live at HEAD over two seeded edges — one `label='uncertain'`, one `label='confirmed'` — `qai isnad trace --from narrator:abu-hurayra --to narrator:malik --k 5` printed `… -TRANSMITS-> … B 0.95 (uncertain)` for the uncertain link and `… -TRANSMITS-> … B 0.75` (no suffix) for the confirmed link. Per-link label genuinely drives the suffix (`crates/cli/src/isnad.rs:237-245`), fed from `isnad_edges.label` via `SqliteIsnadGraphStore` attrs → `MemGraphStore` path edges. Tests `budget_exhaustion_truncated`, `empty_path_complete`, `traversal_excludes_tombstoned_disputed_effective`, `non_narrator_edge_endpoint_is_skipped_not_fatal` green. |
| 3 | **SC3** Reviewer can approve or reject a narrator-identity proposal in a recorded workflow | ✓ VERIFIED | Live at HEAD: `identity propose` → id `nia:…`; `identity accept --reviewer rev-9` → DB row `accepted / rev-9 / decided_at set`; `identity reject` without `--yes` → exit 2 and no write; with `--yes` → `rejected / rev-10`. DB CHECK `decision='pending' OR (reviewer IS NOT NULL AND reviewer != '' AND decided_at IS NOT NULL)` enforced by `db_check_rejects_identity_accepted_without_decided_at`. |
| 4 | Narrator record with stable id `narrator:{slug}` round-trips with variants and rijal sources | ✓ VERIFIED | `NarratorId::new`/`FromStr` both validate 1..=128 `[A-Za-z0-9_-]` (`crates/isnad-graph/src/lib.rs:27-70`); `narrator_id_rejects_invalid_slugs`, `narrator_id_stable_form`, `narrator_roundtrip_serde` green. PK is the stable id (`0031_narrators.up.sql`). |
| 5 | Transmission edges (TRANSMITS, NARRATES) stage into the graph vocabulary without rejection | ✓ VERIFIED | `EDGE_VOCABULARY` contains `TRANSMITS`/`NARRATES` (`crates/quran-graph/src/model.rs:97-98`); `is_allowed_edge_transmits_and_narrates` green; `stage_narrator_nodes_and_edges` green. Live trace above traverses staged `TRANSMITS` edges. |
| 6 | Identity proposals are assertions with Pending/Accepted/Rejected/Superseded/Disputed states | ✓ VERIFIED | `propose_pending_reviewer_null`, `accept_sets_reviewer_and_decided_at`, `reject_retained_not_deleted`, `dispute_stays_effective`, `supersede_creates_new_referencing_old_old_retained`, `supersession_chain_resolves_latest` green; live accept/reject above. |
| 7 | Chain traversal reuses quran-graph traversal with uncertainty on each link | ✓ VERIFIED | `IsnadReaderService::trace_paths` → `SqliteIsnadGraphStore::bounded_paths` → `MemGraphStore` (delegating `GraphStore` impl); path edges carry the full `GraphEdge` incl. `attrs` (`crates/quran-graph/src/mem.rs:339`). Live trace demonstrates the attrs survive to the CLI. |
| 8 | Typed tools (isnad.narrator/trace/identity) return ToolResult with research_checksum | ✓ VERIFIED | `isnad_tools_dispatch_through_the_shared_registry` drives the real dispatcher and asserts `research_checksum.hex` is a string; `--json` live output has a 64-hex checksum. Every backend maps `IsnadReaderError` to typed `ToolError`. |
| 9 | `qai isnad narrator/rijal/trace/identity …` verbs work from the CLI | ✓ VERIFIED | All four verbs plus all five identity sub-verbs exercised live at HEAD (see spot-checks); `Commands::Isnad` wired at `crates/cli/src/lib.rs:150,626`. |
| 10 | CLI output follows `CommandOutput { human, json }` with research_checksum in JSON | ✓ VERIFIED | `handle_isnad_async` renders human text on default and the full `ToolResult` JSON with `--json`; live `--json` run returned `tool_name=isnad.narrator`, 64-char checksum, 2 variants, 2 rijal rows. |
| 11 | HTTP exposes isnad tools via the generic typed-tool route | ✓ VERIFIED | Live: `POST /api/v1/quran/tool/isnad.narrator` on `qai serve` returned `data.tool_name=isnad.narrator`, both variants, 2 rijal rows and `meta.research_checksum=sha256:…`; `POST …/isnad.identity_accept` → 403 `QAI-ISNAD-0326`. Route dispatches through the same shared name→tool table (`crates/application/src/quran_tools.rs:878-921`) wired in the serve arm (`crates/cli/src/lib.rs:388-395`). Test `isnad_identity_mutations_are_forbidden_on_http_by_default` green. |
| 12 | TUI exposes narrator record and identity review screens | ✓ VERIFIED | `Screen::IsnadNarrator/IsnadTrace/IsnadIdentity` exist, are palette-reachable (`crates/tui/src/palette.rs:41-43`) and map from command names (`crates/tui/src/screens/mod.rs:96-98`). Each renders an honest, actionable empty state pointing at the equivalent `qai isnad …` verb (`crates/tui/src/app.rs:572-588`) — the accepted Phase 7 deliverable per the recorded human validation (commit `b366e64`, "human validation passed (TUI empty-state accepted)"); data-rendering screens are deferred to a later phase. |
| 13 | DB CHECK rejects confirmed/accepted without reviewer + decided_at | ✓ VERIFIED | `db_check_rejects_rijal_accepted_without_reviewer`, `db_check_rejects_isnad_edge_confirmed_without_reviewer`, `db_check_rejects_identity_accepted_without_decided_at`, `d_layer_gate_requires_fields` green; CHECKs present in migrations 0032–0034. |
| 14 | Disputed stays effective; rejected/superseded retained; supersession chain ordered | ✓ VERIFIED | `dispute_stays_effective`, `traversal_excludes_tombstoned_disputed_effective`, `supersede_creates_new_referencing_old_old_retained`, `supersede_binds_supersedes_id_in_the_insert`, `failed_supersede_leaves_no_orphan_tombstone`, `supersession_chain_resolves_latest` green. Supersede is one transaction (tombstone + chained insert). |
| 15 | Budget exhaustion returns truncated + incomplete_reason; confidence bands high/medium/low | ✓ VERIFIED | `budget_exhaustion_truncated` green (`truncated == true`, non-empty `incomplete_reason`); `confidence_bands_thresholds` green in both `isnad-graph` (0.95 High, 0.9/0.7 Medium, 0.69 Low) and `budget_exhaustion_propagates_truncated` green at graph level. |
| 16 | Open owner decisions recorded with exact closing commands; deferred scope recorded | ✓ VERIFIED | `docs/05-followups/phase-07-owner-gates.md` — OD-07-01..04 all 🔴 BLOCKED with owner + evidence path + exact closing action, none answered. `docs/05-followups/phase-07-deferrals.md` — D-07-01..04 with rationale + resume point. |
| 17 | Full workspace test suite green | ✓ VERIFIED | `cargo test --workspace --no-fail-fast` at HEAD: 203 suites passed, 0 failed (log `/tmp/qai-cargo-test.log`); the operator-stopped long-running `soak_full` 50k soak is excluded and re-running separately, so its pass is not asserted here. Re-confirmed in this pass: `isnad-graph` 7/7, `application --test isnad_reader` 11/11, `--test isnad_identity` 12/12, `server --test api isnad_identity_mutations_…` 1/1. No Phase 7 debt markers. |

**Score:** 17/17 truths verified (0 present, behavior-unverified)

### Deferred Items

| # | Item | Addressed In | Evidence |
|---|------|-------------|----------|
| 1 | Full narrator database ingestion | Phase 9 (OD-07-01) | Owner gate BLOCKED; operator runs approved ingestion once source data is licensed |
| 2 | Comparative isnad across collections | Phase 9 (OD-07-02) | Owner gate BLOCKED |
| 3 | Backfill `hadith_records.narrator_names_json` to narrator nodes | After OD-07-01 (D-07-03) | Deferred scope in `phase-07-deferrals.md`; store already skips non-narrator edge endpoints without failing |
| 4 | Narrator identity auto-resolution (never presented as certainty) | V2-04 (D-07-04) | Deferred scope in `phase-07-deferrals.md` |

### Required Artifacts

| Artifact | Expected    | Status | Details |
| -------- | ----------- | ------ | ------- |
| `migrations/sqlite/0031_narrators.up.sql` | narrators table, PK `narrator:{slug}` | ✓ VERIFIED | Applied by `qai db migrate` → "schema version 44" (later phases added 0035–0044) |
| `migrations/sqlite/0032_narrator_rijal_sources.up.sql` | per-scholar rijal sources, no consensus column | ✓ VERIFIED | Owner-gate + D-layer CHECKs; `rijal_source_per_scholar_no_consensus` asserts no `consensus` field |
| `migrations/sqlite/0033_isnad_edges.up.sql` | transmission edges with owner gate | ✓ VERIFIED | `label` default `uncertain`; confirmed requires reviewer + decided_at; edge CHECK limits to TRANSMITS/NARRATES |
| `migrations/sqlite/0034_narrator_identity_assertions.up.sql` | identity assertions | ✓ VERIFIED | decision CHECK + D-layer gate (algorithm/version/confidence) |
| `crates/isnad-graph/src/lib.rs` | port-crate domain types | ✓ VERIFIED | 216 lines; `NarratorId`/`RijalSource`/`IsnadLinkLabel`/`ConfidenceBand`; slug validation |
| `crates/application/src/isnad_graph_store.rs` | SQLite→GraphStore adapter | ✓ VERIFIED | 265 lines; loads narrators + edges, rejected-edge filter, skipped-edge accounting surfaced as incompleteness |
| `crates/application/src/isnad_reader.rs` | IsnadReader trait + service | ✓ VERIFIED | 416 lines; transactional supersede, pre-write confidence + D-layer validation |
| `crates/application/src/isnad_tools.rs` | IsnadBackend implementation | ✓ VERIFIED | 381 lines; checksums as typed errors; rijal read failure becomes an explicit warning |
| `crates/application/src/quran_tools.rs` | shared name→tool dispatch table incl. `isnad.*` | ✓ VERIFIED | All eight `isnad.*` arms present (`:878-921`); serves CLI/HTTP identically |
| `crates/cli/src/isnad.rs` | `qai isnad` command tree | ✓ VERIFIED | 521 lines; human/JSON split; `--yes` gate on reject; usage errors on malformed JSON |
| `crates/application/tests/isnad_reader.rs` | reader/CHECK tests | ✓ VERIFIED | 11 tests, 11 passed |
| `crates/application/tests/isnad_identity.rs` | identity/supersession tests | ✓ VERIFIED | 12 tests, 12 passed |
| `crates/isnad-graph/tests/conformance.rs` | conformance tests | ✓ VERIFIED | 7 tests, 7 passed |
| `docs/05-followups/phase-07-owner-gates.md` | owner-gate ledger | ✓ VERIFIED | 4 BLOCKED gates, each with exact closing action |
| `docs/05-followups/phase-07-deferrals.md` | deferral ledger | ✓ VERIFIED | 4 deferred items with rationale + resume point |

### Key Link Verification

| From | To  | Via | Status | Details |
| ---- | --- | --- | ------ | ------- |
| `crates/cli/src/isnad.rs` | `IsnadReaderService` + `IsnadToolBackend` | direct construction in `handle_isnad_async` | WIRED | Live CLI produced real rows |
| CLI output | `CommandOutput { human, json }` + ToolResult | `print!` human vs `serde_json` ToolResult | WIRED | Live human + `--json` runs |
| HTTP generic route | shared `dispatch_registered_tool` name→tool table | serve arm `tools.set_isnad_backend(...)` | WIRED | Live HTTP read returned data; mutating gate returned typed 403 (WR-07/WR-08) |
| `SqliteIsnadGraphStore` | `quran_graph::GraphStore` (MemGraphStore) | trait delegation | WIRED | Live trace traversed staged narrators/edges |
| isnad edges | `EDGE_VOCABULARY` (TRANSMITS/NARRATES) | `is_allowed_edge` | WIRED | Conformance test green |
| `tool_registry::IsnadBackend` | `ToolResult` with `research_checksum` | `IsnadToolBackend` | WIRED | Checksum asserted in test and observed live (CLI + HTTP) |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| `qai isnad narrator` | `results.narrator.name_variants`, `results.rijal_sources` | `SELECT … FROM narrators` / `narrator_rijal_sources` | Yes — live rows rendered verbatim | ✓ FLOWING |
| `qai isnad trace` | `results.paths[].edges[].attrs.label/confidence` | `isnad_edges` → `MemGraphStore` path edges → JSON | Yes — uncertain link suffixed, confirmed link not | ✓ FLOWING |
| `HTTP isnad.narrator` | `data.results.narrator`, `meta.research_checksum` | same service via shared dispatcher | Yes — live HTTP body carried variants + checksum | ✓ FLOWING |
| `qai isnad identity *` | assertion decision/reviewer/decided_at | `narrator_identity_assertions` | Yes — DB rows reflect each verb | ✓ FLOWING |
| TUI isnad screens | — | none (no selection/queue state) | Empty state accepted by human (`b366e64`) | ✓ ACCEPTED |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| Migration set applies | `qai --data-dir /tmp/qai-v7 db migrate` | `migrations applied; schema version 44` | ✓ PASS |
| Narrator record + rijal render | `qai isnad narrator --id narrator:abu-hurayra` (seeded DB) | name + 2 variants + 2 scholar rows | ✓ PASS |
| Per-link uncertainty | `qai isnad trace --from narrator:abu-hurayra --to narrator:malik --k 5` | `… 0.95 (uncertain)` then `… 0.75` (confirmed, no suffix) | ✓ PASS |
| Identity accept recorded | `identity accept --id nia:… --reviewer rev-9` | DB `accepted / rev-9 / decided_at IS NOT NULL` | ✓ PASS |
| Reject requires `--yes` | `identity reject` (no flag) → exit 2; with `--yes` → recorded | no write without flag; `rejected / rev-10` with it | ✓ PASS |
| JSON envelope | `qai --json isnad narrator …` | `tool_name=isnad.narrator`, 64-hex checksum, 2 variants, 2 rijal | ✓ PASS |
| HTTP read path | `POST /api/v1/quran/tool/isnad.narrator` on `qai serve` | `data.tool_name=isnad.narrator`, variants + checksum present | ✓ PASS |
| HTTP mutation gate | `POST … /isnad.identity_accept` | 403 `QAI-ISNAD-0326` | ✓ PASS |
| Isnad-graph conformance | `cargo test -p isnad-graph` | 7 passed, 0 failed | ✓ PASS |
| Application isnad suites | `cargo test -p application --test isnad_reader --test isnad_identity` | 11 + 12 passed, 0 failed | ✓ PASS |
| Server isnad gate test | `cargo test -p server --test api isnad_identity_mutations_are_forbidden_on_http_by_default` | 1 passed | ✓ PASS |
| Full workspace | `cargo test --workspace --no-fail-fast` (HEAD) | 203 suites passed, 0 failed (long soak excluded, re-running separately) | ✓ PASS |

### Probe Execution

Not applicable — no probes declared in any Phase 7 PLAN and no `scripts/*/tests/probe-*.sh` in the repo.

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ---------- | ----------- | ------ | -------- |
| REQ-isnad-narrator-graph | 07-01..07-05 (all five declare it) | Isnad/narrator graph with node/edge types and identity-uncertainty handling (PRD §15) | ✓ SATISFIED | Truths 1–17; `NodeKind::Narrator`, `AssertionKind::NarratorIdentity`, TRANSMITS/NARRATES; `.planning/REQUIREMENTS.md` maps it to Phase 7 and marks it Complete |
| REQ-hadith-corpus | none (not a Phase 7 requirement) | Source-aware hadith records (PRD §14) | ➖ OUT OF SCOPE | `.planning/REQUIREMENTS.md:111` maps it to **Phase 6** (Complete). No Phase 7 plan claims it; Phase 7 only *defers* the `narrator_names_json` backfill (D-07-03). Accounted, not a Phase 7 obligation. |
| REQ-source-catalog-trust | none (not a Phase 7 requirement) | Source catalog + per-source trust profiles (PRD §22, §23) | ➖ OUT OF SCOPE | `.planning/REQUIREMENTS.md:121` maps it to **Phase 11**. No Phase 7 plan claims it; nothing in the isnad implementation depends on it. Accounted, not a Phase 7 obligation. |

**Orphaned requirements:** none. `REQUIREMENTS.md` maps exactly one requirement to Phase 7, and every Phase 7 plan declares it.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| — | — | No `TBD`/`FIXME`/`XXX`/`TODO`/`HACK`/`PLACEHOLDER` in any of the 16 scanned phase files | — | Clean debt-marker scan (blocker gate not triggered) |
| `crates/tool-registry/src/lib.rs` | 2251 | `ISNAD_TOOL_NAMES` (6 entries) is dead — declared, never referenced, and omits `identity_dispute`/`identity_supersede` | ℹ️ Info | Cosmetic; dispatch uses the real name→tool table. Carry from REVIEW IN-01. |
| `crates/application/src/isnad_reader.rs` | 137 | `get_rijal_sources` has no `ORDER BY` | ℹ️ Info | Row order (and thus `research_checksum` for multi-row narrators) is not stably reproducible across runs. Carry from REVIEW IN-06. |
| `docs/05-followups/phase-07-owner-gates.md` | 12 | Says canonical rows live in `docs/05-followups/decisions-needed.md`, but that file's `OD-07` is an unrelated Phase-3.5 coverage-gate row; the Phase 7 gates are not in it | ⚠️ Warning | Inaccurate cross-reference (ID namespace collision). The closing actions themselves are correct and actionable; no owner decision is falsely recorded. |
| `.planning/ROADMAP.md` (Phase 7) / `docs/06-progress/status.md` | — | Phase 7 shows `Plans: TBD` (all five boxes checked) and status.md carries no Phase 7 row | ℹ️ Info | Project-level bookkeeping lag; does not affect delivered behaviour. |
| `.planning/REQUIREMENTS.md` | 121 | `REQ-source-catalog-trust` still shows `Pending` although ROADMAP marks Phase 11 completed (2026-10-19) | ℹ️ Info | Outside Phase 7; noted only because that ID was supplied with this verification request. |

### Human Verification Required

None. The single prior human item (TUI isnad screens render records vs. empty states) was resolved by recorded human validation — empty-state accepted (commit `b366e64`) — and the current tree still matches that accepted shape.

### Gaps Summary

No blockers, no gaps. All three roadmap success criteria were re-proven against the current tree at HEAD, not read from the old report:

1. **View a narrator record with name variants and rijal sources** — live CLI and live HTTP both render the seeded narrator with 2 variants and 2 per-scholar rijal rows.
2. **Trace chain paths with uncertainty marked on each link** — live trace marks the `uncertain` link with `(uncertain)` and leaves the `confirmed` link bare, so the label is genuinely read per link rather than hardcoded.
3. **Reviewer approve/reject a narrator-identity proposal in a recorded workflow** — live accept/reject persist reviewer + `decided_at`, `--yes` gates rejection, and the DB CHECK enforces the invariant.

Changes made after the prior report (Phases 8–12) did not regress Phase 7: the isnad backend still rides the shared name→tool table, the CLI wiring is intact, and all three Phase 7 suites plus the server gate test are green at HEAD. The remaining items are informational (dead `ISNAD_TOOL_NAMES` constant, unordered rijal read, owner-gate cross-reference, project bookkeeping) and none affects goal achievement.

---

_Verified: 2026-11-15_
_Verifier: the agent (gsd-verifier)_
