mod common;
use common::Session;
use serde_json::json;

/// Helper: register+login, create a vehicle, return (session, vehicle_id).
async fn setup(app: &common::TestApp, email: &str, plate: &str) -> (Session, String) {
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
    let vid = v["data"]["id"].as_str().unwrap().to_string();
    (s, vid)
}

/// Seed:
///   - 2 service records (cost 100.00 + 200.00 = 300.00 total)
///   - 1 fuel log (40 liters × 10.00 = 400.00 total_cost)
///   - 1 expense (amount 500.00)
///   - 1 reminder due within 30 days, not completed
///
/// Asserts: counts and costs match exactly.
#[tokio::test]
async fn summary_aggregates_correctly() {
    let app = common::spawn_app().await;
    let (s, vid) = setup(&app, "summary@example.com", "B 0001 SUM").await;
    let (cn, cv) = common::csrf_header(&s.csrf);

    // Service record 1 — cost 100
    app.client
        .post(&format!("/vehicles/{vid}/services"))
        .add_cookies(s.cookies.clone())
        .add_header(cn.clone(), cv.clone())
        .json(&json!({
            "service_date": "2026-01-10",
            "odometer": 10500,
            "description": "Oil change",
            "workshop": "FastLube",
            "cost": "100.00"
        }))
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    // Service record 2 — cost 200
    app.client
        .post(&format!("/vehicles/{vid}/services"))
        .add_cookies(s.cookies.clone())
        .add_header(cn.clone(), cv.clone())
        .json(&json!({
            "service_date": "2026-02-15",
            "odometer": 11000,
            "description": "Brake pads",
            "workshop": "FastLube",
            "cost": "200.00"
        }))
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    // Fuel log — 40 liters × 10.00 = 400.00 total_cost (GENERATED column)
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

    // Expense — 500.00
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

    // Reminder due in 7 days — within the 30-day window, not completed
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

    // GET /vehicles/{vid}/summary
    let resp = app
        .client
        .get(&format!("/vehicles/{vid}/summary"))
        .add_cookies(s.cookies.clone())
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();

    assert_eq!(
        body["data"]["total_services"].as_i64().unwrap(),
        2,
        "service count"
    );
    assert_eq!(
        body["data"]["total_service_cost"].as_str().unwrap(),
        "300.00",
        "service cost"
    );
    assert_eq!(
        body["data"]["total_refuels"].as_i64().unwrap(),
        1,
        "fuel log count"
    );
    assert_eq!(
        body["data"]["total_fuel_cost"].as_str().unwrap(),
        "400.00",
        "fuel cost"
    );
    assert_eq!(
        body["data"]["total_expenses"].as_str().unwrap(),
        "500.00",
        "expense amount"
    );
    assert_eq!(
        body["data"]["upcoming_reminders"].as_i64().unwrap(),
        1,
        "upcoming reminders"
    );
}

/// A completed reminder must NOT appear in upcoming_reminders, even if its
/// due_date is within 30 days.
#[tokio::test]
async fn completed_reminder_not_counted() {
    let app = common::spawn_app().await;
    let (s, vid) = setup(&app, "summary_completed@example.com", "B 0002 SUM").await;
    let (cn, cv) = common::csrf_header(&s.csrf);

    let due_date = (chrono::Utc::now() + chrono::Duration::days(5))
        .format("%Y-%m-%d")
        .to_string();
    let created: serde_json::Value = app
        .client
        .post(&format!("/vehicles/{vid}/reminders"))
        .add_cookies(s.cookies.clone())
        .add_header(cn.clone(), cv.clone())
        .json(&json!({
            "title": "Done reminder",
            "reminder_type": "date",
            "due_date": due_date
        }))
        .await
        .json();
    let rid = created["data"]["id"].as_str().unwrap();

    // Mark it complete
    app.client
        .patch(&format!("/vehicles/{vid}/reminders/{rid}"))
        .add_cookies(s.cookies.clone())
        .add_header(cn, cv)
        .json(&json!({ "is_completed": true }))
        .await
        .assert_status_ok();

    let resp = app
        .client
        .get(&format!("/vehicles/{vid}/summary"))
        .add_cookies(s.cookies.clone())
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(
        body["data"]["upcoming_reminders"].as_i64().unwrap(),
        0,
        "completed reminder must not be counted"
    );
}

/// Requesting the summary for a vehicle owned by another user returns 404.
#[tokio::test]
async fn summary_for_other_users_vehicle_returns_404() {
    let app = common::spawn_app().await;
    let (a, vid) = setup(&app, "summary_owner@example.com", "B 0003 SUM").await;
    let b = common::register_and_login(&app, "summary_intruder@example.com").await;

    // Silence the unused-variable warning from the owner session
    let _ = a;

    let resp = app
        .client
        .get(&format!("/vehicles/{vid}/summary"))
        .add_cookies(b.cookies.clone())
        .await;
    resp.assert_status(axum::http::StatusCode::NOT_FOUND);
}

/// A fresh vehicle with no records returns zeroes for all aggregates.
#[tokio::test]
async fn summary_empty_vehicle_returns_zeroes() {
    let app = common::spawn_app().await;
    let (s, vid) = setup(&app, "summary_empty@example.com", "B 0004 SUM").await;

    let resp = app
        .client
        .get(&format!("/vehicles/{vid}/summary"))
        .add_cookies(s.cookies.clone())
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();

    assert_eq!(body["data"]["total_services"].as_i64().unwrap(), 0);
    assert_eq!(body["data"]["total_service_cost"].as_str().unwrap(), "0");
    assert_eq!(body["data"]["total_refuels"].as_i64().unwrap(), 0);
    assert_eq!(body["data"]["total_fuel_cost"].as_str().unwrap(), "0");
    assert_eq!(body["data"]["total_expenses"].as_str().unwrap(), "0");
    assert_eq!(body["data"]["upcoming_reminders"].as_i64().unwrap(), 0);
}
