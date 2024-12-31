use crate::channels::DB_TO_DNS;
use crate::models::Record;
use crate::channels::{SOLANA_TO_DNS};
use dashmap::DashMap;
use dns_server::{DnsQuestion, DnsRecord, DnsName, DnsType, DnsClass};
use permit::Permit;
use signal_hook::consts::{SIGHUP, SIGINT, SIGQUIT, SIGTERM};
use signal_hook::iterator::Signals;
use tracing::info;
use once_cell::sync::Lazy;
use tokio::time::sleep;
use tokio::sync::Mutex;
use tokio::sync::mpsc::Receiver;
use std::sync::Arc;

type RecordStore = DashMap<DnsName, DnsRecord>;

static A_RECORDS: Lazy<RecordStore> = Lazy::new(|| DashMap::new());
static AAAA_RECORDS: Lazy<RecordStore> = Lazy::new(|| DashMap::new());
static CNAME_RECORDS: Lazy<RecordStore> = Lazy::new(|| DashMap::new());

pub fn resolve(query: &DnsQuestion) -> Vec<DnsRecord> {
    let mut records = Vec::new();
    match query.typ {
        DnsType::A => {
            let a_records = A_RECORDS.get(&query.name);
            if a_records.is_some() {
                records.push(a_records.unwrap().clone());
            }
        },
        _ => {},
    }
    records;
    vec![DnsRecord::A(query.name.clone(), std::net::Ipv4Addr::new(127, 0, 0, 1))]
}

pub async fn start() {
    info!("Starting DNS server on port 53");
    
    let dns_service = async move {
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
        loop {}
    };

    let record_recv = async {
        let mut recv_lock: &mut tokio::sync::MutexGuard<Receiver<Record>> = &mut SOLANA_TO_DNS.1.lock().await;
        loop {
            let record = recv_lock.recv().await;
            if record.is_some() {
                let record = record.unwrap();
                info!("New DNS record: {:?}", record);
                let dns_name = DnsName::new(&record.host.clone()).unwrap();
                let ipv4_addr: std::net::Ipv4Addr = record.value.parse().unwrap();
                let dns_record = DnsRecord::A(dns_name.clone(), ipv4_addr);
                A_RECORDS.insert(dns_name, dns_record);
                info!("Inserted record. A_RECORDS size: {}", A_RECORDS.len());
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    };

    let persisted_record_recv = async move {
        let mut recv_lock: &mut tokio::sync::MutexGuard<Receiver<Record>> = &mut DB_TO_DNS.1.lock().await;
        loop {
            let record = recv_lock.try_recv();
            if record.is_ok() {
                let record = record.unwrap();
                info!("New persisted DNS record: {:?}", record);
                let dns_name = DnsName::new(&record.host.clone()).unwrap();
                let ipv4_addr: std::net::Ipv4Addr = record.value.parse().unwrap();
                let dns_record = DnsRecord::A(dns_name.clone(), ipv4_addr);
                A_RECORDS.insert(dns_name, dns_record);
                info!("Inserted record. A_RECORDS size: {}", A_RECORDS.len());
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
        ()
    };

    // TODO: add record recv back
    tokio::join!(persisted_record_recv, record_recv, dns_service);
}
