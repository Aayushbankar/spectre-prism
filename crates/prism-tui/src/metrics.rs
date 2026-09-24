use crate::app::TuiState;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::sync::atomic::Ordering;

fn parse_dlq(item: &str) -> (String, String, String) {
    if item.starts_with('{') {
        let ts = item.split("\"timestamp\":\"").nth(1).unwrap_or("").split("\"").next().unwrap_or("Unknown");
        let err = item.split("\"error\":\"").nth(1).unwrap_or("").split("\"").next().unwrap_or("Unknown");
        let payload = item.split("\"payload\":\"").nth(1).unwrap_or("").split("\"").next().unwrap_or("...");
        if ts != "Unknown" || err != "Unknown" {
            return (ts.to_string(), err.to_string(), payload.to_string());
        }
    }
    
    let parts: Vec<&str> = item.splitn(3, '|').collect();
    if parts.len() == 3 {
        (parts[0].to_string(), parts[1].to_string(), parts[2].to_string())
    } else {
        (item.to_string(), "Unknown".to_string(), "...".to_string())
    }
}

fn read_last_lines(path: &std::path::Path, num_lines: usize) -> Vec<String> {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return vec![],
    };
    
    let metadata = match file.metadata() {
        Ok(m) => m,
        Err(_) => return vec![],
    };
    
    let len = metadata.len();
    let read_size = std::cmp::min(16384, len) as usize;
    let mut buffer = vec![0; read_size];
    
    if file.seek(SeekFrom::End(-(read_size as i64))).is_err() {
        return vec![];
    }
    
    if file.read_exact(&mut buffer).is_err() {
        return vec![];
    }
    
    let content = String::from_utf8_lossy(&buffer);
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    if lines.len() > 1 && len > read_size as u64 {
        lines.remove(0);
    }
    
    let skip = if lines.len() > num_lines { lines.len() - num_lines } else { 0 };
    lines.into_iter().skip(skip).collect()
}

pub fn poll_metrics(state: &mut TuiState) {
    let current_eps = state.eps.load(Ordering::Relaxed);
    state.eps_history.push_back(current_eps);
    if state.eps_history.len() > 100 {
        state.eps_history.pop_front();
    }

    let dlq_lines = read_last_lines(&state.dlq_path, 20);
    state.dlq_items = dlq_lines.into_iter().map(|s| parse_dlq(&s)).collect();

    state.tick_count += 1.0;
    
    // Efficiently get file size to estimate or check ledger count
    if let Ok(metadata) = std::fs::metadata(&state.ledger_path) {
        let len = metadata.len();
        state.ledger_line_count = len / 64; 
        state.ledger_history.push((state.tick_count, state.ledger_line_count as f64));
        if state.ledger_history.len() > 100 {
            state.ledger_history.remove(0);
        }
    }
    
    let ledger_lines = read_last_lines(&state.ledger_path, 1);
    if let Some(tail) = ledger_lines.last() {
        state.ledger_tail = tail.clone();
    } else {
        state.ledger_tail = "Empty".to_string();
    }

    if let Ok(dir) = std::fs::read_dir(&state.rules_dir) {
        state.hitl_rules = dir
            .filter_map(Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .collect();
    } else {
        state.hitl_rules.clear();
    }
}
