//! Metric catalog for the observability crate.
//!
//! Contains all the metrics specified in plan §D0.8:
//! - `qai_jobs_enqueued_total{kind}`
//! - `qai_jobs_completed_total{kind,outcome}`
//! - `qai_job_duration_seconds{kind}` (histogram)
//! - `qai_db_query_duration_seconds{op}` (histogram)
//! - `qai_db_pool_in_use{pool}`
//! - `qai_audit_events_total{action}`
//! - `qai_config_reloads_total{outcome}`
//! - `qai_errors_total{code}`
//! - `qai_doctor_checks_total{check,status}`

use std::sync::OnceLock;

/// Initialize all metrics in the catalog.
///
/// This registers all metric instruments with the global metrics recorder.
/// It should be called early in application startup, after setting up
/// the global recorder but before using any metrics.
pub fn init() {
    // The `metrics` crate's declarative macros create metrics
    // lazily on first use. Calling them here establishes the
    // catalog. The convenience wrappers below use the same names.

    static INIT: OnceLock<()> = OnceLock::new();
    let _ = INIT.get_or_init(|| {
        // Register all metric names by creating them once.
        // The macros are idempotent — subsequent calls return
        // the existing instrument.
        let _ = metrics::counter!("qai_jobs_enqueued_total");
        let _ = metrics::counter!("qai_jobs_completed_total");
        let _ = metrics::histogram!("qai_job_duration_seconds");
        let _ = metrics::histogram!("qai_db_query_duration_seconds");
        let _ = metrics::gauge!("qai_db_pool_in_use");
        let _ = metrics::counter!("qai_audit_events_total");
        let _ = metrics::counter!("qai_config_reloads_total");
        let _ = metrics::counter!("qai_errors_total");
        let _ = metrics::counter!("qai_doctor_checks_total");
    });
}

/// Record a job enqueue event.
pub fn enqueue_job(kind: &str) {
    metrics::counter!("qai_jobs_enqueued_total", "kind" => kind.to_string()).increment(1);
}

/// Record a job completion.
pub fn complete_job(kind: &str, outcome: &str) {
    metrics::counter!(
        "qai_jobs_completed_total",
        "kind" => kind.to_string(),
        "outcome" => outcome.to_string()
    )
    .increment(1);
}

/// Record job duration.
pub fn observe_job_duration(kind: &str, duration_secs: f64) {
    metrics::histogram!("qai_job_duration_seconds", "kind" => kind.to_string())
        .record(duration_secs);
}

/// Record database query duration.
pub fn observe_db_query_duration(operation: &str, duration_secs: f64) {
    metrics::histogram!("qai_db_query_duration_seconds", "op" => operation.to_string())
        .record(duration_secs);
}

/// Set the number of in-use database connections.
pub fn set_db_pool_in_use(pool: &str, value: f64) {
    metrics::gauge!("qai_db_pool_in_use", "pool" => pool.to_string()).set(value);
}

/// Record an audit event.
pub fn record_audit_event(action: &str) {
    metrics::counter!("qai_audit_events_total", "action" => action.to_string())
        .increment(1);
}

/// Record a configuration reload.
pub fn record_config_reload(outcome: &str) {
    metrics::counter!("qai_config_reloads_total", "outcome" => outcome.to_string())
        .increment(1);
}

/// Record an error.
pub fn record_error(code: &str) {
    metrics::counter!("qai_errors_total", "code" => code.to_string()).increment(1);
}

/// Record a doctor check result.
pub fn record_doctor_check(check: &str, status: &str) {
    metrics::counter!(
        "qai_doctor_checks_total",
        "check" => check.to_string(),
        "status" => status.to_string()
    )
    .increment(1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_catalog_initialization() {
        init();
        // If we get here without panicking, the catalog is initialized.
        // The metrics crate creates instruments lazily; init()
        // just ensures they exist.
    }

    #[test]
    fn test_record_metrics() {
        init();
        // Recording metrics should not panic.
        enqueue_job("test_job");
        complete_job("test_job", "success");
        observe_job_duration("test_job", 1.5);
        observe_db_query_duration("SELECT", 0.05);
        set_db_pool_in_use("default", 3.0);
        record_audit_event("config_change");
        record_config_reload("success");
        record_error("QAI-TEST-0001");
        record_doctor_check("database.reachable", "pass");
    }
}