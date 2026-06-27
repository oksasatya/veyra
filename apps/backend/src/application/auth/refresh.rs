use std::sync::Arc;

use uuid::Uuid;

use crate::ports::{
    auth::{AuthError, AuthPort},
    session::{RotateOutcome, SessionStore},
};

// ── Public types ─────────────────────────────────────────────────────────────

pub struct RefreshUseCase {
    pub sessions: Arc<dyn SessionStore>,
    pub auth: Arc<dyn AuthPort>,
    pub access_ttl_secs: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RefreshOutput {
    pub access_token: String,
    pub family_id: Uuid,
    pub raw_secret: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RefreshError {
    /// Presented token is invalid or a theft was detected → HTTP 401.
    Invalid,
    /// Session store or signing service unavailable → HTTP 503.
    Unavailable,
}

// ── Implementation ────────────────────────────────────────────────────────────

impl RefreshUseCase {
    pub async fn execute(
        &self,
        family_id: Uuid,
        presented_secret: &str,
    ) -> Result<RefreshOutput, RefreshError> {
        match self.sessions.rotate(family_id, presented_secret).await {
            Ok(RotateOutcome::Rotated {
                user_id,
                new_raw_secret,
            }) => {
                let jti = Uuid::new_v4();
                // sid == family_id by invariant (see plan §Global Constraints)
                let access_token = self
                    .auth
                    .sign_access(user_id, family_id, jti)
                    .map_err(|_: AuthError| RefreshError::Unavailable)?;

                Ok(RefreshOutput {
                    access_token,
                    family_id,
                    raw_secret: new_raw_secret,
                })
            }

            Ok(RotateOutcome::Reused) => {
                // Theft response: revoke the entire family and its active session.
                // The Redis adapter also revokes atomically on reuse (in the Lua
                // script), so these calls are defensive redundancy — they keep the
                // theft-response policy explicit and testable at the application
                // layer without depending on adapter internals.
                //
                // intentional: revoke errors are swallowed here because we are
                // already on the failure path (theft detected). The caller will
                // receive Invalid regardless of whether these writes succeed.
                let _ = self.sessions.revoke(family_id).await;
                let _ = self
                    .sessions
                    .revoke_session(family_id, self.access_ttl_secs)
                    .await;

                Err(RefreshError::Invalid)
            }

            Ok(RotateOutcome::NotFound) => Err(RefreshError::Invalid),

            Err(_) => Err(RefreshError::Unavailable),
        }
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Mutex;
    use uuid::Uuid;

    use crate::ports::{
        auth::{AccessClaims, AuthError, AuthPort},
        session::{NewSession, RotateOutcome, SessionError, SessionResult, SessionStore},
    };

    // ── Fake SessionStore ─────────────────────────────────────────────────────

    /// Records which calls were made so tests can assert theft-response behaviour.
    struct FakeSessionStore {
        /// The outcome returned by `rotate`.
        rotate_outcome: RotateOutcome,
        /// Tracks whether `revoke` was called.
        revoke_called: Mutex<bool>,
        /// Tracks whether `revoke_session` was called.
        revoke_session_called: Mutex<bool>,
        /// When true, `rotate` returns `Err(Unavailable)` instead.
        store_unavailable: bool,
    }

    impl FakeSessionStore {
        fn returning(outcome: RotateOutcome) -> Self {
            Self {
                rotate_outcome: outcome,
                revoke_called: Mutex::new(false),
                revoke_session_called: Mutex::new(false),
                store_unavailable: false,
            }
        }

        fn unavailable() -> Self {
            Self {
                rotate_outcome: RotateOutcome::NotFound, // never reached
                revoke_called: Mutex::new(false),
                revoke_session_called: Mutex::new(false),
                store_unavailable: true,
            }
        }
    }

    #[async_trait]
    impl SessionStore for FakeSessionStore {
        async fn create(&self, _user_id: Uuid) -> SessionResult<NewSession> {
            unimplemented!("not needed for refresh tests")
        }

        async fn rotate(
            &self,
            _family_id: Uuid,
            _presented_secret: &str,
        ) -> SessionResult<RotateOutcome> {
            if self.store_unavailable {
                return Err(SessionError::Unavailable("fake error".into()));
            }
            // RotateOutcome doesn't implement Clone, so we re-construct from the
            // stored discriminant each time rotate is called.
            match &self.rotate_outcome {
                RotateOutcome::Rotated {
                    user_id,
                    new_raw_secret,
                } => Ok(RotateOutcome::Rotated {
                    user_id: *user_id,
                    new_raw_secret: new_raw_secret.clone(),
                }),
                RotateOutcome::Reused => Ok(RotateOutcome::Reused),
                RotateOutcome::NotFound => Ok(RotateOutcome::NotFound),
            }
        }

        async fn revoke(&self, _family_id: Uuid) -> SessionResult<()> {
            *self.revoke_called.lock().unwrap() = true;
            Ok(())
        }

        async fn revoke_session(&self, _sid: Uuid, _ttl_secs: u64) -> SessionResult<()> {
            *self.revoke_session_called.lock().unwrap() = true;
            Ok(())
        }

        async fn is_session_revoked(&self, _sid: Uuid) -> SessionResult<bool> {
            Ok(false)
        }
    }

    // ── Fake AuthPort ─────────────────────────────────────────────────────────

    struct FakeAuthPort;

    impl AuthPort for FakeAuthPort {
        fn sign_token(&self, _user_id: Uuid) -> Result<String, AuthError> {
            Ok("legacy.jwt".into())
        }

        fn verify_token(&self, _token: &str) -> Result<Uuid, AuthError> {
            Ok(Uuid::new_v4())
        }

        fn sign_access(&self, _user_id: Uuid, _sid: Uuid, _jti: Uuid) -> Result<String, AuthError> {
            Ok("mock.jwt".into())
        }

        fn verify_access(&self, _token: &str) -> Result<AccessClaims, AuthError> {
            Ok(AccessClaims {
                user_id: Uuid::new_v4(),
                sid: Uuid::new_v4(),
                jti: Uuid::new_v4(),
            })
        }
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn make_uc(store: FakeSessionStore) -> RefreshUseCase {
        RefreshUseCase {
            sessions: Arc::new(store),
            auth: Arc::new(FakeAuthPort),
            access_ttl_secs: 900,
        }
    }

    // ── Tests ─────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn rotated_returns_ok_with_mock_jwt() {
        let user_id = Uuid::new_v4();
        let store = FakeSessionStore::returning(RotateOutcome::Rotated {
            user_id,
            new_raw_secret: "new-secret".into(),
        });
        let uc = make_uc(store);
        let family_id = Uuid::new_v4();

        let result = uc.execute(family_id, "old-secret").await;

        let output = result.expect("expected Ok");
        assert_eq!(output.access_token, "mock.jwt");
        assert_eq!(output.family_id, family_id);
        assert_eq!(output.raw_secret, "new-secret");
    }

    #[tokio::test]
    async fn reused_returns_invalid_and_calls_both_revokes() {
        let store = FakeSessionStore::returning(RotateOutcome::Reused);
        let revoke_called = Arc::new(Mutex::new(false));
        let revoke_session_called = Arc::new(Mutex::new(false));

        // We need to capture the flags from the store before moving it.
        // Use a wrapper that shares the Mutex references.
        struct TrackingStore {
            revoke_called: Arc<Mutex<bool>>,
            revoke_session_called: Arc<Mutex<bool>>,
        }

        #[async_trait]
        impl SessionStore for TrackingStore {
            async fn create(&self, _user_id: Uuid) -> SessionResult<NewSession> {
                unimplemented!()
            }
            async fn rotate(
                &self,
                _family_id: Uuid,
                _presented_secret: &str,
            ) -> SessionResult<RotateOutcome> {
                Ok(RotateOutcome::Reused)
            }
            async fn revoke(&self, _family_id: Uuid) -> SessionResult<()> {
                *self.revoke_called.lock().unwrap() = true;
                Ok(())
            }
            async fn revoke_session(&self, _sid: Uuid, _ttl_secs: u64) -> SessionResult<()> {
                *self.revoke_session_called.lock().unwrap() = true;
                Ok(())
            }
            async fn is_session_revoked(&self, _sid: Uuid) -> SessionResult<bool> {
                Ok(false)
            }
        }

        let tracking_store = TrackingStore {
            revoke_called: Arc::clone(&revoke_called),
            revoke_session_called: Arc::clone(&revoke_session_called),
        };

        let uc = RefreshUseCase {
            sessions: Arc::new(tracking_store),
            auth: Arc::new(FakeAuthPort),
            access_ttl_secs: 900,
        };

        let result = uc.execute(Uuid::new_v4(), "stale-secret").await;

        assert_eq!(result, Err(RefreshError::Invalid));
        assert!(
            *revoke_called.lock().unwrap(),
            "revoke should have been called on theft detection"
        );
        assert!(
            *revoke_session_called.lock().unwrap(),
            "revoke_session should have been called on theft detection"
        );

        // suppress unused binding warning
        let _ = store;
    }

    #[tokio::test]
    async fn not_found_returns_invalid() {
        let store = FakeSessionStore::returning(RotateOutcome::NotFound);
        let uc = make_uc(store);

        let result = uc.execute(Uuid::new_v4(), "any-secret").await;

        assert_eq!(result, Err(RefreshError::Invalid));
    }

    #[tokio::test]
    async fn store_error_returns_unavailable() {
        let store = FakeSessionStore::unavailable();
        let uc = make_uc(store);

        let result = uc.execute(Uuid::new_v4(), "any-secret").await;

        assert_eq!(result, Err(RefreshError::Unavailable));
    }
}
