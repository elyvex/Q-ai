# Agent Prompt — Plan and Implement Phase 2 (Quran Search, Normalization & Linguistics)

> Copy everything inside the fenced block below and give it to the coding agent as its task.

```text
ROLE
You are a senior Rust engineer and technical lead working in the Q-ai repository. You will (1)
produce an execution plan for Phase 2, then (2) begin implementing it, in that order, keeping the
workspace compiling and all gates green after every increment. Phases 0 and 1 are the foundation;
Phase 2 turns the canonical corpus into a searchable, linguistically explorable, fully
explainable research engine — without changing one byte of canonical text.

REPOSITORY
- Root: /Users/ali/dev/rust/Q-ai
- Language/toolchain: Rust, edition 2024, workspace resolver "2", pinned toolchain in
  rust-toolchain.toml. Do not raise/lower the toolchain.
- Phase-2 crates `crates/quran-normalization`, `crates/quran-search`, and
  `crates/quran-morphology` currently exist as EMPTY placeholders
  (`//! Phase 2 placeholder`). You will fill them in; do not rename or relocate them.
- Platform: macOS (darwin), zsh. The workspace currently resolves and builds OFFLINE, so before
  adding any new external crate, confirm it is present in the local Cargo registry cache
  (~/.cargo/registry/). Already confirmed cached: unicode-segmentation, unicode-normalization,
  regex-automata, async-trait. NOT in cache (checked 2026-09-14): tantivy, redb, fst — treat
  Tantivy as unconfirmed until you verify, and keep the ADR-0201 FTS5-fallback option open.

MANDATORY READING (in this order, before writing any code)
1. /Users/ali/dev/rust/Q-ai/AGENTS.md
2. /Users/ali/dev/rust/Q-ai/docs/00-overview/project-overview.md
3. /Users/ali/dev/rust/Q-ai/docs/01-requirements/requirements.md  (the PRD; §8, §9, §11.1–11.3,
   §11.6–11.7, §12, §32.2, §40, §46, §50 are the load-bearing sections for Phase 2)
4. /Users/ali/dev/rust/Q-ai/docs/03-plan/phases/phase-02-rag/plan.md  (full Phase-2 plan)
5. /Users/ali/dev/rust/Q-ai/docs/03-plan/phases/phase-02-rag/README.md
6. /Users/ali/dev/rust/Q-ai/docs/03-plan/phases/phase-02-rag/tasks.md
7. /Users/ali/dev/rust/Q-ai/docs/03-plan/phases/phase-02-rag/acceptance.md
8. /Users/ali/dev/rust/Q-ai/docs/03-plan/phases/phase-02-rag/done.md
9. /Users/ali/dev/rust/Q-ai/docs/03-plan/phases/phase-01-core/done.md §7 (handoff assets you must
   REUSE — canonical rows, token offsets, reference grammar, QuranQuotation, ToolResult contract,
   validator machinery — never re-derive them)
10. /Users/ali/dev/rust/Q-ai/docs/06-progress/status.md
11. The crates you must reuse (read their public APIs, do not re-implement):
    - crates/quran-core (reference grammar, QuranQuotation, edition/surah/ayah/token types)
    - crates/quran-corpus (format, adapters, tokenizer, hashing, validation QV-001..QV-028)
    - crates/domain (ids, ContentHash, SemVer, DataLayer, Diagnostic)
    - crates/storage + crates/storage-sqlite (repositories, migration runner, UnitOfWork::quran())
    - crates/sources (ValidatorRegistry, manifests), crates/jobs (checkpoints, cancellation)
    - crates/provenance (ApprovalToken, CanonicalWriter), crates/audit, crates/observability
    - crates/tools + crates/tool-registry (§12 ToolResult contract), crates/citations (resolver)
    - crates/cli (clap tree, doctor engine), crates/server, crates/testkit, xtask

TOOLING — GET CONTEXT FROM THE GRAPH, NOT FROM BLIND GREP
This repo has a graft code graph. Use it before opening source files:
- `graft map` for orientation; `graft ask "<question>" --source` for ranked answers with spans.
- `graft skeleton <file>` for a file's API; `graft callers <symbol>` for exact edges.
Prefer these over reading whole files. After large changes run `graft build`.

NON-NEGOTIABLE CONSTRAINTS (violating any of these fails the phase)
- I8 Normalization/index builds NEVER mutate canonical text. Derived data goes only to derived
  tables/index dirs at Layer B or D; MV-018 (canonical-unchanged verifier) runs in every build job.
- I9 Every SearchHit carries a NormalizationTrace (ordered rule ids + heuristic flags). No
  constructor without it — mirror the Phase-1 QuranQuotation pattern.
