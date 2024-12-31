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

    dedns::config::init()?;

    tokio::spawn(async move {
        _ = dedns::dns_service::start().await;
    });

    tokio::spawn(async move {
        _ = dedns::solana_service::subscribe().await;
    });

    tokio::spawn(async move {
        _ = dedns::record_index_service::start().await;
    });
    
    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("Received Ctrl+C, initiating shutdown...");
            std::process::exit(0);
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        unsafe { std::env::set_var("RUST_LOG", "info"); }
    }
    tracing_subscriber::fmt::init();
    bootstrap().await?;
    Ok(())
}


