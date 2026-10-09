//! Graph navigation screen: edge provenance with verbatim truncation.
//!
//! A truncated payload renders `truncated` + `incomplete_reason` verbatim
//! (ADR-0217) — never as "no path" or absence (T-05-13).

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::sanitize_terminal_text;

/// Pre-rendered graph view: display lines plus the honest truncation state.
///
/// Mapping rule: built ONLY from a `NeighborsOutput` (or sibling payload) —
/// `lines` from nodes/edges with provenance, `truncated`/`incomplete_reason`
/// copied verbatim.
#[derive(Debug, Clone)]
pub struct GraphNavView {
    /// Display lines (node/edge rows with provenance).
    pub lines: Vec<String>,
    /// Whether a budget cut the search short.
    pub truncated: bool,
    /// Why the search stopped early, verbatim.
    pub incomplete_reason: Option<String>,
}

/// Render the navigation view with verbatim truncation state.
pub fn render_graph_nav(frame: &mut Frame, view: &GraphNavView) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(4), Constraint::Length(3)])
        .split(area);

    let items: Vec<ListItem> =
        view.lines.iter().map(|line| ListItem::new(sanitize_terminal_text(line))).collect();
    let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Graph"));
    frame.render_widget(list, chunks[0]);

    let (text, color) = if view.truncated {
        (
            format!(
                "truncated: {}",
                sanitize_terminal_text(
                    view.incomplete_reason.as_deref().unwrap_or("reason withheld")
                )
            ),
            Color::Yellow,
        )
    } else {
        ("complete".to_string(), Color::Green)
    };
    let status = Paragraph::new(text).style(Style::default().fg(color));
    frame.render_widget(status, chunks[1]);
}
