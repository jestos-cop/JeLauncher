use std::sync::RwLock;

use jelauncher_core::auth::AccountStore;
use jelauncher_core::meta::MetaService;
use jelauncher_core::net::HttpClient;
use jelauncher_core::paths::DataPaths;
use jelauncher_core::settings::GlobalSettings;

pub struct AppState {
    pub paths: DataPaths,
    pub settings: RwLock<GlobalSettings>,
    pub accounts: RwLock<AccountStore>,
    pub meta: MetaService,
}

impl AppState {
    pub fn load() -> jelauncher_core::Result<Self> {
        let paths = DataPaths::platform_default()?;
        let settings = GlobalSettings::load(&paths.settings_file())?;
        let accounts = AccountStore::load(&paths.accounts_file())?;
        let http = HttpClient::new(&settings.network)?;
        let meta = MetaService::new(http, paths.meta());
        Ok(Self {
            paths,
            settings: RwLock::new(settings),
            accounts: RwLock::new(accounts),
            meta,
        })
    }
}
