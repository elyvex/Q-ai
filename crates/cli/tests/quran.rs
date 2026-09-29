//! CLI acceptance snapshots — end-to-end flows (AC-P1-16, P1-T50, P2-T23).
//!
//! Points the real `qai` binary at a fresh temp database via `QAI_DATA_DIR`,
//! then drives `trycmd` cases (see `tests/quran/`). Cases in a file run in
//! order against the one database; human output elides volatile values.
//!
//! Each file gets its own database: trycmd files execute in parallel, so two
//! files sharing one `QAI_DATA_DIR` race (notably two `db migrate` runs on
//! the same SQLite file).

fn run_cases(pattern: &str) {
    let dir = tempfile::tempdir().unwrap();
    trycmd::TestCases::new()
        .default_bin_name("qai")
        .env("QAI_DATA_DIR", dir.path().to_str().unwrap())
        .case("tests/quran/*.toml")
        .case(pattern);
    // Keep the temp database alive until assertions complete.
    drop(dir);
}

/// Phase 1 reading flow (AC-P1-16, P1-T50): host-backed segments — the queued
/// imports reach terminal state under `qai serve` before dependent commands.
#[test]
fn quran_snapshots() {
    let guard = ServeGuard::start();
    guard.run_segments(&[
        "tests/quran/read_flow_s1.trycmd",
        "tests/quran/read_flow_s2.trycmd",
        "tests/quran/read_flow_s3.trycmd",
        "tests/quran/read_flow_s4.trycmd",
    ]);
}

/// Phase 2 normalization introspection (P2-T23): host-backed segments.
#[test]
fn quran_normalize_snapshots() {
    let guard = ServeGuard::start();
    guard.run_segments(&["tests/quran/normalize_s1.trycmd", "tests/quran/normalize_s2.trycmd"]);
}

/// Phase 2 search surfaces (P2-T51/T52): host-backed segments.
#[test]
fn quran_search_snapshots() {
    let guard = ServeGuard::start();
    guard.run_segments(&["tests/quran/search_s1.trycmd", "tests/quran/search_s2.trycmd"]);
}

/// Phase 2 counting + Phase 4 graph CLI surfaces (P2-T104, TASK-424 slice):
/// host-backed segments.
#[test]
fn quran_counting_graph_snapshots() {
    let guard = ServeGuard::start();
    guard.run_segments(&[
        "tests/quran/counting_graph_s1.trycmd",
        "tests/quran/counting_graph_s2.trycmd",
        "tests/quran/counting_graph_s3.trycmd",
    ]);
}

