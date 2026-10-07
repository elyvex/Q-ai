//! Phase 5 — TUI operations cockpit.
//!
//! A ratatui/crossterm terminal UI over the shared `application` services
//! layer (D-10/D-12/D-13). The TUI reads data through `Arc<dyn …>` service
//! handles built by the same composition path as `qai serve` — it never
//! shells out to the `qai` CLI and never speaks HTTP.

pub mod app;
pub mod palette;
pub mod screens;

pub use app::{App, TuiServices, run};
pub use palette::{Command, Palette};

/// Strip terminal control sequences from untrusted text before render.
///
/// Dataset strings rendered by ratatui can otherwise inject terminal control
/// sequences (T-05-05). Drops C0/C1 controls (keeping `\n` and `\t`), DEL,
/// bare ESC, and any CSI (`ESC [ … final`) / OSC (`ESC ] … BEL`) sequence it
/// opens. Everything else — including Arabic code points — passes through.
///
/// This is the terminal-side analogue of the server's `escape_html`.
#[must_use]
pub fn sanitize_terminal_text(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            match chars.peek() {
                // CSI: ESC [ params… final-byte.
                Some('[') => {
                    chars.next();
                    for end in chars.by_ref() {
                        if ('\x40'..='\x7e').contains(&end) {
                            break;
                        }
                    }
                }
                // OSC: ESC ] … terminated by BEL or ESC \.
                Some(']') => {
                    chars.next();
                    let mut prev_was_esc = false;
                    loop {
                        match chars.next() {
                            None | Some('\x07') => break,
                            Some('\x1b') => {
                                prev_was_esc = true;
                            }
                            Some('\\') if prev_was_esc => break,
                            Some(_) => {
                                prev_was_esc = false;
                            }
                        }
                    }
                }
                // Charset selection (ESC ( X, ESC ) X, ESC # X): one more char.
                Some('(') | Some(')') | Some('#') => {
                    chars.next();
                }
                // Bare ESC or unknown introducer: drop the ESC, keep the rest.
                _ => {}
            }
            continue;
        }
        if ch == '\n' || ch == '\t' {
            out.push(ch);
        } else if !ch.is_control() {
            out.push(ch);
        }
    }
    out
}
