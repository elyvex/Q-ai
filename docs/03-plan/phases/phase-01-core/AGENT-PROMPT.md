# Agent Prompt — Plan and Implement Phase 1 (Canonical Quran Core)

> Copy everything inside the fenced block below and give it to the coding agent as its task.

```text
ROLE
You are a senior Rust engineer and technical lead working in the Q-ai repository. You will (1)
produce an execution plan for Phase 1, then (2) begin implementing it, in that order, keeping the
workspace compiling and all gates green after every increment. Phase 0 is complete; Phase 1 builds
the canonical Quran engine on top of it.

REPOSITORY
- Root: /Users/ali/dev/rust/Q-ai
- Language/toolchain: Rust, edition 2024, workspace resolver "2", pinned toolchain in
  rust-toolchain.toml. Do not raise/lower the toolchain.
- Workspace members are in the root Cargo.toml. Phase-1 crates `crates/quran-core`,
  `crates/quran-corpus`, and `crates/citations` currently exist as EMPTY placeholders
  (`//! Phase 1 placeholder`). You will fill them in; do not rename or relocate them.
- Platform: macOS (darwin), zsh. The workspace currently resolves and builds OFFLINE, so before
  adding any new external crate, confirm it is present in the local Cargo registry cache
  (~/.cargo/registry/). Prefer the crates listed in the Phase-1 technology register.

MANDATORY READING (in this order, before writing any code)
1. /Users/ali/dev/rust/Q-ai/AGENTS.md
2. /Users/ali/dev/rust/Q-ai/docs/00-overview/project-overview.md
3. /Users/ali/dev/rust/Q-ai/docs/01-requirements/requirements.md  (the PRD; §7, §12, §34, §35.1, §40,
   §46, §50, §55, §56 are the load-bearing sections for Phase 1)
4. /Users/ali/dev/rust/Q-ai/docs/03-plan/phases/phase-01-core/plan.md  (full Phase-1 plan)
5. /Users/ali/dev/rust/Q-ai/docs/03-plan/phases/phase-01-core/README.md
6. /Users/ali/dev/rust/Q-ai/docs/03-plan/phases/phase-01-core/tasks.md
7. /Users/ali/dev/rust/Q-ai/docs/03-plan/phases/phase-01-core/acceptance.md
8. /Users/ali/dev/rust/Q-ai/docs/03-plan/phases/phase-01-core/technology-stack.md
9. /Users/ali/dev/rust/Q-ai/docs/03-plan/phases/phase-01-core/done.md
10. /Users/ali/dev/rust/Q-ai/docs/06-progress/status.md
11. The Phase-0 crates you must reuse (read their public APIs, do not re-implement):
    - crates/domain (ids, ContentHash, SemVer, Timestamp, LicenseRecord, DataLayer, TrustLevel,
      DerivationVersions, Diagnostic)
    - crates/storage + crates/storage-sqlite (Database, UnitOfWork, repositories, migration runner)
    - crates/provenance (ApprovalToken, CanonicalWriter, CanonicalChangeRequest/Session)
    - crates/sources (manifest, StructureValidator registry, DifferenceReport)
    - crates/jobs (JobHandler, checkpoints, cancellation, idempotency)
    - crates/audit (hash-chained events), crates/observability (spans, metric catalog)
    - crates/cli (clap tree, doctor engine), crates/server (health server)
    - crates/testkit (fixtures), xtask (arch-check, migrate-check, gen-schema, ci)

TOOLING — GET CONTEXT FROM THE GRAPH, NOT FROM BLIND GREP
This repo has a graft code graph. Use it before opening source files:
- `graft map` for orientation; `graft ask "<question>" --source` for ranked answers with spans.
- `graft skeleton <file>` for a file's API; `graft callers <symbol>` for exact edges.
Prefer these over reading whole files. After large changes run `graft build`.

NON-NEGOTIABLE CONSTRAINTS (violating any of these fails the phase)
- I1 Canonical ayah text is immutable per edition version. Canonical tables are insert-only with
  DB triggers; writes only via the Phase-0 CanonicalWriter/ApprovalToken path. No bypass API.
- I2 Canonical text is never produced by a model. `quran-core` may depend only on {domain, serde,
  thiserror} (+ Unicode crates); neither `quran-core` nor `quran-corpus` may depend on
  `llm`, `embeddings`, `retrieval`, or any vector store. This is enforced by `cargo xtask arch-check`.
- I3 Different readings/editions are never merged. `edition_id` is in every canonical primary key.
- I4 Token order is stable/verifiable; per-ayah order checksum + unique (edition,surah,ayah,position).
- I5 A partially imported edition can never become Active. Staging tables are physically separate;
  activation is a single transaction ending in a pointer flip; the importer holds no ApprovalToken.
- I6 Every returned quotation carries edition id + version + hash. No QuranQuotation can be
  constructed without them.
