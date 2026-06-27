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

/// Validates that a `SameSite=None` cookie is only configured alongside `Secure=true`.
///
/// Browsers reject `SameSite=None` cookies that lack the `Secure` attribute; this
/// guard surfaces the misconfiguration at startup instead of silently producing
/// cookies that every browser will drop.
pub fn validate_cookie_combo(samesite: SameSiteCfg, secure: bool) -> Result<()> {
    anyhow::ensure!(
        !matches!(samesite, SameSiteCfg::None) || secure,
        "COOKIE_SAMESITE=none requires COOKIE_SECURE=true (browsers reject SameSite=None without Secure)"
    );
    Ok(())
}

/// Rejects a wildcard (`"*"`) CORS origin.
///
/// The API sends credentialed CORS (`allow_credentials(true)`), which is
/// incompatible with a wildcard origin — browsers refuse it, and accepting it
/// would invite credential leakage to any site. The allowlist must be explicit;
/// surface the misconfiguration at startup rather than silently emitting a
/// header browsers will reject.
pub fn validate_cors_origins(origins: &[String]) -> Result<()> {
    anyhow::ensure!(
        !origins.iter().any(|o| o.trim() == "*"),
        "CORS_ALLOWED_ORIGINS must not contain '*' (wildcard is illegal with credentialed CORS — use an explicit allowlist)"
    );
    Ok(())
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
        validate_cookie_combo(config.cookie_samesite, config.cookie_secure)?;
        validate_cors_origins(&config.cors_allowed_origins)?;
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

    // Fix 2: validate_cookie_combo unit tests
    #[test]
    fn samesite_none_without_secure_is_err() {
        let result = validate_cookie_combo(SameSiteCfg::None, false);
        assert!(
            result.is_err(),
            "SameSite=None without Secure must be rejected"
        );
    }

    #[test]
    fn samesite_none_with_secure_is_ok() {
        let result = validate_cookie_combo(SameSiteCfg::None, true);
        assert!(result.is_ok(), "SameSite=None with Secure must be accepted");
    }

    #[test]
    fn samesite_strict_without_secure_is_ok() {
        let result = validate_cookie_combo(SameSiteCfg::Strict, false);
        assert!(
            result.is_ok(),
            "SameSite=Strict without Secure must be accepted"
        );
    }

    #[test]
    fn samesite_lax_without_secure_is_ok() {
        let result = validate_cookie_combo(SameSiteCfg::Lax, false);
        assert!(
            result.is_ok(),
            "SameSite=Lax without Secure must be accepted"
        );
    }

    // Fix 3: wildcard CORS origin must be rejected (illegal with credentials).
    #[test]
    fn cors_wildcard_origin_is_rejected() {
        let result = validate_cors_origins(&["*".to_owned()]);
        assert!(
            result.is_err(),
            "wildcard '*' CORS origin must be rejected (illegal with allow_credentials)"
        );
    }

    #[test]
    fn cors_wildcard_among_valid_origins_is_rejected() {
        let result = validate_cors_origins(&["https://veyra.dev".to_owned(), "*".to_owned()]);
        assert!(
            result.is_err(),
            "a wildcard anywhere in the allowlist must be rejected"
        );
    }

    #[test]
    fn cors_explicit_origins_are_accepted() {
        let result = validate_cors_origins(&[
            "https://veyra.dev".to_owned(),
            "https://app.veyra.dev".to_owned(),
        ]);
        assert!(result.is_ok(), "explicit origins must be accepted");
    }

    #[test]
    fn cors_empty_allowlist_is_accepted() {
        let result = validate_cors_origins(&[]);
        assert!(result.is_ok(), "an empty allowlist (same-origin) is valid");
    }
}
