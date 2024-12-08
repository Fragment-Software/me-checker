use std::sync::Arc;

use reqwest::{cookie::Jar, Proxy};
use solana_sdk::signature::Keypair;
use tokio::{fs::File, io::AsyncWriteExt, sync::Mutex};
use uuid::Uuid;

use crate::{
    crypto::{
        messages::{get_link_wallet_message, get_verify_message},
        signer::sign_message,
    },
    me::{
        api::{auth_link_wallet, auth_session, verify_and_create_session, wallets},
        schemas::LinkWalletResponse,
        utils::extract_allocation_amount,
    },
};

pub async fn create_session(
    signer: &Keypair,
    signer_address: &str,
    proxy: Option<&Proxy>,
    cookie_jar: &Arc<Jar>,
) -> eyre::Result<()> {
    let uuid = Uuid::new_v4().to_string();

    if auth_session(&uuid, proxy, Some(cookie_jar.clone()))
        .await
        .is_err()
    {
        eyre::bail!("Auth session failed");
    }

    let verify_message = get_verify_message(&uuid);

    let verify_signature =
        sign_message(signer, &verify_message).expect("Failed to sign verify message");

    match verify_and_create_session(
        signer_address,
        &verify_signature,
        &verify_message,
        proxy,
        Some(cookie_jar.clone()),
    )
    .await
    {
        Ok(verify_and_create_response) => match verify_and_create_response {
            Some(response) => {
                if !response.success {
                    eyre::bail!("Verify and create session is not successful");
                }
            }
            None => eyre::bail!("Verify and create session failed"),
        },
        Err(e) => eyre::bail!("Verify and create session failed: {e}"),
    }

    if auth_session(&uuid, proxy, Some(cookie_jar.clone()))
        .await
        .is_err()
    {
        eyre::bail!("Second auth session failed");
    }

    Ok(())
}

pub async fn link_wallet(
    target_wallet: &Keypair,
    claim_address: &str,
    target_address: &str,
    proxy: Option<&Proxy>,
    cookie_jar: &Arc<Jar>,
) -> eyre::Result<Option<LinkWalletResponse>> {
    let link_message = get_link_wallet_message(claim_address, target_address);

    let signature =
        sign_message(target_wallet, &link_message).expect("Failed to sign link message");

    auth_link_wallet(
        &link_message,
        target_address,
        &signature,
        proxy,
        Some(Arc::clone(cookie_jar)),
    )
    .await
}

pub async fn points(
    target_wallet: &Keypair,
    claim_address: &str,
    target_address: &str,
    proxy: Option<&Proxy>,
    cookie_jar: &Arc<Jar>,
    eligible_file: &Arc<Mutex<File>>,
) -> eyre::Result<()> {
    if let Ok(Some(response_items)) = link_wallet(
        target_wallet,
        claim_address,
        target_address,
        proxy,
        cookie_jar,
    )
    .await
    {
        if let Some(response_item) = response_items.first().and_then(|item| item.as_ref()) {
            if let Some(result) = &response_item.result {
                if let Some(data) = &result.data {
                    if let Some(json) = &data.json {
                        if let Some(eligibility) = &json.eligibility {
                            if let Some(eligible) = &eligibility.eligibility {
                                if eligible == "eligible" {
                                    let wallets_result =
                                        wallets(proxy, Some(Arc::clone(cookie_jar))).await;

                                    let entry: String = if let Ok(Some(allocation_response)) =
                                        wallets_result
                                    {
                                        if let Some(amount) =
                                            extract_allocation_amount(&allocation_response)
                                        {
                                            if amount == 0 {
                                                format!("{}\n", target_address)
                                            } else {
                                                let allocation: f64 = amount as f64 / 10f64.powi(6);

                                                format!("{}: {}\n", target_address, allocation)
                                            }
                                        } else {
                                            format!("{}\n", target_address)
                                        }
                                    } else {
                                        format!("{}\n", target_address)
                                    };

                                    let mut f = eligible_file.lock().await;
                                    if let Err(e) = f.write_all(entry.as_bytes()).await {
                                        tracing::error!("Failed to write to file: {}", e);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
