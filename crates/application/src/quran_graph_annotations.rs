//! Assertion authority plus review state machine (Phase 4, D-06/D-07/D-08).
//!
//! [`propose`] records a manual typed edge (layer `B`), [`suggest`] queues an
//! algorithmic edge (layer `D`, algorithm/version/confidence required by the
//! 0022 `CHECK`), and [`accept`]/[`reject`]/[`dispute`]/[`correct`] decide with
//! reviewer plus timestamp (required by the 0022 `CHECK`). [`correct`] inserts
//! a NEW assertion row carrying `supersedes_id` while the old row keeps its
//! identity and moves to `Superseded` — never an update in place.
//!
//! Every write commits assertion plus provenance plus audit plus outbox rows in
//! ONE sqlx transaction (the `quran_forms` single-seam shape). Graph tables
//! have no `storage` repository seam, so — like
//! [`crate::quran_graph_build`] — writes go through a dedicated short-lived
//! pool on the same database file; the audit chain linkage and the
//! provenance/outbox row shapes mirror `storage-sqlite` and `audit_bridge`
//! exactly so the rows verify under the production readers. Assertion IDs are
//! content-addressed on conflict: re-proposing the identical claim reuses the
//! existing row instead of conflicting, while a different claim under a taken
//! ID is a typed rejection.
//!
//! Adjacency derives strictly from effective assertions
//! ([`Assertion::is_effective`]): rejected and superseded rows vanish from
//! traversal and export while retained for audit. The store adapter
//! ([`crate::quran_graph_store`]) already consults `is_effective` plus
//! `AuthzScope` during expansion; [`visible_export_sets`] is the matching
//! pre-serialization policy filter for export surfaces (plan 04-03 consumes
//! it).

use std::collections::{HashMap, HashSet};

use quran_graph::{
    Assertion, AssertionDecision, AssertionKind, GraphEdge, GraphError, GraphNode, ProvenanceLayer,
    export::{assertion_allowlist_predicate, retain_visible},
    is_allowed_edge,
};

/// Scope string for the outbox/generation rows written beside assertions.
const OUTBOX_SCOPE: &str = "quran-graph";

/// Subject URN prefix for assertion authority rows.
fn assertion_urn(id: &str) -> String {
    format!("quran-graph-assertion:{id}")
}

/// Reviewer plus timestamp recorded on every decided row (D-06).
#[derive(Debug, Clone)]
pub struct DecideInput {
    /// Assertion ID to decide.
    pub id: String,
    /// Reviewer identity (non-empty; never invented).
    pub reviewer: String,
    /// Decision timestamp; defaults to now when absent.
    pub decided_at: Option<String>,
    /// Recording operator principal (owns the `created_by`/`reviewed_by`
    /// foreign keys; the scholarly attribution names `reviewer`).
    pub invoked_by: String,
}

/// Manual typed edge (layer `B`) with PRD section-10.3 seven-field provenance.
#[derive(Debug, Clone)]
pub struct ProposeInput {
    /// Caller assertion ID; generated when absent.
    pub id: Option<String>,
    /// Assertion family.
    pub kind: AssertionKind,
    /// Edge source stable ID.
    pub src: String,
    /// Allowlisted edge predicate.
    pub edge: String,
    /// Edge destination stable ID.
    pub dst: String,
    /// Supporting evidence (must be a JSON object).
    pub evidence: serde_json::Value,
    /// PRD 10.3 `source_id`.
    pub source_id: String,
    /// PRD 10.3 `source_location`.
    pub source_location: String,
    /// PRD 10.3 `author_or_algorithm`: the creating human.
    pub author: String,
    /// Recording operator principal (owns the `created_by` foreign key; the
    /// scholarly attribution names `author`).
    pub invoked_by: String,
    /// Authority scope: projection family.
    pub projection_id: String,
    /// Authority scope: edition.
    pub edition_id: String,
    /// Authority scope: dataset (empty when unscoped).
    pub dataset_scope: String,
}

/// Algorithmic edge proposal (layer `D`) queued for human review (D-06).
///
/// Suggestions stay labeled pending in every surface until a recorded human
/// decision accepts them; nothing here can self-promote.
#[derive(Debug, Clone)]
pub struct SuggestInput {
    /// Caller assertion ID; generated when absent.
    pub id: Option<String>,
    /// Assertion family.
    pub kind: AssertionKind,
    /// Edge source stable ID.
    pub src: String,
    /// Allowlisted edge predicate.
    pub edge: String,
    /// Edge destination stable ID.
    pub dst: String,
    /// Supporting evidence shown in the review queue (must be a JSON object).
    pub evidence: serde_json::Value,
    /// PRD 10.3 `source_id`.
    pub source_id: String,
    /// PRD 10.3 `source_location`.
    pub source_location: String,
    /// PRD 10.3 `author_or_algorithm`: producing algorithm.
    pub algorithm: String,
    /// PRD 10.3 `version`: algorithm version.
    pub algorithm_version: String,
    /// PRD 10.3 `confidence`: opaque attributed metadata (never a threshold).
    pub confidence: f64,
    /// Recording operator principal (owns the `created_by` foreign key; the
    /// computational attribution names `algorithm`).
    pub invoked_by: String,
    /// Authority scope: projection family.
    pub projection_id: String,
    /// Authority scope: edition.
    pub edition_id: String,
    /// Authority scope: dataset (empty when unscoped).
    pub dataset_scope: String,
}

/// Correction of a decided or pending assertion (D-07).
///
/// Unset claim fields default to the old row's values; kind and provenance
/// layer are inherited from the old row (layer-`D` corrections keep the old
/// algorithm triple, since the correction re-targets the claim, not the
/// method).
#[derive(Debug, Clone)]
pub struct CorrectInput {
    /// Assertion ID being corrected.
    pub id: String,
    /// Correcting reviewer (non-empty; never invented).
    pub reviewer: String,
    /// Decision timestamp; defaults to now when absent.
    pub decided_at: Option<String>,
    /// Corrected source (defaults to the old claim's).
    pub src: Option<String>,
    /// Corrected predicate (defaults to the old claim's).
    pub edge: Option<String>,
    /// Corrected destination (defaults to the old claim's).
    pub dst: Option<String>,
    /// Corrected evidence (defaults to the old row's).
    pub evidence: Option<serde_json::Value>,
    /// Corrected source location (defaults to the old row's).
    pub source_location: Option<String>,
    /// Recording operator principal (owns the `created_by`/`reviewed_by`
    /// foreign keys; the scholarly attribution names `reviewer`).
    pub invoked_by: String,
}

/// One committed annotation write.
#[derive(Debug, Clone)]
pub struct AnnotationReport {
    /// The written (or reused) assertion row.
    pub assertion: Assertion,
    /// Whether an adjacency row points at the assertion in the active build.
    pub edge_staged: bool,
    /// Active build row the edge staged into, if any.
    pub build_row_id: Option<String>,
    /// Whether the assertion row already existed with identical content
    /// (content-addressed reuse: no new provenance/audit/outbox rows).
    pub reused: bool,
}

