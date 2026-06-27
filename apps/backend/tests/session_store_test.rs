mod common;
use common::{redis_store, redis_store_with_grace};
use fred::prelude::KeysInterface;
use uuid::Uuid;
use veyra::ports::session::{RotateOutcome, SessionStore};

#[tokio::test]
async fn rotate_then_old_secret_after_grace_is_reused_and_revokes() {
    let (store, _g) = redis_store().await; // grace = 0
    let uid = Uuid::new_v4();
    let s = store.create(uid).await.unwrap();
    let new1 = match store.rotate(s.family_id, &s.raw_secret).await.unwrap() {
        RotateOutcome::Rotated {
            new_raw_secret,
            user_id,
        } => {
            assert_eq!(user_id, uid);
            new_raw_secret
        }
        o => panic!("expected Rotated, got {o:?}"),
    };
    assert!(matches!(
        store.rotate(s.family_id, &s.raw_secret).await.unwrap(),
        RotateOutcome::Reused
    ));
    assert!(matches!(
        store.rotate(s.family_id, &new1).await.unwrap(),
        RotateOutcome::NotFound
    ));
}

#[tokio::test]
async fn in_grace_previous_secret_still_rotates() {
    let (store, _g) = redis_store_with_grace(5).await;
    let s = store.create(Uuid::new_v4()).await.unwrap();
    let _ = store.rotate(s.family_id, &s.raw_secret).await.unwrap();
    assert!(matches!(
        store.rotate(s.family_id, &s.raw_secret).await.unwrap(),
        RotateOutcome::Rotated { .. }
    ));
}

#[tokio::test]
async fn revoke_session_then_is_revoked_true() {
    let (store, _g) = redis_store().await;
    let sid = Uuid::new_v4();
    store.revoke_session(sid, 900).await.unwrap();
    assert!(store.is_session_revoked(sid).await.unwrap());
    assert!(!store.is_session_revoked(Uuid::new_v4()).await.unwrap());
}

/// Fix A (TDD RED): create must set a TTL — session key must not be immortal.
#[tokio::test]
async fn create_sets_ttl() {
    let (store, _g) = redis_store().await;
    let uid = Uuid::new_v4();
    let s = store.create(uid).await.unwrap();

    // Access the inner pool via the store's pool() helper (added for testing).
    let pool = store.pool_ref();
    let key = format!("session:{}", s.family_id);
    let ttl: i64 = pool.ttl(&key).await.unwrap();
    // TTL > 0 means the key has an expiry; -1 = persistent (no TTL), -2 = missing.
    assert!(
        ttl > 0,
        "session key must have a TTL after create, got {ttl}"
    );
}

/// Fix B (TDD RED): on REUSED the revoke key must be set atomically in the Lua script.
#[tokio::test]
async fn reuse_revokes_session_access() {
    // grace = 0 → the old secret is immediately stale after rotation
    let (store, _g) = redis_store_with_grace(0).await;
    let uid = Uuid::new_v4();
    let s = store.create(uid).await.unwrap();

    // First rotation (valid) — moves s.raw_secret to `prev`, issues new secret.
    let _new_secret = match store.rotate(s.family_id, &s.raw_secret).await.unwrap() {
        RotateOutcome::Rotated { new_raw_secret, .. } => new_raw_secret,
        o => panic!("expected Rotated on first rotate, got {o:?}"),
    };

    // Second rotation with the original (now stale) secret → REUSED.
    assert!(matches!(
        store.rotate(s.family_id, &s.raw_secret).await.unwrap(),
        RotateOutcome::Reused
    ));

    // The Lua script on REUSED must have written `revoke:{family_id}`.
    // family_id == sid by the port invariant.
    assert!(
        store.is_session_revoked(s.family_id).await.unwrap(),
        "reuse detection must atomically set the revoke key"
    );
}