- I7 Corrections create a new version with a DifferenceReport and human approval.
- Principle 5: translations are never presented as the original; `AyahView.canonical` cannot hold a
  translation and `AttributedTranslation` requires a non-empty translator + edition_ref.
- Deny-by-default everywhere; no network, no filesystem escapes, no secret leakage.
- Never modify `plan.md`. Update `tasks.md`, `acceptance.md`, and `done.md` as you progress.
- Do NOT commit unless the human asks. Do not create git branches, tags, or push.

BLOCKERS AND HOW TO HANDLE THEM
- ADR-0101 (initial Quran dataset + license) is a legal/editorial decision you cannot make. Do NOT
  invent or download a Quran text. Instead:
    * Implement the general machinery against a small, clearly non-canonical SYNTHETIC test edition
      (`fixtures/quran/test-edition-min/`, ~5 surahs, ASCII/placeholder Arabic-shaped tokens) plus
      the 16 adversarial corpora. This is the documented fallback in README §8.
    * Write ADR-0101 as a DRAFT with the bundle-vs-user-supplied decision marked "pending human
      sign-off"; never mark it Accepted.
  Until a real approved dataset is supplied by a human, the canonical pipeline must work end to end
  on the synthetic fixture.
- ADR-0114 (reference corpus + sign-off) is also human; write it as a draft and make QV-015 skip
  (not silently pass) when no reference corpus is configured.
- The estimate reconciliation (README §9.1: rows sum to 131 ed, plan says 82) and the Sprint 1.2
  split (README §9.2) are OWNER decisions. Do not silently compress estimates. Propose the split in
  your plan and proceed with the smaller of the agreed units; flag the decision in done.md §7.
- HTTP framework for API v1 is an open choice (technology-stack.md §4). Default recommendation is
  `axum` + `tower-http` (both cached). Treat it as a provisional decision, record it in an ADR before
  P1-T39, and keep the Phase-0 health endpoints working.

SCOPE (Phase 1 only — see plan.md §2 and README §4)
IN: quran-core domain + reference grammar; edition source format + adapters; tokenizer + separators +
hashing; corpus validator QV-001..QV-028; importer job with 13 checkpoints; staging + atomic
activation + rollback + corpus generation; edition differ; QuranReader lookup + context + cache; the
Quran read API v1; ToolResult contract + quran.get_ayah/quran.get_context; citation resolver v1;
CLI `qai quran …`; `qai doctor --quran`; a minimal debug reader; the corpus integrity test suite;
the D1.14 docs.
OUT (do not build): all search/normalization/morphology (Phase 2), graph (Phase 3), web reading UI
(Phase 4), tafsir/hadith/scripture (Phase 5/8), recitation/tajwid/qira'at, agents/LLM (Phase 9).

STEP 1 — PRODUCE A PLAN (before coding)
Write `docs/03-plan/phases/phase-01-core/execution-plan.md` containing:
- the sequence of work in small, independently verifiable increments (call them M1..Mn), each
  mapped to P1-Tnn tasks and D1.x deliverables;
- the exact files/modules you will create per increment;
- the new workspace dependencies you will add and which cached crate/version you will use;
- the arch-check allowlist entries you will add;
- the test suites (from acceptance.md §3.2) that each increment turns green;
- an explicit statement of which owner decisions you are blocked on and how you will proceed
  without them (see BLOCKERS);
- a risk list mirroring README §13 with the concrete mitigation you will implement.
Keep the plan honest about the 82-vs-131 ed discrepancy; do not present 131 ed of work as fitting
in 6 weeks.

STEP 2 — IMPLEMENT, INCREMENTALLY, KEEPING THE BUILD GREEN
Work in this order; do not start an increment until the previous one builds and tests green:
- M1  quran-core skeleton: newtypes (SurahNumber/AyahNumber/TokenPosition), enums (Script,
      NumberingScheme, BasmalaPolicy, UnicodeForm, RevelationPlace, SajdahKind, EditionStatus,
      SegmentKind, EditionSelector), structs (QuranEdition, EditionStatistics, Surah, Ayah, Segment,
      Token), QuranQuotation, and the QAI-QUR-01xx error codes. (P1-T06, T07, T10)
- M2  Reference grammar: hand-written parser + serializer per plan §3.3 EBNF, with all variants and
      exhaustive coded errors; golden file with 300 cases; `parse(serialize(ref)) == ref` property.
      (P1-T08, T09; ADR-0102 draft)
- M3  Intermediate format types + JSON Schema; adapter trait + JSON adapter + a second adapter
      (CSV) proving extensibility; the synthetic + adversarial fixtures. (P1-T15, T16, T17; P1-T05)
- M4  Tokenizer (whitespace-preserving) + separators + byte/char/grapheme offsets; hashing
      text_hash/structure_hash/token_order_hash; round-trip and offset property tests. (P1-T21,
      T22, T23; ADR-0104/0105/0108 drafts)
