//! Cookie builder — env-driven prefix derivation + attribute matrix.
//!
//! Security model (from project brief):
//! - `__Host-` prefix: `secure == true` AND `domain.is_none()` (forbids Domain attr + requires Path=/)
//! - `__Secure-` prefix: `secure == true` AND (`domain.is_some()` OR forced for refresh under `__Host-` rules)
//! - No prefix: `secure == false` (HTTP dev)
//!
//! Refresh cookie is ALWAYS on `Path=/auth`; `__Host-` requires `Path=/` → refresh can never use `__Host-`.
//! Access + CSRF use `Path=/`; CSRF is NOT `HttpOnly` (JS reads it).

use axum_extra::extract::cookie::{Cookie, SameSite};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::RngCore;
use time::Duration;

use crate::bootstrap::config::SameSiteCfg;

// ── Cookie name base strings ──────────────────────────────────────────────────

pub const ACCESS_BASE: &str = "veyra_access";
pub const REFRESH_BASE: &str = "veyra_refresh";
pub const CSRF_BASE: &str = "veyra_csrf";

/// The double-submit CSRF request header name. Single source of truth shared by
/// the CSRF middleware (the header it checks), the router (the CORS allowed
/// header), and the integration tests — so the allowed header can never drift
/// from the checked one.
pub const X_CSRF_TOKEN: &str = "x-csrf-token";

// ── Prefix constants (never duplicate these string literals) ──────────────────

const PREFIX_HOST: &str = "__Host-";
const PREFIX_SECURE: &str = "__Secure-";

// ── Cookie paths ──────────────────────────────────────────────────────────────

const PATH_ROOT: &str = "/";
const PATH_AUTH: &str = "/auth";

// ── Policy ────────────────────────────────────────────────────────────────────

/// Runtime cookie policy derived from [`crate::bootstrap::config::Config`].
#[derive(Debug, Clone)]
pub struct CookiePolicy {
    pub secure: bool,
    pub samesite: SameSiteCfg,
    pub domain: Option<String>,
    pub access_ttl_secs: u64,
    pub refresh_ttl_secs: u64,
}

// ── Cookie kind ───────────────────────────────────────────────────────────────

pub enum CookieKind {
    Access,
    Refresh,
    Csrf,
}

// ── SameSite mapping ──────────────────────────────────────────────────────────

fn map_samesite(cfg: SameSiteCfg) -> SameSite {
    match cfg {
        SameSiteCfg::Strict => SameSite::Strict,
        SameSiteCfg::Lax => SameSite::Lax,
        SameSiteCfg::None => SameSite::None,
    }
}

// ── Prefix derivation ─────────────────────────────────────────────────────────

/// `true` when `secure && domain.is_none()` — eligible for `__Host-` prefix.
///
/// Callers must NOT use this for the refresh cookie (its `Path=/auth` violates the `__Host-` spec).
fn host_eligible(p: &CookiePolicy) -> bool {
    p.secure && p.domain.is_none()
}

/// `true` when `secure && domain.is_some()` — eligible for `__Secure-` prefix.
fn secure_with_domain(p: &CookiePolicy) -> bool {
    p.secure && p.domain.is_some()
}

/// Cookie name for the access token.
/// `Path=/` → eligible for `__Host-` when `secure && no domain`.
pub fn access_name(p: &CookiePolicy) -> String {
    if host_eligible(p) {
        format!("{PREFIX_HOST}{ACCESS_BASE}")
    } else if secure_with_domain(p) {
        format!("{PREFIX_SECURE}{ACCESS_BASE}")
    } else {
        ACCESS_BASE.to_owned()
    }
}

/// Cookie name for the refresh token.
/// NEVER `__Host-` (Path=/auth violates the prefix spec). Falls back to `__Secure-` or no prefix.
pub fn refresh_name(p: &CookiePolicy) -> String {
    // Any secure config (domain or no domain) → __Secure- for refresh
    if p.secure {
        format!("{PREFIX_SECURE}{REFRESH_BASE}")
    } else {
        REFRESH_BASE.to_owned()
    }
}

/// Cookie name for the CSRF token.
/// Same prefix rules as access (Path=/).
pub fn csrf_name(p: &CookiePolicy) -> String {
    if host_eligible(p) {
        format!("{PREFIX_HOST}{CSRF_BASE}")
    } else if secure_with_domain(p) {
        format!("{PREFIX_SECURE}{CSRF_BASE}")
    } else {
        CSRF_BASE.to_owned()
    }
}

// ── Cookie builders ───────────────────────────────────────────────────────────

