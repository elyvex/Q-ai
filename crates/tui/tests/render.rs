use ratatui::{Terminal, backend::TestBackend};
use tui::{App, Palette, TuiServices, render_dashboard, sanitize_terminal_text};

fn services() -> TuiServices {
    // Task 2 tests render paths only; service handles are never invoked.
    // Concrete doubles live behind the same trait seams the shell uses.
    TuiServices::for_tests()
}

#[test]
fn dashboard_renders_title_and_palette_hint() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let app = App::new(services());
    terminal.draw(|f| render_dashboard(f, &app)).unwrap();
    let buf = terminal.backend().buffer().clone();
    let text: String = buf.content().iter().map(|c| c.symbol()).collect();
    assert!(text.contains("Q-ai"), "dashboard shows product title");
    assert!(text.contains("qai tui"), "dashboard shows the palette hint");
}

#[test]
fn palette_filter_matches_doctor_by_prefix_and_substring() {
    let palette = Palette::new();
    let names: Vec<&str> = palette.filter("doc").iter().map(|c| c.name.as_str()).collect();
    assert!(
        names.iter().any(|n| n.contains("doctor")),
        "filter('doc') returns the doctor command, got {names:?}"
    );
    assert!(
        !names.iter().any(|n| n.contains("graph")),
        "filter('doc') excludes non-matching commands, got {names:?}"
    );
}

#[test]
fn sanitizer_strips_control_sequences_and_keeps_arabic() {
    let dirty = "\x1b[31m\x07hello\u{0}\u{9f}بِسْمِ";
    let clean = sanitize_terminal_text(dirty);
    assert_eq!(clean, "helloبِسْمِ");
}

#[test]
fn app_quits_on_q() {
    let mut app = App::new(services());
    app.on_key(ratatui::crossterm::event::KeyCode::Char('q'));
    assert!(app.should_quit());
}
