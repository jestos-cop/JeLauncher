//! Which installed mods need which, from their own descriptors and their platform sources.

use std::collections::{BTreeSet, HashMap};

use super::ResourceFile;
use super::sources::enabled_name;
use crate::modplatform::DependencyKind;

/// Fills `requires` and `required_by` of every file with the file names it is linked to.
pub fn attach_relations(files: &mut [ResourceFile]) {
    let links = links(files);
    for file in files.iter_mut() {
        let name = enabled_name(&file.file_name).to_owned();
        file.requires = links
            .iter()
            .filter(|(from, _)| *from == name)
            .map(|(_, to)| to.clone())
            .collect();
        file.required_by = links
            .iter()
            .filter(|(_, to)| *to == name)
            .map(|(from, _)| from.clone())
            .collect();
    }
}

/// Files whose enabled state should follow when `selected` are enabled or disabled: enabling
/// pulls in what they require, disabling takes along what requires them, transitively. Only
/// files that are not already in the wanted state are returned.
pub fn affected_by_toggle(
    files: &[ResourceFile],
    selected: &[String],
    enable: bool,
) -> Vec<String> {
    let links = links(files);
    let selected: BTreeSet<String> = selected
        .iter()
        .map(|name| enabled_name(name).to_owned())
        .collect();
    let mut seen = selected.clone();
    let mut queue: Vec<String> = selected.iter().cloned().collect();
    let mut affected = Vec::new();
    while let Some(name) = queue.pop() {
        let next = links.iter().filter_map(|(from, to)| match enable {
            true if *from == name => Some(to),
            false if *to == name => Some(from),
            _ => None,
        });
        for other in next {
            if !seen.insert(other.clone()) {
                continue;
            }
            queue.push(other.clone());
            if let Some(file) = files
                .iter()
                .find(|file| enabled_name(&file.file_name) == other)
                && file.enabled != enable
            {
                affected.push(file.file_name.clone());
            }
        }
    }
    affected.sort();
    affected
}

/// Marks files whose source names game versions that do not include `game_version`.
pub fn attach_compatibility(files: &mut [ResourceFile], game_version: &str) {
    for file in files.iter_mut() {
        file.incompatible = file.source.as_ref().is_some_and(|source| {
            !source.game_versions.is_empty()
                && !source
                    .game_versions
                    .iter()
                    .any(|version| version == game_version)
        });
    }
}

/// `(requiring file, required file)` pairs by enabled file name.
fn links(files: &[ResourceFile]) -> BTreeSet<(String, String)> {
    let by_mod_id: HashMap<&str, &str> = files
        .iter()
        .filter_map(|file| {
            Some((
                file.metadata.as_ref()?.id.as_str(),
                enabled_name(&file.file_name),
            ))
        })
        .collect();
    let by_project: HashMap<(_, &str), &str> = files
        .iter()
        .filter_map(|file| {
            let source = file.source.as_ref()?;
            Some((
                (source.provider, source.project_id.as_str()),
                enabled_name(&file.file_name),
            ))
        })
        .collect();
    let mut links = BTreeSet::new();
    for file in files {
        let from = enabled_name(&file.file_name);
        let declared = file
            .metadata
            .iter()
            .flat_map(|metadata| &metadata.dependencies)
            .filter_map(|id| by_mod_id.get(id.as_str()));
        let published = file.source.iter().flat_map(|source| {
            source
                .dependencies
                .iter()
                .filter(|dependency| dependency.kind == DependencyKind::Required)
                .filter_map(|dependency| {
                    by_project.get(&(source.provider, dependency.project_id.as_str()))
                })
        });
        for to in declared.chain(published) {
            if *to != from {
                links.insert((from.to_owned(), (*to).to_owned()));
            }
        }
    }
    links
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::ResourceMetadata;

    fn file(name: &str, id: &str, enabled: bool, dependencies: &[&str]) -> ResourceFile {
        ResourceFile {
            file_name: if enabled {
                name.to_owned()
            } else {
                format!("{name}.disabled")
            },
            enabled,
            size: 1,
            modified: 0,
            metadata: Some(ResourceMetadata {
                id: id.into(),
                name: id.into(),
                version: String::new(),
                description: String::new(),
                authors: Vec::new(),
                homepage: None,
                dependencies: dependencies.iter().map(|id| (*id).to_owned()).collect(),
            }),
            source: None,
            requires: Vec::new(),
            required_by: Vec::new(),
            incompatible: false,
        }
    }

    fn files() -> Vec<ResourceFile> {
        vec![
            file("sodium.jar", "sodium", false, &[]),
            file("iris.jar", "iris", true, &["sodium"]),
            file("addon.jar", "addon", true, &["iris"]),
            file("other.jar", "other", true, &["missing"]),
        ]
    }

    #[test]
    fn links_mods_by_declared_ids() {
        let mut files = files();
        attach_relations(&mut files);
        assert_eq!(files[1].requires, ["sodium.jar"]);
        assert_eq!(files[1].required_by, ["addon.jar"]);
        assert!(files[3].requires.is_empty());
    }

    #[test]
    fn toggling_follows_dependencies_transitively() {
        let files = files();
        assert_eq!(
            affected_by_toggle(&files, &["addon.jar".into()], true),
            ["sodium.jar.disabled"]
        );
        assert_eq!(
            affected_by_toggle(&files, &["sodium.jar.disabled".into()], false),
            ["addon.jar", "iris.jar"]
        );
        assert!(affected_by_toggle(&files, &["other.jar".into()], false).is_empty());
    }
}
