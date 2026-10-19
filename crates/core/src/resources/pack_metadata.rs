use std::io::Read;
use std::path::Path;

use serde_json::Value;

use super::ResourceMetadata;

/// Reads `pack.mcmeta` from a zipped or unpacked resource/data pack.
pub fn read_pack_metadata(path: &Path, name: &str) -> Option<ResourceMetadata> {
    let text = if path.is_dir() {
        std::fs::read_to_string(path.join("pack.mcmeta")).ok()?
    } else {
        let file = std::fs::File::open(path).ok()?;
        let mut archive = zip::ZipArchive::new(file).ok()?;
        let mut entry = archive.by_name("pack.mcmeta").ok()?;
        let mut text = String::new();
        entry.read_to_string(&mut text).ok()?;
        text
    };
    parse_pack_mcmeta(&text, name)
}

fn parse_pack_mcmeta(text: &str, name: &str) -> Option<ResourceMetadata> {
    let root: Value = serde_json::from_str(text.trim_start_matches('\u{feff}')).ok()?;
    let pack = root.get("pack")?;
    let format = pack.get("pack_format").and_then(Value::as_u64);
    Some(ResourceMetadata {
        id: name.to_owned(),
        name: name.to_owned(),
        version: format.map(|f| format!("format {f}")).unwrap_or_default(),
        description: pack
            .get("description")
            .map(flatten_text)
            .unwrap_or_default(),
        authors: Vec::new(),
        homepage: None,
    })
}

/// Descriptions may be plain strings or Minecraft text components.
fn flatten_text(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Array(parts) => parts.iter().map(flatten_text).collect(),
        Value::Object(component) => {
            let text = component
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let extra: String = component
                .get("extra")
                .and_then(Value::as_array)
                .map(|parts| parts.iter().map(flatten_text).collect())
                .unwrap_or_default();
            format!("{text}{extra}")
        }
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_plain_and_component_descriptions() {
        let plain = parse_pack_mcmeta(
            r#"{"pack":{"pack_format":34,"description":"Faithful 32x"}}"#,
            "faithful.zip",
        );
        assert_eq!(
            plain.map(|m| (m.version, m.description)),
            Some(("format 34".into(), "Faithful 32x".into()))
        );

        let component = parse_pack_mcmeta(
            r#"{"pack":{"pack_format":15,"description":[{"text":"Fresh "},{"text":"Animations","extra":[{"text":"!"}]}]}}"#,
            "fa.zip",
        );
        assert_eq!(
            component.map(|m| m.description),
            Some("Fresh Animations!".into())
        );
    }
}
