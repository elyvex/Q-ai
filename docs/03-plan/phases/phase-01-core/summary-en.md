# Phase 1 Summary — Canonical Quran Core

**Phase:** P1 — Canonical Quran Core
**Full name:** Canonical Quran Corpus (structure, addressing, import, validation, reader)
**Plan version:** 1.0.0
**PRD baseline:** Q-ai PRD v0.3.2
**Depends on:** Phase 0 (all deliverables)
**Blocks:** Phase 2 (search/linguistics), Phase 3 (graph), Phase 4 (Web reader), Phase 5 (Quran↔hadith/tafsir links)
**Target duration:** 6 calendar weeks (~15–17 engineer-weeks, 3 engineers)
**Sum of task-row estimates:** 131 engineer-days
**Status:** Not Started

---

## 1. What This Document Is and What Was Produced at This Stage

Before any explanation, it must be clear that at this stage only the **planning documents** for
Phase 1 were written and completed, and no executable code was created or modified. The folder
`docs/03-plan/phases/phase-01-core/` previously contained only `plan.md`, with four empty files —
`README.md`, `tasks.md`, `acceptance.md`, and `done.md` — alongside it. In this stage those four
documents were written on the basis of `plan.md` and `docs/01-requirements/requirements.md` (the
PRD, version 0.3.2), following the same style and format as the Phase 0 documents. `README.md` was
prepared as the phase overview, `tasks.md` as the task board, `acceptance.md` as the acceptance
criteria and exit gate, and `done.md` as the append-only completion ledger. Therefore what follows
is a summary of the content of Phase 1: what Phase 1 commits to building, and the documents written
to manage and accept it.

---

## 2. The Fundamental Objective of Phase 1

The objective of Phase 1 is to stand up the **canonical Quran engine**: a lossless, hash-verified,
versioned, structurally validated corpus with stable addressing and deterministic, LLM-free lookup.
This corpus is the foundation every other Q-ai capability cites: Phase 2 search, the Phase 3 graph,
the Phase 4 web reader, and the Phase 5 Quran↔hadith/tafsir links. If this foundation is weak, every
hidden defect in the text is inherited by all subsequent citations, graph edges, and research
reports, and no amount of downstream tooling can detect it. For this reason the criterion in this
phase is not "probably correct" but **byte-for-byte equality** with the approved source.

---

## 3. The Promise of Phase 1

The explicit promise of Phase 1 is this: for any legal reference, Q-ai returns the exact canonical
Arabic text of an identified edition version, byte-for-byte equal to the approved source, with a
resolvable citation, in single-digit milliseconds, **without** an LLM, without a vector store, and
without a network call — and no code path exists that can silently change that text. This promise
has three parts: correctness of the text, determinism of retrieval, and the absence of any hidden
mutation path. The third part is harder than the first two, because a single incorrect write path,
even if it is never executed, invalidates the promise. For this reason the bulk of the Phase 1
design is devoted to closing unauthorized write paths, not merely to speed or features.

---

## 4. The Seven Invariants

Phase 1 introduces seven invariants that all subsequent work must preserve. The first (I1) states
that canonical ayah text is immutable per edition version, enforced by insert-only tables, database
triggers, and Phase 0's `CanonicalWriter`/`ApprovalToken`. The second (I2) states that canonical
text is never produced by a model; this is enforced by an architecture test that forbids an `llm`
dependency in `quran-corpus`. The third (I3) states that different readings and editions are never
merged into a synthetic text, and that `edition_id` is part of the primary key of every canonical
record and of every returned record. The fourth (I4) guarantees a stable, verifiable token order.
The fifth (I5) states that a partially imported edition can never become `Active`. The sixth (I6)
states that every returned quotation carries the edition id, version, and hash. And the seventh (I7)
states that corrections create a new version with a difference report and human approval.

---

## 5. Domain Model — Edition, Surah, Ayah, Segment, Token

The Phase 1 domain model defines a five-level hierarchy: Edition → Surah → Ayah → Segment → Token.
At the edition level, all fields from §7.2 of the PRD are recorded: identifier, name, script (e.g.
Uthmani), riwayah (e.g. Hafs), qiraah (e.g. Asim), publisher, source URL, license, language,
verse-numbering scheme, basmala policy, Unicode normalization form, and four different hashes
(`text_hash`, `structure_hash`, `token_order_hash`, and `manifest_hash`) together with version and
status. At the ayah level, the canonical `text` is stored exactly as it appears in the approved
source, with no processing or correction applied. At the token level, Phase 1 stores only the
**surface form**; lemma, root, and morphology arrive in Phase 2.

