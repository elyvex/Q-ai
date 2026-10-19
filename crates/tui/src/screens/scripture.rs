//! Scripture panels for the TUI (Phase 9).

use ratatui::{
    Frame,
    widgets::{Block, Borders, List, ListItem},
};

use crate::sanitize_terminal_text;

#[derive(Debug, Clone)]
pub struct ScriptureBrowseRow {
    pub reference: String,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct ParallelPassageRow {
    pub reference: String,
    pub marker: String,
}

#[derive(Debug, Clone)]
pub struct CrossDomainRow {
    pub domain: String,
    pub reference: String,
    pub snippet: String,
}

pub fn render_scripture_browse(frame: &mut Frame, rows: &[ScriptureBrowseRow]) {
    let area = frame.area();
    let items: Vec<ListItem> = rows
        .iter()
        .map(|r| ListItem::new(format!("{} — {}", sanitize_terminal_text(&r.reference), sanitize_terminal_text(&r.text))))
        .collect();
    let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Scripture browse"));
    frame.render_widget(list, area);
}

pub fn render_parallel_passages(frame: &mut Frame, rows: &[ParallelPassageRow]) {
    let area = frame.area();
    let items: Vec<ListItem> = rows
        .iter()
        .map(|r| ListItem::new(format!("{} [{}]", sanitize_terminal_text(&r.reference), sanitize_terminal_text(&r.marker))))
        .collect();
    let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Parallel passages (uncertain labeled)"));
    frame.render_widget(list, area);
}

pub fn render_cross_domain(frame: &mut Frame, rows: &[CrossDomainRow]) {
    let area = frame.area();
    let items: Vec<ListItem> = rows
        .iter()
        .map(|r| ListItem::new(format!("[{}] {} — {}", sanitize_terminal_text(&r.domain), sanitize_terminal_text(&r.reference), sanitize_terminal_text(&r.snippet))))
        .collect();
    let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Cross-domain search (domain toggles)"));
    frame.render_widget(list, area);
}

pub fn render_scripture(frame: &mut Frame, rows: &[ScriptureBrowseRow]) {
    render_scripture_browse(frame, rows);
}
