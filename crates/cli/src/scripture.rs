//! `qai scripture …` commands.

use clap::Subcommand;
use std::sync::Arc;
use storage_sqlite::SqliteDatabase;

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
    /// Parallel passages / verse links.
    Link {
        #[command(subcommand)]
        action: LinkAction,
    },
}

#[derive(Subcommand)]
pub enum LinkAction {
    /// Run computational matching for a scripture verse.
    Suggest {
        scripture_verse_id: String,
    },
    /// Confirm a link (owner gate).
    Confirm {
        link_id: String,
        #[arg(long)]
        reviewer: String,
    },
    /// Reject a link (owner gate).
    Reject {
        link_id: String,
        #[arg(long)]
        reviewer: String,
    },
    /// List links for a Quran verse.
    List {
        quran_ref: String,
        #[arg(long)]
        include_uncertain: bool,
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
    use application::verse_link::VerseLinker;
    use tool_registry::{ScriptureBackend, ScriptureRef};
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
        ScriptureAction::Link { action } => {
            let service = backend.links();
            // ensure service type
            let _ = &service;
            match action {
                LinkAction::Suggest { scripture_verse_id } => {
                    match service.suggest_scripture_links(&scripture_verse_id).await {
                        Ok(links) => {
                            // label uncertain explicitly
                            let human = format!("scripture.link.suggest: {} uncertain suggestions", links.len());
                            let jsonv = serde_json::to_value(&links).unwrap_or(serde_json::Value::Null);
                            application::quran_cli::CommandOutput::ok(human, jsonv)
                        }
                        Err(e) => application::quran_cli::CommandOutput::err(exit_code::GENERIC, e.to_string()),
                    }
                }
                LinkAction::Confirm { link_id, reviewer } => {
                    let id = match link_id.parse::<application::verse_link::VerseLinkId>() {
                        Ok(v) => v,
                        Err(_) => {
                            let o = application::quran_cli::CommandOutput::err(exit_code::GENERIC, "invalid link id".to_string());
                            if json { println!("{}", serde_json::to_string_pretty(&o.json).unwrap_or_default()); } else { println!("{}", o.human); }
                            return o.exit;
                        }
                    };
                    match service.confirm_link(&id, &reviewer).await {
                        Ok(()) => application::quran_cli::CommandOutput::ok("confirmed".to_string(), serde_json::json!({"link_id": link_id})),
                        Err(e) => application::quran_cli::CommandOutput::err(exit_code::GENERIC, e.to_string()),
                    }
                }
                LinkAction::Reject { link_id, reviewer } => {
                    let id = match link_id.parse::<application::verse_link::VerseLinkId>() {
                        Ok(v) => v,
                        Err(_) => {
                            let o = application::quran_cli::CommandOutput::err(exit_code::GENERIC, "invalid link id".to_string());
                            if json { println!("{}", serde_json::to_string_pretty(&o.json).unwrap_or_default()); } else { println!("{}", o.human); }
                            return o.exit;
                        }
                    };
                    match service.reject_link(&id, &reviewer).await {
                        Ok(()) => application::quran_cli::CommandOutput::ok("rejected".to_string(), serde_json::json!({"link_id": link_id})),
                        Err(e) => application::quran_cli::CommandOutput::err(exit_code::GENERIC, e.to_string()),
                    }
                }
                LinkAction::List { quran_ref, include_uncertain } => {
                    let parts: Vec<&str> = quran_ref.split(':').collect();
                    if parts.len() != 2 {
                        let o = application::quran_cli::CommandOutput::err(exit_code::GENERIC, "quran ref must be surah:ayah".to_string());
                        if json { println!("{}", serde_json::to_string_pretty(&o.json).unwrap_or_default()); } else { println!("{}", o.human); }
                        return o.exit;
                    }
                    let surah: u16 = match parts[0].parse() { Ok(v) => v, Err(_) => {
                        let o = application::quran_cli::CommandOutput::err(exit_code::GENERIC, "invalid surah".to_string());
                        if json { println!("{}", serde_json::to_string_pretty(&o.json).unwrap_or_default()); } else { println!("{}", o.human); }
                        return o.exit;
                    }};
                    let ayah: u32 = match parts[1].parse() { Ok(v) => v, Err(_) => {
                        let o = application::quran_cli::CommandOutput::err(exit_code::GENERIC, "invalid ayah".to_string());
                        if json { println!("{}", serde_json::to_string_pretty(&o.json).unwrap_or_default()); } else { println!("{}", o.human); }
                        return o.exit;
                    }};
                    match service.get_links_for_verse(surah, ayah, include_uncertain).await {
                        Ok(links) => {
                            let human = format!("scripture.link.list: {} links", links.len());
                            let jsonv = serde_json::to_value(&links).unwrap_or(serde_json::Value::Null);
                            application::quran_cli::CommandOutput::ok(human, jsonv)
                        }
                        Err(e) => application::quran_cli::CommandOutput::err(exit_code::GENERIC, e.to_string()),
                    }
                }
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
