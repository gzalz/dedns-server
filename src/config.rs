use anyhow::Result;
use dotenv::dotenv;
use log::info;
use std::env;

pub fn init() -> Result<()> {
    dotenv().ok();
    info!("Configured environment variables from .env");
    for (key, value) in env::vars() {
        if key.starts_with("DEDNS") {
            info!("{}\t= {}", key, value);
        }
    }
    Ok(())
}