- I10 Every match maps back to exact canonical char/byte/token ranges via bidirectional SpanMap.
- I11 Competing analyses coexist; the schema must contain NO is_correct/is_primary/selected
  column. Preference is a per-request AnalysisPolicy echoed in output with suppression counts.
- I12 Machine-derived rows are Layer D with algorithm + version + confidence; nothing reaches
  human_verified without a reviewer id.
- I13/I15 FamilyRelation is a closed enum; every numeric output carries a complete CountingRules
  block. numeric_report contains no interpretive commentary; interval_analysis and
  missing_expected_form emit their fixed disclaimers verbatim.
- I16 Regex uses a DFA engine only (regex-automata, no backtracking) + 1 MiB/4 MiB size limits +
  512-char patterns + 3 s budget + anchor rules + per-principal rate limits + agent-policy gating.
- Architecture: NONE of quran-normalization/quran-search/quran-morphology may depend on llm,
  embeddings, retrieval, or any vector-store crate. Enforced by `cargo xtask arch-check`.
- Profiles are APPEND-ONLY: a changed rule list requires a new version + full rebuild + drift
  report, never an in-place edit. Drift is a warning (QAI-IDX-0101), never an auto-repair.
- Never generate roots/lemmas with an LLM. Computational analyzers write Layer D rows with
  Attribution::Computational + confidence + needs_review only.
- Deny-by-default everywhere; no network, no filesystem escapes, no secret leakage.
- Never modify plan.md. Update tasks.md, acceptance.md, and done.md as you progress.
- Do NOT commit unless the human asks. Do not create git branches, tags, or push.

BLOCKERS AND HOW TO HANDLE THEM
- ADR-0203 (morphology dataset + license + alignment + attribution) is a legal/data decision you
  cannot make. Do NOT invent linguistic data. Instead: implement the full multi-analysis schema,
  importer, alignment validator, and tools against a small, clearly non-authoritative
  PUBLIC-DOMAIN test lexicon covering the fixture edition; tools degrade to typed "dataset
  unavailable" errors naming the missing capability. Write ADR-0203 as a DRAFT; never mark it
  Accepted. This is the documented fallback in README §8.
- The Arabic linguist (0.4 FTE, P2-X02) is a hiring decision you cannot make. Write the rule
  catalog, tagset, root convention, and golden sets so a linguist can review them, mark every
  affected sign-off (AC-P2-02/03/23/24/25/46) as pending linguist review, and record P2-X02 in
  done.md §7. Never present your own judgment as linguist sign-off.
- The estimate reconciliation (README §9.1: rows sum to 278.0 ed, plan says ≈112 — a 2.5x gap,
  ~18.5 weeks not 8) and the Sprint 2.3–2.6 overload (README §9.2, each 42–51 ed) are OWNER
  decisions. Do not silently compress estimates. Propose the 2.3a/2.3b + 2.4a/2.4b splits in your
  plan and proceed with the smaller of the agreed units; flag the decision in done.md §7.
- FTS backend (ADR-0201) is provisional: Tantivy is the plan default but is NOT in the offline
  cache. Verify availability first; if adding it breaks the offline build, fall back to the
  FTS5-backed implementation of the same FullTextIndex trait, and record the decision in ADR-0201
  before P2-T30. The trait boundary must survive either choice.

SCOPE (Phase 2 only — see plan.md §2 and README §4)
IN: normalization rules N01–N24 + pipeline + profiles L0–L8 + SpanMap + traces; derived forms +
skeletons; FullTextIndex trait + backend + ar_* tokenizers; the 5 search tools
(exact/normalized/concatenated/phrase/regex); morphology import (12 checkpoints) + MV-001..018
+ alignment + lexicons; the 6 morphology tools + word-family engine; the 13 counting/discovery
tools + CountingRules; index lifecycle (build/rebuild/verify/gc, manifests, drift,
reconciliation, cache); search API v1 + SSE + normalization preview; CLI search/linguistics/
counting/normalize/dataset/index groups; doctor --indexes (19 checks); eval harness + golden/
property suites; D2.13 docs.
OUT (do not build): graph nodes/edges (Phase 3), semantic/vector search (Phase 7), web UI
search page + word inspector (Phase 4), rhetorical/structural tools except near_duplicates
(Phase 3), tafsir/hadith/scripture search (Phase 5/8), transliteration/phonetics beyond reserved
stubs (Phase 4), full fuzzy quality beyond experimental L8 (Phase 4), reranking/query
decomposition (Phase 7).

STEP 1 — PRODUCE A PLAN (before coding)
Write `docs/03-plan/phases/phase-02-rag/execution-plan.md` containing:
- the sequence of work in small, independently verifiable increments (call them M1..Mn), each
  mapped to P2-Tnn tasks and D2.x deliverables;
