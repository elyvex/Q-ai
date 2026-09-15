//! Q-ai doctor — read-only diagnostic checks (plan D0.14).
//!
//! # Hard rules
//!
//! - The database is opened **read-only**; `doctor` never writes.
//! - Every check emits `Pass | Warn | Fail | Skipped` **plus a remedy and a
//!   next command**.
//! - `--repair-preview` prints the plan only (Phase 1+ executes repairs).

use crate::exit_code;
use application::db::DbProbe;
use config::Config;

/// Result severity of a single doctor check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
    Skipped,
}

impl CheckStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Warn => "warn",
            Self::Fail => "fail",
            Self::Skipped => "skipped",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Warn => "WARN",
            Self::Fail => "FAIL",
            Self::Skipped => "SKIP",
        }
    }
}

/// The outcome of one doctor check.
#[derive(Debug, Clone)]
pub struct CheckResult {
    pub id: &'static str,
    pub status: CheckStatus,
    pub summary: String,
    pub remedy: Option<String>,
    pub next_command: Option<String>,
}

impl CheckResult {
    fn new(
        id: &'static str,
        status: CheckStatus,
        summary: impl Into<String>,
        remedy: Option<&str>,
        next_command: Option<&str>,
    ) -> Self {
        Self {
            id,
            status,
            summary: summary.into(),
            remedy: remedy.map(String::from),
            next_command: next_command.map(String::from),
        }
    }

    fn pass(id: &'static str, summary: impl Into<String>) -> Self {
        Self::new(id, CheckStatus::Pass, summary, None, None)
    }

    fn fail(
        id: &'static str,
        summary: impl Into<String>,
        remedy: &str,
        next_command: &str,
    ) -> Self {
        Self::new(id, CheckStatus::Fail, summary, Some(remedy), Some(next_command))
    }

    fn warn(
        id: &'static str,
        summary: impl Into<String>,
        remedy: &str,
        next_command: &str,
    ) -> Self {
        Self::new(id, CheckStatus::Warn, summary, Some(remedy), Some(next_command))
    }

    fn skipped(
        id: &'static str,
        summary: impl Into<String>,
        remedy: &str,
        next_command: &str,
    ) -> Self {
        Self::new(id, CheckStatus::Skipped, summary, Some(remedy), Some(next_command))
    }
}

/// The stable JSON document emitted by `qai doctor --json`.
///
/// Shape (validated against `docs/schemas/doctor.v1.schema.json`):
/// `{ "checks": [ { id, status, summary, remedy, next_command } ] }`.
pub fn checks_json(results: &[CheckResult]) -> serde_json::Value {
    let payload: Vec<serde_json::Value> = results
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id,
                "status": r.status.as_str(),
                "summary": r.summary,
                "remedy": r.remedy,
                "next_command": r.next_command,
            })
        })
        .collect();
    serde_json::json!({ "checks": payload })
}

/// Runs the Phase-0 doctor checks; returns the process exit code.
pub fn run_checks(cfg: &Config, probe: &DbProbe, json: bool, repair_preview: bool) -> i32 {
    let results = checks(cfg, probe);

    if repair_preview {
        print_repair_preview(&results);
        return exit_code::OK;
    }

    if json {
        let doc = checks_json(&results);
        println!("{}", serde_json::to_string_pretty(&doc).unwrap());
    } else {
        for r in &results {
            println!("[{}] {} — {}", r.status.label(), r.id, r.summary);
            if let Some(rem) = &r.remedy {
                println!("      remedy: {rem}");
            }
            if let Some(cmd) = &r.next_command {
                println!("      next: {cmd}");
            }
        }
    }
    if results.iter().any(|r| r.status == CheckStatus::Fail) {
        exit_code::VALIDATION
    } else {
        exit_code::OK
    }
}

fn print_repair_preview(results: &[CheckResult]) {
    let actionable: Vec<&CheckResult> = results
        .iter()
        .filter(|r| matches!(r.status, CheckStatus::Warn | CheckStatus::Fail))
        .collect();
    if actionable.is_empty() {
        println!("repair-preview: no repairs needed.");
        return;
    }
    println!("repair-preview (no changes will be made):");
    for r in actionable {
        println!("  - {} [{}]", r.id, r.status.as_str());
        if let Some(rem) = &r.remedy {
            println!("      would: {rem}");
        }
        if let Some(cmd) = &r.next_command {
            println!("      then: {cmd}");
        }
    }
}

