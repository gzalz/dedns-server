use log::{info, trace};
use tokio::sync::watch::Receiver;
use std::time::Instant;
use std::sync::Arc;
use vdf::{PietrzakVDFParams, VDFParams, VDF};

const N: u64 = 10;
const NUM_BITS: u16 = 512;
const DIFFICULTY: u64 = 1048576;

pub async fn start(shutdown_rx: Arc<Receiver<()>>) {
    let mut sum: f64 = 0.0;
    let mut count: u64 = 0;
    loop {
        let pietrzak_vdf = PietrzakVDFParams(NUM_BITS).new();
        let input = b"\xaa";
        let start = Instant::now();
        let _solution = pietrzak_vdf.solve(input, DIFFICULTY).unwrap();
        let duration = start.elapsed();
        trace!("Produced a new uptime proof in {:?} seconds", duration);
        if shutdown_rx.has_changed().unwrap_or(true) {
            break;
        }
        sum += duration.as_secs_f64();
        count += 1;
        if count % N == 0 {
            info!("{:.2}s average proof generation time over the last {} proofs", sum/(count as f64), N);
            sum = 0.0;
            count = 0;
        }
    }
}
