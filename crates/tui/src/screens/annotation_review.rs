//! Annotation-review screen: queued proposals with decision status.

use ratatui::{
    Frame,
    layout::Constraint,
    widgets::{Block, Borders, Row, Table},
};

use crate::sanitize_terminal_text;

/// One queued annotation proposal.
#[derive(Debug, Clone)]
pub struct AnnotationRow {
    /// Assertion id.
    pub id: String,
    /// Assertion kind.
    pub kind: String,
    /// Decision (`pending`, `accepted`, …).
    pub decision: String,
    /// One-line claim summary.
    pub summary: String,
}

/// Render the pending-review queue in deterministic order.
pub fn render_annotation_review(frame: &mut Frame, rows: &[AnnotationRow]) {
    let area = frame.area();
    let table_rows: Vec<Row> = rows
        .iter()
        .map(|row| {
            Row::new(vec![
                row.id.clone(),
                row.kind.clone(),
                row.decision.clone(),
                sanitize_terminal_text(&row.summary),
            ])
        })
        .collect();
    let table = Table::new(
        table_rows,
        [
            Constraint::Length(16),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Min(10),
        ],
    )
    .block(Block::default().borders(Borders::ALL).title("Annotation review"));
    frame.render_widget(table, area);
}