fn build_failed(stage: &str, detail: String) -> GraphError {
    GraphError::BuildFailed { stage: stage.to_string(), detail }
}

fn rejected(detail: String) -> GraphError {
    GraphError::PatternRejected { detail }
}

fn now_rfc3339() -> String {
    domain::Timestamp::now().to_string()
}

/// Provenance-record shape for one annotation write (migration 0003 domain).
///
/// Principal foreign keys (`created_by`, `reviewed_by`) name the recording
/// operator (`invoked_by`); scholarly attribution (reviewer/author names)
/// travels in the attribution JSON, the assertion row, and the audit event.
struct ProvenanceShape<'a> {
    layer: &'a str,
    kind: &'a str,
    trust: &'a str,
    status: &'a str,
    reviewed: bool,
}

/// Ensure the recording operator exists before the write transaction (the
/// `created_by`/`reviewed_by` foreign keys must resolve; upsert is
/// idempotent, mirroring `quran_forms` principal handling).
async fn ensure_operator(db_path: &str, invoked_by: &str, at: &str) -> Result<(), GraphError> {
    if std::fs::symlink_metadata(db_path).is_err() {
        return Err(build_failed(
            "open",
            format!("no database at '{db_path}'; run `qai db migrate`"),
        ));
    }
    let db = storage_sqlite::SqliteDatabase::new(db_path, 4, true)
        .await
        .map_err(|error| build_failed("open", error.to_string()))?;
    crate::quran::ensure_principal(&db, invoked_by, "graph review operator", at)
        .await
        .map_err(|error| build_failed("principal", error.to_string()))
}

fn open_pool(db_path: &str) -> Result<sqlx::sqlite::SqlitePool, GraphError> {
    let options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(false)
        .foreign_keys(true)
        .busy_timeout(std::time::Duration::from_secs(5));
    Ok(sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect_lazy_with(options))
}

/// Open the pool synchronously for callers without a runtime yet.
async fn pool(db_path: &str) -> Result<sqlx::sqlite::SqlitePool, GraphError> {
    let pool = open_pool(db_path)?;
    pool.acquire().await.map_err(|error| build_failed("open", error.to_string()))?;
    Ok(pool)
}

fn is_unique_violation(error: &sqlx::Error) -> bool {
    match error {
        sqlx::Error::Database(db) => {
            let code = db.code().unwrap_or_default();
            code == "2067" || code == "1555" || db.message().contains("UNIQUE")
        }
        _ => false,
    }
}

/// Parse an assertion kind from its stored/CLI spelling.
pub fn parse_assertion_kind(raw: &str) -> Result<AssertionKind, GraphError> {
    match raw {
        "annotation" => Ok(AssertionKind::Annotation),
        "concept_link" => Ok(AssertionKind::ConceptLink),
        "entity_link" => Ok(AssertionKind::EntityLink),
        "family_link" => Ok(AssertionKind::FamilyLink),
        "import" => Ok(AssertionKind::Import),
        other => Err(rejected(format!("unknown assertion kind '{other}'"))),
    }
}

fn decision_str(decision: AssertionDecision) -> &'static str {
    match decision {
        AssertionDecision::Pending => "pending",
        AssertionDecision::Accepted => "accepted",
        AssertionDecision::Rejected => "rejected",
        AssertionDecision::Superseded => "superseded",
        AssertionDecision::Disputed => "disputed",
    }
}

fn parse_decision(raw: &str) -> Result<AssertionDecision, GraphError> {
    match raw {
        "pending" => Ok(AssertionDecision::Pending),
        "accepted" => Ok(AssertionDecision::Accepted),
        "rejected" => Ok(AssertionDecision::Rejected),
        "superseded" => Ok(AssertionDecision::Superseded),
        "disputed" => Ok(AssertionDecision::Disputed),
        other => Err(build_failed("read", format!("unknown assertion decision '{other}'"))),
    }
}

fn kind_str(kind: AssertionKind) -> &'static str {
    match kind {
        AssertionKind::Annotation => "annotation",
        AssertionKind::ConceptLink => "concept_link",
        AssertionKind::EntityLink => "entity_link",
        AssertionKind::FamilyLink => "family_link",
        AssertionKind::Import => "import",
    }
}

fn layer_str(layer: ProvenanceLayer) -> &'static str {
    match layer {
        ProvenanceLayer::B => "B",
        ProvenanceLayer::D => "D",
    }
}

fn parse_json_or_null(raw: Option<String>) -> serde_json::Value {
    match raw {
        Some(text) if !text.trim().is_empty() => serde_json::from_str(&text).unwrap_or_default(),
        _ => serde_json::Value::Null,
    }
}

fn row_to_assertion(row: &sqlx::sqlite::SqliteRow) -> Result<Assertion, GraphError> {
    use sqlx::Row as _;
    Ok(Assertion {
        id: row.get("id"),
        kind: parse_assertion_kind(row.get("assertion_kind")).map_err(|_| {
            build_failed("read", "unknown assertion kind in authority table".into())
        })?,
        claim: parse_json_or_null(row.get("claim_json")),
        evidence: parse_json_or_null(row.get("evidence_json")),
        source_location: row.get("source_location"),
        reviewer: row.get("reviewer"),
        decision: parse_decision(row.get("decision"))?,
        decided_at: row.get("decided_at"),
        supersedes_id: row.get("supersedes_id"),
        layer: match row.get::<String, _>("provenance_layer").as_str() {
            "B" => ProvenanceLayer::B,
            "D" => ProvenanceLayer::D,
            other => {
                return Err(build_failed(
                    "read",
                    format!("unknown provenance layer '{other}' in authority table"),
                ));
            }
        },
        algorithm: row.get("algorithm"),
        algorithm_version: row.get("algorithm_version"),
        confidence: row.get::<Option<f64>, _>("confidence"),
        created_at: row.get("created_at"),
    })
}

async fn fetch_assertion(
    executor: &mut sqlx::SqliteConnection,
    id: &str,
) -> Result<Option<Assertion>, GraphError> {
    let row = sqlx::query(
        "SELECT id, assertion_kind, claim_json, evidence_json, source_location,
                reviewer, decision, decided_at, supersedes_id, provenance_layer,
                algorithm, algorithm_version, confidence, created_at
         FROM graph_assertions WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&mut *executor)
    .await
    .map_err(|error| build_failed("read", error.to_string()))?;
    row.map(|row| row_to_assertion(&row)).transpose()
}

