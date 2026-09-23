# ADR-0213 — Index Generation Stamping, Activation, Retention, and Drift Policy

- Status: Proposed (implementation complete; acceptance pending owner review)
- Phase: 2 — Quran Search and Linguistics
- Date: 2026-09-23
- Related decisions: ADR-0201 (FTS), ADR-0208 (SpanMap), ADR-0702 (consistency)
- Requirements: PRD invariants I5 (extended to indexes), I8; plan §9.3;
  AC-P2-31…AC-P2-35
- Implementation: `crates/application/src/quran_index.rs` (`rebuild_index`,
  `gc_index`); `crates/quran-search/src/fts5.rs`, `trigram.rs`;
  migration `0015_quran_indexes`; suites `tests/index_build.rs`,
  `tests/index_lifecycle.rs`

## Context

Search indexes are derived data that must never serve partial results, must
survive crashed builds, must roll back one step, and must visibly age out
when their inputs change (profile versions, corpus generations, tokenizer).

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| Generation directories + single pointer row (chosen) | Atomic flip in one SQLite transaction; previous generation retained on disk; crash leaves pointer untouched | Disk cost per retained generation; needs retention GC |
| In-place index mutation | No disk overhead | Partial index can serve; crash recovery complex; no rollback |
| Content-addressed generations with leases | Fine-grained sharing | Overkill for one-index-per-corpus; lease bookkeeping |

## Decision

- Builds stage into `<root>/gen-<N>/` (`index.db` + `trigram.db`), commit,
  reopen with the counted manifest, verify, run MV-018 pre- AND post-checks,
  then flip `index_pointers` + run states in ONE SQLite transaction.
  Generations are monotonic and never reused (a killed attempt consumes its
  number; the retry takes the next).
- `index_pointers` is the ONLY mutable Phase-2 catalog row; everything else
  is append-only history (`index_build_runs`).
- Retention (`qai quran index gc`, default keep 2 = active + previous):
  older superseded generations lose directory + run row; failed runs lose
  orphaned staging dirs but keep history rows; non-terminal runs
  (`staged`/`verifying`) are never touched (may belong to a live build).
  GC is crash-safe (dir removal precedes row deletion; missing dirs report 0)
  and cancel-safe; the active generation is never removed.
- Manifest binds `index_id`, `schema_version`, `corpus_generation`, edition
  identity (slug@version, not row-surrogate id — re-imports reproduce the
  hash), `rule_set_versions`, `tokenizer_version`, `doc_count`,
  `trigram_postings`, and `content_hash`. Drift is a WARNING
  (`QAI-IDX-0101` staleness on every hit), never an auto-repair; `doctor`
  stays read-only and suggests `qai quran index rebuild`.
- Result cache keys bind tool + params + profile + generation; stale
  generations can never be served from cache (T50 contract, `search_cache`
  suite).

Crash matrix (P2-T37, `index_lifecycle.rs`): cancellation at every build
stage leaves the pointer unchanged and the retry activates; cancelling a
rebuild keeps the previous generation serving. Cold rebuild gate (P2-T38):
fixture rebuild timed in CI with a < 6 min assertion; full-corpus timing
belongs to the reindex runbook on licensed data.

## Accuracy and Religious-Source Implications

Stale indexes served silently would present outdated derivations as current
scholarship-adjacent data. Mitigation: generation stamps on every artifact,
`QAI-IDX-0101` warnings on drift, no auto-repair, MV-018 canonical-unchanged
checks bracketing every build.

## Licensing Implications

None — lifecycle mechanics, no data.

## Security Implications

Staging directories are wiped and rebuilt per attempt (no cross-run
contamination). GC never deletes the serving generation even when asked
(`keep` clamps to ≥ 1 and the active gen is always protected).

## Operational Implications

- `qai quran index rebuild` / `verify` / `gc [--keep N]`; `qai doctor
  --indexes` reports drift; nightly `quran.index.verify` reconciliation
  (P2-T107, follow-up) reuses `verify()` + MV-018.
- Full cold rebuild < 6 min (CI gate on fixture; runbook timing on corpus).
- Retention default 2; raise `--keep` before risky profile bumps.

## Migration Strategy

`0015_quran_indexes` (pointers + run tracking). Future lifecycle changes
(e.g. new run states) ride new forward-only migrations; the pointer shape is
stable.

## Reversal Cost

Low. Generations are disposable derived data; `rebuild --all` reconstructs
from canonical sources + manifests.

## Acceptance Criteria

- Kill at any stage → previous generation serves (AC-P2-31).
- Cold rebuild < 6 min (AC-P2-32).
- Version bumps → exact drift reports + `QAI-IDX-0101` (AC-P2-33).
- Drift never auto-repaired; doctor read-only (AC-P2-34).
- No stale cache across generation bumps (AC-P2-35).
