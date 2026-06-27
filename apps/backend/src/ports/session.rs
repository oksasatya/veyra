use async_trait::async_trait;
use uuid::Uuid;

/// The result of creating a new session.
///
/// `family_id` is also the `sid` claim in the access token — they are identical
/// by definition. The access token's `sid` IS the refresh family id.
pub struct NewSession {
    pub family_id: Uuid,
    pub raw_secret: String,
}

/// Result of a refresh-token rotation attempt.
#[derive(Debug)]
pub enum RotateOutcome {
    /// Rotation succeeded. Returns the new raw secret and the owning user id.
    Rotated {
        user_id: Uuid,
        new_raw_secret: String,
    },
    /// The presented secret was a previously-used (stale) token outside the
    /// grace window — the family is revoked and all tokens are invalidated.
    Reused,
    /// The family does not exist (expired, already revoked, or never created).
    NotFound,
}

/// Errors produced by the session store.
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("session store unavailable: {0}")]
    Unavailable(String),
}

/// Convenience alias.
pub type SessionResult<T> = Result<T, SessionError>;

/// Port for managing refresh-token families in a backing store (Redis).
///
/// ## Invariant: `sid == family_id`
/// The `sid` claim embedded in an access token is always equal to the refresh
/// family id. There is no separate concept; they share the same UUID.
#[async_trait]
pub trait SessionStore: Send + Sync {
    /// Create a new session for `user_id`. Returns the family id (== sid) and
    /// a raw secret that the caller includes in the refresh token.
    async fn create(&self, user_id: Uuid) -> SessionResult<NewSession>;

    /// Atomically rotate the refresh token for `family_id`.
    ///
    /// - If `presented_secret` matches the current secret (or is within the
    ///   grace window), a new secret is issued and `Rotated` is returned.
    /// - If `presented_secret` is a stale secret (reuse detected), the family
    ///   is atomically deleted and `Reused` is returned.
    /// - If the family does not exist, `NotFound` is returned.
    async fn rotate(&self, family_id: Uuid, presented_secret: &str)
        -> SessionResult<RotateOutcome>;

    /// Revoke an entire refresh-token family (e.g. on logout).
    async fn revoke(&self, family_id: Uuid) -> SessionResult<()>;

    /// Write a short-lived revocation marker for an access token identified by
    /// `sid`. Subsequent calls to [`is_session_revoked`] return `true` until
    /// the marker expires after `ttl_secs`.
    async fn revoke_session(&self, sid: Uuid, ttl_secs: u64) -> SessionResult<()>;

    /// Check whether the access token identified by `sid` has been revoked.
    async fn is_session_revoked(&self, sid: Uuid) -> SessionResult<bool>;
}
