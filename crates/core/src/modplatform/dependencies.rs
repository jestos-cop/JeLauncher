//! Working out which dependencies a version brings along, and installing the chosen ones.

use std::collections::{HashMap, HashSet, VecDeque};

use serde::Serialize;

use super::{
    DependencyKind, InstallTarget, InstalledFile, ModPlatforms, ProjectVersion, Provider,
    download_version,
};
use crate::resources::read_sources;
use crate::{Error, Result};

/// A project a version to install depends on, directly or through other dependencies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PlannedDependency {
    pub project_id: String,
    pub name: String,
    /// Optional dependencies are only offered for the version itself, not for its dependencies.
    pub kind: DependencyKind,
    /// The version that would be installed; `None` when none fits the game version and loader.
    pub version: Option<ProjectVersion>,
    /// Projects in the plan that need this one: the installed version or other dependencies.
    pub required_by: Vec<String>,
    /// Already installed from the same platform, so it is left alone.
    pub installed: bool,
}

impl ModPlatforms {
    /// The dependencies installing `version` would bring along, required ones transitively,
    /// in the order they were found.
    pub async fn plan_dependencies(
        &self,
        provider: Provider,
        version: &ProjectVersion,
        target: &InstallTarget<'_>,
    ) -> Result<Vec<PlannedDependency>> {
        let installed = installed_projects(provider, target);
        let mut planned: Vec<PlannedDependency> = Vec::new();
        let mut queue = VecDeque::from([version.clone()]);
        while let Some(parent) = queue.pop_front() {
            let is_root = parent.project_id == version.project_id;
            for dependency in &parent.dependencies {
                let offered = dependency.is_required()
                    || (is_root && dependency.kind == DependencyKind::Optional);
                if !offered {
                    continue;
                }
                let Some(project_id) = self.dependency_project(provider, dependency).await? else {
                    continue;
                };
                if project_id == version.project_id {
                    continue;
                }
                if let Some(known) = planned
                    .iter_mut()
                    .find(|known| known.project_id == project_id)
                {
                    if !known.required_by.contains(&parent.project_id) {
                        known.required_by.push(parent.project_id.clone());
                    }
                    if dependency.is_required() {
                        known.kind = DependencyKind::Required;
                    }
                    continue;
                }
                let is_installed = installed.contains(&project_id);
                let resolved = match (&dependency.version_id, is_installed) {
                    (_, true) => None,
                    (Some(version_id), false) => {
                        Some(self.version(provider, &project_id, version_id).await?)
                    }
                    (None, false) => {
                        self.latest_compatible(provider, &project_id, target)
                            .await?
                    }
                };
                if let Some(resolved) = &resolved {
                    queue.push_back(resolved.clone());
                }
                planned.push(PlannedDependency {
                    project_id,
                    name: String::new(),
                    kind: dependency.kind,
                    version: resolved,
                    required_by: vec![parent.project_id.clone()],
                    installed: is_installed,
                });
            }
        }
        self.name_dependencies(provider, &mut planned).await;
        Ok(planned)
    }

    /// Installs the version and, recursively, the required dependencies that fit the target.
    /// Returns the installed file names, the requested version first.
    pub async fn install_with_dependencies(
        &self,
        provider: Provider,
        version: ProjectVersion,
        target: &InstallTarget<'_>,
    ) -> Result<Vec<String>> {
        let planned = self.plan_dependencies(provider, &version, target).await?;
        let mut dependencies = Vec::new();
        for dependency in required_only(&version.project_id, planned) {
            match dependency.version {
                Some(version) => dependencies.push(version),
                None if dependency.installed => {}
                None => return Err(no_compatible_version(&dependency.name, target)),
            }
        }
        self.install_versions(provider, version, dependencies, target)
            .await
    }

    /// Installs a version along with dependency versions the user picked from its plan.
    /// Returns the installed file names, the requested version first.
    pub async fn install_versions(
        &self,
        provider: Provider,
        version: ProjectVersion,
        dependencies: Vec<ProjectVersion>,
        target: &InstallTarget<'_>,
    ) -> Result<Vec<String>> {
        let mut installed = Vec::new();
        let mut recorded = Vec::new();
        for version in std::iter::once(version).chain(dependencies) {
            let file_name = download_version(self.http(), &version, target.dir).await?;
            if let Some(file) = version.primary_file() {
                recorded.push(InstalledFile {
                    file_name: file_name.clone(),
                    file: file.clone(),
                    version,
                });
            }
            installed.push(file_name);
        }
        if target.track_sources {
            self.record_sources(target.dir, target.kind, provider, &recorded, false)
                .await?;
        }
        Ok(installed)
    }

