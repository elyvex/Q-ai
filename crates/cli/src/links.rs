//! `qai links …` — verse-link inspection (Phase 6 SC3).

use clap::Subcommand;
use std::sync::Arc;
use storage_sqlite::SqliteDatabase;

use crate::exit_code;

#[derive(Subcommand)]
pub enum LinksAction {
    /// Show hadith/tafsir links for a verse.
    Verse {
        surah: u16,
        ayah: u32,
        #[arg(long)]
        include_uncertain: bool,
    },
}

pub fn handle_links(action: LinksAction, db_path: &str, json: bool) -> i32 {
    let runtime = match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
        Ok(rt) => rt,
        Err(err) => {
            eprintln!("failed to start async runtime: {err}");
            return exit_code::INTERNAL;
        }
    };
    runtime.block_on(handle_links_async(action, db_path, json))
}

async fn handle_links_async(action: LinksAction, db_path: &str, json: bool) -> i32 {
    use application::quran_cli::CommandOutput;
    use application::verse_link::VerseLinker;

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
    let service = application::verse_link::VerseLinkService::new(db);

    let output: CommandOutput = match action {
        LinksAction::Verse { surah, ayah, include_uncertain } => {
            match VerseLinker::get_links_for_verse(&service, surah, ayah, include_uncertain).await {
                Ok(links) => {
                    let human = format!("verse.links: {} links", links.len());
                    let jsonv = serde_json::to_value(&links).unwrap_or(serde_json::Value::Null);
                    CommandOutput::ok(human, jsonv)
                }
                Err(err) => CommandOutput::err(exit_code::GENERIC, err.to_string()),
            }
        }
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&output.json).unwrap_or_default());
    } else {
        println!("{}", output.human);
    }
    output.exit
}
