//! Read-only Quran graph doctor checks (Phase 4, D-08).
//!
//! [`run_quran_graph_checks`] verifies every projection manifest read-only:
//! the generation stamp versus the active corpus generation, dataset versions
//! versus the active morphology datasets, and dependency-snapshot
//! completeness; it scans for dangling edges and probes tombstone
//! invisibility (rejected/superseded assertion ids appear in authority but in
//! neither traversal results nor export bytes).
//!
//! Read-only by construction: the caller opens the database through
//! [`storage_sqlite::SqliteDatabase::open_read_only`], canonical state is read
//! through one [`storage::Database::write`] unit of work that is always rolled
//! back, graph state is read through `SELECT`-only queries on the read pool,
//! and no write method is invoked anywhere on this path. Repair lives outside
//! this module path: explicit confirmed commands (never inside doctor).
//!
//! Severity policy (mirrors `quran_doctor_indexes`): a fresh database with
//! nothing built reports `Skipped` with a build remedy, never `Fail` and
//! never `Pass`; missing optional projections (word-root, annotated) are
//! `Skipped`; stale-but-servable drift is `Warn`; corruption, dangling
//! endpoints, and tombstone leaks are `Fail`.
//!
//! Scan depth: `deep = false` bounds the dangling-edge scan
//! ([`SHALLOW_EDGE_SCAN_CAP`] edges per active projection) and the tombstone
//! traversal probe ([`SHALLOW_TOMBSTONE_SRC_CAP`] source nodes). Hitting a
//! scan bound reports `Warn` naming `--deep` — a distinct bounded status,
//! never a pass and never a fail. `deep = true` scans everything.
//!
//! Check output orders by stable check id so runs diff cleanly.

use std::collections::BTreeMap;

use storage::Database as _;
use storage::error::StorageError;
use storage_sqlite::SqliteDatabase;

use super::quran_doctor::{CheckLevel, QuranDoctorCheck};
use super::quran_graph_build::{
    ANNOTATED_BUILDER_VERSION, ANNOTATED_PROJECTION_ID, CONCEPT_SEED_SCOPE,
    WORDROOT_BUILDER_VERSION, WORDROOT_PROJECTION_ID,
};

/// Stable graph doctor check ids, in output order.
pub const GRAPH_CHECK_IDS: [&str; 6] = [
    "quran.graph.annotated.current",
    "quran.graph.dangling_edges",
    "quran.graph.dependency_snapshot",
    "quran.graph.structural.current",
    "quran.graph.tombstone_invisibility",
    "quran.graph.wordroot.current",
];

/// Shallow dangling-edge scan cap per active projection (deep scans all).
pub const SHALLOW_EDGE_SCAN_CAP: usize = 5000;
/// Shallow tombstone-probe cap on distinct edge sources (deep probes all).
pub const SHALLOW_TOMBSTONE_SRC_CAP: usize = 50;

fn pass(id: &'static str, summary: String) -> QuranDoctorCheck {
    QuranDoctorCheck { id, status: CheckLevel::Pass, summary, remedy: None, next_command: None }
}

fn fail(id: &'static str, summary: String, remedy: &str, next: &str) -> QuranDoctorCheck {
    QuranDoctorCheck {
        id,
        status: CheckLevel::Fail,
        summary,
        remedy: Some(remedy.to_string()),
        next_command: Some(next.to_string()),
    }
}

fn warn(id: &'static str, summary: String, remedy: &str, next: &str) -> QuranDoctorCheck {
    QuranDoctorCheck {
        id,
        status: CheckLevel::Warn,
        summary,
        remedy: Some(remedy.to_string()),
        next_command: Some(next.to_string()),
    }
}

fn skipped(id: &'static str, summary: String, remedy: &str, next: &str) -> QuranDoctorCheck {
    QuranDoctorCheck {
        id,
        status: CheckLevel::Skipped,
        summary,
        remedy: Some(remedy.to_string()),
        next_command: Some(next.to_string()),
    }
}

fn storage_unavailable(error: sqlx::Error) -> StorageError {
    let _ = error;
    StorageError::StorageUnavailable
}

/// Canonical state read through one rolled-back unit of work.
struct CanonicalState {
    has_edition: bool,
    edition_id: String,
    corpus_generation: i64,
    active_dataset: Option<(String, String)>,
}

/// One active projection build row plus its scanned adjacency facts.
struct ProjectionState {
    row_id: String,
    projection_id: String,
    builder_version: String,
    edition_id: String,
    corpus_generation: i64,
    dataset_versions: BTreeMap<String, String>,
    dependency_snapshot: BTreeMap<String, String>,
    manifest_parse_ok: bool,
    node_count: i64,
    edge_count: i64,
    /// Dangling (src, edge, dst) triples, capped at `cap + 1` for bound
    /// detection. Each edge counts once regardless of assertion count: the
    /// scan keys on edge rows, never on assertion rows.
    dangling: Vec<(String, String, String)>,
    /// Whether the dangling scan hit its bound (shallow only).
    dangling_bounded: bool,
    /// Rejected/superseded assertion ids retained in authority.
    tombstoned_ids: Vec<String>,
}

/// One active `graph_projections` row: identity plus raw manifest payloads.
type ProjectionRow = (String, String, String, String, i64, String, String, String);

fn parse_map(raw: &str) -> Option<BTreeMap<String, String>> {
    if raw.trim().is_empty() {
        return Some(BTreeMap::new());
    }
    serde_json::from_str(raw).ok()
}

