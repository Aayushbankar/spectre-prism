use std::fs::{File, OpenOptions};
use std::io::Write;
use anyhow::Result;
use prism_common::RawEvent;
use chrono::Utc;

pub struct DeadLetterQueue {
    file: File,
}

impl DeadLetterQueue {
    pub fn new(path: &str) -> Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        Ok(Self { file })
    }

    pub fn push(&mut self, event: &RawEvent, reason: &str) -> Result<()> {
        let payload_str = std::str::from_utf8(&event.payload).unwrap_or("<binary>");
        let ts = Utc::now().to_rfc3339();
        writeln!(self.file, "[{}] REASON={} PAYLOAD={}", ts, reason, payload_str)?;
        self.file.sync_all()?;
        Ok(())
    }
}