/// Phase 4 tracer CLI surface (D-11): host-backed import, then the
/// SQLite-backed structural slice — build persists the projection, inspect
/// shows the manifest with its generation stamp, neighbors open a bounded
/// view with pinned canonical quotations, and selection/validation errors
/// stay typed.
#[test]
fn quran_graph_snapshots() {
    let guard = ServeGuard::start();
    guard.run_segments(&["tests/quran/graph_s1.trycmd"]);

    let out = qai_out(guard.dir.path(), &["quran", "activate", "test-edition-min@0.1.0", "--yes"]);
    assert!(out.status.success(), "activate: {}", String::from_utf8_lossy(&out.stderr));
    let human = String::from_utf8_lossy(&out.stdout);
    assert!(human.contains("activated test-edition-min@0.1.0 (generation 1)"), "{human}");

    // Build persists the structural projection into SQLite.
    let out = qai_out(guard.dir.path(), &["quran", "graph", "build"]);
    assert!(out.status.success(), "build: {}", String::from_utf8_lossy(&out.stderr));
    let human = String::from_utf8_lossy(&out.stdout);
    assert!(human.contains("structural projection:"), "{human}");
    assert!(human.contains("(edition"), "{human}");

    // The JSON document pins the structural family identity.
    let out = qai_out(guard.dir.path(), &["quran", "graph", "build", "--json"]);
    assert!(out.status.success(), "build --json: {}", String::from_utf8_lossy(&out.stderr));
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).expect("one JSON document");
    assert_eq!(doc["manifest"]["projection_id"], "quran-structural-v1", "{doc}");
    assert!(doc["nodes"].as_array().is_some_and(|nodes| !nodes.is_empty()), "{doc}");
    assert!(doc["edges"].as_array().is_some_and(|edges| !edges.is_empty()), "{doc}");

    // Read companion segment (D-09/D-11/D-14): neighbors, all three path
    // modes, subgraph, pattern, export, and one verbatim truncation.
    guard.run_segments(&["tests/quran/graph_s2.trycmd"]);

    // Inspect reads the manifest back from SQLite with its generation stamp.
    let out = qai_out(guard.dir.path(), &["quran", "graph", "inspect", "--db"]);
    assert!(out.status.success(), "inspect --db: {}", String::from_utf8_lossy(&out.stderr));
    let human = String::from_utf8_lossy(&out.stdout);
    assert!(human.contains("quran-structural-v1"), "{human}");
    assert!(human.contains("generation 1"), "{human}");

    // Exactly one of --file or --db is required.
    let out = qai_out(guard.dir.path(), &["quran", "graph", "inspect"]);
    assert_eq!(out.status.code(), Some(2), "both flags absent is a usage error");
    let out = qai_out(guard.dir.path(), &["quran", "graph", "inspect", "--file", "x.json", "--db"]);
    assert_eq!(out.status.code(), Some(2), "both flags present is a usage error");

    // Bounded neighbors with pinned canonical quotations on ayah hits.
    let out =
        qai_out(guard.dir.path(), &["quran", "graph", "neighbors", "--db", "--node", "ayah:1:1"]);
    assert!(out.status.success(), "neighbors: {}", String::from_utf8_lossy(&out.stderr));
    let human = String::from_utf8_lossy(&out.stdout);
    assert!(human.contains("neighbors of ayah:1:1"), "{human}");
    assert!(human.contains("ayah:1:1 -> quran:"), "{human}");

    let out = qai_out(
        guard.dir.path(),
        &["quran", "graph", "neighbors", "--db", "--node", "ayah:1:1", "--json"],
    );
    assert!(out.status.success(), "neighbors --json: {}", String::from_utf8_lossy(&out.stderr));
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).expect("one JSON document");
    assert_eq!(doc["truncated"], false, "{doc}");
    assert_eq!(doc["manifest"]["projection_id"], "quran-structural-v1", "{doc}");
    let quotations = doc["quotations"].as_array().cloned().unwrap_or_default();
    assert!(!quotations.is_empty(), "ayah hits carry quotations: {doc}");
    for quotation in &quotations {
        let reference = quotation["reference"].as_str().unwrap_or_default();
        assert!(reference.starts_with("quran:"), "pinned canonical ref: {quotation}");
        assert!(reference.contains("@0.1.0:") || reference.contains('@'), "{quotation}");
    }

    // Unknown nodes are not-found; out-of-range budgets are validation errors.
    let out =
        qai_out(guard.dir.path(), &["quran", "graph", "neighbors", "--db", "--node", "ayah:9:99"]);
    assert_eq!(out.status.code(), Some(5), "unknown node is not found");
    let human = String::from_utf8_lossy(&out.stdout);
    assert!(human.contains("QAI-GRAPH-0004"), "typed code renders: {human}");
    let out = qai_out(
        guard.dir.path(),
        &["quran", "graph", "neighbors", "--db", "--node", "ayah:1:1", "--hops", "0"],
    );
    assert_eq!(out.status.code(), Some(3), "out-of-range hops are a validation error");

    // All three path modes resolve the active projection with per-edge
    // provenance in JSON.
    let out = qai_out(
        guard.dir.path(),
        &[
            "quran",
            "graph",
            "path",
            "--db",
            "--mode",
            "reachability",
            "--from",
            "ayah:1:1",
            "--to",
            "ayah:1:3",
            "--json",
        ],
    );
    assert!(out.status.success(), "reachability: {}", String::from_utf8_lossy(&out.stderr));
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).expect("one JSON document");
    assert_eq!(doc["mode"], "reachability", "{doc}");
    assert_eq!(doc["reachable"], true, "{doc}");
    assert_eq!(doc["hops"], 2, "{doc}");
    assert_eq!(doc["truncated"], false, "{doc}");
    assert_eq!(doc["explanation"]["snapshot"]["projection_id"], "quran-structural-v1", "{doc}");

    let out = qai_out(
        guard.dir.path(),
        &[
            "quran", "graph", "path", "--db", "--mode", "shortest", "--from", "ayah:1:1", "--to",
            "ayah:1:3", "--json",
        ],
    );
    assert!(out.status.success(), "shortest: {}", String::from_utf8_lossy(&out.stderr));
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).expect("one JSON document");
    assert_eq!(doc["mode"], "shortest", "{doc}");
    assert_eq!(
        doc["path"]["node_ids"],
        serde_json::json!(["ayah:1:1", "surah:1", "ayah:1:3"]),
        "{doc}"
    );
    assert_eq!(doc["no_path_proven"], false, "{doc}");
    let edges = doc["explanation"]["paths"][0]["edges"].as_array().cloned().unwrap_or_default();
    assert_eq!(edges.len(), 2, "every traversed edge is explained: {doc}");
    for edge in &edges {
        assert_eq!(edge["provenance"]["kind"], "structural", "{edge}");
        assert!(edge["provenance"]["input_version"].is_string(), "{edge}");
    }

    let out = qai_out(
        guard.dir.path(),
        &[
            "quran",
            "graph",
            "path",
            "--db",
            "--mode",
            "paths",
            "--paths",
            "3",
            "--from",
            "token:1:1:1",
            "--to",
            "token:1:1:3",
            "--json",
        ],
    );
    assert!(out.status.success(), "up-to-K: {}", String::from_utf8_lossy(&out.stderr));
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).expect("one JSON document");
    assert_eq!(doc["paths"].as_array().map(Vec::len), Some(3), "{doc}");
    assert_eq!(doc["truncated"], false, "{doc}");

    // Unknown path endpoints are not-found; K beyond max-paths and unknown
    // modes fail before any I/O.
    let out = qai_out(
        guard.dir.path(),
        &[
            "quran",
            "graph",
            "path",
            "--db",
            "--mode",
            "shortest",
            "--from",
            "ayah:1:1",
            "--to",
            "ayah:9:99",
        ],
    );
    assert_eq!(out.status.code(), Some(5), "unknown path endpoint is not found");
    let out = qai_out(
        guard.dir.path(),
        &[
            "quran", "graph", "path", "--db", "--mode", "paths", "--paths", "11", "--from",
            "ayah:1:1", "--to", "ayah:1:2",
        ],
    );
    assert_eq!(out.status.code(), Some(3), "K beyond max-paths is a pre-flight error");
    let out = qai_out(
        guard.dir.path(),
        &["quran", "graph", "path", "--db", "--mode", "bogus", "--from", "a", "--to", "b"],
    );
    assert_eq!(out.status.code(), Some(2), "unknown mode is a usage error");

    // Subgraph plus pattern resolve the active projection with budgets from
    // flags; unknown predicates are validation failures.
    let out =
        qai_out(guard.dir.path(), &["quran", "graph", "subgraph", "--seed", "ayah:1:1", "--json"]);
    assert!(out.status.success(), "subgraph: {}", String::from_utf8_lossy(&out.stderr));
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).expect("one JSON document");
    assert!(!doc["nodes"].as_array().cloned().unwrap_or_default().is_empty(), "{doc}");
    assert!(!doc["edges"].as_array().cloned().unwrap_or_default().is_empty(), "{doc}");
    assert_eq!(doc["truncated"], false, "{doc}");
    assert_eq!(doc["explanation"]["snapshot"]["corpus_generation"], 1, "{doc}");
    let out = qai_out(
        guard.dir.path(),
        &["quran", "graph", "subgraph", "--seed", "ayah:1:1", "--hops", "0"],
    );
    assert_eq!(out.status.code(), Some(3), "out-of-range subgraph hops are validation");
    let out = qai_out(guard.dir.path(), &["quran", "graph", "subgraph"]);
    assert_eq!(out.status.code(), Some(2), "missing seeds are a usage error");

    let out = qai_out(
        guard.dir.path(),
        &["quran", "graph", "pattern", "--seed", "ayah:1:1", "--step", "NEXT", "--json"],
    );
    assert!(out.status.success(), "pattern: {}", String::from_utf8_lossy(&out.stderr));
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).expect("one JSON document");
    assert!(
        doc["nodes"]
            .as_array()
            .is_some_and(|nodes| { nodes.iter().any(|node| node["stable_id"] == "ayah:1:2") }),
        "NEXT reaches ayah:1:2: {doc}"
    );
    let out = qai_out(
        guard.dir.path(),
        &["quran", "graph", "pattern", "--seed", "ayah:1:1", "--step", "PRECEDES"],
    );
    assert_eq!(out.status.code(), Some(3), "unknown pattern edge is a validation error");
    let out = qai_out(
        guard.dir.path(),
        &["quran", "graph", "pattern", "--seed", "ayah:1:1", "--step", "NEXT:Nope"],
    );
    assert_eq!(out.status.code(), Some(3), "unknown pattern kind is a validation error");

    // Truncated reads carry the flag plus the verbatim reason in JSON, never
    // an empty list masquerading as absence.
    let out = qai_out(
        guard.dir.path(),
        &[
            "quran",
            "graph",
            "neighbors",
            "--db",
            "--node",
            "ayah:1:1",
            "--max-nodes",
            "1",
            "--json",
        ],
    );
    assert!(out.status.success(), "truncated read: {}", String::from_utf8_lossy(&out.stderr));
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).expect("one JSON document");
    assert_eq!(doc["truncated"], true, "{doc}");
    assert!(
        doc["incomplete_reason"].as_str().is_some_and(|reason| reason.contains("node budget")),
        "verbatim reason renders: {doc}"
    );

    // Export resolves the active projection through the policy filter with
    // the version marker and counts.
    let out = qai_out(guard.dir.path(), &["quran", "graph", "export", "--db", "--json"]);
    assert!(out.status.success(), "export: {}", String::from_utf8_lossy(&out.stderr));
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).expect("one JSON document");
    assert_eq!(doc["format"], "quran-graph-json-v1", "{doc}");
    assert_eq!(doc["counts"]["nodes"], 84, "{doc}");
    assert_eq!(doc["counts"]["edges"], 142, "{doc}");
    assert_eq!(doc["truncation"]["truncated"], false, "{doc}");

    // Static rendering writes DOT/SVG files carrying node and edge
    // identities (coordinates stay presentational, never asserted).
    let dot_path = guard.dir.path().join("graph.dot");
    let out = qai_out(
        guard.dir.path(),
        &[
            "quran",
            "graph",
            "export",
            "--db",
            "--format",
            "dot",
            "--out",
            dot_path.to_str().unwrap(),
        ],
    );
    assert!(out.status.success(), "export dot: {}", String::from_utf8_lossy(&out.stderr));
    let dot = std::fs::read_to_string(&dot_path).expect("DOT file writes");
    assert!(dot.contains("digraph"), "{dot}");
    assert!(dot.contains("ayah:1:1"), "{dot}");
    assert!(dot.contains("NEXT"), "{dot}");
    let svg_path = guard.dir.path().join("graph.svg");
    let out = qai_out(
        guard.dir.path(),
        &[
            "quran",
            "graph",
            "export",
            "--db",
            "--format",
            "svg",
            "--seed",
            "ayah:1:1",
            "--out",
            svg_path.to_str().unwrap(),
        ],
    );
    assert!(out.status.success(), "export svg: {}", String::from_utf8_lossy(&out.stderr));
    let svg = std::fs::read_to_string(&svg_path).expect("SVG file writes");
    assert!(svg.contains("<svg"), "{svg}");
    assert!(svg.contains("ayah:1:1"), "{svg}");
    // DOT/SVG without --out and unknown formats are usage errors.
    let out = qai_out(guard.dir.path(), &["quran", "graph", "export", "--db", "--format", "dot"]);
    assert_eq!(out.status.code(), Some(2), "render without --out is a usage error");
    let out =
        qai_out(guard.dir.path(), &["quran", "graph", "export", "--db", "--format", "graphml"]);
    assert_eq!(out.status.code(), Some(2), "unknown format is a usage error");

    // Word-root reads without an active dataset are the typed unavailable
    // capability (exit 5 with the morphology diagnostic code).
    let out = qai_out(guard.dir.path(), &["quran", "graph", "root-family", "ktb"]);
    assert_eq!(out.status.code(), Some(5), "unavailable word-root capability is not found");
    let human = String::from_utf8_lossy(&out.stdout);
    assert!(human.contains("QAI-MORPH-0004"), "morphology diagnostic code renders: {human}");

    // Review management verbs (D-06/D-11): propose then suggest then accept
    // then correct, each audited, against the host-backed database.
    let out = qai_out(
        guard.dir.path(),
        &[
            "quran",
            "graph",
            "review",
            "propose",
            "--id",
            "rw-cli-propose",
            "--src",
            "ayah:1:1",
            "--edge",
            "PARALLELS",
            "--dst",
            "ayah:1:2",
            "--source-id",
            "cli-snapshot",
            "--source-location",
            "quran.rs",
            "--author",
            "pending-scholar",
        ],
    );
    assert!(out.status.success(), "propose: {}", String::from_utf8_lossy(&out.stderr));
    let human = String::from_utf8_lossy(&out.stdout);
    assert!(human.contains("proposed rw-cli-propose (pending)"), "{human}");
    assert!(human.contains("pending-scholar"), "{human}");

    let out = qai_out(
        guard.dir.path(),
        &[
            "quran",
            "graph",
            "review",
            "suggest",
            "--id",
            "rw-cli-suggest",
            "--src",
            "ayah:1:1",
            "--edge",
            "PARALLELS",
            "--dst",
            "ayah:1:2",
            "--source-id",
            "cli-snapshot",
            "--source-location",
            "quran.rs",
            "--algorithm",
            "tracer-suggest-v1",
            "--algorithm-version",
            "1.0.0",
            "--confidence",
            "0.42",
        ],
    );
    assert!(out.status.success(), "suggest: {}", String::from_utf8_lossy(&out.stderr));
    let human = String::from_utf8_lossy(&out.stdout);
    assert!(human.contains("suggested rw-cli-suggest (pending)"), "{human}");

    let out = qai_out(
        guard.dir.path(),
        &[
            "quran",
            "graph",
            "review",
            "accept",
            "--id",
            "rw-cli-suggest",
            "--reviewer",
            "pending-scholar",
        ],
    );
    assert!(out.status.success(), "accept: {}", String::from_utf8_lossy(&out.stderr));
    let human = String::from_utf8_lossy(&out.stdout);
    assert!(human.contains("accepted rw-cli-suggest (accepted)"), "{human}");
    assert!(human.contains("pending-scholar"), "{human}");

    let out = qai_out(
        guard.dir.path(),
        &[
            "quran",
            "graph",
            "review",
            "correct",
            "--id",
            "rw-cli-suggest",
            "--dst",
            "ayah:2:1",
            "--reviewer",
            "pending-scholar",
        ],
    );
    assert!(out.status.success(), "correct: {}", String::from_utf8_lossy(&out.stderr));
    let human = String::from_utf8_lossy(&out.stdout);
    assert!(human.contains("corrected"), "{human}");
    assert!(human.contains("supersedes rw-cli-suggest"), "{human}");

    // The full assertion record rides the JSON surface with reviewer plus
    // timestamp plus decision.
    let out = qai_out(
        guard.dir.path(),
        &[
            "quran",
            "graph",
            "review",
            "accept",
            "--id",
            "rw-cli-propose",
            "--reviewer",
            "pending-scholar",
            "--json",
        ],
    );
    assert!(out.status.success(), "accept --json: {}", String::from_utf8_lossy(&out.stderr));
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).expect("one JSON document");
    assert_eq!(doc["assertion"]["decision"], "accepted", "{doc}");
    assert_eq!(doc["assertion"]["reviewer"], "pending-scholar", "{doc}");
    assert!(doc["assertion"]["decided_at"].is_string(), "{doc}");

    // Unknown assertion ids are not-found; empty reviewers are validation.
    let out = qai_out(
        guard.dir.path(),
        &[
            "quran",
            "graph",
            "review",
            "accept",
            "--id",
            "rw-no-such",
            "--reviewer",
            "pending-scholar",
        ],
    );
    assert_eq!(out.status.code(), Some(5), "unknown assertion is not found");
    let out = qai_out(
        guard.dir.path(),
        &["quran", "graph", "review", "accept", "--id", "rw-cli-propose", "--reviewer", ""],
    );
    assert_eq!(out.status.code(), Some(3), "empty reviewer is a validation error");

    // Every decision above emitted audit events on the same database.
    let out = qai_out(guard.dir.path(), &["audit", "list"]);
    assert!(out.status.success(), "audit list: {}", String::from_utf8_lossy(&out.stderr));
    let human = String::from_utf8_lossy(&out.stdout);
    assert!(human.contains("graph_assertion_proposed"), "{human}");
    assert!(human.contains("graph_assertion_decided"), "{human}");
}