---

## 6. Lossless Text Guarantee: Token and Separator

One critically important technical point in Phase 1 is that tokenization must be **lossless**. For
every ayah, if we join the tokens in position order and reapply the recorded separators, the stored
text must be reconstructed exactly and byte-for-byte. Likewise, for every token,
`ayah.text[byte_start..byte_end] == surface` must hold. To this end the `quran_token_separators`
table is designed to hold the exact spacing between tokens (and the leading/trailing text of the
ayah), so reconstruction is possible even in the presence of unusual whitespace or waqf marks. This
guarantee is measured in full-corpus tests and in acceptance criteria AC-P1-05 and AC-P1-06; if it
fails for even one ayah, Phase 1 does not exit.

---

## 7. Stable Addressing and the Reference Grammar

§7.4 of the PRD requires stable addressing, and Phase 1 implements it with a formal reference
grammar. References can point to an edition, a surah, an ayah, an ayah range, a specific token, a
juz, a hizb, a rubʿ, a manzil, a page, a ruku, or a sajdah. The EBNF grammar is defined in the
Phase 1 plan and frozen by ADR-0102; examples such as `quran:1:1`, `2:255`, `quran:2:255-257`,
`quran:2:255:token:3`, `quran:hafs-uthmani@1.0.0:112`, `quran:juz:30`, and `quran:page:604` all
resolve deterministically. The canonical full form used for storage and citation is always the
version-pinned form, such as `quran:hafs-uthmani@1.0.0:2:255`. One important property is that once
the grammar is frozen, changing it breaks every stored citation; therefore the 300-case golden test
set and the `parse(serialize(ref)) == ref` property test must be green before the freeze is
declared.

---

## 8. Phase 1 Deliverables

Phase 1 has fourteen deliverables. `D1.1` is the pure `quran-core` crate (domain types, the
reference grammar, and `QuranQuotation`) with no I/O and no async dependency. `D1.2` is the edition
source format and manifest extension together with the adapters. `D1.3` is the canonical importer
with its 13 checkpoints. `D1.4` is the corpus validator with rules QV-001 through QV-028, the
Unicode auditor, and the reference-corpus comparator. `D1.5` creates the database migrations `0010`
through `0015`. `D1.6` builds the deterministic `QuranReader` lookup service. `D1.7` provides the
Quran read API version 1. `D1.8` defines the tool result contract and the first two tools,
`quran.get_ayah` and `quran.get_context`. `D1.9` builds citation resolver v1. `D1.10` builds the
`qai quran` command-line surface. `D1.11` adds the `qai doctor --quran` checks. `D1.12` is a minimal
debug reader. And `D1.13` and `D1.14` cover the corpus integrity test suite and the documentation,
respectively.

---

## 9. The Canonical Importer and Its 13 Checkpoints

Deliverable `D1.3` is a background job named `quran.import` that follows §34 of the PRD exactly and
passes through thirteen stages: claim the source version, verify the declared file hash against the
observed hash, detect the format and select an adapter, parse into the intermediate format, run the
Unicode audit, perform structural validation, tokenize and build separators and offsets, compute
the three hashes, write to the staging tables, verify round-trip against the staged rows, compare
against the reference corpus if configured, produce a difference report against the current active
version, and finally reach the `Staged` state and emit an approval request. Every stage has a
resumable checkpoint, and the job is idempotent on the combination
`(source_version_id, adapter_version, parser_version)`.

---

## 10. Separation of Importer and Activation

The most important security guarantee of the importer is that it **cannot activate**. The importer
holds no `ApprovalToken` and is architecturally incapable of doing so. Activation is a separate,
human-driven command, `qai quran activate`, performed in a single transaction: insert or update the
active pointer row, change the previous active version's status to `Deprecated`, change the new
version's status to `Active`, write an audit event, and increment the corpus generation counter. The
staging tables (`quran_stg_*`) are physically separate from the canonical tables, and nothing except
validation and the activation transaction reads them. As a result, a crash in the middle of an
import leaves the version `Staged` and it never becomes `Active`; that is, no reader can ever
observe a half-complete corpus.

