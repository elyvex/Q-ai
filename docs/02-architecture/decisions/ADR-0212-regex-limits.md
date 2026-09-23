# ADR-0212 — Regex/Pattern-Search Resource Limits and Engine Choice

- Status: Proposed (implementation complete; acceptance pending owner review)
- Phase: 2 — Quran Search and Linguistics
- Date: 2026-09-23
- Related decisions: ADR-0201 (FTS)
- Requirements: PRD invariant I16; plan §4.3; AC-P2-13
- Implementation: `crates/quran-search/src/regex.rs`, `fts5.rs`
  (`regex_expression`); service `search_regex`; suites
  `tests/regex_dos.rs`, `search_goldens.rs` (40 regex goldens)

## Context

Pattern search over scripture is a ReDoS and resource-exhaustion vector:
user-supplied patterns run against the full term dictionary and corpus.
Backtracking engines make pathological patterns a denial-of-service.

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| DFA-only (`regex-automata` dense) + budgets (chosen) | No backtracking by construction; bounded construction (1 MiB NFA / 4 MiB DFA); predictable execution | Patterns the DFA cannot build are rejected (no fallback) |
| Backtracking engine with timeouts | More patterns accepted | Timeouts are racy; worst case still DoS-adjacent |
| FTS-side regex without guards | Simple | Unbounded vocab scans and expansions |

## Decision

- Exactly one engine: dense-DFA via `compile_dfa` + `first_match`; the FTS
  adapter delegates to it (one engine, no fallback). A DFA build failure is
  a rejection, never a retry elsewhere.
- Up-front guards: 512-char patterns, no leading `.*`/`.+` (anchored
  patterns only), indexed text fields only.
- Recall via `fts5vocab` term-dictionary expansion (bounded: 50k vocab scan
  cap, 128-term expansion cap); span resolution re-runs the IDENTICAL
  automaton over normalized ayah text; `terms_examined`/`documents_scanned`
  always reported via `RegexReport`.
- Execution budget wraps the backend call (default 3000 ms, ceiling 10000);
  per-principal rate limit (10/min sliding window, `QAI-IDX-0007`).
- Agent-policy gating is NOT enforced in the service (policy engine lands
  Phase 7); agent calls pass the tool-registry gate (T110), the only path
  that checks grants.

Known limitation (recorded 2026-09-23): recall is vocab-term-shaped —
FTS5's unicode61 tokenizer splits terms at harakat, so anchored patterns
with interior diacritics have no recall (terms never contain them).
Harakat-anchored recall needs a mark-preserving tokenizer or a scan
fallback; filed as a follow-up. The 15-pattern abuse suite and 40 golden
patterns stay within the supported contract.

## Accuracy and Religious-Source Implications

A rejected pattern must say why (typed `QAI-IDX-0002` with remedy), never
fail silently; expansion terms are reported so results stay explainable.

## Licensing Implications

None — `regex-automata` is MIT-licensed (see `deny.toml`).

## Security Implications

This ADR IS the I16 mitigation: DFA-only, construction budgets, execution
budget, rate limits, field allowlist, agent-policy gating at the registry.
Abuse suite (`regex_dos.rs`) pins all 15 pathological patterns.

## Operational Implications

Rate-limit counters are in-memory per process (restart resets); tighten
`max_per_minute` for shared deployments. Monitor `QAI-IDX-0007` rates.

## Migration Strategy

Budget changes are constants (`regex.rs`); tightening is backward-compatible
(more rejections), loosening needs a new abuse-suite run.

## Reversal Cost

Low. Engine swap requires re-running the abuse suite + goldens; the
single-engine rule (`compile_dfa` choke point) makes the swap local.

## Acceptance Criteria

- 15 pathological patterns rejected or bounded with reported costs (AC-P2-13).
- Rate limit enforced per principal.
- No backtracking engine anywhere in the path (architecture test).
