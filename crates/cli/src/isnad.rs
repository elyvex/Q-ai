//! `qai isnad …` commands (Phase 7).

use clap::Subcommand;
use std::sync::Arc;
use storage_sqlite::SqliteDatabase;
use tool_registry::{IdentityDecisionParams, IdentityProposalParams, IsnadBackend, TraceParams};

use crate::exit_code;

#[derive(Subcommand)]
pub enum IdentityAction {
    Propose {
        #[arg(long)]
        claim: String,
        #[arg(long)]
        evidence: String,
        #[arg(long)]
        source_location: Option<String>,
        #[arg(long)]
        provenance_layer: String,
        #[arg(long)]
        algorithm: Option<String>,
        #[arg(long)]
        algorithm_version: Option<String>,
        #[arg(long)]
        confidence: Option<f64>,
    },
    Accept {
        #[arg(long)]
        id: String,
        #[arg(long)]
        reviewer: String,
    },
    Reject {
        #[arg(long)]
        id: String,
        #[arg(long)]
        reviewer: String,
        #[arg(long)]
        yes: bool,
    },
    Dispute {
        #[arg(long)]
        id: String,
        #[arg(long)]
        reviewer: String,
    },
    Supersede {
        #[arg(long)]
        id: String,
        #[arg(long)]
        new_id: String,
        #[arg(long)]
        reviewer: String,
    },
}

#[derive(Subcommand)]
pub enum IsnadAction {
    Narrator {
        #[arg(long)]
        id: String,
    },
    Rijal {
        #[arg(long)]
        id: String,
    },
    Trace {
        #[arg(long)]
        from: String,
        #[arg(long)]
        to: String,
        #[arg(long, default_value_t = 3)]
        k: usize,
    },
    Identity {
        #[command(subcommand)]
        action: IdentityAction,
    },
}

pub fn handle_isnad(action: IsnadAction, db_path: &str, json: bool) -> i32 {
    let runtime = match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
        Ok(rt) => rt,
        Err(err) => {
            eprintln!("failed to start async runtime: {err}");
            return exit_code::INTERNAL;
        }
    };
    runtime.block_on(handle_isnad_async(action, db_path, json))
}

