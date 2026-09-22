## Conflict Detection Report

Ingest set: 36 classified documents (33 ADRs, 2 SPECs, 1 PRD), MODE=new
(bootstrap, no existing .planning context). Precedence: ADR > SPEC > PRD.
Bucket mapping: BLOCKERS = unresolved-blockers, WARNINGS = competing-variants,
INFO = auto-resolved.

### BLOCKERS (0)

(none)

### WARNINGS (0)

(none)

### INFO (8)

[INFO] Cross-reference cycles are benign navigation links, synthesis unaffected
  Found: cross_refs graph over the 36 classified docs contains 8 reference circuits in 3 strongly connected groups: (a) ADR-0001 <-> ADR-0201 <-> ADR-0701 <-> ADR-0702 (relational store, full-text engine, vector store, cross-store consistency); (b) ADR-0201 <-> ADR-0203 (full-text engine, morphology dataset); (c) ADR-0004 <-> ADR-0005 (config precedence, secret storage)
  Note: every edge is a "Related decisions" backlink between docs that each carry a standalone Decision section; extraction is per-doc and no synthesis step traverses cross_refs, so there is no traversal loop and no garbage-synthesis mechanism. Graph depth is far below the 50-level cap (36 nodes). All 36 docs were synthesized; no locked decision was dropped. Rationale logged here for transparency.

[INFO] Auto-resolved: locked ADR-0001 settles PRD open item "primary database"
  Found: source docs/02-architecture/decisions/ADR-0001-relational-store.md (Accepted, locked) selects SQLite as the default authoritative relational store with PostgreSQL-portable contracts
  Note: PRD §25.15 lists "Primary database" among items to evaluate, and PRD §32.1 already states SQLite initial / PostgreSQL future-server, so the locked ADR is the evaluation outcome, consistent with the PRD — ADR wins per ADR > PRD, no contradiction remains.

[INFO] Auto-resolved: locked ADR-0003 settles PRD open item "job queue implementation"
  Found: source docs/02-architecture/decisions/ADR-0003-durable-job-system.md (Accepted, locked) selects a DB-backed leased SQLite queue via the jobs crate with no external broker in local mode
  Note: PRD §25.15 lists "Job queue implementation" among items to evaluate — the locked ADR is the evaluation outcome; ADR wins per ADR > PRD.

[INFO] Auto-resolved: locked ADR-0007 settles PRD open item "source-manifest signing method"
  Found: source docs/02-architecture/decisions/ADR-0007-manifest-format-signing.md (Accepted, locked) selects ed25519 detached signatures over canonical JSON bytes
  Note: PRD §48 lists "Source-manifest signing method" among decisions requiring ADRs — the locked ADR satisfies it; ADR wins per ADR > PRD.

[INFO] Auto-resolved: locked ADR-0111 settles PRD open item "citation-support verification method"
  Found: source docs/02-architecture/decisions/ADR-0111-citation-identity-and-deep-links.md (Accepted, locked) fixes citation identity, deep-link/URN formats, and verify_quotation verdicts with mismatch as a hard failure on answer paths
  Note: PRD §48 lists "Citation-support verification method" among decisions requiring ADRs — the locked ADR satisfies it; ADR wins per ADR > PRD.

[INFO] Stale draft/proposed labels on accepted ADRs 0111, 0112, 0113
  Found: source docs/02-architecture/decisions/ADR-0111-citation-identity-and-deep-links.md, ADR-0112-translation-alignment-and-attribution.md, ADR-0113-canonical-lookup-caching.md carry "(Draft)" in the title or "## Decision (proposed)" headings while their bodies record Accepted status with shipped evidence (P1-T46/T47, P1-T36/T37, M8 consistency test)
  Note: classification locked:true governs, so all three are synthesized as locked; no content conflict exists — downstream consumers should treat the stale labels as editorial cleanup, not as undecided status.

[INFO] SPECs align with their governing ADRs, no SPEC-vs-ADR contradiction
  Found: source docs/architecture/hashing-spec.md restates the ADR-0006 content-hash format and canonical JSON rules; source docs/07-technical/quran-citation-spec.md declares itself frozen in Phase 1 under ADR-0111 and reuses its identity
  Note: both SPECs explicitly defer to higher-precedence ADRs (ADR > SPEC holds with the ADR as winner by agreement); synthesized as constraints without modification.

[INFO] PRD open items with no locked ADR yet remain open for the roadmapper
  Found: PRD §48 items still map to Draft/Proposed ADRs only — initial Quran dataset/license to source docs/02-architecture/decisions/ADR-0101-initial-quran-dataset.md (Draft, pending human sign-off), morphology dataset to source docs/02-architecture/decisions/ADR-0203-quran-morphology-dataset.md (Draft), full-text engine to source docs/02-architecture/decisions/ADR-0201-full-text-engine.md (Proposed), graph store to source docs/02-architecture/decisions/ADR-0202-graph-store.md (Proposed), vector store to source docs/02-architecture/decisions/ADR-0701-vector-store.md (Proposed), retrieval topology to source docs/02-architecture/decisions/ADR-0301-rag-strategy.md (undecided)
  Note: these are not conflicts (a Draft/Proposed ADR does not contradict the PRD that calls for it); recorded so the roadmapper plans explicit decision points instead of assuming settled content.