- the exact files/modules you will create per increment;
- the new workspace dependencies you will add and which cached crate/version you will use
  (with the Tantivy-vs-FTS5 decision and its trigger);
- the arch-check allowlist entries you will add;
- the test suites (from acceptance.md §3.2) that each increment turns green;
- an explicit statement of which owner decisions you are blocked on and how you will proceed
  without them (see BLOCKERS);
- a risk list mirroring README §13 with the concrete mitigation you will implement.
Keep the plan honest about the 112-vs-278 ed discrepancy; do not present 278 ed of work as
fitting in 8 weeks.

STEP 2 — IMPLEMENT, INCREMENTALLY, KEEPING THE BUILD GREEN
Work in this order; do not start an increment until the previous one builds and tests green:
- M1  Normalization engine: RuleId/NormalizationRule trait + RuleKind, deterministic rules
  N01–N17, heuristic N18–N22 with Heuristic tagging, NormalizationPipeline + profile registry
  (append-only) + NormalizationTrace with no-default-constructor guard; migration 0020.
  SpanMap (segments, compose, to_canonical, to_derived) + all 5 properties × every ayah ×
  every profile + idempotency/associativity/fuzz batteries; 2,000-pair golden harness green.
  CLI normalize --explain/--list-profiles/--show-rule + preview/profiles endpoints.
  (P2-T13..T24; ADR-0204/0205 drafts)
- M2  Derived forms + FTS foundation: migration 0021 (token/ayah forms, skeletons) +
  forms.rebuild job + skeleton builder (ayah + 3-ayah windows) + MV-018 wiring; FullTextIndex
  trait + manifest + FtsQuery/SearchOpts; backend schema/writer/reader; ar_* tokenizers on the
  SHARED pipeline instance + 5,000-substring parity test; migration 0023 + index_pointers;
  index.build job (staging → verify → atomic flip) + gc/rollback; trigram skeleton postings;
  crash/cancel matrix; cold-rebuild < 6 min gate. (P2-T25..T39; ADR-0201/0208/0213)
- M3  Search tools: SearchHit/ScoreExplain + unified assembly; exact (+ zero-result hint),
  normalized (ad-hoc rules + explain), phrase (slop), concatenated (candidate → verify →
  segmentation + cross-ayah dedup), regex (all I16 guards); total_matches path; filters;
  highlighting; cache migration 0025 + generation invalidation + LRU; API + SSE; CLI group;
  400-query golden + DoS abuse + latency-bench suites. (P2-T40..T56; ADR-0207/0212/0214)
- M4  Morphology import + lexicons: migrations 0022 + 0024; intermediate format + JSON Schema;
  chosen-dataset adapter + second adapter proving extensibility; DirectKey + AlignmentTable
  engine with per-surah unmatched reporting; tagset mapper (native tags verbatim); MV-001..018;
  lexicon builder; unification as review_queue suggestions (never merge); import job (12
  checkpoints) + approval-gated activation (enqueues FTS rebuild); coverage reports + threshold
  gate; differ; Layer B/D provenance; 18-fault adversarial + crash-matrix suites; FTS lexicon
  fields. (P2-T57..T74; ADR-0209)
- M5  Morphology + family tools: AnalysisPolicy + suppression reporting; morphology,
  morphology_compare (verdicts, NO winner field), root_search, lemma_search, pattern_search
  (+ capability-unavailable path), affix_search (backend labeled Attested vs Heuristic);
  FamilyRelation taxonomy + 5-path resolution + relation builders + per-member explanations +
  opt-in suggestions with confidence + mandatory labels + review_queue promotion to
  ScholarVerified; API + CLI; 500-case root/lemma + 120-family curated + non-merge suites.
  (P2-T75..T93)
- M6  Counting/discovery/doctor/eval: CountingRules type; frequency (exact SQL, never FTS
  counts), distribution (+ partition provenance), cooccurrence, collocation (PMI+LLR+t-score),
  first/last + intervals (+ disclaimer), numeric_report (+ checksum, no commentary), hapax,
  unusual_usage, near_duplicates (MinHash + verify), missing_expected_form (+ disclaimer);
  API + CLI; 19 doctor checks incl. search.smoke + drift reporting (QAI-IDX-0101) + nightly
  reconcile + eval harness + determinism + 22-tool contract suites. (P2-T94..T113; ADR-0216)
- M7  Hardening/soak/handoff: full soak (rebuild → 50k randomized queries → doctor →
  reconcile, zero findings/panics/hash changes); the 6 D2.13 docs (normalization spec,
  profile catalog, search cookbook, adapter guide, counting-rules explainer, reindex runbook);
  exit-gate review + handoff-p2-to-p3.md. (P2-T111, T113, T114)
