//! RAG debug screen: typed unavailability or full route trace (Phase 8).

use application::RouteDecision;
use ratatui::{
    Frame,
    widgets::{Block, Borders, Paragraph},
};

/// RAG debug state. `Available` carries a server-computed `RouteDecision`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RagState {
    /// No RAG project is configured on this machine.
    Unavailable,
    /// A route decision is available to render.
    Available {
        router: String,
        tools: Vec<String>,
        sources: Vec<String>,
        candidates: usize,
        merged: usize,
        reranked: usize,
        final_count: usize,
        checksum: String,
        excluded: usize,
    },
}

impl RagState {
    /// Build an `Available` state from a server-computed route decision.
    pub fn from_route_decision(rd: &RouteDecision) -> Self {
        Self::Available {
            router: rd.router.clone(),
            tools: rd.tools.clone(),
            sources: rd.sources.clone(),
            candidates: rd.candidates,
            merged: rd.merged,
            reranked: rd.reranked,
            final_count: rd.final_ranking.len(),
            checksum: rd.checksum.clone(),
            excluded: rd.excluded,
        }
    }
}

/// Render the typed RAG state.
pub fn render_rag_debug(frame: &mut Frame, state: &RagState) {
    let area = frame.area();
    let text = match state {
        RagState::Unavailable => {
            "RAG not configured / unavailable — retrieval lands in Phase 8.\nNo results are shown because none were retrieved.".to_string()
        }
        RagState::Available {
            router,
            tools,
            sources,
            candidates,
            merged,
            reranked,
            final_count,
            checksum,
            excluded,
        } => {
            let mut lines = Vec::new();
            lines.push(format!("  route: {}", router));
            lines.push(format!("  tools: {}", tools.join(", ")));
            lines.push(format!("  sources: {}", sources.join(", ")));
            lines.push(format!("  candidates: {}", candidates));
            lines.push(format!("  merged (RRF): {}", merged));
            lines.push(format!("  reranked: {}", reranked));
            lines.push(format!("  final: {}", final_count));
            lines.push(format!("  checksum: {}", checksum));
            if *excluded > 0 {
                lines.push(format!("  excluded (non-active): {}", excluded));
            }
            lines.join("\n")
        }
    };
    let body = Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("RAG debug"));
    frame.render_widget(body, area);
}
