use std::sync::Arc;

use reqwest::{
    cookie::Jar,
    header::{
        HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, CONNECTION, CONTENT_TYPE, HOST, REFERER,
        REFERRER_POLICY, USER_AGENT,
    },
    Method, Proxy,
};

use crate::utils::fetch::{send_http_request, RequestParams};

use super::{
    constants::{AUTH_LINK_WALLET, AUTH_SESSION, VERIFY_AND_CREATE_SESSION, WALLETS},
    schemas::{
        LinkWalletBody, LinkWalletResponse, VerifyAndCreateSessionBody,
        VerifyAndCreateSessionResponse,
    },
    typedefs::RootJson,
};

pub async fn verify_and_create_session(
    address: &str,
    signature: &str,
    message: &str,
    proxy: Option<&Proxy>,
    cookie_jar: Option<Arc<Jar>>,
) -> eyre::Result<Option<VerifyAndCreateSessionResponse>> {
    let body = VerifyAndCreateSessionBody::new(address, signature, message);

    let mut headers = HeaderMap::new();

    headers.insert(HOST, HeaderValue::from_static("api-mainnet.magiceden.io"));
    headers.insert("x-exodus-app-id", HeaderValue::from_static("magic-eden"));
    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(
        "x-requested-with",
        HeaderValue::from_static("magic-eden 2.30.0 mobile"),
    );
    headers.insert("x-exodus-platform", HeaderValue::from_static("ios"));
    headers.insert(
        ACCEPT_LANGUAGE,
        HeaderValue::from_static("en-US;q=0.5,en;q=0.3"),
    );
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("Magic%20Eden/194 CFNetwork/1496.0.7 Darwin/23.5.0"),
    );
    headers.insert(CONNECTION, HeaderValue::from_static("keep-alive"));
    headers.insert("x-exodus-version", HeaderValue::from_static("2.30.0"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let request_params = RequestParams {
        url: VERIFY_AND_CREATE_SESSION,
        method: Method::POST,
        body: Some(body),
        query_args: None,
        proxy,
        headers: Some(headers),
    };

    send_http_request::<VerifyAndCreateSessionResponse>(request_params, cookie_jar).await
}

pub async fn auth_session(
    uuid: &str,
    proxy: Option<&Proxy>,
    cookie_jar: Option<Arc<Jar>>,
) -> eyre::Result<()> {
    let mut headers = HeaderMap::new();

    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36 OPR/114.0.0.0 (Edition Yx GX)"),
    );
    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(
        ACCEPT_LANGUAGE,
        HeaderValue::from_static("en-US;q=0.5,en;q=0.3"),
    );
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://mefoundation.com/login"),
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert("x-trpc-source", HeaderValue::from_static("nextjs-react"));
    headers.insert(
        "sentry-trace",
        HeaderValue::from_static("4c8344fb2c0942bca3995cd102a4223c-ab959a225a809cc8-1"),
    );
    headers.insert(
        "baggage",
        HeaderValue::from_static("sentry-environment=production,sentry-release=OXJ8HjdYzWapTs_F5Efi8,sentry-public_key=1a5e7baa354df159cf3efd1eeca5baea,sentry-trace_id=4c8344fb2c0942bca3995cd102a4223c,sentry-sample_rate=1,sentry-sampled=true"),
    );
    headers.insert(CONNECTION, HeaderValue::from_static("keep-alive"));
    headers.insert("Sec-Fetch-Dest", HeaderValue::from_static("empty"));
    headers.insert("Sec-Fetch-Mode", HeaderValue::from_static("cors"));
    headers.insert("Sec-Fetch-Site", HeaderValue::from_static("same-origin"));
    headers.insert("Priority", HeaderValue::from_static("u=4"));
    headers.insert("TE", HeaderValue::from_static("trailers"));

    let query = RootJson::to_string(uuid).expect("Failed to stringify receipt query");

    let query_args = [("batch", "1"), ("input", query.as_str())]
        .into_iter()
        .collect();

    let request_params = RequestParams {
        url: AUTH_SESSION,
        method: Method::GET,
        body: None::<serde_json::Value>,
        query_args: Some(query_args),
        proxy,
        headers: Some(headers),
    };

    send_http_request::<serde_json::Value>(request_params, cookie_jar).await?;

    Ok(())
}

