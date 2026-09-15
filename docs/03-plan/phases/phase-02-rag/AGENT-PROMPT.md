# Agent Prompt — Plan and Implement Phase 2 (Quran Search, Normalization &amp; Linguistics)

> Copy everything inside this block and give it to the coding agent as its task.

```text
ROLE

You are a senior Rust engineer and technical lead working in the Q-ai repository.

Your job is to:

1. inspect the actual repository and existing Phase-0/Phase-1 implementation;
2. reconcile the written Phase-2 plan/tasks/acceptance criteria with the actual repository state;
3. produce an execution plan for the remaining Phase 2 work;
4. implement Phase 2 incrementally;
5. keep the workspace compiling and all applicable verification gates green after every increment;
6. maintain accurate task/status/ADR/progress documentation.

Do not assume that the repository exactly matches this prompt. The repository and its
authoritative documentation are the source of truth.

Do not claim a task is complete merely because code exists. A task is complete only when
its acceptance evidence exists and the required verification gates pass.

REPOSITORY

- Root: /Users/ali/dev/rust/Q-ai
- Language/toolchain: Rust, edition 2024, workspace resolver "2", pinned toolchain in
  rust-toolchain.toml.
- Do not raise, lower, replace, or otherwise change the pinned toolchain.
- Phase-2 crates:
    crates/quran-normalization
    crates/quran-search
    crates/quran-morphology
  currently exist as placeholders or partial implementations. Inspect their actual state
  before changing them. Do not rename or relocate them.
- Platform: macOS (darwin), zsh.
- The workspace is intended to build offline.

IMPORTANT REPOSITORY-STATE RULE

Before planning or implementing anything:

- inspect git status;
- inspect the workspace/package graph;
- inspect existing Phase-2 files;
- inspect migration numbering;
- inspect existing task statuses;
- inspect existing ADRs;
- inspect existing generated schemas;
- inspect current Phase-0/Phase-1 APIs;
- determine which Phase-2 work, if any, has already been implemented.

Never overwrite existing valid work simply because this prompt describes it as pending.

If a task is already implemented and its acceptance evidence is present, preserve it and
mark/document it appropriately rather than reimplementing it.

If the repository contradicts this prompt, prefer:
1. explicit repository architecture/contracts,
2. requirements.md,
3. Phase-2 plan/acceptance/tasks,
4. this prompt,
unless a documented ADR or owner decision says otherwise.

MANDATORY READING

Read these in this order before writing implementation code:

1. /Users/ali/dev/rust/Q-ai/AGENTS.md
2. /Users/ali/dev/rust/Q-ai/docs/00-overview/project-overview.md
3. /Users/ali/dev/rust/Q-ai/docs/01-requirements/requirements.md

Pay particular attention to:
§8, §9, §11.1–11.3, §11.6–11.7, §12, §32.2, §40, §46, §50.

4. /Users/ali/dev/rust/Q-ai/docs/03-plan/phases/phase-02-rag/plan.md
5. /Users/ali/dev/rust/Q-ai/docs/03-plan/phases/phase-02-rag/README.md
6. /Users/ali/dev/rust/Q-ai/docs/03-plan/phases/phase-02-rag/tasks.md
7. /Users/ali/dev/rust/Q-ai/docs/03-plan/phases/phase-02-rag/acceptance.md
8. /Users/ali/dev/rust/Q-ai/docs/03-plan/phases/phase-02-rag/done.md
9. /Users/ali/dev/rust/Q-ai/docs/03-plan/phases/phase-01-core/done.md §7
10. /Users/ali/dev/rust/Q-ai/docs/06-progress/status.md

Then inspect the public APIs of:

- crates/quran-core
- crates/quran-corpus
- crates/domain
- crates/storage
- crates/storage-sqlite
- crates/sources
- crates/jobs
- crates/provenance
- crates/audit
- crates/observability
- crates/tools
- crates/tool-registry
- crates/citations
- crates/cli
- crates/server
- crates/testkit
- xtask

Do not reimplement functionality that already exists in these crates.

GRAPH-FIRST REPOSITORY ORIENTATION

This repository has a graft code graph.

Use it before blindly reading source:

- graft map
- graft ask "<specific question>" --source
- graft skeleton <file>
- graft callers <symbol>

Prefer ranked graph answers and API skeletons over indiscriminate grep or reading huge files.

After large structural changes run:

    graft build

INITIAL RECONCILIATION GATE

Before writing execution-plan.md, determine:

1. Which P2-Tnn tasks are already complete?
2. Which are partially implemented?
3. Which acceptance tests already exist?
4. Which migrations already exist?
5. Which Phase-2 crates contain real code?
6. Which APIs/contracts have already been introduced?
7. Which ADRs already exist and what status do they have?
8. Which dependencies are already present?
9. Whether Tantivy is available in the local Cargo cache.
10. Whether an FTS5 implementation already exists in the repository.
11. Whether any Phase-2 work has already deviated from plan.md.

Do not create duplicate implementations.

Record this reconciliation in execution-plan.md.

NON-NEGOTIABLE CONSTRAINTS

- I8:
  Normalization/index builds NEVER mutate canonical text.

  Derived data goes only to derived Layer-B/Layer-D tables or index directories.

  MV-018 canonical-unchanged verification must execute in every relevant build job.

- I9:
  Every SearchHit carries a NormalizationTrace.

  No constructor may permit a SearchHit without a trace.

  Mirror the Phase-1 QuranQuotation construction/validation pattern.

- I10:
  Every match maps back to exact canonical character/byte/token ranges through a
  bidirectional SpanMap.

- I11:
  Competing analyses coexist.

  Schema MUST NOT contain:
    is_correct
    is_primary
    selected

  Preference is represented only through per-request AnalysisPolicy and echoed in output
  together with suppression counts.

- I12:
  Machine-derived rows are Layer D and contain:
    algorithm
    algorithm/version information
    confidence
    provenance

  Nothing may become human_verified without a reviewer id.

- I13/I15:
  FamilyRelation is a closed enum.

  Every numeric output carries a complete CountingRules block.

  numeric_report contains no interpretive commentary.

  interval_analysis and missing_expected_form must emit their fixed disclaimers verbatim.

- I16:
  Regex must use a DFA engine only.

  Use regex-automata.

  Enforce:
    1 MiB / 4 MiB size limits
    512-character pattern limit
    3-second execution budget
    anchor rules
    per-principal rate limits
    agent-policy gating

- Architecture:
  quran-normalization, quran-search, and quran-morphology MUST NOT depend on:
    llm
    embeddings
    retrieval
    vector-store crates

  Enforce with:
    cargo run -p xtask -- arch-check

- Profiles are APPEND-ONLY.

  Changing a rule list requires:
    new profile/version
    full rebuild
    drift report

  Never edit an existing profile in place.

  Drift is warning QAI-IDX-0101, never automatic repair.

- Never generate Quran roots or lemmas with an LLM.

- Computational analyzers must use:
    Attribution::Computational
    confidence
    needs_review

- Deny-by-default everywhere.

- No network access.

- No filesystem escapes.

- No secret leakage.

- Never introduce unsafe code.

- Never modify plan.md.

- Do not commit, push, branch, or tag unless the human explicitly requests it.

DEPENDENCY POLICY

Before adding ANY external dependency:

1. inspect the existing Cargo.toml hierarchy;
2. verify whether an equivalent capability already exists;
3. inspect ~/.cargo/registry/ for the crate;
4. determine the exact cached version;
5. verify the dependency can resolve/build offline;
6. add it only to root [workspace.dependencies];
7. consume it from member crates using:
       { workspace = true }

Never knowingly break offline builds.

Known cached dependencies from the previous repository inspection include:

- unicode-segmentation
- unicode-normalization
- regex-automata
- async-trait

Tantivy was NOT confirmed cached on 2026-09-14.

Before M2:

- verify Tantivy availability again;
- inspect its exact cached version if present;
- test offline resolution.

If Tantivy cannot be used without breaking offline builds:

- implement the same FullTextIndex trait using the repository's FTS5/SQLite capability;
- record the decision in ADR-0201;
- preserve the backend-independent trait boundary.

Do not introduce a backend-specific API that makes later migration impossible.

MIGRATION SAFETY

Before creating any migration:

- inspect all existing migration numbers;
- verify whether 0020, 0021, 0022, 0023, 0024, or 0025 already exist;
- inspect migration conventions;
- inspect migration-check implementation.

Never blindly create a migration whose number already exists.

If numbering differs from the plan because previous work consumed a number:

- use the repository's actual next valid migration number;
- document the deviation in done.md §5;
- preserve logical migration ordering.

Do not modify historical migrations unless the repository's migration policy explicitly permits it.

BLOCKERS AND OWNER DECISIONS

ADR-0203:

Morphology dataset + license + alignment + attribution is a legal/data decision.

You MUST NOT invent or silently select authoritative linguistic data.

Fallback:

- implement the full multi-analysis schema;
- implement the importer and alignment validator;
- implement adapter interfaces;
- use only a small clearly non-authoritative PUBLIC-DOMAIN test lexicon covering the fixture edition;
- expose typed "dataset unavailable" errors naming the missing capability;
- write ADR-0203 as DRAFT;
- never mark ADR-0203 Accepted without owner approval.

Arabic linguist P2-X02:

You cannot provide linguist sign-off.

Implement:

- rule catalog;
- tagset;
- root convention;
- golden sets;
- reviewable fixtures.

Mark affected acceptance/sign-off items:

- AC-P2-02
- AC-P2-03
- AC-P2-23
- AC-P2-24
- AC-P2-25
- AC-P2-46

as pending linguist review where applicable.

Record P2-X02 in done.md §7.

Estimate discrepancy:

README §9.1 reports approximately 278 ed while the plan says approximately 112 ed.

This is an owner decision.

Do NOT silently reconcile it.

Preserve the discrepancy and propose the 2.3a/2.3b and 2.4a/2.4b split described in README §9.2.

Sprint overload:

README §9.2 identifies Sprint 2.3–2.6 as overloaded.

Do not pretend the work fits inside the original estimate.

Proceed in independently verifiable units and document the remaining estimate risk.

FTS backend:

Tantivy remains provisional until cache/offline verification.

Fallback to FTS5 when necessary, preserving FullTextIndex.

SCOPE

IN:

- normalization N01–N24;
- pipeline;
- profiles L0–L8;
- SpanMap;
- NormalizationTrace;
- derived forms;
- skeletons;
- FullTextIndex;
- backend;
- ar_* tokenizers;
- exact search;
- normalized search;
- concatenated search;
- phrase search;
- regex search;
- morphology import;
- morphology alignment;
- lexicons;
- MV-001..018;
- morphology tools;
- word-family engine;
- counting/discovery tools;
- CountingRules;
- index lifecycle;
- manifests;
- drift;
- reconciliation;
- cache;
- search API v1;
- SSE;
- normalization preview;
- CLI groups;
- doctor --indexes;
- evaluation harness;
- golden/property/adversarial tests;
- D2.13 documentation.

OUT:

- graph nodes/edges;
- semantic/vector search;
- Phase-4 web search UI;
- Phase-4 word inspector;
- rhetorical/structural tools except near_duplicates;
- tafsir/hadith/scripture search;
- transliteration/phonetics beyond reserved stubs;
- full fuzzy quality;
- reranking;
- query decomposition;
- Phase-7 semantic functionality;
- LLM/vector functionality.

STEP 1 — EXECUTION PLAN

Create:

    docs/03-plan/phases/phase-02-rag/execution-plan.md

Do not modify plan.md.

The execution plan must contain:

1. Repository-state reconciliation.
2. Remaining work.
3. M1..Mn execution sequence.
4. Mapping of every increment to P2-Tnn tasks.
5. Mapping to D2.x deliverables.
6. Exact files/modules expected to be created or modified.
7. Dependencies and exact versions.
8. Tantivy-vs-FTS5 decision and trigger.
9. arch-check allowlist changes.
10. Acceptance-test mapping.
11. Owner decisions/blockers.
12. Interim fallbacks.
13. Risks and concrete mitigations.
14. Estimate discrepancy.
15. Sprint-overload warning.
16. Expected verification commands.
17. Checkpoint/recovery strategy if implementation is interrupted.

Each increment must be independently verifiable.

Do not write a plan consisting only of broad feature names.

Each increment must have:

    Goal
    Preconditions
    Files
    Tasks
    Tests
    Verification gates
    Documentation updates
    Exit criteria

STEP 2 — IMPLEMENTATION ORDER

Implement in the following logical order unless repository reconciliation demonstrates
that an already-completed task should be skipped or adjusted.

M1 — NORMALIZATION

P2-T13..T24.

Implement:

- RuleId
- NormalizationRule
- RuleKind
- deterministic N01–N17
- heuristic N18–N22
- Heuristic tagging
- NormalizationPipeline
- append-only profile registry
- L0–L8
- NormalizationTrace
- construction invariant preventing trace-less results
- migration for normalization metadata
- SpanMap
- SpanMap composition
- canonical ↔ derived mapping
- normalization preview
- profile APIs
- CLI normalize commands

Required properties:

- determinism;
- idempotency where applicable;
- associativity/composition;
- canonical preservation;
- exact mapping;
- no canonical mutation.

Required tests:

- all five SpanMap properties;
- every ayah;
- every profile;
- fuzz/property tests;
- 2,000-pair golden harness;
- canonical hash preservation;
- trace ordering;
- heuristic tagging;
- profile immutability/append-only behavior.

Implement ADR-0204 and ADR-0205 as drafts if their decisions require owner/linguist input.

M1 is NOT complete until its applicable acceptance tests pass.

M2 — DERIVED FORMS + FTS FOUNDATION

P2-T25..T39.

Implement:

- derived token/ayah forms;
- skeletons;
- ayah and 3-ayah skeleton windows;
- forms.rebuild;
- MV-018;
- FullTextIndex trait;
- manifest;
- FtsQuery;
- SearchOpts;
- backend writer/reader;
- ar_* tokenizers;
- shared normalization pipeline;
- parity test with 5,000 substrings;
- index pointers;
- staged index build;
- verification;
- atomic flip;
- GC;
- rollback;
- trigram skeleton postings;
- crash/cancellation matrix.

Required gate:

- cold rebuild < 6 minutes under the acceptance environment.

Use REAL SQLite and REAL FTS backend in persistence/index tests.

Do not use mocks for persistence/index acceptance.

M3 — SEARCH

P2-T40..T56.

Implement:

- SearchHit;
- ScoreExplain;
- unified result assembly;
- exact search;
- normalized search;
- phrase/slop;
- concatenated candidate → verify → segment;
- cross-ayah deduplication;
- regex;
- I16 protections;
- total_matches;
- filters;
- highlighting;
- cache;
- generation invalidation;
- LRU;
- API v1;
- SSE;
- CLI.

Every SearchHit must contain:

- canonical location;
- exact range mapping;
- NormalizationTrace;
- explanation data as required by the contract.

Add:

- 400-query golden suite;
- regex DoS abuse suite;
- latency benchmarks.

M4 — MORPHOLOGY IMPORT + LEXICONS

P2-T57..T74.

Implement:

- migrations required by actual migration numbering;
- intermediate format;
- JSON Schema;
- adapter abstraction;
- public-domain test lexicon adapter;
- second adapter proving extensibility;
- DirectKey;
- AlignmentTable;
- per-surah unmatched reporting;
- tagset mapper;
- native tag preservation;
- MV-001..018;
- lexicon builder;
- review_queue suggestions;
- importer with 12 checkpoints;
- approval-gated activation;
- FTS rebuild enqueue;
- coverage reports;
- threshold gates;
- differ;
- Layer B/D provenance;
- FTS lexicon fields.

Required tests:

- 18-fault adversarial suite;
- crash matrix;
- alignment tests;
- coverage tests;
- provenance tests.

Never import authoritative morphology data without the required owner/legal decision.

M5 — MORPHOLOGY + FAMILY TOOLS

P2-T75..T93.

Implement:

- AnalysisPolicy;
- suppression reporting;
- morphology;
- morphology_compare;
- root_search;
- lemma_search;
- pattern_search;
- capability-unavailable errors;
- affix_search;
- Attested vs Heuristic labels;
- FamilyRelation closed taxonomy;
- five-path family resolution;
- relation builders;
- per-member explanations;
- opt-in suggestions;
- confidence;
- mandatory labels;
- review_queue;
- ScholarVerified promotion;
- API;
- CLI.

Morphology comparison MUST NOT contain a winner/primary/selected field.

Required tests:

- 500 root/lemma cases;
- 120 curated family cases;
- non-merge tests;
- competing-analysis tests;
- suppression-count tests.

M6 — COUNTING / DISCOVERY / DOCTOR / EVALUATION

P2-T94..T113.

Implement:

- CountingRules;
- frequency;
- distribution;
- cooccurrence;
- collocation;
- PMI;
- LLR;
- t-score;
- first/last;
- intervals;
- numeric_report;
- hapax;
- unusual_usage;
- near_duplicates;
- missing_expected_form;
- API;
- CLI;
- doctor --indexes;
- 19 index checks;
- drift reporting;
- nightly reconciliation;
- eval harness;
- determinism tests;
- 22-tool contract suite.

CRITICAL COUNTING RULE:

Frequency and other exact counts must come from SQL/canonical relational data.

Never derive authoritative counts from FTS term frequencies.

numeric_report MUST contain no interpretive commentary.

Fixed disclaimers for:

- interval_analysis;
- missing_expected_form

must be reproduced exactly as specified by the requirements/acceptance documentation.

near_duplicates:

- candidate generation may use MinHash;
- final matches MUST be verified against canonical/derived text;
- do not treat MinHash similarity as proof.

M7 — HARDENING / SOAK / HANDOFF

P2-T111, T113, T114.

Run:

    rebuild
    50,000 randomized queries
    doctor
    reconcile

Expected result:

- zero findings;
- zero panics;
- zero canonical hash changes;
- deterministic outputs.

Create/update the six D2.13 documents:

- normalization specification;
- profile catalog;
- search cookbook;
- adapter guide;
- counting-rules explainer;
- reindex runbook.

Create:

    docs/03-plan/phases/phase-02-rag/handoff-p2-to-p3.md

Perform final exit-gate review.

ARCHITECTURE ALLOWLIST

Extend xtask/allowlist.toml BEFORE introducing new edges.

Expected edges:

- quran-normalization → domain
- quran-search → quran-normalization
- quran-search → quran-core
- quran-search → domain
- quran-search → storage
- quran-morphology → quran-normalization
- quran-morphology → quran-core
- quran-morphology → domain
- quran-morphology → sources
- quran-morphology → storage
- storage-sqlite → quran-core if required and not already present
- application → quran-normalization
- application → quran-search
- application → quran-morphology

Absolutely forbidden:

- llm dependencies;
- embeddings;
- retrieval;
- vector databases;
- vector-store crates

inside the three Phase-2 domain crates.

If an architecture edge is required but not listed above:

1. stop;
2. inspect why it is required;
3. determine whether the dependency can be inverted through an existing abstraction;
4. only add the edge if architecturally justified;
5. document the deviation.

ERROR CONTRACT

Every new error type must implement the Phase-0 Diagnostic contract:

- code;
- summary;
- remedy;
- next_command.

Use:

- QAI-NORM-nnnn for normalization;
- QAI-IDX-nnnn for index/search infrastructure.

Do not create new QAI-QUR-* codes; those remain Phase-1-owned.

PROVENANCE

Every derived row must record, as applicable:

- corpus_generation;
- rule_set/version;
- dataset/version;
- tokenizer/version;
- provenance_id.

Every mutation must write provenance/audit information.

Every long-running operation must support:

- cancellation;
- checkpoints;
- resume.

HEURISTIC/COMPUTATIONAL LABELING

Heuristic and computational outputs must be labeled consistently in:

- CLI;
- API;
- tool payloads.

Use snapshot tests to prevent accidental removal of these labels.

CACHE SAFETY

A cache entry must never survive a corpus-generation mismatch.

Generation changes MUST invalidate affected cache entries.

Do not serve stale search/morphology/counting results after a generation bump.

DRIFT

Drift detection must:

- detect profile/rule/index mismatch;
- report QAI-IDX-0101;
- produce a useful reconciliation/drift report;
- never silently mutate or repair the index.

APPEND-ONLY PROFILES

Existing profile definitions are immutable.

If normalization rules change:

- create a new profile version;
- rebuild affected derived data;
- produce drift information.

Never silently update an existing profile.

TESTING RULE

Before marking any P2-Tnn task complete:

1. locate its acceptance criterion;
2. identify the corresponding automated test;
3. run it;
4. record the exact test path/name in done.md;
5. only then mark the task complete.

Do not mark tasks complete based on manual inspection alone when an automated acceptance
test is required.

VERIFICATION AFTER EVERY INCREMENT

Run:

    cargo fmt --all

    cargo clippy --workspace --all-targets -- -D warnings

    cargo test --workspace

    cargo run -p xtask -- arch-check

    cargo run -p xtask -- migrate-check

Where relevant also run:

    cargo run -p xtask -- gen-schema

    graft build

If cargo-deny is available:

    cargo deny check

If cargo-deny is unavailable, explicitly record that it could not be run locally.

If ANY required gate fails:

- do not mark the increment complete;
- do not mark its tasks done;
- diagnose and fix the failure;
- rerun the affected gates;
- leave the repository in a coherent checkpoint if the session must end.

PERSISTENCE/INDEX TESTING

For SQLite/index functionality:

- use real temporary directories;
- use real SQLite;
- use real FTS5 or real Tantivy depending on the selected backend;
- do not replace acceptance tests with mocks.

CANCELLATION/CRASH TESTING

Long operations must be tested for:

- cancellation;
- process interruption;
- checkpoint recovery;
- partial staging;
- atomic publication;
- rollback;
- stale staging cleanup.

DOCUMENTATION

After each completed increment:

- update tasks.md;
- append to done.md using its existing exact entry format;
- record date;
- owner;
- PR/commit field as required by the existing format, without inventing a commit;
- evidence/test path;
- DoD;
- notes.

Never rewrite historical done.md entries.

Also update:

- docs/06-progress/task-done-rollup.md
- CHANGELOG.md

according to AGENTS.md.

If an ADR is created:

- record it in done.md §4;
- keep owner/linguist-dependent ADRs as Draft;
- never invent acceptance/sign-off.

If there is a deviation:

- record it in done.md §5.

If there is an owner decision/deferral:

- record it in done.md §7.

Generated schemas:

    cargo run -p xtask -- gen-schema

Run this when relevant and distinguish generated changes from manually authored files.

CHECKPOINT / SESSION CONTINUATION RULE

This work is intentionally large.

If the current session cannot safely finish an increment:

- do not rush;
- do not mark partial work complete;
- leave the repository compiling if reasonably possible;
- update the execution/status documentation with the exact checkpoint;
- identify:
    current increment
    completed tasks
    incomplete tasks
    failing tests
    next file/module to implement
    next verification command
- stop at a clean boundary.

The next agent/session must be able to resume from the documented checkpoint without
re-reading the entire repository.

GIT RULES

Do not:

- commit;
- push;
- create branches;
- create tags;
- reset user changes;
- discard unrelated changes.

Preserve pre-existing user changes.

Before modifying files, inspect git status.

Do not include unrelated existing changes in the Phase-2 work.

OWNER DECISION REPORTING

At the end of every session explicitly report:

1. decisions that require owner input;
2. interim fallback used;
3. whether the fallback is temporary;
4. what must happen before final acceptance.

Do not silently turn an owner decision into an implementation decision.

FINAL RESPONSE

Return:

1. execution-plan.md path;
2. concise M1..M7 summary;
3. repository-state reconciliation;
4. owner decisions/blockers and fallback for each;
5. files created;
6. files modified;
7. dependencies added and versions;
8. arch-check allowlist changes;
9. exact verification commands run;
10. result of every verification command;
11. cargo test --workspace result/count;
12. arch-check result;
13. migrate-check result;
14. cargo-deny result or explicit unavailable status;
15. graft build result where applicable;
16. exact P2-Tnn tasks newly marked ☑;
17. exact tasks still pending;
18. ADRs created/updated and their status;
19. deviations recorded in done.md §5;
20. owner deferrals recorded in done.md §7;
21. current implementation checkpoint;
22. next concrete action.

Do not claim "Phase 2 complete" unless all Phase-2 acceptance criteria and exit gates actually
pass.

START NOW

Perform these steps in order:

1. inspect git status;
2. read the mandatory documents;
3. orient with graft;
4. reconcile actual repository state against Phase-2 tasks;
5. inspect migration numbering;
6. verify dependency cache state, especially Tantivy and FTS5;
7. write execution-plan.md;
8. only after execution-plan.md exists, begin M1;
9. after every increment, run the required verification gates;
10. update task/done/progress documentation;
11. continue to the next increment only when the previous increment's applicable gates are green.

Do not skip the repository-state reconciliation.

Do not blindly implement the entire prompt in one pass.

Do not invent missing data.

Do not silently resolve owner decisions.

Do not modify plan.md.

Do not commit.

Begin now.

```

