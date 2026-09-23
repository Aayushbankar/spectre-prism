use flume::{Receiver, Sender};
use prism_common::RawEvent;
use crate::common::IngestConfig;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub struct Dispatcher {
    data_tx: Sender<RawEvent>,
    data_rx: Receiver<RawEvent>,
    
    provenance_tx: Sender<RawEvent>,
    provenance_rx: Receiver<RawEvent>,
    
    drop_count: Arc<AtomicU64>,
}

impl Dispatcher {
    pub fn new(config: &IngestConfig) -> Self {
        let (data_tx, data_rx) = flume::bounded(config.channel_capacity);
        let (provenance_tx, provenance_rx) = flume::bounded(config.channel_capacity);
        
        Self { 
            data_tx, data_rx,
            provenance_tx, provenance_rx,
            drop_count: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn sender(&self) -> DispatcherSender {
        DispatcherSender {
            data_tx: self.data_tx.clone(),
            provenance_tx: self.provenance_tx.clone(),
            drop_count: self.drop_count.clone(),
        }
    }

    pub fn data_receiver(&self) -> Receiver<RawEvent> {
        self.data_rx.clone()
    }

    pub fn provenance_receiver(&self) -> Receiver<RawEvent> {
        self.provenance_rx.clone()
    }
    
    pub fn drop_count(&self) -> Arc<AtomicU64> {
        self.drop_count.clone()
    }
}

#[derive(Clone)]
pub struct DispatcherSender {
    data_tx: Sender<RawEvent>,
    provenance_tx: Sender<RawEvent>,
    drop_count: Arc<AtomicU64>,
}

#[derive(Debug)]
pub struct BroadcastError;

impl DispatcherSender {
    pub fn try_broadcast(&self, event: RawEvent) -> Result<(), BroadcastError> {
        if self.data_tx.is_full() || self.provenance_tx.is_full() {
            self.drop_count.fetch_add(1, Ordering::Relaxed);
            return Err(BroadcastError);
        }
        
        let event_clone = event.clone();
        
        if self.data_tx.try_send(event).is_err() {
            self.drop_count.fetch_add(1, Ordering::Relaxed);
            return Err(BroadcastError);
        }
        if self.provenance_tx.try_send(event_clone).is_err() {
            self.drop_count.fetch_add(1, Ordering::Relaxed);
            return Err(BroadcastError);
        }
        
        Ok(())
    }
}
