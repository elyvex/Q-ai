//! Search screen: service-ordered hits with canonical snippets.
//!
//! Hits are rendered in the exact order received — the screen never
//! re-sorts. Every snippet is sanitized before render (T-05-12).

use ratatui::{
    Frame,
    widgets::{Block, Borders, List, ListItem},
};

use crate::sanitize_terminal_text;

/// One rendered hit: reference + canonical-display snippet.
///
/// Mapping rule: `snippet` originates ONLY from the hit's canonical
/// quotation (`hit.quotation().arabic_text()`), never from generated text.
#[derive(Debug, Clone)]
pub struct SearchRow {
    /// Fully-qualified canonical reference.
    pub reference: String,
    /// Canonical-display snippet.
    pub snippet: String,
}

/// Render hits in service order.
pub fn render_search(frame: &mut Frame, hits: &[SearchRow]) {
    let area = frame.area();
    let items: Vec<ListItem> = hits
        .iter()
        .map(|hit| {
            ListItem::new(format!(
                "{} — {}",
                sanitize_terminal_text(&hit.reference),
                sanitize_terminal_text(&hit.snippet)
            ))
        })
        .collect();
    let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Search"));
    frame.render_widget(list, area);
}
