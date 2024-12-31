use borsh::{BorshDeserialize, BorshSerialize};
use solana_pubkey::Pubkey;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Payload {
    pub params: Params,
}

#[derive(Deserialize)]
pub struct Params {
    pub result: ResultField,
}

#[derive(Deserialize)]
pub struct ResultField {
    pub value: ValueField,
}

#[derive(Deserialize)]
pub struct ValueField {
    pub account: Account,
}

#[derive(Deserialize)]
pub struct Account {
    pub data: Vec<String>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Zone {
    pub owner: Pubkey,
    pub lamports_per_second: i64,
    pub min_lease_duration_secs: i64,
    pub domain: String,
    pub subdivided: bool,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Lease {
    pub zone_account: Pubkey,
    pub owner: Pubkey,
    pub domain: String,
    pub expiration: i64,
    pub expired: bool,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug)]
pub struct Record {
    pub host: String,
    pub ttl: i64,
    pub record_type: String,
    pub value: String,
}
