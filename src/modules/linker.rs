use std::sync::Arc;

use reqwest::{cookie::Jar, Proxy};
use tokio::task::JoinSet;

use crate::{
    config::Config,
    crypto::signer::{get_address, get_wallet},
    utils::{
        constants::{PROXIES_FILE_PATH, SECRETS_FILE_PATH},
        files::read_file_lines,
    },
};

use super::processor::{create_session, link_wallet};

pub async fn linker(config: &Config) -> eyre::Result<()> {
    let proxies: Vec<Proxy> = read_file_lines(PROXIES_FILE_PATH)
        .await?
        .iter()
        .map(|proxy_url| Proxy::all(proxy_url).expect("Invalid proxy URL"))
        .collect();

    let proxies = Arc::new(proxies);
    let proxies_len = proxies.len();

    let main_wallet = get_wallet(&config.claim_wallet_secret).expect("Invalid main wallet secret");
    let main_address = Arc::new(get_address(&main_wallet));

    let cookie_jar = Arc::new(Jar::default());

    create_session(&main_wallet, &main_address, proxies.first(), &cookie_jar).await?;

    let all_wallets = read_file_lines(SECRETS_FILE_PATH).await?;

    let mut join_set = JoinSet::new();

    for (index, secret) in all_wallets.into_iter().enumerate() {
        let proxies = Arc::clone(&proxies);
        let cookie_jar = Arc::clone(&cookie_jar);
        let main_address = Arc::clone(&main_address);

        join_set.spawn(async move {
            let proxy = proxies[index % proxies_len].clone();

            let wallet = match get_wallet(&secret) {
                Ok(wallet) => wallet,
                Err(e) => {
                    tracing::error!("{e}");
                    return;
                }
            };
            let address = get_address(&wallet);

            if let Err(e) =
                link_wallet(&wallet, &main_address, &address, Some(&proxy), &cookie_jar).await
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

    tracing::info!("Finished! Successfully linked wallets to: {main_address}");

    Ok(())
}
