mod common;

use fred::prelude::*;
use serde_json::json;
use uuid::Uuid;
use veyra::adapters::outbound::redis::cache::RedisCache;

/// List initially returns 1 vehicle; after inserting a 2nd (which bumps the
/// cache version), list must return 2 — i.e. the stale 1-vehicle cached
/// response is NOT served.
#[tokio::test]
async fn write_invalidates_list_cache() {
    let app = common::spawn_app().await;
    let s = common::register_and_login(&app, "cache_user@example.com").await;

    // Create first vehicle.
    app.client
        .post("/vehicles")
        .authorization_bearer(&s.access)
        .json(&json!({
            "brand": "Toyota",
            "model": "Avanza",
            "year": 2020,
            "plate_number": "B 0001 CCC",
            "fuel_type": "petrol",
            "current_odometer": 0
        }))
        .await;

    // First list call — primes the cache with 1 vehicle.
    let resp = app
        .client
        .get("/vehicles")
        .authorization_bearer(&s.access)
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(
        body["data"].as_array().unwrap().len(),
        1,
        "expected 1 vehicle after first insert"
    );

    // Create second vehicle — this write should bump the cache version,
    // invalidating the stale list key.
    app.client
        .post("/vehicles")
        .authorization_bearer(&s.access)
        .json(&json!({
            "brand": "Honda",
            "model": "Jazz",
            "year": 2021,
            "plate_number": "B 0002 CCC",
            "fuel_type": "diesel",
            "current_odometer": 500
        }))
        .await;

    // Second list call — must return 2, not the cached 1.
    let resp = app
        .client
        .get("/vehicles")
        .authorization_bearer(&s.access)
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(
        body["data"].as_array().unwrap().len(),
        2,
        "expected 2 vehicles after second insert — stale cache must not be served"
    );
}

/// User B's list must never return user A's vehicles even after A's results
/// have been cached under A's versioned key.
#[tokio::test]
async fn cross_user_cache_isolation() {
    let app = common::spawn_app().await;
    let a = common::register_and_login(&app, "alice_cache@example.com").await;
    let b = common::register_and_login(&app, "bob_cache@example.com").await;

    // Alice creates a vehicle — populates her slice of the cache on the first list.
    app.client
        .post("/vehicles")
        .authorization_bearer(&a.access)
        .json(&json!({
            "brand": "Suzuki",
            "model": "Ertiga",
            "year": 2019,
            "plate_number": "B 9991 AAX",
            "fuel_type": "petrol",
            "current_odometer": 10000
        }))
        .await;

    // Warm Alice's cache.
    let resp = app
        .client
        .get("/vehicles")
        .authorization_bearer(&a.access)
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["data"].as_array().unwrap().len(), 1);

    // Bob lists — must see 0 (his own empty list), never Alice's.
    let resp = app
        .client
        .get("/vehicles")
        .authorization_bearer(&b.access)
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(
        body["data"].as_array().unwrap().len(),
        0,
        "Bob must not see Alice's cached vehicles"
    );
}

/// After a vehicle is updated via PUT, the detail endpoint must return the
/// fresh value — not a stale cached one.
#[tokio::test]
async fn update_invalidates_detail_cache() {
    let app = common::spawn_app().await;
    let s = common::register_and_login(&app, "detail_cache@example.com").await;

    // Create vehicle.
    let resp = app
        .client
        .post("/vehicles")
        .authorization_bearer(&s.access)
        .json(&json!({
            "brand": "Mitsubishi",
            "model": "Xpander",
            "year": 2022,
            "plate_number": "B 7777 XXX",
            "fuel_type": "petrol",
            "current_odometer": 0
        }))
        .await;
    let body: serde_json::Value = resp.json();
    let vehicle_id = body["data"]["id"].as_str().unwrap();

    // Prime the detail cache.
    let resp = app
        .client
        .get(&format!("/vehicles/{vehicle_id}"))
        .authorization_bearer(&s.access)
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["data"]["brand"].as_str().unwrap(), "Mitsubishi");

    // Update the vehicle — bumps cache version.
    app.client
        .put(&format!("/vehicles/{vehicle_id}"))
        .authorization_bearer(&s.access)
        .json(&json!({
            "brand": "Mitsubishi",
            "model": "Xpander Cross",
            "year": 2023,
            "color": "White",
            "fuel_type": "petrol",
            "current_odometer": 5000
        }))
        .await;

    // Detail must reflect the update, not the cached old value.
    let resp = app
        .client
        .get(&format!("/vehicles/{vehicle_id}"))
        .authorization_bearer(&s.access)
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(
        body["data"]["model"].as_str().unwrap(),
        "Xpander Cross",
        "expected updated model — stale cache must not be served"
    );
    assert_eq!(body["data"]["year"].as_u64().unwrap(), 2023);
}

/// After `GET /vehicles` populates the list cache, the Redis key must carry a
/// positive TTL (the backstop so stale data self-heals if an INCR is lost).
///
/// Sequence:
/// 1. register + login → obtain user_id from response body
/// 2. POST /vehicles → insert bumps version to 1
/// 3. GET /vehicles → cache miss → `set_json` writes `cache:v1:vehicles:{uid}`
///    with EX 300
/// 4. `TTL cache:v1:vehicles:{uid}` via raw fred client → must be > 0 and ≤ 300
#[tokio::test]
async fn cached_list_entry_has_ttl() {
    let app = common::spawn_app().await;

    // Register + login — capture user_id from the JSON response body.
    let s = common::register_and_login(&app, "ttl_test@example.com").await;
    let me_resp = app.client.get("/me").authorization_bearer(&s.access).await;
    let me_body: serde_json::Value = me_resp.json();
    let user_id: Uuid = me_body["data"]["id"]
        .as_str()
        .expect("me response must have id field")
        .parse()
        .expect("id must be a valid UUID");

    // Insert one vehicle — bumps version counter to 1.
    app.client
        .post("/vehicles")
        .authorization_bearer(&s.access)
        .json(&json!({
            "brand": "Daihatsu",
            "model": "Rocky",
            "year": 2023,
            "plate_number": "B 5555 TTL",
            "fuel_type": "petrol",
            "current_odometer": 0
        }))
        .await;

    // GET /vehicles — cache miss at v1 → populates cache:v1:vehicles:{user_id}
    // with EX 300.
    let resp = app
        .client
        .get("/vehicles")
        .authorization_bearer(&s.access)
        .await;
    resp.assert_status_ok();

    // Build the expected list cache key (version=1, same formula as RedisCache).
    let list_key = RedisCache::list_key(1, user_id);

    // Run `TTL <list_key>` via the raw fred pool from TestApp.
    let ttl_secs: i64 = app
        .redis_pool
        .ttl(&list_key)
        .await
        .expect("TTL command must succeed");

    assert!(
        ttl_secs > 0,
        "list cache key '{list_key}' must have a positive TTL (got {ttl_secs}); \
         backstop TTL was not set by set_json"
    );
    assert!(
        ttl_secs <= 300,
        "list cache key TTL {ttl_secs}s exceeds VEHICLE_CACHE_TTL_SECS=300"
    );
}
