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
pub struct TelemetryData {
    pub fortinet: u64,
    pub cisco: u64,
    pub paloalto: u64,
    pub latency_us: u64,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct MetricsData {
    pub eps: u64,
    pub processed: u64,
    pub drops: u64,
    pub dlq: u64,
    #[serde(default)]
    pub telemetry: TelemetryData,
}

#[derive(PartialEq)]
pub enum ActiveTab {
    Dashboard,
    Telemetry,
    Gatekeeper,
    DlqViewer,
}

pub struct App {
    pub should_quit: bool,
    pub active_tab: ActiveTab,
    
    // Data state
    pub metrics: MetricsData,
    pub eps_history: Vec<(f64, f64)>, // Format for Chart
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
            eps_history: Vec::new(),
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
                tick_count += 0.1;
                
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
                                let ts = line.split(']').next().unwrap_or("Just now").trim_start_matches('[');
                                let err = line.split("REASON=").nth(1).unwrap_or("Unknown").split(" PAYLOAD=").next().unwrap_or("Unknown");
                                let payload = line.split("PAYLOAD=").nth(1).unwrap_or(line);
                                dlq_res.push((ts.to_string(), err.to_string(), payload.to_string()));
                            }
                        }
                    }
                }
                let _ = tx_poll.send(Action::UpdateDlq(dlq_res));
                
                // Read Ledger
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
                                ActiveTab::Dashboard => ActiveTab::Telemetry,
                                ActiveTab::Telemetry => ActiveTab::Gatekeeper,
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
                self.eps_history.push((self.tick_count, self.metrics.eps as f64));
                if self.eps_history.len() > 100 {
                    self.eps_history.remove(0);
                }
            }
            Action::UpdateHitl(rules) => {
                self.hitl_rules = rules.into_iter().filter(|r| !r.ends_with(".approved") && r.ends_with(".vrl")).collect();
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
        let size = f.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Tabs
                Constraint::Min(0),    // Main Content
                Constraint::Length(1), // Footer
            ])
            .split(size);

        // Elegant Title & Tabs
        let titles = vec![" [1] Dashboard ", " [2] Telemetry ", " [3] Gatekeeper (HitL) ", " [4] DLQ Explorer "];
        let tab_index = match self.active_tab {
            ActiveTab::Dashboard => 0,
            ActiveTab::Telemetry => 1,
            ActiveTab::Gatekeeper => 2,
            ActiveTab::DlqViewer => 3,
        };
        
        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Double).title(" PRISM ADVANCED DATA PLANE ").style(Style::default().fg(Color::Cyan)))
            .highlight_style(Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD))
            .select(tab_index)
            .divider(Span::raw(" | "));
        f.render_widget(tabs, chunks[0]);

        match self.active_tab {
            ActiveTab::Dashboard => self.draw_dashboard(f, chunks[1]),
            ActiveTab::Telemetry => self.draw_telemetry(f, chunks[1]),
            ActiveTab::Gatekeeper => self.draw_gatekeeper(f, chunks[1]),
            ActiveTab::DlqViewer => self.draw_dlq(f, chunks[1]),
        }

        let footer = Paragraph::new(Line::from(vec![
            Span::raw(" (Tab) Switch Views | (Q) Quit | (Up/Down) Navigate | (A/Enter) Approve Rule "),
        ])).alignment(Alignment::Center).style(Style::default().fg(Color::DarkGray));
        f.render_widget(footer, chunks[2]);
    }

    fn draw_dashboard(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Slim EPS Gauge
        let max_eps = self.eps_history.iter().map(|(_, y)| *y as u64).max().unwrap_or(1).max(1);
        let gauge_pct = ((self.metrics.eps * 100) / max_eps).min(100) as u16;
        let eps_gauge = Gauge::default()
            .block(Block::default().title(" INGESTION THROUGHPUT ").borders(Borders::ALL).border_type(BorderType::Rounded))
            .gauge_style(Style::default().fg(Color::LightGreen).bg(Color::DarkGray))
            .percent(gauge_pct)
            .label(format!(" {} EPS (peak: {}) ", self.metrics.eps, max_eps));
        f.render_widget(eps_gauge, chunks[0]);

        // Beautiful Line Chart for EPS
        let eps_ds = vec![
            Dataset::default()
                .name("EPS Velocity")
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Cyan))
                .graph_type(GraphType::Line)
                .data(&self.eps_history),
        ];
        
        let max_eps = self.eps_history.iter().map(|(_, y)| *y).fold(0.0, f64::max).max(100.0);
        let chart = Chart::new(eps_ds)
            .block(Block::default().title(" EVENT VELOCITY ").borders(Borders::ALL).border_type(BorderType::Rounded))
            .x_axis(Axis::default().title("Time").bounds([self.tick_count.max(10.0) - 10.0, self.tick_count.max(10.0)]))
            .y_axis(Axis::default().title("EPS").bounds([0.0, max_eps]).labels(vec![
                Span::raw("0"),
                Span::raw(format!("{}", max_eps / 2.0)),
                Span::raw(format!("{}", max_eps)),
            ]));
        f.render_widget(chart, chunks[1]);

        let bottom_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
            .split(chunks[2]);

        let stats = format!(
            "\n\nTotal Processed: {}\n\nTotal Dropped: {}\n\nDLQ Count: {}\n\nRoot:\n{}",
            self.metrics.processed, self.metrics.drops, self.metrics.dlq,
            if self.ledger_tail.len() > 16 { &self.ledger_tail[..16] } else { &self.ledger_tail }
        );
        let stats_widget = Paragraph::new(stats)
            .block(Block::default().title(" SYSTEM STATS ").borders(Borders::ALL).border_type(BorderType::Rounded).style(Style::default().fg(Color::Magenta)))
            .alignment(Alignment::Center);
        f.render_widget(stats_widget, bottom_chunks[0]);

        let ledger_ds = vec![
            Dataset::default()
                .name("Ledger Size")
                .marker(symbols::Marker::Dot)
                .style(Style::default().fg(Color::Yellow))
                .graph_type(GraphType::Scatter)
                .data(&self.ledger_history),
        ];
        let max_ledger = self.ledger_history.iter().map(|(_, y)| *y).fold(0.0, f64::max).max(10.0);
        let ledger_chart = Chart::new(ledger_ds)
            .block(Block::default().title(" IMMUTABLE PROVENANCE LEDGER ").borders(Borders::ALL).border_type(BorderType::Rounded))
            .x_axis(Axis::default().bounds([self.tick_count.max(10.0) - 10.0, self.tick_count.max(10.0)]))
            .y_axis(Axis::default().bounds([0.0, max_ledger]));
        f.render_widget(ledger_chart, bottom_chunks[1]);
    }

    fn draw_telemetry(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(area);
            
        let top_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(33), Constraint::Percentage(34), Constraint::Percentage(33)])
            .split(chunks[0]);
            
        // Latency
        let lat_str = format!("\n\n{} μs (estimated)", self.metrics.telemetry.latency_us);
        let lat_widget = Paragraph::new(lat_str)
            .block(Block::default().title(" AVG LATENCY ").borders(Borders::ALL).border_type(BorderType::Rounded))
            .style(Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        f.render_widget(lat_widget, top_chunks[0]);
        
        // AI Status
        let mut ai_running = false;
        if let Ok(content) = std::fs::read_to_string("/tmp/prism_ai_status") {
            if let Ok(ts) = content.trim().parse::<f64>() {
                if let Ok(sys_time) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
                    if sys_time.as_secs_f64() - ts < 10.0 {
                        ai_running = true;
                    }
                }
            } else if content.trim() == "online" {
                ai_running = true;
            }
        }
        
        let ai_status = if ai_running { "\n\n🟢 ONLINE" } else { "\n\n🔴 OFFLINE" };
        let ai_widget = Paragraph::new(ai_status)
            .block(Block::default().title(" AI CONTROL PLANE ").borders(Borders::ALL).border_type(BorderType::Rounded))
            .style(Style::default().fg(if ai_running { Color::Green } else { Color::Red }).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        f.render_widget(ai_widget, top_chunks[1]);

        // Drops
        let drops_str = format!("\n\n{}", self.metrics.drops);
        let drops_widget = Paragraph::new(drops_str)
            .block(Block::default().title(" PACKET DROPS ").borders(Borders::ALL).border_type(BorderType::Rounded))
            .style(Style::default().fg(if self.metrics.drops > 0 { Color::Red } else { Color::Green }).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        f.render_widget(drops_widget, top_chunks[2]);
        
        // Vendor Breakdown
        let total = (self.metrics.telemetry.fortinet + self.metrics.telemetry.cisco + self.metrics.telemetry.paloalto).max(1);
        let f_pct = (self.metrics.telemetry.fortinet * 100) / total;
        let c_pct = (self.metrics.telemetry.cisco * 100) / total;
        let p_pct = (self.metrics.telemetry.paloalto * 100) / total;
        
        let vendor_stats = format!(
            "\n  Fortinet:    {} ({}%)\n\n  Cisco ASA:   {} ({}%)\n\n  Palo Alto:   {} ({}%)",
            self.metrics.telemetry.fortinet, f_pct,
            self.metrics.telemetry.cisco, c_pct,
            self.metrics.telemetry.paloalto, p_pct
        );
        let vendor_widget = Paragraph::new(vendor_stats)
            .block(Block::default().title(" LOG VENDOR BREAKDOWN ").borders(Borders::ALL).border_type(BorderType::Rounded))
            .style(Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Left);
        f.render_widget(vendor_widget, chunks[1]);
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
            .block(Block::default().title(" AI PARSERS AWAITING APPROVAL ").borders(Borders::ALL).border_type(BorderType::Rounded).style(Style::default().fg(Color::Yellow)))
            .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD))
            .highlight_symbol(">> ");
        
        f.render_stateful_widget(list, chunks[0], &mut self.hitl_state);

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
            Row::new(vec![
                Cell::from(ts.as_str()).style(Style::default().fg(Color::DarkGray)), 
                Cell::from(err.as_str()).style(Style::default().fg(Color::LightRed)), 
                Cell::from(payload.as_str()).style(Style::default().fg(Color::White))
            ])
        }).collect();

        let table = Table::new(rows, [Constraint::Length(25), Constraint::Length(25), Constraint::Min(20)])
            .header(header)
            .block(Block::default().title(" DEAD LETTER QUEUE (UNKNOWN LOGS) ").borders(Borders::ALL).border_type(BorderType::Rounded).style(Style::default().fg(Color::LightRed)))
            .highlight_style(Style::default().bg(Color::Red).fg(Color::White).add_modifier(Modifier::BOLD))
            .highlight_symbol(">> ");
        
        f.render_stateful_widget(table, area, &mut self.dlq_state);
    }
}