/// Phase 2 six-family integrity surface (D-10/D-11): host-backed segments.
#[test]
fn quran_verify_snapshots() {
    let guard = ServeGuard::start();
    guard.run_segments(&["tests/quran/verify_s1.trycmd", "tests/quran/verify_s2.trycmd"]);
}

/// Phase 2 operator reference-corpus path (QV-015, ADR-0114): host-backed
/// segments; the mismatch import queues and the host records the failure.
#[test]
fn quran_reference_snapshots() {
    let guard = ServeGuard::start();
    guard.run_segments(&[
        "tests/quran/reference_s1.trycmd",
        "tests/quran/reference_s2.trycmd",
        "tests/quran/reference_s3.trycmd",
        "tests/quran/reference_s4.trycmd",
        "tests/quran/reference_s5.trycmd",
        "tests/quran/reference_s6.trycmd",
    ]);
}

/// Phase 2 edition identity/primary/license operator surface (D-07):
/// host-backed segments.
#[test]
fn edition_identity_snapshots() {
    let guard = ServeGuard::start();
    guard.run_segments(&[
        "tests/quran/edition_identity_s1.trycmd",
        "tests/quran/edition_identity_s2.trycmd",
    ]);
}

/// Phase 2 quotation hard-failure surface (D-15, QC-07): host-backed segments.
#[test]
fn quran_verify_quotation_snapshots() {
    let guard = ServeGuard::start();
    guard.run_segments(&[
        "tests/quran/verify_quotation_s1.trycmd",
        "tests/quran/verify_quotation_s2.trycmd",
    ]);
}

