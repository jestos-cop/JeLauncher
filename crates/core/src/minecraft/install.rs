use std::path::{Path, PathBuf};

use super::assets::AssetIndex;
use super::rule::Environment;
use super::version::{Artifact, VersionJson};
use crate::net::{Download, DownloadProgress, HttpClient};
use crate::paths::DataPaths;
use crate::{Error, Result, archive, fs};

/// Everything a version needs on disk, resolved for one platform.
#[derive(Debug, Clone)]
pub struct GameFiles {
    pub client_jar: PathBuf,
    pub classpath: Vec<PathBuf>,
    pub native_jars: Vec<PathBuf>,
    pub asset_index: PathBuf,
    pub logging_config: Option<PathBuf>,
    assets_dir: PathBuf,
    asset_index_id: String,
    downloads: Vec<Download>,
}

impl GameFiles {
    pub fn resolve(version: &VersionJson, env: &Environment, paths: &DataPaths) -> Self {
        let libraries_dir = paths.libraries();
        let mut downloads = Vec::new();
        let mut classpath = Vec::new();
        let mut native_jars = Vec::new();

        let mut library_download = |artifact: Artifact| -> Option<PathBuf> {
            let path = libraries_dir.join(artifact.path.as_ref()?);
            downloads.push(artifact_download(&artifact, path.clone()));
            Some(path)
        };
        for library in version.libraries.iter().filter(|l| env.allows(&l.rules)) {
            if let Some(path) = library.artifact().and_then(&mut library_download) {
                classpath.push(path);
            }
            if let Some(path) = library.native_artifact(env).and_then(&mut library_download) {
                native_jars.push(path);
            }
        }

        let client_jar = paths
            .versions()
            .join(&version.id)
            .join(format!("{}.jar", version.id));
        downloads.push(artifact_download(
            &version.downloads.client,
            client_jar.clone(),
        ));

        let asset_index = paths
            .assets()
            .join("indexes")
            .join(format!("{}.json", version.asset_index.id));
        downloads.push(Download {
            url: version.asset_index.url.clone(),
            path: asset_index.clone(),
            sha1: Some(version.asset_index.sha1.clone()),
            size: Some(version.asset_index.size),
        });

        let logging_config = version
            .logging
            .as_ref()
            .and_then(|l| l.client.as_ref())
            .map(|client| {
                let path = paths.assets().join("log_configs").join(&client.file.id);
                downloads.push(Download {
                    url: client.file.url.clone(),
                    path: path.clone(),
                    sha1: Some(client.file.sha1.clone()),
                    size: Some(client.file.size),
                });
                path
            });

        dedup_downloads(&mut downloads);
        classpath.dedup();
        Self {
            client_jar,
            classpath,
            native_jars,
            asset_index,
            logging_config,
            assets_dir: paths.assets(),
            asset_index_id: version.asset_index.id.clone(),
            downloads,
        }
    }

    /// Downloads libraries, client jar and assets, then prepares natives and legacy assets.
    pub async fn install(
        &self,
        client: &HttpClient,
        natives_dir: &Path,
        game_dir: &Path,
        concurrency: usize,
        on_progress: impl Fn(DownloadProgress),
    ) -> Result<()> {
        // The asset index tells us which objects to fetch, so it has to come first.
        if let Some(index_download) = self.downloads.iter().find(|d| d.path == self.asset_index) {
            client.download(index_download).await?;
        }
        let index = self.read_asset_index()?;
        let mut downloads = self.downloads.clone();
        downloads.extend(index.downloads(&self.assets_dir));
        client
            .download_all(downloads, concurrency, on_progress)
            .await?;

        extract_natives(&self.native_jars, natives_dir)?;
        index.reconstruct(&self.assets_dir, &self.asset_index_id, game_dir)
    }

    pub fn assets_dir(&self) -> &Path {
        &self.assets_dir
    }

    pub fn asset_index_id(&self) -> &str {
        &self.asset_index_id
    }

    pub fn read_asset_index(&self) -> Result<AssetIndex> {
        fs::read_json(&self.asset_index)?
            .ok_or_else(|| Error::io(&self.asset_index, std::io::ErrorKind::NotFound.into()))
    }
}

fn artifact_download(artifact: &Artifact, path: PathBuf) -> Download {
    Download {
        url: artifact.url.clone(),
        path,
        sha1: Some(artifact.sha1.clone()).filter(|sha1| !sha1.is_empty()),
        size: Some(artifact.size).filter(|&size| size > 0),
    }
}

fn dedup_downloads(downloads: &mut Vec<Download>) {
    let mut seen = std::collections::HashSet::new();
    downloads.retain(|download| seen.insert(download.path.clone()));
}

/// Unpacks legacy native jars, skipping their signature metadata.
fn extract_natives(native_jars: &[PathBuf], natives_dir: &Path) -> Result<()> {
    for jar in native_jars {
        archive::extract(jar, natives_dir, |name| {
            (!name.starts_with("META-INF")).then(|| name.to_path_buf())
        })?;
    }
    Ok(())
}
