mod common;

use serde_json::json;

/// Helper: register+login, create a vehicle, return (session, vehicle_id).
async fn setup(app: &common::TestApp, email: &str, plate: &str) -> (common::Session, String) {
    let s = common::register_and_login(app, email).await;
    let (cn, cv) = common::csrf_header(&s.csrf);
    let v: serde_json::Value = app
        .client
        .post("/vehicles")
        .add_cookies(s.cookies.clone())
        .add_header(cn, cv)
        .json(&json!({
            "brand": "Toyota", "model": "Avanza", "year": 2021,
            "plate_number": plate,
            "fuel_type": "petrol",
            "current_odometer": 10000
        }))
        .await
        .json();
    let vid = v["id"].as_str().unwrap().to_string();
    (s, vid)
}

/// Summary endpoint returns correct aggregated data (2 services, 1 fuel, 1 expense, 1 reminder).
///
/// A second call within TTL must return the same values — served from cache.
/// This verifies: (a) correctness, (b) cache hit idempotence.
#[tokio::test]
async fn summary_cached_within_ttl() {
    let app = common::spawn_app().await;
    let (s, vid) = setup(&app, "cache_summary@example.com", "B 1001 CSM").await;
    let (cn, cv) = common::csrf_header(&s.csrf);

    // Seed: 2 service records
    app.client
        .post(&format!("/vehicles/{vid}/services"))
        .add_cookies(s.cookies.clone())
        .add_header(cn.clone(), cv.clone())
        .json(&json!({
            "service_date": "2026-01-10",
            "odometer": 10500,
            "description": "Oil change",
            "cost": "100.00"
        }))
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    app.client
        .post(&format!("/vehicles/{vid}/services"))
        .add_cookies(s.cookies.clone())
        .add_header(cn.clone(), cv.clone())
        .json(&json!({
            "service_date": "2026-02-15",
            "odometer": 11000,
            "description": "Brake pads",
            "cost": "200.00"
        }))
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    // Seed: 1 fuel log
    app.client
        .post(&format!("/vehicles/{vid}/fuel-logs"))
        .add_cookies(s.cookies.clone())
        .add_header(cn.clone(), cv.clone())
        .json(&json!({
            "log_date": "2026-03-01",
            "odometer": 11500,
            "liters": "40.00",
            "price_per_liter": "10.00",
            "is_full_tank": true
        }))
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    // Seed: 1 expense
    app.client
        .post(&format!("/vehicles/{vid}/expenses"))
        .add_cookies(s.cookies.clone())
        .add_header(cn.clone(), cv.clone())
        .json(&json!({
            "expense_date": "2026-04-01",
            "category": "tire",
            "description": "Front tires",
            "amount": "500.00"
        }))
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    // Seed: 1 upcoming reminder
    let due_date = (chrono::Utc::now() + chrono::Duration::days(7))
        .format("%Y-%m-%d")
        .to_string();
    app.client
        .post(&format!("/vehicles/{vid}/reminders"))
        .add_cookies(s.cookies.clone())
        .add_header(cn.clone(), cv.clone())
        .json(&json!({
            "title": "Tyre rotation",
            "reminder_type": "date",
            "due_date": due_date
        }))
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    // First call — cache miss, reads from DB and populates cache.
    let resp1 = app
        .client
        .get(&format!("/vehicles/{vid}/summary"))
        .add_cookies(s.cookies.clone())
        .await;
    resp1.assert_status_ok();
    let body1: serde_json::Value = resp1.json();

    assert_eq!(
        body1["total_services"].as_i64().unwrap(),
        2,
        "first call: service count"
    );
    assert_eq!(
        body1["total_service_cost"].as_str().unwrap(),
        "300.00",
        "first call: service cost"
    );
    assert_eq!(
        body1["total_refuels"].as_i64().unwrap(),
        1,
        "first call: refuel count"
    );
    assert_eq!(
        body1["total_fuel_cost"].as_str().unwrap(),
        "400.00",
        "first call: fuel cost"
    );
    assert_eq!(
        body1["total_expenses"].as_str().unwrap(),
        "500.00",
        "first call: expense amount"
    );
    assert_eq!(
        body1["upcoming_reminders"].as_i64().unwrap(),
        1,
        "first call: reminder count"
    );

    // Second call within TTL — same values (served from cache).
    let resp2 = app
        .client
        .get(&format!("/vehicles/{vid}/summary"))
        .add_cookies(s.cookies.clone())
        .await;
    resp2.assert_status_ok();
    let body2: serde_json::Value = resp2.json();

    assert_eq!(
        body1, body2,
        "second call within TTL must return identical values (cache hit)"
    );
}

/// User B's summary request for User A's vehicle returns 404.
/// User B also cannot read the cached summary that User A populated.
/// Key format is `cache:summary:{user_id}:{vehicle_id}` — scoped to the owner;
/// B requesting A's vehicle always goes to the DB (which returns NotFound).
#[tokio::test]
async fn cross_user_summary_isolation() {
    let app = common::spawn_app().await;
    let (a, vid) = setup(&app, "sum_alice@example.com", "B 2001 CSM").await;

    // Warm Alice's cache: call her own summary.
    let resp = app
        .client
        .get(&format!("/vehicles/{vid}/summary"))
        .add_cookies(a.cookies.clone())
        .await;
    resp.assert_status_ok();

    // Bob cannot see Alice's vehicle summary — must get 404, never Alice's cached data.
    let b = common::register_and_login(&app, "sum_bob@example.com").await;
    let resp = app
        .client
        .get(&format!("/vehicles/{vid}/summary"))
        .add_cookies(b.cookies.clone())
        .await;
    resp.assert_status(axum::http::StatusCode::NOT_FOUND);
}
