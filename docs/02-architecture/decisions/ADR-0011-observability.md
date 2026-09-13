# ADR-0011 — Observability Stack: Tracing + Metrics + Optional OTLP (Telemetry Opt-In)

- Status: Accepted
- Phase: 0 — Foundations
- Date: 2026-09-11
- Related decisions: ADR-0001, ADR-0003
- Requirements: PRD §§39, 25.13

## Context

Q-ai must be debuggable in local-first mode where there is no central logging
infrastructure. Operators need distributed-style tracing spans across the
storage, job, and source subsystems, plus a named metric catalog for future
Grafana dashboards. Privacy is paramount: telemetry must never be enabled
without an explicit opt-in, and certain fields are forbidden from export even
when enabled.

## Options Consided

| Option | Advantages | Disadvantages |
|---|---|---|
| **tracing + metrics + optional OTLP** | Standard Rust ecosystem; env-filter; JSON/text formats; OpenTelemetry compatibility | OTLP must be opt-in; metric naming discipline required |
| slog + custom metrics | Lightweight | Smaller ecosystem; harder to integrate with Grafana/Loki later |
| OpenTelemetry SDK directly | Strongest vendor compatibility | Heavier; requires explicit SDK selection per backend |
| Log-only (no metrics) | Zero complexity | No dashboards; no request-level tracing across subsystems |

## Decision

Deploy `tracing` + `tracing-subscriber` (env-filter, two formatters: text, JSON)
plus the `metrics` crate with a **named** metric catalog so Phase 12 dashboards
are stable.

Span conventions (enforced by macro + documented in docs):
`qai.component`, `qai.operation`, `qai.job_id`, `qai.run_id`, `qai.source_id`,
`qai.source_version`, `qai.principal_id`, `qai.workspace_id`,
`qai.duration_ms`, `qai.outcome`.

Metric catalog (Phase 0 + reserved future names):

```text
qai_jobs_enqueued_total{kind}
qai_jobs_completed_total{kind,outcome}
qai_job_duration_seconds{kind}            (histogram)
qai_db_query_duration_seconds{op}         (histogram)
qai_db_pool_in_use{pool}
qai_audit_events_total{action}
qai_config_reloads_total{outcome}
qai_errors_total{code}
qai_doctor_checks_total{check,status}
```

**Privacy gate**: OTLP exporter is behind `telemetry.enabled = false` (opt-in,
PRD §39). A **compile-time-tested** field denylist forbids export of: query text,
prompt text, document content, research questions, model responses.
Denylist violations fail the build (`tests/observability/telemetry_privacy.rs`).

## Accuracy and Religious-Source Implications

None directly. Observability must never export scripture content, research
queries, or AI-generated answers — the denylist is the enforcement mechanism.

## Licensing Implications

`tracing` (MIT/Apache-2.0), `metrics` (MIT/Apache-2.0), `opentelemetry`
(Apache-2.0). No impact.

## Security Implications

- Metrics are scoped; no metric captures secret values or content text.
- OTLP traffic (Phase 1+) requires `tls` to the collector; not allowed over
  plain HTTP outside localhost (enforced in config validation).
- Spans carry `PrincipalId` for accountability; the trace must never carry a
  secret (tested by `secret_leak.rs`).

## Operational Implications

- Log level and format are configurable (`logging.level`, `logging.format`,
  `logging.file`) — default is text to stderr, no file.
- `qai doctor observability.subscriber_installed` checks that a subscriber is
  attached at startup (Phase 0 always installs one).
- OTLP endpoint is blank by default; setting it without `telemetry.enabled = true`
  is a validation error.

## Migration Strategy

The metric catalog is treated as a stable API. Phase 12 can add metrics but
cannot rename or remove Phase 0 metric names without a major-version bump of the
binary (breaking tool integration).

## Reversal Cost

Low for Phase 0 (tracing is ubiquitous); removing the named metric catalog later
would break dashboard compatibility.

## Acceptance Criteria

- `telemetry.enabled = false` by default; enabling it never exports denylisted
  content fields (compile-time + runtime test).
- `qai doctor` checks `observability.subscriber_installed`.
- All Phase 0 metrics are registered in the catalog; no ad-hoc `metrics!(...)`
  outside the catalog (clippy lint).