/// Edge triple plus PRD 10.3 seven-field provenance for one claim payload.
struct ClaimParts<'a> {
    src: &'a str,
    edge: &'a str,
    dst: &'a str,
    source_id: &'a str,
    source_location: &'a str,
    author_or_algorithm: &'a str,
    version: Option<&'a str>,
    confidence: Option<f64>,
    verification_status: &'a str,
    created_at: &'a str,
}

/// Claim payload: the edge triple plus the PRD 10.3 seven-field provenance.
///
/// `verification_status` mirrors the review decision (`pending` →
/// `unverified`, `accepted` → `verified`, otherwise the decision name), so
/// every surface can label unverified suggestions without joining tables.
fn claim_json(parts: ClaimParts<'_>) -> serde_json::Value {
    serde_json::json!({
        "src": parts.src,
        "edge": parts.edge,
        "dst": parts.dst,
        "provenance": {
            "source_id": parts.source_id,
            "source_location": parts.source_location,
            "author_or_algorithm": parts.author_or_algorithm,
            "version": parts.version,
            "confidence": parts.confidence,
            "verification_status": parts.verification_status,
            "created_at": parts.created_at,
        },
    })
}

fn validate_edge(src: &str, edge: &str, dst: &str) -> Result<(), GraphError> {
    if src.trim().is_empty() || dst.trim().is_empty() {
        return Err(rejected("edge endpoints must both be non-empty stable IDs".to_string()));
    }
    if src == dst {
        return Err(rejected(format!("refuses self-loop edge on '{src}'")));
    }
    if !is_allowed_edge(edge) {
        return Err(rejected(format!("unknown edge predicate '{edge}'")));
    }
    Ok(())
}

fn validate_scope(projection_id: &str, edition_id: &str) -> Result<(), GraphError> {
    if projection_id.trim().is_empty() {
        return Err(rejected("projection_id must be non-empty".to_string()));
    }
    if edition_id.trim().is_empty() {
        return Err(rejected("edition_id must be non-empty".to_string()));
    }
    Ok(())
}

fn validate_evidence(evidence: &serde_json::Value) -> Result<(), GraphError> {
    if !evidence.is_object() {
        return Err(rejected("evidence must be a JSON object".to_string()));
    }
    Ok(())
}

fn validate_reviewer(reviewer: &str) -> Result<(), GraphError> {
    if reviewer.trim().is_empty() {
        return Err(rejected("reviewer must be non-empty (never invented)".to_string()));
    }
    Ok(())
}

/// Mirror of `audit_bridge` linkage arithmetic for the raw transaction:
/// sequence is max-plus-one, previous hash chains, genesis when empty.
fn genesis_hash() -> domain::ContentHash {
    domain::ContentHash { algorithm: domain::HashAlgorithm::Sha256, hex: "00".repeat(32) }
}

fn hash_str(hash: &domain::ContentHash) -> String {
    let algorithm = match hash.algorithm {
        domain::HashAlgorithm::Sha256 => "sha256",
        domain::HashAlgorithm::Blake3 => "blake3",
    };
    format!("{algorithm}:{}", hash.hex)
}

fn action_str(action: &audit::AuditAction) -> String {
    serde_json::to_value(action)
        .ok()
        .and_then(|value| value.as_str().map(str::to_string))
        .unwrap_or_else(|| "unknown".to_string())
}

async fn append_audit(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    actor_name: &str,
    action: audit::AuditAction,
    subject_urn: &str,
    after: serde_json::Value,
    at: &str,
) -> Result<(), GraphError> {
    let max: Option<i64> = sqlx::query_scalar("SELECT MAX(sequence) FROM audit_events")
        .fetch_one(&mut **tx)
        .await
        .map_err(|error| build_failed("audit", error.to_string()))?;
    let sequence = max.unwrap_or(0) + 1;
    let prev_hash = if sequence == 1 {
        genesis_hash()
    } else {
        let raw: Option<String> =
            sqlx::query_scalar("SELECT chain_hash FROM audit_events WHERE sequence = ?")
                .bind(sequence - 1)
                .fetch_optional(&mut **tx)
                .await
                .map_err(|error| build_failed("audit", error.to_string()))?
                .flatten();
        match raw.and_then(|text| {
            text.split_once(':').and_then(|(algorithm, hex)| match algorithm {
                "sha256" => {
                    domain::ContentHash::try_new(domain::HashAlgorithm::Sha256, hex.to_string())
                        .ok()
                }
                "blake3" => {
                    domain::ContentHash::try_new(domain::HashAlgorithm::Blake3, hex.to_string())
                        .ok()
                }
                _ => None,
            })
        }) {
            Some(hash) => hash,
            None => genesis_hash(),
        }
    };
    let after_json = serde_json::to_string(&after).unwrap_or_default();
    let event = audit::AuditEvent {
        id: uuid::Uuid::new_v4()
            .to_string()
            .parse()
            .map_err(|error| build_failed("audit", format!("synthetic audit id: {error}")))?,
        sequence: sequence as u64,
        occurred_at: at.parse().map_err(|_| build_failed("audit", "bad timestamp".into()))?,
        actor: audit::Actor::System { name: actor_name.to_string() },
        action,
        subject: domain::SubjectRef(subject_urn.to_string()),
        outcome: audit::AuditOutcome::Allowed,
        reason: None,
        before: None,
        after: Some(after),
        request_id: None,
        prev_chain_hash: prev_hash.clone(),
        chain_hash: genesis_hash(),
    };
    let chain_hash = audit::HashChainWriter::compute_chain_hash(&prev_hash, &event);
    sqlx::query(
        "INSERT INTO audit_events
            (id, sequence, occurred_at, actor_kind, actor_id, action, subject_urn, outcome,
             reason, before_json, after_json, request_id, prev_chain_hash, chain_hash)
         VALUES (?, ?, ?, 'system', ?, ?, ?, 'allowed', NULL, NULL, ?, NULL, ?, ?)",
    )
    .bind(event.id.to_string())
    .bind(sequence)
    .bind(at)
    .bind(actor_name)
    .bind(action_str(&event.action))
    .bind(subject_urn)
    .bind(after_json)
    .bind(hash_str(&prev_hash))
    .bind(hash_str(&chain_hash))
    .execute(&mut **tx)
    .await
    .map_err(|error| build_failed("audit", error.to_string()))?;
    Ok(())
}

/// One provenance write beside an annotation write (migration 0003 domain).
struct ProvenanceWrite<'a> {
    id: String,
    shape: ProvenanceShape<'a>,
    subject_urn: String,
    attribution: serde_json::Value,
    confidence: Option<f64>,
    actor: &'a str,
    at: String,
}