/// Phase 3 word-family / lemma CLI surface (SC3, G-02, D-10): host-backed
/// segments — the family and lemma commands are reachable and typed-unavailable
/// until a morphology dataset is active.
#[test]
fn quran_family_snapshots() {
    let guard = ServeGuard::start();
    guard.run_segments(&["tests/quran/family_s1.trycmd", "tests/quran/family_s2.trycmd"]);
}

/// Upstream catalog ingestion (metadata-only, no database).
#[test]
fn quran_catalog_snapshots() {
    run_cases("tests/quran/catalog.trycmd");
}

// ─── 01-04-01: real `qai serve` host proof (D-13/D-14/D-16) ───────────
//
// Spawns the built `qai serve` binary on an isolated loopback port against a
// migrated temp database, proves readiness via `/readyz`, seeds a real
// `quran.import` job through SQL, observes the host driving it to
// `Succeeded`, then delivers SIGINT and asserts a joined shutdown: exit 0,
// port released, and later jobs untouched (no detached worker).

const HOST_MANIFEST: &str = include_str!("../../../fixtures/quran/test-edition-min/manifest.json");
const HOST_PRINCIPAL: &str = "00000000-0000-0000-0000-000000000000";
const HOST_CREATED_AT: &str = "2026-09-27T00:00:00Z";

