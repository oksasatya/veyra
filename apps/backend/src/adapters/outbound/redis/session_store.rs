use async_trait::async_trait;
use base64::Engine;
use fred::clients::Pool;
use fred::prelude::*;
use rand::RngCore;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::ports::session::{NewSession, RotateOutcome, SessionError, SessionResult, SessionStore};

const SESSION_PREFIX: &str = "session:";
const REVOKE_PREFIX: &str = "revoke:";

/// Atomic Lua script — create a session hash AND set its TTL in one eval.
///
/// KEYS[1] = session hash key (e.g. `session:<family_id>`)
/// ARGV[1] = user_id (string)
/// ARGV[2] = sha256(raw_secret)
/// ARGV[3] = refresh_ttl_secs
///
/// This atomicity guarantee means a crash between HSET and EXPIRE is impossible:
/// the key either has a TTL or was never written.
const CREATE_LUA: &str = r#"
redis.call('HSET', KEYS[1], 'user_id', ARGV[1], 'current', ARGV[2], 'prev', '', 'prev_until', '0')
redis.call('EXPIRE', KEYS[1], tonumber(ARGV[3]))
return 'OK'
"#;

/// Atomic Lua rotate script.
///
/// KEYS[1] = session hash key.
/// KEYS[2] = revoke string key (e.g. `revoke:<family_id>`).
/// ARGV[1] = sha256(presented_secret)
/// ARGV[2] = sha256(new_secret)
/// ARGV[3] = grace_secs
/// ARGV[4] = refresh_ttl_secs
/// ARGV[5] = unix_now (seconds)
/// ARGV[6] = access_ttl_secs   (used to set the revoke key TTL on REUSED)
///
/// Returns an array: ["ROTATED", "<user_id>"] | ["REUSED"] | ["NOTFOUND"].
///
/// Logic:
///   - Key missing → NOTFOUND.
///   - Presented matches `current` OR (matches `prev` AND now < prev_until) → rotate.
///   - Anything else → REUSED (stale secret detected). DEL the session AND atomically
///     write the revoke key so in-flight access tokens are immediately invalidated.
const ROTATE_LUA: &str = r#"
local key = KEYS[1]
local revoke_key = KEYS[2]
if redis.call('EXISTS', key) == 0 then return {'NOTFOUND'} end
local cur     = redis.call('HGET', key, 'current')
local prev    = redis.call('HGET', key, 'prev')
local pu      = tonumber(redis.call('HGET', key, 'prev_until') or '0')
local uid     = redis.call('HGET', key, 'user_id')
local presented = ARGV[1]
local now     = tonumber(ARGV[5])
local in_grace = (prev ~= false and prev == presented and now < pu)
if presented == cur or in_grace then
  redis.call('HSET', key, 'prev', cur, 'prev_until', now + tonumber(ARGV[3]), 'current', ARGV[2])
  redis.call('EXPIRE', key, tonumber(ARGV[4]))
  return {'ROTATED', uid}
end
redis.call('DEL', key)
redis.call('SET', revoke_key, '1', 'EX', tonumber(ARGV[6]))
return {'REUSED'}
"#;

/// Redis-backed implementation of [`SessionStore`].
///
/// # Session layout (Hash)
/// Key: `session:<family_id>`
/// Fields:
///   - `user_id`    — UUID string of the owning user
///   - `current`    — sha256-hex of the current refresh secret
///   - `prev`       — sha256-hex of the previous secret (for grace-window rotation)
///   - `prev_until` — unix timestamp after which `prev` is no longer accepted
///
/// # Revocation layout (String)
/// Key: `revoke:<sid>` → value `"1"`, TTL = access-token lifetime.
///
/// # Why the single-prev + grace window is safe
///
/// Cookies are shared **per browser**: when a rotation completes the browser's
/// cookie jar is updated to the new secret, so the same browser never holds two
/// valid refresh secrets simultaneously. The grace window covers in-flight
/// concurrent refreshes (e.g. two tabs rotating at the exact same instant).
/// Each **login** creates a *separate* family — different devices never share a
/// family — so one device being compromised does not grant access to another
/// device's family. The only multi-secret window is the brief grace interval for
/// in-flight concurrent refreshes, which is bounded and short.
///
/// # Big O
/// Every public method is **O(1)**: one key per family (no scans), and the Lua
/// scripts perform a constant number of field reads/writes regardless of session
/// count. Space is O(1) per session: four hash fields + optional revoke string.
#[derive(Clone)]
pub struct RedisSessionStore {
    pool: Pool,
    refresh_ttl_secs: u64,
    access_ttl_secs: u64,
    grace_secs: u64,
}

