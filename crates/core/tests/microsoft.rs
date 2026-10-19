//! Talks to the real Microsoft endpoints; needs `MICROSOFT_CLIENT_ID`.
//! Run with `cargo test --test microsoft -- --ignored --nocapture`.

use jelauncher_core::auth::MicrosoftAuth;
use jelauncher_core::net::HttpClient;
use jelauncher_core::settings::NetworkSettings;

#[tokio::test]
#[ignore = "requires network access and a client id"]
async fn requests_a_device_code() -> jelauncher_core::Result<()> {
    let Ok(client_id) = std::env::var("MICROSOFT_CLIENT_ID") else {
        panic!("MICROSOFT_CLIENT_ID is not set");
    };
    let http = HttpClient::new(&NetworkSettings::default())?;
    let auth = MicrosoftAuth {
        http: &http,
        client_id: &client_id,
    };
    let code = auth.request_device_code().await?;
    println!(
        "{} at {}",
        code.prompt.user_code, code.prompt.verification_uri
    );
    assert!(!code.prompt.user_code.is_empty());
    Ok(())
}

#[tokio::test]
#[ignore = "interactive: prints a code that has to be approved in the browser"]
async fn signs_in_interactively() -> jelauncher_core::Result<()> {
    let Ok(client_id) = std::env::var("MICROSOFT_CLIENT_ID") else {
        panic!("MICROSOFT_CLIENT_ID is not set");
    };
    let http = HttpClient::new(&NetworkSettings::default())?;
    let auth = MicrosoftAuth {
        http: &http,
        client_id: &client_id,
    };
    let code = auth.request_device_code().await?;
    println!(
        "CODE {} at {}",
        code.prompt.user_code, code.prompt.verification_uri
    );
    let account = auth.wait_for_login(&code, || false).await?;
    println!("signed in as {} ({})", account.username, account.id);
    let refreshed = auth.refresh(&account).await?;
    assert_eq!(refreshed.id, account.id);
    println!("refresh works");
    Ok(())
}
