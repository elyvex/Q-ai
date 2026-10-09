//! Command palette inventory: every cockpit destination with
//! substring/prefix filtering (no fuzzy dependency).

/// One palette destination.
#[derive(Debug, Clone)]
pub struct Command {
    /// Stable command id (also the screen selector).
    pub name: String,
    /// One-line operator description.
    pub description: String,
}

impl Command {
    fn new(name: &str, description: &str) -> Self {
        Self { name: name.to_string(), description: description.to_string() }
    }
}

/// The cockpit command inventory (D-10): dashboard, reader, search, graph,
///
/// annotation review, RAG debug, doctor, jobs, and index status.
#[derive(Debug, Clone, Default)]
pub struct Palette {
    commands: Vec<Command>,
}

impl Palette {
    /// Full command inventory in cockpit order.
    pub fn new() -> Self {
        Self {
            commands: vec![
                Command::new("dashboard", "Cockpit overview and service status"),
                Command::new("reader", "Read canonical text with translations"),
                Command::new("search", "Search across normalization modes"),
                Command::new("graph", "Navigate the verse/word/root graph"),
                Command::new("annotation-review", "Review queued annotation proposals"),
                Command::new("rag-debug", "Inspect typed retrieval envelopes"),
                Command::new("doctor", "Corpus, index, and cross-store health"),
                Command::new("jobs", "Background job queue status"),
                Command::new("index", "Index generations and drift status"),
            ],
        }
    }

    /// All commands in cockpit order.
    pub fn commands(&self) -> &[Command] {
        &self.commands
    }

    /// Case-insensitive substring filter over name and description.
    /// An empty query returns the whole inventory.
    pub fn filter(&self, query: &str) -> Vec<&Command> {
        let q = query.to_lowercase();
        if q.is_empty() {
            return self.commands.iter().collect();
        }
        self.commands
            .iter()
            .filter(|c| {
                c.name.to_lowercase().contains(&q) || c.description.to_lowercase().contains(&q)
            })
            .collect()
    }
}
