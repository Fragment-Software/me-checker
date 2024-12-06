use solana_sdk::{signature::Keypair, signer::Signer as SolSigner};

pub fn get_wallet(private_key: &str) -> Keypair {
    Keypair::from_base58_string(private_key)
}

pub fn get_address(signer: &Keypair) -> String {
    signer.pubkey().to_string()
}

pub fn sign_message(signer: &Keypair, message: &str) -> eyre::Result<String> {
    let signature = signer.sign_message(message.as_bytes());

    Ok(signature.to_string())
}
