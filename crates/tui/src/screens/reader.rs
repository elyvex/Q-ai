//! Reader screen: canonical Arabic in its own styled region.
//!
//! Mapping rule (the canonical-slot invariant): the `arabic` string passed
//! here originates ONLY from `AyahView.canonical.arabic_text()` and the
//! `reference` ONLY from `AyahView.canonical.reference()` — see
//! `App::reader_view`. No translation, gloss, or token string may occupy
//! the canonical region.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

use crate::sanitize_terminal_text;

/// Render one ayah: canonical Arabic on top, translations below.
/// Every dataset string is sanitized before render (T-05-12).
pub fn render_reader(
    frame: &mut Frame,
    reference: &str,
    arabic: &str,
    translations: &[(String, String)],
) {
    let area = frame.area();
    let mut constraints = vec![Constraint::Length(3), Constraint::Min(5)];
    constraints.extend(translations.iter().map(|_| Constraint::Length(3)));
    let chunks =
        Layout::default().direction(Direction::Vertical).constraints(constraints).split(area);

    let title = Paragraph::new(sanitize_terminal_text(reference))
        .block(Block::default().borders(Borders::ALL).title("Reader"));
    frame.render_widget(title, chunks[0]);

    let canonical = Paragraph::new(sanitize_terminal_text(arabic))
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::ALL).title("Canonical"));
    frame.render_widget(canonical, chunks[1]);

    for (i, (slug, text)) in translations.iter().enumerate() {
        let body = Paragraph::new(sanitize_terminal_text(text))
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL).title(format!("Translation: {slug}")));
        frame.render_widget(body, chunks[i + 2]);
    }
}
