use jelauncher_core::launch::QuickPlay;
use serde::Serialize;

/// What the launcher was asked to do on its command line.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, specta::Type, tauri_specta::Event)]
#[serde(rename_all = "camelCase")]
pub struct CliRequest {
    /// Instance id to launch.
    pub launch: Option<String>,
    pub quick_play: Option<QuickPlay>,
    /// Archive path or URL to import.
    pub import: Option<String>,
}

pub const USAGE: &str = "usage: jelauncher [--launch <instance> [--server <address> | --world <folder>]] [--import <file or url>]";

impl CliRequest {
    /// Parses the arguments after the program name. Returns `None` when there is nothing to do.
    pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Option<Self>, String> {
        let mut request = Self::default();
        let mut args = args.into_iter();
        while let Some(flag) = args.next() {
            let mut value = || {
                args.next()
                    .ok_or_else(|| format!("{flag} needs a value; {USAGE}"))
            };
            match flag.as_str() {
                "-l" | "--launch" => request.launch = Some(value()?),
                "-s" | "--server" => request.quick_play = Some(QuickPlay::Server(value()?)),
                "-w" | "--world" => request.quick_play = Some(QuickPlay::World(value()?)),
                "-I" | "--import" => request.import = Some(value()?),
                "--show" => {}
                // macOS adds a process serial number when apps are opened from Finder.
                other if other.starts_with("-psn_") => {}
                other => return Err(format!("unknown argument {other}; {USAGE}")),
            }
        }
        if request.quick_play.is_some() && request.launch.is_none() {
            return Err(format!("--server and --world need --launch; {USAGE}"));
        }
        Ok((request != Self::default()).then_some(request))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Option<CliRequest>, String> {
        CliRequest::parse(args.iter().map(|arg| (*arg).to_owned()))
    }

    #[test]
    fn parses_launch_and_quick_play() {
        assert_eq!(
            parse(&["--launch", "survival", "--server", "mc.example.com"]),
            Ok(Some(CliRequest {
                launch: Some("survival".into()),
                quick_play: Some(QuickPlay::Server("mc.example.com".into())),
                import: None,
            }))
        );
        assert_eq!(parse(&["--show"]), Ok(None));
        assert_eq!(parse(&[]), Ok(None));
    }

    #[test]
    fn rejects_incomplete_arguments() {
        assert!(parse(&["--launch"]).is_err());
        assert!(parse(&["--world", "New World"]).is_err());
        assert!(parse(&["--bogus"]).is_err());
    }
}