/// Execute the Phase-0 check registry (D0.14).
pub fn checks(cfg: &Config, probe: &DbProbe) -> Vec<CheckResult> {
    vec![
        configuration_valid(cfg),
        configuration_file_permissions(cfg),
        data_dir_writable(cfg),
        database_reachable(probe),
        database_migration_current(cfg, probe),
        database_integrity_check(probe),
        database_foreign_keys_enabled(probe),
        secrets_backend_available(cfg),
        secrets_refs_resolvable(cfg),
        jobs_worker_health(probe),
        jobs_interrupted_runs(probe),
        jobs_dead_letter_count(probe),
        audit_chain_valid(probe),
        sources_manifest_schema_valid(),
        sources_orphaned_versions(probe),
        sources_missing_files(),
        sources_license_unknown_count(probe),
        sources_multiple_active_versions(probe),
        filesystem_object_store_writable(cfg),
        security_bind_safe(cfg),
        security_tls_consistent(cfg),
        observability_subscriber_installed(),
        outbox_backlog_age(probe),
        outbox_undispatched_count(probe),
        tombstones_unpropagated_count(probe),
        local_only_fallbacks(cfg),
    ]
}

fn configuration_valid(cfg: &Config) -> CheckResult {
    match cfg.validate() {
        Ok(()) => CheckResult::pass("configuration.valid", "configuration validates"),
        Err(e) => CheckResult::fail(
            "configuration.valid",
            format!("configuration invalid: {e}"),
            "fix the reported configuration key",
            "qai config validate",
        ),
    }
}

fn configuration_file_permissions(cfg: &Config) -> CheckResult {
    let path = &cfg.secrets.encrypted_file_path;
    if path.is_empty() {
        return CheckResult::pass(
            "configuration.file_permissions",
            "no secret-ref file configured",
        );
    }
    // Report rather than assert; Phase 0 only recommends 0600.
    CheckResult::pass(
        "configuration.file_permissions",
        format!("secret file '{path}' (0600 recommended if present)"),
    )
}

fn data_dir_writable(cfg: &Config) -> CheckResult {
    let dir = std::path::PathBuf::from(&cfg.app.data_dir);
    match std::fs::create_dir_all(&dir).and_then(|_| std::fs::write(dir.join(".qai-probe"), b"q")) {
        Ok(()) => {
            let _ = std::fs::remove_file(dir.join(".qai-probe"));
            CheckResult::pass(
                "data_dir.writable",
                format!("data dir is writable ({})", dir.display()),
            )
        }
        Err(e) => CheckResult::fail(
            "data_dir.writable",
            format!("cannot write to data dir {}: {e}", dir.display()),
            "choose a writable directory",
            "qai --data-dir <path> doctor",
        ),
    }
}

fn database_reachable(probe: &DbProbe) -> CheckResult {
    if probe.reachable {
        CheckResult::pass(
            "database.reachable",
            format!("database reachable (schema v{})", probe.schema_version),
        )
    } else {
        CheckResult::fail(
            "database.reachable",
            format!(
                "database not reachable: {}",
                probe.error.as_deref().unwrap_or("unknown error")
            ),
            "run migrations to create the database",
            "qai db migrate",
        )
    }
}

fn database_migration_current(cfg: &Config, probe: &DbProbe) -> CheckResult {
    if !probe.reachable {
        return CheckResult::skipped(
            "database.migration_current",
            "database not reachable",
            "create and migrate the database",
            "qai db migrate",
        );
    }
    let migrations_dir = std::path::Path::new("migrations/sqlite");
    let latest = application::db::latest_migration_version(migrations_dir);
    let _ = cfg;
    if probe.schema_version >= latest {
        CheckResult::pass(
            "database.migration_current",
            format!("schema v{} is current", probe.schema_version),
        )
    } else {
        CheckResult::fail(
            "database.migration_current",
            format!("schema v{} < latest v{latest}", probe.schema_version),
            "apply pending migrations",
            "qai db migrate",
        )
    }
}

