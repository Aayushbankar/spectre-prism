use color_eyre::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    widgets::{Block, Borders, BorderType, Sparkline, Gauge, Paragraph, Table, Row, Cell, List, ListItem, ListState, TableState, Chart, Dataset, GraphType, Axis, Tabs},
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
    ApproveRule(String),
    Quit,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct MetricsData {
    pub eps: u64,
    pub processed: u64,
    pub drops: u64,
    pub dlq: u64,
}

#[derive(PartialEq)]
pub enum ActiveTab {
    Dashboard,
    Gatekeeper,
    DlqViewer,
}

pub struct App {
    pub should_quit: bool,
    pub active_tab: ActiveTab,
    
    // Data state
    pub metrics: MetricsData,
    pub eps_history: VecDeque<u64>,
    pub dlq_items: Vec<(String, String, String)>,
    pub hitl_rules: Vec<String>,
    pub ledger_tail: String,
    pub ledger_history: Vec<(f64, f64)>,
    pub tick_count: f64,
    
    // UI state
    pub hitl_state: ListState,
    pub dlq_state: TableState,
}

impl App {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            active_tab: ActiveTab::Dashboard,
            metrics: MetricsData::default(),
            eps_history: VecDeque::with_capacity(200),
            dlq_items: Vec::new(),
            hitl_rules: Vec::new(),
            ledger_tail: "Empty".to_string(),
            ledger_history: Vec::new(),
            tick_count: 0.0,
            hitl_state: ListState::default(),
            dlq_state: TableState::default(),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut terminal = ratatui::init();
        let (action_tx, mut action_rx) = mpsc::unbounded_channel();
        
