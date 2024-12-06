mod config;
mod crypto;
mod me;
mod utils;

use config::Config;

use crypto::{
    messages::{get_link_wallet_message, get_verify_message},
    signer::{get_address, get_wallet, sign_message},
};

use me::{
    api::{auth_link_wallet, auth_session, verify_and_create_session, wallets},
    utils::extract_allocation_amount,
};

use solana_sdk::signature::Keypair;
use utils::{
    constants::{ELIGIBLE_FILE_PATH, PROXIES_FILE_PATH, SECRETS_FILE_PATH},
    files::read_file_lines,
    logger::init_default_logger,
};

use reqwest::{cookie::Jar, Proxy};

use tokio::{fs::OpenOptions, io::AsyncWriteExt};

use futures::stream::{self, StreamExt};
use std::sync::Arc;
use uuid::Uuid;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let _guard = init_default_logger();

    let config = Config::read_default().await;

    let proxies: Vec<Proxy> = read_file_lines(PROXIES_FILE_PATH)
        .await?
        .iter()
        .map(|proxy_url| Proxy::all(proxy_url).expect("Invalid proxy URL"))
        .collect();

    let secrets = Arc::new(read_file_lines(SECRETS_FILE_PATH).await?);

    let all_wallets: Vec<String> = {
        let mut wallets = Vec::new();
        wallets.extend(secrets.iter().map(|key| key.to_owned()));
        wallets
    };

    let batched_wallets: Vec<Vec<String>> = all_wallets
        .chunks(config.batch_size)
        .map(|chunk| chunk.to_vec())
        .collect();

    let cookie_jar = Arc::new(Jar::default());

    let proxies = Arc::new(proxies);

    stream::iter(batched_wallets)
        .for_each_concurrent(config.parallelism, |batch| {
            let proxies = Arc::clone(&proxies);
            let cookie_jar = Arc::clone(&cookie_jar);

            let random_wallet = Keypair::new();
            let main_address = get_address(&random_wallet);

            async move {
                let mut eligible_file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(ELIGIBLE_FILE_PATH)
                    .await
                    .expect("Failed to open file for writing");

                for (index, secret) in batch.into_iter().enumerate() {
                    let uuid = Uuid::new_v4().to_string();

                    let proxy = proxies[index % proxies.len()].clone();

                    let wallet = match get_wallet(&secret) {
                        Ok(wallet) => wallet,
                        Err(e) => {
                            tracing::error!("{e}");
                            continue;
                        }
                    };

                    let address = get_address(&wallet);

                    if auth_session(&uuid, Some(&proxy), Some(Arc::clone(&cookie_jar)))
                        .await
                        .is_err()
                    {
                        tracing::error!("Auth session failed for address {}", address);
                        continue;
                    }

                    let verify_message = get_verify_message(&uuid);

                    let verify_signature = sign_message(&random_wallet, &verify_message)
                        .expect("Failed to sign verify message");

                    if let Ok(Some(verify_and_create_response)) = verify_and_create_session(
                        &main_address,
                        &verify_signature,
                        &verify_message,
                        Some(&proxy),
                        Some(Arc::clone(&cookie_jar)),
                    )
                    .await
                    {
                        if !verify_and_create_response.success {
                            tracing::error!(
                                "Verify and create session failed for address {}",
                                address
                            );
                            continue;
                        }
                    } else {
                        tracing::error!("Verify and create session failed for address {}", address);
                        continue;
                    }

                    if auth_session(&uuid, Some(&proxy), Some(Arc::clone(&cookie_jar)))
                        .await
                        .is_err()
                    {
                        tracing::error!("Second auth session failed for address {}", address);
                        continue;
                    }

                    let link_message = get_link_wallet_message(&main_address, &address);

                    let signature =
                        sign_message(&wallet, &link_message).expect("Failed to sign link message");

                    if let Ok(Some(response_items)) = auth_link_wallet(
                        &link_message,
                        &address,
                        &signature,
                        Some(&proxy),
                        Some(Arc::clone(&cookie_jar)),
                    )
                    .await
                    {
                        if let Some(response_item) =
                            response_items.first().and_then(|item| item.as_ref())
                        {
                            if let Some(result) = &response_item.result {
                                if let Some(data) = &result.data {
                                    if let Some(json) = &data.json {
                                        if let Some(eligibility) = &json.eligibility {
                                            if let Some(eligible) = &eligibility.eligibility {
                                                if eligible == "eligible" {
                                                    if let Ok(Some(allocation_response)) = wallets(
                                                        Some(&proxy),
                                                        Some(Arc::clone(&cookie_jar)),
                                                    )
                                                    .await
                                                    {
                                                        if let Some(amount) =
                                                            extract_allocation_amount(
                                                                &allocation_response,
                                                            )
                                                        {
                                                            let allocation =
                                                                amount as f64 / 10f64.powi(6);

                                                            let entry = format!(
                                                                "{}: {}\n",
                                                                address, allocation
                                                            );
                                                            eligible_file
                                                                .write_all(entry.as_bytes())
                                                                .await
                                                                .expect("Failed to write to file");
                                                        } else {
                                                            let entry = format!("{}\n", address);
                                                            eligible_file
                                                                .write_all(entry.as_bytes())
                                                                .await
                                                                .expect("Failed to write to file");
                                                        }
                                                    } else {
                                                        let entry = format!("{}\n", address);
                                                        eligible_file
                                                            .write_all(entry.as_bytes())
                                                            .await
                                                            .expect("Failed to write to file");
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        })
        .await;

    tracing::info!("Finished! Eligible wallets are in data/eligible.txt");

    Ok(())
}
