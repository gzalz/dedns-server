use crate::channels::{SOLANA_TO_DB, DB_TO_DNS};
use crate::models::Record;
use tracing::info;
use redb::{Database, Error, ReadableTable, TableDefinition};
use once_cell::sync::Lazy;
use std::path::Path;
use std::collections::HashMap;

const A_RECORD: TableDefinition<&str, Vec<u8>> = TableDefinition::new("A_RECORD");
const AAAA_RECORD: TableDefinition<&str, Vec<u8>> = TableDefinition::new("AAAA_RECORD");
const CNAME_RECORD: TableDefinition<&str, Vec<u8>> = TableDefinition::new("CNAME_RECORD");

pub async fn start(){
    let db_a_index = Database::open(Path::new("idx_a")).unwrap();
    let db_aaaa_index = Database::open(Path::new("idx_aaaa")).unwrap();
    let db_cname_index = Database::open(Path::new("idx_cname")).unwrap();
    let task = async {
        info!("Starting record indexer {:?}", ["A", "AAAA", "CNAME"]);
        {
            let read_txn = db_a_index.begin_read().unwrap();
            let mut table = read_txn.open_table(A_RECORD).unwrap();
            let mut iter = table.range("A".."z").unwrap();
            while let a = iter.next() {
                if a.is_none() {
                    break;
                }
                let r = a.unwrap();
                let record: Record = borsh::BorshDeserialize::try_from_slice(&r.unwrap().1.value()).unwrap();
                info!("Found record {:?}", record);
                let sender = &DB_TO_DNS.0.lock().await;
                sender.send(record).await.unwrap();
                info!("Sent record to DNS");
            }
        }
        loop {
            let mut receiver = &mut SOLANA_TO_DB.1.lock().await;
            let maybe_recv = receiver.try_recv();
            if maybe_recv.is_err() {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                continue;
            }
            let record = maybe_recv.unwrap();
            match record.record_type.as_str() {
                "A" => {
                    let write_txn = db_a_index.begin_write().unwrap();
                    {
                        let mut table = write_txn.open_table(A_RECORD).unwrap();
                        table.insert(&*record.host, borsh::to_vec(&record).unwrap()).unwrap();
                    
                        let mut fetched = table.get(&*record.host).unwrap().unwrap().value();
                        // lol move me to a unit test
                        let inserted_record: Record = borsh::BorshDeserialize::try_from_slice(&fetched).unwrap();
                        info!("Persisted record {:?}", inserted_record);
                    }
                    write_txn.commit().unwrap();
                    let db_to_dns_sender = DB_TO_DNS.0.lock().await;
                    db_to_dns_sender.send(record).await.unwrap();
                }
                "AAAA" => {
                }
                "CNAME" => {
                }
                _ => {
                    info!("Unknown record type {:?}", record.record_type);
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    };
    task.await;
}
