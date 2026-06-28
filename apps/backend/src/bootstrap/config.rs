use anyhow::Result;
use figment::{providers::Env, Figment};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_access_ttl")]
    pub access_ttl_secs: u64,
    #[serde(default = "default_refresh_ttl")]
    pub refresh_ttl_secs: u64,
    #[serde(default = "default_refresh_grace")]
    pub refresh_grace_secs: u64,
}

fn default_port() -> u16 {
    8080
}
fn default_access_ttl() -> u64 {
    900
}
fn default_refresh_ttl() -> u64 {
    604_800
}
fn default_refresh_grace() -> u64 {
    10
}

impl Config {
    pub fn load() -> Result<Self> {
        dotenvy::dotenv().ok();
        let config: Self = Figment::new().merge(Env::raw()).extract()?;
        anyhow::ensure!(
            config.jwt_secret.len() >= 32,
            "JWT_SECRET must be at least 32 bytes (got {})",
            config.jwt_secret.len()
        );
        anyhow::ensure!(!config.redis_url.is_empty(), "REDIS_URL must be set");
        Ok(config)
    }
}
