//! Phase 2 — analysis policy (`policy`, ADR-0209).
//!
//! Competing analyses of one token stay side-by-side: there is no winner,
//! primary, or selected field anywhere in this module. An
//! [`AnalysisPolicy`] selects which datasets a consumer reads; everything
//! else is suppressed — and suppression is **always counted, never
//! silent** via [`PolicyView::suppressed_count`].

use serde::{Deserialize, Serialize};

use crate::dataset::DatasetRef;

/// One analysis row presented to the policy filter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolicyAnalysis {
    /// Stable row id (application key).
    pub id: String,
    /// Dataset attribution of this analysis.
    pub dataset: DatasetRef,
    /// Opaque payload (tag strings, features, …).
    #[serde(default)]
    pub body: serde_json::Value,
}

impl PolicyAnalysis {
    /// Build a policy-filterable analysis row.
    pub fn new(id: impl Into<String>, dataset: DatasetRef, body: serde_json::Value) -> Self {
        Self { id: id.into(), dataset, body }
    }
}

/// Which datasets a consumer reads.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisPolicy {
    /// Datasets visible to the consumer.
    #[serde(default)]
    pub datasets: Vec<DatasetRef>,
    /// When `true`, suppressed rows are still returned (for auditing) but
    /// remain counted as suppressed.
    #[serde(default)]
    pub include_suppressed: bool,
}

impl AnalysisPolicy {
    /// Build a policy over the given datasets.
    pub fn new(datasets: Vec<DatasetRef>, include_suppressed: bool) -> Self {
        Self { datasets, include_suppressed }
    }
}

/// The filtered view: returned rows plus the never-silent suppression count.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolicyView {
    /// Rows visible under the policy (plus suppressed rows when
    /// `include_suppressed` is set).
    pub returned: Vec<PolicyAnalysis>,
    /// How many rows the policy suppressed. Always exact, even when
    /// `include_suppressed` returns them for auditing.
    pub suppressed_count: usize,
    /// The suppressed rows themselves.
    pub analyses_suppressed: Vec<PolicyAnalysis>,
}

/// Apply an [`AnalysisPolicy`] to a multi-analysis row set.
///
/// Rows whose dataset is listed in the policy are returned; all others are
/// suppressed and counted. No ranking, merging, or winner election occurs.
pub fn apply_policy(analyses: Vec<PolicyAnalysis>, policy: &AnalysisPolicy) -> PolicyView {
    let mut returned = Vec::new();
    let mut suppressed = Vec::new();
    for analysis in analyses {
        if policy.datasets.contains(&analysis.dataset) {
            returned.push(analysis);
        } else {
            suppressed.push(analysis);
        }
    }
    let suppressed_count = suppressed.len();
    if policy.include_suppressed {
        returned.extend(suppressed.clone());
    }
    PolicyView { returned, suppressed_count, analyses_suppressed: suppressed }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn row(id: &str, slug: &str) -> PolicyAnalysis {
        PolicyAnalysis::new(id, DatasetRef::new(slug, "0.1.0"), json!({"surface": "كَتَبَ"}))
    }

    #[test]
    fn suppression_is_counted_never_silent() {
        let policy = AnalysisPolicy::new(vec![DatasetRef::new("a", "0.1.0")], false);
        let view = apply_policy(vec![row("1", "a"), row("2", "b")], &policy);
        assert_eq!(view.returned.len(), 1);
        assert_eq!(view.suppressed_count, 1);
        assert_eq!(view.analyses_suppressed.len(), 1);
        assert_eq!(view.analyses_suppressed[0].id, "2");
    }

    #[test]
    fn include_suppressed_still_counts() {
        let policy = AnalysisPolicy::new(vec![DatasetRef::new("a", "0.1.0")], true);
        let view = apply_policy(vec![row("1", "a"), row("2", "b")], &policy);
        assert_eq!(view.returned.len(), 2);
        assert_eq!(view.suppressed_count, 1);
    }

    #[test]
    fn empty_policy_suppresses_everything_explicitly() {
        let policy = AnalysisPolicy::new(Vec::new(), false);
        let view = apply_policy(vec![row("1", "a")], &policy);
        assert!(view.returned.is_empty());
        assert_eq!(view.suppressed_count, 1);
    }
}
