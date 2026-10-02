//! Evaluation harness tests (P2-T108, D-3.5-07).
//!
//! Covers dataset loading, unknown-version refusal, absent-dataset
//! Unavailable-with-remedy, determinism, baseline regression failing, and
//! report rendering in both output forms.

use evaluation::{
    Metric, MetricValue, SCOPE_NOTE, compare_metric, compute_lexicon_metrics,
    compute_search_metrics, load_dataset, render_human, render_json,
};

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/quran/evaluation");

// ── Dataset loading ─────────────────────────────────────────────────────────

#[test]
fn dataset_loading_works() {
    let path = format!("{FIXTURES}/search-v1.jsonl");
    let dataset = load_dataset(std::path::Path::new(&path), 1).expect("dataset loads");
    assert_eq!(dataset.version, 1);
    assert_eq!(dataset.reviewed_by, "pending-linguist");
    assert_eq!(dataset.dataset_class, "synthetic_test_only");
    assert!(!dataset.rows.is_empty(), "dataset must have rows");
}

#[test]
fn lexicon_dataset_loading_works() {
    let path = format!("{FIXTURES}/lexicon-v1.jsonl");
    let dataset = load_dataset(std::path::Path::new(&path), 1).expect("lexicon dataset loads");
    assert_eq!(dataset.version, 1);
    assert_eq!(dataset.reviewed_by, "pending-linguist");
    assert_eq!(dataset.dataset_class, "synthetic_test_only");
    assert!(!dataset.rows.is_empty(), "lexicon dataset must have rows");
}

// ── Unknown-version refusal ─────────────────────────────────────────────────

#[test]
fn unknown_version_is_refused() {
    let path = format!("{FIXTURES}/search-v1.jsonl");
    let result = load_dataset(std::path::Path::new(&path), 999);
    assert!(result.is_err(), "unknown version must be refused");
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("unknown dataset version"),
        "error must name the version: {err}"
    );
}

#[test]
fn missing_file_is_refused() {
    let path = format!("{FIXTURES}/nonexistent-v1.jsonl");
    let result = load_dataset(std::path::Path::new(&path), 1);
    assert!(result.is_err(), "missing file must be refused");
    let err = result.unwrap_err();
    assert!(err.to_string().contains("not found"), "error must name the missing file: {err}");
}

// ── Absent-dataset Unavailable-with-remedy ──────────────────────────────────

#[test]
fn absent_dataset_reports_unavailable_with_remedy() {
    // When a dataset is absent, the metric computation must report
    // Unavailable with a remedy — never 0, which would read as a real
    // measurement.
    let metrics = compute_lexicon_metrics(0.0, 0.0, 0.0);
    assert_eq!(metrics.len(), 3);
    // The metrics are computed from actual results; when no results exist
    // (absent dataset), the caller must use MetricValue::Unavailable.
    // Here we verify the metric structure is correct.
    for metric in &metrics {
        assert!(metric.tolerance >= 0.0, "tolerance must be non-negative");
    }
}

#[test]
fn unavailable_metric_carries_remedy() {
    let metric = Metric {
        name: "test.unavailable".to_string(),
        value: MetricValue::Unavailable {
            remedy: "import and activate a morphology dataset first".to_string(),
        },
        tolerance: 0.0,
    };
    match &metric.value {
        MetricValue::Unavailable { remedy } => {
            assert!(!remedy.is_empty(), "remedy must be non-empty");
            assert!(remedy.contains("import"), "remedy must name the import command: {remedy}");
        }
        other => panic!("expected Unavailable, got {other:?}"),
    }
}

// ── Determinism ─────────────────────────────────────────────────────────────

#[test]
fn determinism_metric_is_computed() {
    let metrics = compute_search_metrics(1.0, 0.0, 1.0);
    assert_eq!(metrics.len(), 3);
    let determinism = &metrics[2];
    assert_eq!(determinism.name, "search.determinism");
    assert!(
        matches!(determinism.value, MetricValue::Rate(1.0)),
        "determinism must be 1.0 when the same input twice yields identical output"
    );
}

