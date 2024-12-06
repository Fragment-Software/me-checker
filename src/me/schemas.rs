#![allow(dead_code)]
#![allow(unused_variables)]
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Metadata<'a> {
    platform: &'a str,
    #[serde(rename = "patchVersion")]
    patch_version: u8,
    #[serde(rename = "minorVersion")]
    minor_version: u8,
    #[serde(rename = "majorVersion")]
    major_version: u8,
}

impl<'a> Default for Metadata<'a> {
    fn default() -> Self {
        Self {
            platform: "ios",
            patch_version: 0,
            minor_version: 30,
            major_version: 2,
        }
    }
}

#[derive(Serialize)]
pub struct VerifyAndCreateSessionBody<'a> {
    wallet: &'a str,
    signature: &'a str,
    message: &'a str,
    metadata: Metadata<'a>,
}

impl<'a> VerifyAndCreateSessionBody<'a> {
    pub fn new(wallet: &'a str, signature: &'a str, message: &'a str) -> Self {
        Self {
            wallet,
            signature,
            message,
            metadata: Metadata::default(),
        }
    }
}

#[derive(Deserialize)]
pub struct VerifyAndCreateSessionResponse {
    pub success: bool,
}

#[derive(Serialize)]
pub struct LinkWalletData<'a> {
    message: &'a str,
    wallet: &'a str,
    chain: &'a str,
    signature: &'a str,
    #[serde(rename = "allocationEvent")]
    allocation_event: &'a str,
    #[serde(rename = "isLedger")]
    is_ledger: bool,
}

impl<'a> LinkWalletData<'a> {
    pub fn new(message: &'a str, wallet: &'a str, signature: &'a str) -> Self {
        Self {
            message,
            chain: "sol",
            wallet,
            signature,
            allocation_event: "tge-airdrop-final",
            is_ledger: false,
        }
    }
}

#[derive(Serialize)]
pub struct LinkWalletJsonWrapper<'a> {
    json: LinkWalletData<'a>,
}

impl<'a> LinkWalletJsonWrapper<'a> {
    pub fn new(data: LinkWalletData<'a>) -> Self {
        Self { json: data }
    }
}

#[derive(Serialize)]
pub struct LinkWalletBody<'a> {
    #[serde(rename = "0")]
    outer: LinkWalletJsonWrapper<'a>,
}

impl<'a> LinkWalletBody<'a> {
    pub fn new(message: &'a str, wallet: &'a str, signature: &'a str) -> Self {
        let data = LinkWalletData::new(message, wallet, signature);
        let json_wrapper = LinkWalletJsonWrapper::new(data);

        Self {
            outer: json_wrapper,
        }
    }
}

#[derive(Deserialize)]
pub struct Eligibility {
    pub eligibility: Option<String>,
}

#[derive(Deserialize)]
pub struct JsonData {
    pub json: Option<JsonEligibility>,
}

#[derive(Deserialize)]
pub struct JsonEligibility {
    pub eligibility: Option<Eligibility>,
}

#[derive(Deserialize)]
pub struct ResultData {
    pub data: Option<JsonData>,
}

#[derive(Deserialize)]
pub struct ResponseItem {
    pub result: Option<ResultData>,
}

pub type LinkWalletResponse = Vec<Option<ResponseItem>>;
