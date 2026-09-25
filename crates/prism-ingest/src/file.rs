
use std::path::PathBuf;
use linemux::MuxedLines;
use anyhow::Result;

use crate::dispatcher::DispatcherSender;
use prism_common::{LogSource, RawEvent, ProvenanceMeta};
use bytes::Bytes;
use chrono::Utc;

pub struct FileTailer {
    file_path: PathBuf,
    sender: DispatcherSender,
}

impl FileTailer {
    pub fn new(file_path: PathBuf, sender: DispatcherSender) -> Self {
        Self { file_path, sender }
    }

    pub async fn run(self) -> Result<()> {
        let mut lines = MuxedLines::new()?;
        
        let path_str = self.file_path.to_string_lossy().to_string();
        lines.add_file(&self.file_path).await?;
        println!("File Tailer started on {}", path_str);

        while let Ok(Some(line)) = lines.next_line().await {
            let line_content = line.line().trim_end();
            if line_content.is_empty() {
                continue;
            }

            let payload_bytes = Bytes::copy_from_slice(line_content.as_bytes());
            let source = LogSource::FileTail { 
                path: path_str.clone(),
                offset: 0, // linemux abstracts absolute offsets, so we log 0 for streaming tail
            };
            
            let hash = blake3::hash(&payload_bytes);
            let timestamp = Utc::now();
            
            let event = RawEvent {
                payload: payload_bytes,
                metadata: ProvenanceMeta {
                    hash: hash.into(),
                    timestamp,
                    source,
                },
            };
            
            // Apply backpressure natively via flume bounded channel
            let _ = self.sender.try_broadcast(event);
        }

        Ok(())
    }
}
