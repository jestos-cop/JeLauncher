//! Microsoft account sign-in: device code → Microsoft OAuth → Xbox Live → XSTS → Minecraft.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::{Account, AccountKind};
use crate::net::HttpClient;
use crate::{Error, Result};

const DEVICE_CODE_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode";
const TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
const XBOX_AUTH_URL: &str = "https://user.auth.xboxlive.com/user/authenticate";
const XSTS_URL: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";
const MINECRAFT_LOGIN_URL: &str =
    "https://api.minecraftservices.com/authentication/login_with_xbox";
const PROFILE_URL: &str = "https://api.minecraftservices.com/minecraft/profile";
const ENTITLEMENTS_URL: &str = "https://api.minecraftservices.com/entitlements/mcstore";
const SCOPE: &str = "XboxLive.signin offline_access";
const DEVICE_CODE_GRANT: &str = "urn:ietf:params:oauth:grant-type:device_code";
/// Tokens are refreshed this long before they expire, so a game never starts with a stale one.
const REFRESH_MARGIN_SECS: i64 = 300;

/// Tokens of a signed-in Microsoft account; stored with the account, never shown in the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MicrosoftSession {
    pub refresh_token: String,
    pub access_token: String,
    /// Unix time in seconds.
    pub expires_at: i64,
}

/// What the user needs to finish signing in on another device or in the browser.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCodePrompt {
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u32,
}

#[derive(Debug, Clone)]
pub struct DeviceCode {
    pub prompt: DeviceCodePrompt,
    device_code: String,
    interval: u64,
}

pub struct MicrosoftAuth<'a> {
    pub http: &'a HttpClient,
    pub client_id: &'a str,
}

impl MicrosoftAuth<'_> {
    pub async fn request_device_code(&self) -> Result<DeviceCode> {
        let response = self
            .form(
                DEVICE_CODE_URL,
                &[("client_id", self.client_id), ("scope", SCOPE)],
            )
            .await?;
        Ok(DeviceCode {
            prompt: DeviceCodePrompt {
                user_code: string(&response, "user_code")?,
                verification_uri: string(&response, "verification_uri")?,
                expires_in: response["expires_in"]
                    .as_u64()
                    .map_or(900, |secs| secs as u32),
            },
            device_code: string(&response, "device_code")?,
            interval: response["interval"].as_u64().unwrap_or(5),
        })
    }

    /// Polls until the user approves the code, then signs in to Minecraft.
    /// `is_cancelled` is checked between polls.
    pub async fn wait_for_login(
        &self,
        code: &DeviceCode,
        is_cancelled: impl Fn() -> bool,
    ) -> Result<Account> {
        let mut interval = code.interval;
        loop {
            tokio::time::sleep(Duration::from_secs(interval)).await;
            if is_cancelled() {
                return Err(Error::InvalidInput("sign-in was cancelled".into()));
            }
            let (status, response) = self
                .http
                .post_form_with_status(
                    TOKEN_URL,
                    &[
                        ("grant_type", DEVICE_CODE_GRANT),
                        ("client_id", self.client_id),
                        ("device_code", &code.device_code),
                    ],
                )
                .await?;
            if status == 200 {
                return self.sign_in_to_minecraft(&response).await;
            }
            match response["error"].as_str() {
                Some("authorization_pending") => {}
                Some("slow_down") => interval += 5,
                Some("expired_token") => {
                    return Err(Error::InvalidInput(
                        "the sign-in code expired; try again".into(),
                    ));
                }
                Some("authorization_declined") => {
                    return Err(Error::InvalidInput("sign-in was declined".into()));
                }
                _ => return Err(oauth_error(&response)),
            }
        }
    }

    /// Exchanges the stored refresh token for fresh tokens and re-reads the profile.
    pub async fn refresh(&self, account: &Account) -> Result<Account> {
        let session = account.microsoft.as_ref().ok_or_else(|| {
            Error::InvalidInput(format!("{} is not a Microsoft account", account.username))
        })?;
        let response = self
            .form(
                TOKEN_URL,
                &[
                    ("grant_type", "refresh_token"),
                    ("client_id", self.client_id),
                    ("refresh_token", &session.refresh_token),
                    ("scope", SCOPE),
                ],
            )
            .await
            .map_err(|err| {
                Error::InvalidInput(format!(
                    "signing in {} again failed ({err}); remove and add the account",
                    account.username
                ))
            })?;
        self.sign_in_to_minecraft(&response).await
    }

    async fn sign_in_to_minecraft(&self, oauth: &Value) -> Result<Account> {
        let microsoft_token = string(oauth, "access_token")?;
        let refresh_token = string(oauth, "refresh_token")?;

        let xbox = self
            .json(
                XBOX_AUTH_URL,
                &json!({
                    "Properties": {
                        "AuthMethod": "RPS",
                        "SiteName": "user.auth.xboxlive.com",
                        "RpsTicket": format!("d={microsoft_token}"),
                    },
                    "RelyingParty": "http://auth.xboxlive.com",
                    "TokenType": "JWT",
                }),
                "Xbox Live",
            )
            .await?;
        let (status, xsts) = self
            .http
            .post_json_with_status(
                XSTS_URL,
                &json!({
                    "Properties": { "SandboxId": "RETAIL", "UserTokens": [string(&xbox, "Token")?] },
                    "RelyingParty": "rp://api.minecraftservices.com/",
                    "TokenType": "JWT",
                }),
            )
            .await?;
        if status != 200 {
            return Err(xsts_error(&xsts));
        }
        let user_hash = xsts["DisplayClaims"]["xui"][0]["uhs"]
            .as_str()
            .ok_or_else(|| Error::InvalidInput("Xbox Live returned no user hash".into()))?;

        let (status, minecraft) = self
            .http
            .post_json_with_status(
                MINECRAFT_LOGIN_URL,
                &json!({ "identityToken": format!("XBL3.0 x={user_hash};{}", string(&xsts, "Token")?) }),
            )
            .await?;
        match status {
            200 => {}
            403 => {
                return Err(Error::InvalidInput(
                    "Minecraft rejected this launcher's app registration; it may still be awaiting approval by Mojang"
                        .into(),
                ));
            }
            _ => {
                return Err(Error::InvalidInput(format!(
                    "Minecraft sign-in failed with status {status}"
                )));
            }
        }
        let access_token = string(&minecraft, "access_token")?;
        let expires_in = minecraft["expires_in"].as_i64().unwrap_or(86_400);

        self.check_ownership(&access_token).await?;
        let (status, profile) = self
            .http
            .get_bearer_with_status(PROFILE_URL, &access_token)
            .await?;
        if status == 404 {
            return Err(Error::InvalidInput(
                "this Microsoft account does not own Minecraft: Java Edition or has no profile yet"
                    .into(),
            ));
        }
        if status != 200 {
            return Err(Error::InvalidInput(format!(
                "reading the Minecraft profile failed with status {status}"
            )));
        }
        Ok(Account {
            id: string(&profile, "id")?,
            username: string(&profile, "name")?,
            kind: AccountKind::Microsoft,
            microsoft: Some(MicrosoftSession {
                refresh_token,
                access_token,
                expires_at: unix_now() + expires_in,
            }),
        })
    }

    /// Fails unless the account owns the game, bought or through Game Pass.
    async fn check_ownership(&self, access_token: &str) -> Result<()> {
        let (status, entitlements) = self
            .http
            .get_bearer_with_status(ENTITLEMENTS_URL, access_token)
            .await?;
        if status != 200 {
            return Err(Error::InvalidInput(format!(
                "checking game ownership failed with status {status}"
            )));
        }
        if owns_java_edition(&entitlements) {
            Ok(())
        } else {
            Err(Error::InvalidInput(
                "this Microsoft account does not own Minecraft: Java Edition".into(),
            ))
        }
    }

    async fn form(&self, url: &str, form: &[(&str, &str)]) -> Result<Value> {
        let (status, response) = self.http.post_form_with_status(url, form).await?;
        if status == 200 {
            Ok(response)
        } else {
            Err(oauth_error(&response))
        }
    }

    async fn json(&self, url: &str, body: &Value, service: &str) -> Result<Value> {
        let (status, response) = self.http.post_json_with_status(url, body).await?;
        if status == 200 {
            Ok(response)
        } else {
            Err(Error::InvalidInput(format!(
                "{service} sign-in failed with status {status}"
            )))
        }
    }
}

