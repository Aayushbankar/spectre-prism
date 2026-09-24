use crate::app::TuiState;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph, Sparkline, Gauge, Chart, Dataset, Axis, GraphType, Table, Row, Cell, List, ListItem},
    style::{Color, Style, Modifier},
    text::{Span, Line},
    symbols,
    Frame,
};
use std::sync::atomic::Ordering;

pub fn render_ui(f: &mut Frame, state: &TuiState) {
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
    let eps_val = state.eps.load(Ordering::Relaxed);
    let gauge_chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(top_chunks[0]);

    let gauge = Gauge::default()
        .block(Block::default().title("EPS Throughput (Gauge)").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Green).bg(Color::Black))
        .percent((eps_val.min(100000) as u16 / 1000).min(100))
        .label(format!("{} EPS", eps_val));
    f.render_widget(gauge, gauge_chunk[0]);

    let history_vec: Vec<u64> = state.eps_history.iter().copied().collect();
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
    
    let rows: Vec<Row> = state.dlq_items.iter().map(|(ts, err, payload)| {
        Row::new(vec![Cell::from(ts.as_str()), Cell::from(err.as_str()), Cell::from(payload.as_str())])
    }).collect();

    let table = Table::new(rows, [Constraint::Length(20), Constraint::Length(20), Constraint::Min(20)])
        .header(header)
        .block(Block::default().title("Dead Letter Queue (DLQ)").borders(Borders::ALL));
    f.render_widget(table, top_chunks[1]);

    // Pane 3: Merkle Ticker Chart
    let chart_chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(bottom_chunks[0]);

    let ledger_tail = if state.ledger_tail.is_empty() {
        "Empty".to_string()
    } else {
        state.ledger_tail.clone()
    };
    let tail_widget = Paragraph::new(format!("Latest Root: {}", ledger_tail))
        .block(Block::default().title("Integrity Merkle Tail").borders(Borders::ALL));
    f.render_widget(tail_widget, chart_chunk[0]);

    let datasets = vec![
        Dataset::default()
            .name("Ledger Count")
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(Color::Yellow))
            .graph_type(GraphType::Line)
            .data(&state.ledger_history),
    ];
    let chart = Chart::new(datasets)
        .block(Block::default().title("Root History").borders(Borders::ALL))
        .x_axis(Axis::default().title("Time").bounds([state.tick_count.max(100.0) - 100.0, state.tick_count.max(100.0)]))
        .y_axis(Axis::default().title("Entries").bounds([0.0, state.ledger_history.last().map(|(_, y)| *y).unwrap_or(100.0).max(10.0)]));
    f.render_widget(chart, chart_chunk[1]);

    // Pane 4: HitL List
    let items: Vec<ListItem> = state.hitl_rules.iter().map(|rule| {
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
