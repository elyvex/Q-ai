//! Ops screens: doctor/health, jobs, and index/corpus status over the
//! read-only `application` services (D-10/D-12). Skipped checks and drift
//! render honestly — never as pass.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Row, Table},
};

use crate::sanitize_terminal_text;
use application::quran_doctor::{CheckLevel, QuranDoctorCheck};

/// One job row for the jobs screen.
#[derive(Debug, Clone)]
pub struct JobRow {
    /// Job id (short prefix shown).
    pub id: String,
    /// Job kind (`quran.import`, …).
    pub kind: String,
    /// Lifecycle state.
    pub state: String,
    /// Last error, when failed.
    pub last_error: Option<String>,
}

/// Index/corpus status for the index screen.
#[derive(Debug, Clone)]
pub struct IndexStatus {
    /// Active corpus generation.
    pub generation: i64,
    /// Drift detail when the serving generation trails, if any.
    pub drift: Option<String>,
}

fn level_label(level: CheckLevel) -> (&'static str, Color) {
    match level {
        CheckLevel::Pass => ("PASS", Color::Green),
        CheckLevel::Warn => ("WARN", Color::Yellow),
        CheckLevel::Fail => ("FAIL", Color::Red),
        CheckLevel::Skipped => ("SKIP", Color::DarkGray),
    }
}

/// Render doctor families with honest status labels: `SKIP` stays `SKIP`.
pub fn render_doctor(frame: &mut Frame, checks: &[QuranDoctorCheck]) {
    let area = frame.area();
    let rows: Vec<Row> = checks
        .iter()
        .map(|check| {
            let (label, color) = level_label(check.status);
            Row::new(vec![
                label.to_string(),
                check.id.to_string(),
                sanitize_terminal_text(&check.summary),
            ])
            .style(Style::default().fg(color))
        })
        .collect();
    let table =
        Table::new(rows, [Constraint::Length(6), Constraint::Length(28), Constraint::Min(10)])
            .block(Block::default().borders(Borders::ALL).title("Doctor"));
    frame.render_widget(table, area);
}

/// Render jobs with state and last-error columns.
pub fn render_jobs(frame: &mut Frame, jobs: &[JobRow]) {
    let area = frame.area();
    let rows: Vec<Row> = jobs
        .iter()
        .map(|job| {
            Row::new(vec![
                job.id.chars().take(8).collect::<String>(),
                job.kind.clone(),
                job.state.clone(),
                sanitize_terminal_text(job.last_error.as_deref().unwrap_or("")),
            ])
        })
        .collect();
    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Length(16),
            Constraint::Length(12),
            Constraint::Min(10),
        ],
    )
    .block(Block::default().borders(Borders::ALL).title("Jobs"));
    frame.render_widget(table, area);
}

/// Render index/corpus generation; drift shows as `WARN`, never pass.
pub fn render_index_status(frame: &mut Frame, status: &IndexStatus) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(3)])
        .split(area);
    let generation = Paragraph::new(format!("corpus generation: {}", status.generation))
        .block(Block::default().borders(Borders::ALL).title("Index"));
    frame.render_widget(generation, chunks[0]);
    let (text, color) = match &status.drift {
        Some(drift) => (format!("WARN drift: {}", sanitize_terminal_text(drift)), Color::Yellow),
        None => ("PASS serving generation current".to_string(), Color::Green),
    };
    let drift = Paragraph::new(text).style(Style::default().fg(color));
    frame.render_widget(drift, chunks[1]);
}
