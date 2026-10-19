use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::version::{Arguments, Library, VersionJson};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "lowercase")]
pub enum LoaderKind {
    Fabric,
    Quilt,
    Forge,
    NeoForge,
}

impl LoaderKind {
    /// Lowercase id used in file names and by mod platforms.
    pub fn id(self) -> &'static str {
        match self {
            Self::Fabric => "fabric",
            Self::Quilt => "quilt",
            Self::Forge => "forge",
            Self::NeoForge => "neoforge",
        }
    }

    /// Forge and NeoForge are installed by running their installer instead of a meta profile.
    pub fn uses_installer(self) -> bool {
        matches!(self, Self::Forge | Self::NeoForge)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct LoaderSpec {
    pub kind: LoaderKind,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
pub struct LoaderVersion {
    pub version: String,
    pub stable: bool,
}

/// A loader's version file, which inherits everything else from the vanilla version.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoaderProfile {
    pub main_class: String,
    #[serde(default)]
    pub arguments: Option<Arguments>,
    #[serde(default)]
    pub libraries: Vec<Library>,
}

impl VersionJson {
    /// Layers a loader profile on top of this vanilla version, the way `inheritsFrom` works:
    /// the loader's main class wins, its arguments are appended and its libraries replace
    /// vanilla libraries with the same identity.
    pub fn apply_loader(&mut self, profile: LoaderProfile) {
        self.main_class = profile.main_class;

        let replaced: HashSet<String> = profile.libraries.iter().map(Library::identity).collect();
        self.libraries
            .retain(|library| !replaced.contains(&library.identity()));
        let mut libraries = profile.libraries;
        libraries.append(&mut self.libraries);
        self.libraries = libraries;

        if let Some(extra) = profile.arguments {
            let arguments = self.arguments.get_or_insert_with(Arguments::default);
            arguments.game.extend(extra.game);
            arguments.jvm.extend(extra.jvm);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loader_profile_overrides_main_class_and_libraries() -> Result<(), serde_json::Error> {
        let mut version: VersionJson = serde_json::from_str(
            r#"{"id":"1.21.4","mainClass":"net.minecraft.client.main.Main","assets":"19",
                "assetIndex":{"id":"19","sha1":"s","size":1,"url":"u"},
                "downloads":{"client":{"sha1":"s","size":1,"url":"u"}},
                "arguments":{"game":["--demo"],"jvm":["-cp","${classpath}"]},
                "libraries":[{"name":"org.ow2.asm:asm:9.6"},{"name":"com.mojang:brigadier:1.0"}]}"#,
        )?;
        let profile: LoaderProfile = serde_json::from_str(
            r#"{"mainClass":"net.fabricmc.loader.impl.launch.knot.KnotClient",
                "arguments":{"game":[],"jvm":["-DFabricMcEmu= net.minecraft.client.main.Main "]},
                "libraries":[{"name":"org.ow2.asm:asm:9.7.1","url":"https://maven.fabricmc.net/","sha1":"f0","size":1}]}"#,
        )?;
        version.apply_loader(profile);

        assert_eq!(
            version.main_class,
            "net.fabricmc.loader.impl.launch.knot.KnotClient"
        );
        let names: Vec<_> = version.libraries.iter().map(|l| l.name.as_str()).collect();
        assert_eq!(names, ["org.ow2.asm:asm:9.7.1", "com.mojang:brigadier:1.0"]);
        assert_eq!(version.arguments.map(|a| a.jvm.len()), Some(3));
        Ok(())
    }

    #[test]
    fn library_identity_ignores_version() -> Result<(), serde_json::Error> {
        let library: Library =
            serde_json::from_str(r#"{"name":"org.lwjgl:lwjgl:3.3.3:natives-macos"}"#)?;
        assert_eq!(library.identity(), "org.lwjgl:lwjgl:natives-macos");
        Ok(())
    }
}