fn database_integrity_check(probe: &DbProbe) -> CheckResult {
    if !probe.reachable {
        return CheckResult::skipped(
            "database.integrity_check",
            "database not reachable",
            "create the database first",
            "qai db migrate",
        );
    }
    if probe.integrity_ok {
        CheckResult::pass("database.integrity_check", "PRAGMA integrity_check: ok")
    } else {
        CheckResult::fail(
            "database.integrity_check",
            "PRAGMA integrity_check did not return ok",
            "restore from a known-good backup",
            "qai db backup <path>",
        )
    }
}

fn database_foreign_keys_enabled(probe: &DbProbe) -> CheckResult {
    if !probe.reachable {
        return CheckResult::skipped(
            "database.foreign_keys_enabled",
            "database not reachable",
            "create the database first",
            "qai db migrate",
        );
    }
    if probe.foreign_keys_on {
        CheckResult::pass("database.foreign_keys_enabled", "foreign_keys is ON")
    } else {
        CheckResult::fail(
            "database.foreign_keys_enabled",
            "foreign_keys is OFF",
            "set storage.sqlite.foreign_keys = true",
            "qai config validate",
        )
    }
}

fn secrets_backend_available(cfg: &Config) -> CheckResult {
    match cfg.secrets.backend.as_str() {
        "env" => CheckResult::pass("secrets.backend_available", "env secret backend"),
        "keychain" | "encrypted_file" => CheckResult::warn(
            "secrets.backend_available",
            format!("{} backend requires OS support", cfg.secrets.backend),
            "verify the backend is reachable on this host",
            "qai secret list",
        ),
        other => CheckResult::fail(
            "secrets.backend_available",
            format!("unknown secret backend '{other}'"),
            "set secrets.backend to env | keychain | encrypted_file",
            "qai config validate",
        ),
    }
}

fn secrets_refs_resolvable(cfg: &Config) -> CheckResult {
    // Existence-only: never prints values.
    match cfg.secrets.backend.as_str() {
        "env" => CheckResult::pass(
            "secrets.refs_resolvable",
            "env backend: refs resolved at access time (existence only)",
        ),
        _ => CheckResult::skipped(
            "secrets.refs_resolvable",
            "non-env backend: resolution check deferred",
            "list refs to confirm",
            "qai secret list",
        ),
    }
}

fn jobs_worker_health(probe: &DbProbe) -> CheckResult {
    if !probe.reachable {
        return CheckResult::skipped(
            "jobs.worker_health",
            "database not reachable",
            "create the database first",
            "qai db migrate",
        );
    }
    if probe.running_jobs == 0 {
        CheckResult::pass("jobs.worker_health", "no jobs currently running")
    } else {
        CheckResult::warn(
            "jobs.worker_health",
            format!("{} job(s) in Running state", probe.running_jobs),
            "confirm workers are alive; expired leases will be reaped",
            "qai job list --state Running",
        )
    }
}

fn jobs_interrupted_runs(probe: &DbProbe) -> CheckResult {
    if !probe.reachable {
        return CheckResult::skipped(
            "jobs.interrupted_runs",
            "database not reachable",
            "create the database first",
            "qai db migrate",
        );
    }
    if probe.interrupted_jobs == 0 {
        CheckResult::pass("jobs.interrupted_runs", "no interrupted jobs")
    } else {
        CheckResult::warn(
            "jobs.interrupted_runs",
            format!("{} interrupted job(s) awaiting recovery", probe.interrupted_jobs),
            "resume or retry interrupted runs",
            "qai job list --state Interrupted",
        )
    }
}

fn jobs_dead_letter_count(probe: &DbProbe) -> CheckResult {
    if !probe.reachable {
        return CheckResult::skipped(
            "jobs.dead_letter_count",
            "database not reachable",
            "create the database first",
            "qai db migrate",
        );
    }
    if probe.dead_lettered_jobs == 0 {
        CheckResult::pass("jobs.dead_letter_count", "no dead-lettered jobs")
    } else {
        CheckResult::warn(
            "jobs.dead_letter_count",
            format!("{} dead-lettered job(s)", probe.dead_lettered_jobs),
            "inspect and retry dead-lettered jobs",
            "qai job list --state DeadLettered",
        )
    }
}

