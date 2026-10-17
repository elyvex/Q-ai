//! `qai rag …` commands (Phase 8).

use clap::Subcommand;
use std::sync::Arc;
use application::rag_reader::RagReader;
use storage_sqlite::SqliteDatabase;

use crate::exit_code;

#[derive(Subcommand)]
pub enum RagAction {
    /// Create a new RAG project.
    Create {
        name: String,
        #[arg(long)]
        dir: Option<std::path::PathBuf>,
    },
    /// Import documents into a RAG project.
    Import {
        project: String,
        docs: std::path::PathBuf,
        #[arg(long)]
        glob: Option<String>,
    },
    /// Run a hybrid retrieval query.
    Query {
        project: String,
        query: String,
        #[arg(long)]
        filter: Vec<String>,
        #[arg(long)]
        limit: Option<u32>,
    },
    /// Debug a retrieval: print full route decision.
    Debug {
        project: String,
        query: String,
    },
    /// Source management.
    Source {
        #[command(subcommand)]
        action: SourceAction,
    },
}

#[derive(Subcommand)]
pub enum SourceAction {
    Activate {
        project: String,
        source: String,
    },
    Deactivate {
        project: String,
        source: String,
    },
    Tombstone {
        project: String,
        source: String,
        #[arg(long)]
        yes: bool,
    },
}

pub fn handle_rag(action: RagAction, json: bool) -> i32 {
    let runtime = match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
        Ok(rt) => rt,
        Err(err) => {
            eprintln!("failed to start async runtime: {err}");
            return exit_code::INTERNAL;
        }
    };
    runtime.block_on(handle_rag_async(action, json))
}

