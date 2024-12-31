use borsh::{BorshDeserialize, BorshSerialize};
use crate::channels::{SOLANA_TO_DNS, SOLANA_TO_DB};
use crate::models::*;
use serde::Deserialize;
use futures_util::{future, pin_mut};
use futures_util::{SinkExt, StreamExt};
use tracing::{trace, info, error};
use tokio::time::sleep;
use tokio::sync::watch::Receiver;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use solana_program::pubkey::Pubkey;
use std::sync::Arc;
use tungstenite::protocol::frame::Payload::Vec as FrameVec;

const RPC_URL: &str = "wss://devnet.helius-rpc.com/?api-key=32051878-0678-4d69-ba30-9a0370d30f7a";

pub async fn subscribe() {
    info!("Subscribing to RPC websocket...");
    let (mut ws_stream, _) = connect_async(RPC_URL).await.expect("Failed to connect");
    info!("WebSocket handshake has been successfully completed");

    ws_stream.send(Message::Text(
        String::from("{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"programSubscribe\",\"params\": [\"CUY6GfKAdtnyVFCmQz8NuadKkn25KxTTgxqcktRRy89m\",{\"encoding\": \"base64\", \"commitment\": \"finalized\"}]}").into()
    )).await.unwrap();

   info!("Sent subscription request");

    let (mut write, read) = ws_stream.split();

    let ws_to_stdout = {
        read.for_each(|message| async {
            if !message.is_ok() {
                error!("{:?}", message);
                return;
            }
            let message = message.unwrap();
            let data = message.to_text();
            let unwrapped_data = data.unwrap();
            trace!("Received message: {:?}", unwrapped_data);
            let parsed = serde_json::from_str(unwrapped_data);
            if parsed.is_err() {
                return;
            }
            let program_update: crate::models::Payload = parsed.unwrap();
            let account_data = &program_update.params.result.value.account.data[0];
            trace!("Extracted account data: {:?}", account_data);
            let mut decoded_data = &mut base64::decode(account_data).expect("Failed to decode Base64");
            trace!("Decoded account data: {:?}", decoded_data);
            if decoded_data.first() == Some(&1u8) {
                let zone = Zone::deserialize(&mut &decoded_data[1..]).expect("Failed to deserialize Zone");
                info!("{:?}", zone);
            }
            if decoded_data.first() == Some(&2u8) {
                let lease = Lease::deserialize(&mut &decoded_data[1..]).expect("Failed to deserialize Lease");
                info!("{:?}", lease);
            }
            if decoded_data.first() == Some(&3u8) {
                let record = Record::deserialize(&mut &decoded_data[1..]).expect("Failed to deserialize Record");
                let dns_service_sender = &SOLANA_TO_DNS.0.lock().await;
                let db_service_sender = &SOLANA_TO_DB.0.lock().await;
                dns_service_sender.send(record.clone()).await.unwrap();
                db_service_sender.send(record.clone()).await.unwrap();
                info!("{:?}", record);
            }
        })
    };
    let keepalive_loop = async move {
        loop {
            sleep(std::time::Duration::from_secs(15)).await;
            write.send(Message::Text(String::from("keepalive").into())).await.unwrap();
            trace!("Sent keepalive ping");
        }
    };
    tokio::join!(keepalive_loop, ws_to_stdout);
}
