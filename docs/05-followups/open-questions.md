# Open Questions

## Container verification — P0-T56, 2026-09-17

- Added `Dockerfile`, `docker-compose.yml`, and allowlisted `.dockerignore`. Compose configuration validation passed; Docker CLI is installed but its daemon socket is unavailable.
- Runtime image builds, shared-library compatibility, named-volume ownership and non-root execution remain unverified. Once the daemon is available, run `docker compose build`, `docker compose run --rm app db migrate`, `docker compose run --rm app db verify`, then `docker compose up`.
- The stub intentionally uses `network_mode: none` and publishes no ports: the current server permits loopback only. Access stays inside the container network namespace; no unauthenticated public binding is enabled. Database migration is an explicit initialization step, not an automatic startup mutation.
- Verify UID 65532, persisted database access and `/healthz`, `/readyz`, `/api/v1/meta` from inside that network namespace before closing T56.
- Rechecked 2026-09-18: daemon socket still absent; T56 stays partial. Unrelated
  Phase-0 CLI work (FU-10/DEV-02) completed the same session without touching
  container files.
- Rechecked 2026-09-23: daemon socket still absent (`docker info` fails); T56
  stays ◐. No container files touched.

## Jobs scheduling — P0-T39/T40, 2026-09-17

- `cargo test -p jobs` intermittently returned Idle rather than Succeeded in `worker::tests::retries_then_succeeds` after zero-delay rescheduling. Isolated and serial runs pass.
- Timestamp ordering corrected in `InMemoryJobQueue`: claim and lease reaping compare parsed instants, not variable-precision RFC3339 strings. Fixed-time regression tests pass; 25 consecutive parallel jobs-suite runs passed (21 tests each).
- **Update 2026-09-24:**
  - `plus` subsecond truncation **fixed** in `crates/jobs/src/queue.rs`: `plus`
    now adds whole seconds + subsecond nanos instead of `d.as_secs()` only.
    Regression: `queue::tests::plus_preserves_subsecond_delays`
    (`cargo test -p jobs --lib` 22/22 green).
  - SQLite scheduling **validated**: new
    `storage-sqlite/tests/recovery_jobs.rs::rescheduled_job_is_not_claimable_until_due`
    proves `reschedule(job, 3600s)` blocks `claim_next` until `available_at`
    passes, then claims normally (4/4 green). Note the SQLite `reschedule`
    API is seconds-granular by design (`delay_seconds: u64`), so the subsecond
    issue never applied there; all writer timestamps come from one formatter,
    keeping lexical `available_at <= now` comparison chronological.
- Still open: wall-clock lease-recovery retry policy validation under real
  time (backdate simulation only proves ordering, not timing). Intermittent
  parallel `retries_then_succeeds` flake stays under watch.
- Restored failure-path idempotency and jitter-cap tests pass.

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

## Orchestrator second pass — 2026-09-22 (recommendations, pending ratification)

> Neither item closes on orchestrator word alone; both need a human/lead
> ratification recorded here with date.

### Q: Phase-4 graph backend — ratify SQLite-CTE default + conditional CozoDB spike?

- **Recommended:** keep SQLite adjacency + bounded recursive CTEs / batched
  frontier traversal as default (ADR-0202; zero new deps, transactional,
  local-first). Authorize CozoDB spike (TASK-411) only if a Phase-3-exit
  benchmark on a realistic word/root graph misses agreed latency targets;
  park TASK-412 (sqlite-graph) meanwhile. Kuzu stays archived.
- **Needs to close:** latency targets + benchmark dataset + ratifier + date.

### Q: OTLP span scrubbing — ratify exporter-boundary design?

- **Recommended:** telemetry off by default (opt-in only); scrubber at the
  exporter boundary; attribute keys on explicit allowlist (deny-by-default);
  file paths hashed; no Arabic text, user strings, or query payloads in span
  attributes; scrub fixtures covered by unit tests.
- **Needs to close:** ratifier + date + follow-up task ID (Phase 3+).

> Status 2026-09-22: both technicals pending ratification; per owner-authored
> A9 they become effective 2026-09-25 unless the owner objects in writing.
> Tier-2 corpus note: per A1, Tier-2 signer ≠ OD-02 L3 reviewer
> (`single-reviewer-risk` stamp is the fallback).
