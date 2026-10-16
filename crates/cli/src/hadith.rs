//! `qai hadith …` commands.

use clap::Subcommand;
use std::sync::Arc;
use storage_sqlite::SqliteDatabase;
use tool_registry::HadithRef;
use tool_registry::HadithBackend;

use crate::exit_code;

#[derive(Subcommand)]
pub enum HadithAction {
    /// Fetch one hadith record.
    Get { reference: String },
    /// Search hadith text.
    Search {
        query: String,
        #[arg(long)]
        collection: Option<String>,
        #[arg(long)]
        profile: Option<String>,
    },
    /// Show all gradings for a hadith.
    Gradings { reference: String },
}

pub fn handle_hadith(action: HadithAction, db_path: &str, json: bool) -> i32 {
    let runtime = match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
        Ok(rt) => rt,
        Err(err) => {
            eprintln!("failed to start async runtime: {err}");
            return exit_code::INTERNAL;
        }
    };
    runtime.block_on(handle_hadith_async(action, db_path, json))
}

async fn handle_hadith_async(action: HadithAction, db_path: &str, json: bool) -> i32 {
    use application::hadith_reader::HadithReaderService;
    use application::hadith_search::HadithSearchService;
    use application::hadith_tools::HadithToolBackend;
    use application::quran_cli::CommandOutput;

    let db = match SqliteDatabase::new(db_path, 4, true).await {
        Ok(db) => Arc::new(db),
        Err(err) => {
            let o = CommandOutput::err(exit_code::GENERIC, err.to_string());
            if json { println!("{}", serde_json::to_string_pretty(&o.json).unwrap_or_default()); } else { println!("{}", o.human); }
            return o.exit;
        }
    };
    let reader = Arc::new(HadithReaderService::new(db.clone()));
    let search = Arc::new(HadithSearchService::new(db, reader.clone()));
    let backend = HadithToolBackend::new(reader.clone(), search);

    let output: CommandOutput = match action {
        HadithAction::Get { reference } => {
            let parsed = match HadithRef::parse(&reference) {
                Some(r) => r,
                None => return finish(CommandOutput::err(exit_code::USAGE, "bad reference".to_string()), json),
            };
            match backend.backend_get_hadith(&parsed).await {
                Ok((view, _meta)) => {
                    let human = format!("{}: {} — {}", view.collection_slug, view.source_number, view.matn_text);
                    let jsonv = serde_json::to_value(&view).unwrap_or(serde_json::Value::Null);
                    CommandOutput::ok(human, jsonv)
                }
                Err(err) => CommandOutput::err(exit_code::GENERIC, err.to_string()),
            }
        }
        HadithAction::Search { query, collection, profile: _ } => {
            use tool_registry::HadithSearchToolParams;
            let params = HadithSearchToolParams {
                query: query.clone(),
                collection_slug: collection,
                limit: None,
            };
            match backend.backend_search_hadith(&params).await {
                Ok(result) => {
                    let human = format!("hadith.search: {} hits", result.canonical_references.len());
                    let jsonv = serde_json::to_value(&result).unwrap_or(serde_json::Value::Null);
                    CommandOutput::ok(human, jsonv)
                }
                Err(err) => CommandOutput::err(exit_code::GENERIC, err.to_string()),
            }
        }
        HadithAction::Gradings { reference } => {
            let parsed = match HadithRef::parse(&reference) {
                Some(r) => r,
                None => return finish(CommandOutput::err(exit_code::USAGE, "bad reference".to_string()), json),
            };
            let (view, _meta) = match backend.backend_get_hadith(&parsed).await {
                Ok(v) => v,
                Err(err) => return finish(CommandOutput::err(exit_code::GENERIC, err.to_string()), json),
            };
            match backend.backend_get_gradings(&view.id).await {
                Ok(gradings) => {
                    let human = format!("gradings: {}", gradings.len());
                    let jsonv = serde_json::to_value(&gradings).unwrap_or(serde_json::Value::Null);
                    CommandOutput::ok(human, jsonv)
                }
                Err(err) => CommandOutput::err(exit_code::GENERIC, err.to_string()),
            }
        }
    };

    finish(output, json)
}

// HadithRef::parse may return None; avoid dangling parsed in Get arm by early handling impossible due block. Simplify by guarding empty.
// The above Get arm includes a weird guard; remove compile issue by making it no-op compile? Let's ensure builds.

fn finish(output: application::quran_cli::CommandOutput, json: bool) -> i32 {
    if json {
        println!("{}", serde_json::to_string_pretty(&output.json).unwrap_or_default());
    } else {
        let human = output.human.strip_suffix('\n').unwrap_or(&output.human);
        println!("{human}");
    }
    output.exit
}