fn migrate_db(dir: &std::path::Path) {
    let mut cfg = config::Config::default();
    cfg.storage.sqlite.path = dir.join("qai.db").display().to_string();
    let migrations =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations/sqlite");
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    runtime.block_on(application::db::migrate_database(&cfg, &migrations)).unwrap();
}

fn qai_out(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_qai"));
    cmd.args(["--data-dir", dir.to_str().unwrap()]);
    cmd.args(args);
    cmd.output().expect("run qai")
}

/// Seed one job row plus the FK parents a `quran.import` handler needs.
/// `payload` is the raw `payload_json`; pass `"{}"` for inert kinds.
fn seed_job(db_path: &std::path::Path, id: &str, kind: &str, payload: &str) {
    let db = db_path.to_str().unwrap().to_string();
    let (id, kind, payload) = (id.to_string(), kind.to_string(), payload.to_string());
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    runtime.block_on(async {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(&db))
            .await
            .unwrap();
        for setup in [
            format!(
                "INSERT OR IGNORE INTO principals (id, kind, display_name, created_at) \
                 VALUES ('{HOST_PRINCIPAL}', 'local_user', 'Serve host test', '{HOST_CREATED_AT}')"
            ),
            "INSERT OR IGNORE INTO sources (id, title, content_type, created_at, updated_at) \
             VALUES ('src-test-edition-min', 'test-edition-min', 'quran_edition', \
             '2026-09-27T00:00:00Z', '2026-09-27T00:00:00Z')"
                .to_string(),
            "INSERT OR IGNORE INTO source_versions \
                (id, source_id, version, schema_version, state, trust_level, \
                 license_status, license_json, created_at) \
             VALUES ('sv-host-1', 'src-test-edition-min', '0.1.0', 1, 'Staged', \
                     'ImportedUnverified', 'Unknown', '{}', '2026-09-27T00:00:00Z')"
                .to_string(),
        ] {
            sqlx::query(&setup).execute(&pool).await.unwrap();
        }
        let key = application::quran::import_idempotency_key("sv-host-1");
        sqlx::query(
            "INSERT INTO jobs (id, kind, payload_json, idempotency_key, state, priority, \
             attempts, max_attempts, available_at, created_at, created_by) \
             VALUES (?, ?, ?, ?, 'Queued', 0, 0, 5, '2026-09-27T00:00:00Z', \
             '2026-09-27T00:00:00Z', '00000000-0000-0000-0000-000000000000')",
        )
        .bind(&id)
        .bind(&kind)
        .bind(&payload)
        .bind(if kind == "quran.import" { key } else { format!("{id}:manual") })
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)").execute(&pool).await.unwrap();
        pool.close().await;
    });
}

