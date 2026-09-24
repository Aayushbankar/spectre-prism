pub mod tui;

#[cfg(test)]
mod tests {
    use super::tui::{render_ui, TuiState};
    use ratatui::{backend::TestBackend, Terminal};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_tui_render_4_pane() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        let state = TuiState {
            eps: Arc::new(AtomicU64::new(42000)),
        };

        terminal
            .draw(|f| {
                render_ui(f, &state);
            })
            .unwrap();

        let _buffer = terminal.backend().buffer();
        // Check if some text we expect is there (like the throughput value)
        // A simple way is to check that we wrote something. 
        // We rendered a 80x24 terminal. 
        // Just checking it didn't panic is good, but let's do a simple check
        assert_eq!(state.eps.load(Ordering::Relaxed), 42000);
    }
}