async fn handle_isnad_async(action: IsnadAction, db_path: &str, json: bool) -> i32 {
    use application::isnad_reader::IsnadReader;
    use application::isnad_reader::IsnadReaderService;
    use application::quran_cli::CommandOutput;

    let db = match SqliteDatabase::new(db_path, 4, true).await {
        Ok(db) => Arc::new(db),
        Err(err) => {
            let o = CommandOutput::err(exit_code::GENERIC, err.to_string());
            if json {
                println!("{}", serde_json::to_string_pretty(&o.json).unwrap_or_default());
            } else {
                println!("{}", o.human);
            }
            return o.exit;
        }
    };
    let reader: Arc<dyn IsnadReader> = Arc::new(IsnadReaderService::new(db.clone()));
    let backend = application::isnad_tools::IsnadToolBackend::new(reader.clone());

    match action {
        IsnadAction::Narrator { id } => match backend.backend_get_narrator(&id).await {
            Ok(tr) => {
                let narrator = &tr.results["narrator"];
                let rijal = tr.results["rijal_sources"].as_array().cloned().unwrap_or_default();
                let stable_id =
                    narrator.get("id").and_then(|v| v.as_str()).unwrap_or(&id).to_string();
                let primary_name =
                    narrator.get("primary_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let variants: Vec<String> = narrator
                    .get("name_variants")
                    .and_then(|v| v.as_array())
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default();
                let mut human = format!("{} — {}\n", stable_id, primary_name);
                if variants.is_empty() {
                    human.push_str("variants: (none)\n");
                } else {
                    human.push_str(&format!("variants: {}\n", variants.join(", ")));
                }
                human.push_str(&format!("rijal sources ({} scholar(s)):\n", rijal.len()));
                for r in &rijal {
                    let scholar = r.get("scholar_name").and_then(|v| v.as_str()).unwrap_or("");
                    let cite = r.get("source_citation").and_then(|v| v.as_str()).unwrap_or("");
                    let grade =
                        r.get("grading_value").and_then(|v| v.as_str()).unwrap_or("no grade");
                    let decision = r.get("decision").and_then(|v| v.as_str()).unwrap_or("");
                    human.push_str(&format!("{} — {} — {} ({})\n", scholar, cite, grade, decision));
                }
                let jsonv = serde_json::to_value(&tr).unwrap_or(serde_json::Value::Null);
                if json {
                    println!("{}", serde_json::to_string_pretty(&jsonv).unwrap_or_default());
                } else {
                    print!("{}", human);
                }
                exit_code::OK
            }
            Err(err) => print_tool_error(&err, json),
        },
        IsnadAction::Rijal { id } => match backend.backend_get_rijal(&id).await {
            Ok(tr) => {
                let rijal = tr.results["rijal_sources"].as_array().cloned().unwrap_or_default();
                let mut human = String::new();
                if rijal.is_empty() {
                    human.push_str(&format!("rijal sources for {}: 0 scholar(s)\n", id));
                } else {
                    human.push_str(&format!("rijal sources ({} scholar(s)):\n", rijal.len()));
                    for r in &rijal {
                        let scholar = r.get("scholar_name").and_then(|v| v.as_str()).unwrap_or("");
                        let cite = r.get("source_citation").and_then(|v| v.as_str()).unwrap_or("");
                        let grade =
                            r.get("grading_value").and_then(|v| v.as_str()).unwrap_or("no grade");
                        let decision = r.get("decision").and_then(|v| v.as_str()).unwrap_or("");
                        human.push_str(&format!(
                            "{} — {} — {} ({})\n",
                            scholar, cite, grade, decision
                        ));
                    }
                }
                let jsonv = serde_json::to_value(&tr).unwrap_or(serde_json::Value::Null);
                if json {
                    println!("{}", serde_json::to_string_pretty(&jsonv).unwrap_or_default());
                } else {
                    print!("{}", human);
                }
                exit_code::OK
            }
            Err(err) => print_tool_error(&err, json),
        },
        IsnadAction::Trace { from, to, k } => {
            let params = TraceParams { from: from.clone(), to: to.clone(), k: Some(k) };
            match backend.backend_trace(&params).await {
                Ok(tr) => {
                    let res = &tr.results;
                    let paths =
                        res.get("paths").and_then(|v| v.as_array()).cloned().unwrap_or_default();
                    let truncated = res.get("truncated").and_then(|v| v.as_bool()).unwrap_or(false);
                    let reason = res.get("incomplete_reason").and_then(|v| v.as_str());
                    let mut human = String::new();
                    if truncated {
                        human.push_str("… (truncated)\n");
                    }
                    if paths.is_empty() {
                        if truncated {
                            human.push_str("unknown (truncated)\n");
                        } else {
                            human.push_str("no path (complete search)\n");
                        }
                    } else {
                        human.push_str(&format!("{} path(s)\n", paths.len()));
                        for path in &paths {
                            let edges = path
                                .get("edges")
                                .and_then(|v| v.as_array())
                                .cloned()
                                .unwrap_or_default();
                            for e in &edges {
                                let src = e.get("src").and_then(|v| v.as_str()).unwrap_or("");
                                let edge = e.get("edge").and_then(|v| v.as_str()).unwrap_or("");
                                let dst = e.get("dst").and_then(|v| v.as_str()).unwrap_or("");
                                let attrs =
                                    e.get("attrs").cloned().unwrap_or(serde_json::Value::Null);
                                let prov = attrs
                                    .get("provenance_layer")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("B");
                                let conf = attrs.get("confidence").and_then(|v| v.as_f64());
                                let conf_str =
                                    conf.map(|c| format!("{:.2}", c)).unwrap_or_default();
                                human.push_str(&format!(
                                    "  {} -{}-> {} {} {} (uncertain)\n",
                                    src, edge, dst, prov, conf_str
                                ));
                            }
                        }
                    }
                    if truncated {
                        if let Some(r) = reason {
                            human.push_str(&format!("incomplete: {}\n", r));
                        }
                    }
                    let jsonv = serde_json::to_value(&tr).unwrap_or(serde_json::Value::Null);
                    if json {
                        println!("{}", serde_json::to_string_pretty(&jsonv).unwrap_or_default());
                    } else {
                        print!("{}", human);
                    }
                    exit_code::OK
                }
                Err(err) => print_tool_error(&err, json),
            }
        }
        IsnadAction::Identity { action } => match action {
            IdentityAction::Propose {
                claim,
                evidence,
                source_location,
                provenance_layer,
                algorithm,
                algorithm_version,
                confidence,
            } => {
                let claim_v = parse_json(&claim);
                let evidence_v = parse_json(&evidence);
                let params = IdentityProposalParams {
                    claim: claim_v,
                    evidence: evidence_v,
                    source_location: source_location.unwrap_or_default(),
                    provenance_layer,
                    algorithm,
                    algorithm_version,
                    confidence,
                };
                match backend.backend_propose_identity(&params).await {
                    Ok(tr) => {
                        let assertion = &tr.results;
                        let id = assertion.get("id").and_then(|v| v.as_str()).unwrap_or("");
                        let created =
                            assertion.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
                        let actor = assertion
                            .get("algorithm")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unreviewed");
                        let human =
                            format!("proposed {} (pending) by {} at {}\n", id, actor, created);
                        let jsonv = serde_json::to_value(&tr).unwrap_or(serde_json::Value::Null);
                        if json {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&jsonv).unwrap_or_default()
                            );
                        } else {
                            print!("{}", human);
                        }
                        exit_code::OK
                    }
                    Err(err) => print_tool_error(&err, json),
                }
            }
            IdentityAction::Accept { id, reviewer } => {
                let params = IdentityDecisionParams { id: id.clone(), reviewer: reviewer.clone() };
                match backend.backend_accept_identity(&params).await {
                    Ok(tr) => {
                        let (decided_at, rev) = fetch_decided(&db, &id).await;
                        let reviewer = rev.unwrap_or(reviewer);
                        let decided_at = decided_at.unwrap_or_default();
                        let human = format!(
                            "accepted {} (accepted) by {} at {}\n",
                            id, reviewer, decided_at
                        );
                        let jsonv = serde_json::to_value(&tr).unwrap_or(serde_json::Value::Null);
                        if json {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&jsonv).unwrap_or_default()
                            );
                        } else {
                            print!("{}", human);
                        }
                        exit_code::OK
                    }
                    Err(err) => print_tool_error(&err, json),
                }
            }
            IdentityAction::Reject { id, reviewer, yes } => {
                if !yes {
                    if json {
                        println!(
                            "{}",
                            serde_json::json!({"error": "refusing to reject without --yes"})
                        );
                    } else {
                        println!("refusing to reject without --yes");
                    }
                    return exit_code::USAGE;
                }
                let params = IdentityDecisionParams { id: id.clone(), reviewer: reviewer.clone() };
                match backend.backend_reject_identity(&params).await {
                    Ok(tr) => {
                        let (decided_at, rev) = fetch_decided(&db, &id).await;
                        let reviewer = rev.unwrap_or(reviewer);
                        let decided_at = decided_at.unwrap_or_default();
                        let human = format!(
                            "rejected {} (rejected) by {} at {}\n",
                            id, reviewer, decided_at
                        );
                        let jsonv = serde_json::to_value(&tr).unwrap_or(serde_json::Value::Null);
                        if json {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&jsonv).unwrap_or_default()
                            );
                        } else {
                            print!("{}", human);
                        }
                        exit_code::OK
                    }
                    Err(err) => print_tool_error(&err, json),
                }
            }
            IdentityAction::Dispute { id, reviewer } => {
                let params = IdentityDecisionParams { id: id.clone(), reviewer: reviewer.clone() };
                match backend.backend_dispute_identity(&params).await {
                    Ok(tr) => {
                        let (decided_at, rev) = fetch_decided(&db, &id).await;
                        let reviewer = rev.unwrap_or(reviewer);
                        let decided_at = decided_at.unwrap_or_default();
                        let human = format!(
                            "disputed {} (disputed) by {} at {}\n",
                            id, reviewer, decided_at
                        );
                        let jsonv = serde_json::to_value(&tr).unwrap_or(serde_json::Value::Null);
                        if json {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&jsonv).unwrap_or_default()
                            );
                        } else {
                            print!("{}", human);
                        }
                        exit_code::OK
                    }
                    Err(err) => print_tool_error(&err, json),
                }
            }
            IdentityAction::Supersede { id, new_id, reviewer } => {
                let _ = new_id;
                let proposal = IdentityProposalParams {
                    claim: serde_json::json!({}),
                    evidence: serde_json::json!({}),
                    source_location: String::new(),
                    provenance_layer: "B".to_string(),
                    algorithm: None,
                    algorithm_version: None,
                    confidence: None,
                };
                match backend.backend_supersede_identity(&id, &proposal, &reviewer).await {
                    Ok(tr) => {
                        let (decided_at, rev) = fetch_decided(&db, &id).await;
                        let reviewer = rev.unwrap_or(reviewer);
                        let decided_at = decided_at.unwrap_or_default();
                        let human = format!(
                            "superseded {} (superseded) by {} at {}\n",
                            id, reviewer, decided_at
                        );
                        let jsonv = serde_json::to_value(&tr).unwrap_or(serde_json::Value::Null);
                        if json {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&jsonv).unwrap_or_default()
                            );
                        } else {
                            print!("{}", human);
                        }
                        exit_code::OK
                    }
                    Err(err) => print_tool_error(&err, json),
                }
            }
        },
    }
}

