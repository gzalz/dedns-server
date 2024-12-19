use borsh::{BorshDeserialize, BorshSerialize};
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

const RPC_URL: &str = "wss://api.devnet.solana.com";

#[derive(Deserialize)]
struct Payload {
    params: Params,
}

#[derive(Deserialize)]
struct Params {
    result: ResultField,
}

#[derive(Deserialize)]
struct ResultField {
    value: ValueField,
}

#[derive(Deserialize)]
struct ValueField {
    account: Account,
}

#[derive(Deserialize)]
struct Account {
    data: Vec<String>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Zone {
    owner: Pubkey,
    lamports_per_second: i64,
    min_lease_duration_secs: i64,
    domain: String,
    subdivided: bool,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Lease {
    zone_account: Pubkey,
    owner: Pubkey,
    domain: String,
    expiration: i64,
    expired: bool,
}

pub async fn subscribe(shutdown_rx: Arc<Mutex<Receiver<()>>>) {
    info!("Subscribing to RPC websocket...");
    let (mut ws_stream, _) = connect_async(RPC_URL).await.expect("Failed to connect");
    info!("WebSocket handshake has been successfully completed");

    ws_stream.send(Message::Text(
        String::from("{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"programSubscribe\",\"params\": [\"CUY6GfKAdtnyVFCmQz8NuadKkn25KxTTgxqcktRRy89m\",{\"encoding\": \"base64\", \"commitment\": \"finalized\"}]}").into()
    )).await.unwrap();

   info!("Sent subscription request");

    let (write, read) = ws_stream.split();

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
            let program_update: Payload = parsed.unwrap();
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
        })
    };
    ws_to_stdout.await;
    loop {
        sleep(std::time::Duration::from_secs(15)).await;
        //ws_stream.send(Message::Ping(tungstenite::protocol::frame::Payload::Vec(vec![]))).await.unwrap();
        let mut shutdown_lock = shutdown_rx.lock().await;
        if shutdown_lock.changed().await.is_ok() {
            info!("Shutting down Solana websocket subscription...");
            break;
        }
    }
}