async fn insert_provenance(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    write: ProvenanceWrite<'_>,
) -> Result<(), GraphError> {
    let result = sqlx::query(
        "INSERT INTO provenance_records
            (id, layer, subject_urn, attribution_kind, attribution_json, source_version_id,
             trust_level, verification_status, confidence, versions_json,
             created_at, created_by, reviewed_by)
         VALUES (?, ?, ?, ?, ?, NULL, ?, ?, ?, '{}', ?, ?, ?)",
    )
    .bind(&write.id)
    .bind(write.shape.layer)
    .bind(&write.subject_urn)
    .bind(write.shape.kind)
    .bind(write.attribution.to_string())
    .bind(write.shape.trust)
    .bind(write.shape.status)
    .bind(write.confidence)
    .bind(&write.at)
    .bind(write.actor)
    .bind(write.shape.reviewed.then_some(write.actor))
    .execute(&mut **tx)
    .await;
    match result {
        Ok(_) => Ok(()),
        // Content-addressed reuse: the identical derivation already recorded
        // its provenance; a retry reuses it instead of conflicting.
        Err(error) if is_unique_violation(&error) => Ok(()),
        Err(error) => Err(build_failed("provenance", error.to_string())),
    }
}

async fn enqueue_outbox(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    operation: &str,
    subject_urn: &str,
    idempotency_key: &str,
    payload: &serde_json::Value,
    reason: &str,
    at: &str,
) -> Result<(), GraphError> {
    let current: Option<i64> = sqlx::query_scalar(
        "SELECT number FROM corpus_generations WHERE scope = ? ORDER BY number DESC LIMIT 1",
    )
    .bind(OUTBOX_SCOPE)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|error| build_failed("outbox", error.to_string()))?
    .flatten();
    let next = current.unwrap_or(0) + 1;
    let generation_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO corpus_generations (id, scope, number, reason, created_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&generation_id)
    .bind(OUTBOX_SCOPE)
    .bind(next)
    .bind(reason)
    .bind(at)
    .execute(&mut **tx)
    .await
    .map_err(|error| build_failed("outbox", error.to_string()))?;
    sqlx::query(
        "INSERT INTO outbox_events
            (id, scope, target_generation, operation, subject_urn, idempotency_key,
             payload_json, state, attempts, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, 'Pending', 0, ?)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(OUTBOX_SCOPE)
    .bind(&generation_id)
    .bind(operation)
    .bind(subject_urn)
    .bind(idempotency_key)
    .bind(payload.to_string())
    .bind(at)
    .execute(&mut **tx)
    .await
    .map_err(|error| build_failed("outbox", error.to_string()))?;
    Ok(())
}