/// Builds a cookie with common attributes; optionally sets `Domain` (not under `__Host-`).
///
/// `Secure` is only emitted when `p.secure == true`; emitting an explicit `Secure=false`
/// is harmless but produces noisier `Set-Cookie` headers and can confuse some middleware.
fn build_cookie(
    name: String,
    value: String,
    path: &'static str,
    http_only: bool,
    max_age: Duration,
    p: &CookiePolicy,
    use_host_prefix: bool,
) -> Cookie<'static> {
    let mut c = Cookie::build((name, value))
        .http_only(http_only)
        .same_site(map_samesite(p.samesite))
        .path(path)
        .max_age(max_age)
        .build();

    // Only mark Secure when the policy says so — avoids emitting an explicit Secure=false.
    if p.secure {
        c.set_secure(true);
    }

    // Attach Domain only when not using __Host- (which forbids it)
    if !use_host_prefix {
        if let Some(ref d) = p.domain {
            c.set_domain(d.clone());
        }
    }

    c
}

/// Access-token cookie: `HttpOnly`, `Path=/`, TTL = `access_ttl_secs`.
pub fn access_cookie(p: &CookiePolicy, value: &str) -> Cookie<'static> {
    let ttl = Duration::seconds(p.access_ttl_secs as i64);
    build_cookie(
        access_name(p),
        value.to_owned(),
        PATH_ROOT,
        true,
        ttl,
        p,
        host_eligible(p),
    )
}

/// Refresh-token cookie: `HttpOnly`, `Path=/auth`, TTL = `refresh_ttl_secs`.
pub fn refresh_cookie(p: &CookiePolicy, value: &str) -> Cookie<'static> {
    let ttl = Duration::seconds(p.refresh_ttl_secs as i64);
    // Refresh is never __Host- so domain can be attached when present
    build_cookie(
        refresh_name(p),
        value.to_owned(),
        PATH_AUTH,
        true,
        ttl,
        p,
        false, // never __Host- for refresh
    )
}

/// CSRF cookie: NOT `HttpOnly` (JS must read it), `Path=/`,
/// TTL = `refresh_ttl_secs` (must outlive the refresh path it guards).
///
/// `/auth/refresh` is protected by the double-submit CSRF check. If the CSRF cookie
/// were set to `access_ttl_secs` (15 min), it would expire while the refresh cookie
/// is still valid (7 days), making it impossible for the SPA to call `/auth/refresh`
/// after an idle period — resulting in a forced re-login and defeating token rotation.
pub fn csrf_cookie(p: &CookiePolicy, value: &str) -> Cookie<'static> {
    let ttl = Duration::seconds(p.refresh_ttl_secs as i64);
    build_cookie(
        csrf_name(p),
        value.to_owned(),
        PATH_ROOT,
        false,
        ttl,
        p,
        host_eligible(p),
    )
}

// ── Clear ─────────────────────────────────────────────────────────────────────

/// Returns a deletion cookie (same name/path/domain/secure/samesite, `Max-Age=0`).
///
/// `HttpOnly` matches the attribute the cookie was SET with per kind:
/// `Access` and `Refresh` are `HttpOnly=true`; `Csrf` is `HttpOnly=false` (JS reads it).
/// Matching all attributes is correct and safe — browsers key deletion on
/// (name, path, domain), but attribute parity prevents confusion in middleware.
pub fn clear(p: &CookiePolicy, kind: CookieKind) -> Cookie<'static> {
    let (name, path, http_only, is_host) = match kind {
        CookieKind::Access => (access_name(p), PATH_ROOT, true, host_eligible(p)),
        CookieKind::Refresh => (refresh_name(p), PATH_AUTH, true, false),
        CookieKind::Csrf => (csrf_name(p), PATH_ROOT, false, host_eligible(p)),
    };
    build_cookie(
        name,
        String::new(),
        path,
        http_only,
        Duration::ZERO,
        p,
        is_host,
    )
}

// ── CSRF token generation ─────────────────────────────────────────────────────