async fn handle_rag_async(action: RagAction, json: bool) -> i32 {
    use tool_registry::RagBackend;

    match action {
        RagAction::Create { name, dir } => {
            if name.trim().is_empty() || name.contains('/') || name.contains('\\') {
                let o = application::quran_cli::CommandOutput::err(exit_code::USAGE, "invalid project name".to_string());
                if json { println!("{}", serde_json::to_string_pretty(&o.json).unwrap_or_default()); } else { println!("{}", o.human); }
                return o.exit;
            }
            let project_dir = dir.unwrap_or_else(|| std::path::PathBuf::from(&name));
            if let Err(e) = std::fs::create_dir_all(&project_dir) {
                let o = application::quran_cli::CommandOutput::err(exit_code::GENERIC, e.to_string());
                if json { println!("{}", serde_json::to_string_pretty(&o.json).unwrap_or_default()); } else { println!("{}", o.human); }
                return o.exit;
            }
            let config_str = format!(r#"[project]
name = "{}"
version = "1"

[embedding]
provider = "local-hash"
dimension = 128

[reranker]
provider = "score"

[router]
name = "hybrid-rrf"

[[sources]]
path = "./docs"
glob = "**/*.md"
"#, name);
            if let Err(e) = std::fs::write(project_dir.join("config.toml"), &config_str) {
                let o = application::quran_cli::CommandOutput::err(exit_code::GENERIC, e.to_string());
                if json { println!("{}", serde_json::to_string_pretty(&o.json).unwrap_or_default()); } else { println!("{}", o.human); }
                return o.exit;
            }
            let db_path = project_dir.join("index.db");
            let db_path_str = db_path.to_string_lossy().to_string();
            let migrations = migrations_dir();
            if let Err(e) = storage_sqlite::migrate::apply_migrations(&db_path_str, &migrations).await {
                let o = application::quran_cli::CommandOutput::err(exit_code::GENERIC, e.to_string());
                if json { println!("{}", serde_json::to_string_pretty(&o.json).unwrap_or_default()); } else { println!("{}", o.human); }
                return o.exit;
            }
            let db = match SqliteDatabase::new(&db_path_str, 4, true).await {
                Ok(db) => Arc::new(db),
                Err(e) => {
                    let o = application::quran_cli::CommandOutput::err(exit_code::GENERIC, e.to_string());
                    if json { println!("{}", serde_json::to_string_pretty(&o.json).unwrap_or_default()); } else { println!("{}", o.human); }
                    return o.exit;
                }
            };
            let reader = Arc::new(application::rag_reader::SqliteRagReader::new(db));
            match reader.create_project(&name, &config_str).await {
                Ok(_) => {
                    println!("created {} at {}", name, project_dir.display());
                    exit_code::OK
                }
                Err(e) => {
                    let o = application::quran_cli::CommandOutput::err(exit_code::GENERIC, e.to_string());
                    if json { println!("{}", serde_json::to_string_pretty(&o.json).unwrap_or_default()); } else { println!("{}", o.human); }
                    o.exit
                }
            }
        }
        RagAction::Import { project, docs, glob } => {
            let project_dir = std::path::PathBuf::from(&project);
            let db_path = project_dir.join("index.db");
            let db_path_str = db_path.to_string_lossy().to_string();
            let migrations = migrations_dir();
            let _ = storage_sqlite::migrate::apply_migrations(&db_path_str, &migrations).await;
            let db = match SqliteDatabase::new(&db_path_str, 4, true).await {
                Ok(db) => Arc::new(db),
                Err(e) => {
                    let o = application::quran_cli::CommandOutput::err(exit_code::GENERIC, e.to_string());
                    if json { println!("{}", serde_json::to_string_pretty(&o.json).unwrap_or_default()); } else { println!("{}", o.human); }
                    return o.exit;
                }
            };
            let reader = Arc::new(application::rag_reader::SqliteRagReader::new(db));
            // determine glob
            let glob_str = match glob {
                Some(g) => g,
                None => {
                    let cfg_text = std::fs::read_to_string(project_dir.join("config.toml")).unwrap_or_default();
                    extract_glob(&cfg_text).unwrap_or_else(|| "**/*".to_string())
                }
            };
            // walk docs
            let mut files: Vec<(String, String, serde_json::Value)> = Vec::new();
            if docs.is_dir() {
                walk_dir(&docs, &docs, &glob_str, &mut files);
            } else {
                let o = application::quran_cli::CommandOutput::err(exit_code::USAGE, format!("not a directory: {}", docs.display()));
                if json { println!("{}", serde_json::to_string_pretty(&o.json).unwrap_or_default()); } else { println!("{}", o.human); }
                return o.exit;
            }
            let source_path = docs.to_string_lossy().to_string();
            match reader.import_documents(&project, &source_path, &glob_str, files).await {
                Ok((imported, skipped)) => {
                    println!("imported {} document(s) (skipped {})", imported, skipped);
                    exit_code::OK
                }
                Err(e) => {
                    let o = application::quran_cli::CommandOutput::err(exit_code::GENERIC, e.to_string());
                    if json { println!("{}", serde_json::to_string_pretty(&o.json).unwrap_or_default()); } else { println!("{}", o.human); }
                    o.exit
                }
            }
        }
        RagAction::Query { project, query, filter: _, limit } => {
            let project_dir = std::path::PathBuf::from(&project);
            let db_path = project_dir.join("index.db");
            let db_path_str = db_path.to_string_lossy().to_string();
            let migrations = migrations_dir();
            let _ = storage_sqlite::migrate::apply_migrations(&db_path_str, &migrations).await;
            let db = match SqliteDatabase::new(&db_path_str, 4, true).await {
                Ok(db) => Arc::new(db),
                Err(e) => {
                    let o = application::quran_cli::CommandOutput::err(exit_code::GENERIC, e.to_string());
                    if json { println!("{}", serde_json::to_string_pretty(&o.json).unwrap_or_default()); } else { println!("{}", o.human); }
                    return o.exit;
                }
            };
            let backend = application::rag_tools::RagToolBackend::from_db(db.clone());
            match backend.backend_query(&project, &query).await {
                Ok(tr) => {
                    let hits = tr.results["hits"].as_array().cloned().unwrap_or_default();
                    if hits.is_empty() {
                        println!("0 hit(s) for \"{}\"", query);
                        return exit_code::OK;
                    }
                    println!("{} hit(s)", hits.len());
                    for h in &hits {
                        let rank = h.get("rank").and_then(|v| v.as_i64()).unwrap_or(0);
                        let doc_id = h.get("doc_id").and_then(|v| v.as_str()).unwrap_or("");
                        let score = h.get("score").cloned().unwrap_or(serde_json::Value::Null);
                        println!("{}. {} — {}", rank, doc_id, score);
                    }
                    let rd = &tr.results["route_decision"];
                    let router = rd.get("router").and_then(|v| v.as_str()).unwrap_or("");
                    let checksum = rd.get("checksum").and_then(|v| v.as_str()).unwrap_or("");
                    println!("route: {} (checksum {})", router, checksum);
                    let _ = limit;
                    exit_code::OK
                }
                Err(e) => {
                    let o = application::quran_cli::CommandOutput::err(exit_code::GENERIC, e.to_string());
                    if json { println!("{}", serde_json::to_string_pretty(&o.json).unwrap_or_default()); } else { println!("{}", o.human); }
                    o.exit
                }
            }
        }
        RagAction::Debug { project, query } => {
            let project_dir = std::path::PathBuf::from(&project);
            let db_path = project_dir.join("index.db");
            let db_path_str = db_path.to_string_lossy().to_string();
            let migrations = migrations_dir();
            let _ = storage_sqlite::migrate::apply_migrations(&db_path_str, &migrations).await;
            let db = match SqliteDatabase::new(&db_path_str, 4, true).await {
                Ok(db) => Arc::new(db),
                Err(e) => {
                    let o = application::quran_cli::CommandOutput::err(exit_code::GENERIC, e.to_string());
                    if json { println!("{}", serde_json::to_string_pretty(&o.json).unwrap_or_default()); } else { println!("{}", o.human); }
                    return o.exit;
                }
            };
            let backend = application::rag_tools::RagToolBackend::from_db(db.clone());
            match backend.backend_debug(&project, &query).await {
                Ok(tr) => {
                    let rd = &tr.results["route_decision"];
                    let router = rd.get("router").and_then(|v| v.as_str()).unwrap_or("");
                    println!("route: {}", router);
                    let tools = rd.get("tools").and_then(|v| v.as_array()).cloned().unwrap_or_default();
                    let tools_str: Vec<String> = tools.iter().filter_map(|v| v.as_str().map(String::from)).collect();
                    println!("  tools: {}", tools_str.join(", "));
                    let sources = rd.get("sources").and_then(|v| v.as_array()).cloned().unwrap_or_default();
                    let sources_str: Vec<String> = sources.iter().filter_map(|v| v.as_str().map(String::from)).collect();
                    println!("  sources: {}", sources_str.join(", "));
                    let candidates = rd.get("candidates").and_then(|v| v.as_u64()).unwrap_or(0);
                    let merged = rd.get("merged").and_then(|v| v.as_u64()).unwrap_or(0);
                    let reranked = rd.get("reranked").and_then(|v| v.as_u64()).unwrap_or(0);
                    let final_count = rd.get("final_ranking").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
                    println!("  candidates: {}", candidates);
                    println!("  merged (RRF): {}", merged);
                    println!("  reranked: {}", reranked);
                    println!("  final: {}", final_count);
                    let checksum = rd.get("checksum").and_then(|v| v.as_str()).unwrap_or("");
                    println!("  checksum: {}", checksum);
                    let excluded = rd.get("excluded").and_then(|v| v.as_u64()).unwrap_or(0);
                    if excluded > 0 {
                        println!("  excluded (non-active): {}", excluded);
                    }
                    exit_code::OK
                }
                Err(e) => {
                    let o = application::quran_cli::CommandOutput::err(exit_code::GENERIC, e.to_string());
                    if json { println!("{}", serde_json::to_string_pretty(&o.json).unwrap_or_default()); } else { println!("{}", o.human); }
                    o.exit
                }
            }
        }
        RagAction::Source { action } => {
            match action {
                SourceAction::Activate { project, source } => {
                    let project_dir = std::path::PathBuf::from(&project);
                    let db_path = project_dir.join("index.db");
                    let db_path_str = db_path.to_string_lossy().to_string();
                    let migrations = migrations_dir();
                    let _ = storage_sqlite::migrate::apply_migrations(&db_path_str, &migrations).await;
                    let db = match SqliteDatabase::new(&db_path_str, 4, true).await {
                        Ok(db) => Arc::new(db),
                        Err(e) => {
                            let o = application::quran_cli::CommandOutput::err(exit_code::GENERIC, e.to_string());
                            if json { println!("{}", serde_json::to_string_pretty(&o.json).unwrap_or_default()); } else { println!("{}", o.human); }
                            return o.exit;
                        }
                    };
                    let backend = application::rag_tools::RagToolBackend::from_db(db.clone());
                    match backend.backend_source_status(&project, &source, "active").await {
                        Ok(_) => { println!("activated {}", source); exit_code::OK }
                        Err(e) => { let o = application::quran_cli::CommandOutput::err(exit_code::GENERIC, e.to_string()); if json { println!("{}", serde_json::to_string_pretty(&o.json).unwrap_or_default()); } else { println!("{}", o.human); } o.exit }
                    };
                }
                SourceAction::Deactivate { project, source } => {
                    let project_dir = std::path::PathBuf::from(&project);
                    let db_path = project_dir.join("index.db");
                    let db_path_str = db_path.to_string_lossy().to_string();
                    let migrations = migrations_dir();
                    let _ = storage_sqlite::migrate::apply_migrations(&db_path_str, &migrations).await;
                    let db = match SqliteDatabase::new(&db_path_str, 4, true).await {
                        Ok(db) => Arc::new(db),
                        Err(e) => {
                            let o = application::quran_cli::CommandOutput::err(exit_code::GENERIC, e.to_string());
                            if json { println!("{}", serde_json::to_string_pretty(&o.json).unwrap_or_default()); } else { println!("{}", o.human); }
                            return o.exit;
                        }
                    };
                    let backend = application::rag_tools::RagToolBackend::from_db(db.clone());
                    match backend.backend_source_status(&project, &source, "deactivated").await {
                        Ok(_) => { println!("deactivated {}", source); exit_code::OK }
                        Err(e) => { let o = application::quran_cli::CommandOutput::err(exit_code::GENERIC, e.to_string()); if json { println!("{}", serde_json::to_string_pretty(&o.json).unwrap_or_default()); } else { println!("{}", o.human); } o.exit }
                    };
                }
                SourceAction::Tombstone { project, source, yes } => {
                    if !yes {
                        // interactive confirmation
                        if !json {
                            print!("Type APPROVE to tombstone `{}`: ", source);
                            use std::io::Write;
                            let _ = std::io::stdout().flush();
                        }
                        let mut input = String::new();
                        if std::io::stdin().read_line(&mut input).is_err() || input.trim() != "APPROVE" {
                            println!("refusing to tombstone without --yes");
                            return exit_code::GENERIC;
                        }
                    }
                    let project_dir = std::path::PathBuf::from(&project);
                    let db_path = project_dir.join("index.db");
                    let db_path_str = db_path.to_string_lossy().to_string();
                    let migrations = migrations_dir();
                    let _ = storage_sqlite::migrate::apply_migrations(&db_path_str, &migrations).await;
                    let db = match SqliteDatabase::new(&db_path_str, 4, true).await {
                        Ok(db) => Arc::new(db),
                        Err(e) => {
                            let o = application::quran_cli::CommandOutput::err(exit_code::GENERIC, e.to_string());
                            if json { println!("{}", serde_json::to_string_pretty(&o.json).unwrap_or_default()); } else { println!("{}", o.human); }
                            return o.exit;
                        }
                    };
                    let backend = application::rag_tools::RagToolBackend::from_db(db.clone());
                    match backend.backend_source_status(&project, &source, "tombstoned").await {
                        Ok(tr) => {
                            let seq = tr.results.get("audit_seq").and_then(|v| v.as_i64()).unwrap_or(0);
                            println!("tombstoned {} (audit #{})", source, seq);
                            exit_code::OK
                        }
                        Err(e) => {
                            let o = application::quran_cli::CommandOutput::err(exit_code::GENERIC, e.to_string());
                            if json { println!("{}", serde_json::to_string_pretty(&o.json).unwrap_or_default()); } else { println!("{}", o.human); }
                            o.exit
                        }
                    };
                }
            }
            exit_code::OK
        }
    }
}

fn migrations_dir() -> std::path::PathBuf {
    let from_cwd = std::path::PathBuf::from("migrations/sqlite");
    if from_cwd.is_dir() {
        return from_cwd;
    }
    let from_manifest =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../migrations/sqlite");
    if from_manifest.is_dir() {
        return from_manifest;
    }
    from_cwd
}

fn walk_dir(base: &std::path::Path, dir: &std::path::Path, glob: &str, out: &mut Vec<(String, String, serde_json::Value)>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk_dir(base, &path, glob, out);
            } else if path.is_file() {
                if matches_glob(&path, glob) {
                    if let Ok(text) = std::fs::read_to_string(&path) {
                        let rel = path.strip_prefix(base).unwrap_or(&path).to_string_lossy().to_string();
                        out.push((rel, text, serde_json::json!({})));
                    }
                }
            }
        }
    }
}

fn extract_glob(cfg: &str) -> Option<String> {
    for line in cfg.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("glob") {
            let rest = rest.trim_start_matches(|c: char| c == ' ' || c == '=' || c == ' ');
            let rest = rest.trim_matches('"');
            if !rest.is_empty() {
                return Some(rest.to_string());
            }
        }
    }
    None
}

fn matches_glob(path: &std::path::Path, glob: &str) -> bool {
    // minimal glob: handle **/*.ext and * patterns
    let g = glob.trim();
    if g == "**/*" || g == "*" {
        return true;
    }
    if let Some(ext) = g.strip_prefix("**/*.") {
        // ext may include trailing
        let ext = ext.trim_end_matches('*');
        if let Some(name_ext) = path.extension().and_then(|e| e.to_str()) {
            return name_ext == ext;
        }
        return false;
    }
    if let Some(ext) = g.strip_prefix("*.") {
        let ext = ext.trim_end_matches('*');
        if let Some(name_ext) = path.extension().and_then(|e| e.to_str()) {
            return name_ext == ext;
        }
        return false;
    }
    // fallback: suffix match after last '*'
    if let Some(idx) = g.rfind('*') {
        let suffix = &g[idx+1..];
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        return name.ends_with(suffix);
    }
    true
}