---

## 11. The Corpus Validator and the QV Rules

Deliverable `D1.4` is a validator that produces a machine-readable `ValidationReport` with one entry
per rule. Twenty-eight rules are defined, QV-001 through QV-028, and all of them run; the report
lists every failure and does not fail fast, because a partial report is useless for editorial
review. Examples of the rules include: the surah count equals the manifest expectation (QV-001),
surah numbers are contiguous with no duplicates (QV-002), each surah's ayah count equals its
declared count (QV-003), no text is empty (QV-006), the text is in the declared Unicode
normalization form (QV-007), no forbidden code points such as BOM or bidi overrides are present
(QV-008), token positions are correctly ordered (QV-010), token round-trip is verified (QV-011), and
offsets are valid (QV-012). Each rule's severity is `Fatal`, `Error`, `Warning`, or `Info`; a
`Fatal` blocks reaching `Staged`, and an `Error` blocks approval unless an explicit override is
recorded.

---

## 12. The Sixteen Adversarial Corpora

An important part of the Phase 1 testing strategy is sixteen deliberately broken corpora, each of
which must be rejected with a **specific rule id**, not merely with a generic validation error. For
example, a removed ayah must be rejected with QV-005, NFD text declaring itself NFC with QV-007, an
injected U+202E with QV-008, permuted tokens with QV-010 or QV-024, off-by-one offsets with QV-012,
a tampered manifest hash with QV-013, and a translation submitted as an Arabic edition with QV-027.
This precision is deliberate: a validator that rejects everything with a single generic code passes
AC-P1-02 but fails AC-P1-04. These corpora are built by task P1-T05 and form the basis of Phase 1
acceptance.

---

## 13. Database Migrations

Deliverable `D1.5` defines six migrations. Migration `0010_quran_editions` creates the editions
table, the active pointer, and the edition statistics. Migration `0011_quran_structure` creates the
surah, ayah, token, separator, and segment tables together with the immutability triggers. Migration
`0012_quran_divisions` creates the divisions (juz, hizb, rubʿ, manzil, page, ruku, sajdah). Migration
`0013_quran_translations` creates translation editions, translation passages, footnotes, and word
glosses. Migration `0014_quran_staging` creates the `quran_stg_*` mirrors with an `import_run_id`
column and without immutability triggers. And migration `0015_quran_validation` creates the
validation reports, difference reports, and the citations table. The canonical tables are
insert-only, and triggers `QAI-QUR-0001` through `QAI-QUR-0005` halt any `UPDATE` or `DELETE` with a
coded error.

---

## 14. The Lookup Service and Performance

Deliverable `D1.6` defines a deterministic, LLM-free service called `QuranReader`, providing
operations such as `get_ayah`, `get_ayahs`, `get_context`, `get_tokens`, `list_surahs`, and
`resolve`. A key point is that `get_context` retrieves context according to **canonical structure**
(surah, juz, ruku, page) and never according to arbitrary chunking. The performance targets are
explicit: `get_ayah` with a warm cache under 0.2 ms at p50 and under 1 ms at p99, `get_context(±5)`
under 3 ms at p50, `get_surah` for the longest surah under 25 ms at p50, and reference parsing under
20 µs. The cache layer is keyed by
`(edition_id, version, corpus_generation, ref, options_hash)` and is invalidated wholesale when the
corpus generation changes. A consistency test proves that after activating a new version, no stale
text is ever served.

---

## 15. Tool Contract and Citation Resolver

Deliverable `D1.8` defines the tool result contract once and for all, so that all later tools conform
to it. Every tool result includes the tool name, version, exact query, normalization rules, edition
id and version, results, canonical references, analysis sources, confidence, warnings, execution
time, and reproducibility data. The first two tools are `quran.get_ayah` and `quran.get_context`,
both read-only. Deliverable `D1.9` builds a citation resolver that checks that the source exists,
the location resolves, the quotation matches the text, the edition exists, permission is granted,
and the hash and version are available. Among its important outputs are `Mismatch` and
`LocationNotFound`, which must be turned into a hard failure on any answer path; this is the very
mechanism Phase 9's citation verifier reuses.

---

