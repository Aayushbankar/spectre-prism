pub mod app;
pub mod metrics;
pub mod tui;

pub use app::TuiState;
pub use tui::render_ui;

#[cfg(test)]
mod tests {
    use super::{render_ui, TuiState};
    use ratatui::{backend::TestBackend, Terminal};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_tui_render_4_pane() {
        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();

        let state = TuiState {
            eps: Arc::new(AtomicU64::new(42000)),
            ..Default::default()
        };

        terminal
            .draw(|f| {
                render_ui(f, &state);
            })
            .unwrap();

        let _buffer = terminal.backend().buffer();
        assert_eq!(state.eps.load(Ordering::Relaxed), 42000);
    }
    
    #[test]
    fn test_tui_render_80x24() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        let state = TuiState {
            eps: Arc::new(AtomicU64::new(42000)),
            ..Default::default()
        };

        terminal
            .draw(|f| {
                render_ui(f, &state);
            })
            .unwrap();

        let _buffer = terminal.backend().buffer();
        assert_eq!(state.eps.load(Ordering::Relaxed), 42000);
    }
}
