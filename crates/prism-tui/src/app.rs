use color_eyre::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Sparkline, Gauge, Paragraph, Table, Row, Cell, List, ListItem, Chart, Dataset, GraphType, Axis},
    style::{Color, Style, Modifier},
    text::{Span, Line},
    symbols,
    Frame,
};
use serde::Deserialize;
use std::collections::VecDeque;
use std::time::Duration;
use tokio::sync::mpsc;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Action {
    Tick,
    Render,
    Key(KeyEvent),
    Resize(u16, u16),
    UpdateMetrics(MetricsData),
    UpdateDlq(Vec<(String, String, String)>),
    UpdateHitl(Vec<String>),
    UpdateLedger(String, f64, u64),
    Quit,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct MetricsData {
    pub eps: u64,
    pub processed: u64,
    pub drops: u64,
    pub dlq: u64,
}

pub struct App {
    pub should_quit: bool,
    pub metrics: MetricsData,
    pub eps_history: VecDeque<u64>,
    pub dlq_items: Vec<(String, String, String)>,
    pub hitl_rules: Vec<String>,
    pub ledger_tail: String,
    pub ledger_history: Vec<(f64, f64)>,
    pub tick_count: f64,
}

impl App {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            metrics: MetricsData::default(),
            eps_history: VecDeque::with_capacity(100),
            dlq_items: Vec::new(),
            hitl_rules: Vec::new(),
            ledger_tail: "Empty".to_string(),
            ledger_history: Vec::new(),
            tick_count: 0.0,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut terminal = ratatui::init();
        
        let (action_tx, mut action_rx) = mpsc::unbounded_channel();
        
        // Background Task: Crossterm Events
        let tx = action_tx.clone();
        tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            use futures::StreamExt;
            while let Some(Ok(evt)) = reader.next().await {
                match evt {
                    CrosstermEvent::Key(key) => {
                        let _ = tx.send(Action::Key(key));
                    }
                    CrosstermEvent::Resize(x, y) => {
                        let _ = tx.send(Action::Resize(x, y));
                    }
                    _ => {}
                }
            }
        });

        // Background Task: Polling Metrics & Files
        let tx = action_tx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(500));
            let mut tick_count = 0.0;
            loop {
                interval.tick().await;
                let _ = tx.send(Action::Tick);
                let _ = tx.send(Action::Render);
                
                // Read metrics JSON
                if let Ok(content) = std::fs::read_to_string("/tmp/prism_metrics.json") {
                    if let Ok(metrics) = serde_json::from_str::<MetricsData>(&content) {
                        let _ = tx.send(Action::UpdateMetrics(metrics));
                    }
                }
                
                // Read rules dir (Gatekeeper HitL)
                let rules_dir = PathBuf::from("/tmp/prism/rules");
                if let Ok(dir) = std::fs::read_dir(&rules_dir) {
                    let rules: Vec<String> = dir
                        .filter_map(Result::ok)
                        .map(|entry| entry.file_name().to_string_lossy().to_string())
                        .collect();
                    let _ = tx.send(Action::UpdateHitl(rules));
                }
                
                // Read DLQ
                let vault_dir = PathBuf::from("/tmp/prism/vault");
                let mut dlq_res = Vec::new();
                if let Ok(dir) = std::fs::read_dir(&vault_dir) {
                    let mut latest_dlq = None;
                    let mut latest_time = std::time::SystemTime::UNIX_EPOCH;
                    for entry in dir.filter_map(Result::ok) {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if name.starts_with("dlq_") && name.ends_with(".log") {
                            if let Ok(meta) = entry.metadata() {
                                if let Ok(modified) = meta.modified() {
                                    if modified > latest_time {
                                        latest_time = modified;
                                        latest_dlq = Some(entry.path());
                                    }
                                }
                            }
                        }
                    }
                    if let Some(path) = latest_dlq {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            for line in content.lines().rev().take(20) {
                                let ts = line.split("\"timestamp\":\"").nth(1).unwrap_or("Just now").split('"').next().unwrap_or("Just now");
                                let err = line.split("\"error\":\"").nth(1).unwrap_or("Unknown").split('"').next().unwrap_or("Unknown");
                                let payload = line.split("\"payload\":\"").nth(1).unwrap_or(line).split('"').next().unwrap_or(line);
                                dlq_res.push((ts.to_string(), err.to_string(), payload.to_string()));
                            }
                        }
                    }
                }
                let _ = tx.send(Action::UpdateDlq(dlq_res));
                
                // Read Vault Ledger
                let ledger_path = PathBuf::from("/tmp/prism/vault/ledger.log");
                tick_count += 1.0;
                let mut count = 0;
                let mut tail = "Empty".to_string();
                if let Ok(content) = std::fs::read_to_string(&ledger_path) {
                    let lines: Vec<&str> = content.lines().collect();
                    count = lines.len() as u64;
                    if let Some(last) = lines.last() {
                        tail = last.to_string();
                    }
                }
                let _ = tx.send(Action::UpdateLedger(tail, tick_count, count));
            }
        });

        // Main Loop
        while !self.should_quit {
            if let Some(action) = action_rx.recv().await {
                self.update(action.clone())?;
                if let Action::Render = action {
                    terminal.draw(|f| self.draw(f))?;
                }
            }
        }

        ratatui::restore();
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<()> {
        match action {
            Action::Quit => {
                self.should_quit = true;
            }
            Action::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            self.should_quit = true;
                        }
                        KeyCode::Char('c') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                            self.should_quit = true;
                        }
                        _ => {}
                    }
                }
            }
            Action::UpdateMetrics(m) => {
                self.metrics = m;
                self.eps_history.push_back(self.metrics.eps);
                if self.eps_history.len() > 100 {
                    self.eps_history.pop_front();
                }
            }
            Action::UpdateHitl(rules) => {
                self.hitl_rules = rules;
            }
            Action::UpdateDlq(items) => {
                self.dlq_items = items;
            }
            Action::UpdateLedger(tail, tick, count) => {
                self.ledger_tail = tail;
                self.tick_count = tick;
                self.ledger_history.push((self.tick_count, count as f64));
                if self.ledger_history.len() > 100 {
                    self.ledger_history.remove(0);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn draw(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(f.area());

        let top_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[0]);

        let bottom_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);

        // Pane 1: EPS Gauge + Sparkline
        let gauge_chunk = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(top_chunks[0]);

        let gauge = Gauge::default()
            .block(Block::default().title("EPS Throughput (Gauge)").borders(Borders::ALL))
            .gauge_style(Style::default().fg(Color::Green).bg(Color::Black))
            .percent((self.metrics.eps.min(100000) as u16 / 1000).min(100))
            .label(format!("{} EPS | {} Processed", self.metrics.eps, self.metrics.processed));
        f.render_widget(gauge, gauge_chunk[0]);

        let history_vec: Vec<u64> = self.eps_history.iter().copied().collect();
        let sparkline = Sparkline::default()
            .block(Block::default().title("EPS History").borders(Borders::ALL))
            .data(&history_vec)
            .style(Style::default().fg(Color::Cyan));
        f.render_widget(sparkline, gauge_chunk[1]);

        // Pane 2: DLQ Table
        let header_cells = ["Timestamp", "Error", "Payload"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
        let header = Row::new(header_cells).style(Style::default().bg(Color::DarkGray)).height(1).bottom_margin(1);
        
        let rows: Vec<Row> = self.dlq_items.iter().map(|(ts, err, payload)| {
            Row::new(vec![Cell::from(ts.as_str()), Cell::from(err.as_str()), Cell::from(payload.as_str())])
        }).collect();

        let table = Table::new(rows, [Constraint::Length(15), Constraint::Length(15), Constraint::Min(20)])
            .header(header)
            .block(Block::default().title("Dead Letter Queue (DLQ)").borders(Borders::ALL));
        f.render_widget(table, top_chunks[1]);

        // Pane 3: Merkle Ticker Chart
        let chart_chunk = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(bottom_chunks[0]);

        let tail_widget = Paragraph::new(format!("Latest Root: {}", self.ledger_tail))
            .block(Block::default().title("Integrity Merkle Tail").borders(Borders::ALL));
        f.render_widget(tail_widget, chart_chunk[0]);

        let datasets = vec![
            Dataset::default()
                .name("Ledger Count")
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Yellow))
                .graph_type(GraphType::Line)
                .data(&self.ledger_history),
        ];
        let chart = Chart::new(datasets)
            .block(Block::default().title("Root History").borders(Borders::ALL))
            .x_axis(Axis::default().title("Time").bounds([self.tick_count.max(100.0) - 100.0, self.tick_count.max(100.0)]))
            .y_axis(Axis::default().title("Entries").bounds([0.0, self.ledger_history.last().map(|(_, y)| *y).unwrap_or(100.0).max(10.0)]));
        f.render_widget(chart, chart_chunk[1]);

        // Pane 4: HitL List
        let items: Vec<ListItem> = self.hitl_rules.iter().map(|rule| {
            ListItem::new(Line::from(vec![
                Span::styled("[Pending] ", Style::default().fg(Color::Yellow)),
                Span::raw(rule),
            ]))
        }).collect();
        
        let list = List::new(items)
            .block(Block::default().title("HitL Gatekeeper (Approve Key: 'A')").borders(Borders::ALL))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(Color::DarkGray))
            .highlight_symbol(">> ");
        
        f.render_widget(list, bottom_chunks[1]);
    }
}
