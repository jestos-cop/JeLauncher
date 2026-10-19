use serde::{Deserialize, Serialize};

use super::ResourceFile;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum ModListFormat {
    PlainText,
    Markdown,
    Html,
    Json,
    Csv,
}

#[derive(Serialize)]
struct Entry<'a> {
    name: &'a str,
    version: &'a str,
    authors: String,
    url: &'a str,
    file: &'a str,
}

/// Renders the enabled resources as a shareable list.
pub fn format_mod_list(files: &[ResourceFile], format: ModListFormat) -> String {
    let entries: Vec<Entry> = files
        .iter()
        .filter(|file| file.enabled)
        .map(|file| {
            let metadata = file.metadata.as_ref();
            Entry {
                name: metadata.map_or(&file.file_name, |m| &m.name),
                version: metadata.map_or("", |m| &m.version),
                authors: metadata.map(|m| m.authors.join(", ")).unwrap_or_default(),
                url: metadata.and_then(|m| m.homepage.as_deref()).unwrap_or(""),
                file: &file.file_name,
            }
        })
        .collect();
    match format {
        ModListFormat::PlainText => lines(&entries, |e| with_version(e.name, e.version)),
        ModListFormat::Markdown => lines(&entries, |e| {
            let name = if e.url.is_empty() {
                e.name.to_owned()
            } else {
                format!("[{}]({})", e.name, e.url)
            };
            format!("- {}", with_version(&name, e.version))
        }),
        ModListFormat::Html => {
            let items = lines(&entries, |e| {
                let name = html_escape(e.name);
                let name = if e.url.is_empty() {
                    name
                } else {
                    format!("<a href=\"{}\">{name}</a>", html_escape(e.url))
                };
                format!(
                    "  <li>{}</li>",
                    with_version(&name, &html_escape(e.version))
                )
            });
            format!("<ul>\n{items}</ul>\n")
        }
        ModListFormat::Json => serde_json::to_string_pretty(&entries).unwrap_or_default(),
        ModListFormat::Csv => {
            let rows = lines(&entries, |e| {
                [e.name, e.version, &e.authors, e.url, e.file]
                    .map(csv_field)
                    .join(",")
            });
            format!("name,version,authors,url,file\n{rows}")
        }
    }
}

fn lines(entries: &[Entry], line: impl Fn(&Entry) -> String) -> String {
    entries
        .iter()
        .map(|entry| format!("{}\n", line(entry)))
        .collect()
}

fn with_version(name: &str, version: &str) -> String {
    if version.is_empty() {
        name.to_owned()
    } else {
        format!("{name} {version}")
    }
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn csv_field(field: &str) -> String {
    if field.contains([',', '"', '\n']) {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::ResourceMetadata;

    fn files() -> Vec<ResourceFile> {
        let file = |name: &str, enabled: bool, metadata: Option<ResourceMetadata>| ResourceFile {
            file_name: name.into(),
            enabled,
            size: 1,
            modified: 0,
            metadata,
        };
        vec![
            file(
                "sodium.jar",
                true,
                Some(ResourceMetadata {
                    id: "sodium".into(),
                    name: "Sodium".into(),
                    version: "0.6".into(),
                    description: String::new(),
                    authors: vec!["Jelly, Squid".into()],
                    homepage: Some("https://modrinth.com/mod/sodium".into()),
                }),
            ),
            file("plain.jar", true, None),
            file("off.jar.disabled", false, None),
        ]
    }

    #[test]
    fn formats_enabled_mods() {
        assert_eq!(
            format_mod_list(&files(), ModListFormat::PlainText),
            "Sodium 0.6\nplain.jar\n"
        );
        assert_eq!(
            format_mod_list(&files(), ModListFormat::Markdown),
            "- [Sodium](https://modrinth.com/mod/sodium) 0.6\n- plain.jar\n"
        );
        assert!(
            format_mod_list(&files(), ModListFormat::Csv)
                .contains("Sodium,0.6,\"Jelly, Squid\",https://modrinth.com/mod/sodium,sodium.jar")
        );
        assert!(
            format_mod_list(&files(), ModListFormat::Html)
                .contains("<a href=\"https://modrinth.com/mod/sodium\">Sodium</a> 0.6")
        );
    }
}