## 16. CLI, doctor, and the Debug Reader

Deliverable `D1.10` extends the command-line surface and adds commands such as `qai quran get`,
`context`, `surah`, `division`, `resolve`, and the `edition`, `import`, `validate`, `diff`,
`activate`, `rollback`, `deprecate`, and `hashes` groups. Deliverable `D1.11` adds nineteen
diagnostic checks under `qai doctor --quran`, including: exactly one active edition, the stored hash
matching the recomputed hash, the structure and token-order hashes matching, surah and ayah counts,
no gaps in identifiers, token round-trip, Unicode form matching, complete division coverage,
complete provenance, correct metadata layering, and license status. The `--deep` mode performs full
corpus hash recomputation and full token round-trip in under thirty seconds, and `doctor` remains
strictly read-only in all cases. `D1.12` builds a simple page at `/debug/read/{edition}/{surah}` so
engineers and editorial reviewers can view the imported text with correct RTL and a web font; this
page is explicitly labelled "debug view", has no data persistence, and is excluded from product
navigation.

---

## 17. Data Prerequisite and the Hard ADR-0101 Gate

Phase 1 **cannot** ship without an approved edition dataset. This is a legal and editorial task, not
a programming task, and it must begin before Sprint 1.1. ADR-0101 must record the candidate datasets
with their provenance, script, riwayah, numbering scheme, Unicode normalization form, and license;
it must specify redistribution rights (PRD §38); it must identify the reference corpus used for
independent comparison in validation; and it must record the editorial sign-off of a **named**
qualified reviewer. If no dataset can be bundled, the fallback is for Q-ai to ship the schema,
validator, importer, and a small public-domain test corpus, and for the user to import their own
approved edition with `qai quran import <manifest>`. This fallback decision must be made in
ADR-0101, not at the moment of crisis. Engineering delay is recoverable, but a delay in a legal or
editorial decision halts the entire phase.

---

## 18. The Task Board and Sprints

The Phase 1 task board contains 60 tasks across six sprints plus a swimlane for external decisions.
Sprint 1.0, "Data & Decisions", runs in parallel with Phase 0 Sprint 0.5. Sprint 1.1, "Domain &
Addressing", builds the `quran-core` crate, the reference grammar, and migrations `0010` through
`0012`. Sprint 1.2, "Import & Validation", is the heaviest part and covers the intermediate format,
adapters, tokenizer, hashing, the QV rules, the importer, activation, and the differ. Sprint 1.3,
"Reader, Translations, API", builds the `QuranReader` service, the cache, the translation model, and
API v1. Sprint 1.4, "Tools, Citations, CLI, and doctor", builds the tool contract, the citation
resolver, the CLI, and the diagnostic checks. And Sprint 1.5, "Debug Reader, Hardening, Exit",
covers the editorial review, golden-set expansion, property tests, full-corpus soak, documentation,
and the exit-gate review.

---

## 19. Estimate Mismatch and Sprint 1.2 Overload

