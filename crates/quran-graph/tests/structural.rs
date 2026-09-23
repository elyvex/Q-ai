//! Structural golden test (TASK-404 slice): the two-surah mini corpus in
//! `fixtures/quran/graph/mini-structural.json` must build to an exact node
//! and edge set — including no `NEXT` across the surah boundary — with
//! input-version provenance on every edge.

use std::collections::HashSet;
use std::path::PathBuf;

use quran_graph::{StructuralInput, build_structural};

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/quran/graph/mini-structural.json")
}

#[test]
fn mini_corpus_golden_node_and_edge_sets() {
    let raw = std::fs::read_to_string(fixture_path()).unwrap();
    let input: StructuralInput = serde_json::from_str(&raw).unwrap();
    assert_eq!(input.input_version, "v1");
    let built = build_structural(&input);

    let nodes: HashSet<&str> = built.nodes.iter().map(|n| n.stable_id.as_str()).collect();
    let expected_nodes: HashSet<&str> = [
        "edition:test-min@v1",
        "surah:1",
        "surah:2",
        "ayah:1:1",
        "ayah:1:2",
        "ayah:2:1",
        "token:1:1:1",
        "token:1:1:2",
        "token:1:2:1",
        "token:2:1:1",
        "division:juz-1",
    ]
    .into_iter()
    .collect();
    assert_eq!(nodes, expected_nodes);

    let edges: HashSet<(&str, &str, &str)> =
        built.edges.iter().map(|e| (e.src.as_str(), e.edge.as_str(), e.dst.as_str())).collect();
    let expected_edges: HashSet<(&str, &str, &str)> = [
        ("edition:test-min@v1", "CONTAINS", "surah:1"),
        ("edition:test-min@v1", "CONTAINS", "surah:2"),
        ("edition:test-min@v1", "CONTAINS", "division:juz-1"),
        ("surah:1", "CONTAINS", "ayah:1:1"),
        ("surah:1", "CONTAINS", "ayah:1:2"),
        ("surah:2", "CONTAINS", "ayah:2:1"),
        ("ayah:1:1", "CONTAINS", "token:1:1:1"),
        ("ayah:1:1", "CONTAINS", "token:1:1:2"),
        ("ayah:1:2", "CONTAINS", "token:1:2:1"),
        ("ayah:2:1", "CONTAINS", "token:2:1:1"),
        ("ayah:1:1", "NEXT", "ayah:1:2"),
        ("token:1:1:1", "NEXT", "token:1:1:2"),
    ]
    .into_iter()
    .collect();
    assert_eq!(edges, expected_edges);

    // No NEXT crosses the surah boundary: ayah:1:2 has no NEXT successor and
    // ayah:2:1 has no NEXT predecessor.
    assert!(!edges.contains(&("ayah:1:2", "NEXT", "ayah:2:1")));
    assert!(edges.iter().all(|(s, e, d)| {
        *e != "NEXT"
            || !(s.starts_with("ayah:1:") && d.starts_with("ayah:2:")
                || s.starts_with("token:1:") && d.starts_with("token:2:"))
    }));

    // Input-version provenance on every edge; structural edges need no
    // assertion pointer.
    for edge in &built.edges {
        assert_eq!(edge.attrs["input_version"], "v1");
        assert_eq!(edge.attrs["edition_id"], "test-min");
        assert!(edge.assertion_id.is_none());
    }

    // Deterministic across runs.
    assert_eq!(built, build_structural(&input));
}