        let tx_keys = action_tx.clone();
        tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            use futures::StreamExt;
            while let Some(Ok(evt)) = reader.next().await {
                if let CrosstermEvent::Key(key) = evt {
                    let _ = tx_keys.send(Action::Key(key));
                }
            }
        });

        let tx_poll = action_tx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100));
            let mut tick_count = 0.0;
            loop {
                interval.tick().await;
                let _ = tx_poll.send(Action::Tick);
                let _ = tx_poll.send(Action::Render);
                
                // Read metrics
                if let Ok(content) = std::fs::read_to_string("/tmp/prism_metrics.json") {
                    if let Ok(metrics) = serde_json::from_str::<MetricsData>(&content) {
                        let _ = tx_poll.send(Action::UpdateMetrics(metrics));
                    }
                }
                
                // Read Gatekeeper rules
                if let Ok(dir) = std::fs::read_dir("/tmp/prism/rules") {
                    let rules: Vec<String> = dir
                        .filter_map(Result::ok)
                        .map(|e| e.file_name().to_string_lossy().to_string())
                        .collect();
                    let _ = tx_poll.send(Action::UpdateHitl(rules));
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
                            for line in content.lines().rev().take(100) {
                                let ts = line.split("\"timestamp\":\"").nth(1).unwrap_or("Just now").split('"').next().unwrap_or("Just now");
                                let err = line.split("\"error\":\"").nth(1).unwrap_or("Unknown").split('"').next().unwrap_or("Unknown");
                                let payload = line.split("\"payload\":\"").nth(1).unwrap_or(line).split('"').next().unwrap_or(line);
                                dlq_res.push((ts.to_string(), err.to_string(), payload.to_string()));
                            }
                        }
                    }
                }
                let _ = tx_poll.send(Action::UpdateDlq(dlq_res));
                
                // Read Ledger
                tick_count += 0.1;
                let mut count = 0;
                let mut tail = "Empty".to_string();
                if let Ok(content) = std::fs::read_to_string("/tmp/prism/vault/ledger.log") {
                    let lines: Vec<&str> = content.lines().collect();
                    count = lines.len() as u64;
                    if let Some(last) = lines.last() {
                        tail = last.to_string();
                    }
                }
                let _ = tx_poll.send(Action::UpdateLedger(tail, tick_count, count));
            }
        });

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
            Action::Quit => self.should_quit = true,
            Action::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
                        KeyCode::Tab => {
                            self.active_tab = match self.active_tab {
                                ActiveTab::Dashboard => ActiveTab::Gatekeeper,
                                ActiveTab::Gatekeeper => ActiveTab::DlqViewer,
                                ActiveTab::DlqViewer => ActiveTab::Dashboard,
                            };
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if self.active_tab == ActiveTab::Gatekeeper {
                                let i = match self.hitl_state.selected() {
                                    Some(i) => if i >= self.hitl_rules.len().saturating_sub(1) { 0 } else { i + 1 },
                                    None => 0,
                                };
                                self.hitl_state.select(Some(i));
                            } else if self.active_tab == ActiveTab::DlqViewer {
                                let i = match self.dlq_state.selected() {
                                    Some(i) => if i >= self.dlq_items.len().saturating_sub(1) { 0 } else { i + 1 },
                                    None => 0,
                                };
                                self.dlq_state.select(Some(i));
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            if self.active_tab == ActiveTab::Gatekeeper {
                                let i = match self.hitl_state.selected() {
                                    Some(i) => if i == 0 { self.hitl_rules.len().saturating_sub(1) } else { i - 1 },
                                    None => 0,
                                };
                                self.hitl_state.select(Some(i));
                            } else if self.active_tab == ActiveTab::DlqViewer {
                                let i = match self.dlq_state.selected() {
                                    Some(i) => if i == 0 { self.dlq_items.len().saturating_sub(1) } else { i - 1 },
                                    None => 0,
                                };
                                self.dlq_state.select(Some(i));
                            }
                        }
                        KeyCode::Char('a') | KeyCode::Enter => {
                            if self.active_tab == ActiveTab::Gatekeeper {
                                if let Some(i) = self.hitl_state.selected() {
                                    if i < self.hitl_rules.len() {
                                        let rule = self.hitl_rules[i].clone();
                                        // Simulate approval by renaming
                                        let _ = std::fs::rename(
                                            format!("/tmp/prism/rules/{}", rule),
                                            format!("/tmp/prism/rules/{}.approved", rule)
                                        );
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            Action::UpdateMetrics(m) => {
                self.metrics = m;
                self.eps_history.push_back(self.metrics.eps);
                if self.eps_history.len() > 150 {
                    self.eps_history.pop_front();
                }
            }
            Action::UpdateHitl(rules) => {
                self.hitl_rules = rules.into_iter().filter(|r| !r.ends_with(".approved")).collect();
            }
            Action::UpdateDlq(items) => {
                self.dlq_items = items;
            }
            Action::UpdateLedger(tail, tick, count) => {
                self.ledger_tail = tail;
                self.tick_count = tick;
                self.ledger_history.push((self.tick_count, count as f64));
                if self.ledger_history.len() > 150 {
                    self.ledger_history.remove(0);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn draw(&mut self, f: &mut Frame) {
        let size = f.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Tabs
                Constraint::Min(0),    // Main Content
                Constraint::Length(1), // Footer
            ])
            .split(size);

        // Header / Tabs
        let titles = vec![" [1] Dashboard ", " [2] Gatekeeper (HitL) ", " [3] DLQ Explorer "];
        let tab_index = match self.active_tab {
            ActiveTab::Dashboard => 0,
            ActiveTab::Gatekeeper => 1,
            ActiveTab::DlqViewer => 2,
        };
        
        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title(" PRISM SIEM ENGINE "))
            .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD).add_modifier(Modifier::REVERSED))
            .select(tab_index)
            .divider(Span::raw("|"));
        f.render_widget(tabs, chunks[0]);

        // Main Content Area
        match self.active_tab {
            ActiveTab::Dashboard => self.draw_dashboard(f, chunks[1]),
            ActiveTab::Gatekeeper => self.draw_gatekeeper(f, chunks[1]),
            ActiveTab::DlqViewer => self.draw_dlq(f, chunks[1]),
        }

        // Footer
        let footer = Paragraph::new(Line::from(vec![
            Span::raw(" (Tab) Switch Views | (Q) Quit | (Up/Down) Navigate | (A/Enter) Approve Rule "),
        ])).alignment(Alignment::Center).style(Style::default().fg(Color::DarkGray));
        f.render_widget(footer, chunks[2]);
    }

    fn draw_dashboard(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        // Top: EPS Metrics
        let eps_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(0)])
            .split(chunks[0]);

        // Huge EPS Gauge
        let eps_gauge = Gauge::default()
            .block(Block::default().title(" LIVE EPS THROUGHPUT ").borders(Borders::ALL).border_type(BorderType::Rounded))
            .gauge_style(Style::default().fg(Color::Green).bg(Color::Black).add_modifier(Modifier::BOLD))
            .percent((self.metrics.eps.min(200000) as u16 / 2000).min(100))
            .label(format!(" {} EPS ", self.metrics.eps));
        f.render_widget(eps_gauge, eps_chunks[0]);

        let history_vec: Vec<u64> = self.eps_history.iter().copied().collect();
        let sparkline = Sparkline::default()
            .block(Block::default().title(" EVENT VELOCITY ").borders(Borders::ALL).border_type(BorderType::Rounded))
            .data(&history_vec)
            .style(Style::default().fg(Color::Cyan));
        f.render_widget(sparkline, eps_chunks[1]);

        // Bottom: Merkle Ledger
        let ledger_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(chunks[1]);

        let stats = format!(
            "\nTotal Processed:\n{}\n\nTotal Dropped:\n{}\n\nDLQ Count:\n{}\n\n\nLatest Merkle Root:\n{}",
            self.metrics.processed, self.metrics.drops, self.metrics.dlq, self.ledger_tail
        );
        
        let stats_widget = Paragraph::new(stats)
            .block(Block::default().title(" SYSTEM STATS ").borders(Borders::ALL).border_type(BorderType::Rounded).style(Style::default().fg(Color::Magenta)))
            .alignment(Alignment::Center);
        f.render_widget(stats_widget, ledger_chunks[0]);

        let datasets = vec![
            Dataset::default()
                .name("Prov. Ledger Size")
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Yellow))
                .graph_type(GraphType::Line)
                .data(&self.ledger_history),
        ];
        let chart = Chart::new(datasets)
            .block(Block::default().title(" IMMUTABLE PROVENANCE LEDGER ").borders(Borders::ALL).border_type(BorderType::Rounded))
            .x_axis(Axis::default().title("Time").bounds([self.tick_count.max(15.0) - 15.0, self.tick_count.max(15.0)]))
            .y_axis(Axis::default().title("Entries").bounds([0.0, self.ledger_history.last().map(|(_, y)| *y).unwrap_or(100.0).max(10.0)]));
        f.render_widget(chart, ledger_chunks[1]);
    }

    fn draw_gatekeeper(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(area);

        let items: Vec<ListItem> = self.hitl_rules.iter().map(|rule| {
            ListItem::new(Line::from(vec![
                Span::styled(" ⚠ [PENDING] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(rule),
            ]))
        }).collect();
        
        let list = List::new(items)
            .block(Block::default().title(" AI PARSERS AWAITING APPROVAL ").borders(Borders::ALL).border_type(BorderType::Rounded))
            .highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White).add_modifier(Modifier::BOLD))
            .highlight_symbol(">> ");
        
        f.render_stateful_widget(list, chunks[0], &mut self.hitl_state);

        // Preview Pane
        let preview_text = if let Some(i) = self.hitl_state.selected() {
            if i < self.hitl_rules.len() {
                if let Ok(content) = std::fs::read_to_string(format!("/tmp/prism/rules/{}", self.hitl_rules[i])) {
                    content
                } else {
                    "File unreadable".to_string()
                }
            } else {
                "No rule selected".to_string()
            }
        } else {
            "Select a rule to preview".to_string()
        };

        let preview = Paragraph::new(preview_text)
            .block(Block::default().title(" RULE PREVIEW ").borders(Borders::ALL).border_type(BorderType::Rounded).style(Style::default().fg(Color::Cyan)));
        f.render_widget(preview, chunks[1]);
    }

    fn draw_dlq(&mut self, f: &mut Frame, area: Rect) {
        let header_cells = ["Timestamp", "Error Type", "Raw Payload"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)));
        let header = Row::new(header_cells).style(Style::default().bg(Color::DarkGray)).height(1).bottom_margin(1);
        
        let rows: Vec<Row> = self.dlq_items.iter().map(|(ts, err, payload)| {
            Row::new(vec![Cell::from(ts.as_str()), Cell::from(err.as_str()), Cell::from(payload.as_str())])
        }).collect();

        let table = Table::new(rows, [Constraint::Length(25), Constraint::Length(30), Constraint::Min(20)])
            .header(header)
            .block(Block::default().title(" DEAD LETTER QUEUE (UNKNOWN LOGS) ").borders(Borders::ALL).border_type(BorderType::Rounded).style(Style::default().fg(Color::LightRed)))
            .highlight_style(Style::default().bg(Color::Red).fg(Color::White).add_modifier(Modifier::BOLD))
            .highlight_symbol(">> ");
        
        f.render_stateful_widget(table, area, &mut self.dlq_state);
    }
}