fn import_payload() -> String {
    serde_json::json!({
        "run_id": "run-host-1",
        "job_id": "job-host-import",
        "source_version_id": "sv-host-1",
        "adapter": "json",
        "manifest_text": HOST_MANIFEST,
        "declared_manifest_hash": null,
        "invoked_by": HOST_PRINCIPAL,
        "license_status": "Unknown",
        "license_json": "{}",
        "created_at": HOST_CREATED_AT,
        "reference_manifest_text": null,
    })
    .to_string()
}

fn free_port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    port
}

fn is_terminal_state(state: &str) -> bool {
    matches!(state, "Succeeded" | "Failed" | "DeadLettered" | "Cancelled")
}

fn readyz_ok(port: u16) -> bool {
    use std::io::{Read, Write};
    let addr: std::net::SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
    let Ok(mut stream) =
        std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_millis(200))
    else {
        return false;
    };
    stream.set_read_timeout(Some(std::time::Duration::from_secs(2))).ok();
    if write!(stream, "GET /readyz HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
        .is_err()
    {
        return false;
    }
    let mut buf = Vec::new();
    if stream.read_to_end(&mut buf).is_err() {
        return false;
    }
    let text = String::from_utf8_lossy(&buf);
    text.contains("200") && text.contains("ready")
}

/// A live `qai serve` child with its tempdir and port. `Drop` SIGKILLs and
/// reaps survivors so no test leaves a server behind; the test itself asserts
/// the graceful SIGINT exit before the guard drops.
struct ServeGuard {
    dir: tempfile::TempDir,
    port: u16,
    child: Option<std::process::Child>,
}

