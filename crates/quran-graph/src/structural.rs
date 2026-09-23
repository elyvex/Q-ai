//! Pure structural projection builder (TASK-404).
//!
//! Input: edition/surah/ayah/token/division records. Output: a deterministic
//! [`BuiltProjection`] with `CONTAINS` hierarchy edges and `NEXT` sequencing
//! edges.
//!
//! Purity contract: this is a pure function over its input. It never touches
//! canonical stores, caches, or I/O. Canonical lookup therefore keeps working
//! when the projection is missing or stale: quotation resolution reads the
//! canonical store, never the graph.
//!
//! Text policy: `AyahInput.text` and `TokenInput.surface` are accepted for
//! caller-side alignment but are never embedded in node attributes. Display
//! text (including diacritics and orthography) stays in the canonical store;
//! graph payloads carry IDs, structural numbers, and provenance only.
//!
//! Surah-boundary rule: `NEXT` never crosses a surah boundary. In practice
//! the builder is stricter and fully explicit:
//!
//! - ayah `NEXT` links position-sorted neighbors *within one surah*;
//! - token `NEXT` links position-sorted neighbors *within one ayah* (hence
//!   never across ayahs, and never across surahs).
//!
//! Stable IDs are deterministic:
//!
//! - edition: `edition:<slug>@<input-version>`
//! - surah: `surah:<n>`
//! - ayah: `ayah:<s>:<a>`
//! - token: `token:<s>:<a>:<p>`
//! - division: `division:<id>`
//!
//! Implied parents: surah nodes are derived from the union of listed surahs
//! and surahs referenced by ayahs/tokens; ayah nodes from the union of listed
//! ayahs and ayahs referenced by tokens. The builder therefore never emits a
//! dangling edge.
//!
//! Division membership (division-to-ayah links) is out of scope for this
//! slice: divisions hang off the edition via `CONTAINS` only.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::model::{GraphEdge, GraphNode, NodeKind};

/// Builder name stamped into every structural edge's attributes.
pub const STRUCTURAL_BUILDER_VERSION: &str = "structural-v1";

/// Projection family ID for the structural graph.
pub const STRUCTURAL_PROJECTION_ID: &str = "quran-structural-v1";

/// One surah number in the input corpus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurahInput {
    /// Surah number (1-based).
    pub number: u32,
}

/// One ayah record. `text` is alignment-only; see the module docs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AyahInput {
    /// Surah number (1-based).
    pub surah: u32,
    /// Ayah number within the surah (1-based).
    pub ayah: u32,
    /// Canonical text, accepted for alignment, never embedded.
    #[serde(default)]
    pub text: String,
}

/// One token record. `surface` is alignment-only; see the module docs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenInput {
    /// Surah number (1-based).
    pub surah: u32,
    /// Ayah number within the surah (1-based).
    pub ayah: u32,
    /// Token position within the ayah (1-based).
    pub position: u32,
    /// Surface form, accepted for alignment, never embedded.
    #[serde(default)]
    pub surface: String,
}

/// One division record (juz, hizb, ruku, page, …).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DivisionInput {
    /// Division identifier, e.g. `juz-01`.
    pub id: String,
    /// Division kind, e.g. `juz`.
    pub kind: String,
}

/// Structural builder input: one edition snapshot plus its hierarchy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuralInput {
    /// Edition slug, e.g. `madani`.
    pub edition_id: String,
    /// Input corpus version; stamped on the edition ID and every edge.
    pub input_version: String,
    /// Listed surahs (implied surahs are added automatically).
    #[serde(default)]
    pub surahs: Vec<SurahInput>,
    /// Ayah records.
    #[serde(default)]
    pub ayahs: Vec<AyahInput>,
    /// Token records.
    #[serde(default)]
    pub tokens: Vec<TokenInput>,
    /// Division records.
    #[serde(default)]
    pub divisions: Vec<DivisionInput>,
}

/// Built structural projection: deterministic node and edge sets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuiltProjection {
    /// Nodes sorted by stable ID.
    pub nodes: Vec<GraphNode>,
    /// Edges sorted by `(src, edge, dst)`.
    pub edges: Vec<GraphEdge>,
}