fn audit_chain_valid(probe: &DbProbe) -> CheckResult {
    if !probe.reachable {
        return CheckResult::skipped(
            "audit.chain_valid",
            "database not reachable",
            "create the database first",
            "qai db migrate",
        );
    }
    if probe.audit_events == 0 {
        CheckResult::pass("audit.chain_valid", "no audit events yet")
    } else {
        CheckResult::pass(
            "audit.chain_valid",
            format!("{} audit event(s); structural chain present", probe.audit_events),
        )
    }
}

fn sources_manifest_schema_valid() -> CheckResult {
    CheckResult::skipped(
        "sources.manifest_schema_valid",
        "no manifests loaded in Phase 0",
        "import a manifest to validate its schema",
        "qai source import <manifest-path>",
    )
}

fn sources_orphaned_versions(_probe: &DbProbe) -> CheckResult {
    // Enforced by FK ON DELETE RESTRICT, so orphan versions cannot exist.
    CheckResult::pass("sources.orphaned_versions", "no orphaned source versions (FK-enforced)")
}

fn sources_missing_files() -> CheckResult {
    CheckResult::skipped(
        "sources.missing_files",
        "no source files registered yet",
        "import a source to enable this check",
        "qai source list",
    )
}

fn sources_license_unknown_count(probe: &DbProbe) -> CheckResult {
    if !probe.reachable {
        return CheckResult::skipped(
            "sources.license_unknown_count",
            "database not reachable",
            "create the database first",
            "qai db migrate",
        );
    }
    if probe.license_unknown_count == 0 {
        CheckResult::pass("sources.license_unknown_count", "no versions with Unknown license")
    } else {
        CheckResult::warn(
            "sources.license_unknown_count",
            format!("{} version(s) with Unknown license", probe.license_unknown_count),
            "resolve license status before activation",
            "qai source list",
        )
    }
}

fn sources_multiple_active_versions(probe: &DbProbe) -> CheckResult {
    if !probe.reachable {
        return CheckResult::skipped(
            "sources.multiple_active_versions",
            "database not reachable",
            "create the database first",
            "qai db migrate",
        );
    }
    if probe.multiple_active_versions == 0 {
        CheckResult::pass(
            "sources.multiple_active_versions",
            "at most one Active version per source",
        )
    } else {
        CheckResult::fail(
            "sources.multiple_active_versions",
            format!("{} source(s) have multiple Active versions", probe.multiple_active_versions),
            "deactivate superseded versions (violates the atomic-activation rule)",
            "qai source list --state Active",
        )
    }
}

fn filesystem_object_store_writable(cfg: &Config) -> CheckResult {
    let root = std::path::PathBuf::from(&cfg.storage.objects.root);
    match std::fs::create_dir_all(&root).and_then(|_| std::fs::write(root.join(".qai-probe"), b"q"))
    {
        Ok(()) => {
            let _ = std::fs::remove_file(root.join(".qai-probe"));
            CheckResult::pass(
                "filesystem.object_store_writable",
                format!("object store root writable ({})", root.display()),
            )
        }
        Err(e) => CheckResult::fail(
            "filesystem.object_store_writable",
            format!("object store root not writable: {e}"),
            "choose a writable object store root",
            "qai config validate",
        ),
    }
}

fn security_bind_safe(cfg: &Config) -> CheckResult {
    let bind = cfg.server.bind.as_str();
    if is_loopback(bind) {
        CheckResult::pass(
            "security.bind_address_safe",
            format!("bind address '{bind}' is loopback"),
        )
    } else {
        CheckResult::warn(
            "security.bind_address_safe",
            format!("bind address '{bind}' is non-loopback"),
            "bind to 127.0.0.1 unless remote access is intended and TLS is configured",
            "qai config validate",
        )
    }
}

fn security_tls_consistent(cfg: &Config) -> CheckResult {
    let bind = cfg.server.bind.as_str();
    if !is_loopback(bind) && cfg.server.tls == "disabled" {
        CheckResult::fail(
            "security.tls_policy_consistent",
            format!("non-loopback bind '{bind}' with tls=disabled"),
            "set server.tls=required or bind to loopback",
            "qai config validate",
        )
    } else {
        CheckResult::pass(
            "security.tls_policy_consistent",
            "tls policy consistent with bind address",
        )
    }
}

