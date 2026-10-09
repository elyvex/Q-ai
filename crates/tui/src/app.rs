//! TUI application shell: shared-services handles, event loop with
//! panic-safe terminal restore, and the palette-driven screen selector.

use std::io;
use std::sync::Arc;
use std::time::Duration;

use application::quran_tools::dispatch_registered_tool;
use application::{
    quran_counting::FrequencyReport,
    quran_morphology::FamilyMemberView,
    quran_search::{SearchError, SearchOutput},
};
use application::{quran_graph_api::*, quran_lexicon_api::*, quran_search_api::*};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{Event, KeyCode, KeyModifiers, poll, read},
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    Palette,
    screens::{
        AnnotationRow, GraphNavView, IndexStatus, JobRow, RagState, Screen, SearchRow,
        render_annotation_review, render_dashboard, render_doctor, render_graph_nav,
        render_index_status, render_jobs, render_rag_debug, render_reader, render_search,
    },
};

struct UnavailableSearch;

#[async_trait::async_trait]
impl application::quran_search_api::SearchBackend for UnavailableSearch {
    async fn search_exact(&self, _args: ExactArgs) -> Result<SearchOutput, SearchError> {
        unimplemented!()
    }
    async fn search_normalized(&self, _args: NormalizedArgs) -> Result<SearchOutput, SearchError> {
        unimplemented!()
    }
    async fn search_phrase(&self, _args: PhraseArgs) -> Result<SearchOutput, SearchError> {
        unimplemented!()
    }
    async fn search_concatenated(
        &self,
        _args: ConcatenatedArgs,
    ) -> Result<SearchOutput, SearchError> {
        unimplemented!()
    }
    async fn search_regex(&self, _args: RegexArgs) -> Result<SearchOutput, SearchError> {
        unimplemented!()
    }
}

struct UnavailableLexicon;

#[async_trait::async_trait]
impl application::quran_lexicon_api::LexiconBackend for UnavailableLexicon {
    async fn word_family(
        &self,
        _args: FamilyArgs,
    ) -> Result<Vec<FamilyMemberView>, LexiconApiError> {
        unimplemented!()
    }
    async fn root_frequency(
        &self,
        _args: RootFrequencyArgs,
    ) -> Result<FrequencyReport, LexiconApiError> {
        unimplemented!()
    }
    async fn lemma_frequency(
        &self,
        _args: LemmaFrequencyArgs,
    ) -> Result<FrequencyReport, LexiconApiError> {
        unimplemented!()
    }
    async fn count_frequency(
        &self,
        _target: &str,
        _profile: &str,
    ) -> Result<FrequencyReport, LexiconApiError> {
        unimplemented!()
    }
    async fn count_distribution(
        &self,
        _target: &str,
        _profile: &str,
    ) -> Result<application::quran_counting::DistributionReport, LexiconApiError> {
        unimplemented!()
    }
    async fn count_occurrences(
        &self,
        _target: &str,
        _profile: &str,
    ) -> Result<application::quran_counting::OccurrenceSpan, LexiconApiError> {
        unimplemented!()
    }
    async fn count_hapax(
        &self,
        _profile: &str,
        _limit: usize,
    ) -> Result<application::quran_counting::HapaxReport, LexiconApiError> {
        unimplemented!()
    }
    async fn count_cooccurrence(
        &self,
        _target: &str,
        _profile: &str,
        _window: usize,
        _limit: usize,
    ) -> Result<
        (
            application::quran_counting::CountingRules,
            Vec<application::quran_counting::CooccurrenceHit>,
        ),
        LexiconApiError,
    > {
        unimplemented!()
    }
    async fn count_collocation(
        &self,
        _target: &str,
        _profile: &str,
        _window: usize,
        _limit: usize,
    ) -> Result<
        (
            application::quran_counting::CountingRules,
            Vec<application::quran_counting::CollocationHit>,
        ),
        LexiconApiError,
    > {
        unimplemented!()
    }
    async fn count_numeric_report(
        &self,
        _target: &str,
        _profile: &str,
    ) -> Result<application::quran_counting::NumericReport, LexiconApiError> {
        unimplemented!()
    }
    async fn count_missing_form(
        &self,
        _target: &str,
        _profile: &str,
    ) -> Result<application::quran_counting::MissingFormReport, LexiconApiError> {
        unimplemented!()
    }
    async fn count_near_duplicates(
        &self,
        _threshold: f64,
        _limit: usize,
    ) -> Result<
        (
            application::quran_counting::CountingRules,
            Vec<application::quran_counting::NearDuplicateHit>,
        ),
        LexiconApiError,
    > {
        unimplemented!()
    }
    async fn count_interval(
        &self,
        _target: &str,
        _profile: &str,
    ) -> Result<application::quran_counting::OccurrenceSpan, LexiconApiError> {
        unimplemented!()
    }
    async fn count_unusual_usage(
        &self,
        _target: &str,
        _profile: &str,
    ) -> Result<application::quran_counting::NumericReport, LexiconApiError> {
        unimplemented!()
    }
}

