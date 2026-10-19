use std::io::Read;
use std::path::Path;

use serde::Serialize;
use serde_json::Value;

use crate::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ModMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub authors: Vec<String>,
    pub homepage: Option<String>,
}

type Archive = zip::ZipArchive<std::fs::File>;

/// Reads mod metadata from a jar, trying each loader's descriptor in turn.
pub fn read_mod_metadata(path: &Path) -> Result<Option<ModMetadata>> {
    let file = std::fs::File::open(path).map_err(|err| Error::io(path, err))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|err| Error::archive(path, err))?;

    if let Some(text) = read_entry(&mut archive, "fabric.mod.json") {
        return Ok(parse_fabric(&text));
    }
    if let Some(text) = read_entry(&mut archive, "quilt.mod.json") {
        return Ok(parse_quilt(&text));
    }
    for descriptor in ["META-INF/neoforge.mods.toml", "META-INF/mods.toml"] {
        if let Some(text) = read_entry(&mut archive, descriptor) {
            let jar_version = read_entry(&mut archive, "META-INF/MANIFEST.MF")
                .and_then(|manifest| manifest_value(&manifest, "Implementation-Version"));
            return Ok(parse_forge(&text, jar_version.as_deref()));
        }
    }
    if let Some(text) = read_entry(&mut archive, "mcmod.info") {
        return Ok(parse_mcmod_info(&text));
    }
    Ok(None)
}

fn read_entry(archive: &mut Archive, name: &str) -> Option<String> {
    let mut entry = archive.by_name(name).ok()?;
    let mut text = String::new();
    entry.read_to_string(&mut text).ok()?;
    Some(text)
}

fn manifest_value(manifest: &str, key: &str) -> Option<String> {
    manifest
        .lines()
        .filter_map(|line| line.split_once(": "))
        .find(|(name, _)| *name == key)
        .map(|(_, value)| value.trim().to_owned())
}

fn text(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

/// Authors are either plain strings or `{ "name": ... }` objects.
fn people(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|person| person.as_str().or_else(|| person.get("name")?.as_str()))
        .map(str::to_owned)
        .collect()
}

fn parse_fabric(json: &str) -> Option<ModMetadata> {
    let root: Value = serde_json::from_str(json).ok()?;
    let id = text(&root, "id");
    Some(ModMetadata {
        name: Some(text(&root, "name"))
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| id.clone()),
        version: text(&root, "version"),
        description: text(&root, "description"),
        authors: people(root.get("authors")),
        homepage: root
            .pointer("/contact/homepage")
            .and_then(Value::as_str)
            .map(str::to_owned),
        id,
    })
}

fn parse_quilt(json: &str) -> Option<ModMetadata> {
    let root: Value = serde_json::from_str(json).ok()?;
    let loader = root.get("quilt_loader")?;
    let metadata = loader.get("metadata").cloned().unwrap_or(Value::Null);
    let id = text(loader, "id");
    let authors = metadata
        .get("contributors")
        .and_then(Value::as_object)
        .map(|contributors| contributors.keys().cloned().collect())
        .unwrap_or_default();
    Some(ModMetadata {
        name: Some(text(&metadata, "name"))
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| id.clone()),
        version: text(loader, "version"),
        description: text(&metadata, "description"),
        authors,
        homepage: metadata
            .pointer("/contact/homepage")
            .and_then(Value::as_str)
            .map(str::to_owned),
        id,
    })
}

fn parse_forge(toml_text: &str, jar_version: Option<&str>) -> Option<ModMetadata> {
    let root: toml::Table = toml_text.parse().ok()?;
    let first = root.get("mods")?.as_array()?.first()?.as_table()?;
    let field = |key: &str| {
        first
            .get(key)
            .and_then(toml::Value::as_str)
            .map(str::to_owned)
    };
    let id = field("modId")?;
    let version = match field("version") {
        Some(version) if version == "${file.jarVersion}" => {
            jar_version.unwrap_or_default().to_owned()
        }
        other => other.unwrap_or_default(),
    };
    let authors = field("authors")
        .or_else(|| {
            root.get("authors")
                .and_then(toml::Value::as_str)
                .map(str::to_owned)
        })
        .into_iter()
        .collect();
    Some(ModMetadata {
        name: field("displayName").unwrap_or_else(|| id.clone()),
        version,
        description: field("description").unwrap_or_default().trim().to_owned(),
        authors,
        homepage: field("displayURL"),
        id,
    })
}

fn parse_mcmod_info(json: &str) -> Option<ModMetadata> {
    let root: Value = serde_json::from_str(json).ok()?;
    // Version 2 wraps the list in `modList`.
    let list = root.get("modList").unwrap_or(&root);
    let first = list.as_array()?.first()?;
    let id = text(first, "modid");
    let mut authors = people(first.get("authorList"));
    if authors.is_empty() {
        authors = people(first.get("authors"));
    }
    Some(ModMetadata {
        name: Some(text(first, "name"))
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| id.clone()),
        version: text(first, "version"),
        description: text(first, "description"),
        authors,
        homepage: Some(text(first, "url")).filter(|url| !url.is_empty()),
        id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_fabric() {
        let meta = parse_fabric(
            r#"{"id":"sodium","name":"Sodium","version":"0.6.0","description":"Fast","authors":["JellySquid",{"name":"IMS"}],"contact":{"homepage":"https://x"}}"#,
        );
        assert_eq!(
            meta.as_ref().map(|m| m.authors.clone()),
            Some(vec!["JellySquid".into(), "IMS".into()])
        );
        assert_eq!(meta.and_then(|m| m.homepage), Some("https://x".into()));
    }

    #[test]
    fn parses_quilt() {
        let meta = parse_quilt(
            r#"{"quilt_loader":{"id":"qsl","version":"1.0","metadata":{"name":"QSL","contributors":{"Quilt":"Owner"}}}}"#,
        );
        assert_eq!(
            meta.map(|m| (m.name, m.authors)),
            Some(("QSL".into(), vec!["Quilt".into()]))
        );
    }

    #[test]
    fn parses_forge_with_jar_version() {
        let meta = parse_forge(
            "modLoader=\"javafml\"\n[[mods]]\nmodId=\"jei\"\nversion=\"${file.jarVersion}\"\ndisplayName=\"JEI\"\nauthors=\"mezz\"\n",
            Some("19.0.0"),
        );
        assert_eq!(
            meta.map(|m| (m.id, m.version, m.authors)),
            Some(("jei".into(), "19.0.0".into(), vec!["mezz".into()]))
        );
    }

    #[test]
    fn parses_mcmod_info_both_formats() {
        let v1 = parse_mcmod_info(
            r#"[{"modid":"old","name":"Old Mod","version":"1.0","authorList":["A"]}]"#,
        );
        let v2 = parse_mcmod_info(
            r#"{"modListVersion":2,"modList":[{"modid":"old2","authors":["B"]}]}"#,
        );
        assert_eq!(v1.map(|m| m.authors), Some(vec!["A".into()]));
        assert_eq!(
            v2.map(|m| (m.name, m.authors)),
            Some(("old2".into(), vec!["B".into()]))
        );
    }

    #[test]
    fn reads_manifest_values() {
        let manifest = "Manifest-Version: 1.0\r\nImplementation-Version: 2.3.4\r\n";
        assert_eq!(
            manifest_value(manifest, "Implementation-Version").as_deref(),
            Some("2.3.4")
        );
    }
}