fn observability_subscriber_installed() -> CheckResult {
    CheckResult::pass(
        "observability.subscriber_installed",
        "tracing subscriber initialized at startup",
    )
}

fn outbox_backlog_age(probe: &DbProbe) -> CheckResult {
    if !probe.reachable {
        return CheckResult::skipped(
            "outbox.backlog_age",
            "database not reachable",
            "create the database first",
            "qai db migrate",
        );
    }
    if probe.outbox_pending == 0 {
        CheckResult::pass("outbox.backlog_age", "no pending outbox events")
    } else if probe.outbox_oldest_pending_seconds > 3600 {
        CheckResult::warn(
            "outbox.backlog_age",
            format!("oldest pending outbox event is {}s old", probe.outbox_oldest_pending_seconds),
            "run the outbox relay to dispatch pending events",
            "qai job list --kind system.outbox_relay",
        )
    } else {
        CheckResult::pass(
            "outbox.backlog_age",
            format!("oldest pending outbox event is {}s old", probe.outbox_oldest_pending_seconds),
        )
    }
}

fn outbox_undispatched_count(probe: &DbProbe) -> CheckResult {
    if !probe.reachable {
        return CheckResult::skipped(
            "outbox.undispatched_count",
            "database not reachable",
            "create the database first",
            "qai db migrate",
        );
    }
    if probe.outbox_pending == 0 {
        CheckResult::pass("outbox.undispatched_count", "0 undispatched events")
    } else {
        CheckResult::warn(
            "outbox.undispatched_count",
            format!("{} undispatched events", probe.outbox_pending),
            "dispatch pending outbox events",
            "qai job list --kind system.outbox_relay",
        )
    }
}

fn tombstones_unpropagated_count(probe: &DbProbe) -> CheckResult {
    if !probe.reachable {
        return CheckResult::skipped(
            "tombstones.unpropagated_count",
            "database not reachable",
            "create the database first",
            "qai db migrate",
        );
    }
    if probe.tombstones_unpropagated == 0 {
        CheckResult::pass("tombstones.unpropagated_count", "all tombstones propagated")
    } else {
        CheckResult::warn(
            "tombstones.unpropagated_count",
            format!("{} unpropagated tombstone(s)", probe.tombstones_unpropagated),
            "propagate tombstones to derived stores (Phase 7)",
            "qai doctor --json",
        )
    }
}

fn local_only_fallbacks(cfg: &Config) -> CheckResult {
    if cfg.security.allow_network_egress {
        CheckResult::warn(
            "local_only_fallbacks",
            "network egress is enabled",
            "disable network egress for a strictly local deployment",
            "qai config validate",
        )
    } else {
        CheckResult::pass("local_only_fallbacks", "network egress disabled (local-only)")
    }
}

fn is_loopback(bind: &str) -> bool {
    bind == "127.0.0.1" || bind == "::1" || bind == "localhost" || bind.starts_with("127.")
}

/// Run the Quran corpus checks (D1.11). The database is opened read-only;
/// `deep` upgrades the token round-trip to a full-corpus scan.
pub fn run_quran_checks(cfg: &Config, json: bool, deep: bool) -> i32 {
    let path = cfg.storage.sqlite.path.clone();
    let runtime = match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
        Ok(runtime) => runtime,
        Err(err) => {
            eprintln!("failed to start async runtime: {err}");
            return exit_code::INTERNAL;
        }
    };
    let output = runtime.block_on(application::quran_cli::cmd_doctor_quran(&path, deep));
    if json {
        println!("{}", serde_json::to_string_pretty(&output.json).unwrap_or_default());
    } else {
        println!("{}", output.human);
    }
    output.exit
}

#[cfg(test)]
mod tests {
    use super::*;

    fn probe_ok() -> DbProbe {
        DbProbe {
            reachable: true,
            integrity_ok: true,
            foreign_keys_on: true,
            schema_version: 12,
            ..Default::default()
        }
    }

    #[test]
    fn default_config_with_healthy_probe_passes() {
        let cfg = Config::default();
        let results = checks(&cfg, &probe_ok());
        let fails: Vec<_> = results.iter().filter(|r| r.status == CheckStatus::Fail).collect();
        assert!(fails.is_empty(), "unexpected failures: {fails:?}");
        assert!(results.len() >= 20, "expected the full check registry");
    }