struct UnavailableGraph;

#[async_trait::async_trait]
impl application::quran_graph_api::GraphBackend for UnavailableGraph {
    async fn neighbors(&self, _args: NeighborsArgs) -> Result<NeighborsOutput, GraphApiError> {
        unimplemented!()
    }
    async fn reachability(&self, _args: PathArgs) -> Result<ReachabilityOutput, GraphApiError> {
        unimplemented!()
    }
    async fn shortest_path(&self, _args: PathArgs) -> Result<ShortestOutput, GraphApiError> {
        unimplemented!()
    }
    async fn paths(&self, _args: PathsArgs) -> Result<PathsOutput, GraphApiError> {
        unimplemented!()
    }
    async fn subgraph(&self, _args: SubgraphArgs) -> Result<SubgraphOutput, GraphApiError> {
        unimplemented!()
    }
    async fn pattern(&self, _args: PatternArgs) -> Result<PatternOutput, GraphApiError> {
        unimplemented!()
    }
    async fn root_family(&self, _args: RootFamilyArgs) -> Result<RootFamilyOutput, GraphApiError> {
        unimplemented!()
    }
    async fn snapshot_meta(&self) -> Result<GraphSnapshotMeta, GraphApiError> {
        unimplemented!()
    }
}
/// Shared service handles (D-12): the TUI reads through these, never
/// through HTTP and never by shelling out to `qai`.
#[derive(Clone)]
pub struct TuiServices {
    /// Canonical reader over the active database file.
    pub reader: Arc<application::quran_reader::QuranReaderService>,
    /// Attributed search backend.
    pub search: Arc<dyn SearchBackend>,
    /// Lexicon + counting backend.
    pub lexicon: Arc<dyn LexiconBackend>,
    /// Graph backend over the active structural projection.
    pub graph: Arc<dyn GraphBackend>,
    /// Database file path (jobs listing, annotation review).
    pub db_path: String,
}

impl TuiServices {
    /// Construction-only doubles for render tests: a real reader over an
    /// empty scratch file (schema version falls back to 0; no query ever
    /// runs) and `unimplemented!()` backends behind the same trait seams
    /// the shell uses. Never used on a production path.
    pub fn for_tests() -> Self {
        static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "qai-tui-render-test-{}-{}.db",
            std::process::id(),
            SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        if !path.exists() {
            std::fs::File::create(&path).expect("scratch reader database");
        }
        let path_str = path.to_str().expect("utf-8 temp path").to_string();
        let reader = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("test runtime")
            .block_on(application::quran_cli::open_reader(&path_str))
            .expect("scratch reader opens");
        Self {
            reader: Arc::new(reader),
            search: Arc::new(UnavailableSearch),
            lexicon: Arc::new(UnavailableLexicon),
            graph: Arc::new(UnavailableGraph),
            db_path: path_str,
        }
    }
}

