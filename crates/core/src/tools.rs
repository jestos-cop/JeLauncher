//! External programs the launcher hands an instance to: profilers and a text editor.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde::{Deserialize, Serialize};

use crate::settings::ToolSettings;
use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum Profiler {
    JProfiler,
    VisualVm,
}

impl Profiler {
    /// The program that attaches this profiler, if its configured installation looks usable.
    pub fn check(self, settings: &ToolSettings) -> Result<PathBuf> {
        match self {
            Self::JProfiler => {
                let dir = settings.jprofiler_path.as_deref().ok_or_else(|| {
                    Error::InvalidInput("set the JProfiler folder in the settings".into())
                })?;
                jprofiler_attach_program(Path::new(dir))
            }
            Self::VisualVm => match settings.visualvm_path.as_deref() {
                Some(path) if Path::new(path).is_file() => Ok(PathBuf::from(path)),
                Some(path) => Err(Error::InvalidInput(format!("{path} does not exist"))),
                None => find_on_path(&["jvisualvm", "visualvm"]).ok_or_else(|| {
                    Error::InvalidInput(
                        "VisualVM was not found; set its path in the settings".into(),
                    )
                }),
            },
        }
    }

    /// Starts the profiler attached to the game process `pid`; it keeps running on its own.
    pub fn attach(self, settings: &ToolSettings, pid: u32) -> Result<()> {
        let program = self.check(settings)?;
        let args = match self {
            Self::JProfiler => vec![
                "-d".to_owned(),
                pid.to_string(),
                "--gui".to_owned(),
                "-p".to_owned(),
                settings.jprofiler_port.to_string(),
            ],
            Self::VisualVm => vec!["--openpid".to_owned(), pid.to_string()],
        };
        spawn_detached(&program, &args)
    }
}

/// `bin/jpenable` of a JProfiler installation directory.
fn jprofiler_attach_program(dir: &Path) -> Result<PathBuf> {
    let bin = dir.join("bin");
    let program = bin.join(if cfg!(windows) {
        "jpenable.exe"
    } else {
        "jpenable"
    });
    if program.is_file() && bin.join("agent.jar").is_file() {
        Ok(program)
    } else {
        Err(Error::InvalidInput(format!(
            "{} is not a JProfiler installation",
            dir.display()
        )))
    }
}

/// Opens `file` with the configured editor command, e.g. `code --wait`.
pub fn open_in_editor(editor: &str, file: &Path) -> Result<()> {
    let mut parts = shlex::split(editor)
        .filter(|parts| !parts.is_empty())
        .ok_or_else(|| Error::InvalidInput(format!("cannot parse editor command: {editor}")))?;
    let program = PathBuf::from(parts.remove(0));
    parts.push(file.to_string_lossy().into_owned());
    spawn_detached(&program, &parts)
}

fn spawn_detached(program: &Path, args: &[String]) -> Result<()> {
    Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(drop)
        .map_err(|err| Error::io(program, err))
}

fn find_on_path(names: &[&str]) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .flat_map(|dir| {
            names.iter().map(move |name| {
                dir.join(if cfg!(windows) {
                    format!("{name}.exe")
                } else {
                    (*name).to_owned()
                })
            })
        })
        .find(|candidate| candidate.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jprofiler_needs_its_attach_program_and_agent() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|err| Error::io("tempdir", err))?;
        let settings = ToolSettings {
            jprofiler_path: Some(dir.path().to_string_lossy().into_owned()),
            ..Default::default()
        };
        assert!(Profiler::JProfiler.check(&settings).is_err());

        let bin = dir.path().join("bin");
        std::fs::create_dir(&bin).map_err(|err| Error::io(&bin, err))?;
        let program = bin.join(if cfg!(windows) {
            "jpenable.exe"
        } else {
            "jpenable"
        });
        for file in [&program, &bin.join("agent.jar")] {
            std::fs::write(file, "").map_err(|err| Error::io(file, err))?;
        }
        assert_eq!(Profiler::JProfiler.check(&settings)?, program);
        Ok(())
    }

    #[test]
    fn missing_visualvm_path_is_rejected() {
        let settings = ToolSettings {
            visualvm_path: Some("/nonexistent/visualvm".into()),
            ..Default::default()
        };
        assert!(Profiler::VisualVm.check(&settings).is_err());
    }
}
