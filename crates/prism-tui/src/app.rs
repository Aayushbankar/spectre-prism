use color_eyre::{Result as EyreResult, eyre::anyhow as eyre_anyhow};
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    widgets::{Block, Borders, BorderType, Gauge, Paragraph, Table, Row, Cell, List, ListItem, ListState, TableState, Chart, Dataset, GraphType, Axis, Tabs},
    style::{Color, Style, Modifier},
    text::{Span, Line},
    symbols,
    Frame,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::mpsc;
use std::path::PathBuf;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleMetadata {
    pub rule_id: String,
    pub device_type: String,
    pub vendor_name: String,
    pub signature: String,
    pub state: String,
    pub created_at: f64,
    pub updated_at: f64,
    pub vrl_path: String,
    pub yaml_path: String,
    pub raw_log_sample: String,
    pub dry_run_result: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Action {
    Tick,
    Render,
    Key(KeyEvent),
    Resize(u16, u16),
    UpdateMetrics(MetricsData),
    UpdateDlq(Vec<(String, String, String)>),
    UpdateHitl(Vec<RuleMetadata>),
    UpdateLedger(String, f64, u64),
    ApproveRule(String),
    RejectRule(String),
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
    pub eps_history: Vec<(f64, f64)>,
    pub dlq_items: Vec<(String, String, String)>,
    pub hitl_rules: Vec<RuleMetadata>,
    pub ledger_tail: String,
    pub ledger_history: Vec<(f64, f64)>,
    pub tick_count: f64,
    
    // UI state
    pub hitl_state: ListState,
    pub dlq_state: TableState,
    pub show_help: bool,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
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
            show_help: false,
        }
    }

    pub async fn run(&mut self) -> EyreResult<()> {
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
            let mut interval = tokio::time::interval(Duration::from_millis(1000));
            let mut tick_count = 0.0;
            loop {
                interval.tick().await;
                tick_count += 1.0;
                
                let _ = tx_poll.send(Action::Tick);
                let _ = tx_poll.send(Action::Render);
                
                // Read metrics
                if let Ok(content) = fs::read_to_string("/tmp/prism_metrics.json")
                    && let Ok(metrics) = serde_json::from_str::<MetricsData>(&content) {
                        let _ = tx_poll.send(Action::UpdateMetrics(metrics));
                    }
                
                // Read Gatekeeper rules from metadata
                if let Ok(rules) = Self::read_rule_metadata() {
                    let _ = tx_poll.send(Action::UpdateHitl(rules));
                }
                
                // Read DLQ
                let vault_dir = Self::get_base_dir().join("vault");
                let mut dlq_res = Vec::new();
                if let Ok(dir) = fs::read_dir(&vault_dir) {
                    let mut latest_dlq = None;
                    let mut latest_time = std::time::SystemTime::UNIX_EPOCH;
                    for entry in dir.filter_map(Result::ok) {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if name.starts_with("dlq_") && name.ends_with(".log")
                            && let Ok(meta) = entry.metadata()
                                && let Ok(modified) = meta.modified()
                                    && modified > latest_time {
                                        latest_time = modified;
                                        latest_dlq = Some(entry.path());
                                    }
                    }
                    if let Some(path) = latest_dlq
                        && let Ok(content) = fs::read_to_string(&path) {
                            for line in content.lines().rev().take(100) {
                                let ts = line.split(']').next().unwrap_or("Just now").trim_start_matches('[');
                                let err = line.split("REASON=").nth(1).unwrap_or("Unknown").split(" PAYLOAD=").next().unwrap_or("Unknown");
                                let payload = line.split("PAYLOAD=").nth(1).unwrap_or(line);
                                dlq_res.push((ts.to_string(), err.to_string(), payload.to_string()));
                            }
                        }
                }
                let _ = tx_poll.send(Action::UpdateDlq(dlq_res));
                
                // Read Ledger
                let mut count = 0;
                let mut tail = "Empty".to_string();
                let ledger_path = Self::get_base_dir().join("vault").join("ledger.log");
                if let Ok(content) = fs::read_to_string(&ledger_path) {
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

    fn get_base_dir() -> PathBuf {
        std::env::var("PRISM_BASE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/tmp/prism"))
    }

    fn read_rule_metadata() -> EyreResult<Vec<RuleMetadata>> {
        let mut rules = Vec::new();
        let metadata_dir = Self::get_base_dir().join("rule_metadata");
        if metadata_dir.exists() {
            for entry in fs::read_dir(metadata_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json")
                    && let Ok(content) = fs::read_to_string(&path)
                        && let Ok(meta) = serde_json::from_str::<RuleMetadata>(&content) {
                            rules.push(meta);
                        }
            }
        }
        rules.sort_by(|a, b| b.created_at.partial_cmp(&a.created_at).unwrap_or(std::cmp::Ordering::Equal));
        Ok(rules)
    }

    fn approve_rule_via_gatekeeper(rule_id: &str) -> EyreResult<()> {
        let metadata_dir = Self::get_base_dir().join("rule_metadata");
        let meta_path = metadata_dir.join(format!("{}.json", rule_id));
        
        if !meta_path.exists() {
            return Err(eyre_anyhow!("Rule metadata not found"));
        }
        
        let content = fs::read_to_string(&meta_path)?;
        let mut meta: RuleMetadata = serde_json::from_str(&content)?;
        
        if meta.state != "pending" {
            return Err(eyre_anyhow!("Rule is not in pending state"));
        }
        
        // Move VRL to active rules directory
        let vrl_src = PathBuf::from(&meta.vrl_path);
        let rules_dir = PathBuf::from("/tmp/prism/rules");
        fs::create_dir_all(&rules_dir)?;
        let vrl_dst = rules_dir.join(vrl_src.file_name().unwrap());
        
        // Atomic copy
        let tmp_dst = vrl_dst.with_extension("vrl.tmp");
        fs::copy(&vrl_src, &tmp_dst)?;
        fs::rename(&tmp_dst, &vrl_dst)?;
        
        // Update metadata
        meta.state = "approved".to_string();
        meta.vrl_path = vrl_dst.to_string_lossy().to_string();
        meta.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs_f64();
        
        fs::write(&meta_path, serde_json::to_string_pretty(&meta)?)?;
        
        Ok(())
    }

    fn reject_rule_via_gatekeeper(rule_id: &str) -> EyreResult<()> {
        let metadata_dir = Self::get_base_dir().join("rule_metadata");
        let meta_path = metadata_dir.join(format!("{}.json", rule_id));
        
        if !meta_path.exists() {
            return Err(eyre_anyhow!("Rule metadata not found"));
        }
        
        let content = fs::read_to_string(&meta_path)?;
        let mut meta: RuleMetadata = serde_json::from_str(&content)?;
        
        if meta.state != "pending" && meta.state != "failed" {
            return Err(eyre_anyhow!("Rule cannot be rejected"));
        }
        
        // Move files to rejected dir
        let rejected_dir = Self::get_base_dir().join("rejected_rules");
        fs::create_dir_all(&rejected_dir)?;
        
        for src_path_str in [&meta.vrl_path, &meta.yaml_path] {
            let src = PathBuf::from(src_path_str);
            if src.exists() {
                let dst = rejected_dir.join(src.file_name().unwrap());
                fs::rename(&src, &dst)?;
            }
        }
        
        meta.state = "rejected".to_string();
        meta.error = Some("Rejected by operator via TUI".to_string());
        meta.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs_f64();
        
        fs::write(&meta_path, serde_json::to_string_pretty(&meta)?)?;
        
        Ok(())
    }

    fn update(&mut self, action: Action) -> EyreResult<()> {
        match action {
            Action::Quit => self.should_quit = true,
            Action::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            if self.show_help {
                                self.show_help = false;
                            } else {
                                self.should_quit = true;
                            }
                        }
                        KeyCode::Char('?') | KeyCode::F(1) => {
                            self.show_help = !self.show_help;
                        }
                        KeyCode::Tab => {
                            self.active_tab = match self.active_tab {
                                ActiveTab::Dashboard => ActiveTab::Telemetry,
                                ActiveTab::Telemetry => ActiveTab::Gatekeeper,
                                ActiveTab::Gatekeeper => ActiveTab::DlqViewer,
                                ActiveTab::DlqViewer => ActiveTab::Dashboard,
                            };
                        }
                        KeyCode::Char('1') => {
                            self.active_tab = ActiveTab::Dashboard;
                        }
                        KeyCode::Char('2') => {
                            self.active_tab = ActiveTab::Telemetry;
                        }
                        KeyCode::Char('3') => {
                            self.active_tab = ActiveTab::Gatekeeper;
                        }
                        KeyCode::Char('4') => {
                            self.active_tab = ActiveTab::DlqViewer;
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
                            if self.active_tab == ActiveTab::Gatekeeper
                                && let Some(i) = self.hitl_state.selected()
                                    && i < self.hitl_rules.len() {
                                        let rule_id = self.hitl_rules[i].rule_id.clone();
                                        match Self::approve_rule_via_gatekeeper(&rule_id) {
                                            Ok(_) => {
                                                // Refresh will happen on next poll
                                            }
                                            Err(e) => {
                                                eprintln!("Failed to approve rule: {}", e);
                                            }
                                        }
                                    }
                        }
                        KeyCode::Char('r') | KeyCode::Delete => {
                            if self.active_tab == ActiveTab::Gatekeeper
                                && let Some(i) = self.hitl_state.selected()
                                    && i < self.hitl_rules.len() {
                                        let rule_id = self.hitl_rules[i].rule_id.clone();
                                        match Self::reject_rule_via_gatekeeper(&rule_id) {
                                            Ok(_) => {}
                                            Err(e) => {
                                                eprintln!("Failed to reject rule: {}", e);
                                            }
                                        }
                                    }
                        }
                        KeyCode::Char('p') => {
                            if self.active_tab == ActiveTab::Gatekeeper
                                && let Some(i) = self.hitl_state.selected()
                                    && i < self.hitl_rules.len() {
                                        // Show full preview in a larger area
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
                self.hitl_rules = rules;
                if self.hitl_state.selected().is_none() && !self.hitl_rules.is_empty() {
                    self.hitl_state.select(Some(0));
                }
            }
            Action::UpdateDlq(items) => {
                self.dlq_items = items;
                if self.dlq_state.selected().is_none() && !self.dlq_items.is_empty() {
                    self.dlq_state.select(Some(0));
                }
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
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(if self.show_help { 8 } else { 1 }),
            ])
            .split(size);

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

        if self.show_help {
            let help_text = vec![
                Line::from(vec![
                    Span::styled("KEYBINDINGS: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw("Tab/1-4: Switch tabs | "),
                    Span::styled("↑/↓ or j/k: ", Style::default().fg(Color::Cyan)),
                    Span::raw("Navigate | "),
                    Span::styled("Enter/A: ", Style::default().fg(Color::Green)),
                    Span::raw("Approve rule | "),
                    Span::styled("R/Del: ", Style::default().fg(Color::Red)),
                    Span::raw("Reject rule | "),
                    Span::styled("?: ", Style::default().fg(Color::Yellow)),
                    Span::raw("Toggle help | "),
                    Span::styled("Q: ", Style::default().fg(Color::Red)),
                    Span::raw("Quit"),
                ]),
                Line::from(vec![
                    Span::styled("GATEKEEPER STATES: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::styled("⏳ PENDING ", Style::default().fg(Color::Yellow)),
                    Span::raw("| "),
                    Span::styled("✅ APPROVED ", Style::default().fg(Color::Green)),
                    Span::raw("| "),
                    Span::styled("❌ REJECTED ", Style::default().fg(Color::Red)),
                    Span::raw("| "),
                    Span::styled("🚀 DEPLOYED ", Style::default().fg(Color::Blue)),
                    Span::raw("| "),
                    Span::styled("💥 FAILED ", Style::default().fg(Color::Magenta)),
                ]),
            ];
            let help = Paragraph::new(help_text)
                .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title(" HELP "))
                .alignment(Alignment::Left);
            f.render_widget(help, chunks[2]);
        } else {
            let footer = Paragraph::new(Line::from(vec![
                Span::raw(" (Tab/1-4) Switch | (↑/↓) Navigate | (A/Enter) Approve | (R) Reject | (?) Help | (Q) Quit "),
            ])).alignment(Alignment::Center).style(Style::default().fg(Color::DarkGray));
            f.render_widget(footer, chunks[2]);
        }
    }

    fn draw_dashboard(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let max_eps = self.eps_history.iter().map(|(_, y)| *y as u64).max().unwrap_or(1).max(1);
        let gauge_pct = ((self.metrics.eps * 100) / max_eps).min(100) as u16;
        let eps_gauge = Gauge::default()
            .block(Block::default().title(" INGESTION THROUGHPUT ").borders(Borders::ALL).border_type(BorderType::Rounded))
            .gauge_style(Style::default().fg(Color::LightGreen).bg(Color::DarkGray))
            .percent(gauge_pct)
            .label(format!(" {} EPS (peak: {}) ", self.metrics.eps, max_eps));
        f.render_widget(eps_gauge, chunks[0]);

        let eps_ds = vec![
            Dataset::default()
                .name("EPS Velocity")
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Cyan))
                .graph_type(GraphType::Line)
                .data(&self.eps_history),
        ];
        
        let max_eps_f = self.eps_history.iter().map(|(_, y)| *y).fold(0.0, f64::max).max(100.0);
        let chart = Chart::new(eps_ds)
            .block(Block::default().title(" EVENT VELOCITY ").borders(Borders::ALL).border_type(BorderType::Rounded))
            .x_axis(Axis::default().title("Time").bounds([self.tick_count.max(10.0) - 10.0, self.tick_count.max(10.0)]))
            .y_axis(Axis::default().title("EPS").bounds([0.0, max_eps_f]).labels(vec![
                Span::raw("0"),
                Span::raw(format!("{}", max_eps_f / 2.0)),
                Span::raw(format!("{}", max_eps_f)),
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
            
        let lat_str = format!("\n\n{} μs (estimated)", self.metrics.telemetry.latency_us);
        let lat_widget = Paragraph::new(lat_str)
            .block(Block::default().title(" AVG LATENCY ").borders(Borders::ALL).border_type(BorderType::Rounded))
            .style(Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        f.render_widget(lat_widget, top_chunks[0]);
        
        let mut ai_running = false;
        if let Ok(content) = fs::read_to_string("/tmp/prism_ai_status") {
            if let Ok(ts) = content.trim().parse::<f64>() {
                if let Ok(sys_time) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
                    && sys_time.as_secs_f64() - ts < 10.0 {
                        ai_running = true;
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

        let drops_str = format!("\n\n{}", self.metrics.drops);
        let drops_widget = Paragraph::new(drops_str)
            .block(Block::default().title(" PACKET DROPS ").borders(Borders::ALL).border_type(BorderType::Rounded))
            .style(Style::default().fg(if self.metrics.drops > 0 { Color::Red } else { Color::Green }).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        f.render_widget(drops_widget, top_chunks[2]);
        
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
            let (state_label, state_color) = match rule.state.as_str() {
                "pending" => ("⏳ PENDING", Color::Yellow),
                "approved" => ("✅ APPROVED", Color::Green),
                "rejected" => ("❌ REJECTED", Color::Red),
                "deployed" => ("🚀 DEPLOYED", Color::Blue),
                "failed" => ("💥 FAILED", Color::Magenta),
                _ => ("❓ UNKNOWN", Color::Gray),
            };
            
            let dry_run_indicator = if rule.dry_run_result.is_some() {
                if rule.state == "failed" { " ✗" } else { " ✓" }
            } else { "" };
            
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {} {}{}", state_label, dry_run_indicator, ""), Style::default().fg(state_color).add_modifier(Modifier::BOLD)),
                Span::raw(format!(" [{}] ", rule.device_type)),
                Span::raw(&rule.rule_id),
            ]))
        }).collect();
        
        let list = List::new(items)
            .block(Block::default().title(" AI PARSERS - HITL APPROVAL QUEUE ").borders(Borders::ALL).border_type(BorderType::Rounded).style(Style::default().fg(Color::Yellow)))
            .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD))
            .highlight_symbol(">> ");
        
        f.render_stateful_widget(list, chunks[0], &mut self.hitl_state);

        let preview_text = if let Some(i) = self.hitl_state.selected() {
            if i < self.hitl_rules.len() {
                let rule = &self.hitl_rules[i];
                let mut preview = format!(
                    "Rule ID: {}\nDevice: {}\nSignature: {}\nState: {}\nCreated: {}\n\n--- VRL CODE ---\n",
                    rule.rule_id, rule.device_type, rule.signature, rule.state.to_uppercase(),
                    chrono::DateTime::from_timestamp(rule.created_at as i64, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| "Unknown".to_string())
                );
                
                // Read VRL content
                let vrl_path = PathBuf::from(&rule.vrl_path);
                if vrl_path.exists() {
                    if let Ok(content) = fs::read_to_string(&vrl_path) {
                        preview.push_str(&content);
                    } else {
                        preview.push_str("Error reading VRL file");
                    }
                } else {
                    preview.push_str("VRL file not found at path");
                }
                
                if let Some(err) = &rule.error {
                    preview.push_str(&format!("\n\n--- ERROR ---\n{}", err));
                }
                if let Some(dry_run) = &rule.dry_run_result {
                    preview.push_str(&format!("\n\n--- DRY-RUN OUTPUT ---\n{}", dry_run));
                }
                
                preview
            } else {
                "No rule selected".to_string()
            }
        } else {
            "Select a rule to preview (A=Approve, R=Reject)".to_string()
        };

        let preview = Paragraph::new(preview_text)
            .block(Block::default().title(" RULE PREVIEW & DRY-RUN OUTPUT ").borders(Borders::ALL).border_type(BorderType::Rounded).style(Style::default().fg(Color::Cyan)))
            .wrap(ratatui::widgets::Wrap { trim: false });
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
            .row_highlight_style(Style::default().bg(Color::Red).fg(Color::White).add_modifier(Modifier::BOLD))
            .highlight_symbol(">> ");
        
        f.render_stateful_widget(table, area, &mut self.dlq_state);
    }
}