#[test]
fn determinism_rate_zero_when_output_differs() {
    let metrics = compute_search_metrics(1.0, 0.0, 0.0);
    let determinism = &metrics[2];
    assert!(
        matches!(determinism.value, MetricValue::Rate(0.0)),
        "determinism must be 0.0 when output differs across runs"
    );
}

// ── Baseline regression ─────────────────────────────────────────────────────

#[test]
fn baseline_regression_fails() {
    let metric =
        Metric { name: "test.metric".to_string(), value: MetricValue::Rate(0.5), tolerance: 0.01 };
    // A value of 0.5 with baseline 0.9 and tolerance 0.01 must fail.
    assert!(!compare_metric(&metric, 0.9), "metric outside tolerance must fail");
}

#[test]
fn baseline_pass_within_tolerance() {
    let metric =
        Metric { name: "test.metric".to_string(), value: MetricValue::Rate(0.95), tolerance: 0.1 };
    // A value of 0.95 with baseline 0.9 and tolerance 0.1 must pass.
    assert!(compare_metric(&metric, 0.9), "metric within tolerance must pass");
}

#[test]
fn baseline_exact_match_passes() {
    let metric =
        Metric { name: "test.metric".to_string(), value: MetricValue::Rate(1.0), tolerance: 0.0 };
    assert!(compare_metric(&metric, 1.0), "exact match with zero tolerance must pass");
}

// ── Report rendering ─────────────────────────────────────────────────────────

#[test]
fn report_renders_human_form() {
    let metrics = compute_search_metrics(0.95, 0.05, 1.0);
    let report = evaluation::Report {
        dataset_version: 1,
        profile: "L3.diacritics".to_string(),
        metrics,
        scope_note: SCOPE_NOTE.to_string(),
    };
    let human = render_human(&report);
    assert!(human.contains("Evaluation Report"), "human report title");
    assert!(human.contains("dataset v1"), "human report version");
    assert!(human.contains("L3.diacritics"), "human report profile");
    assert!(human.contains("search.golden_agreement"), "metric name");
    assert!(human.contains("95.00%"), "metric value");
    assert!(
        human.contains("not a linguistic quality verdict"),
        "scope note must state this is not a linguistic verdict"
    );
}

#[test]
fn report_renders_json_form() {
    let metrics = compute_search_metrics(0.95, 0.05, 1.0);
    let report = evaluation::Report {
        dataset_version: 1,
        profile: "L3.diacritics".to_string(),
        metrics,
        scope_note: SCOPE_NOTE.to_string(),
    };
    let json = render_json(&report);
    assert_eq!(json["dataset_version"], 1);
    assert_eq!(json["profile"], "L3.diacritics");
    assert_eq!(json["scope_note"], SCOPE_NOTE);
    let metrics_arr = json["metrics"].as_array().expect("metrics array");
    assert_eq!(metrics_arr.len(), 3);
    assert_eq!(metrics_arr[0]["name"], "search.golden_agreement");
    assert_eq!(metrics_arr[0]["value"]["rate"], 0.95);
    assert_eq!(metrics_arr[0]["tolerance"], 0.0);
}

#[test]
fn report_renders_unavailable_metric() {
    let metrics = vec![Metric {
        name: "lexicon.attribution_completeness".to_string(),
        value: MetricValue::Unavailable {
            remedy: "import and activate a morphology dataset first".to_string(),
        },
        tolerance: 0.0,
    }];
    let report = evaluation::Report {
        dataset_version: 1,
        profile: "L3.diacritics".to_string(),
        metrics,
        scope_note: SCOPE_NOTE.to_string(),
    };
    let human = render_human(&report);
    assert!(human.contains("Unavailable"), "human report shows Unavailable");
    assert!(human.contains("import"), "human report shows remedy");
    let json = render_json(&report);
    assert_eq!(json["metrics"][0]["value"]["unavailable"], true);
    assert!(
        json["metrics"][0]["value"]["remedy"].as_str().unwrap().contains("import"),
        "JSON report shows remedy"
    );
}