pub async fn auth_link_wallet(
    message: &str,
    wallet: &str,
    signature: &str,
    proxy: Option<&Proxy>,
    cookie_jar: Option<Arc<Jar>>,
) -> eyre::Result<Option<LinkWalletResponse>> {
    let mut headers = HeaderMap::new();

    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(
        ACCEPT_LANGUAGE,
        HeaderValue::from_static("en-US;q=0.5,en;q=0.3"),
    );
    headers.insert(
        "baggage",
        HeaderValue::from_static("sentry-environment=production,sentry-release=jY6mki4_Tqyy2LJT5ljgm,sentry-public_key=9db2fb508ab642eedd5d51bf3618740b,sentry-trace_id=fdac1520ca6c46a7afcc8f20fb119f2d,sentry-replay_id=c753b4fe121042339939e5a16010d415,sentry-sample_rate=0.05,sentry-sampled=true"),
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        "sec-ch-ua",
        HeaderValue::from_static(
            r#""Not_A Brand";v="8", "Chromium";v="120", "Google Chrome";v="120""#,
        ),
    );
    headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
    headers.insert("sec-ch-ua-platform", HeaderValue::from_static(r#""macOS""#));
    headers.insert("Sec-Fetch-Dest", HeaderValue::from_static("empty"));
    headers.insert("Sec-Fetch-Mode", HeaderValue::from_static("cors"));
    headers.insert("Sec-Fetch-Site", HeaderValue::from_static("same-origin"));
    headers.insert(
        "sentry-trace",
        HeaderValue::from_static("4c8344fb2c0942bca3995cd102a4223c-ab959a225a809cc8-1"),
    );
    headers.insert("x-trpc-source", HeaderValue::from_static("nextjs-react"));
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://mefoundation.com/wallets?eligible=false"),
    );
    headers.insert(
        REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    let query_args = [("batch", "1")].into_iter().collect();

    let body = LinkWalletBody::new(message, wallet, signature);

    let request_params = RequestParams {
        url: AUTH_LINK_WALLET,
        method: Method::POST,
        body: Some(body),
        query_args: Some(query_args),
        proxy,
        headers: Some(headers),
    };

    send_http_request::<LinkWalletResponse>(request_params, cookie_jar).await
}

pub async fn wallets(
    proxy: Option<&Proxy>,
    cookie_jar: Option<Arc<Jar>>,
) -> eyre::Result<Option<String>> {
    let mut headers = HeaderMap::new();

    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.5"));
    headers.insert("baggage", HeaderValue::from_static("sentry-environment=production,sentry-release=rUjks-Y9GR01z74atxAEP,sentry-public_key=43f5a6f01fe6dff7b5c0d7c54530d6a0,sentry-trace_id=ef57f76f823948928981c4fe54fdb863,sentry-sample_rate=0.05,sentry-sampled=false"));
    headers.insert("priority", HeaderValue::from_static("u=1, i"));
    headers.insert("next-router-state-tree", HeaderValue::from_static("%5B%22%22%2C%7B%22children%22%3A%5B%22(dashboard)%22%2C%7B%22children%22%3A%5B%22(link)%22%2C%7B%22children%22%3A%5B%22wallets%22%2C%7B%22children%22%3A%5B%22__PAGE__%22%2C%7B%7D%2C%22%2Fwallets%22%2C%22refresh%22%5D%7D%5D%7D%5D%7D%5D%7D%2Cnull%2C%22refetch%22%5D"));
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://mefoundation.com/wallets"),
    );
    headers.insert("rsc", HeaderValue::from_static("1"));
    headers.insert("sec-fetch-dest", HeaderValue::from_static("empty"));
    headers.insert("sec-fetch-mode", HeaderValue::from_static("cors"));
    headers.insert("sec-fetch-site", HeaderValue::from_static("same-origin"));
    headers.insert("sec-fetch-user", HeaderValue::from_static("?1"));
    headers.insert("upgrade-insecure-requests", HeaderValue::from_static("1"));
    headers.insert("user-agent", HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36 OPR/114.0.0.0 (Edition Yx GX)"));

    let query_args = [("_rsc", "1vr9w")].into_iter().collect();

    let request_params = RequestParams {
        url: WALLETS,
        method: Method::GET,
        body: None::<serde_json::Value>,
        query_args: Some(query_args),
        proxy,
        headers: Some(headers),
    };

    send_http_request::<String>(request_params, cookie_jar).await
}