    /// The project a dependency points at; Modrinth sometimes only names the version.
    async fn dependency_project(
        &self,
        provider: Provider,
        dependency: &super::Dependency,
    ) -> Result<Option<String>> {
        match (&dependency.project_id, &dependency.version_id) {
            (Some(project_id), _) => Ok(Some(project_id.clone())),
            (None, Some(version_id)) if provider == Provider::Modrinth => {
                Ok(Some(self.modrinth.version(version_id).await?.project_id))
            }
            (None, _) => Ok(None),
        }
    }

    async fn latest_compatible(
        &self,
        provider: Provider,
        project_id: &str,
        target: &InstallTarget<'_>,
    ) -> Result<Option<ProjectVersion>> {
        Ok(self
            .versions(
                provider,
                project_id,
                target.kind,
                Some(target.game_version),
                target.loader,
            )
            .await?
            .into_iter()
            .next())
    }

    /// Fills in project names; a project the platform does not describe keeps its id.
    async fn name_dependencies(&self, provider: Provider, planned: &mut [PlannedDependency]) {
        let ids: Vec<String> = planned.iter().map(|dep| dep.project_id.clone()).collect();
        let infos = self
            .project_infos(provider, &ids)
            .await
            .unwrap_or_else(|err| {
                log::warn!("could not name dependencies: {err}");
                HashMap::new()
            });
        for dependency in planned {
            dependency.name = infos
                .get(&dependency.project_id)
                .map(|info| info.name.clone())
                .unwrap_or_else(|| dependency.project_id.clone());
        }
    }
}

/// Project ids installed from `provider` in the target folder, when sources are tracked.
fn installed_projects(provider: Provider, target: &InstallTarget<'_>) -> HashSet<String> {
    if !target.track_sources {
        return HashSet::new();
    }
    read_sources(target.dir, target.kind)
        .into_iter()
        .filter(|(_, source)| source.provider == provider)
        .map(|(_, source)| source.project_id)
        .collect()
}

/// The dependencies reached from `root` through required links only.
fn required_only(root: &str, planned: Vec<PlannedDependency>) -> Vec<PlannedDependency> {
    let mut reached = HashSet::from([root.to_owned()]);
    let mut changed = true;
    while changed {
        changed = false;
        for dependency in &planned {
            if dependency.kind == DependencyKind::Required
                && !reached.contains(&dependency.project_id)
                && dependency
                    .required_by
                    .iter()
                    .any(|parent| reached.contains(parent))
            {
                reached.insert(dependency.project_id.clone());
                changed = true;
            }
        }
    }
    planned
        .into_iter()
        .filter(|dependency| {
            reached.contains(&dependency.project_id) && dependency.project_id != root
        })
        .collect()
}

fn no_compatible_version(name: &str, target: &InstallTarget<'_>) -> Error {
    Error::InvalidInput(format!(
        "no version of required dependency {name} supports Minecraft {}",
        target.game_version
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn planned(id: &str, kind: DependencyKind, required_by: &[&str]) -> PlannedDependency {
        PlannedDependency {
            project_id: id.into(),
            name: id.into(),
            kind,
            version: None,
            required_by: required_by.iter().map(|id| (*id).to_owned()).collect(),
            installed: false,
        }
    }

    #[test]
    fn required_only_skips_optional_branches() {
        let plan = vec![
            planned("api", DependencyKind::Required, &["root"]),
            planned("lib", DependencyKind::Required, &["api"]),
            planned("extra", DependencyKind::Optional, &["root"]),
            planned("extra-lib", DependencyKind::Required, &["extra"]),
        ];
        let ids: Vec<String> = required_only("root", plan)
            .into_iter()
            .map(|dep| dep.project_id)
            .collect();
        assert_eq!(ids, ["api", "lib"]);
    }
}
