use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct VersionManifest {
    pub latest: LatestVersions,
    pub versions: Vec<VersionEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize, specta::Type)]
pub struct LatestVersions {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct VersionEntry {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: VersionType,
    pub url: String,
    pub sha1: String,
    pub release_time: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum VersionType {
    Release,
    Snapshot,
    OldBeta,
    OldAlpha,
    #[serde(other)]
    Other,
}

impl VersionManifest {
    pub fn find(&self, id: &str) -> Option<&VersionEntry> {
        self.versions.iter().find(|entry| entry.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_manifest_entries() -> Result<(), serde_json::Error> {
        let manifest: VersionManifest = serde_json::from_str(
            r#"{"latest":{"release":"1.21.4","snapshot":"25w01a"},"versions":[
                {"id":"1.21.4","type":"release","url":"u","time":"t","releaseTime":"r","sha1":"s","complianceLevel":1},
                {"id":"b1.7.3","type":"old_beta","url":"u","time":"t","releaseTime":"r","sha1":"s"}]}"#,
        )?;
        assert_eq!(
            manifest.find("b1.7.3").map(|v| v.kind),
            Some(VersionType::OldBeta)
        );
        assert!(manifest.find("missing").is_none());
        Ok(())
    }
}
