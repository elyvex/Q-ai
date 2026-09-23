//! Typed pattern queries (TASK-420): allowlisted predicates, typed steps,
//! pattern-size budget. Unsupported operations are rejected; raw query text
//! is never accepted or forwarded.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::AtomicBool;

use serde::{Deserialize, Serialize};

use crate::error::GraphError;
use crate::model::{GraphNode, NodeKind, QueryBudgets, TraversalResult, is_allowed_edge};
use crate::store::{AuthzScope, EdgeFilter, GraphStore, cancel_requested};

/// Maximum steps per pattern (query-safety budget).
pub const PATTERN_SIZE_BUDGET: usize = 8;

/// One typed traversal step: follow `edge`, then keep only nodes of
/// `node_kind` when set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatternStep {
    /// Edge predicate; must be in the edge vocabulary.
    pub edge: String,
    /// Optional node-kind constraint applied after the hop.
    #[serde(default)]
    pub node_kind: Option<NodeKind>,
}

impl PatternStep {
    /// Build a step with an edge predicate and no kind constraint.
    pub fn edge(edge: impl Into<String>) -> Self {
        Self { edge: edge.into(), node_kind: None }
    }

    /// Build a step with an edge predicate plus a node-kind constraint.
    pub fn edge_to(edge: impl Into<String>, kind: NodeKind) -> Self {
        Self { edge: edge.into(), node_kind: Some(kind) }
    }
}

/// A typed pattern: an ordered list of [`PatternStep`]s matched left to
/// right starting from seed nodes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pattern {
    /// Ordered steps, `1..=PATTERN_SIZE_BUDGET` after validation.
    #[serde(default)]
    pub steps: Vec<PatternStep>,
}

impl Pattern {
    /// Build a pattern from steps (validation happens on execution).
    pub fn new(steps: Vec<PatternStep>) -> Self {
        Self { steps }
    }
}

/// Whether a string resembles raw query text rather than a vocabulary term.
/// Vocabulary terms are `A-Z` plus underscore; anything with whitespace,
/// quoting, punctuation, or operator characters is rejected.
fn looks_like_raw_text(s: &str) -> bool {
    if s.is_empty() {
        return true;
    }
    s.chars().any(|c| {
        c.is_whitespace()
            || matches!(
                c,
                ';' | '\''
                    | '"'
                    | '`'
                    | '('
                    | ')'
                    | '*'
                    | '?'
                    | '$'
                    | '/'
                    | '\\'
                    | '|'
                    | '&'
                    | '!'
                    | '<'
                    | '>'
                    | '='
                    | '+'
                    | '{'
                    | '}'
                    | '['
                    | ']'
                    | ','
                    | ':'
                    | '.'
                    | '#'
                    | '@'
            )
    })
}

/// Validate a pattern: non-empty, within [`PATTERN_SIZE_BUDGET`], every edge
/// allowlisted, and nothing resembling raw query text.
pub fn validate_pattern(pattern: &Pattern) -> Result<(), GraphError> {
    if pattern.steps.is_empty() {
        return Err(GraphError::PatternRejected {
            detail: format!("pattern must contain 1..={PATTERN_SIZE_BUDGET} steps, got 0"),
        });
    }
    if pattern.steps.len() > PATTERN_SIZE_BUDGET {
        return Err(GraphError::PatternRejected {
            detail: format!(
                "pattern has {} steps, budget allows at most {PATTERN_SIZE_BUDGET}",
                pattern.steps.len()
            ),
        });
    }
    for (i, step) in pattern.steps.iter().enumerate() {
        if looks_like_raw_text(&step.edge) {
            return Err(GraphError::PatternRejected {
                detail: format!("step {i}: edge name carries raw query text"),
            });
        }
        if !is_allowed_edge(&step.edge) {
            return Err(GraphError::PatternRejected {
                detail: format!("step {i}: unknown edge '{}'", step.edge),
            });
        }
    }
    Ok(())
}