async fn load_canonical(db: &SqliteDatabase) -> Result<CanonicalState, StorageError> {
    let mut uow = db.write().await?;
    let active = uow.quran().get_active().await?;
    let Some(active) = active else {
        uow.rollback().await?;
        return Ok(CanonicalState {
            has_edition: false,
            edition_id: String::new(),
            corpus_generation: 0,
            active_dataset: None,
        });
    };
    let edition_id = active.edition_id.clone();
    let corpus_generation = active.corpus_generation;
    let active_dataset = uow.quran().active_dataset().await?.map(|row| (row.slug, row.version));
    uow.rollback().await?;
    Ok(CanonicalState { has_edition: true, edition_id, corpus_generation, active_dataset })
}

async fn load_projections(
    db: &SqliteDatabase,
    deep: bool,
) -> Result<Vec<ProjectionState>, StorageError> {
    let pool = db.read_pool();
    let families =
        [quran_graph::STRUCTURAL_PROJECTION_ID, WORDROOT_PROJECTION_ID, ANNOTATED_PROJECTION_ID];
    let mut out = Vec::new();
    for family in families {
        let row: Option<ProjectionRow> = sqlx::query_as(
            "SELECT id, projection_id, builder_version, edition_id, corpus_generation,
                        dataset_versions_json, dependency_snapshot_json, manifest_json
                 FROM graph_projections
                 WHERE projection_id = ? AND status = 'active'
                 ORDER BY created_at DESC LIMIT 1",
        )
        .bind(family)
        .fetch_optional(pool)
        .await
        .map_err(storage_unavailable)?;
        let Some((
            row_id,
            projection_id,
            builder_version,
            edition_id,
            generation,
            datasets_raw,
            snapshot_raw,
            _manifest_raw,
        )) = row
        else {
            continue;
        };
        let dataset_versions = parse_map(&datasets_raw);
        let dependency_snapshot = parse_map(&snapshot_raw);
        let manifest_parse_ok = dataset_versions.is_some() && dependency_snapshot.is_some();
        let node_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM graph_nodes WHERE projection_row_id = ?")
                .bind(&row_id)
                .fetch_one(pool)
                .await
                .map_err(storage_unavailable)?;
        let edge_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM graph_edges WHERE projection_row_id = ?")
                .bind(&row_id)
                .fetch_one(pool)
                .await
                .map_err(storage_unavailable)?;
        let scan_cap = if deep { usize::MAX } else { SHALLOW_EDGE_SCAN_CAP };
        // One row per dangling edge (atomic: a multi-assertion edge fails
        // once regardless of assertion count).
        let dangling_rows: Vec<(String, String, String)> = sqlx::query_as(
            "SELECT e.src_stable_id, e.edge, e.dst_stable_id FROM graph_edges e
             WHERE e.projection_row_id = ?
               AND (NOT EXISTS (SELECT 1 FROM graph_nodes n
                                WHERE n.projection_row_id = e.projection_row_id
                                  AND n.stable_id = e.src_stable_id)
                    OR NOT EXISTS (SELECT 1 FROM graph_nodes n
                                   WHERE n.projection_row_id = e.projection_row_id
                                     AND n.stable_id = e.dst_stable_id))
             ORDER BY e.src_stable_id, e.edge, e.dst_stable_id
             LIMIT ?",
        )
        .bind(&row_id)
        .bind((scan_cap.saturating_add(1)) as i64)
        .fetch_all(pool)
        .await
        .map_err(storage_unavailable)?;
        let dangling_bounded = dangling_rows.len() > scan_cap;
        let mut dangling = dangling_rows;
        dangling.truncate(scan_cap.min(dangling.len()));
        let tombstoned_ids: Vec<String> = sqlx::query_scalar(
            "SELECT id FROM graph_assertions
             WHERE projection_row_id = ? AND decision IN ('rejected', 'superseded')
             ORDER BY id",
        )
        .bind(&row_id)
        .fetch_all(pool)
        .await
        .map_err(storage_unavailable)?;
        out.push(ProjectionState {
            row_id,
            projection_id,
            builder_version,
            edition_id,
            corpus_generation: generation,
            dataset_versions: dataset_versions.unwrap_or_default(),
            dependency_snapshot: dependency_snapshot.unwrap_or_default(),
            manifest_parse_ok,
            node_count,
            edge_count,
            dangling,
            dangling_bounded,
            tombstoned_ids,
        });
    }
    Ok(out)
}

fn find<'a>(projections: &'a [ProjectionState], family: &str) -> Option<&'a ProjectionState> {
    projections.iter().find(|state| state.projection_id == family)
}

