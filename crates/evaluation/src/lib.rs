//! Evaluation harness for Q-ai (P2-T108, D-3.5-07).
//!
//! This crate provides a regression-measuring instrument: versioned datasets,
//! metric computation, and report rendering. It gates **mechanical**
//! regressions only — it asserts nothing about linguistic correctness
//! (ADR-0204/0211 are Draft). A metric whose dataset or lexicon is absent
//! reports `Unavailable` with the import command as remedy — never `0`,
//! which would read as a real measurement.

pub mod dataset;
pub mod metrics;
pub mod report;

pub use dataset::{Dataset, DatasetError, DatasetHeader, load_dataset};
pub use metrics::{
    Metric, MetricValue, compare_metric, compute_lexicon_metrics, compute_search_metrics,
};
pub use report::{Report, render_human, render_json};

/// The scope note that appears in every report. States plainly that the
/// harness is not a linguistic quality verdict.
pub const SCOPE_NOTE: &str =
    "Mechanical regression gate only — not a linguistic quality verdict (ADR-0204/0211 Draft)";

/// The remedy reported when a dataset is absent.
pub const DATASET_REMEDY: &str = "import and activate a morphology dataset first";
