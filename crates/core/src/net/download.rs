use std::path::{Path, PathBuf};

use futures_util::{StreamExt, TryStreamExt, stream};
use serde::Serialize;
use sha1::{Digest, Sha1};
use tokio::fs;
use tokio::io::AsyncWriteExt;

use super::HttpClient;
use crate::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Download {
    pub url: String,
    pub path: PathBuf,
    pub sha1: Option<String>,
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub completed: u32,
    pub total: u32,
    pub downloaded_bytes: u64,
}

impl HttpClient {
    /// Downloads one file unless an up-to-date copy is already on disk.
    /// Returns the number of bytes fetched from the network.
    pub async fn download(&self, download: &Download) -> Result<u64> {
        if is_up_to_date(download).await? {
            return Ok(0);
        }
        let mut attempt = 0;
        loop {
            match self.fetch(download).await {
                Ok(bytes) => return Ok(bytes),
                Err(_) if attempt < self.retries => attempt += 1,
                Err(err) => return Err(err),
            }
        }
    }

    /// Downloads files concurrently, reporting progress after each finished file.
    pub async fn download_all(
        &self,
        downloads: Vec<Download>,
        concurrency: usize,
        on_progress: impl Fn(DownloadProgress),
    ) -> Result<()> {
        let mut progress = DownloadProgress {
            total: u32::try_from(downloads.len()).unwrap_or(u32::MAX),
            ..Default::default()
        };
        on_progress(progress);

        let mut results = stream::iter(downloads)
            .map(|download| async move { self.download(&download).await })
            .buffer_unordered(concurrency.max(1));
        while let Some(bytes) = results.try_next().await? {
            progress.completed += 1;
            progress.downloaded_bytes += bytes;
            on_progress(progress);
        }
        Ok(())
    }

    async fn fetch(&self, download: &Download) -> Result<u64> {
        let url = download.url.as_str();
        let response = self
            .inner
            .get(url)
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|err| Error::http(url, err))?;

        let path = &download.path;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|err| Error::io(parent, err))?;
        }
        let tmp = path.with_extension("part");
        let mut file = fs::File::create(&tmp)
            .await
            .map_err(|err| Error::io(&tmp, err))?;
        let mut hasher = Sha1::new();
        let mut written = 0;
        let mut body = response.bytes_stream();
        while let Some(chunk) = body.next().await {
            let chunk = chunk.map_err(|err| Error::http(url, err))?;
            hasher.update(&chunk);
            file.write_all(&chunk)
                .await
                .map_err(|err| Error::io(&tmp, err))?;
            written += chunk.len() as u64;
        }
        file.flush().await.map_err(|err| Error::io(&tmp, err))?;
        drop(file);

        if let Some(expected) = &download.sha1 {
            let actual = hex::encode(hasher.finalize());
            if !actual.eq_ignore_ascii_case(expected) {
                let _ = fs::remove_file(&tmp).await;
                return Err(Error::Checksum {
                    path: path.clone(),
                    expected: expected.clone(),
                    actual,
                });
            }
        }
        fs::rename(&tmp, path)
            .await
            .map_err(|err| Error::io(path, err))?;
        Ok(written)
    }
}

async fn is_up_to_date(download: &Download) -> Result<bool> {
    let Ok(metadata) = fs::metadata(&download.path).await else {
        return Ok(false);
    };
    if download.size.is_some_and(|size| size != metadata.len()) {
        return Ok(false);
    }
    match &download.sha1 {
        Some(expected) => Ok(sha1_of(&download.path)
            .await?
            .eq_ignore_ascii_case(expected)),
        None => Ok(true),
    }
}

/// Checks a file against the MD5 some platforms publish instead of SHA-1.
pub fn verify_md5(path: &Path, expected: &str) -> Result<()> {
    let bytes = std::fs::read(path).map_err(|err| Error::io(path, err))?;
    let actual = hex::encode(md5::Md5::digest(&bytes));
    if actual.eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        Err(Error::Checksum {
            path: path.to_path_buf(),
            expected: expected.to_owned(),
            actual,
        })
    }
}

pub async fn sha1_of(path: &Path) -> Result<String> {
    let bytes = fs::read(path).await.map_err(|err| Error::io(path, err))?;
    Ok(hex::encode(Sha1::digest(&bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn existing_file_with_matching_hash_is_up_to_date() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|err| Error::io("tempdir", err))?;
        let path = dir.path().join("file.txt");
        std::fs::write(&path, b"hello").map_err(|err| Error::io(&path, err))?;
        let download = Download {
            url: String::new(),
            path: path.clone(),
            sha1: Some("aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d".into()),
            size: Some(5),
        };
        assert!(is_up_to_date(&download).await?);

        let stale = Download {
            sha1: Some("0000000000000000000000000000000000000000".into()),
            ..download
        };
        assert!(!is_up_to_date(&stale).await?);
        Ok(())
    }
}
