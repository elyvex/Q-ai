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
