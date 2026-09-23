use flume::{Receiver, Sender};
use prism_common::RawEvent;
use crate::common::IngestConfig;

pub struct Dispatcher {
    data_tx: Sender<RawEvent>,
    data_rx: Receiver<RawEvent>,
    
    provenance_tx: Sender<RawEvent>,
    provenance_rx: Receiver<RawEvent>,
}

impl Dispatcher {
    pub fn new(config: &IngestConfig) -> Self {
        let (data_tx, data_rx) = flume::bounded(config.channel_capacity);
        let (provenance_tx, provenance_rx) = flume::bounded(config.channel_capacity);
        
        Self { 
            data_tx, data_rx,
            provenance_tx, provenance_rx
        }
    }

    pub fn sender(&self) -> DispatcherSender {
        DispatcherSender {
            data_tx: self.data_tx.clone(),
            provenance_tx: self.provenance_tx.clone(),
        }
    }

    pub fn data_receiver(&self) -> Receiver<RawEvent> {
        self.data_rx.clone()
    }

    pub fn provenance_receiver(&self) -> Receiver<RawEvent> {
        self.provenance_rx.clone()
    }
}

#[derive(Clone)]
pub struct DispatcherSender {
    data_tx: Sender<RawEvent>,
    provenance_tx: Sender<RawEvent>,
}

impl DispatcherSender {
    pub async fn broadcast(&self, event: RawEvent) -> Result<(), flume::SendError<RawEvent>> {
        // Clone for the second channel. `Bytes` clone is cheap (O(1)).
        let event_clone = event.clone();
        
        let data_res = self.data_tx.send_async(event).await;
        let prov_res = self.provenance_tx.send_async(event_clone).await;
        
        data_res.and(prov_res)
    }

    pub fn try_broadcast(&self, event: RawEvent) -> Result<(), flume::TrySendError<RawEvent>> {
        let event_clone = event.clone();
        
        let data_res = self.data_tx.try_send(event);
        let prov_res = self.provenance_tx.try_send(event_clone);
        
        data_res.and(prov_res)
    }
}
