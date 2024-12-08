use std::sync::Arc;

use reqwest::{cookie::Jar, Proxy};
use tokio::task::JoinSet;

use crate::{
    config::Config,
    crypto::signer::{get_address, get_wallet},
    utils::{
        constants::{CLAIM_SECRETS_FILE_PATH, PROXIES_FILE_PATH, SECRETS_FILE_PATH},
        files::read_file_lines,
    },
};

use super::processor::{create_session, link_wallet};

async fn process_wallet(
    secret: String,
    claim_wallets: Arc<Vec<String>>,
    proxies: Arc<Vec<Proxy>>,
    proxies_len: usize,
    index: usize,
) -> eyre::Result<(), (usize, String)> {
    let main_wallet = get_wallet(&claim_wallets[index]).expect("Invalid main wallet secret");
    let main_address = Arc::new(get_address(&main_wallet));

    let cookie_jar = Arc::new(Jar::default());
    let proxy = proxies[index % proxies_len].clone();

    if let Err(e) = create_session(&main_wallet, &main_address, proxies.first(), &cookie_jar).await
    {
        tracing::error!("{e}");
        return Err((index, secret));
    };

    let wallet = match get_wallet(&secret) {
        Ok(wallet) => wallet,
        Err(e) => {
            tracing::error!("{e}");
            return Err((index, secret));
        }
    };
    let address = get_address(&wallet);

    if let Err(e) = link_wallet(&wallet, &main_address, &address, Some(&proxy), &cookie_jar).await {
        tracing::error!("{e}");
        return Err((index, secret));
    };

    tracing::info!("Wallet {address} linked to {main_address}");

    Ok(())
}

async fn process_wallet_with_retries(
    mut secret: String,
    claim_wallets: Arc<Vec<String>>,
    proxies: Arc<Vec<Proxy>>,
    proxies_len: usize,
    index: usize,
) {
    loop {
        match process_wallet(
            secret.clone(),
            claim_wallets.clone(),
            proxies.clone(),
            proxies_len,
            index,
        )
        .await
        {
            Ok(()) => break,
            Err((failed_index, failed_secret)) => {
                tracing::warn!("Retrying wallet at index {failed_index}");
                secret = failed_secret;
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    }
}

pub async fn linker(config: &Config) -> eyre::Result<()> {
    let proxies: Vec<Proxy> = read_file_lines(PROXIES_FILE_PATH)
        .await?
        .iter()
        .map(|proxy_url| Proxy::all(proxy_url).expect("Invalid proxy URL"))
        .collect();

    let proxies = Arc::new(proxies);
    let proxies_len = proxies.len();

    let claim_wallets = Arc::new(read_file_lines(CLAIM_SECRETS_FILE_PATH).await?);
    let all_wallets = read_file_lines(SECRETS_FILE_PATH).await?;

    if claim_wallets.len() != all_wallets.len() {
        tracing::warn!("Number of claim wallets (claim_secrets.txt) not equals to airdrop wallets (secrets.txt)");
        return Ok(());
    }

    let mut join_set = JoinSet::new();

    for (index, secret) in all_wallets.into_iter().enumerate() {
        let proxies = Arc::clone(&proxies);
        let claim_wallets = Arc::clone(&claim_wallets);

        join_set.spawn(async move {
            process_wallet_with_retries(secret, claim_wallets, proxies, proxies_len, index).await;
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

    tracing::info!("Finished! Successfully linked wallets");

    Ok(())
}
