# Open Questions

## Graph backends

### Q: Which graph backend should ship for Phase 4?
- **Default:** SQLite adjacency tables + bounded recursive CTEs / batched
  frontier traversal (ADR-0202). No extra runtime dependency.
- **CozoDB spike (TASK-411):** Rust, embeddable, Datalog recursion, MVCC.
  Candidate accelerator for deep transitive workloads (word-root families,
  isnad). License: MPL-2.0. Go/no-go after benchmark.
- **SQLite graph extension spike (TASK-412):** Cypher-in-SQLite; zero extra
  files. Alpha status — current bar: >1k nodes, stable releases. Go/no-go.
- **Kuzu:** ARCHIVED (Oct 2025, read-only). Rejected as future adapter.

### Q: Should word-root graph precompute ayah×ayah edges?
- **Decision (ADR-0202 §8.1):** No. Ayahs reach roots via one-hop through
  tokens. Root hub topology avoids quadratic edge growth while preserving
  per-edge provenance.

## Telemetry & Redaction

### Q: OTLP span-field scrubbing (P0-T16 follow-up, spec Assumptions)
- **Context:** The global tracing redaction layer (P0-T16) scrubs stderr
  output via `RedactingWriter` (Rule A on field names). OTLP span export
  uses a separate exporter pipeline that bypasses stderr — span fields
  (including those matching secret keys) currently reach the OTLP backend
  unscrubbed.
- **Mitigations in place:** Telemetry is off by default; the content-field
  denylist (`is_forbidden_field`) gates export payloads (AC-P0-18).
- **Open:** Add span-field scrubbing for the OTLP exporter when
  `redact_secrets=true` (either a span processor or an OTLP-specific
  `FormatFields` hook). Record as a follow-up task for Phase 3 or later.
