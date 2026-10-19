//! `qai tafsir …` commands.

use clap::Subcommand;
use std::sync::Arc;
use storage_sqlite::SqliteDatabase;
use tool_registry::TafsirBackend;
use tool_registry::TafsirRef;

use crate::exit_code;

#[derive(Subcommand)]
pub enum TafsirAction {
    /// Fetch one tafsir record.
    Get {
        /// `work_slug[:volume[:page]]` or `work_slug:chapter`
        reference: String,
    },
    /// Search tafsir text.
    Search {
        query: String,
        #[arg(long)]
        work: Option<String>,
        #[arg(long)]
        profile: Option<String>,
    },
}

pub fn handle_tafsir(action: TafsirAction, db_path: &str, json: bool) -> i32 {
    let runtime = match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
        Ok(rt) => rt,
        Err(err) => {
            eprintln!("failed to start async runtime: {err}");
            return exit_code::INTERNAL;
        }
    };
    runtime.block_on(handle_tafsir_async(action, db_path, json))
}

async fn handle_tafsir_async(action: TafsirAction, db_path: &str, json: bool) -> i32 {
    use application::quran_cli::CommandOutput;
    use application::tafsir_reader::TafsirReaderService;
    use application::tafsir_search::TafsirSearchService;
    use application::tafsir_tools::TafsirToolBackend;
    use tool_registry::TafsirSearchToolParams;

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
    let reader = Arc::new(TafsirReaderService::new(db.clone()));
    let search = Arc::new(TafsirSearchService::new(db, reader.clone()));
    let backend = TafsirToolBackend::new(reader, search);

    let output: CommandOutput = match action {
        TafsirAction::Get { reference } => {
            if reference.trim().is_empty() {
                return finish(
                    CommandOutput::err(exit_code::USAGE, "bad reference".to_string()),
                    json,
                );
            }
            let parsed = match TafsirRef::parse(&reference) {
                Some(r) => r,
                None => {
                    return finish(
                        CommandOutput::err(exit_code::USAGE, "bad reference".to_string()),
                        json,
                    );
                }
            };
            match backend.backend_get_tafsir(&parsed).await {
                Ok((view, _meta)) => {
                    let human = format!("{}: {}", view.work_slug, view.text);
                    let jsonv = serde_json::to_value(&view).unwrap_or(serde_json::Value::Null);
                    CommandOutput::ok(human, jsonv)
                }
                Err(err) => CommandOutput::err(exit_code::GENERIC, err.to_string()),
            }
        }
        TafsirAction::Search { query, work, profile: _ } => {
            let params =
                TafsirSearchToolParams { query: query.clone(), work_slug: work, limit: None };
            match backend.backend_search_tafsir(&params).await {
                Ok(result) => {
                    let human =
                        format!("tafsir.search: {} hits", result.canonical_references.len());
                    let jsonv = serde_json::to_value(&result).unwrap_or(serde_json::Value::Null);
                    CommandOutput::ok(human, jsonv)
                }
                Err(err) => CommandOutput::err(exit_code::GENERIC, err.to_string()),
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
