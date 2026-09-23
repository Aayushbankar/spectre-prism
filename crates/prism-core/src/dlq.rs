use std::fs::{File, OpenOptions};
use std::io::Write;
use anyhow::Result;
use prism_common::RawEvent;
use chrono::Utc;

pub struct DeadLetterQueue {
    plaintext_file: File,
    jsonl_file: File,
}

impl DeadLetterQueue {
    pub fn new(path: Option<&str>) -> Result<Self> {
        let actual_path = path.unwrap_or("/var/run/prism/dlq.log");
        if let Some(parent) = std::path::Path::new(actual_path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let plaintext_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(actual_path)?;
            
        let jsonl_path = if actual_path.ends_with(".log") {
            actual_path.replace(".log", ".jsonl")
        } else {
            format!("{}.jsonl", actual_path)
        };
        
        let jsonl_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&jsonl_path)?;

        Ok(Self { plaintext_file, jsonl_file })
    }

    pub fn push(&mut self, event: &RawEvent, reason: &str) -> Result<()> {
        let payload_str = std::str::from_utf8(&event.payload).unwrap_or("<binary>");
        let ts = Utc::now().to_rfc3339();
        
        // Plaintext for TUI
        writeln!(self.plaintext_file, "[{}] REASON={} PAYLOAD={}", ts, reason, payload_str)?;
        self.plaintext_file.sync_all()?;
        
        // JSONL for prism-brain IPC
        let json = serde_json::to_string(event)?;
        writeln!(self.jsonl_file, "{}", json)?;
        self.jsonl_file.sync_all()?;
        
        Ok(())
    }
}
