use chrono::{SecondsFormat, Utc};

pub fn get_verify_message(uuid: &str) -> String {
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);

    format!(
        "URI: mefoundation.com\nChain ID: sol\nNonce: {}\nIssued At: {}",
        uuid, now
    )
}

pub fn get_link_wallet_message(claim_wallet: &str, target_wallet: &str) -> String {
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    format!(
            "URI: mefoundation.com\nIssued At: {}\nChain ID: sol\nAllocation Wallet: {}\nClaim Wallet: {}",
            now,
            target_wallet,
            claim_wallet
        )
}