impl ServeGuard {
    /// Migrate a temp database and start a real `qai serve` child on an
    /// isolated loopback port, waiting for `/readyz`. Seeds nothing: callers
    /// enqueue what they need (task 1 seeds its proof job; task-3 flows
    /// import through the CLI).
    fn start() -> Self {
        let dir = tempfile::tempdir().unwrap();
        migrate_db(dir.path());
        let port = free_port();
        let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_qai"));
        cmd.args(["--data-dir", dir.path().to_str().unwrap()]);
        cmd.args(["serve", "--bind", &format!("127.0.0.1:{port}")]);
        cmd.stdout(std::process::Stdio::null()).stderr(std::process::Stdio::piped());
        let child = cmd.spawn().expect("spawn qai serve");
        let guard = Self { dir, port, child: Some(child) };
        let start = std::time::Instant::now();
        let budget = std::time::Duration::from_secs(20);
        while !readyz_ok(guard.port) {
            assert!(
                start.elapsed() < budget,
                "qai serve never became ready on {port}",
                port = guard.port
            );
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        guard
    }

    fn db_path(&self) -> std::path::PathBuf {
        self.dir.path().join("qai.db")
    }

    /// Run trycmd segment files in order against this guard's database, with
    /// host-backed synchronization after every segment: a queued import
    /// reaches its terminal state before the next segment's dependent
    /// commands run (D-13). trycmd has no wait primitive, so one-shot files
    /// are split at import boundaries instead.
    fn run_segments(&self, segments: &[&str]) {
        for segment in segments {
            {
                trycmd::TestCases::new()
                    .default_bin_name("qai")
                    .env("QAI_DATA_DIR", self.dir.path().to_str().unwrap())
                    .case("tests/quran/*.toml")
                    .case(segment);
            }
            self.wait_all_imports_terminal(std::time::Duration::from_secs(120));
        }
    }

    /// Poll the existing application read path until every `quran.import` job
    /// is terminal. Gentle polling: each check is cheap and read-only, and
    /// hammering the single write connection starves a running import.
    fn wait_all_imports_terminal(&self, budget: std::time::Duration) {
        let path = self.db_path();
        let path = path.to_str().unwrap();
        let start = std::time::Instant::now();
        loop {
            let runtime =
                tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
            let page = runtime.block_on(application::db::list_jobs(path)).expect("list jobs");
            let items = page["items"].as_array().cloned().unwrap_or_default();
            let mut pending = Vec::new();
            for item in &items {
                if item["kind"] == "quran.import"
                    && !is_terminal_state(item["state"].as_str().unwrap_or_default())
                {
                    pending.push(format!(
                        "{}={}",
                        item["id"].as_str().unwrap_or("?"),
                        item["state"].as_str().unwrap_or("?")
                    ));
                }
            }
            if pending.is_empty() {
                return;
            }
            assert!(
                start.elapsed() < budget,
                "queued imports not terminal within {budget:?}: {pending:?}"
            );
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    fn job_state(&self, id: &str) -> String {
        let out = qai_out(self.dir.path(), &["job", "show", id, "--json"]);
        assert!(out.status.success(), "job show {id}: {}", String::from_utf8_lossy(&out.stderr));
        serde_json::from_slice::<serde_json::Value>(&out.stdout).unwrap()["state"]
            .as_str()
            .unwrap()
            .to_string()
    }

    fn wait_job_terminal(&self, id: &str, budget: std::time::Duration) -> String {
        let start = std::time::Instant::now();
        loop {
            let state = self.job_state(id);
            if ["Succeeded", "Failed", "DeadLettered", "Cancelled"].contains(&state.as_str()) {
                return state;
            }
            assert!(
                start.elapsed() < budget,
                "job {id} not terminal within {budget:?} (state {state})"
            );
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    /// Deliver SIGINT and wait for the joined exit. Returns the exit status.
    fn shutdown_graceful(&mut self, budget: std::time::Duration) -> std::process::ExitStatus {
        let child = self.child.as_mut().expect("serve child running");
        let pid = child.id();
        let status = std::process::Command::new("kill")
            .arg("-INT")
            .arg(pid.to_string())
            .status()
            .expect("kill -INT the serve child");
        assert!(status.success(), "kill -INT {pid}");
        let start = std::time::Instant::now();
        loop {
            if let Some(status) = child.try_wait().expect("try_wait the serve child") {
                return status;
            }
            assert!(start.elapsed() < budget, "serve did not exit within {budget:?} after SIGINT");
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    fn stderr(&mut self) -> String {
        let mut text = String::new();
        if let Some(mut child) = self.child.take() {
            if let Some(mut pipe) = child.stderr.take() {
                use std::io::Read;
                let _ = pipe.read_to_string(&mut text);
            }
            let _ = child.try_wait();
            self.child = Some(child);
        }
        text
    }
}

impl Drop for ServeGuard {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

#[test]
fn serve_hosts_the_worker_and_shuts_down_joined() {
    let mut guard = ServeGuard::start();
    // The task-1 proof job is seeded explicitly: flow guards start bare so
    // CLI-driven imports never collide with a pre-seeded edition.
    seed_job(&guard.db_path(), "job-host-import", "quran.import", &import_payload());

    // The host owns the seeded import: exactly one claimed-and-terminal job,
    // with no one-shot drain involved.
    assert_eq!(
        guard.wait_job_terminal("job-host-import", std::time::Duration::from_secs(90)),
        "Succeeded"
    );
    let out = qai_out(guard.dir.path(), &["job", "list", "--json"]);
    assert!(out.status.success());
    let items = serde_json::from_slice::<serde_json::Value>(&out.stdout).unwrap()["items"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert_eq!(items.len(), 1, "exactly one job: {items:?}");
    assert_eq!(items[0]["state"], "Succeeded");

    // Staging landed through the host: the edition validates from storage.
    let out = qai_out(guard.dir.path(), &["quran", "validate", "test-edition-min@0.1.0"]);
    assert!(
        out.status.success(),
        "staged edition validates: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // Signal-driven joined shutdown: SIGINT exits 0 after the worker joins.
    let status = guard.shutdown_graceful(std::time::Duration::from_secs(15));
    assert!(status.success(), "serve exits 0 on SIGINT (stderr: {})", guard.stderr());

    // The port is released and no detached worker survives: a later job stays
    // queued well past the host poll interval.
    assert!(
        std::net::TcpStream::connect(format!("127.0.0.1:{}", guard.port)).is_err(),
        "serve port released after shutdown"
    );
    seed_job(&guard.dir.path().join("qai.db"), "job-host-late", "system.noop_test", "{}");
    std::thread::sleep(std::time::Duration::from_secs(2));
    assert_eq!(guard.job_state("job-host-late"), "Queued");
}

#[test]
fn serve_refuses_without_a_migrated_database() {
    let dir = tempfile::tempdir().unwrap();
    let out = qai_out(dir.path(), &["serve", "--bind", "127.0.0.1:18737"]);
    assert_eq!(out.status.code(), Some(3), "exit VALIDATION");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("qai db migrate"), "names the remedy: {err}");
    assert!(!dir.path().join("qai.db").exists(), "serve creates no state");
}

#[test]
fn serve_still_refuses_non_loopback_binds() {
    let dir = tempfile::tempdir().unwrap();
    let out = qai_out(dir.path(), &["serve", "--bind", "0.0.0.0:18738"]);
    assert_eq!(out.status.code(), Some(4), "exit POLICY");
}

// ─── 01-04-02: enqueue-only import (D-13) ─────────────────────────────
//
// Without any host running, `qai quran import` validates, persists a queued
// `quran.import` job, and returns its id in `Queued` state. The test fails if
// the one-shot process constructs or drains a worker: then the job would be
// terminal and the edition already staged.

#[test]
fn enqueue_only_import_is_queued_without_host() {
    let dir = tempfile::tempdir().unwrap();
    migrate_db(dir.path());
    // No `qai serve` child exists anywhere in this test.
    let manifest = "../../fixtures/quran/test-edition-min/manifest.json";

    let out = qai_out(dir.path(), &["quran", "import", manifest, "--json"]);
    assert!(out.status.success(), "import enqueues: {}", String::from_utf8_lossy(&out.stderr));
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).expect("one JSON document");
    assert_eq!(doc["kind"], "quran.import", "{doc}");
    assert_eq!(doc["state"], "Queued", "{doc}");
    let job_id = doc["job_id"].as_str().expect("durable job id");
    assert!(!job_id.is_empty());
    assert!(doc["max_attempts"].as_u64().unwrap_or(0) >= 1, "retry metadata: {doc}");
    assert!(
        doc["inspect"].as_str().unwrap_or_default().contains(job_id),
        "inspect command names the job: {doc}"
    );

    // The job is inspectable through the existing read path and still queued:
    // no worker ran inside the one-shot process.
    let out = qai_out(dir.path(), &["job", "show", job_id, "--json"]);
    assert!(out.status.success());
    let shown: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(shown["state"], "Queued", "{shown}");

    // Nothing is staged until a host processes the job.
    let out = qai_out(dir.path(), &["quran", "validate", "test-edition-min@0.1.0"]);
    assert_eq!(out.status.code(), Some(5), "unstaged edition is not found");

    // Human wording reports queued state and never claims terminal staging.
    // A second import uses the v2 manifest: re-importing the same
    // `slug@version` is a catalog conflict (UNIQUE(source_id, version),
    // pre-existing on the sync path — the setup seam is untouched here).
    let out =
        qai_out(dir.path(), &["quran", "import", "../../fixtures/quran/test-edition-min-v2.json"]);
    assert!(
        out.status.success(),
        "second import enqueues: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let human = String::from_utf8_lossy(&out.stdout);
    assert!(human.contains("queued"), "queued wording: {human}");
    assert!(human.contains("Queued"), "queued state: {human}");
    // The human import mints its own job; assert the shape, not the id.
    assert!(human.contains("job"), "names the queued job: {human}");
    assert!(human.contains("qai job show"), "points at inspection: {human}");
    assert!(!human.contains("to Staged"), "no terminal claim: {human}");
}