impl RedisSessionStore {
    /// Create a new store.
    ///
    /// - `pool`              — initialized fred connection pool
    /// - `refresh_ttl_secs` — lifetime of a refresh-token family (e.g. 604 800 = 7 days)
    /// - `access_ttl_secs`  — lifetime of an access token (e.g. 900 = 15 min);
    ///   used to set the revoke-key TTL when reuse is detected
    /// - `grace_secs`       — seconds the previous secret remains valid after a rotation
    pub fn new(pool: Pool, refresh_ttl_secs: u64, access_ttl_secs: u64, grace_secs: u64) -> Self {
        Self {
            pool,
            refresh_ttl_secs,
            access_ttl_secs,
            grace_secs,
        }
    }

    fn session_key(family_id: Uuid) -> String {
        format!("{SESSION_PREFIX}{family_id}")
    }

    fn revoke_key(sid: Uuid) -> String {
        format!("{REVOKE_PREFIX}{sid}")
    }

    /// Expose the underlying pool.
    ///
    /// Used by integration tests for out-of-band assertions (e.g. checking TTL).
    /// Not part of the [`SessionStore`] port — production callers should use the
    /// trait methods only.
    pub fn pool_ref(&self) -> &Pool {
        &self.pool
    }
}

/// Hash `raw` with SHA-256 and return the lowercase hex string.
fn hash_secret(raw: &str) -> String {
    let mut h = Sha256::new();
    h.update(raw.as_bytes());
    hex::encode(h.finalize())
}

/// Generate a cryptographically-secure 32-byte random secret encoded as
/// URL-safe base64 (no padding).
fn random_secret() -> String {
    let mut buf = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut buf);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(buf)
}

/// Convert a fred `Error` into `SessionError::Unavailable`.
fn map_err(e: Error) -> SessionError {
    SessionError::Unavailable(e.to_string())
}

#[async_trait]
impl SessionStore for RedisSessionStore {
    async fn create(&self, user_id: Uuid) -> SessionResult<NewSession> {
        let family_id = Uuid::new_v4();
        let raw_secret = random_secret();
        let key = Self::session_key(family_id);

        let keys = vec![key];
        let args = vec![
            user_id.to_string(),
            hash_secret(&raw_secret),
            self.refresh_ttl_secs.to_string(),
        ];

        let _: String = self
            .pool
            .eval(CREATE_LUA, keys, args)
            .await
            .map_err(map_err)?;

        Ok(NewSession {
            family_id,
            raw_secret,
        })
    }

    /// Atomically rotate the refresh token for `family_id`.
    ///
    /// On reuse detection the session hash is deleted AND the revoke key
    /// (`revoke:<family_id>`) is written — all inside a single Lua `eval` so
    /// the two operations are atomic. This ensures in-flight access tokens
    /// are invalidated the moment theft is detected, with no window between
    /// the DEL and the SET.
    async fn rotate(
        &self,
        family_id: Uuid,
        presented_secret: &str,
    ) -> SessionResult<RotateOutcome> {
        let new_secret = random_secret();
        let now = chrono::Utc::now().timestamp();
        let keys = vec![Self::session_key(family_id), Self::revoke_key(family_id)];
        let args = vec![
            hash_secret(presented_secret),
            hash_secret(&new_secret),
            self.grace_secs.to_string(),
            self.refresh_ttl_secs.to_string(),
            now.to_string(),
            self.access_ttl_secs.to_string(),
        ];

        let out: Vec<String> = self
            .pool
            .eval(ROTATE_LUA, keys, args)
            .await
            .map_err(map_err)?;

        match out.first().map(String::as_str) {
            Some("ROTATED") => {
                let user_id = out.get(1).and_then(|s| s.parse().ok()).ok_or_else(|| {
                    SessionError::Unavailable("missing user_id in rotate result".into())
                })?;
                Ok(RotateOutcome::Rotated {
                    user_id,
                    new_raw_secret: new_secret,
                })
            }
            Some("REUSED") => Ok(RotateOutcome::Reused),
            Some("NOTFOUND") => Ok(RotateOutcome::NotFound),
            _ => Err(SessionError::Unavailable("unexpected rotate result".into())),
        }
    }

    async fn revoke(&self, family_id: Uuid) -> SessionResult<()> {
        let _: () = self
            .pool
            .del(Self::session_key(family_id))
            .await
            .map_err(map_err)?;
        Ok(())
    }

    async fn revoke_session(&self, sid: Uuid, ttl_secs: u64) -> SessionResult<()> {
        let _: () = self
            .pool
            .set(
                Self::revoke_key(sid),
                "1",
                Some(Expiration::EX(ttl_secs as i64)),
                None,
                false,
            )
            .await
            .map_err(map_err)?;
        Ok(())
    }

    async fn is_session_revoked(&self, sid: Uuid) -> SessionResult<bool> {
        let exists: bool = self
            .pool
            .exists(Self::revoke_key(sid))
            .await
            .map_err(map_err)?;
        Ok(exists)
    }
}
