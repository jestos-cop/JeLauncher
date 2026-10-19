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
        let inner = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .connect_timeout(Duration::from_secs(15))
            .read_timeout(Duration::from_secs(settings.request_timeout_secs.into()))
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
        self.inner
            .get(url)
            .query(query)
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|err| Error::http(url, err))?
            .json()
            .await
            .map_err(|err| Error::http(url, err))
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

    pub async fn post_json<B: Serialize + ?Sized, T: DeserializeOwned>(
        &self,
        url: &str,
        body: &B,
    ) -> Result<T> {
        self.inner
            .post(url)
            .json(body)
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|err| Error::http(url, err))?
            .json()
            .await
            .map_err(|err| Error::http(url, err))
    }
}
