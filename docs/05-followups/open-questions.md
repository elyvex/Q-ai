# Open Questions

## Container verification — P0-T56, 2026-09-17

- Added `Dockerfile`, `docker-compose.yml`, and allowlisted `.dockerignore`. Compose configuration validation passed; Docker CLI is installed but its daemon socket is unavailable.
- Runtime image builds, shared-library compatibility, named-volume ownership and non-root execution remain unverified. Once the daemon is available, run `docker compose build`, `docker compose run --rm app db migrate`, `docker compose run --rm app db verify`, then `docker compose up`.
- The stub intentionally uses `network_mode: none` and publishes no ports: the current server permits loopback only. Access stays inside the container network namespace; no unauthenticated public binding is enabled. Database migration is an explicit initialization step, not an automatic startup mutation.
- Verify UID 65532, persisted database access and `/healthz`, `/readyz`, `/api/v1/meta` from inside that network namespace before closing T56.
- Rechecked 2026-09-18: daemon socket still absent; T56 stays partial. Unrelated
  Phase-0 CLI work (FU-10/DEV-02) completed the same session without touching
  container files.

## Jobs scheduling — P0-T39/T40, 2026-09-17

- `cargo test -p jobs` intermittently returned Idle rather than Succeeded in `worker::tests::retries_then_succeeds` after zero-delay rescheduling. Isolated and serial runs pass.
- Timestamp ordering corrected in `InMemoryJobQueue`: claim and lease reaping compare parsed instants, not variable-precision RFC3339 strings. Fixed-time regression tests pass; 25 consecutive parallel jobs-suite runs passed (21 tests each).
- Still open: `plus` discards subsecond delays; production SQLite scheduling and wall-clock behavior need separate validation. This fix does not address those paths.
- Restored failure-path idempotency and jitter-cap tests pass. Lease-recovery retry policy still needs separate validation.

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