- M5  Validator: all QV-001..QV-028 with severities and a machine-readable ValidationReport; the 16
      adversarial fixtures must each fail with the SPECIFIC rule id. (P1-T18, T19, T20; also T30)
- M6  Migrations 0010..0015 (+ checksums.json update) and repository read paths; raw-SQL trigger
      tests and the immutability suite. (P1-T12, T13, T14, T24)
- M7  Importer job with 13 checkpoints, cancellation, resume + activation transaction + rollback +
      corpus_generation + differ. Crash matrix at all 13 checkpoints. (P1-T25..T29, T31)
- M8  QuranReader (get_ayah/get_ayahs/get_tokens/get_context/divisions) + cache keyed by corpus
      generation; property tests for context boundaries; perf smoke. (P1-T32..T35; P1-T41)
- M9  API v1 + envelope + ETag/content-language + error body; ToolResult contract + the two tools;
      citation resolver + deep links; CLI; doctor --quran; debug reader; docs. (P1-T36..T54, T59)
- M10 Hardening/soak: golden-set expansion, full property suite, full-corpus import→validate→activate
      →10k lookups→doctor --deep. (P1-T56..T58)
If a later increment depends on a human decision (dataset, reference corpus, HTTP framework), keep
going on the synthetic fixture and mark the decision in done.md §7.

ENGINEERING RULES
- Mimic Phase-0 code style exactly. Read a neighbouring file before writing a new one.
- New external deps go in root Cargo.toml [workspace.dependencies], referenced as { workspace = true }.
- Extend `xtask/allowlist.toml` for the new path edges BEFORE adding them, or arch-check fails
  (fail-closed by design). Expected edges: quran-core -> domain; quran-corpus -> quran-core, domain,
  sources, storage; citations -> quran-core, domain; storage-sqlite -> +quran-core; application ->
  +quran-core, +quran-corpus, +citations; server -> +application (already allowed), cli -> application.
- Every error type implements the Phase-0 `Diagnostic` trait with code + summary + remedy +
  next_command. Use the reserved QAI-QUR-nnnn namespace.
- Every mutation writes provenance/audit; every long operation supports cancellation + checkpoints.
- No `unsafe` (workspace forbids it). No comments unless they add non-obvious value.
- Determinism: no wall-clock or randomness in hashes/checksums.

VERIFICATION — AFTER EVERY INCREMENT (definition of done)
Run and make green:
  cargo fmt --all
  cargo clippy --workspace --all-targets -- -D warnings
  cargo test --workspace
  cargo run -p xtask -- arch-check
  cargo run -p xtask -- migrate-check
(If cargo-deny is unavailable locally, note it; CI runs it.) For persistence work, test against REAL
SQLite via a tempdir, never a mock. For every acceptance criterion you claim, add/point to the
automated test named in acceptance.md §3.2.

DOCUMENTATION / STATUS UPDATES (in the same commit as the code)
- Flip P1-Tnn statuses in tasks.md.
- Append a completed-task entry to done.md using its exact entry format (date, owner, PR/commit,
  evidence = test path, DoD, notes). Append, never edit.
- Record any ADR you write in done.md §4; keep ADR-0101/0114 as drafts if human input is missing.
- Add deviations to done.md §5 and deferrals to §7 with owners.
- Update docs/06-progress/task-done-rollup.md and CHANGELOG.md per AGENTS.md.
- Regenerate schemas where relevant: `cargo run -p xtask -- gen-schema`.
- Run `graft build` after large structural changes.

WHAT TO RETURN TO ME
1. The path to execution-plan.md and a short summary of M1..M10.
2. The list of owner decisions you are blocked on, each with your interim fallback.
3. For the increments you completed this session: the files created, the new dependencies added, the
   allowlist entries added, and the exact commands you ran with their result.
4. The current status: `cargo test --workspace` count, arch-check/migrate-check result, and which
   P1-Tnn tasks are now ☑ in tasks.md.
5. Any deviation from plan.md, recorded in done.md §5.

WHAT NOT TO DO
- Do not implement search, normalization, morphology, graph, web UI, hadith, tafsir, audio, or any
  LLM/vector functionality — those are later phases.
- Do not add `llm`/`embeddings`/`retrieval`/vector-store dependencies anywhere in the Quran crates.
- Do not invent, download, or hardcode real Quran text; use the synthetic fixture until a licensed
  dataset is supplied.
- Do not modify plan.md, do not commit/push, do not mark tasks done that fail their acceptance test.
- Do not silently resolve the estimate discrepancy or the Sprint 1.2 overload; surface them.

START NOW with STEP 1: read the required documents, orient with graft, and write execution-plan.md.
Then begin M1 (quran-core) and proceed increment by increment, verifying after each.
```
