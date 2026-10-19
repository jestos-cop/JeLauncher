//! Desktop shortcuts that start an instance through the launcher's `--launch` argument.

use std::path::{Path, PathBuf};

use crate::instance::Instance;
use crate::{Error, Result};

/// Creates a shortcut for `instance` in `dir` that runs `launcher_exe --launch <id>`.
/// Returns the shortcut's path.
pub async fn create_shortcut(
    instance: &Instance,
    launcher_exe: &Path,
    dir: &Path,
) -> Result<PathBuf> {
    std::fs::create_dir_all(dir).map_err(|err| Error::io(dir, err))?;
    let name = file_name(&instance.config.name);
    platform::create(&name, &instance.id, launcher_exe, dir).await
}

/// The instance name without characters that file systems reject.
fn file_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| {
            if r#"/\:*?"<>|"#.contains(c) || c.is_control() {
                '_'
            } else {
                c
            }
        })
        .collect();
    let cleaned = cleaned.trim().trim_matches('.').to_owned();
    if cleaned.is_empty() {
        "Minecraft".into()
    } else {
        cleaned
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};

    use crate::{Error, Result, fs};

    /// A tiny app bundle whose executable is a script that hands off to the launcher.
    pub async fn create(
        name: &str,
        instance_id: &str,
        launcher_exe: &Path,
        dir: &Path,
    ) -> Result<PathBuf> {
        let bundle = dir.join(format!("{name}.app"));
        let macos = bundle.join("Contents").join("MacOS");
        std::fs::create_dir_all(&macos).map_err(|err| Error::io(&macos, err))?;
        let script = macos.join("launch");
        let exe = shell_quote(&launcher_exe.to_string_lossy());
        fs::write_atomic(
            &script,
            format!(
                "#!/bin/sh\nexec {exe} --launch {}\n",
                shell_quote(instance_id)
            )
            .as_bytes(),
        )?;
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755))
            .map_err(|err| Error::io(&script, err))?;
        let plist = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleExecutable</key><string>launch</string>
  <key>CFBundleName</key><string>{}</string>
  <key>CFBundleIdentifier</key><string>com.jestos.jelauncher.instance.{}</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>LSUIElement</key><true/>
</dict>
</plist>
"#,
            xml_escape(name),
            xml_escape(&instance_id.replace(|c: char| !c.is_ascii_alphanumeric(), "-")),
        );
        fs::write_atomic(
            &bundle.join("Contents").join("Info.plist"),
            plist.as_bytes(),
        )?;
        Ok(bundle)
    }

    fn shell_quote(value: &str) -> String {
        format!("'{}'", value.replace('\'', r"'\''"))
    }

    fn xml_escape(value: &str) -> String {
        value
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
mod platform {
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};

    use crate::{Error, Result, fs};

    pub async fn create(
        name: &str,
        instance_id: &str,
        launcher_exe: &Path,
        dir: &Path,
    ) -> Result<PathBuf> {
        let path = dir.join(format!("{name}.desktop"));
        let entry = format!(
            "[Desktop Entry]\nType=Application\nName={name}\nExec={} --launch {}\nTerminal=false\nCategories=Game;\n",
            exec_quote(&launcher_exe.to_string_lossy()),
            exec_quote(instance_id),
        );
        fs::write_atomic(&path, entry.as_bytes())?;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
            .map_err(|err| Error::io(&path, err))?;
        Ok(path)
    }

    /// Quoting rules of the Desktop Entry `Exec` key.
    fn exec_quote(value: &str) -> String {
        let escaped: String = value
            .chars()
            .flat_map(|c| match c {
                '"' | '`' | '$' | '\\' => vec!['\\', c],
                '%' => vec!['%', '%'],
                _ => vec![c],
            })
            .collect();
        format!("\"{escaped}\"")
    }
}

#[cfg(windows)]
mod platform {
    use std::path::{Path, PathBuf};

    use tokio::process::Command;

    use crate::{Error, Result};

    /// Windows shortcuts are binary files, so PowerShell's WScript.Shell writes them.
    pub async fn create(
        name: &str,
        instance_id: &str,
        launcher_exe: &Path,
        dir: &Path,
    ) -> Result<PathBuf> {
        let path = dir.join(format!("{name}.lnk"));
        let quote = |value: &str| format!("'{}'", value.replace('\'', "''"));
        let script = format!(
            "$s = (New-Object -ComObject WScript.Shell).CreateShortcut({}); $s.TargetPath = {}; $s.Arguments = {}; $s.Save()",
            quote(&path.to_string_lossy()),
            quote(&launcher_exe.to_string_lossy()),
            quote(&format!("--launch \"{instance_id}\"")),
        );
        let status = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .status()
            .await
            .map_err(|err| Error::io("powershell", err))?;
        if status.success() {
            Ok(path)
        } else {
            Err(Error::InvalidInput(format!(
                "creating the shortcut failed with {status}"
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleans_names_for_file_systems() {
        assert_eq!(file_name("Survival: 1.21/beta"), "Survival_ 1.21_beta");
        assert_eq!(file_name(" .. "), "Minecraft");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn creates_a_shortcut_that_runs_the_launcher() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|err| Error::io("tempdir", err))?;
        let instance = crate::instance::create_instance(
            &dir.path().join("instances"),
            crate::instance::NewInstance {
                name: "It's Modded".into(),
                minecraft_version: "1.21.4".into(),
                loader: None,
                group: None,
                icon_key: None,
            },
        )?;
        let shortcut = create_shortcut(
            &instance,
            Path::new("/opt/Je Launcher/jelauncher"),
            dir.path(),
        )
        .await?;
        let launcher = if cfg!(target_os = "macos") {
            shortcut.join("Contents/MacOS/launch")
        } else {
            shortcut.clone()
        };
        let contents =
            std::fs::read_to_string(&launcher).map_err(|err| Error::io(&launcher, err))?;
        assert!(contents.contains("--launch"));
        assert!(contents.contains(&instance.id));
        assert!(contents.contains("Je Launcher"));
        Ok(())
    }
}
