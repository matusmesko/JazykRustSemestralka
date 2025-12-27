use config::{Config, ConfigError, File};
use serde::Deserialize;


#[derive(Debug, Deserialize, Default, Clone)]
pub struct CorsSettings {
    pub enabled: bool,
    pub allowed_origins: Option<Vec<String>>,
    pub allowed_methods: Option<Vec<String>>,
    pub allowed_headers: Option<Vec<String>>,
    pub allow_credentials: Option<bool>,
    pub max_age: Option<u64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub port: u16,
    pub database_url: String,
    pub cors: Option<CorsSettings>,
}

impl Settings {
    pub fn load() -> Result<Self, ConfigError> {
        let config = Config::builder()
            .add_source(File::with_name("config.toml"))
            .build()?;
        config.try_deserialize()
    }
}