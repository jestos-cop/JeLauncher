use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::{Mutex, RwLock};

use jelauncher_core::auth::{AccountStore, DeviceCode};
use jelauncher_core::meta::MetaService;
use jelauncher_core::modplatform::ModPlatforms;
use jelauncher_core::net::HttpClient;
use jelauncher_core::paths::DataPaths;
use jelauncher_core::settings::GlobalSettings;
use tokio::sync::oneshot;

pub struct AppState {
    pub paths: DataPaths,
    pub settings: RwLock<GlobalSettings>,
    pub accounts: RwLock<AccountStore>,
    pub http: HttpClient,
    pub meta: MetaService,
    pub platforms: ModPlatforms,
    /// Azure application id used for Microsoft sign-in, set at build time.
    pub microsoft_client_id: Option<&'static str>,
    /// The device code of the Microsoft sign-in in progress, tagged with its attempt number.
    pub pending_login: Mutex<Option<(u64, DeviceCode)>>,
    /// Bumped by every new or cancelled sign-in so an older attempt stops polling.
    pub login_attempt: AtomicU64,
    /// Instances being prepared (`None`) or running, with a channel that kills the game.
    pub running: Mutex<HashMap<String, Option<oneshot::Sender<()>>>>,
}

impl AppState {
    pub fn load() -> jelauncher_core::Result<Self> {
        let paths = DataPaths::platform_default()?;
        let settings = GlobalSettings::load(&paths.settings_file())?;
        let accounts = AccountStore::load(&paths.accounts_file())?;
        let http = HttpClient::new(&settings.network)?;
        let meta = MetaService::new(http.clone(), paths.meta());
        let platforms = ModPlatforms::new(
            http.clone(),
            option_env!("CURSEFORGE_API_KEY").map(str::to_owned),
        );
        Ok(Self {
            paths,
            settings: RwLock::new(settings),
            accounts: RwLock::new(accounts),
            http,
            meta,
            platforms,
            microsoft_client_id: option_env!("MICROSOFT_CLIENT_ID").filter(|id| !id.is_empty()),
            pending_login: Mutex::default(),
            login_attempt: AtomicU64::default(),
            running: Mutex::default(),
        })
    }
}
