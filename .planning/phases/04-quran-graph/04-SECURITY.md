---
phase: "04"
slug: "quran-graph"
status: verified
threats_open: 0
asvs_level: 1
created: "2026-10-13"
---

# Phase 04 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.
> Source: `<threat_model>` blocks in `04-01-PLAN.md` … `04-05-PLAN.md` (all five
> plans authored with a parseable threat model → `register_authored_at_plan_time: true`).
> Posture: ASVS Level 1; unmitigated `high` blocks. L1 verification depth applied;
> short-circuit taken per workflow (threats_open: 0, register authored at plan time,
> ASVS 1). No new network/auth trust boundary class was introduced beyond the
> graph HTTP routes, which sit on the existing shared router layers behind the
> same body-limit/timeout/concurrency envelope as Phase 3.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| operator CLI → SQLite graph tables | CLI flags select projections and nodes; untrusted strings cross here | CLI flags (untrusted) → graph lookup |
| SQLite adapter → traversal results | Row data becomes neighbor/path results; tombstoned and restricted rows must not cross | graph rows → neighbor/path results |
| graph records → canonical reader | Graph carries refs plus hashes only; display text crosses from the reader, never from graph rows | refs + hashes → pinned quotation |
| researcher input → assertion authority | Operator-supplied claim/evidence/source_location crosses here; shape-validated, source-attributed | claim/evidence (untrusted) → authority row |
| review decision → verified edge set | A human accept promotes a suggestion into traversal/export visibility | review decision → verified set |
| rebuild job → authority tables | Adjacency wipe must never touch authority rows | rebuild → adjacency only |
| CLI flags → path/pattern engine | Mode selectors, K values, pattern steps cross here; validated before I/O | selectors/K/steps (untrusted) → engine |
| subgraph result → export serializer | In-memory result crosses into bytes; policy filter sits between | result → filtered bytes |
| export bytes → local files | Graph JSON, DOT, SVG written to operator paths | bytes → local files |
| HTTP JSON body → graph query | Node ids, pattern steps, budget numbers cross here; validated pre-I/O | request body (untrusted) → query |
| tool call → read service | Agent-supplied arguments cross here; same validation as CLI/HTTP | tool args (untrusted) → read service |
| error path → HTTP/tool caller | Failures cross here; must not leak hidden graph content | error → code + scope descriptor |
| operator confirm → repair mutation | A typed yes crosses into destructive action; audit records it | confirmation → repair/GC |
| doctor read → operator terminal | Health findings cross here; severity must be honest, never inflated or softened | findings → terminal |
| GC retention → authority rows | Age-based deletion crosses here; live rows must be unreachable | retention gate → tombstone delete |

---

## Threat Register

