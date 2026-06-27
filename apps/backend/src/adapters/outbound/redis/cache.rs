use fred::clients::Pool;
use fred::prelude::*;
use serde::{de::DeserializeOwned, Serialize};
use uuid::Uuid;

/// Redis key-prefix constants — all cache keys start with one of these.
const VER_PREFIX: &str = "cache:ver:";
const LIST_PREFIX: &str = "cache:v";
const LIST_SUFFIX: &str = ":vehicles:";
const DETAIL_SUFFIX: &str = ":vehicle:";

/// TTL in seconds for cached vehicle data.
///
/// 300 s (5 min) is short enough that a Redis failure leaves the system
/// degraded for at most 5 minutes before the next cache warm; it is long
/// enough to absorb the typical read burst on a vehicle detail page.
pub const VEHICLE_CACHE_TTL_SECS: u64 = 300;

/// Lightweight Redis helper used exclusively inside the outbound redis adapter.
///
/// All methods are **fail-open**: a Redis error or a cache miss never propagates
/// to the caller — the application degrades silently to the underlying
/// repository. Errors are logged at `tracing::debug` level so they appear in
/// verbose logs but do not pollute production output.
///
/// # Key schema
/// ```text
/// cache:ver:{user_id}              → integer version counter (INCR)
/// cache:v{ver}:vehicles:{user_id} → JSON Vec<Vehicle> (list)
/// cache:v{ver}:vehicle:{user_id}:{vehicle_id} → JSON Vehicle (detail)
/// ```
///
/// # Big O
/// * `get_json` / `set_json` — O(1): one GET / SET on a known key.
/// * `bump_version` — O(1): one INCR on the version key, which atomically
///   makes every existing versioned list/detail key for that user unreachable.
///   **No key scan needed** — the version prefix changes, so stale keys simply
///   expire via their TTL instead of being explicitly deleted.
/// * `version` — O(1): one GET on the version key.
///
/// Space: O(1) per user for the version key; O(V) total cached entries where V
/// is bounded by `VEHICLE_CACHE_TTL_SECS`.
#[derive(Clone)]
pub struct RedisCache {
    pool: Pool,
}

impl RedisCache {
    /// Create a new `RedisCache` backed by the given fred connection pool.
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// Build the version key for a user.
    fn ver_key(user_id: Uuid) -> String {
        format!("{VER_PREFIX}{user_id}")
    }

    /// Build the list cache key using the current version.
    pub fn list_key(ver: u64, user_id: Uuid) -> String {
        format!("{LIST_PREFIX}{ver}{LIST_SUFFIX}{user_id}")
    }

    /// Build the detail cache key using the current version.
    pub fn detail_key(ver: u64, user_id: Uuid, vehicle_id: Uuid) -> String {
        format!("{LIST_PREFIX}{ver}{DETAIL_SUFFIX}{user_id}:{vehicle_id}")
    }

    /// Deserialize a cached value from Redis.
    ///
    /// Returns `None` on any error (key miss, Redis unavailable, JSON decode
    /// failure). Errors are logged at DEBUG — they are expected in normal
    /// operation (cold start, after a version bump, after Redis restart).
    pub async fn get_json<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let raw: Option<String> = match self.pool.get(key).await {
            Ok(v) => v,
            Err(e) => {
                tracing::debug!(key, error = %e, "cache GET failed");
                return None;
            }
        };
        let raw = raw?; // None on key miss → return None
        match serde_json::from_str::<T>(&raw) {
            Ok(v) => Some(v),
            Err(e) => {
                tracing::debug!(key, error = %e, "cache JSON decode failed");
                None
            }
        }
    }

    /// Serialize `val` to JSON and store it in Redis with the given TTL.
    ///
    /// Errors are silently discarded — a failed write means the next read
    /// will be a cache miss, which is acceptable.
    pub async fn set_json<T: Serialize>(&self, key: &str, val: &T, ttl_secs: u64) {
        let json = match serde_json::to_string(val) {
            Ok(j) => j,
            Err(e) => {
                tracing::debug!(key, error = %e, "cache JSON encode failed");
                return;
            }
        };
        let result: Result<(), _> = self
            .pool
            .set(
                key,
                json.as_str(),
                Some(Expiration::EX(ttl_secs as i64)),
                None,
                false,
            )
            .await;
        if let Err(e) = result {
            tracing::debug!(key, error = %e, "cache SET failed");
        }
    }

    /// Atomically increment the version counter for `user_id`.
    ///
    /// Returns `true` when the INCR succeeds (version advanced, all old versioned
    /// keys are now unreachable). Returns `false` on Redis error — the caller is
    /// responsible for best-effort stale-key cleanup and a `tracing::warn!`.
    pub async fn bump_version(&self, user_id: Uuid) -> bool {
        match self.pool.incr::<u64, _>(Self::ver_key(user_id)).await {
            Ok(_) => true,
            Err(e) => {
                tracing::debug!(user_id = %user_id, error = %e, "cache INCR (bump_version) failed");
                false
            }
        }
    }

    /// Delete a single cache key. Fail-open: errors are silently discarded.
    ///
    /// Used for best-effort stale-key cleanup when `bump_version` fails.
    pub async fn delete(&self, key: &str) {
        let result: Result<(), _> = self.pool.del(key).await;
        if let Err(e) = result {
            tracing::debug!(key, error = %e, "cache DEL failed");
        }
    }

    /// Read the current version counter for `user_id`.
    ///
    /// Returns:
    /// - `Some(0)` when the version key is absent (new user — correct, reads/writes
    ///   proceed under v0 since no versioned data has been written yet).
    /// - `Some(n)` when the key exists and holds a valid counter.
    /// - `None` when the Redis GET itself errors — the caller must **bypass the
    ///   cache entirely** and read from the inner repository instead, to avoid
    ///   serving a stale `cache:v0:…` entry from a degraded Redis instance.
    pub async fn version(&self, user_id: Uuid) -> Option<u64> {
        let raw: Option<String> = match self.pool.get(Self::ver_key(user_id)).await {
            Ok(v) => v,
            Err(e) => {
                tracing::debug!(user_id = %user_id, error = %e, "cache GET (version) failed");
                return None;
            }
        };
        // Key absent → new user → version 0 (no stale data exists under v0).
        // Key present but unparseable → treat as version 0 (safe: forces a miss).
        Some(raw.and_then(|s| s.parse().ok()).unwrap_or(0))
    }
}
