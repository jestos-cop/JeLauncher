use std::path::{Path, PathBuf};

use reqwest::StatusCode;
use reqwest::header::{ETAG, IF_NONE_MATCH};
use serde::de::DeserializeOwned;

use super::HttpClient;
use crate::{Error, Result, fs};

impl HttpClient {
    /// Fetches JSON that changes over time (e.g. version manifests), revalidating
    /// with ETag and falling back to the cached copy when offline.
    pub async fn get_json_cached<T: DeserializeOwned>(&self, url: &str, cache: &Path) -> Result<T> {
        match self.revalidate(url, cache).await {
            Ok(value) => Ok(value),
            Err(err) => fs::read_json(cache)?.ok_or(err),
        }
    }

    async fn revalidate<T: DeserializeOwned>(&self, url: &str, cache: &Path) -> Result<T> {
        let etag_path = etag_path(cache);
        let mut request = self.inner.get(url);
        if cache.is_file()
            && let Ok(etag) = std::fs::read_to_string(&etag_path)
        {
            request = request.header(IF_NONE_MATCH, etag.trim());
        }
        let response = request.send().await.map_err(|err| Error::http(url, err))?;

        if response.status() == StatusCode::NOT_MODIFIED
            && let Some(value) = fs::read_json(cache)?
        {
            return Ok(value);
        }
        let response = response
            .error_for_status()
            .map_err(|err| Error::http(url, err))?;
        let etag = response
            .headers()
            .get(ETAG)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        let bytes = response
            .bytes()
            .await
            .map_err(|err| Error::http(url, err))?;
        let value = serde_json::from_slice(&bytes).map_err(|err| Error::json(url, err))?;

        fs::write_atomic(cache, &bytes)?;
        match etag {
            Some(etag) => fs::write_atomic(&etag_path, etag.as_bytes())?,
            None => {
                let _ = std::fs::remove_file(&etag_path);
            }
        }
        Ok(value)
    }
}

fn etag_path(cache: &Path) -> PathBuf {
    let mut name = cache.as_os_str().to_owned();
    name.push(".etag");
    PathBuf::from(name)
}
