use std::sync::Arc;

use uuid::Uuid;

use crate::ports::session::{SessionError, SessionStore};

// ── Public types ─────────────────────────────────────────────────────────────

pub struct LogoutUseCase {
    pub sessions: Arc<dyn SessionStore>,
    pub access_ttl_secs: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LogoutError {
    /// Any Redis error → fail-closed → HTTP 503.
    Unavailable,
}

// ── Implementation ────────────────────────────────────────────────────────────

impl LogoutUseCase {
    /// Revoke the refresh-token family *and* mark the current access token's
    /// session id as revoked.
    ///
    /// Fail-closed: any store error produces [`LogoutError::Unavailable`].
    /// We never silently succeed when a revocation write fails — the caller
    /// must surface a 503 so the client retries.
    pub async fn execute(&self, family_id: Uuid, sid: Uuid) -> Result<(), LogoutError> {
        self.sessions
            .revoke(family_id)
            .await
            .map_err(|_: SessionError| LogoutError::Unavailable)?;

        self.sessions
            .revoke_session(sid, self.access_ttl_secs)
            .await
            .map_err(|_: SessionError| LogoutError::Unavailable)?;

        Ok(())
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use uuid::Uuid;

    use crate::ports::session::{
        NewSession, RotateOutcome, SessionError, SessionResult, SessionStore,
    };

    // ── Fake SessionStore ─────────────────────────────────────────────────────

    #[derive(Clone, Copy)]
    enum StoreMode {
        AllOk,
        RevokeFails,
        RevokeSessionFails,
    }

    struct FakeSessionStore {
        mode: StoreMode,
    }

    #[async_trait]
    impl SessionStore for FakeSessionStore {
        async fn create(&self, _user_id: Uuid) -> SessionResult<NewSession> {
            unimplemented!("not needed for logout tests")
        }

        async fn rotate(
            &self,
            _family_id: Uuid,
            _presented_secret: &str,
        ) -> SessionResult<RotateOutcome> {
            unimplemented!("not needed for logout tests")
        }

        async fn revoke(&self, _family_id: Uuid) -> SessionResult<()> {
            match self.mode {
                StoreMode::RevokeFails => {
                    Err(SessionError::Unavailable("fake revoke error".into()))
                }
                _ => Ok(()),
            }
        }

        async fn revoke_session(&self, _sid: Uuid, _ttl_secs: u64) -> SessionResult<()> {
            match self.mode {
                StoreMode::RevokeSessionFails => Err(SessionError::Unavailable(
                    "fake revoke_session error".into(),
                )),
                _ => Ok(()),
            }
        }

        async fn is_session_revoked(&self, _sid: Uuid) -> SessionResult<bool> {
            Ok(false)
        }
    }

    fn make_uc(mode: StoreMode) -> LogoutUseCase {
        LogoutUseCase {
            sessions: Arc::new(FakeSessionStore { mode }),
            access_ttl_secs: 900,
        }
    }

    // ── Tests ─────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn both_writes_ok_returns_unit() {
        let uc = make_uc(StoreMode::AllOk);
        let result = uc.execute(Uuid::new_v4(), Uuid::new_v4()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn revoke_error_returns_unavailable() {
        let uc = make_uc(StoreMode::RevokeFails);
        let result = uc.execute(Uuid::new_v4(), Uuid::new_v4()).await;
        assert_eq!(result, Err(LogoutError::Unavailable));
    }

    #[tokio::test]
    async fn revoke_session_error_returns_unavailable() {
        let uc = make_uc(StoreMode::RevokeSessionFails);
        let result = uc.execute(Uuid::new_v4(), Uuid::new_v4()).await;
        assert_eq!(result, Err(LogoutError::Unavailable));
    }
}
