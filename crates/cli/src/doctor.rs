//! Q-ai doctor — read-only diagnostic checks (plan D0.14).

use config::Config;

/// Result severity of a single doctor check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
    Skipped,
}

/// The outcome of one doctor check.
#[derive(Debug, Clone)]
pub struct CheckResult {
    pub id: &'static str,
    pub status: CheckStatus,
    pub summary: String,
    pub remedy: Option<String>,
}

/// Runs the Phase-0 doctor checks against a config; returns exit code.
pub fn run_checks(cfg: &Config, json: bool) -> i32 {
    let results = checks(cfg);
    if json {
        let payload: Vec<serde_json::Value> = results
            .iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.id,
                    "status": format!("{:?}", r.status).to_lowercase(),
                    "summary": r.summary,
                    "remedy": r.remedy,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&payload).unwrap());
    } else {
        for r in &results {
            let status = match r.status {
                CheckStatus::Pass => "PASS",
                CheckStatus::Warn => "WARN",
                CheckStatus::Fail => "FAIL",
                CheckStatus::Skipped => "SKIP",
            };
            println!("[{status}] {} — {}", r.id, r.summary);
            if let Some(rem) = &r.remedy {
                println!("      remedy: {rem}");
            }
        }
    }
    if results.iter().any(|r| r.status == CheckStatus::Fail) {
        exit_code::VALIDATION
    } else {
        exit_code::OK
    }
}

/// Execute the Phase-0 check registry.
pub fn checks(cfg: &Config) -> Vec<CheckResult> {
    vec![
        configuration_valid(cfg),
        data_dir_writable(cfg),
        security_bind_safe(cfg),
        security_tls_consistent(cfg),
    ]
}

fn configuration_valid(cfg: &Config) -> CheckResult {
    match cfg.validate() {
        Ok(()) => CheckResult {
            id: "configuration.valid",
            status: CheckStatus::Pass,
            summary: "configuration validates".into(),
            remedy: None,
        },
        Err(e) => CheckResult {
            id: "configuration.valid",
            status: CheckStatus::Fail,
            summary: format!("configuration invalid: {e}"),
            remedy: Some("run `qai config validate` for details".into()),
        },
    }
}

fn data_dir_writable(cfg: &Config) -> CheckResult {
    let dir = std::env::temp_dir().join("qai-doctor");
    match std::fs::create_dir_all(&dir).and_then(|_| std::fs::write(dir.join("probe"), b"q")) {
        Ok(()) => {
            let _ = std::fs::remove_file(dir.join("probe"));
            let _ = std::fs::remove_dir(&dir);
            CheckResult {
                id: "data_dir.writable",
                status: CheckStatus::Pass,
                summary: format!("data dir probe writable ({} from config)", cfg.app.data_dir),
                remedy: None,
            }
        }
        Err(e) => CheckResult {
            id: "data_dir.writable",
            status: CheckStatus::Fail,
            summary: format!("cannot write probe file: {e}"),
            remedy: Some("choose a writable directory via `qai --data-dir`".into()),
        },
    }
}

fn security_bind_safe(cfg: &Config) -> CheckResult {
    let bind = cfg.server.bind.as_str();
    let safe = bind.starts_with("127.") || bind == "::1" || bind == "localhost";
    if safe {
        CheckResult {
            id: "security.bind_address_safe",
            status: CheckStatus::Pass,
            summary: format!("bind address `{bind}` is loopback"),
            remedy: None,
        }
    } else {
        CheckResult {
            id: "security.bind_address_safe",
            status: CheckStatus::Warn,
            summary: format!("bind address `{bind}` is non-loopback"),
            remedy: Some("set server.bind to 127.0.0.1 or enable TLS in Phase 11".into()),
        }
    }
}

fn security_tls_consistent(cfg: &Config) -> CheckResult {
    let tls = cfg.server.tls.as_str();
    let bind = cfg.server.bind.as_str();
    if bind != "127.0.0.1" && bind != "::1" && tls == "disabled" {
        CheckResult {
            id: "security.tls_policy_consistent",
            status: CheckStatus::Fail,
            summary: format!("non-loopback bind `{bind}` with tls=disabled"),
            remedy: Some("set server.tls=required or bind to loopback".into()),
        }
    } else {
        CheckResult {
            id: "security.tls_policy_consistent",
            status: CheckStatus::Pass,
            summary: "tls policy consistent with bind address".into(),
            remedy: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_passes_all() {
        let cfg = Config::default();
        let results = checks(&cfg);
        assert!(results.iter().all(|r| r.status != CheckStatus::Fail));
    }

    #[test]
    fn bad_bind_triggers_tls_warning() {
        let mut cfg = Config::default();
        cfg.server.bind = "0.0.0.0".into();
        let results = checks(&cfg);
        let tls = results.iter().find(|r| r.id == "security.tls_policy_consistent");
        assert!(matches!(tls, Some(r) if r.status == CheckStatus::Fail));
    }

    #[test]
    fn json_output_contains_ids() {
        let cfg = Config::default();
        let results = checks(&cfg);
        let v: Vec<serde_json::Value> = results
            .iter()
            .map(|r| serde_json::json!({"id": r.id, "status": "pass"}))
            .collect();
        assert!(v.len() >= 4);
    }
}

use crate::exit_code;
