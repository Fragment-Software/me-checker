use std::{collections::HashMap, sync::Arc};

use reqwest::{cookie::Jar, header::HeaderMap, Method};
use serde::{de::DeserializeOwned, Serialize};

#[derive(Clone)]
pub struct RequestParams<'a, S: Serialize> {
    pub url: &'a str,
    pub method: Method,
    pub body: Option<S>,
    pub query_args: Option<HashMap<&'a str, &'a str>>,
    pub proxy: Option<&'a reqwest::Proxy>,
    pub headers: Option<HeaderMap>,
}

pub async fn send_http_request<R: DeserializeOwned>(
    request_params: RequestParams<'_, impl Serialize>,
    cookie_jar: Option<Arc<Jar>>,
) -> eyre::Result<Option<R>> {
    let client_builder = reqwest::Client::builder();
    let client = if let Some(proxy) = request_params.proxy {
        client_builder.proxy(proxy.clone())
    } else {
        client_builder
    };

    let client = if let Some(jar) = cookie_jar.clone() {
        client.cookie_provider(jar).build().unwrap_or_else(|err| {
            tracing::error!("Failed to build a client with cookies. Error: {err}");
            reqwest::Client::new()
        })
    } else {
        client.build().unwrap_or_else(|err| {
            tracing::error!("Failed to build a client. Error: {err}");
            reqwest::Client::new()
        })
    };

    let mut request = client.request(request_params.method.clone(), request_params.url);

    if let Some(params) = &request_params.query_args {
        request = request.query(&params);
    }

    if let Some(body) = &request_params.body {
        request = request.json(&body);
    }

    if let Some(headers) = request_params.headers {
        request = request.headers(headers.clone());
    }

    let response = request.send().await.inspect_err(|e| {
        tracing::error!(
            "Request failed: {}. Proxy: {:?}",
            e,
            match request_params.proxy {
                Some(p) => format!("{:?}", p),
                None => "No proxy".to_string(),
            }
        )
    })?;

    let response_headers = response.headers().clone();
    let status = response.status();

    let text = response
        .text()
        .await
        .inspect_err(|e| tracing::error!("Failed to retrieve response text: {}", e))?;

    if !status.is_success() {
        tracing::error!(
            "Request failed with status: {}. Response text: {}. Proxy: {:?}",
            status,
            text,
            match request_params.proxy {
                Some(p) => format!("{:?}", p),
                None => "No proxy".to_string(),
            }
        );
        eyre::bail!("HTTP error: {status} - {text}");
    }

    let content_type = response_headers
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");

    let response_body = if text.trim().is_empty() {
        None
    } else {
        let deserialized = if content_type.contains("application/json") {
            serde_json::from_str::<R>(&text)
        } else {
            let json_value = serde_json::json!(text);
            serde_json::from_value::<R>(json_value)
        }
        .inspect_err(|e| tracing::error!("Failed to deserialize response: {}\n {} ", e, text))?;

        Some(deserialized)
    };

    Ok(response_body)
}
