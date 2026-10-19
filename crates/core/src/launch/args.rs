use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::auth::{Account, AccountKind};
use crate::minecraft::{Environment, VersionJson};
use crate::settings::EffectiveSettings;

pub const LAUNCHER_NAME: &str = "JeLauncher";
const CLASSPATH_SEPARATOR: &str = if cfg!(windows) { ";" } else { ":" };

/// Inputs needed to turn a version file into a command line.
pub struct ArgsInput<'a> {
    pub version: &'a VersionJson,
    pub env: &'a Environment,
    pub settings: &'a EffectiveSettings,
    pub account: &'a Account,
    pub classpath: &'a [PathBuf],
    pub game_dir: &'a Path,
    pub natives_dir: &'a Path,
    pub assets_dir: &'a Path,
    pub legacy_assets_dir: &'a Path,
    pub libraries_dir: &'a Path,
}

/// JVM arguments, main class and game arguments, in launch order.
pub fn build_arguments(input: &ArgsInput) -> Vec<String> {
    let vars = variables(input);
    let substitute = |arg: &str| substitute(arg, &vars);
    let settings = input.settings;

    let mut args = vec![
        format!("-Xms{}M", settings.min_memory_mb),
        format!("-Xmx{}M", settings.max_memory_mb),
    ];
    args.extend(settings.jvm_args.split_whitespace().map(str::to_owned));

    match &input.version.arguments {
        Some(arguments) if !arguments.jvm.is_empty() => {
            args.extend(
                arguments
                    .jvm
                    .iter()
                    .flat_map(|a| a.values(input.env))
                    .map(substitute),
            );
        }
        _ => {
            args.push(substitute("-Djava.library.path=${natives_directory}"));
            args.push("-cp".into());
            args.push(substitute("${classpath}"));
        }
    }

    args.push(input.version.main_class.clone());

    if let Some(arguments) = &input.version.arguments {
        args.extend(
            arguments
                .game
                .iter()
                .flat_map(|a| a.values(input.env))
                .map(substitute),
        );
    } else if let Some(legacy) = &input.version.minecraft_arguments {
        args.extend(legacy.split_whitespace().map(substitute));
        if input.env.features.custom_resolution {
            args.extend(["--width".into(), substitute("${resolution_width}")]);
            args.extend(["--height".into(), substitute("${resolution_height}")]);
        }
    }
    args
}

fn variables(input: &ArgsInput) -> HashMap<&'static str, String> {
    let path = |p: &Path| p.to_string_lossy().into_owned();
    let classpath = input
        .classpath
        .iter()
        .map(|entry| path(entry))
        .collect::<Vec<_>>()
        .join(CLASSPATH_SEPARATOR);
    let user_type = match input.account.kind {
        AccountKind::Offline => "legacy",
    };
    HashMap::from([
        ("auth_player_name", input.account.username.clone()),
        ("auth_uuid", input.account.id.clone()),
        ("auth_access_token", "0".into()),
        ("auth_session", "0".into()),
        ("auth_xuid", "0".into()),
        ("clientid", String::new()),
        ("user_type", user_type.into()),
        ("user_properties", "{}".into()),
        ("version_name", input.version.id.clone()),
        ("version_type", input.version.kind.clone()),
        ("game_directory", path(input.game_dir)),
        ("assets_root", path(input.assets_dir)),
        ("game_assets", path(input.legacy_assets_dir)),
        ("assets_index_name", input.version.asset_index.id.clone()),
        ("natives_directory", path(input.natives_dir)),
        ("library_directory", path(input.libraries_dir)),
        ("classpath", classpath),
        ("classpath_separator", CLASSPATH_SEPARATOR.into()),
        ("launcher_name", LAUNCHER_NAME.into()),
        ("launcher_version", env!("CARGO_PKG_VERSION").into()),
        ("resolution_width", input.settings.window_width.to_string()),
        (
            "resolution_height",
            input.settings.window_height.to_string(),
        ),
    ])
}

/// Replaces every `${name}` with its value; unknown names are left untouched.
fn substitute(arg: &str, vars: &HashMap<&'static str, String>) -> String {
    let mut out = String::with_capacity(arg.len());
    let mut rest = arg;
    while let Some(start) = rest.find("${") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        match after.find('}') {
            Some(end) => {
                let name = &after[..end];
                match vars.get(name) {
                    Some(value) => out.push_str(value),
                    None => out.push_str(&rest[start..start + 2 + end + 1]),
                }
                rest = &after[end + 1..];
            }
            None => {
                out.push_str(&rest[start..]);
                rest = "";
            }
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::minecraft::{Features, OsName};

    #[test]
    fn substitutes_known_variables_only() {
        let vars = HashMap::from([("a", "1".to_owned()), ("b", "two".to_owned())]);
        assert_eq!(substitute("-Dx=${a}/${b}", &vars), "-Dx=1/two");
        assert_eq!(substitute("${missing} ${a}", &vars), "${missing} 1");
        assert_eq!(substitute("${unterminated", &vars), "${unterminated");
    }

    #[test]
    fn builds_legacy_arguments() -> Result<(), serde_json::Error> {
        let version: VersionJson = serde_json::from_str(
            r#"{"id":"1.7.10","mainClass":"net.minecraft.client.main.Main","type":"release","assets":"1.7.10",
                "minecraftArguments":"--username ${auth_player_name} --version ${version_name} --uuid ${auth_uuid}",
                "assetIndex":{"id":"1.7.10","sha1":"s","size":1,"url":"u"},
                "downloads":{"client":{"sha1":"s","size":1,"url":"u"}}}"#,
        )?;
        let env = Environment {
            os: OsName::Linux,
            arch: "x86_64",
            features: Features::default(),
        };
        let settings = crate::settings::InstanceSettings::default().resolve(&Default::default());
        let account = Account {
            id: "abc".into(),
            username: "Steve".into(),
            kind: AccountKind::Offline,
        };
        let classpath = [PathBuf::from("/lib/a.jar"), PathBuf::from("/client.jar")];
        let dir = Path::new("/x");
        let args = build_arguments(&ArgsInput {
            version: &version,
            env: &env,
            settings: &settings,
            account: &account,
            classpath: &classpath,
            game_dir: dir,
            natives_dir: Path::new("/natives"),
            assets_dir: dir,
            legacy_assets_dir: dir,
            libraries_dir: dir,
        });
        assert_eq!(
            args,
            [
                "-Xms512M",
                "-Xmx4096M",
                "-Djava.library.path=/natives",
                "-cp",
                "/lib/a.jar:/client.jar",
                "net.minecraft.client.main.Main",
                "--username",
                "Steve",
                "--version",
                "1.7.10",
                "--uuid",
                "abc",
            ]
        );
        Ok(())
    }
}