/// Whether the account's Minecraft token is missing or about to expire.
pub fn needs_refresh(account: &Account) -> bool {
    account
        .microsoft
        .as_ref()
        .is_some_and(|session| session.expires_at - REFRESH_MARGIN_SECS <= unix_now())
}

fn owns_java_edition(entitlements: &Value) -> bool {
    entitlements["items"].as_array().is_some_and(|items| {
        items.iter().any(|item| {
            matches!(
                item["name"].as_str(),
                Some("game_minecraft" | "product_minecraft")
            )
        })
    })
}

fn string(value: &Value, key: &str) -> Result<String> {
    value[key]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| Error::InvalidInput(format!("sign-in response is missing {key}")))
}

fn oauth_error(response: &Value) -> Error {
    let message = response["error_description"]
        .as_str()
        .or(response["error"].as_str())
        .unwrap_or("unexpected response");
    Error::InvalidInput(format!(
        "Microsoft sign-in failed: {}",
        message.lines().next().unwrap_or(message)
    ))
}

fn xsts_error(response: &Value) -> Error {
    let message = match response["XErr"].as_u64() {
        Some(2_148_916_233) => {
            "this Microsoft account has no Xbox profile yet; sign in once at xbox.com"
        }
        Some(2_148_916_235) => "Xbox Live is not available in this account's country",
        Some(2_148_916_236 | 2_148_916_237) => "this account needs adult verification on xbox.com",
        Some(2_148_916_238) => {
            "this is a child account; an adult has to add it to a Microsoft family"
        }
        _ => "Xbox Live authorization failed",
    };
    Error::InvalidInput(message.into())
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| {
            i64::try_from(elapsed.as_secs()).unwrap_or(i64::MAX)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_java_edition_entitlements() {
        let owned =
            json!({ "items": [{ "name": "product_minecraft" }, { "name": "game_minecraft" }] });
        let bedrock_only = json!({ "items": [{ "name": "product_minecraft_bedrock" }] });
        assert!(owns_java_edition(&owned));
        assert!(!owns_java_edition(&bedrock_only));
        assert!(!owns_java_edition(&json!({ "items": [] })));
    }

    #[test]
    fn explains_xbox_errors() {
        let child = xsts_error(&json!({ "XErr": 2_148_916_238_u64 })).to_string();
        assert!(child.contains("child account"));
    }

    #[test]
    fn refreshes_shortly_before_expiry() {
        let account = |expires_at| Account {
            id: "id".into(),
            username: "Steve".into(),
            kind: AccountKind::Microsoft,
            microsoft: Some(MicrosoftSession {
                refresh_token: "r".into(),
                access_token: "a".into(),
                expires_at,
            }),
        };
        assert!(needs_refresh(&account(unix_now() + 60)));
        assert!(!needs_refresh(&account(unix_now() + 3_600)));
    }
}