The documents produced deliberately surface two scheduling problems rather than silently
propagating them. The first is that the header of §8 of the plan states a total of about **82
engineer-days**, whereas the actual sum of the task rows is **131 engineer-days** — a gap of
roughly 49 engineer-days, or about 1.6×. At a realistic rate of 15 engineer-days per week for a
three-person team, this is about 8.7 weeks, not 6. The second is that Sprint 1.2, at 42
engineer-days, is roughly three times a normal sprint and contains exactly the two highest-risk
items of the phase (the importer's state machine and the full QV rule set). It is proposed that this
sprint be split into 1.2a (format, adapters, tokenizer, hashing) and 1.2b (rules, importer,
activation, differ). It is emphasized that these problems must not be resolved by silently
compressing estimates, and that the retrofit-impossible deliverables — `quran-core`, the validator,
the migrations, `QuranReader`, and the integrity test suite — must not bear the cost of any scope
reduction.

---

## 20. Acceptance Criteria and the Exit Gate

`acceptance.md` defines thirteen primary acceptance criteria (AC-P1-01 through AC-P1-13) drawn from
the Phase 1 plan, plus eight supporting criteria (AC-P1-14 through AC-P1-21). The primary criteria
include: approval of ADR-0101; import to `Staged` with zero `Fatal` findings; the importer being
unable to activate; full implementation of QV-001 through QV-028 and rejection of the sixteen
adversarial corpora with specific ids; byte-for-byte token reconstruction; token offsets matching
the surface form; recomputed hashes matching; `UPDATE`/`DELETE` aborting with the documented codes;
no public API to write without a `CanonicalChangeSession`; the active edition remaining unchanged
after a crash at each of the thirteen checkpoints; staging rows being removed on cancellation;
correct parsing of the 300 golden references and no panics on invalid input; and
`parse(serialize(ref)) == ref`. The supporting criteria cover the API response contract, the tool
contract, the CLI contract, the doctor checks, the debug reader, cache invalidation, completeness of
the ADRs and documentation, and the citation resolver. The phase is declared complete only when all
of these criteria are verified with documented evidence and a non-implementer reviewer has performed
a recorded live walkthrough.

---

## 21. Out of Scope for Phase 1

To prevent scope creep, Phase 1 explicitly sets several things aside. All search (exact, normalized,
phrase, concatenated, and regex) is deferred to Phase 2. Normalization, roots, lemmas, stems, and
morphology go to Phase 2. Quranic graph nodes and edges go to Phase 3. The web reading UI is
deferred to Phase 4, and Phase 1 delivers only a minimal debug reader. Tafsir, hadith, and other
scriptures go to Phases 5 and 8. Recitation audio, tajwid, and variant qira'at are deferred to Phase
4 or post-MVP, with only schema fields reserved for them. And agents and language models go to Phase
9. Respecting this scope boundary is one of the main reasons a phase succeeds, because it prevents
work from spilling into areas whose prerequisites are not yet ready.

---

## 22. The Main Risks of Phase 1

`README.md` lists ten main risks. The most dangerous is "ADR-0101 slipping" — that is, no licensed,
approved dataset being available by Sprint 1.2, which halts the entire phase. The second risk is
"silent text corruption", meaning a dataset that is almost right but enters the corpus because of
NFD versus NFC, a truncated ayah, or a Latin homoglyph; the response to this risk is running all QV
rules without early stopping and making the comparison with the reference corpus mandatory. The
third risk is "non-lossless tokenization", restrained by the separators and the full-corpus
round-trip test. The fourth risk is "accidental activation", restrained by the physical separation
of staging, the absence of a token in the importer, and the crash matrix at the thirteen checkpoints.
Other risks include a later change to the hashing scheme, the estimate gap, the Sprint 1.2 overload,
an incorrect freeze of the reference grammar, bypassing translation attribution, and serving stale
text from the cache.

---

## 23. The Documents Produced for Managing the Phase

The four documents produced have complementary roles. `README.md` provides the phase overview,
scope, crate diagram, dependency rule, migrations, the data prerequisite, the ADR table, the exit
gate, and the risks, and is the entry point for studying the phase. `tasks.md` is the live task
board with rows P1-T01 through P1-T60, estimates, roles, dependencies, and status, and it keeps the
external-decision swimlane separate. `acceptance.md` defines the acceptance criteria, the mandatory
test suites, the definition of done, the coverage gates, the CI jobs, the exit-gate ritual, and the
Phase 2 handoff assets. And `done.md` is the append-only completion ledger that records the
completion of each task, the verification of each criterion, the acceptance of each ADR, deviations,
corrections, and deferred items, with evidence. These four documents are consistent with the
verified counts of 60 tasks, 21 acceptance criteria, 14 ADRs, and 6 migrations.

---

## 24. Conclusion

Phase 1 is the beating heart of Q-ai's correctness. If this phase is done properly, all subsequent
capabilities are built on a foundation that holds the Quranic text byte-for-byte, version by
version, and in a verifiable way, and that no model or vector search can silently alter. If this
phase is done carelessly, every textual defect is inherited and multiplied across all subsequent
citations and analyses, and can no longer be remedied. For this reason the Phase 1 plan is built
around a few simple but strict principles: the canonical text is immutable; writing it requires
human approval; the importer never activates; staging is separate from canonical; the hashes are
frozen and reproducible; tokenization is lossless; and validation runs all rules without stopping.
The documents produced at this stage turn these principles into tasks, acceptance criteria, and
measurable evidence, so that Phase 1 closes not with "it is probably correct" but with "it is proven
and verifiable".
