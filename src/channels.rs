use crate::models::Record;
use tokio::sync::Mutex;
use std::sync::Arc;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use once_cell::sync::Lazy;

// Invoked upon receiving a new record from the Solana client
pub static SOLANA_TO_DB: Lazy<(Arc<Mutex<Sender<Record>>>, Arc<Mutex<Receiver<Record>>>)> = Lazy::new(|| {
    let (send, recv) = channel(255);
    (Arc::new(Mutex::new(send)), Arc::new(Mutex::new(recv)))
});

pub static SOLANA_TO_DNS: Lazy<(Arc<Mutex<Sender<Record>>>, Arc<Mutex<Receiver<Record>>>)> = Lazy::new(|| {
    let (send, recv) = channel(255);
    (Arc::new(Mutex::new(send)), Arc::new(Mutex::new(recv)))
});

// Stream interated records to dns server on bootup
pub static DB_TO_DNS: Lazy<(Arc<Mutex<Sender<Record>>>, Arc<Mutex<Receiver<Record>>>)> = Lazy::new(|| {
    let (send, recv) = channel(255);
    (Arc::new(Mutex::new(send)), Arc::new(Mutex::new(recv)))
});
