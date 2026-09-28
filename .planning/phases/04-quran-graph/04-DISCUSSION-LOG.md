# Phase 4: Quran Graph - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-09-28
**Phase:** 4-Quran Graph
**Areas discussed:** Graph content scope, Annotation & review, Traversal budgets, Surfaces & visualization

---

## Graph content scope

| Option | Description | Selected |
|--------|-------------|----------|
| Structural + word-root only | Structural + word-form graph only; concepts/entities deferred. Smallest exit gate. | |
| Plus concepts/entities | Adds concept/topic + named-entity annotated projections with review workflow. | ✓ (part) |
| Plus translation links | TRANSLATES edges to existing translation editions, refs-only. Tafsir/hadith still deferred. | ✓ (part) |

**User's choice:** "choose by yourself — i recommend all 1,2,3" → locked: structural + word-root + concepts/entities + translation links; tafsir/hadith deferred to Phase 6.
**Notes:** User delegated the final call to the agent with a recommendation for the full scope.

| Option | Description | Selected |
|--------|-------------|----------|
| Active records only | Projection builds only from active, attributed morphology records; typed "no dataset" notice otherwise. Never heuristics. | |
| Resolve dataset here | Drive QAC license verification + bundling inside Phase 4; reopens OD-11. | |
| You decide | Mechanics at agent discretion within I11/I12/I13 + alignment. | ✓ |

**User's choice:** You decide.
**Notes:** OD-11/OD-12 stay BLOCKED; word-root runs on whatever morphology records are active.

| Option | Description | Selected |
|--------|-------------|----------|
| Curated seed list | Fixed curated versioned fixture + user/scholar additions via annotation. | ✓ |
| User-created only | No bundled concepts; all nodes via annotation/review. | |
| You decide | Agent call within PRD 10.3 provenance rules. | |

**User's choice:** Curated seed list.

| Option | Description | Selected |
|--------|-------------|----------|
| In scope annotated | PARALLELS / CONTRASTS_WITH / EXPLAINS / RELATED_TO with provenance + review history. | ✓ |
| Minimal review | Edges as data; only accept/reject commands, no workflow. | |
| Defer to later | Structural + linguistic edges only. | |

**User's choice:** In scope annotated.

---

## Annotation & review

| Option | Description | Selected |
|--------|-------------|----------|
| Full review queue | Manual creation + suggested queue with evidence, accept/reject/correct, reviewer + timestamp. Full PRD 10.6. | ✓ |
| Manual only | Manual creation with attribution; no suggestion pipeline. | |
| You decide | Agent call; no suggestion verified without human decision. | |

**User's choice:** Full review queue.

| Option | Description | Selected |
|--------|-------------|----------|
| Statuses + supersede | verification_status lifecycle; corrections supersede, never overwrite; SUPPORTED_BY/DISPUTED_BY first-class. | ✓ |
| Simple accept/reject | Accepted/rejected only; corrections overwrite with audit event. | |
| You decide | Agent call within authority-vs-projection rule. | |

**User's choice:** Statuses + supersede.

| Option | Description | Selected |
|--------|-------------|----------|
| Attributed authors | Scholar vs user-created classification per PRD 10.3; no anonymous verified edges. | |
| Single operator | All edges attributed to the single local user. | |
| You decide | Agent call within deny-by-default + approval-gated rules. | ✓ |

**User's choice:** You decide.

| Option | Description | Selected |
|--------|-------------|----------|
| Tombstone + repair | Rejected/superseded tombstoned (hidden immediately, retained for audit); explicit repair/GC. | ✓ |
| Hard delete | Hard-delete with audit event only. | |
| You decide | Agent call; tombstoned data disappears from results immediately. | |

**User's choice:** Tombstone + repair.

---

## Traversal budgets

| Option | Description | Selected |
|--------|-------------|----------|
| Accept ADR-0217 | 6 hops / 500 nodes / 2000 edges / 128 fanout / 10 paths / 5s timeout as specified. | |
| Tighter | Tighter local-MVP budgets; raise later on evidence. | |
| You decide | Agent call within pre-flight-error + typed-incomplete + no-path-requires-completeness. | ✓ |

**User's choice:** You decide.

| Option | Description | Selected |
|--------|-------------|----------|
| Banner + fields | Prominent TRUNCATED banner + reason in human output; truncated + incomplete_reason in JSON. | |
| Fields only | Silent fields; caller must check flags. | |
| You decide | Agent call; truncation never renders as absence. | ✓ |

**User's choice:** You decide.

| Option | Description | Selected |
|--------|-------------|----------|
| All path modes | Reachability + shortest (min-hop) + up-to-K ranked paths. | ✓ |
| Shortest only | Shortest path only; rest deferred. | |
| You decide | Agent call within bounded-budget rules. | |

**User's choice:** All path modes.

| Option | Description | Selected |
|--------|-------------|----------|
| Fixed allowlist | Root-family chains, ayah-neighborhood, entity-shared verses, concept-within-N-hops. No ad-hoc patterns. | |
| Custom patterns | Pattern DSL/builder for custom typed patterns. | |
| You decide | Agent call; no backend query language over the port either way. | ✓ |

**User's choice:** You decide.

---

## Surfaces & visualization

| Option | Description | Selected |
|--------|-------------|----------|
| Full CLI tree | neighbors/path/subgraph/export + build/inspect/review/doctor-repair; build management CLI-only. | ✓ |
| Reads only | Read ops only; build + review via other means. | |
| You decide | Agent call within thin-edge + application-service conventions. | |

**User's choice:** Full CLI tree.

| Option | Description | Selected |
|--------|-------------|----------|
| Full read parity | Every CLI read op has HTTP route + typed tool with identical results. Mutations CLI-only. | ✓ |
| CLI first | CLI first; HTTP + tools cover neighbors/path only. | |
| You decide | Agent call within Envelope + ToolResult conventions. | |

**User's choice:** Full read parity (one question call was interrupted mid-area; re-asked cleanly, same answer).

| Option | Description | Selected |
|--------|-------------|----------|
| Static rendering | Graph JSON v1 + static SVG/DOT neighborhood rendering. No interactive explorer (Phase 5). | ✓ |
| Export only | JSON export + CLI tables; no rendering. | |
| You decide | Agent call; full explorer stays in Phase 5. | |

**User's choice:** Static rendering.

| Option | Description | Selected |
|--------|-------------|----------|
| Full explainability | Full PRD 10.5 field set in human + JSON output. | ✓ |
| Compact + json | Compact human output; full detail in --json only. | |
| You decide | Agent call within every-result-explains-why guarantee. | |

**User's choice:** Full explainability.

---

## Final gate

| Option | Description | Selected |
|--------|-------------|----------|
| Explore more gray areas | Additional areas (backend policy, export detail, manifests/doctor, perf budgets). | |
| I'm ready for context | Write 04-CONTEXT.md from captured decisions. | ✓ |

**User's choice:** I'm ready for context.

---

## Agent's Discretion

- Word-root projection build mechanics (within I11/I12/I13 + alignment).
- Edge authorship/attribution model (within PRD 10.3 + deny-by-default + approval gates).
- Budget values within ADR-0217 ranges; truncation rendering (never as absence); pattern allowlist vs custom (no query language over the port).

## Deferred Ideas

None raised by the user — discussion stayed within phase scope. Standard deferrals recorded in CONTEXT.md: tafsir/hadith/isnad links (Phases 6–7), interactive explorer + GUI/TUI (Phase 5), CozoDB/sqlite-graph spikes (optional), GraphML (deferred lossy), auth/RBAC + production (Phases 11–12).
