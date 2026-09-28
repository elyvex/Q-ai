//! Graph JSON export with policy filter and static rendering (Phase 4,
//! D-02/D-13).
//!
//! # application::quran_graph_export
//!
//! Export assembles through [`quran_graph::export_json_with_notice`] after
//! [`quran_graph::retain_visible`] plus
//! [`quran_graph::assertion_allowlist_predicate`]: only effective assertions
//! for kept edges ship, via the shared
//! [`crate::quran_graph_annotations::visible_export_sets`] composition —
//! the restricted and tombstoned IDs asserted absent from the serialized
//! bytes (T-04-10). Truncated exports carry the incomplete notice with a
//! non-empty reason, never a silent complete flag.
//!
//! Static rendering is plain-text emission with zero new dependencies (the
//! `dot` binary is absent): [`render_dot`] builds a DOT document and
//! [`render_svg`] a minimal layered SVG laid out by hop distance from the
//! seed. Both carry a truncation banner when partial. SVG coordinates are
//! presentational only. Labels are stable IDs (refs-only: no canonical text
//! ever enters graph records); the document root declares `direction="rtl"`
//! so Arabic-script labels from later phases lay out correctly.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use quran_graph::{
    Assertion, AuthzScope, ExportNotice, GraphEdge, GraphError, GraphNode, ProjectionManifest,
    export::export_json_with_notice,
};
use storage_sqlite::SqliteDatabase;

use crate::quran_graph_annotations::visible_export_sets;
use crate::quran_graph_store::SqliteGraphStore;

/// Loaded projection sets: everything one export serializes.
#[derive(Debug, Clone)]
pub struct ProjectionSets {
    /// Build identity, versions, and generation stamp.
    pub manifest: ProjectionManifest,
    /// Pinned nodes (stable-ID order).
    pub nodes: Vec<GraphNode>,
    /// Pinned edges (`(src, edge, dst, assertion_id)` order).
    pub edges: Vec<GraphEdge>,
    /// Authority records for the build row (all decisions).
    pub assertions: Vec<Assertion>,
}

/// Load the full pinned sets of one projection build row (read-only).
///
/// # Errors
///
/// Returns [`GraphError::UnknownProjection`] for an unknown row ID, or
/// [`GraphError::BuildFailed`] when the rows are corrupt.
pub async fn load_projection_sets(
    db: &SqliteDatabase,
    projection_row_id: &str,
) -> Result<ProjectionSets, GraphError> {
    let store = SqliteGraphStore::open(db, projection_row_id).await?;
    let manifest = store.manifest().clone();
    let (nodes, edges, assertions) = store.export_sets();
    Ok(ProjectionSets { manifest, nodes, edges, assertions })
}

/// Assemble a `quran-graph-json-v1` document: retain edges through the
/// allowlist (structural plus effective assertions inside `authz`), then
/// serialize with the notice. Only assertions referenced by kept edges
/// travel, so restricted and tombstoned IDs are absent from the bytes.
pub fn assemble_export(
    sets: &ProjectionSets,
    authz: &AuthzScope,
    notice: &ExportNotice,
) -> serde_json::Value {
    let visible: Vec<Assertion> = sets
        .assertions
        .iter()
        .filter(|record| record.is_effective() && authz.edge_visible(Some(&record.id)))
        .cloned()
        .collect();
    let (nodes, edges, traveling) = visible_export_sets(&sets.nodes, &sets.edges, &visible);
    export_json_with_notice(&nodes, &edges, &traveling, &sets.manifest, notice)
}

/// Load one projection build row and assemble its export document in one
/// step (the CLI export verb path).
///
/// # Errors
///
/// Same as [`load_projection_sets`].
pub async fn export_document(
    db: &SqliteDatabase,
    projection_row_id: &str,
    authz: &AuthzScope,
    notice: &ExportNotice,
) -> Result<serde_json::Value, GraphError> {
    let sets = load_projection_sets(db, projection_row_id).await?;
    Ok(assemble_export(&sets, authz, notice))
}

/// Hop distances from `seed` over the given edges (both directions).
/// Deterministic: neighbors sort by stable ID within each layer. Nodes
/// unreachable from the seed (or every node when the seed is unknown) form
/// the final layer.
pub fn hop_layers(nodes: &[GraphNode], edges: &[GraphEdge], seed: &str) -> Vec<Vec<String>> {
    let known: BTreeSet<&str> = nodes.iter().map(|node| node.stable_id.as_str()).collect();
    let mut adjacent: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for edge in edges {
        if known.contains(edge.src.as_str()) && known.contains(edge.dst.as_str()) {
            adjacent.entry(edge.src.as_str()).or_default().insert(edge.dst.as_str());
            adjacent.entry(edge.dst.as_str()).or_default().insert(edge.src.as_str());
        }
    }
    if !known.contains(seed) {
        let mut all: Vec<String> = known.into_iter().map(str::to_string).collect();
        all.sort();
        return vec![all];
    }
    let mut visited: BTreeSet<String> = BTreeSet::from([seed.to_string()]);
    let mut layers = vec![vec![seed.to_string()]];
    let mut frontier: VecDeque<String> = VecDeque::from([seed.to_string()]);
    while !frontier.is_empty() {
        let mut next: BTreeSet<String> = BTreeSet::new();
        for current in std::mem::take(&mut frontier) {
            if let Some(peers) = adjacent.get(current.as_str()) {
                for peer in peers {
                    if visited.insert((*peer).to_string()) {
                        next.insert((*peer).to_string());
                    }
                }
            }
        }
        if next.is_empty() {
            break;
        }
        frontier = next.into_iter().collect();
        layers.push(frontier.iter().cloned().collect());
    }
    let mut rest: Vec<String> =
        known.into_iter().filter(|id| !visited.contains(*id)).map(str::to_string).collect();
    if !rest.is_empty() {
        rest.sort();
        layers.push(rest);
    }
    layers
}

