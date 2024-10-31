use me::api::get_receipt;
use rand::{seq::SliceRandom, thread_rng};
use reqwest::Proxy;
use utils::{
    constants::{ADDRESSES_FILE_PATH, NOT_CLAIMED_FILE_PATH, PROXIES_FILE_PATH},
    files::read_file_lines,
    logger::init_default_logger,
};

mod me;
mod utils;

use futures::stream::{self, StreamExt};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let _guard = init_default_logger();

    let proxies = read_file_lines(PROXIES_FILE_PATH).await?;
    let addresses = read_file_lines(ADDRESSES_FILE_PATH).await?;

    let eligible = Arc::new(AtomicUsize::new(0));
    let already_claimed = Arc::new(AtomicUsize::new(0));
    let addresses = Arc::new(addresses);
    let proxies = Arc::new(proxies);

    let (tx, mut rx) = mpsc::channel::<String>(100);

    let write_handle = tokio::spawn(async move {
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(NOT_CLAIMED_FILE_PATH)
            .await?;

        while let Some(line) = rx.recv().await {
            file.write_all(line.as_bytes()).await?;
            file.write_all(b"\n").await?;
        }
        Ok::<(), eyre::Report>(())
    });

    let concurrency_limit = 100; // Adjust the concurrency limit as needed

    stream::iter(addresses.iter())
        .for_each_concurrent(concurrency_limit, |address| {
            let proxies = proxies.clone();
            let eligible = eligible.clone();
            let already_claimed = already_claimed.clone();
            let tx = tx.clone();

            async move {
                let address = address.clone();
                let mut rng = thread_rng();

                let mut attempt_count = 0;
                let mut success = false;

                while !success && (proxies.is_empty() || attempt_count < proxies.len()) {
                    let proxy = if proxies.is_empty() {
                        None
                    } else {
                        proxies.choose(&mut rng).map(|random_proxy| {
                            Proxy::all(random_proxy).expect("Proxy to be valid")
                        })
                    };

                    tracing::info!("Start checking: {}", address);

                    match get_receipt(&address, proxy.as_ref()).await {
                        Ok(receipt) => {
                            let response = receipt.first().unwrap();

                            if let Some(err) = &response.error {
                                if err.json.code == -32600 {
                                    already_claimed.fetch_add(1, Ordering::SeqCst);
                                    eligible.fetch_add(1, Ordering::SeqCst);
                                    success = true;
                                } else {
                                    success = true;
                                }
                            } else {
                                if let Err(e) = tx.send(address.clone()).await {
                                    tracing::error!("Failed to send address to writer: {}", e);
                                }
                                eligible.fetch_add(1, Ordering::SeqCst);
                                success = true;
                            }
                        }
                        Err(e) => {
                            tracing::error!("Request failed: {}. Retrying with a new proxy.", e);
                            attempt_count += 1;
                        }
                    }
                }
            }
        })
        .await;

    drop(tx);

    write_handle.await??;

    let total = addresses.len();
    let eligible_count = eligible.load(Ordering::SeqCst);
    let already_claimed_count = already_claimed.load(Ordering::SeqCst);
    let success_rate = (eligible_count as f64 / total as f64) * 100.0;
    tracing::info!(
        "Eligible: {}/{} - {:.2}%",
        eligible_count,
        total,
        success_rate
    );
    tracing::info!("Already claimed: {}", already_claimed_count);

    Ok(())
}
