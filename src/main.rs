use anyhow::Result;
use tracing::info;
use tokio::signal;
use tokio::sync::{watch, Mutex};
use std::sync::Arc;

fn ascii_art() -> Result<()> {
    info!("\t     █████          ██████████   ██████   █████  █████████ ");
    info!("\t    ░░███          ░░███░░░░███ ░░██████ ░░███  ███░░░░░███");
    info!("\t  ███████   ██████  ░███   ░░███ ░███░███ ░███ ░███    ░░░ ");
    info!("\t ███░░███  ███░░███ ░███    ░███ ░███░░███░███ ░░█████████ ");
    info!("\t░███ ░███ ░███████  ░███    ░███ ░███ ░░██████  ░░░░░░░░███");
    info!("\t░███ ░███ ░███░░░   ░███    ███  ░███  ░░█████  ███    ░███");
    info!("\t░░████████░░██████  ██████████   █████  ░░█████░░█████████ ");
    info!("\t ░░░░░░░░  ░░░░░░  ░░░░░░░░░░   ░░░░░    ░░░░░  ░░░░░░░░░  "); 
    Ok(())
}
async fn bootstrap() -> Result<()> {
    let _ = ascii_art();
    let (dns_shutdown_tx, dns_shutdown_rx) = watch::channel(());
    let (solana_shutdown_tx, solana_shutdown_rx) = watch::channel(());
    //let (uptimeproof_shutdown_tx, uptimeproof_shutdown_rx) = watch::channel(());

    dedns::config::init()?;

    tokio::spawn(async move {
        _ = dedns::dns_service::start(Arc::new(dns_shutdown_rx)).await;
    });

    tokio::spawn(async move {
        _ = dedns::solana_service::subscribe(Arc::new(Mutex::new(solana_shutdown_rx))).await;
    });
    
    /*tokio::spawn(async move {
        _ = dedns::proofofuptime::start(Arc::new(uptimeproof_shutdown_rx)).await;
    });*/

    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("Received Ctrl+C, initiating shutdown...");
            let _ = dns_shutdown_tx.send(());
            let _ = solana_shutdown_tx.send(());
            //let _ = uptimeproof_shutdown_tx.send(());
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // if RUST_LOG is not set, set it to info
    if std::env::var("RUST_LOG").is_err() {
        unsafe { std::env::set_var("RUST_LOG", "info"); }
    }
    tracing_subscriber::fmt::init();
    bootstrap().await?;
    Ok(())
}