/// Canonical reader view for the reader screen.
///
/// Mapping rule (the canonical-slot invariant): `arabic` originates ONLY
/// from `AyahView.canonical.arabic_text()` and `reference` ONLY from
/// `AyahView.canonical.reference()` — see `refresh_reader`. Translations
/// travel as `(label, text)` pairs in the translation region only.
#[derive(Debug, Clone, Default)]
pub struct ReaderView {
    /// Fully-qualified canonical reference.
    pub reference: String,
    /// Canonical Arabic text.
    pub arabic: String,
    /// Attributed translations.
    pub translations: Vec<(String, String)>,
}

/// Cockpit application state.
pub struct App {
    services: TuiServices,
    screen: Screen,
    palette: Palette,
    palette_query: String,
    show_palette: bool,
    should_quit: bool,
    tick: u64,
    doctor_checks: Vec<application::quran_doctor::QuranDoctorCheck>,
    jobs: Vec<JobRow>,
    index: IndexStatus,
    reader_view: Option<ReaderView>,
    selected_reference: String,
    search_hits: Vec<SearchRow>,
    graph_view: Option<GraphNavView>,
    annotations: Vec<AnnotationRow>,
}

impl App {
    /// Build the cockpit over already-opened shared services.
    pub fn new(services: TuiServices) -> Self {
        Self {
            services,
            screen: Screen::Dashboard,
            palette: Palette::new(),
            palette_query: String::new(),
            show_palette: false,
            should_quit: false,
            tick: 0,
            doctor_checks: Vec::new(),
            jobs: Vec::new(),
            index: IndexStatus { generation: 0, drift: None },
            reader_view: None,
            selected_reference: "1:1".to_string(),
            search_hits: Vec::new(),
            graph_view: None,
            annotations: Vec::new(),
        }
    }

    /// Shared services (screens read through these).
    pub fn services(&self) -> &TuiServices {
        &self.services
    }

    /// Currently selected screen.
    pub fn screen(&self) -> Screen {
        self.screen
    }

