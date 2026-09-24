# ADR-0218 — Graph Export Formats (Graph JSON v1 mandatory; GraphML deferred)

- Status: Proposed (exporter implemented and tested; owner ratification pending — P4-X05)
- Phase: 3 (dir: phase-04) — Quran Knowledge Graph
- Date: 2026-09-24
- Related decisions: ADR-0202 (no query-language leakage; references + hashes
  only), ADR-0217 (budgets and exhaustion semantics)
- Requirements: PRD §24.4 (Graph JSON / GraphML export), plan §5 M6, AC-P4-23
- Implementation: `crates/quran-graph/src/export.rs` (`GRAPH_JSON_FORMAT`,
  `ExportNotice`, `export_json_with_notice`, `retain_visible`)

## Context

An exported graph is the artifact other tools, reviews, and future backends
consume, so its shape is a contract. Three constraints make format choice more
than a convenience:

1. **No canonical text.** Graph records carry references and hashes only; Arabic
   text is resolved through the reader (ADR-0202 §8.1). An export that embedded
   verses would create a second, unattributed copy of scripture.
2. **Attribution is not optional.** Every non-structural edge must resolve to
   assertion + evidence (source id/location, author/algorithm, version,
   confidence, verification status, timestamp). An export that drops
   assertions while keeping edges would launder opinions into bare structure.
3. **Bounded output.** An export is a query result, not a database dump. Partial
   results must be labelled, and authorization/tombstone policy must apply to
   the exported set exactly as it does during traversal.

GraphML would additionally serve external graph tooling, but it has no
representation for assertion authority, evidence links, versions, or
truncation reasons; those would either be dropped or smuggled into attributes,
which is precisely how attribution gets lost.

## Options Considered

| Option | Advantages | Disadvantages |
|---|---|---|
| GraphML only (rejected) | Opens in off-the-shelf graph tools | No native place for assertions/evidence/versions/truncation; identity and provenance get flattened into free-form attributes |
| Graph JSON v1 only, versioned envelope (chosen) | Lossless for every domain concept already modelled; stable `format` marker; re-importable by this project and testable without a parser dependency | Consumers need the project's own reader; not directly viewable in graph tools |
| Both, generated from one model (deferred) | Best of both | Two places to keep honest; GraphML is explicitly "where practical" in the PRD and is not needed for the phase's exit criteria |

## Decision

1. **`quran-graph-json-v1` is the mandatory, versioned export format.** The
   document is `format` + `manifest` + `nodes` + `edges` + `assertions` +
   `counts` + `truncation`. The `format` marker is a constant, so a future
   incompatible shape is a new marker, never a silent field change.
2. **Assertions travel with edges.** Interpretive edges are never exported
   without their assertion records; a consumer cannot mistake an attributed
   claim for a structural fact. Provenance layers stay distinguishable
   (structural builder provenance vs attributed assertion).
3. **Truncation is explicit.** Every document carries
   `truncation.truncated` and, when truncated, a non-empty
   `incomplete_reason` — the same semantics as ADR-0217. A complete export
   states `truncated: false`; it never looks partial by omission.
4. **Policy applies to the export.** Tombstoned assertions and edges outside the
   caller's authorization scope are removed before serialization, using the same
   predicate traversal uses; restricted evidence cannot be recovered by asking
   for a bigger export.
5. **No text, no unbounded dumps.** Node/edge/assertion records carry
   references + hashes only. Export size is bounded by the caller's query
   budgets; there is no "export everything" path in this phase.
6. **GraphML is deferred, not rejected.** It may be added later as a derived,
   lossy view generated from the same `quran-graph-json-v1` document, with
   assertion/evidence fields mapped into documented attributes and the
   truncation banner rendered in a graph-level attribute. The lossy mapping
   must be declared in the file itself; a GraphML consumer must never mistake it
   for the authoritative form.
7. **No query language travels with an export.** Neither format may embed SQL,
   Cypher, or Datalog.

## Accuracy and Religious-Source Implications

A downstream consumer can tell, from the document alone, which edges are
structural facts, which are attributed claims, and whether the set is
complete. Attribution, version, and verification status are lossless in the
mandatory format, so an exported graph cannot launder a scholar's opinion — or
a superseded/rejected claim — into a bare relationship. Deferred GraphML is
explicitly labelled lossy for the same reason.

## Licensing Implications

None. Both candidate formats are specifications, not dependencies; the
implementation uses `serde_json` already in the workspace.

## Security Implications

Export is an exfiltration surface, so it is treated as one: authorization and
effective-tombstone filtering run before serialization, canonical text is never
present to leak, budgets bound the output, and the lossy GraphML view is
additive-only. No export path accepts a caller-supplied query string.

## Operational Implications

The format marker is the compatibility contract; consumers (CLI `graph export`,
HTTP export route, doctor diffs) must check it before parsing. Truncation
reasons must be surfaced verbatim in user-facing output so an operator can
distinguish a budget stop from a genuinely small graph.

## Migration Strategy

`quran-graph-json-v1` stays readable forever. Additive optional fields are
allowed within v1; any change to the meaning of an existing field requires a new
marker (`quran-graph-json-v2`) and a migration note. GraphML, if added, is
generated from v1 and is versioned separately.

## Reversal Cost

Medium. Choosing GraphML-first would have required re-inventing assertion and
evidence encoding as attributes and would have made the lossy view
authoritative-looking; the chosen order keeps the lossless form canonical and
leaves the lossy form disposable.

## Acceptance Criteria

- `crates/quran-graph/src/export.rs` tests: format marker, counts, truncation
  notice on a budget-limited collection, tombstone/authz filtering
  (`retain_visible` + the standard predicate).
- Export inspection shows references + hashes only and full attribution for
  every non-structural edge (AC-P4-23).
- **Owner ratification required** before Accepted and before M6 closes (P4-X05).
