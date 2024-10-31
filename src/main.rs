use me::api::get_receipt;
use rand::{seq::SliceRandom, thread_rng};
use reqwest::Proxy;
use utils::{
    constants::{ADDRESSES_FILE_PATH, PROXIES_FILE_PATH},
    files::read_file_lines,
    logger::init_default_logger,
};

mod me;
mod utils;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let _guard = init_default_logger();

    let proxies = read_file_lines(PROXIES_FILE_PATH).await?;
    let addresses = read_file_lines(ADDRESSES_FILE_PATH).await?;

    let mut eligible = 0;
    let mut already_claimed = 0;
    let mut rng = thread_rng();

    for address in &addresses {
        let mut attempt_count = 0;
        let mut success = false;

        while !success && (proxies.is_empty() || attempt_count < proxies.len()) {
            let proxy = if proxies.is_empty() {
                None
            } else {
                proxies
                    .choose(&mut rng)
                    .map(|random_proxy| Proxy::all(random_proxy).expect("Proxy to be valid"))
            };

            tracing::info!("Start checking: {}", address);

            match get_receipt(address, proxy.as_ref()).await {
                Ok(receipt) => {
                    let response = receipt.first().unwrap();

                    if let Some(err) = &response.error {
                        if err.json.code == -32600 {
                            already_claimed += 1;
                            eligible += 1;
                            success = true;
                        } else {
                            success = true;
                        }
                    } else {
                        eligible += 1;
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

    let total = addresses.len();
    let success_rate = (eligible as f64 / total as f64) * 100.0;
    tracing::info!("Eligible: {}/{} - {:.2}%", eligible, total, success_rate);
    tracing::info!("Already claimed: {}", already_claimed);

    Ok(())
}