/// Generates a 32-byte CSPRNG token encoded as base64url (no padding).
/// Used by handlers to create a CSRF token value before calling [`csrf_cookie`].
pub fn random_token() -> String {
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bootstrap::config::SameSiteCfg;

    fn policy(secure: bool, domain: Option<&str>) -> CookiePolicy {
        CookiePolicy {
            secure,
            samesite: SameSiteCfg::Strict,
            domain: domain.map(String::from),
            access_ttl_secs: 900,
            refresh_ttl_secs: 604_800,
        }
    }

    #[test]
    fn host_prefix_when_secure_no_domain() {
        let p = policy(true, None);
        assert_eq!(access_name(&p), "__Host-veyra_access");
        assert_eq!(refresh_name(&p), "__Secure-veyra_refresh");
    }

    #[test]
    fn secure_prefix_when_domain_set() {
        assert_eq!(
            access_name(&policy(true, Some("veyra.dev"))),
            "__Secure-veyra_access"
        );
    }

    #[test]
    fn no_prefix_when_insecure() {
        assert_eq!(access_name(&policy(false, None)), "veyra_access");
    }

    #[test]
    fn refresh_scoped_to_auth_path() {
        let c = refresh_cookie(&policy(true, None), "fam.secret");
        assert_eq!(c.path(), Some("/auth"));
        assert_eq!(c.http_only(), Some(true));
    }

    #[test]
    fn csrf_is_readable() {
        assert_ne!(
            csrf_cookie(&policy(true, None), "t").http_only(),
            Some(true)
        );
    }

    #[test]
    fn random_token_distinct_and_nonempty() {
        let a = random_token();
        let b = random_token();
        assert!(!a.is_empty());
        assert!(!b.is_empty());
        assert_ne!(a, b);
    }

    // Fix 1: clear() must set http_only per kind
    #[test]
    fn clear_access_is_http_only() {
        let p = policy(true, None);
        let c = clear(&p, CookieKind::Access);
        assert_eq!(c.http_only(), Some(true), "clear(Access) must be HttpOnly");
    }

    #[test]
    fn clear_refresh_is_http_only() {
        let p = policy(true, None);
        let c = clear(&p, CookieKind::Refresh);
        assert_eq!(c.http_only(), Some(true), "clear(Refresh) must be HttpOnly");
    }

    #[test]
    fn clear_csrf_is_not_http_only() {
        let p = policy(true, None);
        let c = clear(&p, CookieKind::Csrf);
        assert_ne!(
            c.http_only(),
            Some(true),
            "clear(Csrf) must NOT be HttpOnly"
        );
    }

    // C1: CSRF cookie TTL must equal refresh_ttl_secs (not access_ttl_secs)
    // The CSRF cookie guards /auth/refresh (the 7-day path). After ~15 min idle
    // the access cookie expires; without a valid CSRF cookie the SPA cannot call
    // /auth/refresh → 403 → forced re-login. Fix: use refresh_ttl_secs.
    #[test]
    fn csrf_cookie_ttl_equals_refresh_ttl() {
        let p = policy(true, None);
        // refresh_ttl_secs = 604_800 (7 days), access_ttl_secs = 900 (15 min)
        let c = csrf_cookie(&p, "token");
        assert_eq!(
            c.max_age(),
            Some(time::Duration::seconds(604_800)),
            "CSRF cookie must outlive the refresh path it guards (refresh_ttl_secs=604800, not access_ttl_secs=900)"
        );
    }

    #[test]
    fn access_cookie_ttl_equals_access_ttl() {
        let p = policy(true, None);
        let c = access_cookie(&p, "token");
        assert_eq!(
            c.max_age(),
            Some(time::Duration::seconds(900)),
            "Access cookie TTL must equal access_ttl_secs=900"
        );
    }

    #[test]
    fn refresh_cookie_ttl_equals_refresh_ttl() {
        let p = policy(true, None);
        let c = refresh_cookie(&p, "token");
        assert_eq!(
            c.max_age(),
            Some(time::Duration::seconds(604_800)),
            "Refresh cookie TTL must equal refresh_ttl_secs=604800"
        );
    }

    // Fix 3: Secure attribute should only be set when policy.secure == true
    #[test]
    fn access_cookie_no_explicit_secure_when_insecure() {
        let p = policy(false, None);
        let c = access_cookie(&p, "tok");
        // When policy is insecure, Secure attr should not be present (or false)
        assert_ne!(
            c.secure(),
            Some(true),
            "insecure policy must not mark Secure"
        );
    }

    #[test]
    fn access_cookie_secure_when_policy_secure() {
        let p = policy(true, None);
        let c = access_cookie(&p, "tok");
        assert_eq!(c.secure(), Some(true), "secure policy must mark Secure");
    }

    #[test]
    fn clear_no_explicit_secure_when_insecure() {
        let p = policy(false, None);
        let c = clear(&p, CookieKind::Access);
        assert_ne!(
            c.secure(),
            Some(true),
            "clear with insecure policy must not mark Secure"
        );
    }
}