async fn fetch_decided(db: &Arc<SqliteDatabase>, id: &str) -> (Option<String>, Option<String>) {
    use sqlx::Row;
    let row =
        sqlx::query("SELECT decided_at, reviewer FROM narrator_identity_assertions WHERE id = ?")
            .bind(id)
            .fetch_optional(db.read_pool())
            .await
            .ok()
            .flatten();
    match row {
        Some(r) => (r.get("decided_at"), r.get("reviewer")),
        None => (None, None),
    }
}

fn parse_json(s: &str) -> serde_json::Value {
    serde_json::from_str(s).unwrap_or_else(|_| serde_json::Value::String(s.to_string()))
}

fn print_tool_error(err: &tools::ToolError, json: bool) -> i32 {
    use tools::ToolError;
    let code = match err {
        ToolError::Backend { code, .. } => code.clone(),
        ToolError::InvalidInput { .. } => "QAI-ISNAD-0003".to_string(),
    };
    let detail = err.to_string();
    let exit = if code.ends_with("0002") {
        exit_code::NOT_FOUND
    } else if code.ends_with("0003") {
        exit_code::USAGE
    } else if code.ends_with("0004") {
        exit_code::VALIDATION
    } else {
        exit_code::INTERNAL
    };
    if json {
        println!(
            "{}",
            serde_json::json!({
                "code": code,
                "detail": detail,
                "remedy": "Check input and retry.",
                "next": "qai isnad --help",
            })
        );
    } else {
        println!("[{}] {}", code, detail);
        println!("      Remedy: Check input and retry.");
        println!("      Next: qai isnad --help");
    }
    exit
}
