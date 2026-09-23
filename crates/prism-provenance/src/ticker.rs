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
}

impl IntegrityTicker {
    pub fn new(rx: Receiver<RawEvent>, output_dir: &str, batch_size: usize, tick_interval: Duration) -> Result<Self> {
        let vault = VaultWriter::new(output_dir, batch_size)?;
        Ok(Self {
            rx,
            tree: ProvenanceTree::new(),
            vault,
            tick_interval,
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
                            self.tree.push_leaf(&event.metadata.hash);
                            self.vault.append(&event)?;
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
                println!("60s Ticker: Merkle Root Hash: {}", root_hex);
                // Ledger write
                if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open("ledger.log") {
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
