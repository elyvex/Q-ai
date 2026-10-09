//! Dashboard screen: cockpit overview with the palette hint.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use super::Screen;
use crate::App;

/// Render the dashboard: title, screen inventory, and the palette hint.
pub fn render_dashboard(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(4), Constraint::Length(3)])
        .split(area);

    let title = Paragraph::new("Q-ai — Quran research cockpit")
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    let items: Vec<ListItem> = Screen::all()
        .iter()
        .map(|s| {
            let marker = if *s == app.screen() { ">" } else { " " };
            ListItem::new(format!("{marker} {}", s.title()))
        })
        .collect();
    let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Screens"));
    frame.render_widget(list, chunks[1]);

    let hint = Paragraph::new("qai tui — press : for palette, q to quit")
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(hint, chunks[2]);
}
