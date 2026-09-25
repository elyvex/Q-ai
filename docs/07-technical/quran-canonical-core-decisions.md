# Canonical Quran Core — Decisions (Phase 2, plan 02-04)

This record captures the decisions behind the canonical-write fence completed in
plan `02-04` of phase `02-canonical-quran-core`. It exists so later phases do not
re-litigate choices that are already enforced by migration `0021`, the
`provenance` gate, and the importer audit test.

## 1. `quran_segments` is staging-only in Phase 2

Canonical `quran_segments` **stays empty** in Phase 2.

- Nothing consumes canonical segments yet; the table is reserved for the
  morphology/segmentation phase (roadmap Phase 3).
- Activation therefore continues to copy only `quran_surahs`, `quran_ayahs`,
  `quran_tokens`, `quran_token_separators`, and `quran_divisions` from the
  staging mirror. The frozen activation move list is **not** rebuilt.
- `quran_segments` is still fenced by migration `0021` (`QAI-QUR-0010` /
  `QAI-QUR-0011`), so it is provably insert-only even while empty.
- A future phase that needs canonical segments must add a **new forward
  migration** plus a copy step; it must not edit `0021` or the activation
  transaction in place (migrations are append-only, ADR-0002).

This is the recorded resolution of RESEARCH.md § F-1 / Open Question 5 (QC-01).

## 2. Trigger-code map (migration 0021)

`QAI-QUR-0001`…`0005` were already in use before this plan and were neither
reused nor renumbered (ADR-0010 stable taxonomy). Migration `0021` adds
`QAI-QUR-0006`…`0021`, one per table/verb:

| Table | Verb | Trigger | Code |
|---|---|---|---|
| `quran_surahs` | UPDATE | `trg_surah_no_update` | `QAI-QUR-0006` |
| `quran_surahs` | DELETE | `trg_surah_no_delete` | `QAI-QUR-0007` |
| `quran_token_separators` | UPDATE | `trg_separator_no_update` | `QAI-QUR-0008` |
| `quran_token_separators` | DELETE | `trg_separator_no_delete` | `QAI-QUR-0009` |
| `quran_segments` | UPDATE | `trg_segment_no_update` | `QAI-QUR-0010` |
| `quran_segments` | DELETE | `trg_segment_no_delete` | `QAI-QUR-0011` |
| `quran_divisions` | UPDATE | `trg_division_no_update` | `QAI-QUR-0012` |
| `quran_divisions` | DELETE | `trg_division_no_delete` | `QAI-QUR-0013` |
| `translation_editions` | UPDATE | `trg_translation_edition_no_update` | `QAI-QUR-0014` |
| `translation_editions` | DELETE | `trg_translation_edition_no_delete` | `QAI-QUR-0015` |
| `translation_passages` | UPDATE | `trg_translation_passage_no_update` | `QAI-QUR-0016` |
| `translation_passages` | DELETE | `trg_translation_passage_no_delete` | `QAI-QUR-0017` |
| `word_glosses` | UPDATE | `trg_gloss_no_update` | `QAI-QUR-0018` |
| `word_glosses` | DELETE | `trg_gloss_no_delete` | `QAI-QUR-0019` |
| `quran_editions` | DELETE | `trg_edition_no_delete` | `QAI-QUR-0020` |
| `quran_editions` (0020 identity/default columns) | UPDATE OF `upstream_edition_slug`, `qai_edition_id`, `is_primary` | `trg_edition_identity_no_update` | `QAI-QUR-0021` |

Pre-existing codes (unchanged): `QAI-QUR-0001` (`quran_ayahs` UPDATE),
`0002` (`quran_editions` identity/hash UPDATE), `0003` (`quran_ayahs` DELETE),
`0004` (`quran_tokens` UPDATE), `0005` (`quran_tokens` DELETE).

`trg_edition_identity_no_update` is a **second** trigger rather than a
`DROP`/recreate of `trg_edition_immutable_hashes`, so the 0007 trigger stays
byte-identical. Neither trigger covers `status`: deactivation/deprecation writes
that column, which is why activation and rollback remain possible.

### Translation rows are insert-only published artifacts

`translation_editions` / `translation_passages` are fenced on UPDATE and DELETE
because translations are published artifacts: once a translation version exists,
its text and attribution must not be mutated in place. A future status transition
(e.g. retiring a translation) requires a **new forward migration**, never an edit
of `0021`. `word_glosses` is fenced the same way as a canonical-aligned artifact.

## 3. Wire the `CanonicalWriter` gate; do not retire it

RESEARCH.md § QC-06 offered two honest resolutions: wire the type-level
`CanonicalWriter` + `ApprovalToken` gate that `.agent/coding-rules.md` claims, or
retire it with an ADR-level change. **This plan wires it.** The retirement
alternative was rejected because:

- ADR-0000 (`docs/02-architecture/decisions/ADR-0000-project-architecture.md`)
  locks canonical immutability as a **type-level** property: "canonical rows are
  written only through a `CanonicalWriter` that requires an `ApprovalToken`
  obtained from a persisted human `ApprovalRecord`." Retiring the type would
  leave the locked architecture statement false.
- `.agent/coding-rules.md` states the same invariant as a standing rule, so dead
  code would silently contradict a project safety invariant.

Implementation (for traceability):

- `provenance::ApprovalToken::new` is `pub(crate)`; the only public mint path is
  `ApprovalToken::from_approval_row`, which requires a persisted row with
  `decision = "approved"` and a non-empty `subject_urn`.
- `provenance::ApprovalGate` implements `CanonicalWriter`. A change session opens
  only when `token.subject_urn() == change.subject_urn`; otherwise
  `begin_canonical_change` returns `ApprovalSubjectMismatch` **before** any
  storage mutator runs.
- `application::quran::{activate_edition, rollback_edition,
  record_edition_verification, deprecate_edition}` mint the token from the same
  fetched approval row that `check_approval` verified, open a session, run the
  storage mutator, then commit the session — all inside the existing single
  `UnitOfWork`. The activation/rollback transaction ordering, pointer flip, and
  generation bump are unchanged.
- `crates/quran-corpus/tests/import_path.rs` audits `import.rs` source and proves
  the importer never references `ApprovalToken`/`CanonicalWriter`, never calls an
  activation/rollback mutator, never issues a canonical-table INSERT, and ends at
  run state `Staged`.

## 4. Rollback is approval-gated exactly like activation

Rollback is a canonical pointer change, so it passes through the same
`check_approval` → `ApprovalGate` path. Missing, denied, and subject-mismatched
approvals each leave the active pointer, the canonical rows, and
`corpus_generation` untouched; a rollback to an already-active edition returns
the `AlreadyActive` (`QAI-QUR-0313`) conflict and changes nothing. The reader
cache is generation-keyed, so a rollback invalidates it and no stale text is
served (research QC-01; ADR-0113).

---

*Phase: 02-canonical-quran-core · plan 02-04 · recorded 2026-09-25*
