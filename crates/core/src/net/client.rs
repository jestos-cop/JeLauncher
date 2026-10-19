use std::time::Duration;

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::settings::NetworkSettings;
use crate::{Error, Result};

const USER_AGENT: &str = concat!("JeLauncher/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Clone)]
pub struct HttpClient {
    pub(super) inner: reqwest::Client,
    pub(super) retries: u32,
}

impl HttpClient {
    pub fn new(settings: &NetworkSettings) -> Result<Self> {
        let mut builder = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .connect_timeout(Duration::from_secs(15))
            .read_timeout(Duration::from_secs(settings.request_timeout_secs.into()));
        if let Some(proxy) = settings
            .proxy
            .as_deref()
            .filter(|proxy| !proxy.trim().is_empty())
        {
            let proxy = reqwest::Proxy::all(proxy.trim())
                .map_err(|err| Error::InvalidInput(format!("invalid proxy {proxy}: {err}")))?;
            builder = builder.proxy(proxy);
        }
        let inner = builder
            .build()
            .map_err(|err| Error::http("client setup", err))?;
        Ok(Self {
            inner,
            retries: settings.retries,
        })
    }

    pub async fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        self.get_json_query(url, &[]).await
    }

    pub async fn get_json_query<T: DeserializeOwned>(
        &self,
        url: &str,
        query: &[(&str, String)],
    ) -> Result<T> {
        send_json(url, self.inner.get(url).query(query)).await
    }

    /// GET with an extra header, e.g. an API key.
    pub async fn get_json_with_header<T: DeserializeOwned>(
        &self,
        url: &str,
        query: &[(&str, String)],
        header: (&str, &str),
    ) -> Result<T> {
        send_json(
            url,
            self.inner.get(url).query(query).header(header.0, header.1),
        )
        .await
    }

    pub async fn get_text(&self, url: &str) -> Result<String> {
        self.inner
            .get(url)
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|err| Error::http(url, err))?
            .text()
            .await
            .map_err(|err| Error::http(url, err))
    }

    pub async fn post_json_with_header<B: Serialize + ?Sized, T: DeserializeOwned>(
        &self,
        url: &str,
        body: &B,
        header: (&str, &str),
    ) -> Result<T> {
        send_json(
            url,
            self.inner.post(url).json(body).header(header.0, header.1),
        )
        .await
    }

    /// Posts a form and returns the status with the JSON body, even for error statuses.
    pub async fn post_form_with_status(
        &self,
        url: &str,
        form: &[(&str, &str)],
    ) -> Result<(u16, serde_json::Value)> {
        send_with_status(url, self.inner.post(url).form(form)).await
    }

    /// Posts JSON and returns the status with the JSON body, even for error statuses.
    pub async fn post_json_with_status<B: Serialize + ?Sized>(
        &self,
        url: &str,
        body: &B,
    ) -> Result<(u16, serde_json::Value)> {
        send_with_status(url, self.inner.post(url).json(body)).await
    }

    /// GET with a bearer token, returning the status with the JSON body.
    pub async fn get_bearer_with_status(
        &self,
        url: &str,
        token: &str,
    ) -> Result<(u16, serde_json::Value)> {
        send_with_status(url, self.inner.get(url).bearer_auth(token)).await
    }

    pub async fn post_json<B: Serialize + ?Sized, T: DeserializeOwned>(
        &self,
        url: &str,
        body: &B,
    ) -> Result<T> {
        send_json(url, self.inner.post(url).json(body)).await
    }
}

async fn send_with_status(
    url: &str,
    request: reqwest::RequestBuilder,
) -> Result<(u16, serde_json::Value)> {
    let response = request.send().await.map_err(|err| Error::http(url, err))?;
    let status = response.status().as_u16();
    let bytes = response
        .bytes()
        .await
        .map_err(|err| Error::http(url, err))?;
    Ok((status, serde_json::from_slice(&bytes).unwrap_or_default()))
}

async fn send_json<T: DeserializeOwned>(url: &str, request: reqwest::RequestBuilder) -> Result<T> {
    request
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|err| Error::http(url, err))?
        .json()
        .await
        .map_err(|err| Error::http(url, err))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_http_proxies_and_rejects_garbage() {
        let with_proxy = |proxy: &str| NetworkSettings {
            proxy: Some(proxy.into()),
            ..NetworkSettings::default()
        };
        assert!(HttpClient::new(&with_proxy("http://127.0.0.1:8080")).is_ok());
        assert!(HttpClient::new(&with_proxy("  ")).is_ok());
        assert!(HttpClient::new(&with_proxy("not a proxy")).is_err());
    }
}