fn check_structural(
    canonical: &CanonicalState,
    projections: &[ProjectionState],
) -> QuranDoctorCheck {
    const ID: &str = "quran.graph.structural.current";
    const NEXT: &str = "qai quran graph build";
    if !canonical.has_edition {
        return skipped(
            ID,
            "no active Arabic edition".to_string(),
            "import and activate an edition, then build the graph",
            "qai quran import --help",
        );
    }
    let Some(state) = find(projections, quran_graph::STRUCTURAL_PROJECTION_ID) else {
        return skipped(
            ID,
            "structural projection not built".to_string(),
            "build the structural projection from the active edition",
            NEXT,
        );
    };
    if !state.manifest_parse_ok {
        return fail(
            ID,
            format!("structural manifest corrupt (build row {})", state.row_id),
            "rebuild the structural projection; never hand-edit projection rows",
            NEXT,
        );
    }
    if state.builder_version != quran_graph::STRUCTURAL_BUILDER_VERSION {
        return warn(
            ID,
            format!(
                "structural projection built by {}, current builder is {}",
                state.builder_version,
                quran_graph::STRUCTURAL_BUILDER_VERSION
            ),
            "rebuild the structural projection with the current builder",
            NEXT,
        );
    }
    if state.edition_id != canonical.edition_id
        || state.corpus_generation != canonical.corpus_generation
    {
        return warn(
            ID,
            format!(
                "stale-but-servable structural projection: covers edition {} at generation {}, active is {} at {}",
                state.edition_id,
                state.corpus_generation,
                canonical.edition_id,
                canonical.corpus_generation
            ),
            "rebuild the structural projection against the active edition",
            NEXT,
        );
    }
    pass(
        ID,
        format!(
            "structural projection current: {} nodes, {} edges at generation {}",
            state.node_count, state.edge_count, state.corpus_generation
        ),
    )
}

