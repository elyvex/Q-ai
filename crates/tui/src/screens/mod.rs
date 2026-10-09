//! Cockpit screens. Wave 1 ships the dashboard; later plans add
//! reader/search/graph/ops screens behind the same `Screen` selector.

pub mod annotation_review;
pub mod dashboard;
pub mod graph;
pub mod ops;
pub mod rag_debug;
pub mod reader;
pub mod search;

pub use annotation_review::{AnnotationRow, render_annotation_review};
pub use dashboard::render_dashboard;
pub use graph::{GraphNavView, render_graph_nav};
pub use ops::{IndexStatus, JobRow, render_doctor, render_index_status, render_jobs};
pub use rag_debug::{RagState, render_rag_debug};
pub use reader::render_reader;
pub use search::{SearchRow, render_search};

/// Cockpit destination selector. Mirrors the palette inventory one-to-one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Screen {
    /// Cockpit overview (default).
    #[default]
    Dashboard,
    Reader,
    Search,
    Graph,
    AnnotationReview,
    RagDebug,
    Doctor,
    Jobs,
    Index,
}

impl Screen {
    /// All screens in cockpit order.
    pub fn all() -> &'static [Screen] {
        &[
            Screen::Dashboard,
            Screen::Reader,
            Screen::Search,
            Screen::Graph,
            Screen::AnnotationReview,
            Screen::RagDebug,
            Screen::Doctor,
            Screen::Jobs,
            Screen::Index,
        ]
    }

    /// Display title.
    pub fn title(self) -> &'static str {
        match self {
            Screen::Dashboard => "Dashboard",
            Screen::Reader => "Reader",
            Screen::Search => "Search",
            Screen::Graph => "Graph",
            Screen::AnnotationReview => "Annotation review",
            Screen::RagDebug => "RAG debug",
            Screen::Doctor => "Doctor",
            Screen::Jobs => "Jobs",
            Screen::Index => "Index",
        }
    }

    /// Resolve a palette command name to its screen.
    pub fn from_command(name: &str) -> Option<Screen> {
        match name {
            "dashboard" => Some(Screen::Dashboard),
            "reader" => Some(Screen::Reader),
            "search" => Some(Screen::Search),
            "graph" => Some(Screen::Graph),
            "annotation-review" => Some(Screen::AnnotationReview),
            "rag-debug" => Some(Screen::RagDebug),
            "doctor" => Some(Screen::Doctor),
            "jobs" => Some(Screen::Jobs),
            "index" => Some(Screen::Index),
            _ => None,
        }
    }
}
