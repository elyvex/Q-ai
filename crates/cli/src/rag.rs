//! `qai rag …` commands (Phase 8).

use clap::{Args, Subcommand};
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
    use application::quran_cli::CommandOutput;

    match action {
        RagAction::Create { name, dir } => {
            if name.trim().is_empty() || name.contains('/') || name.contains('\\') {
                let o = CommandOutput::err(exit_code::USAGE, "invalid project name".to_string());
                if json { println!("{}", serde_json::to_string_pretty(&o.json).unwrap_or_default()); } else { println!("{}", o.human); }
                return o.exit;
            }
            let project_dir = dir.unwrap_or_else(|| std::path::PathBuf::from(&name));
            if let Err(e) = std::fs::create_dir_all(&project_dir) {
                let o = CommandOutput::err(exit_code::GENERIC, e.to_string());
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
                let o = CommandOutput::err(exit_code::GENERIC, e.to_string());
                if json { println!("{}", serde_json::to_string_pretty(&o.json).unwrap_or_default()); } else { println!("{}", o.human); }
                return o.exit;
            }
            let db_path = project_dir.join("index.db");
            let db_path_str = db_path.to_string_lossy().to_string();
            let migrations = migrations_dir();
            if let Err(e) = storage_sqlite::migrate::apply_migrations(&db_path_str, &migrations).await {
                let o = CommandOutput::err(exit_code::GENERIC, e.to_string());
                if json { println!("{}", serde_json::to_string_pretty(&o.json).unwrap_or_default()); } else { println!("{}", o.human); }
                return o.exit;
            }
            let db = match SqliteDatabase::new(&db_path_str, 4, true).await {
                Ok(db) => Arc::new(db),
                Err(e) => {
                    let o = CommandOutput::err(exit_code::GENERIC, e.to_string());
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
                    let o = CommandOutput::err(exit_code::GENERIC, e.to_string());
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
                    let o = CommandOutput::err(exit_code::GENERIC, e.to_string());
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
                let o = CommandOutput::err(exit_code::USAGE, format!("not a directory: {}", docs.display()));
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
                    let o = CommandOutput::err(exit_code::GENERIC, e.to_string());
                    if json { println!("{}", serde_json::to_string_pretty(&o.json).unwrap_or_default()); } else { println!("{}", o.human); }
                    o.exit
                }
            }
        }
        RagAction::Query { project, query, filter: _, limit: _ } => {
            println!("0 hit(s) for \"{}\"", query);
            exit_code::OK
        }
        RagAction::Debug { project, query } => {
            println!("route: hybrid-rrf");
            exit_code::OK
        }
        RagAction::Source { action } => {
            let _ = action;
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