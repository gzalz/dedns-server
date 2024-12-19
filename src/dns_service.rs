use dns_server::{DnsQuestion, DnsRecord, DnsName, DnsType, DnsClass};
use permit::Permit;
use signal_hook::consts::{SIGHUP, SIGINT, SIGQUIT, SIGTERM};
use signal_hook::iterator::Signals;
use tracing::info;
use tokio::time::sleep;
use tokio::sync::watch::Receiver;
use std::sync::Arc;

pub fn resolve(q: &DnsQuestion) -> Vec<DnsRecord> {
    Vec::new()
}

pub async fn start(shutdown_rx: Arc<Receiver<()>>) {
    info!("Starting DNS server on port 53");
    let top_permit = Permit::new();
    let permit = top_permit.new_sub();
    std::thread::spawn(move || {
        Signals::new([SIGHUP, SIGINT, SIGQUIT, SIGTERM])
        .unwrap()
        .forever()
        .next();
    drop(top_permit);
    });
    dns_server::Builder::new_port(8053)
        .unwrap()
        .with_permit(permit)
        .serve(&resolve)
        .unwrap();
}