If an increment depends on a human decision (dataset, linguist, FTS backend), keep going on
the test lexicon / fallback and mark the decision in done.md §7.

ENGINEERING RULES
- Mimic Phase-0/1 code style exactly. Read a neighbouring file before writing a new one.
- New external deps go in root Cargo.toml [workspace.dependencies], referenced as { workspace
  = true }. Verify offline-cache presence FIRST; never break the offline build.
- Extend `xtask/allowlist.toml` for the new path edges BEFORE adding them, or arch-check fails
  (fail-closed by design). Expected edges: quran-normalization -> domain;
  quran-search -> quran-normalization, quran-core, domain, storage;
  quran-morphology -> quran-normalization, quran-core, domain, sources, storage;
  storage-sqlite -> +quran-core (already); application -> +quran-normalization, +quran-search,
  +quran-morphology. NO edges to llm/embeddings/retrieval/vector crates — ever.
- Every error type implements the Phase-0 `Diagnostic` trait with code + summary + remedy +
  next_command. Use QAI-NORM-nnnn / QAI-IDX-nnnn; QAI-QUR-* stays Phase-1-owned.
- Every derived row records corpus_generation + rule_set/dataset/tokenizer versions +
  provenance_id; every mutation writes provenance/audit; every long operation supports
  cancellation + checkpoints + resume.
- Heuristic (L7/N18–N21) and computational outputs are labeled in CLI, API, AND tool payloads
  (snapshot-tested copy), not just in docs.
- No `unsafe` (workspace forbids it). No comments unless they add non-obvious value.
- Determinism: no wall-clock or randomness in hashes/checksums/counts; counts from SQL, never
  from FTS term frequencies.

VERIFICATION — AFTER EVERY INCREMENT (definition of done)
Run and make green:
  cargo fmt --all
  cargo clippy --workspace --all-targets -- -D warnings
  cargo test --workspace
  cargo run -p xtask -- arch-check
  cargo run -p xtask -- migrate-check
(If cargo-deny is unavailable locally, note it; CI runs it.) For persistence/index work, test
against REAL SQLite/Tantivy (or FTS5 fallback) via a tempdir, never a mock. For every
acceptance criterion you claim, add/point to the automated test named in acceptance.md §3.2.

DOCUMENTATION / STATUS UPDATES (in the same commit as the code)
- Flip P2-Tnn statuses in tasks.md.
- Append a completed-task entry to done.md using its exact entry format (date, owner, PR/commit,
  evidence = test path, DoD, notes). Append, never edit.
- Record any ADR you write in done.md §4; keep ADR-0203 and linguist-gated ADRs as drafts if human
  input is missing.
- Add deviations to done.md §5 and deferrals to §7 with owners.
- Update docs/06-progress/task-done-rollup.md and CHANGELOG.md per AGENTS.md.
- Regenerate schemas where relevant: `cargo run -p xtask -- gen-schema`.
- Run `graft build` after large structural changes.

WHAT TO RETURN TO ME
1. The path to execution-plan.md and a short summary of M1..M7.
2. The list of owner decisions you are blocked on, each with your interim fallback.
3. For the increments you completed this session: the files created, the new dependencies added,
   the allowlist entries added, and the exact commands you ran with their result.
4. The current status: `cargo test --workspace` count, arch-check/migrate-check result, and which
   P2-Tnn tasks are now ☑ in tasks.md.
5. Any deviation from plan.md, recorded in done.md §5.

WHAT NOT TO DO
- Do not build graph nodes/edges, semantic/vector search, the web UI, rhetorical tools (beyond
  near_duplicates), tafsir/hadith/scripture search, transliteration/phonetics beyond reserved
  stubs, full fuzzy quality, reranking, or any LLM/vector functionality — those are later phases.
- Do not add llm/embeddings/retrieval/vector-store dependencies to the three Phase-2 crates.
- Do not invent, download, or hardcode real morphology data; use the test lexicon until a licensed
  dataset is supplied. Do not present your own judgment as linguist sign-off.
- Do not merge competing analyses, do not auto-repair drift, do not serve stale cache across a
  generation bump, do not silently fold under L0.exact.
- Do not modify plan.md, do not commit/push, do not mark tasks done that fail their acceptance test.
- Do not silently resolve the estimate discrepancy or the sprint overloads; surface them.

START NOW with STEP 1: read the required documents, orient with graft, verify Tantivy cache
presence (or commit to the FTS5 fallback path in ADR-0201), and write execution-plan.md.
Then begin M1 (normalization engine) and proceed increment by increment, verifying after each.
```
