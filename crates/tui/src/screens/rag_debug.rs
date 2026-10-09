//! RAG debug screen: typed unavailability, never fabricated retrieval.
//!
//! Until Phase 8 wires retrieval, the only honest state is `Unavailable`.
//! There is deliberately no retrieval code path behind this screen (D-11,
//! T-05-14): rendering results here would present fiction as evidence.

use ratatui::{
    Frame,
    widgets::{Block, Borders, Paragraph},
};

/// RAG debug state. The available variant ships in Phase 8.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RagState {
    /// No RAG project is configured on this machine.
    Unavailable,
}

/// Render the typed RAG state.
pub fn render_rag_debug(frame: &mut Frame, state: &RagState) {
    let area = frame.area();
    let text = match state {
        RagState::Unavailable => {
            "RAG not configured / unavailable — retrieval lands in Phase 8.\nNo results are shown because none were retrieved."
        }
    };
    let body =
        Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("RAG debug"));
    frame.render_widget(body, area);
}