/// Edition stable ID: `edition:<slug>@<input-version>`.
pub fn edition_stable_id(edition_id: &str, input_version: &str) -> String {
    format!("edition:{edition_id}@{input_version}")
}

/// Surah stable ID: `surah:<n>`.
pub fn surah_stable_id(number: u32) -> String {
    format!("surah:{number}")
}

/// Ayah stable ID: `ayah:<s>:<a>`.
pub fn ayah_stable_id(surah: u32, ayah: u32) -> String {
    format!("ayah:{surah}:{ayah}")
}

/// Token stable ID: `token:<s>:<a>:<p>`.
pub fn token_stable_id(surah: u32, ayah: u32, position: u32) -> String {
    format!("token:{surah}:{ayah}:{position}")
}

/// Division stable ID: `division:<id>`.
pub fn division_stable_id(id: &str) -> String {
    format!("division:{id}")
}

/// Build the structural projection for one edition snapshot.
///
/// Pure and total: any input (including empty) yields a deterministic,
/// dangling-edge-free projection.
pub fn build_structural(input: &StructuralInput) -> BuiltProjection {
    let edition_id = edition_stable_id(&input.edition_id, &input.input_version);

    let mut surahs: BTreeSet<u32> = input.surahs.iter().map(|s| s.number).collect();
    for a in &input.ayahs {
        surahs.insert(a.surah);
    }
    for t in &input.tokens {
        surahs.insert(t.surah);
    }
    let mut ayah_keys: BTreeSet<(u32, u32)> =
        input.ayahs.iter().map(|a| (a.surah, a.ayah)).collect();
    for t in &input.tokens {
        ayah_keys.insert((t.surah, t.ayah));
    }
    let token_keys: BTreeSet<(u32, u32, u32)> =
        input.tokens.iter().map(|t| (t.surah, t.ayah, t.position)).collect();
    let mut divisions: BTreeMap<&str, &str> = BTreeMap::new();
    for d in &input.divisions {
        divisions.insert(d.id.as_str(), d.kind.as_str());
    }

    let mut nodes: BTreeMap<String, GraphNode> = BTreeMap::new();
    let mut edges: BTreeMap<(String, String, String), GraphEdge> = BTreeMap::new();

    let provenance = serde_json::json!({
        "builder": STRUCTURAL_BUILDER_VERSION,
        "edition_id": input.edition_id,
        "input_version": input.input_version,
    });
    let mut link = |src: &str, edge: &str, dst: &str| {
        edges.insert(
            (src.to_string(), edge.to_string(), dst.to_string()),
            GraphEdge::structural(src, edge, dst, provenance.clone()),
        );
    };

    nodes.insert(
        edition_id.clone(),
        GraphNode::new(
            edition_id.clone(),
            NodeKind::Edition,
            serde_json::json!({
                "slug": input.edition_id,
                "input_version": input.input_version,
            }),
        ),
    );
    for n in &surahs {
        let id = surah_stable_id(*n);
        nodes.insert(
            id.clone(),
            GraphNode::new(id.clone(), NodeKind::Surah, serde_json::json!({"number": n})),
        );
        link(&edition_id, "CONTAINS", &id);
    }
    for (id, kind) in &divisions {
        let node_id = division_stable_id(id);
        nodes.insert(
            node_id.clone(),
            GraphNode::new(
                node_id.clone(),
                NodeKind::Division,
                serde_json::json!({"division_id": id, "kind": kind}),
            ),
        );
        link(&edition_id, "CONTAINS", &node_id);
    }
    for (s, a) in &ayah_keys {
        let id = ayah_stable_id(*s, *a);
        nodes.insert(
            id.clone(),
            GraphNode::new(id.clone(), NodeKind::Ayah, serde_json::json!({"surah": s, "ayah": a})),
        );
        link(&surah_stable_id(*s), "CONTAINS", &id);
    }
    for (s, a, p) in &token_keys {
        let id = token_stable_id(*s, *a, *p);
        nodes.insert(
            id.clone(),
            GraphNode::new(
                id.clone(),
                NodeKind::Token,
                serde_json::json!({"surah": s, "ayah": a, "position": p}),
            ),
        );
        link(&ayah_stable_id(*s, *a), "CONTAINS", &id);
    }

    // Ayah NEXT: consecutive ayahs within one surah only. No NEXT crosses a
    // surah boundary: grouping by surah makes that structural.
    let mut ayahs_by_surah: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    for (s, a) in &ayah_keys {
        ayahs_by_surah.entry(*s).or_default().push(*a);
    }
    for (s, list) in &ayahs_by_surah {
        let mut sorted = list.clone();
        sorted.sort_unstable();
        sorted.dedup();
        for pair in sorted.windows(2) {
            link(&ayah_stable_id(*s, pair[0]), "NEXT", &ayah_stable_id(*s, pair[1]));
        }
    }

    // Token NEXT: consecutive positions within one ayah only, hence never
    // across ayahs and never across surahs.
    let mut tokens_by_ayah: BTreeMap<(u32, u32), Vec<u32>> = BTreeMap::new();
    for (s, a, p) in &token_keys {
        tokens_by_ayah.entry((*s, *a)).or_default().push(*p);
    }
    for ((s, a), list) in &tokens_by_ayah {
        let mut sorted = list.clone();
        sorted.sort_unstable();
        sorted.dedup();
        for pair in sorted.windows(2) {
            link(&token_stable_id(*s, *a, pair[0]), "NEXT", &token_stable_id(*s, *a, pair[1]));
        }
    }

    BuiltProjection { nodes: nodes.into_values().collect(), edges: edges.into_values().collect() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mini() -> StructuralInput {
        StructuralInput {
            edition_id: "test-min".into(),
            input_version: "v1".into(),
            surahs: vec![SurahInput { number: 1 }, SurahInput { number: 2 }],
            ayahs: vec![
                AyahInput { surah: 1, ayah: 1, text: "a".into() },
                AyahInput { surah: 1, ayah: 2, text: "b".into() },
                AyahInput { surah: 2, ayah: 1, text: "c".into() },
            ],
            tokens: vec![
                TokenInput { surah: 1, ayah: 1, position: 1, surface: "w1".into() },
                TokenInput { surah: 1, ayah: 1, position: 2, surface: "w2".into() },
                TokenInput { surah: 1, ayah: 2, position: 1, surface: "w3".into() },
                TokenInput { surah: 2, ayah: 1, position: 1, surface: "w4".into() },
            ],
            divisions: vec![DivisionInput { id: "juz-1".into(), kind: "juz".into() }],
        }
    }

    #[test]
    fn golden_counts_and_no_cross_surah_next() {
        let built = build_structural(&mini());
        // 1 edition + 2 surah + 3 ayah + 4 token + 1 division = 11 nodes.
        assert_eq!(built.nodes.len(), 11);
        // CONTAINS: ed->surah x2, ed->div x1, surah->ayah x3, ayah->token x4 = 10;
        // NEXT: ayah 1 + token 1 = 2. Total 12.
        assert_eq!(built.edges.len(), 12);
        for e in &built.edges {
            assert!(e.assertion_id.is_none());
            assert_eq!(e.attrs["input_version"], "v1");
            if e.edge == "NEXT" {
                let surah_of = |id: &str| -> String {
                    let parts: Vec<&str> = id.split(':').collect();
                    match parts[0] {
                        "ayah" => format!("surah:{}", parts[1]),
                        "token" => format!("surah:{}", parts[1]),
                        other => panic!("unexpected NEXT endpoint {other}"),
                    }
                };
                assert_eq!(surah_of(&e.src), surah_of(&e.dst));
            }
        }
        // Deterministic.
        assert_eq!(built, build_structural(&mini()));
    }

    #[test]
    fn no_canonical_text_embedded() {
        let built = build_structural(&mini());
        let blob = serde_json::to_string(&built).unwrap();
        assert!(!blob.contains("\"surface\""));
        assert!(!blob.contains("\"text\""));
    }
}
