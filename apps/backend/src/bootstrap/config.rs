use anyhow::Result;
use figment::{providers::Env, Figment};
use serde::{Deserialize, Deserializer};
use std::str::FromStr;

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
    #[serde(default = "default_cookie_secure")]
    pub cookie_secure: bool,
    #[serde(default)]
    pub cookie_samesite: SameSiteCfg,
    #[serde(default)]
    pub cookie_domain: Option<String>,
    #[serde(default, deserialize_with = "deserialize_csv")]
    pub cors_allowed_origins: Vec<String>,
}

/// Split an `Option<String>` on commas, trim whitespace, and drop empty entries.
/// Accepts an absent field (→ empty Vec) or a comma-separated string.
fn split_csv(raw: Option<String>) -> Vec<String> {
    match raw {
        None => Vec::new(),
        Some(s) => s
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect(),
    }
}

fn deserialize_csv<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = Option::<String>::deserialize(deserializer)?;
    Ok(split_csv(raw))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SameSiteCfg {
    #[default]
    Strict,
    Lax,
    None,
}

impl FromStr for SameSiteCfg {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "strict" => Ok(Self::Strict),
            "lax" => Ok(Self::Lax),
            "none" => Ok(Self::None),
            other => Err(format!("invalid COOKIE_SAMESITE: {other}")),
        }
    }
}

fn default_port() -> u16 {
    3000
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
fn default_cookie_secure() -> bool {
    true
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn samesite_parses_case_insensitive() {
        assert_eq!(
            "strict".parse::<SameSiteCfg>().unwrap(),
            SameSiteCfg::Strict
        );
        assert_eq!("LAX".parse::<SameSiteCfg>().unwrap(), SameSiteCfg::Lax);
        assert_eq!("None".parse::<SameSiteCfg>().unwrap(), SameSiteCfg::None);
        assert!("bogus".parse::<SameSiteCfg>().is_err());
    }

    #[test]
    fn cors_csv_splits_and_trims() {
        let result = split_csv(Some("https://veyra.dev, https://app.veyra.dev".to_owned()));
        assert_eq!(result, vec!["https://veyra.dev", "https://app.veyra.dev"]);
    }

    #[test]
    fn cors_csv_single_origin() {
        let result = split_csv(Some("https://veyra.dev".to_owned()));
        assert_eq!(result, vec!["https://veyra.dev"]);
    }

    #[test]
    fn cors_csv_none_yields_empty_vec() {
        assert_eq!(split_csv(None), Vec::<String>::new());
    }

    #[test]
    fn cors_csv_empty_string_yields_empty_vec() {
        assert_eq!(split_csv(Some(String::new())), Vec::<String>::new());
    }

    #[test]
    fn cors_csv_drops_blank_entries() {
        let result = split_csv(Some("https://a.dev,,  ,https://b.dev".to_owned()));
        assert_eq!(result, vec!["https://a.dev", "https://b.dev"]);
    }
}