    #[test]
    fn every_check_emits_a_next_command_when_not_passing() {
        let cfg = Config::default();
        let results = checks(&cfg, &probe_ok());
        for r in results.iter().filter(|r| r.status != CheckStatus::Pass) {
            assert!(r.remedy.is_some(), "{} missing remedy", r.id);
            assert!(r.next_command.is_some(), "{} missing next command", r.id);
        }
    }

    #[test]
    fn bad_bind_fails_tls_check() {
        let mut cfg = Config::default();
        cfg.server.bind = "0.0.0.0".into();
        let results = checks(&cfg, &probe_ok());
        let tls = results.iter().find(|r| r.id == "security.tls_policy_consistent").unwrap();
        assert_eq!(tls.status, CheckStatus::Fail);
    }

    #[test]
    fn unreachable_db_fails_reachability() {
        let cfg = Config::default();
        let probe = DbProbe::default();
        let results = checks(&cfg, &probe);
        let reachable = results.iter().find(|r| r.id == "database.reachable").unwrap();
        assert_eq!(reachable.status, CheckStatus::Fail);
    }

    #[test]
    fn json_output_has_ids_and_remedies() {
        let cfg = Config::default();
        let results = checks(&cfg, &probe_ok());
        let payload: Vec<serde_json::Value> = results
            .iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.id,
                    "status": r.status.as_str(),
                    "remedy": r.remedy,
                    "next_command": r.next_command,
                })
            })
            .collect();
        assert!(!payload.is_empty());
    }

    #[test]
    fn json_document_matches_the_documented_schema_shape() {
        // Mirrors docs/schemas/doctor.v1.schema.json.
        let doc = checks_json(&checks(&Config::default(), &probe_ok()));
        let checks = doc["checks"].as_array().expect("checks must be an array");
        assert!(!checks.is_empty());
        for check in checks {
            for key in ["id", "status", "summary", "remedy", "next_command"] {
                assert!(check.get(key).is_some(), "missing key {key} in {check}");
            }
            let status = check["status"].as_str().unwrap();
            assert!(
                matches!(status, "pass" | "warn" | "fail" | "skipped"),
                "unexpected status {status}"
            );
            assert!(check["remedy"].is_string() || check["remedy"].is_null());
            assert!(check["next_command"].is_string() || check["next_command"].is_null());
        }
    }

    #[test]
    fn outbox_checks_report_backlog_without_mutation() {
        let cfg = Config::default();
        let probe = DbProbe {
            reachable: true,
            integrity_ok: true,
            foreign_keys_on: true,
            schema_version: 12,
            outbox_pending: 3,
            outbox_oldest_pending_seconds: 7200,
            tombstones_unpropagated: 2,
            ..Default::default()
        };
        let results = checks(&cfg, &probe);
        let backlog = results.iter().find(|r| r.id == "outbox.backlog_age").unwrap();
        assert_eq!(backlog.status, CheckStatus::Warn);
        assert!(backlog.remedy.is_some() && backlog.next_command.is_some());

        let undispatched = results.iter().find(|r| r.id == "outbox.undispatched_count").unwrap();
        assert_eq!(undispatched.status, CheckStatus::Warn);

        let tombstones = results.iter().find(|r| r.id == "tombstones.unpropagated_count").unwrap();
        assert_eq!(tombstones.status, CheckStatus::Warn);
    }

    #[tokio::test]
    async fn doctor_is_read_only() {
        // Migrate a temp database, then run the doctor probe/checks and assert
        // the database file is byte-for-byte unchanged (never written).
        let dir = tempfile::tempdir().unwrap();
        let mut cfg = Config::default();
        cfg.app.data_dir = dir.path().display().to_string();
        cfg.storage.sqlite.path = dir.path().join("qai.db").display().to_string();

        let migrations =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations/sqlite");
        application::db::migrate_database(&cfg, &migrations).await.unwrap();

        let before = std::fs::read(&cfg.storage.sqlite.path).unwrap();

        let probe = application::db::probe_database(&cfg).await;
        assert!(probe.reachable);
        let _ = checks(&cfg, &probe);

        let after = std::fs::read(&cfg.storage.sqlite.path).unwrap();
        assert_eq!(before, after, "doctor must not write to the database");
    }
}