fn escape_dot(raw: &str) -> String {
    raw.replace('\\', "\\\\").replace('"', "\\\"")
}

fn escape_xml(raw: &str) -> String {
    raw.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Render the node/edge sets as a DOT document (plain-text string
/// building, no `dot` binary needed). Node labels carry stable ID plus
/// kind; asserted edges name their assertion ID. A truncation banner (a
/// comment plus a plaintext node) renders whenever the notice is partial.
pub fn render_dot(nodes: &[GraphNode], edges: &[GraphEdge], notice: &ExportNotice) -> String {
    let mut out = format!(
        "// Graph DOT export: {} node(s), {} edge(s) (quran-graph-json-v1).\n",
        nodes.len(),
        edges.len()
    );
    if notice.truncated {
        out.push_str(&format!(
            "// truncated: {}\n",
            notice.incomplete_reason.as_deref().unwrap_or("unknown reason")
        ));
    }
    out.push_str("digraph quran_graph {\n");
    let mut sorted: Vec<&GraphNode> = nodes.iter().collect();
    sorted.sort_by(|a, b| a.stable_id.cmp(&b.stable_id));
    for node in sorted {
        out.push_str(&format!(
            "  \"{}\" [label=\"{}\\n{}\"];\n",
            escape_dot(&node.stable_id),
            escape_dot(&node.stable_id),
            escape_dot(&node.kind.to_string())
        ));
    }
    let mut ordered: Vec<&GraphEdge> = edges.iter().collect();
    ordered.sort_by(|a, b| {
        (&a.src, &a.edge, &a.dst, &a.assertion_id).cmp(&(&b.src, &b.edge, &b.dst, &b.assertion_id))
    });
    for edge in ordered {
        let label = match &edge.assertion_id {
            Some(id) => format!("{}\\n{id}", edge.edge),
            None => edge.edge.clone(),
        };
        out.push_str(&format!(
            "  \"{}\" -> \"{}\" [label=\"{}\"];\n",
            escape_dot(&edge.src),
            escape_dot(&edge.dst),
            escape_dot(&label)
        ));
    }
    if notice.truncated {
        out.push_str(&format!(
            "  \"__truncated\" [shape=plaintext, label=\"truncated: {}\"];\n",
            escape_dot(notice.incomplete_reason.as_deref().unwrap_or("unknown reason"))
        ));
    }
    out.push_str("}\n");
    out
}

/// Render the node/edge sets as a minimal layered SVG: one row per hop
/// distance from `seed`, nodes spread across the row, edges as lines with
/// predicate labels. A truncation banner renders whenever the notice is
/// partial. Coordinates are presentational only and must never be asserted.
pub fn render_svg(
    nodes: &[GraphNode],
    edges: &[GraphEdge],
    seed: &str,
    notice: &ExportNotice,
) -> String {
    let layers = hop_layers(nodes, edges, seed);
    let width = layers.iter().map(Vec::len).max().unwrap_or(1).max(1) * 160 + 80;
    let row_height = 110;
    let mut height = layers.len() * row_height + 90;
    if notice.truncated {
        height += 40;
    }
    let mut positions: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    for (depth, layer) in layers.iter().enumerate() {
        for (index, id) in layer.iter().enumerate() {
            let x = 80
                + index * 160
                + (layers.iter().map(Vec::len).max().unwrap_or(1) - layer.len()) * 80;
            positions.insert(id.as_str(), (x, 70 + depth * row_height));
        }
    }
    let mut out = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" \
         direction=\"rtl\" role=\"img\">\n<desc>quran-graph-json-v1 neighborhood of {seed}</desc>\n",
        seed = escape_xml(seed)
    );
    for edge in edges {
        let (Some((x1, y1)), Some((x2, y2))) =
            (positions.get(edge.src.as_str()), positions.get(edge.dst.as_str()))
        else {
            continue;
        };
        out.push_str(&format!(
            "<line x1=\"{x1}\" y1=\"{y1}\" x2=\"{x2}\" y2=\"{y2}\" stroke=\"#888\"/>\n<text \
             x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-size=\"10\" \
             fill=\"#555\">{}</text>\n",
            (x1 + x2) / 2,
            (y1 + y2) / 2 - 4,
            escape_xml(&edge.edge)
        ));
    }
    for node in nodes {
        let Some((x, y)) = positions.get(node.stable_id.as_str()).copied() else {
            continue;
        };
        out.push_str(&format!(
            "<g><circle cx=\"{x}\" cy=\"{y}\" r=\"18\" fill=\"#eef\" stroke=\"#448\"/>\n<text x=\"{x}\" \
             y=\"{}\" text-anchor=\"middle\" font-size=\"10\">{}</text></g>\n",
            y + 34,
            escape_xml(&node.stable_id)
        ));
    }
    if notice.truncated {
        out.push_str(&format!(
            "<text x=\"{}\" y=\"{height}\" text-anchor=\"middle\" font-size=\"13\" \
             font-weight=\"bold\" fill=\"#a00\">truncated: {}</text>\n",
            width / 2,
            escape_xml(notice.incomplete_reason.as_deref().unwrap_or("unknown reason"))
        ));
    }
    out.push_str("</svg>\n");
    out
}