    /// Whether the event loop should exit.
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Handle one key press. `q`/`Esc` quits; `:` toggles the palette;
    /// palette mode edits the query and `Enter` jumps to the first match.
    pub fn on_key(&mut self, code: KeyCode) {
        if self.show_palette {
            match code {
                KeyCode::Esc => {
                    self.show_palette = false;
                    self.palette_query.clear();
                }
                KeyCode::Enter => {
                    if let Some(cmd) = self.palette.filter(&self.palette_query).into_iter().next()
                        && let Some(screen) = Screen::from_command(&cmd.name)
                    {
                        self.screen = screen;
                    }
                    self.show_palette = false;
                    self.palette_query.clear();
                }
                KeyCode::Backspace => {
                    self.palette_query.pop();
                }
                KeyCode::Char(c) => {
                    self.palette_query.push(c);
                }
                _ => {}
            }
            return;
        }
        match code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char(':') => self.show_palette = true,
            _ => {}
        }
    }

    /// Run the cockpit on an already-initialized terminal until quit.
    /// The caller owns `init`/`restore`; the panic hook below guarantees
    /// the terminal is restored even on panic (T-05-06). Snapshots refresh
    /// every 50 ticks (5s); a failed refresh keeps the previous snapshot.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        install_panic_hook();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        std::thread::spawn(move || event_pump(tx));
        let mut tick = tokio::time::interval(Duration::from_millis(100));
        self.refresh().await;
        while !self.should_quit {
            tokio::select! {
                _ = tick.tick() => {
                    self.tick += 1;
                    if self.tick.is_multiple_of(50) {
                        self.refresh().await;
                    }
                    terminal.draw(|f| self.render(f))?;
                }
                msg = rx.recv() => {
                    match msg {
                        Some(Event::Key(key)) => {
                            if key.modifiers.contains(KeyModifiers::CONTROL)
                                && key.code == KeyCode::Char('c')
                            {
                                self.should_quit = true;
                            } else {
                                self.on_key(key.code);
                            }
                        }
                        Some(_) => {}
                        None => break,
                    }
                }
            }
        }
        Ok(())
    }

    /// Refresh every snapshot through the shared services (D-12): doctor
    /// checks, jobs, index status, the selected ayah, and the graph
    /// neighborhood. Failures keep the previous snapshot — the screens
    /// render honest empty states, never errors-as-data.
    async fn refresh(&mut self) {
        let db = self.services.reader.database();
        if let Ok(checks) = application::quran_doctor::run_quran_checks(db, false).await {
            self.doctor_checks = checks;
        }
        if let Ok(jobs) = application::db::list_jobs(&self.services.db_path).await {
            self.jobs = parse_job_rows(&jobs);
        }
        if let Ok(snapshot) = application::quran_doctor_indexes::load_snapshot(db).await {
            let drift = snapshot.pointer.as_ref().and_then(|pointer| {
                (pointer.generation != snapshot.corpus_generation).then(|| {
                    format!(
                        "serving generation {} behind active {}",
                        pointer.generation, snapshot.corpus_generation
                    )
                })
            });
            self.index = IndexStatus { generation: snapshot.corpus_generation, drift };
        }
        self.refresh_reader().await;
        self.refresh_graph().await;
        self.refresh_annotations().await;
    }

    /// Refresh the selected ayah through the typed tool (canonical by
    /// construction — the mapping below is the only producer of the
    /// reader screen's canonical slot).
    async fn refresh_reader(&mut self) {
        let params = serde_json::json!({"reference": self.selected_reference});
        let Ok(value) = dispatch_registered_tool(
            &reader_registry(&self.services),
            &graph_registry(&self.services),
            "quran.get_ayah",
            params,
        )
        .await
        else {
            return;
        };
        let Some(view) = value.get("results").and_then(|r| r.get(0)) else {
            return;
        };
        let canonical = &view["canonical"];
        let Some(reference) = canonical.get("reference").and_then(|r| r.as_str()) else {
            return;
        };
        let Some(arabic) = canonical.get("arabic_text").and_then(|t| t.as_str()) else {
            return;
        };
        let translations = view
            .get("translations")
            .and_then(|t| t.as_array())
            .map(|items| {
                items
                    .iter()
                    .map(|item| {
                        (
                            item.get("translator")
                                .and_then(|s| s.as_str())
                                .unwrap_or("?")
                                .to_string(),
                            item.get("text").and_then(|s| s.as_str()).unwrap_or("").to_string(),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        self.reader_view = Some(ReaderView {
            reference: reference.to_string(),
            arabic: arabic.to_string(),
            translations,
        });
    }

    /// Refresh the graph neighborhood of the selected ayah's verse node.
    async fn refresh_graph(&mut self) {
        let node = ayah_node(&self.selected_reference);
        let params = serde_json::json!({"node": node});
        let Ok(value) = dispatch_registered_tool(
            &reader_registry(&self.services),
            &graph_registry(&self.services),
            "quran.graph_neighbors",
            params,
        )
        .await
        else {
            return;
        };
        let results = &value["results"];
        let mut lines = Vec::new();
        if let Some(edges) = results.get("edges").and_then(|e| e.as_array()) {
            for edge in edges.iter().take(20) {
                lines.push(format!(
                    "{} -{}-> {}",
                    edge.get("src").and_then(|s| s.as_str()).unwrap_or("?"),
                    edge.get("edge").and_then(|s| s.as_str()).unwrap_or("?"),
                    edge.get("dst").and_then(|s| s.as_str()).unwrap_or("?"),
                ));
            }
        }
        self.graph_view = Some(GraphNavView {
            lines,
            truncated: results.get("truncated").and_then(|t| t.as_bool()).unwrap_or(false),
            incomplete_reason: results
                .get("incomplete_reason")
                .and_then(|r| r.as_str())
                .map(str::to_string),
        });
    }

    /// Refresh the pending-review queue. The structural projection family
    /// is frozen (`quran-structural-v1`, same value the graph build
    /// publishes); the edition id comes from the live snapshot meta.
    async fn refresh_annotations(&mut self) {
        let Ok(meta) = self.services.graph.snapshot_meta().await else {
            return;
        };
        let Ok(queue) = application::quran_graph_annotations::review_queue(
            &self.services.db_path,
            "quran-structural-v1",
            &meta.edition_id,
        )
        .await
        else {
            return;
        };
        self.annotations = queue
            .iter()
            .map(|assertion| AnnotationRow {
                id: assertion.id.clone(),
                kind: format!("{:?}", assertion.kind).to_lowercase(),
                decision: format!("{:?}", assertion.decision).to_lowercase(),
                summary: assertion
                    .claim
                    .get("summary")
                    .and_then(|s| s.as_str())
                    .unwrap_or("?")
                    .to_string(),
            })
            .collect();
    }

    fn render(&self, frame: &mut Frame) {
        match self.screen {
            Screen::Dashboard => render_dashboard(frame, self),
            Screen::Doctor => render_doctor(frame, &self.doctor_checks),
            Screen::Jobs => render_jobs(frame, &self.jobs),
            Screen::Index => render_index_status(frame, &self.index),
            Screen::Reader => match &self.reader_view {
                Some(view) => {
                    render_reader(frame, &view.reference, &view.arabic, &view.translations)
                }
                None => render_hint(frame, "Reader", "loading the selected ayah…"),
            },
            Screen::Search => {
                if self.search_hits.is_empty() {
                    render_hint(frame, "Search", "no query yet — query input lands next");
                } else {
                    render_search(frame, &self.search_hits);
                }
            }
            Screen::Graph => match &self.graph_view {
                Some(view) => render_graph_nav(frame, view),
                None => render_hint(frame, "Graph", "loading the neighborhood…"),
            },
            Screen::AnnotationReview => render_annotation_review(frame, &self.annotations),
            Screen::RagDebug => render_rag_debug(frame, &RagState::Unavailable),
        }
    }
}

/// Reader tool registry over the shared handles (same constructors as the
/// `qai serve` arm, D-12).
fn reader_registry(services: &TuiServices) -> tool_registry::ToolRegistry {
    application::quran_tools::ReaderToolBackend::registry_with_index_root(
        services.reader.clone(),
        application::quran_index::index_root_for_db(&services.db_path),
    )
}

/// Graph tool registry over the shared handle.
fn graph_registry(services: &TuiServices) -> tool_registry::ToolRegistry {
    application::quran_graph_tools::GraphToolBackend::registry(Arc::new(
        application::quran_graph_api::FileGraphBackend::structural(&services.db_path),
    ))
}

/// Map the `list_jobs` JSON array to rows; unknown shapes yield no rows
/// (the screen renders the honest empty state).
fn parse_job_rows(value: &serde_json::Value) -> Vec<JobRow> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    Some(JobRow {
                        id: item.get("id")?.as_str()?.to_string(),
                        kind: item.get("kind")?.as_str()?.to_string(),
                        state: item.get("state")?.as_str()?.to_string(),
                        last_error: None,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// `"1:1"` → the ayah stable node id the graph serves.
fn ayah_node(reference: &str) -> String {
    format!("ayah:{reference}")
}

/// Render a titled honest-empty hint for screens awaiting data.
fn render_hint(frame: &mut Frame, title: &str, hint: &str) {
    let body = Paragraph::new(hint).block(Block::default().borders(Borders::ALL).title(title));
    frame.render_widget(body, frame.area());
}

/// Launch the cockpit: init the terminal, run the app, restore the
/// terminal. Used by the `qai tui` dispatch arm.
pub async fn run(services: TuiServices) -> io::Result<()> {
    let terminal = ratatui::init();
    let result = App::new(services).run(terminal).await;
    ratatui::restore();
    result
}

/// Blocking crossterm poll/read loop on a dedicated thread; forwards
/// events over `tx` and exits when the receiver is dropped.
fn event_pump(tx: tokio::sync::mpsc::UnboundedSender<Event>) {
    loop {
        match poll(Duration::from_millis(100)) {
            Ok(true) => match read() {
                Ok(event) => {
                    if tx.send(event).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            },
            Ok(false) => {
                if tx.is_closed() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}

/// Restore the terminal before running the previous panic hook, so a
/// mid-render panic never leaves the terminal in raw mode (T-05-06).
fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        ratatui::restore();
        previous(info);
    }));
}
