//! TUI application shell: shared-services handles, event loop with
//! panic-safe terminal restore, and the palette-driven screen selector.

use std::io;
use std::sync::Arc;
use std::time::Duration;

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
    screens::{Screen, render_dashboard},
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
        let reader = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("test runtime")
            .block_on(application::quran_cli::open_reader(path.to_str().expect("utf-8 temp path")))
            .expect("scratch reader opens");
        Self {
            reader: Arc::new(reader),
            search: Arc::new(UnavailableSearch),
            lexicon: Arc::new(UnavailableLexicon),
            graph: Arc::new(UnavailableGraph),
        }
    }
}

/// Cockpit application state.
pub struct App {
    services: TuiServices,
    screen: Screen,
    palette: Palette,
    palette_query: String,
    show_palette: bool,
    should_quit: bool,
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
    /// the terminal is restored even on panic (T-05-06).
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        install_panic_hook();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        std::thread::spawn(move || event_pump(tx));
        let mut tick = tokio::time::interval(Duration::from_millis(100));
        while !self.should_quit {
            tokio::select! {
                _ = tick.tick() => {
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

    fn render(&self, frame: &mut Frame) {
        match self.screen {
            Screen::Dashboard => render_dashboard(frame, self),
            screen => {
                let body = Paragraph::new(format!(
                    "{} — ships in a later wave; the dashboard already works.",
                    screen.title()
                ))
                .block(Block::default().borders(Borders::ALL).title(screen.title()));
                frame.render_widget(body, frame.area());
            }
        }
        // Palette overlay rendering ships with the cockpit screens (05-05);
        // the query state machine above is already live and tested.
    }
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
