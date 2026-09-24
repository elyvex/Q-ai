# ADR-0217 — Graph Query Safety Limits and Exhaustion Semantics

- Status: Proposed (defaults implemented and tested; owner ratification pending — P4-X03)
- Phase: 3 (dir: phase-04) — Quran Knowledge Graph
- Date: 2026-09-24
- Related decisions: ADR-0202 (graph port, SQLite adjacency), ADR-0702
  (snapshot/publication/fencing — Phase-3 subset pending, P4-X02)
- Requirements: PRD §6 safety/budget clause (requirements.md:6639), plan §5 M3,
  AC-P4-12/13/14
- Implementation: `crates/quran-graph/src/model.rs` (`QueryBudgets`,
  `QueryBudgets::check`), `src/traverse.rs`, `src/pattern.rs`, `src/mem.rs`
- Verification: `crates/quran-graph/tests/conformance.rs`,
  `tests/traversal.rs` (cycle, high-degree star, cancellation, truncated≠empty)

## Context

Graph traversal is a denial-of-service amplifier: hubs (roots, entities,
concepts) fan out to thousands of edges, cycles never terminate on their own,
and "does a path exist?" over an unbounded graph is unbounded work. A naive
implementation that pushes a `LIMIT` into the backend's final result set still
does all the expansion work first, so the `LIMIT` protects nothing.

Correctness has a second, quieter requirement: a query that stopped early
because it hit a limit has **not** answered the question. Reporting a truncated
expansion as "no path" is a fabricated negative claim about scripture-level
data — the worst kind of wrong answer in this project.

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| No budgets; rely on the corpus being small (rejected) | Simplest; no false negatives | Quadratic blowup on root hubs; a hang is indistinguishable from a slow machine |
| Budgets applied as a final `LIMIT` only (rejected) | Easy to implement in SQL | Expansion still unbounded; cancellation point is far from the work; `LIMIT` silently changes meaning |
| Budgets validated before execution and enforced **during** expansion, with typed incomplete results (chosen) | Bounded work; failure mode is explicit; portable across backends | Every adapter must implement counter checks in its own expansion loop |

## Decision

Every `GraphStore` query carries a `QueryBudgets` value. Validation happens
before any work; enforcement happens inside expansion.

| Budget | Default | Accepted range | Purpose |
|---|---|---|---|
| `max_hops` | 6 | 1..=32 | depth bound |
| `max_nodes` | 500 | 1..=100_000 | distinct nodes collected |
| `max_edges` | 2 000 | 1..=200_000 | edge relaxations (the real work counter) |
| `max_fanout` | 128 | 1..=10_000 | per-node expansion guard for high-degree hubs |
| `max_paths` | 10 | 1..=1_000 | returned path count |
| `timeout_ms` | 5 000 | 1..=300_000 | wall clock; adapters must enforce natively (SQLite progress/interrupt), not poll a stopwatch between pages |

Rules:

1. **Pre-flight violation is an error, not a truncation.** A budget outside its
   range never runs (`QAI-GRAPH` budget error). Silently clamping would hide a
   caller bug and change the meaning of a recorded result.
2. **In-flight exhaustion is a typed partial result.** When a counter is
   exhausted or cancellation is observed, the result carries
   `truncated = true` and a non-empty `incomplete_reason`; it is never
   returned as an empty-but-complete result.
3. **`"no path"` requires completeness.** `min_hops == None` /
   `paths == []` may only be reported when `truncated == false`; otherwise the
   result is `incomplete` with the reason ("node budget exhausted", "cancelled
   during minimum-hop search", …).
4. **Hidden intermediates cannot influence output.** Authorization,
   source-restriction and effective-tombstone filtering apply to every expanded
   node and edge, so a restricted node can neither appear in a path nor change
   a hop count or expansion counter seen by the caller.
5. **Determinism.** Equal budgets + equal visible input produce equal ordering
   and equal results; ordering is part of the contract, not incidental.
6. **Backend languages stay private.** Budgets are the only resource-control
   surface exposed above the port; no caller can pass SQL/Cypher/Datalog.
7. **Cancellation is a budget outcome.** The cancellation flag is checked during
   expansion and ends the query as `incomplete`, not as an error and not as
   empty.

## Accuracy and Religious-Source Implications

Truncation is a statement about the *query*, never about the text. No partial
traversal may be rendered as "there is no such relationship" or as an absence
of attribution; consumers must surface `truncated`/`incomplete_reason`
alongside results. This keeps a resource limit from becoming a false claim
about Quran structure, roots, or scholarly opinions.

## Licensing Implications

None — no new dependency. Budget enforcement is counter arithmetic in the
existing crate.

## Security Implications

This ADR is the graph-side DoS mitigation: bounded expansion, bounded result
size, native wall-clock enforcement, no raw query text across the port, and
authorization applied to intermediates rather than to the final result set.
The cycle + high-degree conformance fixtures exist to keep an adapter from
"passing" while being unbounded.

## Operational Implications

Defaults are constants in one struct, so a deployment can tighten them without
a code change once configuration plumbing exists (Phase 7). Adapters must
report which budget ended the query; the Phase-4 `qai doctor` and CLI render the
reason verbatim so operators can tell a budget stop from a missing projection.

## Migration Strategy

Changing a default is backward compatible in the tightening direction (more
truncations, never silent data change) and needs the conformance suite re-run in
the loosening direction. Budget *ranges* may only widen together with a
conformance re-run, since the upper bounds are the hard DoS guard.

## Reversal Cost

Medium. Removing budgets is a one-line default change, but every consumer that
relies on `truncated`/`incomplete_reason` (CLI, HTTP envelope, tools, export)
would need to handle an always-complete world; keeping the fields costs nothing
once present.

## Acceptance Criteria

- `crates/quran-graph/tests/conformance.rs`: budget exhaustion is truncated, not
  empty; authz-filtered intermediates do not change paths or counts.
- `tests/traversal.rs`: cycle and high-degree star terminate within limits;
  cancellation yields an explicit incomplete result; up-to-K ordering is stable
  across runs.
- **Owner ratification required** before this ADR is Accepted and before M3
  closes (P4-X03). Ratification is a decision, not an engineering step: an agent
  may not flip this status.
