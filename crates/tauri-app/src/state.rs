use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::{Mutex, RwLock};

use jelauncher_core::auth::{AccountStore, DeviceCode};
use jelauncher_core::meta::MetaService;
use jelauncher_core::modplatform::{ModPlatforms, PlatformConfig};
use jelauncher_core::net::HttpClient;
use jelauncher_core::paths::DataPaths;
use jelauncher_core::settings::GlobalSettings;
use tokio::sync::oneshot;

use crate::cli::CliRequest;

pub struct AppState {
    pub paths: DataPaths,
    pub settings: RwLock<GlobalSettings>,
    pub accounts: RwLock<AccountStore>,
    pub http: HttpClient,
    pub meta: MetaService,
    pub platforms: ModPlatforms,
    /// Azure application id used for Microsoft sign-in: the user's own or the built-in one.
    pub microsoft_client_id: Option<String>,
    /// The device code of the Microsoft sign-in in progress, tagged with its attempt number.
    pub pending_login: Mutex<Option<(u64, DeviceCode)>>,
    /// Bumped by every new or cancelled sign-in so an older attempt stops polling.
    pub login_attempt: AtomicU64,
    /// Command line request waiting for the UI to pick it up after startup.
    pub pending_cli: Mutex<Option<CliRequest>>,
    /// Instances being prepared or running, with a channel that cancels the launch or kills the game.
    pub running: Mutex<HashMap<String, Option<oneshot::Sender<()>>>>,
    /// Cancels the modpack install or import in progress.
    pub modpack_cancel: Mutex<Option<oneshot::Sender<()>>>,
}

impl AppState {
    pub fn load() -> jelauncher_core::Result<Self> {
        let paths = DataPaths::detect()?;
        let settings = GlobalSettings::load(&paths.settings_file())?;
        let accounts = AccountStore::load(&paths.accounts_file())?;
        let http = HttpClient::new(&settings.network)?;
        let meta = MetaService::new(http.clone(), paths.meta());
        let apis = &settings.apis;
        let platforms = ModPlatforms::new(
            http.clone(),
            PlatformConfig {
                curseforge_api_key: configured_or_built_in(
                    &apis.curseforge_api_key,
                    option_env!("CURSEFORGE_API_KEY"),
                ),
                modrinth_token: apis.modrinth_token.clone(),
                technic_client_id: apis.technic_client_id.clone(),
                modrinth_fallback_for_blocked: apis.modrinth_fallback_for_blocked,
            },
        );
        let microsoft_client_id = configured_or_built_in(
            &apis.microsoft_client_id,
            option_env!("MICROSOFT_CLIENT_ID"),
        );
        Ok(Self {
            paths,
            settings: RwLock::new(settings),
            accounts: RwLock::new(accounts),
            http,
            meta,
            platforms,
            microsoft_client_id,
            pending_login: Mutex::default(),
            pending_cli: Mutex::default(),
            login_attempt: AtomicU64::default(),
            running: Mutex::default(),
            modpack_cancel: Mutex::default(),
        })
    }
}

/// The value from the settings, or the one baked into this build when the setting is empty.
fn configured_or_built_in(configured: &Option<String>, built_in: Option<&str>) -> Option<String> {
    configured
        .clone()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            built_in
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
        })
}