| Threat ID | Category | Component | Severity | Disposition | Mitigation | Status |
|-----------|----------|-----------|----------|-------------|------------|--------|
| T-04-01 | Denial of service | quran_graph_store.rs frontier expansion | high | mitigate | `QueryBudgets` pre-flight plus in-expansion node/edge/fanout counters plus per-batch cancel flag; exhaustion returns typed incomplete, never hangs; `validate_k` (`quran_graph_api.rs` L881) rejects out-of-range K with `BudgetExceeded` QAI-GRAPH-0002; pinned by parity-hardening tests (plan 04-01) | closed |
| T-04-02 | Tampering/Injection | single-hop SELECT construction | high | mitigate | Parameterized queries only (`?` bindings throughout `quran_graph_store.rs`); stable IDs never interpolated into SQL; pattern names validated against `EDGE_VOCABULARY` before any I/O; `no_query_language` test names `validate_pattern` as the only edge-name path | closed |
| T-04-03 | Information disclosure | graph record payloads | medium | mitigate | Refs plus hashes only in nodes/edges/exports; per-hit quotation resolves through the reader; `no_canonical_text_embedded` (`quran-graph/src/structural.rs` L327) and refs-only translation-edge cases (`application/tests/graph_build.rs` L313/371) pin it | closed |
| T-04-04 | Spoofing | suggest/accept path | medium | mitigate | Layer-D CHECK requires algorithm plus version plus confidence; accept requires reviewer plus decided_at; suggestions labeled pending in every surface until decided; pinned by graph_review cases (04-02 SUMMARY threat flags confirm) | closed |
| T-04-05 | Information disclosure | tombstoned/restricted assertions | high | mitigate | `is_effective` plus `AuthzScope` applied during expansion (`quran-graph/src/mem.rs` L69; store consults both); `visible_export_sets` allowlist applied pre-serialization; byte-absence of tombstoned ids asserted in tests | closed |
| T-04-06 | Spoofing | word-root builder inputs | medium | mitigate | D-03 gate: active attributed dataset only, typed `UnavailableDataset` (QAI-MORPH-0004) otherwise (`quran_graph_build.rs` L451-468); `synthetic_test_only` labeling enforced; no heuristic root invention; pinned by graph_build cases | closed |
| T-04-07 | Denial of service | up-to-K path enumeration | high | mitigate | max_paths plus max_edges plus max_hops enforced during expansion, never a trailing LIMIT; K beyond range is a pre-flight error (`validate_k`); pinned by graph_paths exhaustion cases plus CLI `--paths 11` exit-3 (04-03 threat flags) | closed |
| T-04-08 | Tampering/Injection | pattern step input | medium | mitigate | Typed `Pattern` plus `EDGE_VOCABULARY` allowlist via `validate_pattern` (`quran-graph/src/pattern.rs` L99; rejects empty/unknown/raw-text); no-query-language pin kept green; parameterized SQL only | closed |
| T-04-09 | Tampering | truncated result rendering | medium | mitigate | `truncated` plus non-empty `incomplete_reason` on every partial; no-path requires completeness enforced in type plus test (`tool-registry/tests/conformance.rs` L842); CLI prints reason verbatim | closed |
| T-04-10 | Information disclosure | export serialization | high | mitigate | `retain_visible` plus `assertion_allowlist_predicate` composed with `visible_export_sets` before serialization (`quran_graph_export.rs` L75-78); tombstoned and restricted ids asserted absent from bytes; pinned by graph_export cases | closed |
| T-04-11 | Information disclosure | typed error mapping | medium | mitigate | `graph_error_status` (`server/src/api.rs` L1723) maps codes only — 422 pre-flight (0002/0003), 404 unknown (0001/0004/0007) plus unavailable dataset (QAI-MORPH-0004), 403 denied (0006), 500 build/storage; unknown vs truncated vs denied distinguishable, codes never leak hidden IDs | closed |
| T-04-12 | Spoofing | tool envelopes for suggestions | medium | mitigate | `suggestion_warnings` labels pending layer-D edges in envelope warnings with algorithm attribution (`quran_graph_tools.rs` L106-124: "pending human review … not verified scholarship"); suggestions never appear in verified result sets; pinned by graph_tools cases | closed |
| T-04-13 | Tampering | ETag and conditional GET | low | mitigate | ETag derived from response meta via the existing `etag_for` helper (`server/src/api.rs` L217/283/399); no new caching semantics invented | closed |
| T-04-14 | Tampering | run_quran_graph_checks | medium | mitigate | One rolled-back `UnitOfWork`, no write method invoked (`quran_graph_doctor.rs` L141/152); doctor is read-only by construction ("never inside doctor" repairs); before-and-after state comparison tests prove immutability | closed |
| T-04-15 | Tampering | repair and tombstone-gc | high | mitigate | Explicit confirmation required (`confirmed: true`, wired to `--yes`/APPROVE); one `DoctorRepairExecuted` audit event per repair; GC retention refuses pending, accepted, and disputed rows by SQL decision filter (`quran_graph_doctor.rs` L802-810); quarantine before deletion (before/after counts); pinned by repair cases | closed |
| T-04-16 | Repudiation | owner-gate ledger | low | mitigate | `docs/05-followups/phase-04-owner-gates.md` states it closes nothing ("Agent-uncloseable"); `decisions-needed.md` (OD-11/OD-12 🔴) stays the source of truth; P4-X01…X05 ratification tracked in swimlane X; no silent passes | closed |
| T-04-SC | Tampering | package installs | low | accept | The only Cargo change in the phase (4347709) adds `[dev-dependencies]` only — workspace-member test wiring (`application`, `storage-sqlite`, `sqlx`, `tempfile`, `tokio`); zero new production dependencies. Recorded as accepted risk AR-04-01 | closed |

*Status: open · closed · open — below high threshold (non-blocking)*
*Severity: critical > high > medium > low — only open threats at or above workflow.security_block_on count toward threats_open*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|
| AR-04-01 | T-04-SC | Zero new production dependencies; the single Cargo change is dev-only workspace-member test wiring; `arch-check` edge gate remains | plans 04-01…04-05 | 2026-10-13 |

*Accepted risks do not resurface in future audit runs.*

---

## Residual Observations (non-blocking, below high threshold)

Owner gates OD-11 / OD-12 plus P4-X01…X05 remain human-only and are **not**
security threats to the engine — they gate licensed activation, scholarly
ratification of the word-root projection, and ADR ratification. They are
tracked in `docs/05-followups/phase-04-owner-gates.md`,
`docs/05-followups/phase-04-deferrals.md`, and
`docs/05-followups/decisions-needed.md`; `T-04-16` verifies no agent has
closed them. No residual security observation at or above the high threshold
was found. No new threats were discovered during this refresh: the auditor
scope was verify-mitigations-exist against the plan-time register (no new-threat
scan per `register_authored_at_plan_time: true`).

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-10-13 | 17 | 17 | 0 | secure-phase stale-verification refresh (plan-register + grep evidence + SUMMARY threat flags, ASVS L1) |

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-10-13
