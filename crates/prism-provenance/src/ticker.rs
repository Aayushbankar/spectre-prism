use flume::Receiver;
use prism_common::RawEvent;
use crate::merkle::ProvenanceTree;
use crate::vault::VaultWriter;
use std::time::Duration;
use tokio::time::interval;
use anyhow::Result;

pub struct IntegrityTicker {
    rx: Receiver<RawEvent>,
    tree: ProvenanceTree,
    vault: VaultWriter,
    tick_interval: Duration,
    output_dir: String,
}

impl IntegrityTicker {
    pub fn new(rx: Receiver<RawEvent>, output_dir: &str, batch_size: usize, tick_interval: Duration) -> Result<Self> {
        let vault = VaultWriter::new(output_dir, batch_size)?;
        
        let ledger_path = format!("{}/ledger.log", output_dir);
        std::fs::OpenOptions::new().create(true).write(true).truncate(true).open(&ledger_path)?;

        Ok(Self {
            rx,
            tree: ProvenanceTree::new(),
            vault,
            tick_interval,
            output_dir: output_dir.to_string(),
        })
    }

    pub async fn run(mut self) -> Result<()> {
        let mut ticker = interval(self.tick_interval);

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    self.process_tick()?;
                }
                event_res = self.rx.recv_async() => {
                    match event_res {
                        Ok(event) => {
                            if self.tree.push_leaf(&event.metadata.hash).is_err() {
                                self.process_tick()?;
                                let _ = self.tree.push_leaf(&event.metadata.hash);
                            }
                            if self.vault.append(&event)? {
                                self.process_tick()?;
                            }
                        }
                        Err(_) => {
                            // Channel closed, flush and exit
                            self.process_tick()?;
                            break;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn process_tick(&mut self) -> Result<()> {
        if !self.tree.is_empty() {
            if let Some(root) = self.tree.root_hash() {
                let root_hex = hex::encode(root);
                println!("{:?} Ticker: Merkle Root Hash: {}", self.tick_interval, root_hex);
                // Ledger write
                let ledger_path = format!("{}/ledger.log", self.output_dir);
                if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(&ledger_path) {
                    use std::io::Write;
                    let _ = writeln!(file, "{},{}", chrono::Utc::now().to_rfc3339(), root_hex);
                }
            }
            self.tree.reset();
        }
        self.vault.flush()?;
        Ok(())
    }
}
