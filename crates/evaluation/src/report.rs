//! Report rendering for the evaluation harness (P2-T108, D-3.5-07).
//!
//! Renders human-readable and JSON reports. Every report carries the dataset
//! version, the profile, and a scope note stating that the harness gates
//! mechanical regressions only — it is not a linguistic quality verdict.

use crate::metrics::{Metric, MetricValue};

/// A full evaluation report.
#[derive(Debug, Clone)]
pub struct Report {
    /// Dataset version.
    pub dataset_version: u32,
    /// Normalization profile used.
    pub profile: String,
    /// Computed metrics.
    pub metrics: Vec<Metric>,
    /// Scope note (always states this is not a linguistic verdict).
    pub scope_note: String,
}

/// Render a human-readable report.
pub fn render_human(report: &Report) -> String {
    let mut out = String::new();
    out.push_str(&format!("Evaluation Report (dataset v{})\n", report.dataset_version));
    out.push_str(&format!("Profile: {}\n", report.profile));
    out.push_str(&format!("Scope: {}\n", report.scope_note));
    out.push_str("Metrics:\n");
    for metric in &report.metrics {
        match &metric.value {
            MetricValue::Rate(rate) => {
                out.push_str(&format!(
                    "  {}: {:.2}% (tolerance: {:.2}%)\n",
                    metric.name,
                    rate * 100.0,
                    metric.tolerance * 100.0
                ));
            }
            MetricValue::Count(count) => {
                out.push_str(&format!(
                    "  {}: {} (tolerance: {})\n",
                    metric.name, count, metric.tolerance
                ));
            }
            MetricValue::Unavailable { remedy } => {
                out.push_str(&format!("  {}: Unavailable (remedy: {})\n", metric.name, remedy));
            }
        }
    }
    out
}

/// Render a JSON report.
pub fn render_json(report: &Report) -> serde_json::Value {
    let metrics: Vec<serde_json::Value> = report
        .metrics
        .iter()
        .map(|m| {
            let value = match &m.value {
                MetricValue::Rate(rate) => serde_json::json!({ "rate": rate }),
                MetricValue::Count(count) => serde_json::json!({ "count": count }),
                MetricValue::Unavailable { remedy } => {
                    serde_json::json!({ "unavailable": true, "remedy": remedy })
                }
            };
            serde_json::json!({
                "name": m.name,
                "value": value,
                "tolerance": m.tolerance,
            })
        })
        .collect();

    serde_json::json!({
        "dataset_version": report.dataset_version,
        "profile": report.profile,
        "scope_note": report.scope_note,
        "metrics": metrics,
    })
}
