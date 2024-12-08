use std::sync::Arc;

use reqwest::{cookie::Jar, Proxy};
use solana_sdk::signature::Keypair;
use tokio::{fs::OpenOptions, sync::Mutex, task::JoinSet};

use crate::{
    config::Config,
    crypto::signer::{get_address, get_wallet},
    utils::{
        constants::{ELIGIBLE_FILE_PATH, PROXIES_FILE_PATH, SECRETS_FILE_PATH},
        files::read_file_lines,
    },
};

use super::processor::{create_session, points};

pub async fn checker(config: &Config) -> eyre::Result<()> {
    let proxies: Vec<Proxy> = read_file_lines(PROXIES_FILE_PATH)
        .await?
        .iter()
        .map(|proxy_url| Proxy::all(proxy_url).expect("Invalid proxy URL"))
        .collect();

    let proxies = Arc::new(proxies);
    let proxies_len = proxies.len();

    let all_wallets = read_file_lines(SECRETS_FILE_PATH).await?;

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(ELIGIBLE_FILE_PATH)
        .await
        .expect("Failed to open file for writing");
    let eligible_file = Arc::new(Mutex::new(file));

    let mut join_set = JoinSet::new();

    for (index, secret) in all_wallets.into_iter().enumerate() {
        let proxies = Arc::clone(&proxies);
        let eligible_file = Arc::clone(&eligible_file);

        join_set.spawn(async move {
            let random_wallet = Keypair::new();
            let random_address = get_address(&random_wallet);

            let cookie_jar = Arc::new(Jar::default());
            let proxy = proxies[index % proxies_len].clone();

            if let Err(e) =
                create_session(&random_wallet, &random_address, Some(&proxy), &cookie_jar).await
            {
                tracing::error!("{e}");
                return;
            };

            let wallet = match get_wallet(&secret) {
                Ok(wallet) => wallet,
                Err(e) => {
                    tracing::error!("{e}");
                    return;
                }
            };
            let address = get_address(&wallet);

            if let Err(e) = points(
                &wallet,
                &random_address,
                &address,
                Some(&proxy),
                &cookie_jar,
                &eligible_file,
            )
            .await
            {
                tracing::error!("{e}");
            };
        });

        if join_set.len() >= config.parallelism {
            if let Some(Err(e)) = join_set.join_next().await {
                tracing::error!("Task failed: {}", e);
            }
        }
    }

    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(_) => {}
            Err(e) => tracing::error!("Task failed: {}", e),
        }
    }

    tracing::info!("Finished! Eligible wallets are in data/eligible.txt");

    Ok(())
}