fn check_wordroot(canonical: &CanonicalState, projections: &[ProjectionState]) -> QuranDoctorCheck {
    const ID: &str = "quran.graph.wordroot.current";
    const NEXT: &str = "qai quran morphology import --help";
    if !canonical.has_edition {
        return skipped(
            ID,
            "no active Arabic edition".to_string(),
            "import and activate an edition first; the word-root projection is optional",
            "qai quran import --help",
        );
    }
    let Some(state) = find(projections, WORDROOT_PROJECTION_ID) else {
        return skipped(
            ID,
            "word-root projection not built (optional subsystem)".to_string(),
            "activate a morphology dataset, then build the word-root projection",
            NEXT,
        );
    };
    if !state.manifest_parse_ok {
        return fail(
            ID,
            format!("word-root manifest corrupt (build row {})", state.row_id),
            "rebuild the word-root projection; never hand-edit projection rows",
            NEXT,
        );
    }
    if state.builder_version != WORDROOT_BUILDER_VERSION {
        return warn(
            ID,
            format!(
                "word-root projection built by {}, current builder is {WORDROOT_BUILDER_VERSION}",
                state.builder_version
            ),
            "rebuild the word-root projection with the current builder",
            NEXT,
        );
    }
    if state.edition_id != canonical.edition_id
        || state.corpus_generation != canonical.corpus_generation
    {
        return warn(
            ID,
            format!(
                "stale-but-servable word-root projection: built for edition {} at generation {}, active is {} at {}",
                state.edition_id,
                state.corpus_generation,
                canonical.edition_id,
                canonical.corpus_generation
            ),
            "rebuild the word-root projection against the active edition",
            NEXT,
        );
    }
    let Some((slug, version)) = canonical.active_dataset.as_ref() else {
        return warn(
            ID,
            format!(
                "no active morphology dataset; word-root projection frozen at {}",
                state
                    .dataset_versions
                    .iter()
                    .map(|(key, value)| format!("{key}@{value}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            "import and activate a morphology dataset to refresh word-root links",
            NEXT,
        );
    };
    let pinned = state.dataset_versions.get(slug.as_str());
    if pinned.is_some_and(|pinned| pinned == version) {
        pass(
            ID,
            format!(
                "word-root projection current: {} nodes, {} edges on dataset {slug}@{version}",
                state.node_count, state.edge_count
            ),
        )
    } else {
        warn(
            ID,
            format!(
                "stale-but-servable word-root projection: pins {}, active dataset is {slug}@{version}",
                state
                    .dataset_versions
                    .iter()
                    .map(|(key, value)| format!("{key}@{value}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            "rebuild the word-root projection against the active dataset",
            NEXT,
        )
    }
}

fn check_annotated(
    canonical: &CanonicalState,
    projections: &[ProjectionState],
) -> QuranDoctorCheck {
    const ID: &str = "quran.graph.annotated.current";
    const NEXT: &str = "qai quran graph build";
    if !canonical.has_edition {
        return skipped(
            ID,
            "no active Arabic edition".to_string(),
            "import and activate an edition first; the annotated projection is optional",
            "qai quran import --help",
        );
    }
    let Some(state) = find(projections, ANNOTATED_PROJECTION_ID) else {
        return skipped(
            ID,
            "annotated projection not built (optional subsystem)".to_string(),
            "build the annotated projection from the concept seed",
            NEXT,
        );
    };
    if !state.manifest_parse_ok {
        return fail(
            ID,
            format!("annotated manifest corrupt (build row {})", state.row_id),
            "rebuild the annotated projection; never hand-edit projection rows",
            NEXT,
        );
    }
    if state.builder_version != ANNOTATED_BUILDER_VERSION {
        return warn(
            ID,
            format!(
                "annotated projection built by {}, current builder is {ANNOTATED_BUILDER_VERSION}",
                state.builder_version
            ),
            "rebuild the annotated projection with the current builder",
            NEXT,
        );
    }
    if state.edition_id != canonical.edition_id
        || state.corpus_generation != canonical.corpus_generation
    {
        return warn(
            ID,
            format!(
                "stale-but-servable annotated projection: built for edition {} at generation {}, active is {} at {}",
                state.edition_id,
                state.corpus_generation,
                canonical.edition_id,
                canonical.corpus_generation
            ),
            "rebuild the annotated projection against the active edition",
            NEXT,
        );
    }
    if !state.dataset_versions.contains_key(CONCEPT_SEED_SCOPE) {
        return warn(
            ID,
            "annotated projection carries no concept-seed version pin".to_string(),
            "rebuild the annotated projection from the versioned concept seed",
            NEXT,
        );
    }
    pass(
        ID,
        format!(
            "annotated projection current: {} nodes, {} edges at generation {}",
            state.node_count, state.edge_count, state.corpus_generation
        ),
    )
}

fn check_dangling(projections: &[ProjectionState], deep: bool) -> QuranDoctorCheck {
    const ID: &str = "quran.graph.dangling_edges";
    const NEXT: &str = "qai quran graph doctor --deep";
    if projections.is_empty() {
        return skipped(
            ID,
            "nothing built".to_string(),
            "build the structural projection from the active edition",
            "qai quran graph build",
        );
    }
    if !deep && projections.iter().any(|state| state.dangling_bounded) {
        let bounded: Vec<String> = projections
            .iter()
            .filter(|state| state.dangling_bounded)
            .map(|state| {
                format!("{} (bounded at {SHALLOW_EDGE_SCAN_CAP} edges)", state.projection_id)
            })
            .collect();
        return warn(
            ID,
            format!("dangling scan bounded: {}", bounded.join(", ")),
            "rerun with --deep for the full dangling-edge scan",
            NEXT,
        );
    }
    let mut offenders: Vec<String> = Vec::new();
    let mut total = 0usize;
    for state in projections {
        if state.dangling.is_empty() {
            continue;
        }
        total += state.dangling.len();
        let examples: Vec<String> = state
            .dangling
            .iter()
            .take(3)
            .map(|(src, edge, dst)| format!("{src} -{edge}-> {dst}"))
            .collect();
        offenders.push(format!(
            "{}: {} dangling ({})",
            state.projection_id,
            state.dangling.len(),
            examples.join(", ")
        ));
    }
    if offenders.is_empty() {
        let scanned: i64 = projections.iter().map(|state| state.edge_count).sum();
        pass(
            ID,
            format!(
                "no dangling edges across {} active projection(s); {scanned} edges scanned",
                projections.len()
            ),
        )
    } else {
        fail(
            ID,
            format!("{total} dangling edge(s): {}", offenders.join("; ")),
            "quarantine the dangling edges with explicit confirmed repair",
            "qai quran graph doctor-repair quarantine-dangling --yes",
        )
    }
}

fn check_snapshot(projections: &[ProjectionState]) -> QuranDoctorCheck {
    const ID: &str = "quran.graph.dependency_snapshot";
    const NEXT: &str = "qai quran graph build";
    if projections.is_empty() {
        return skipped(
            ID,
            "nothing built".to_string(),
            "build the structural projection from the active edition",
            NEXT,
        );
    }
    fn required(family: &str) -> &'static [&'static str] {
        if family == WORDROOT_PROJECTION_ID {
            &["canonical", "dataset"]
        } else if family == ANNOTATED_PROJECTION_ID {
            &["canonical", "seed"]
        } else {
            &["canonical"]
        }
    }
    let mut gaps: Vec<String> = Vec::new();
    for state in projections {
        let mut missing: Vec<String> = Vec::new();
        for key in required(&state.projection_id) {
            let empty =
                state.dependency_snapshot.get(*key).is_none_or(|value| value.trim().is_empty());
            if empty {
                missing.push((*key).to_string());
            }
        }
        if !state.manifest_parse_ok || !missing.is_empty() {
            gaps.push(format!("{} missing: {}", state.projection_id, missing.join(", ")));
        }
    }
    if gaps.is_empty() {
        pass(
            ID,
            format!(
                "dependency snapshots complete across {} active projection(s)",
                projections.len()
            ),
        )
    } else {
        warn(
            ID,
            format!("incomplete dependency snapshot: {}", gaps.join("; ")),
            "rebuild the projection to re-pin its dependency snapshot",
            NEXT,
        )
    }
}

/// Outcome of one build-row tombstone probe.
enum ProbeOutcome {
    /// No tombstoned assertions in authority; nothing to probe.
    Clean,
    /// Every tombstoned id invisible in traversal and export bytes.
    Invisible(usize),
    /// Tombstoned ids leaked into traversal or export bytes.
    Leaked(Vec<String>),
    /// Every traversal probe truncated, so absence proves nothing.
    Inconclusive,
}

/// Probe tombstone invisibility for one active build row: every rejected and
/// superseded assertion id must appear in authority but in neither traversal
/// results nor export bytes.
async fn probe_tombstones(
    db: &SqliteDatabase,
    state: &ProjectionState,
    deep: bool,
) -> Result<ProbeOutcome, StorageError> {
    use quran_graph::{AuthzScope, GraphStore, QueryBudgets};
    use std::sync::atomic::AtomicBool;

    if state.tombstoned_ids.is_empty() {
        return Ok(ProbeOutcome::Clean);
    }
    let tombstoned: std::collections::BTreeSet<&str> =
        state.tombstoned_ids.iter().map(String::as_str).collect();
    let pool = db.read_pool();
    let src_cap = if deep { usize::MAX } else { SHALLOW_TOMBSTONE_SRC_CAP };
    let mut srcs: Vec<String> = Vec::new();
    for id in &state.tombstoned_ids {
        let rows: Vec<String> = sqlx::query_scalar(
            "SELECT DISTINCT src_stable_id FROM graph_edges
             WHERE projection_row_id = ? AND assertion_id = ? ORDER BY src_stable_id",
        )
        .bind(&state.row_id)
        .bind(id)
        .fetch_all(pool)
        .await
        .map_err(storage_unavailable)?;
        srcs.extend(rows);
    }
    srcs.sort();
    srcs.dedup();
    srcs.truncate(src_cap);
    let store = super::quran_graph_store::SqliteGraphStore::open(db, &state.row_id).await.map_err(
        |error| {
            let _ = error;
            StorageError::StorageUnavailable
        },
    )?;
    let mut leaked: Vec<String> = Vec::new();
    let mut probed = 0usize;
    let mut inconclusive = 0usize;
    for src in &srcs {
        let outcome = store.neighbors(
            src,
            &quran_graph::EdgeFilter::any(),
            &QueryBudgets::default(),
            &AtomicBool::new(false),
            &AuthzScope::all_visible(),
        );
        let Ok(result) = outcome else { continue };
        probed += 1;
        if result.truncated {
            inconclusive += 1;
            continue;
        }
        for edge in &result.edges {
            if let Some(aid) = edge.assertion_id.as_deref()
                && tombstoned.contains(aid)
            {
                leaked.push(aid.to_string());
            }
        }
    }
    leaked.sort();
    leaked.dedup();
    if !leaked.is_empty() {
        return Ok(ProbeOutcome::Leaked(leaked));
    }
    // Export-bytes probe: tombstoned ids must not reach serialized bytes.
    let (nodes, edges, assertions) = store.export_sets();
    let (nodes, edges, assertions) =
        super::quran_graph_annotations::visible_export_sets(&nodes, &edges, &assertions);
    let document = quran_graph::export_json(&nodes, &edges, &assertions, store.manifest());
    let bytes = serde_json::to_string(&document).unwrap_or_default();
    let mut byte_leaks: Vec<String> =
        state.tombstoned_ids.iter().filter(|id| bytes.contains(id.as_str())).cloned().collect();
    byte_leaks.sort();
    leaked.append(&mut byte_leaks);
    leaked.dedup();
    if !leaked.is_empty() {
        return Ok(ProbeOutcome::Leaked(leaked));
    }
    if probed > 0 && inconclusive == probed {
        // Every traversal probe truncated: absence proves nothing.
        return Ok(ProbeOutcome::Inconclusive);
    }
    Ok(ProbeOutcome::Invisible(state.tombstoned_ids.len()))
}

async fn check_tombstones(
    db: &SqliteDatabase,
    projections: &[ProjectionState],
    deep: bool,
) -> QuranDoctorCheck {
    const ID: &str = "quran.graph.tombstone_invisibility";
    const NEXT: &str = "qai quran graph build";
    if projections.is_empty() {
        return skipped(
            ID,
            "nothing built".to_string(),
            "build the structural projection from the active edition",
            NEXT,
        );
    }
    let mut probed_total = 0usize;
    let mut inconclusive_builds: Vec<String> = Vec::new();
    for state in projections {
        match probe_tombstones(db, state, deep).await {
            Ok(ProbeOutcome::Clean | ProbeOutcome::Invisible(0)) => {}
            Ok(ProbeOutcome::Invisible(count)) => probed_total += count,
            Ok(ProbeOutcome::Leaked(leaked)) => {
                return fail(
                    ID,
                    format!(
                        "tombstone leak in {}: {} assertion(s) visible ({})",
                        state.projection_id,
                        leaked.len(),
                        leaked.iter().take(5).cloned().collect::<Vec<_>>().join(", ")
                    ),
                    "tombstoned assertions must stay invisible: rebuild the projection to re-derive adjacency",
                    "qai quran graph doctor-repair rebuild-projection --yes",
                );
            }
            Err(_) | Ok(ProbeOutcome::Inconclusive) => {
                inconclusive_builds.push(state.projection_id.clone());
            }
        }
    }
    if !inconclusive_builds.is_empty() {
        return warn(
            ID,
            format!(
                "tombstone traversal probe inconclusive (truncation) in: {}",
                inconclusive_builds.join(", ")
            ),
            "rerun with --deep and wider budgets for a conclusive probe",
            "qai quran graph doctor --deep",
        );
    }
    if probed_total == 0 {
        pass(ID, "no tombstoned assertions; authority clean".to_string())
    } else {
        pass(
            ID,
            format!(
                "{probed_total} tombstoned assertion(s) invisible in traversal and export bytes, retained in authority"
            ),
        )
    }
}

/// Run the six read-only graph doctor checks, ordered by stable check id.
///
/// `deep` upgrades the dangling-edge scan and the tombstone traversal probe
/// from bounded samples to full scans.
pub async fn run_quran_graph_checks(
    db: &SqliteDatabase,
    deep: bool,
) -> Result<Vec<QuranDoctorCheck>, StorageError> {
    let canonical = load_canonical(db).await?;
    let projections = load_projections(db, deep).await?;
    let mut checks = vec![
        check_annotated(&canonical, &projections),
        check_dangling(&projections, deep),
        check_snapshot(&projections),
        check_structural(&canonical, &projections),
        check_tombstones(db, &projections, deep).await,
        check_wordroot(&canonical, &projections),
    ];
    checks.sort_by(|a, b| a.id.cmp(b.id));
    debug_assert!(checks.iter().map(|check| check.id).collect::<Vec<_>>() == GRAPH_CHECK_IDS);
    Ok(checks)
}

// ─── Explicit confirmed repair (D-08/D-11, T-04-15) ───
//
// Doctor itself never repairs: every function below requires explicit
// confirmation (`confirmed: true`, wired to `--yes` / APPROVE at the CLI),
// emits one [`audit::AuditAction::DoctorRepairExecuted`] audit event, and
// returns before-and-after counts. Tombstone GC removes only Rejected and
// Superseded assertion rows older than the stated retention and refuses to
// touch pending, accepted, or disputed rows by construction (the decision
// filter is in the SQL, not in a comment).

/// Confirmation plus operator identity shared by every repair command.
#[derive(Debug, Clone)]
pub struct RepairInput {
    /// Recording operator principal (FK-checked, never invented).
    pub invoked_by: String,
    /// Explicit confirmation (`--yes` / APPROVE at the CLI).
    pub confirmed: bool,
}

/// Tombstone-GC input: confirmation plus the age gate in days.
#[derive(Debug, Clone)]
pub struct TombstoneGcInput {
    /// Recording operator principal (FK-checked, never invented).
    pub invoked_by: String,
    /// Explicit confirmation (`--yes` / APPROVE at the CLI).
    pub confirmed: bool,
    /// Only tombstoned rows with `decided_at` older than this many days are
    /// collected; younger tombstones are retained.
    pub retention_days: u32,
}

/// Rebuild input: confirmation plus the projection family to republish.
#[derive(Debug, Clone)]
pub struct RebuildInput {
    /// Recording operator principal (FK-checked, never invented).
    pub invoked_by: String,
    /// Explicit confirmation (`--yes` / APPROVE at the CLI).
    pub confirmed: bool,
    /// Projection family id (`quran-structural-v1`, `quran-wordroot-v1`,
    /// or `quran-annotated-v1`).
    pub projection: String,
    /// Concept-seed JSON path (required for the annotated family only).
    pub seed_file: Option<String>,
}

/// Before-and-after report for one confirmed repair.
#[derive(Debug, Clone)]
pub struct RepairReport {
    /// Repair operation (`quarantine-dangling`, `tombstone-gc`, `rebuild-projection`).
    pub operation: &'static str,
    /// Human summary with before-and-after counts.
    pub summary: String,
    /// Dangling edges quarantined (quarantine only).
    pub edges_removed: usize,
    /// Tombstoned assertions garbage-collected (GC only).
    pub assertions_removed: usize,
    /// Live (pending/accepted/disputed) rows the GC refused to touch.
    pub live_rows_refused: usize,
    /// Audit sequence of the emitted repair event.
    pub audit_sequence: i64,
    /// Full machine records (removed rows, rebuilt manifest).
    pub records: serde_json::Value,
}

fn build_failed(stage: &str, detail: String) -> quran_graph::GraphError {
    quran_graph::GraphError::BuildFailed { stage: stage.to_string(), detail }
}

fn require_confirmed(
    operation: &'static str,
    confirmed: bool,
) -> Result<(), quran_graph::GraphError> {
    if confirmed {
        return Ok(());
    }
    Err(build_failed(
        "confirm",
        format!(
            "refusing {operation} without explicit confirmation; rerun with --yes (or APPROVE)"
        ),
    ))
}

fn open_write_pool(db_path: &str) -> Result<sqlx::sqlite::SqlitePool, quran_graph::GraphError> {
    if std::fs::symlink_metadata(db_path).is_err() {
        return Err(build_failed(
            "open",
            format!("no database at '{db_path}'; run `qai db migrate`"),
        ));
    }
    let options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(false)
        .foreign_keys(true)
        .busy_timeout(std::time::Duration::from_secs(5));
    Ok(sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect_lazy_with(options))
}

async fn ensure_operator(
    db_path: &str,
    invoked_by: &str,
) -> Result<String, quran_graph::GraphError> {
    if invoked_by.trim().is_empty() {
        return Err(build_failed(
            "principal",
            "invoked_by must be non-empty (never invented)".to_string(),
        ));
    }
    // Guard before `SqliteDatabase::new`: the creating constructor would
    // otherwise materialize an empty database file on a typo'd path.
    if std::fs::symlink_metadata(db_path).is_err() {
        return Err(build_failed(
            "open",
            format!("no database at '{db_path}'; run `qai db migrate`"),
        ));
    }
    let at = domain::Timestamp::now().to_string();
    let db = SqliteDatabase::new(db_path, 4, true)
        .await
        .map_err(|error| build_failed("open", error.to_string()))?;
    crate::quran::ensure_principal(&db, invoked_by, "graph doctor operator", &at)
        .await
        .map_err(|error| build_failed("principal", error.to_string()))?;
    Ok(at)
}

async fn repair_audit_sequence(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
) -> Result<i64, quran_graph::GraphError> {
    sqlx::query_scalar("SELECT MAX(sequence) FROM audit_events")
        .fetch_one(&mut **tx)
        .await
        .map_err(|error| build_failed("audit", error.to_string()))
}

/// Quarantine dangling edges out of every active projection's serving
/// adjacency (D-08/D-11).
///
/// The removed edge rows are recorded verbatim in the repair audit event, so
/// the quarantine is reviewable and the pre-repair doctor output stays
/// reproducible from the audit trail. Authority rows are never touched.
pub async fn repair_quarantine_dangling(
    db_path: &str,
    input: RepairInput,
) -> Result<RepairReport, quran_graph::GraphError> {
    const OPERATION: &str = "quarantine-dangling";
    require_confirmed(OPERATION, input.confirmed)?;
    let at = ensure_operator(db_path, &input.invoked_by).await?;
    let pool = open_write_pool(db_path)?;
    let mut tx =
        pool.begin().await.map_err(|error| build_failed("quarantine", error.to_string()))?;
    let builds: Vec<(String, String)> = sqlx::query_as(
        "SELECT id, projection_id FROM graph_projections WHERE status = 'active' ORDER BY projection_id",
    )
    .fetch_all(&mut *tx)
    .await
    .map_err(|error| build_failed("quarantine", error.to_string()))?;
    let mut removed: Vec<serde_json::Value> = Vec::new();
    let mut before = 0usize;
    for (row_id, family) in &builds {
        let edges: Vec<(String, String, String, String, Option<String>)> = sqlx::query_as(
            "SELECT e.id, e.src_stable_id, e.edge, e.dst_stable_id, e.assertion_id
             FROM graph_edges e WHERE e.projection_row_id = ?
               AND (NOT EXISTS (SELECT 1 FROM graph_nodes n
                                WHERE n.projection_row_id = e.projection_row_id
                                  AND n.stable_id = e.src_stable_id)
                    OR NOT EXISTS (SELECT 1 FROM graph_nodes n
                                   WHERE n.projection_row_id = e.projection_row_id
                                     AND n.stable_id = e.dst_stable_id))
             ORDER BY e.src_stable_id, e.edge, e.dst_stable_id",
        )
        .bind(row_id)
        .fetch_all(&mut *tx)
        .await
        .map_err(|error| build_failed("quarantine", error.to_string()))?;
        before += edges.len();
        for (id, src, edge, dst, assertion_id) in &edges {
            removed.push(serde_json::json!({
                "projection": family,
                "id": id,
                "src": src,
                "edge": edge,
                "dst": dst,
                "assertion_id": assertion_id,
            }));
        }
        let ids: Vec<&str> = edges.iter().map(|(id, _, _, _, _)| id.as_str()).collect();
        for id in ids {
            sqlx::query("DELETE FROM graph_edges WHERE id = ? AND projection_row_id = ?")
                .bind(id)
                .bind(row_id)
                .execute(&mut *tx)
                .await
                .map_err(|error| build_failed("quarantine", error.to_string()))?;
        }
    }
    let records = serde_json::json!({"removed": removed, "edges_before": before, "edges_after": 0});
    super::quran_graph_annotations::append_repair_audit(
        &mut tx,
        &input.invoked_by,
        OPERATION,
        serde_json::json!({"operation": OPERATION, "removed_count": removed.len(), "records": records}),
        &at,
    )
    .await?;
    let audit_sequence = repair_audit_sequence(&mut tx).await?;
    tx.commit().await.map_err(|error| build_failed("quarantine", error.to_string()))?;
    pool.close().await;
    Ok(RepairReport {
        operation: OPERATION,
        summary: format!(
            "quarantined {} dangling edge(s) ({} before, 0 after) across {} active projection(s); adjacency clean",
            removed.len(),
            before,
            builds.len()
        ),
        edges_removed: removed.len(),
        assertions_removed: 0,
        live_rows_refused: 0,
        audit_sequence,
        records,
    })
}

/// Garbage-collect tombstoned assertions older than the retention gate
/// (D-08/D-11).
///
/// Only `rejected`/`superseded` rows with a non-null `decided_at` older than
/// `retention_days` are collected; pending, accepted, and disputed rows are
/// unreachable by construction (the decision filter is in the `DELETE`, and
/// the report counts the live rows refused). Each collected assertion's
/// adjacency edges leave with it (they are invisible by tombstone semantics);
/// authority history younger than retention is retained.
pub async fn repair_tombstone_gc(
    db_path: &str,
    input: TombstoneGcInput,
) -> Result<RepairReport, quran_graph::GraphError> {
    const OPERATION: &str = "tombstone-gc";
    require_confirmed(OPERATION, input.confirmed)?;
    let at = ensure_operator(db_path, &input.invoked_by).await?;
    // Retention compares inside SQLite (`julianday` parses the RFC 3339 UTC
    // `decided_at` values the review writes store): no new date dependency,
    // no format skew between a host-side cutoff string and stored rows.
    // A row is due when its age in days strictly exceeds the retention.
    let retention = f64::from(input.retention_days);
    let pool = open_write_pool(db_path)?;
    let mut tx = pool.begin().await.map_err(|error| build_failed("gc", error.to_string()))?;
    let cutoff_display: String =
        sqlx::query_scalar("SELECT datetime('now', '-' || CAST(? AS TEXT) || ' days')")
            .bind(i64::from(input.retention_days))
            .fetch_one(&mut *tx)
            .await
            .map_err(|error| build_failed("retention", error.to_string()))?;
    let due: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT id, projection_row_id, decision FROM graph_assertions
         WHERE decision IN ('rejected', 'superseded')
           AND decided_at IS NOT NULL
           AND (julianday('now') - julianday(decided_at)) > ?
         ORDER BY id",
    )
    .bind(retention)
    .fetch_all(&mut *tx)
    .await
    .map_err(|error| build_failed("gc", error.to_string()))?;
    let live_rows_refused: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM graph_assertions WHERE decision IN ('pending', 'accepted', 'disputed')",
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|error| build_failed("gc", error.to_string()))?;
    let mut collected: Vec<serde_json::Value> = Vec::new();
    for (id, row_id, decision) in &due {
        let edges: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM graph_edges WHERE projection_row_id = ? AND assertion_id = ?",
        )
        .bind(row_id)
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|error| build_failed("gc", error.to_string()))?;
        sqlx::query("DELETE FROM graph_edges WHERE projection_row_id = ? AND assertion_id = ?")
            .bind(row_id)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|error| build_failed("gc", error.to_string()))?;
        sqlx::query("DELETE FROM graph_assertions WHERE id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|error| build_failed("gc", error.to_string()))?;
        collected.push(serde_json::json!({
            "id": id,
            "build_row_id": row_id,
            "decision": decision,
            "edges_removed": edges,
        }));
    }
    let records = serde_json::json!({
        "retention_days": input.retention_days,
        "cutoff": cutoff_display,
        "collected": collected,
    });
    super::quran_graph_annotations::append_repair_audit(
        &mut tx,
        &input.invoked_by,
        OPERATION,
        serde_json::json!({
            "operation": OPERATION,
            "collected_count": collected.len(),
            "live_rows_refused": live_rows_refused,
            "records": records,
        }),
        &at,
    )
    .await?;
    let audit_sequence = repair_audit_sequence(&mut tx).await?;
    tx.commit().await.map_err(|error| build_failed("gc", error.to_string()))?;
    pool.close().await;
    Ok(RepairReport {
        operation: OPERATION,
        summary: format!(
            "collected {} tombstoned assertion(s) older than {} day(s); refused {} live row(s)",
            collected.len(),
            input.retention_days,
            live_rows_refused
        ),
        edges_removed: 0,
        assertions_removed: collected.len(),
        live_rows_refused: live_rows_refused.max(0) as usize,
        audit_sequence,
        records,
    })
}

/// Rebuild one projection through the fenced publish path, preserving
/// authority (D-08/D-11).
///
/// Structural, word-root, and annotated republishers never touch
/// `graph_assertions` rows: delete-plus-rebuild preserves authority and
/// re-derives adjacency for effective assertions (AC-P4-03). The repair audit
/// event records the new build row plus its node/edge counts.
pub async fn repair_rebuild_projection(
    db_path: &str,
    input: RebuildInput,
) -> Result<RepairReport, quran_graph::GraphError> {
    const OPERATION: &str = "rebuild-projection";
    require_confirmed(OPERATION, input.confirmed)?;
    let at = ensure_operator(db_path, &input.invoked_by).await?;
    let (build_row_id, node_count, edge_count, family) = if input.projection
        == quran_graph::STRUCTURAL_PROJECTION_ID
    {
        let db = SqliteDatabase::new(db_path, 4, true)
            .await
            .map_err(|error| build_failed("open", error.to_string()))?;
        let collected =
            super::quran_graph_build::collect_structural_input(&db).await?.ok_or_else(|| {
                build_failed("collect", "no active edition; import one first".to_string())
            })?;
        let report =
            super::quran_graph_build::publish_structural_build(db_path, &collected).await?;
        (
            report.build_row_id,
            report.node_count,
            report.edge_count,
            quran_graph::STRUCTURAL_PROJECTION_ID.to_string(),
        )
    } else if input.projection == super::quran_graph_build::WORDROOT_PROJECTION_ID {
        let db = SqliteDatabase::new(db_path, 4, true)
            .await
            .map_err(|error| build_failed("open", error.to_string()))?;
        let collected = super::quran_graph_build::collect_wordroot_input(&db)
            .await
            .map_err(|error| build_failed("collect", error.to_string()))?;
        let report = super::quran_graph_build::publish_wordroot_build(db_path, &collected).await?;
        (
            report.build_row_id,
            report.node_count,
            report.edge_count,
            super::quran_graph_build::WORDROOT_PROJECTION_ID.to_string(),
        )
    } else if input.projection == super::quran_graph_build::ANNOTATED_PROJECTION_ID {
        let seed_file = input.seed_file.as_deref().ok_or_else(|| {
            build_failed(
                "collect",
                "annotated rebuild needs --seed-file pointing at the concept seed JSON".to_string(),
            )
        })?;
        let seed_json = std::fs::read_to_string(seed_file)
            .map_err(|error| build_failed("collect", format!("read {seed_file}: {error}")))?;
        let report = super::quran_graph_build::publish_annotated_build(db_path, &seed_json).await?;
        (
            report.build_row_id,
            report.node_count,
            report.edge_count,
            super::quran_graph_build::ANNOTATED_PROJECTION_ID.to_string(),
        )
    } else {
        return Err(build_failed(
            "collect",
            format!(
                "unknown projection '{}'; use {}, {}, or {}",
                input.projection,
                quran_graph::STRUCTURAL_PROJECTION_ID,
                super::quran_graph_build::WORDROOT_PROJECTION_ID,
                super::quran_graph_build::ANNOTATED_PROJECTION_ID,
            ),
        ));
    };
    let records = serde_json::json!({"projection": family, "build_row_id": build_row_id, "nodes": node_count, "edges": edge_count});
    let pool = open_write_pool(db_path)?;
    let mut tx = pool.begin().await.map_err(|error| build_failed("audit", error.to_string()))?;
    super::quran_graph_annotations::append_repair_audit(
        &mut tx,
        &input.invoked_by,
        OPERATION,
        serde_json::json!({"operation": OPERATION, "records": records}),
        &at,
    )
    .await?;
    let audit_sequence = repair_audit_sequence(&mut tx).await?;
    tx.commit().await.map_err(|error| build_failed("audit", error.to_string()))?;
    pool.close().await;
    Ok(RepairReport {
        operation: OPERATION,
        summary: format!(
            "rebuilt {family}: {node_count} nodes, {edge_count} edges; authority preserved"
        ),
        edges_removed: 0,
        assertions_removed: 0,
        live_rows_refused: 0,
        audit_sequence,
        records,
    })
}
