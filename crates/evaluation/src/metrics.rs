//! Metric computation for the evaluation harness (P2-T108, D-3.5-07).
//!
//! Computes the metric set per search mode and per lexicon tool. Every metric
//! carries an explicit tolerance; the comparison is exact against the declared
//! tolerance, never a fuzzy similarity.

/// A single metric with its value and tolerance.
#[derive(Debug, Clone)]
pub struct Metric {
    /// Metric name (e.g., `search.golden_agreement`).
    pub name: String,
    /// The measured value.
    pub value: MetricValue,
    /// Explicit tolerance for baseline comparison.
    pub tolerance: f64,
}

/// The value of a metric.
#[derive(Debug, Clone)]
pub enum MetricValue {
    /// A rate in [0, 1].
    Rate(f64),
    /// A count.
    Count(u64),
    /// Unavailable with a remedy (never zero, which would read as a real
    /// measurement).
    Unavailable { remedy: String },
}

/// Compute the search metric set.
///
/// - `golden_agreement` — fraction of golden queries whose reference set
///   matches the service output.
/// - `empty_result_rate` — fraction of queries that returned zero hits.
/// - `determinism_rate` — fraction of queries whose output is byte-identical
///   across two runs.
pub fn compute_search_metrics(
    golden_agreement: f64,
    empty_result_rate: f64,
    determinism_rate: f64,
) -> Vec<Metric> {
    vec![
        Metric {
            name: "search.golden_agreement".to_string(),
            value: MetricValue::Rate(golden_agreement),
            tolerance: 0.0,
        },
        Metric {
            name: "search.empty_result_rate".to_string(),
            value: MetricValue::Rate(empty_result_rate),
            tolerance: 0.0,
        },
        Metric {
            name: "search.determinism".to_string(),
            value: MetricValue::Rate(determinism_rate),
            tolerance: 0.0,
        },
    ]
}

/// Compute the lexicon metric set.
///
/// - `attribution_completeness` — fraction of lexicon results that carry
///   dataset attribution.
/// - `typed_unavailable_rate` — fraction of no-dataset calls that return the
///   typed `QAI-MORPH-0004` error (never an empty result).
/// - `determinism_rate` — fraction of calls whose output is byte-identical
///   across two runs.
pub fn compute_lexicon_metrics(
    attribution_completeness: f64,
    typed_unavailable_rate: f64,
    determinism_rate: f64,
) -> Vec<Metric> {
    vec![
        Metric {
            name: "lexicon.attribution_completeness".to_string(),
            value: MetricValue::Rate(attribution_completeness),
            tolerance: 0.0,
        },
        Metric {
            name: "lexicon.typed_unavailable".to_string(),
            value: MetricValue::Rate(typed_unavailable_rate),
            tolerance: 0.0,
        },
        Metric {
            name: "lexicon.determinism".to_string(),
            value: MetricValue::Rate(determinism_rate),
            tolerance: 0.0,
        },
    ]
}

/// Compare a metric against a baseline value with the metric's explicit
/// tolerance. Returns `true` if the metric is within tolerance (pass), `false`
/// if it is outside tolerance (fail).
pub fn compare_metric(metric: &Metric, baseline: f64) -> bool {
    match &metric.value {
        MetricValue::Rate(rate) => (rate - baseline).abs() <= metric.tolerance,
        MetricValue::Count(count) => (*count as f64 - baseline).abs() <= metric.tolerance,
        MetricValue::Unavailable { .. } => false,
    }
}
