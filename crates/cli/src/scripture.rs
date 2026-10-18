//! `qai scripture …` commands.

use clap::Subcommand;
use std::sync::Arc;
use storage_sqlite::SqliteDatabase;
use tool_registry::ScriptureBackend;

use crate::exit_code;

#[derive(Subcommand)]
pub enum ScriptureAction {
    /// List editions.
    Editions,
    /// Fetch one verse.
    Get {
        edition: String,
        book: String,
        chapter: u32,
        verse: u32,
    },
    /// Search scripture text.
    Search {
        query: String,
        #[arg(long)]
        edition: Option<String>,
    },
    /// Get translations for a verse.
    Translations {
        verse_id: String,
        #[arg(long)]
        language: Option<String>,
    },
}

pub fn handle_scripture(action: ScriptureAction, db_path: &str, json: bool) -> i32 {
    let runtime = match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
        Ok(rt) => rt,
        Err(err) => {
            eprintln!("failed to start async runtime: {err}");
            return exit_code::INTERNAL;
        }
    };
    runtime.block_on(handle_scripture_async(action, db_path, json))
}

async fn handle_scripture_async(action: ScriptureAction, db_path: &str, json: bool) -> i32 {
    use application::scripture_reader::ScriptureReaderService;
    use application::scripture_tools::ScriptureToolBackend;
    use tool_registry::ScriptureRef;
    use tool_registry::ScriptureSearchToolParams;

    let db = match SqliteDatabase::new(db_path, 4, true).await {
        Ok(db) => Arc::new(db),
        Err(err) => {
            let o = application::quran_cli::CommandOutput::err(exit_code::GENERIC, err.to_string());
            if json {
                println!("{}", serde_json::to_string_pretty(&o.json).unwrap_or_default());
            } else {
                println!("{}", o.human);
            }
            return o.exit;
        }
    };
    let reader = Arc::new(ScriptureReaderService::new(db.clone()));
    let backend = ScriptureToolBackend::new(reader.clone());
    // ensure search service used? backend has own search; ok

    let output = match action {
        ScriptureAction::Editions => {
            match backend.backend_list_editions().await {
                Ok(v) => application::quran_cli::CommandOutput::ok("editions".to_string(), v),
                Err(e) => application::quran_cli::CommandOutput::err(exit_code::GENERIC, e.to_string()),
            }
        }
        ScriptureAction::Get { edition, book, chapter, verse } => {
            let reference = ScriptureRef { edition, book, chapter, verse };
            match backend.backend_get_scripture(&reference).await {
                Ok((view, _meta)) => {
                    let human = format!("{}", view.verse.text);
                    let jsonv = serde_json::to_value(&view).unwrap_or(serde_json::Value::Null);
                    application::quran_cli::CommandOutput::ok(human, jsonv)
                }
                Err(e) => application::quran_cli::CommandOutput::err(exit_code::GENERIC, e.to_string()),
            }
        }
        ScriptureAction::Search { query, edition } => {
            let params = ScriptureSearchToolParams { query: query.clone(), edition, limit: None };
            match backend.backend_search_scripture(&params).await {
                Ok(result) => {
                    let human = format!("scripture.search: {} hits", result.canonical_references.len());
                    let jsonv = serde_json::to_value(&result).unwrap_or(serde_json::Value::Null);
                    application::quran_cli::CommandOutput::ok(human, jsonv)
                }
                Err(e) => application::quran_cli::CommandOutput::err(exit_code::GENERIC, e.to_string()),
            }
        }
        ScriptureAction::Translations { verse_id, language } => {
            match backend.backend_get_translation(&verse_id, language.as_deref().unwrap_or("")).await {
                Ok(v) => application::quran_cli::CommandOutput::ok("translations".to_string(), v),
                Err(e) => application::quran_cli::CommandOutput::err(exit_code::GENERIC, e.to_string()),
            }
        }
    };

    finish(output, json)
}

fn finish(output: application::quran_cli::CommandOutput, json: bool) -> i32 {
    if json {
        println!("{}", serde_json::to_string_pretty(&output.json).unwrap_or_default());
    } else {
        let human = output.human.strip_suffix('\n').unwrap_or(&output.human);
        println!("{human}");
    }
    output.exit
}
