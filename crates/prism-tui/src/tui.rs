use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::fs;

pub struct TuiState {
    pub eps: Arc<AtomicU64>,
}

impl Default for TuiState {
    fn default() -> Self {
        Self::new()
    }
}

impl TuiState {
    pub fn new() -> Self {
        Self {
            eps: Arc::new(AtomicU64::new(0)),
        }
    }
}

pub fn render_ui(f: &mut Frame, state: &TuiState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ]
            .as_ref(),
        )
        .split(f.area()); // ratatui 0.26+ uses f.area()

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[0]);

    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[1]);

    // Pane 1: EPS
    let eps_val = state.eps.load(Ordering::Relaxed);
    let eps_widget = Paragraph::new(format!("Live EPS: {}", eps_val))
        .block(Block::default().title("Throughput").borders(Borders::ALL));
    f.render_widget(eps_widget, top_chunks[0]);

    // Pane 2: DLQ Rate
    let dlq_lines = fs::read_to_string("/var/run/prism/dlq.jsonl")
        .map(|s| s.lines().count())
        .unwrap_or(0);
    let dlq_widget = Paragraph::new(format!("DLQ Logs: {}", dlq_lines))
        .block(Block::default().title("Dead Letter Queue").borders(Borders::ALL));
    f.render_widget(dlq_widget, top_chunks[1]);

    // Pane 3: Merkle Ticker
    let ledger_tail = fs::read_to_string("/tmp/test_vault_success/ledger.log")
        .or_else(|_| fs::read_to_string("crates/prism-provenance/test_vault_success/ledger.log"))
        .or_else(|_| fs::read_to_string("output_dir/ledger.log"))
        .map(|s| s.lines().last().unwrap_or("Empty").to_string())
        .unwrap_or_else(|_| "No Ledger".to_string());
    let merkle_widget = Paragraph::new(format!("Latest Root: {}", ledger_tail))
        .block(Block::default().title("Integrity Merkle").borders(Borders::ALL));
    f.render_widget(merkle_widget, bottom_chunks[0]);

    // Pane 4: HitL
    let rules_count = fs::read_dir("/etc/prism/rules")
        .map(|dir| dir.count())
        .unwrap_or(0);
    let hitl_widget = Paragraph::new(format!("Active AI Rules: {}", rules_count))
        .block(Block::default().title("Gatekeeper HitL").borders(Borders::ALL));
    f.render_widget(hitl_widget, bottom_chunks[1]);
}
