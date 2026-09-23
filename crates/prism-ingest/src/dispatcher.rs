use flume::{Receiver, Sender};
use prism_common::RawEvent;

pub struct Dispatcher {
    sender: Sender<RawEvent>,
    receiver: Receiver<RawEvent>,
}

impl Dispatcher {
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver) = flume::bounded(capacity);
        Self { sender, receiver }
    }

    pub fn sender(&self) -> Sender<RawEvent> {
        self.sender.clone()
    }

    pub fn receiver(&self) -> Receiver<RawEvent> {
        self.receiver.clone()
    }
}