async fn active_build_row(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    projection_id: &str,
) -> Result<Option<String>, GraphError> {
    let row: Option<String> = sqlx::query_scalar(
        "SELECT id FROM graph_projections
         WHERE projection_id = ? AND status = 'active'
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(projection_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|error| build_failed("stage", error.to_string()))?;
    Ok(row)
}

/// Authority linkage for a new assertion row (D-08, 0022 scoping).
///
/// The 0022 schema retains the build-row foreign key for staging linkage
/// while authority scopes by (`projection_id`, `edition_id`,
/// `dataset_scope`): new rows link the active row, else the family's latest
/// row (any status — rebuilds retain superseded rows precisely so history
/// keeps its linkage). A family with no build row at all cannot take
/// annotations yet: that is a typed request rejection naming the remedy
/// (build first), never a silent orphan.
async fn authority_build_row(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    projection_id: &str,
) -> Result<String, GraphError> {
    if let Some(active) = active_build_row(tx, projection_id).await? {
        return Ok(active);
    }
    let latest: Option<String> = sqlx::query_scalar(
        "SELECT id FROM graph_projections
         WHERE projection_id = ?
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(projection_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|error| build_failed("annotate", error.to_string()))?;
    latest.ok_or_else(|| {
        rejected(format!(
            "no build row for projection '{projection_id}'; build the projection before annotating"
        ))
    })
}

async fn endpoints_staged(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    build_row_id: &str,
    src: &str,
    dst: &str,
) -> Result<bool, GraphError> {
    for endpoint in [src, dst] {
        let present: Option<String> = sqlx::query_scalar(
            "SELECT stable_id FROM graph_nodes
             WHERE projection_row_id = ? AND stable_id = ? LIMIT 1",
        )
        .bind(build_row_id)
        .bind(endpoint)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|error| build_failed("stage", error.to_string()))?;
        if present.is_none() {
            return Ok(false);
        }
    }
    Ok(true)
}

async fn stage_review_edge(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    build_row_id: &str,
    src: &str,
    edge: &str,
    dst: &str,
    assertion_id: &str,
    at: &str,
) -> Result<(), GraphError> {
    let edge_id = format!("review-{assertion_id}");
    let existing: Option<String> = sqlx::query_scalar("SELECT id FROM graph_edges WHERE id = ?")
        .bind(&edge_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|error| build_failed("stage", error.to_string()))?;
    if existing.is_some() {
        return Ok(());
    }
    sqlx::query(
        "INSERT INTO graph_edges
            (id, projection_row_id, src_stable_id, edge, dst_stable_id,
             assertion_id, budgets_json, attrs_json, created_at)
         VALUES (?, ?, ?, ?, ?, ?, '{}', '{}', ?)",
    )
    .bind(&edge_id)
    .bind(build_row_id)
    .bind(src)
    .bind(edge)
    .bind(dst)
    .bind(assertion_id)
    .bind(at)
    .execute(&mut **tx)
    .await
    .map_err(|error| build_failed("stage", error.to_string()))?;
    Ok(())
}

async fn insert_assertion_row(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    assertion: &Assertion,
    build_row_id: &str,
    projection_id: &str,
    edition_id: &str,
    dataset_scope: &str,
) -> Result<(), GraphError> {
    sqlx::query(
        "INSERT INTO graph_assertions
            (id, projection_row_id, projection_id, edition_id, dataset_scope,
             assertion_kind, claim_json, evidence_json, source_location,
             reviewer, decision, decided_at, supersedes_id,
             provenance_layer, algorithm, algorithm_version, confidence, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&assertion.id)
    .bind(build_row_id)
    .bind(projection_id)
    .bind(edition_id)
    .bind(dataset_scope)
    .bind(kind_str(assertion.kind))
    .bind(assertion.claim.to_string())
    .bind(assertion.evidence.to_string())
    .bind(&assertion.source_location)
    .bind(assertion.reviewer.as_deref())
    .bind(decision_str(assertion.decision))
    .bind(assertion.decided_at.as_deref())
    .bind(assertion.supersedes_id.as_deref())
    .bind(layer_str(assertion.layer))
    .bind(assertion.algorithm.as_deref())
    .bind(assertion.algorithm_version.as_deref())
    .bind(assertion.confidence)
    .bind(&assertion.created_at)
    .execute(&mut **tx)
    .await
    .map_err(|error| build_failed("annotate", error.to_string()))?;
    Ok(())
}

/// One new-assertion write: the row plus its provenance, audit, outbox, and
/// staged edge — a single transaction.
struct WriteRequest<'a> {
    assertion: Assertion,
    projection_id: &'a str,
    edition_id: &'a str,
    dataset_scope: &'a str,
    src: &'a str,
    edge: &'a str,
    dst: &'a str,
    actor_name: &'a str,
    invoked_by: &'a str,
    shape: ProvenanceShape<'a>,
    audit_action: audit::AuditAction,
    outbox_operation: &'a str,
    outbox_reason: &'a str,
}

/// Shared propose/suggest write path: validate, insert, provenance, audit,
/// outbox, edge staging — one transaction.
async fn write_new_assertion(
    db_path: &str,
    request: WriteRequest<'_>,
) -> Result<AnnotationReport, GraphError> {
    let WriteRequest {
        assertion,
        projection_id,
        edition_id,
        dataset_scope,
        src,
        edge,
        dst,
        actor_name,
        invoked_by,
        shape,
        audit_action,
        outbox_operation,
        outbox_reason,
    } = request;
    let at = now_rfc3339();
    ensure_operator(db_path, invoked_by, &at).await?;
    let pool = pool(db_path).await?;
    let mut tx = pool.begin().await.map_err(|error| build_failed("annotate", error.to_string()))?;
    let subject = assertion_urn(&assertion.id);

    // Content-addressed conflict reuse happens BEFORE any write: when the ID
    // already carries the identical claim, the existing row is the same
    // assertion (no new side rows); a different claim under a taken ID is a
    // typed rejection. Volatile fields (`created_at`, `verification_status`)
    // are excluded: identity is the stable triple plus evidence, location,
    // and attribution.
    if let Some(existing) = fetch_assertion(&mut tx, &assertion.id).await? {
        let stable = |record: &Assertion| {
            (
                record.kind,
                record.claim.get("src").cloned().unwrap_or_default(),
                record.claim.get("edge").cloned().unwrap_or_default(),
                record.claim.get("dst").cloned().unwrap_or_default(),
                record.evidence.clone(),
                record.source_location.clone(),
                record.layer,
                record.algorithm.clone(),
                record.algorithm_version.clone(),
                record.confidence,
            )
        };
        let _ = tx.rollback().await;
        if stable(&existing) == stable(&assertion) {
            return Ok(AnnotationReport {
                assertion: existing,
                edge_staged: false,
                build_row_id: None,
                reused: true,
            });
        }
        return Err(rejected(format!(
            "assertion id '{}' already carries a different claim",
            assertion.id
        )));
    }

    let authority_row = authority_build_row(&mut tx, projection_id).await?;
    insert_assertion_row(
        &mut tx,
        &assertion,
        &authority_row,
        projection_id,
        edition_id,
        dataset_scope,
    )
    .await?;

    insert_provenance(
        &mut tx,
        ProvenanceWrite {
            id: format!("graph-prov-{}", assertion.id),
            shape,
            subject_urn: subject.clone(),
            attribution: serde_json::json!({
                "assertion_kind": kind_str(assertion.kind),
                "claim": assertion.claim,
                "evidence": assertion.evidence,
            }),
            confidence: assertion.confidence,
            actor: invoked_by,
            at: at.clone(),
        },
    )
    .await?;
    append_audit(
        &mut tx,
        actor_name,
        audit_action,
        &subject,
        serde_json::json!({
            "assertion_id": assertion.id,
            "decision": decision_str(assertion.decision),
            "projection_id": projection_id,
            "edition_id": edition_id,
        }),
        &at,
    )
    .await?;
    enqueue_outbox(
        &mut tx,
        outbox_operation,
        &subject,
        &format!("graph_assertion:{}", assertion.id),
        &serde_json::json!({"assertion_id": assertion.id}),
        outbox_reason,
        &at,
    )
    .await?;

    let mut report =
        AnnotationReport { assertion, edge_staged: false, build_row_id: None, reused: false };
    if let Some(build_row_id) = active_build_row(&mut tx, projection_id).await?
        && endpoints_staged(&mut tx, &build_row_id, src, dst).await?
    {
        stage_review_edge(&mut tx, &build_row_id, src, edge, dst, &report.assertion.id, &at)
            .await?;
        report.edge_staged = true;
        report.build_row_id = Some(build_row_id);
    }
    tx.commit().await.map_err(|error| build_failed("annotate", error.to_string()))?;
    Ok(report)
}

/// Record a manual typed edge with PRD section-10.3 seven-field provenance
/// (D-06, D-07).
pub async fn propose(db_path: &str, input: ProposeInput) -> Result<AnnotationReport, GraphError> {
    validate_edge(&input.src, &input.edge, &input.dst)?;
    validate_scope(&input.projection_id, &input.edition_id)?;
    validate_evidence(&input.evidence)?;
    validate_reviewer(&input.author)?;
    if input.source_id.trim().is_empty() {
        return Err(rejected("source_id must be non-empty (PRD 10.3)".to_string()));
    }
    if input.source_location.trim().is_empty() {
        return Err(rejected("source_location must be non-empty (PRD 10.3)".to_string()));
    }
    let at = now_rfc3339();
    let assertion = Assertion {
        id: input.id.unwrap_or_else(|| format!("qa-{}", &uuid::Uuid::new_v4().to_string()[..8])),
        kind: input.kind,
        claim: claim_json(ClaimParts {
            src: &input.src,
            edge: &input.edge,
            dst: &input.dst,
            source_id: &input.source_id,
            source_location: &input.source_location,
            author_or_algorithm: &input.author,
            version: None,
            confidence: None,
            verification_status: "unverified",
            created_at: &at,
        }),
        evidence: input.evidence.clone(),
        source_location: input.source_location.clone(),
        reviewer: None,
        decision: AssertionDecision::Pending,
        decided_at: None,
        supersedes_id: None,
        layer: ProvenanceLayer::B,
        algorithm: None,
        algorithm_version: None,
        confidence: None,
        created_at: at,
    };
    write_new_assertion(
        db_path,
        WriteRequest {
            assertion,
            projection_id: &input.projection_id,
            edition_id: &input.edition_id,
            dataset_scope: &input.dataset_scope,
            src: &input.src,
            edge: &input.edge,
            dst: &input.dst,
            actor_name: &input.author,
            invoked_by: &input.invoked_by,
            shape: ProvenanceShape {
                layer: "scholarly_annotation",
                kind: "scholar",
                trust: "HumanAttested",
                status: "unverified",
                reviewed: false,
            },
            audit_action: audit::AuditAction::GraphAssertionProposed,
            outbox_operation: "graph_assertion_written",
            outbox_reason: "graph_assertion_proposed",
        },
    )
    .await
}

/// Queue an algorithmic edge for human review (D-06).
///
/// Layer-`D` rows require algorithm plus version plus confidence (the 0022
/// `CHECK`); the suggestion stays pending — labeled, never verified — until
/// a recorded human decision.
pub async fn suggest(db_path: &str, input: SuggestInput) -> Result<AnnotationReport, GraphError> {
    validate_edge(&input.src, &input.edge, &input.dst)?;
    validate_scope(&input.projection_id, &input.edition_id)?;
    validate_evidence(&input.evidence)?;
    if input.source_id.trim().is_empty() {
        return Err(rejected("source_id must be non-empty (PRD 10.3)".to_string()));
    }
    if input.source_location.trim().is_empty() {
        return Err(rejected("source_location must be non-empty (PRD 10.3)".to_string()));
    }
    if input.algorithm.trim().is_empty() || input.algorithm_version.trim().is_empty() {
        return Err(rejected(
            "layer-D suggestions require algorithm plus algorithm_version (PRD 10.3)".to_string(),
        ));
    }
    if !(0.0..=1.0).contains(&input.confidence) {
        return Err(rejected("confidence must lie in [0,1]".to_string()));
    }
    let at = now_rfc3339();
    let assertion = Assertion {
        id: input.id.unwrap_or_else(|| format!("qa-{}", &uuid::Uuid::new_v4().to_string()[..8])),
        kind: input.kind,
        claim: claim_json(ClaimParts {
            src: &input.src,
            edge: &input.edge,
            dst: &input.dst,
            source_id: &input.source_id,
            source_location: &input.source_location,
            author_or_algorithm: &input.algorithm,
            version: Some(&input.algorithm_version),
            confidence: Some(input.confidence),
            verification_status: "unverified",
            created_at: &at,
        }),
        evidence: input.evidence.clone(),
        source_location: input.source_location.clone(),
        reviewer: None,
        decision: AssertionDecision::Pending,
        decided_at: None,
        supersedes_id: None,
        layer: ProvenanceLayer::D,
        algorithm: Some(input.algorithm.clone()),
        algorithm_version: Some(input.algorithm_version.clone()),
        confidence: Some(input.confidence),
        created_at: at,
    };
    write_new_assertion(
        db_path,
        WriteRequest {
            assertion,
            projection_id: &input.projection_id,
            edition_id: &input.edition_id,
            dataset_scope: &input.dataset_scope,
            src: &input.src,
            edge: &input.edge,
            dst: &input.dst,
            actor_name: &input.algorithm,
            invoked_by: &input.invoked_by,
            shape: ProvenanceShape {
                layer: "computational_annotation",
                kind: "computational",
                trust: "ComputedUnverified",
                status: "unverified",
                reviewed: false,
            },
            audit_action: audit::AuditAction::GraphAssertionProposed,
            outbox_operation: "graph_assertion_written",
            outbox_reason: "graph_assertion_suggested",
        },
    )
    .await
}

fn decided_at_or_now(decided_at: &Option<String>) -> String {
    decided_at.clone().unwrap_or_else(now_rfc3339)
}

/// Shared decide write path: guard the transition, update the row, record
/// decision provenance plus audit plus outbox — one transaction.
async fn decide(
    db_path: &str,
    input: &DecideInput,
    to: AssertionDecision,
    allowed_from: &[AssertionDecision],
) -> Result<AnnotationReport, GraphError> {
    validate_reviewer(&input.reviewer)?;
    let at_probe = now_rfc3339();
    ensure_operator(db_path, &input.invoked_by, &at_probe).await?;
    let pool = pool(db_path).await?;
    let mut probe =
        pool.acquire().await.map_err(|error| build_failed("decide", error.to_string()))?;
    let current = fetch_assertion(&mut probe, &input.id).await?;
    drop(probe);
    let Some(current) = current else {
        return Err(GraphError::UnknownAssertion { assertion: input.id.clone() });
    };
    if !allowed_from.contains(&current.decision) {
        return Err(rejected(format!(
            "cannot move assertion '{}' from {} to {}",
            input.id,
            decision_str(current.decision),
            decision_str(to)
        )));
    }
    let mut tx = pool.begin().await.map_err(|error| build_failed("decide", error.to_string()))?;
    let at = decided_at_or_now(&input.decided_at);
    sqlx::query(
        "UPDATE graph_assertions SET decision = ?, reviewer = ?, decided_at = ? WHERE id = ?",
    )
    .bind(decision_str(to))
    .bind(&input.reviewer)
    .bind(&at)
    .bind(&input.id)
    .execute(&mut *tx)
    .await
    .map_err(|error| build_failed("decide", error.to_string()))?;
    let subject = assertion_urn(&input.id);
    let shape = match to {
        AssertionDecision::Accepted => ProvenanceShape {
            layer: "scholarly_annotation",
            kind: "scholar",
            trust: "HumanAttested",
            status: "human_verified",
            reviewed: true,
        },
        AssertionDecision::Rejected => ProvenanceShape {
            layer: "scholarly_annotation",
            kind: "scholar",
            trust: "HumanAttested",
            status: "rejected",
            reviewed: false,
        },
        AssertionDecision::Disputed => ProvenanceShape {
            layer: "scholarly_annotation",
            kind: "scholar",
            trust: "HumanAttested",
            status: "needs_review",
            reviewed: false,
        },
        _ => ProvenanceShape {
            layer: "scholarly_annotation",
            kind: "scholar",
            trust: "HumanAttested",
            status: "unverified",
            reviewed: false,
        },
    };
    insert_provenance(
        &mut tx,
        ProvenanceWrite {
            id: format!("graph-prov-{}-{}", input.id, decision_str(to)),
            shape,
            subject_urn: subject.clone(),
            attribution: serde_json::json!({
                "decision": decision_str(to),
                "reviewer": input.reviewer,
                "decided_at": at,
                "previous_decision": decision_str(current.decision),
            }),
            confidence: current.confidence,
            actor: &input.invoked_by,
            at: at.clone(),
        },
    )
    .await?;
    append_audit(
        &mut tx,
        &input.reviewer,
        audit::AuditAction::GraphAssertionDecided,
        &subject,
        serde_json::json!({
            "assertion_id": input.id,
            "previous_decision": decision_str(current.decision),
            "decision": decision_str(to),
            "reviewer": input.reviewer,
            "decided_at": at,
        }),
        &at,
    )
    .await?;
    enqueue_outbox(
        &mut tx,
        "graph_review_decided",
        &subject,
        &format!("graph_review:{}:{}:{}", input.id, decision_str(to), uuid::Uuid::new_v4()),
        &serde_json::json!({"assertion_id": input.id, "decision": decision_str(to)}),
        "graph_review_decided",
        &at,
    )
    .await?;
    tx.commit().await.map_err(|error| build_failed("decide", error.to_string()))?;
    let mut conn =
        pool.acquire().await.map_err(|error| build_failed("decide", error.to_string()))?;
    let updated = fetch_assertion(&mut conn, &input.id)
        .await?
        .ok_or_else(|| GraphError::UnknownAssertion { assertion: input.id.clone() })?;
    Ok(AnnotationReport {
        assertion: updated,
        edge_staged: false,
        build_row_id: None,
        reused: false,
    })
}

/// Accept a pending or disputed suggestion into the verified set (D-06).
///
/// The recorded reviewer plus timestamp promote the edge into traversal and
/// export visibility.
pub async fn accept(db_path: &str, input: DecideInput) -> Result<AnnotationReport, GraphError> {
    decide(
        db_path,
        &input,
        AssertionDecision::Accepted,
        &[AssertionDecision::Pending, AssertionDecision::Disputed],
    )
    .await
}

/// Reject a suggestion: the row tombstones immediately (hidden from traversal
/// and export) while retained for audit (D-07/D-08).
pub async fn reject(db_path: &str, input: DecideInput) -> Result<AnnotationReport, GraphError> {
    decide(
        db_path,
        &input,
        AssertionDecision::Rejected,
        &[AssertionDecision::Pending, AssertionDecision::Accepted, AssertionDecision::Disputed],
    )
    .await
}

/// Mark live scholarly disagreement: the assertion stays effective for
/// traversal and export (never tombstoned) while the dispute is visible
/// (D-07).
pub async fn dispute(db_path: &str, input: DecideInput) -> Result<AnnotationReport, GraphError> {
    decide(
        db_path,
        &input,
        AssertionDecision::Disputed,
        &[AssertionDecision::Pending, AssertionDecision::Accepted],
    )
    .await
}

/// Correct an assertion: inserts a NEW accepted row carrying `supersedes_id`
/// while the old row keeps its identity and moves to `Superseded` (D-07).
///
/// Neither merged nor deleted: the supersession chain orders deterministically
/// by `decided_at` then assertion id (see [`review_history`]).
pub async fn correct(db_path: &str, input: CorrectInput) -> Result<AnnotationReport, GraphError> {
    validate_reviewer(&input.reviewer)?;
    let at_probe = now_rfc3339();
    ensure_operator(db_path, &input.invoked_by, &at_probe).await?;
    let pool = pool(db_path).await?;
    let mut read_conn =
        pool.acquire().await.map_err(|error| build_failed("correct", error.to_string()))?;
    let old = fetch_assertion(&mut read_conn, &input.id).await?;
    drop(read_conn);
    let Some(old) = old else {
        return Err(GraphError::UnknownAssertion { assertion: input.id.clone() });
    };
    if !matches!(
        old.decision,
        AssertionDecision::Pending | AssertionDecision::Accepted | AssertionDecision::Disputed
    ) {
        return Err(rejected(format!(
            "cannot correct assertion '{}' from {}",
            input.id,
            decision_str(old.decision)
        )));
    }
    let old_claim = old.claim.clone();
    let claim_str = |key: &str| {
        old_claim.get(key).and_then(serde_json::Value::as_str).unwrap_or_default().to_string()
    };
    let src = input.src.unwrap_or_else(|| claim_str("src"));
    let edge = input.edge.unwrap_or_else(|| claim_str("edge"));
    let dst = input.dst.unwrap_or_else(|| claim_str("dst"));
    validate_edge(&src, &edge, &dst)?;
    let evidence = input.evidence.unwrap_or_else(|| old.evidence.clone());
    validate_evidence(&evidence)?;
    let source_location = input.source_location.unwrap_or_else(|| old.source_location.clone());
    if source_location.trim().is_empty() {
        return Err(rejected("source_location must be non-empty (PRD 10.3)".to_string()));
    }
    let at = decided_at_or_now(&input.decided_at);
    let old_provenance = old_claim.get("provenance").cloned().unwrap_or_default();
    let source_id =
        old_provenance.get("source_id").and_then(serde_json::Value::as_str).unwrap_or_default();
    let author_or_algorithm = old_provenance
        .get("author_or_algorithm")
        .and_then(serde_json::Value::as_str)
        .unwrap_or(&input.reviewer);
    let version =
        old_provenance.get("version").and_then(serde_json::Value::as_str).map(str::to_string);
    let confidence = old_provenance.get("confidence").and_then(serde_json::Value::as_f64);
    let new_id = format!("{}-c{}", old.id, &uuid::Uuid::new_v4().to_string()[..8]);
    let new = Assertion {
        id: new_id.clone(),
        kind: old.kind,
        claim: claim_json(ClaimParts {
            src: &src,
            edge: &edge,
            dst: &dst,
            source_id,
            source_location: &source_location,
            author_or_algorithm,
            version: version.as_deref(),
            confidence,
            verification_status: "verified",
            created_at: &at,
        }),
        evidence,
        source_location,
        reviewer: Some(input.reviewer.clone()),
        decision: AssertionDecision::Accepted,
        decided_at: Some(at.clone()),
        supersedes_id: Some(old.id.clone()),
        layer: old.layer,
        algorithm: old.algorithm.clone(),
        algorithm_version: old.algorithm_version.clone(),
        confidence: old.confidence,
        created_at: at.clone(),
    };

    let mut tx = pool.begin().await.map_err(|error| build_failed("correct", error.to_string()))?;
    // Scope columns travel with the correction: the new row stays in the old
    // row's authority scope so rebuilds re-derive it identically.
    let scope: Option<(String, String, String, String)> = sqlx::query_as(
        "SELECT projection_row_id, projection_id, edition_id, dataset_scope
         FROM graph_assertions WHERE id = ?",
    )
    .bind(&old.id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|error| build_failed("correct", error.to_string()))?;
    let Some((authority_row, projection_id, edition_id, dataset_scope)) = scope else {
        let _ = tx.rollback().await;
        return Err(GraphError::UnknownAssertion { assertion: input.id.clone() });
    };
    insert_assertion_row(
        &mut tx,
        &new,
        &authority_row,
        &projection_id,
        &edition_id,
        &dataset_scope,
    )
    .await?;
    // The old row keeps its identity; only its decision (plus the correcting
    // reviewer/timestamp, satisfying the decided-row CHECK for pending olds)
    // moves to Superseded.
    sqlx::query(
        "UPDATE graph_assertions SET decision = 'superseded', reviewer = ?, decided_at = ?
         WHERE id = ?",
    )
    .bind(&input.reviewer)
    .bind(&at)
    .bind(&old.id)
    .execute(&mut *tx)
    .await
    .map_err(|error| build_failed("correct", error.to_string()))?;
    let new_subject = assertion_urn(&new_id);
    insert_provenance(
        &mut tx,
        ProvenanceWrite {
            id: format!("graph-prov-{new_id}"),
            shape: ProvenanceShape {
                layer: "scholarly_annotation",
                kind: "scholar",
                trust: "HumanAttested",
                status: "human_verified",
                reviewed: true,
            },
            subject_urn: new_subject.clone(),
            attribution: serde_json::json!({
                "assertion_kind": kind_str(new.kind),
                "claim": new.claim,
                "evidence": new.evidence,
                "supersedes": old.id,
            }),
            confidence: new.confidence,
            actor: &input.invoked_by,
            at: at.clone(),
        },
    )
    .await?;
    append_audit(
        &mut tx,
        &input.reviewer,
        audit::AuditAction::GraphAssertionDecided,
        &new_subject,
        serde_json::json!({
            "assertion_id": new_id,
            "decision": "accepted",
            "supersedes": old.id,
            "previous_decision": decision_str(old.decision),
            "reviewer": input.reviewer,
            "decided_at": at,
        }),
        &at,
    )
    .await?;
    enqueue_outbox(
        &mut tx,
        "graph_review_decided",
        &new_subject,
        &format!("graph_review:{new_id}:corrected:{}", uuid::Uuid::new_v4()),
        &serde_json::json!({"assertion_id": new_id, "supersedes": old.id}),
        "graph_assertion_corrected",
        &at,
    )
    .await?;
    let mut report =
        AnnotationReport { assertion: new, edge_staged: false, build_row_id: None, reused: false };
    if let Some(build_row_id) = active_build_row(&mut tx, &projection_id).await?
        && endpoints_staged(&mut tx, &build_row_id, &src, &dst).await?
    {
        stage_review_edge(&mut tx, &build_row_id, &src, &edge, &dst, &new_id, &at).await?;
        report.edge_staged = true;
        report.build_row_id = Some(build_row_id);
    }
    tx.commit().await.map_err(|error| build_failed("correct", error.to_string()))?;
    Ok(report)
}

/// Look up one authority record by ID.
pub async fn get_assertion(db_path: &str, id: &str) -> Result<Assertion, GraphError> {
    let pool = pool(db_path).await?;
    let mut conn = pool.acquire().await.map_err(|error| build_failed("read", error.to_string()))?;
    fetch_assertion(&mut conn, id)
        .await?
        .ok_or_else(|| GraphError::UnknownAssertion { assertion: id.to_string() })
}

/// The pending review queue for one authority scope (D-06).
///
/// Deterministic order (`created_at`, then assertion id); an empty queue is a
/// complete-empty result, never an error and never a truncation.
pub async fn review_queue(
    db_path: &str,
    projection_id: &str,
    edition_id: &str,
) -> Result<Vec<Assertion>, GraphError> {
    let pool = pool(db_path).await?;
    let mut conn = pool.acquire().await.map_err(|error| build_failed("read", error.to_string()))?;
    let rows = sqlx::query(
        "SELECT id, assertion_kind, claim_json, evidence_json, source_location,
                reviewer, decision, decided_at, supersedes_id, provenance_layer,
                algorithm, algorithm_version, confidence, created_at
         FROM graph_assertions
         WHERE projection_id = ? AND edition_id = ? AND decision = 'pending'
         ORDER BY created_at ASC, id ASC",
    )
    .bind(projection_id)
    .bind(edition_id)
    .fetch_all(&mut *conn)
    .await
    .map_err(|error| build_failed("read", error.to_string()))?;
    rows.iter().map(row_to_assertion).collect()
}

/// The full review history around one assertion: the row, everything it
/// supersedes (transitively), and everything superseding it (transitively).
///
/// Deterministic order (`decided_at`, then assertion id); pending rows sort
/// first (empty timestamp), matching the plan's ordering assumption.
pub async fn review_history(
    db_path: &str,
    assertion_id: &str,
) -> Result<Vec<Assertion>, GraphError> {
    let pool = pool(db_path).await?;
    let mut conn = pool.acquire().await.map_err(|error| build_failed("read", error.to_string()))?;
    let anchor = fetch_assertion(&mut conn, assertion_id)
        .await?
        .ok_or_else(|| GraphError::UnknownAssertion { assertion: assertion_id.to_string() })?;
    let mut chain: HashMap<String, Assertion> = HashMap::new();
    chain.insert(anchor.id.clone(), anchor.clone());
    // Walk backwards through supersedes links.
    let mut cursor = anchor.supersedes_id.clone();
    while let Some(parent_id) = cursor {
        if chain.contains_key(&parent_id) {
            break;
        }
        let Some(parent) = fetch_assertion(&mut conn, &parent_id).await? else { break };
        cursor = parent.supersedes_id.clone();
        chain.insert(parent.id.clone(), parent);
    }
    // Walk forwards through superseding rows.
    loop {
        let known: Vec<String> = chain.keys().cloned().collect();
        let mut grown = false;
        for id in &known {
            let rows = sqlx::query(
                "SELECT id, assertion_kind, claim_json, evidence_json, source_location,
                        reviewer, decision, decided_at, supersedes_id, provenance_layer,
                        algorithm, algorithm_version, confidence, created_at
                 FROM graph_assertions WHERE supersedes_id = ?",
            )
            .bind(id)
            .fetch_all(&mut *conn)
            .await
            .map_err(|error| build_failed("read", error.to_string()))?;
            for row in &rows {
                let record = row_to_assertion(row)?;
                if !chain.contains_key(&record.id) {
                    chain.insert(record.id.clone(), record);
                    grown = true;
                }
            }
        }
        if !grown {
            break;
        }
    }
    let mut ordered: Vec<Assertion> = chain.into_values().collect();
    ordered.sort_by(|a, b| {
        a.decided_at
            .clone()
            .unwrap_or_default()
            .cmp(&b.decided_at.clone().unwrap_or_default())
            .then_with(|| a.id.cmp(&b.id))
    });
    Ok(ordered)
}

/// Pre-serialization policy filter for export surfaces (T-04-05).
///
/// Structural edges pass through; asserted edges survive only when their
/// assertion is effective (pending/accepted/disputed — never
/// rejected/superseded). Only assertions referenced by kept edges travel with
/// the document, so tombstoned IDs never reach serialized bytes while the
/// authority table retains them for audit.
pub fn visible_export_sets(
    nodes: &[GraphNode],
    edges: &[GraphEdge],
    assertions: &[Assertion],
) -> (Vec<GraphNode>, Vec<GraphEdge>, Vec<Assertion>) {
    let effective: HashSet<String> =
        assertions.iter().filter(|a| a.is_effective()).map(|a| a.id.clone()).collect();
    let (nodes, edges) = retain_visible(nodes, edges, assertion_allowlist_predicate(&effective));
    let referenced: HashSet<&str> =
        edges.iter().filter_map(|edge| edge.assertion_id.as_deref()).collect();
    let by_id: HashMap<&str, &Assertion> =
        assertions.iter().map(|record| (record.id.as_str(), record)).collect();
    let mut traveling: Vec<Assertion> = referenced
        .into_iter()
        .filter_map(|id| by_id.get(id).filter(|record| record.is_effective()).cloned().cloned())
        .collect();
    traveling.sort_by(|a, b| a.id.cmp(&b.id));
    (nodes, edges, traveling)
}