/// Execute a validated pattern against any [`GraphStore`]: stepwise frontier
/// expansion over [`GraphStore::neighbors`], never against a concrete backend.
///
/// Semantics: the frontier starts at `seeds`; each step expands every
/// frontier node along `step.edge` (both directions) and keeps arrivals
/// matching `step.node_kind` when set. Deterministic (frontiers sort by
/// stable ID), cycle-safe (arrivals never revisit a visited node), and
/// budget-explicit (over-long patterns or exhausted node/edge caps return a
/// truncated result, never a silent empty one).
pub fn execute_pattern<S: GraphStore>(
    store: &S,
    pattern: &Pattern,
    seeds: &[String],
    budgets: &QueryBudgets,
    cancel: &AtomicBool,
    authz: &AuthzScope,
) -> Result<TraversalResult, GraphError> {
    validate_pattern(pattern)?;
    budgets.check()?;
    if pattern.steps.len() > budgets.max_hops {
        return Ok(TraversalResult::incomplete(
            Vec::new(),
            Vec::new(),
            format!(
                "pattern needs {} hops but budgets allow {}",
                pattern.steps.len(),
                budgets.max_hops
            ),
            0,
            0,
        ));
    }
    if cancel_requested(cancel) {
        return Ok(TraversalResult::incomplete(
            Vec::new(),
            Vec::new(),
            "cancelled before pattern execution",
            0,
            0,
        ));
    }

    let mut visited: BTreeSet<String> = BTreeSet::new();
    let mut node_records: BTreeMap<String, GraphNode> = BTreeMap::new();
    let mut seen_edges: BTreeSet<(String, String, String, String)> = BTreeSet::new();
    let mut collected_edges = Vec::new();
    let mut expanded_nodes: usize = 0;
    let mut expanded_edges: usize = 0;

    let mut frontier: Vec<String> = seeds.to_vec();
    frontier.sort();
    frontier.dedup();

    for step in pattern.steps.iter() {
        if cancel_requested(cancel) {
            return Ok(TraversalResult::incomplete(
                node_records.into_values().collect(),
                collected_edges,
                "cancelled during pattern execution",
                expanded_nodes,
                expanded_edges,
            ));
        }
        let filter = EdgeFilter {
            edge_types: Some(vec![step.edge.clone()]),
            direction: crate::store::Direction::Both,
        };
        let mut next_frontier: Vec<String> = Vec::new();
        for node_id in &frontier {
            if cancel_requested(cancel) {
                break;
            }
            let nbrs = store.neighbors(node_id, &filter, budgets, cancel, authz)?;
            expanded_nodes += 1;
            expanded_edges += nbrs.expanded_edges;
            if nbrs.truncated {
                return Ok(TraversalResult::incomplete(
                    node_records.into_values().collect(),
                    collected_edges,
                    format!(
                        "neighbor budget exhausted at pattern step: {}",
                        nbrs.incomplete_reason.unwrap_or_default()
                    ),
                    expanded_nodes,
                    expanded_edges,
                ));
            }
            for node in nbrs.nodes {
                if let Some(kind) = step.node_kind
                    && node.kind != kind
                {
                    continue;
                }
                node_records.entry(node.stable_id.clone()).or_insert_with(|| node.clone());
                if visited.contains(&node.stable_id) {
                    continue;
                }
                next_frontier.push(node.stable_id.clone());
            }
            for edge in nbrs.edges {
                let key = (
                    edge.src.clone(),
                    edge.edge.clone(),
                    edge.dst.clone(),
                    edge.assertion_id.clone().unwrap_or_default(),
                );
                if seen_edges.insert(key) {
                    collected_edges.push(edge);
                }
            }
            if node_records.len() > budgets.max_nodes || collected_edges.len() > budgets.max_edges {
                return Ok(TraversalResult::incomplete(
                    node_records.into_values().collect(),
                    collected_edges,
                    "pattern result exceeded node/edge budgets",
                    expanded_nodes,
                    expanded_edges,
                ));
            }
        }
        for id in &next_frontier {
            visited.insert(id.clone());
        }
        next_frontier.sort();
        next_frontier.dedup();
        if next_frontier.is_empty() {
            break;
        }
        frontier = next_frontier;
    }

    // Pin seed records for stable output even when a step matches nothing.
    for seed in seeds {
        if !node_records.contains_key(seed)
            && let Ok(Some(node)) = store.resolve_node(seed, budgets, cancel, authz)
        {
            node_records.insert(seed.clone(), node);
        }
    }

    let mut nodes: Vec<GraphNode> = node_records.into_values().collect();
    nodes.sort_by(|a, b| a.stable_id.cmp(&b.stable_id));
    collected_edges.sort_by(|a, b| (&a.src, &a.edge, &a.dst).cmp(&(&b.src, &b.edge, &b.dst)));
    Ok(TraversalResult {
        nodes,
        edges: collected_edges,
        truncated: false,
        incomplete_reason: None,
        expanded_nodes,
        expanded_edges,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_and_oversized() {
        assert!(validate_pattern(&Pattern::new(vec![])).is_err());
        let big = Pattern::new(vec![PatternStep::edge("NEXT"); PATTERN_SIZE_BUDGET + 1]);
        let err = validate_pattern(&big).unwrap_err();
        assert!(matches!(err, GraphError::PatternRejected { .. }));
    }

    #[test]
    fn rejects_unknown_edges_and_raw_text() {
        let unknown = Pattern::new(vec![PatternStep::edge("PRECEDES")]);
        assert!(matches!(validate_pattern(&unknown), Err(GraphError::PatternRejected { .. })));
        for raw in ["HAS ROOT", "NEXT;", "'NEXT'", "NEXT(x)", "a|b", "A.B"] {
            let p = Pattern::new(vec![PatternStep::edge(raw)]);
            assert!(validate_pattern(&p).is_err(), "should reject {raw:?}");
        }
        let ok = Pattern::new(vec![
            PatternStep::edge_to("MENTIONS_CONCEPT", NodeKind::Concept),
            PatternStep::edge("NEXT"),
        ]);
        assert!(validate_pattern(&ok).is_ok());
    }
}
