use std::path::Path;

use directories::BaseDirs;

use crate::net::HttpClient;
use crate::{Error, Result};

const MCLOGS_URL: &str = "https://api.mclo.gs/1/log";

/// Uploads a log to mclo.gs and returns its public URL. The user's home folder is
/// replaced by `~` first, so the account name in file paths is not published.
pub async fn upload_log(http: &HttpClient, content: &str) -> Result<String> {
    let home = BaseDirs::new().map(|dirs| dirs.home_dir().to_path_buf());
    let redacted = redact_home(content, home.as_deref());
    let (status, response) = http
        .post_form_with_status(MCLOGS_URL, &[("content", &redacted)])
        .await?;
    match response["url"].as_str() {
        Some(url) if status == 200 && response["success"] == true => Ok(url.to_owned()),
        _ => Err(Error::InvalidInput(format!(
            "uploading the log failed: {}",
            response["error"].as_str().unwrap_or("unexpected response")
        ))),
    }
}

fn redact_home(content: &str, home: Option<&Path>) -> String {
    match home.and_then(Path::to_str).filter(|home| home.len() > 1) {
        Some(home) => content.replace(home, "~"),
        None => content.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hides_the_home_folder() {
        let log = "Loading /Users/steve/Library/JeLauncher/mods/a.jar";
        assert_eq!(
            redact_home(log, Some(Path::new("/Users/steve"))),
            "Loading ~/Library/JeLauncher/mods/a.jar"
        );
        assert_eq!(redact_home(log, None), log);
    }
}
