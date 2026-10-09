//! 05-05 cockpit screen snapshots: every screen renders a stable
//! `TestBackend(80, 24)` buffer; honesty states (skipped/warn/truncated/
//! unavailable) are asserted verbatim.

use ratatui::{Terminal, backend::TestBackend};
use tui::screens::{
    AnnotationRow, GraphNavView, IndexStatus, JobRow, RagState, SearchRow,
    render_annotation_review, render_doctor, render_graph_nav, render_index_status, render_jobs,
    render_rag_debug, render_reader, render_search,
};

fn buffer_text(terminal: &Terminal<TestBackend>) -> String {
    terminal.backend().buffer().content().iter().map(|c| c.symbol()).collect()
}

fn draw<W>(width: u16, height: u16, render: W) -> String
where
    W: FnOnce(&mut ratatui::Frame),
{
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(render).unwrap();
    buffer_text(&terminal)
}

// ─── Task 1: ops screens ───

#[test]
fn render_doctor_shows_families_and_honest_skipped() {
    use application::quran_doctor::{CheckLevel, QuranDoctorCheck};
    let checks = vec![
        QuranDoctorCheck {
            id: "quran.edition_active",
            status: CheckLevel::Pass,
            summary: "edition active".to_string(),
            remedy: None,
            next_command: None,
        },
        QuranDoctorCheck {
            id: "quran.token_index",
            status: CheckLevel::Skipped,
            summary: "no backend yet".to_string(),
            remedy: None,
            next_command: None,
        },
    ];
    let text = draw(80, 24, |f| render_doctor(f, &checks));
    assert!(text.contains("quran.edition_active"), "family row shown");
    assert!(text.contains("PASS"), "passing family shown as pass");
    assert!(text.contains("SKIP"), "skipped family shown as skipped, never pass");
}

#[test]
fn render_jobs_lists_state_and_error() {
    let jobs = vec![
        JobRow {
            id: "job-1".to_string(),
            kind: "quran.import".to_string(),
            state: "Succeeded".to_string(),
            last_error: None,
        },
        JobRow {
            id: "job-2".to_string(),
            kind: "quran.index".to_string(),
            state: "Failed".to_string(),
            last_error: Some("disk full".to_string()),
        },
    ];
    let text = draw(80, 24, |f| render_jobs(f, &jobs));
    assert!(text.contains("job-1") && text.contains("Succeeded"), "job + state shown");
    assert!(text.contains("disk full"), "last error shown");
}

#[test]
fn render_index_status_shows_drift_as_warn() {
    let status = IndexStatus {
        generation: 3,
        drift: Some("serving generation 2 behind active 3".to_string()),
    };
    let text = draw(80, 24, |f| render_index_status(f, &status));
    assert!(text.contains("3"), "generation shown");
    assert!(text.contains("WARN"), "drift shown as warn, never pass");
    assert!(text.contains("behind active 3"), "drift detail shown");
}

// ─── Task 2: reader + search ───

fn ayah_parts() -> (&'static str, &'static str, Vec<(String, String)>) {
    (
        "quran:test-edition-min@0.1.0:2:1",
        "نَبلهُم",
        vec![("evil".to_string(), "\x1b[31mred".to_string())],
    )
}

#[test]
fn render_reader_shows_canonical_and_sanitizes() {
    let (reference, arabic, translations) = ayah_parts();
    let text = draw(80, 24, |f| render_reader(f, reference, arabic, &translations));
    assert!(text.contains("نَبلهُم"), "canonical Arabic shown");
    assert!(text.contains("2:1"), "reference shown");
    assert!(!text.contains('\x1b'), "no escape reaches the buffer");
}

#[test]
fn render_search_preserves_order_and_sanitizes() {
    let hits = vec![
        SearchRow { reference: "quran:test@0.1.0:6:1".to_string(), snippet: "zeta".to_string() },
        SearchRow {
            reference: "quran:test@0.1.0:1:1".to_string(),
            snippet: "\x1b[31mred".to_string(),
        },
    ];
    let text = draw(80, 24, |f| render_search(f, &hits));
    let zeta = text.find("zeta").expect("first hit shown");
    let one = text.find("1:1").expect("second hit shown");
    assert!(zeta < one, "hits keep service order, no client re-sort");
    assert!(!text.contains('\x1b'), "crafted escape stripped before render");
}

// ─── Task 3: graph, annotations, RAG debug ───

#[test]
fn render_graph_nav_shows_truncation_verbatim() {
    let view = GraphNavView {
        lines: vec!["ayah:1:1 -NEXT-> ayah:1:2".to_string()],
        truncated: true,
        incomplete_reason: Some("max_edges budget cut the search short".to_string()),
    };
    let text = draw(80, 24, |f| render_graph_nav(f, &view));
    assert!(text.contains("truncated"), "truncation named");
    assert!(text.contains("max_edges budget cut the search short"), "reason verbatim");
    assert!(!text.contains("no path"), "truncation never rendered as absence");
}

#[test]
fn render_annotation_review_shows_pending() {
    let rows = vec![AnnotationRow {
        id: "parity-sug".to_string(),
        kind: "edge".to_string(),
        decision: "pending".to_string(),
        summary: "ayah:1:1 -NEXT-> ayah:1:2".to_string(),
    }];
    let text = draw(80, 24, |f| render_annotation_review(f, &rows));
    assert!(text.contains("parity-sug"), "proposal shown");
    assert!(text.contains("pending"), "status shown");
}

#[test]
fn render_rag_debug_shows_typed_unavailable() {
    let text = draw(80, 24, |f| render_rag_debug(f, &RagState::Unavailable));
    assert!(
        text.contains("unavailable") || text.contains("not configured"),
        "typed unavailable state shown"
    );
}
