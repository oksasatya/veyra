mod common;
use serde_json::json;

/// Helper: create a vehicle and return its id string.
async fn create_vehicle(app: &common::TestApp, token: &str, plate: &str) -> String {
    let v: serde_json::Value = app
        .client
        .post("/vehicles")
        .authorization_bearer(token)
        .json(&json!({
            "brand": "Toyota", "model": "Avanza", "year": 2021,
            "plate_number": plate, "fuel_type": "petrol", "current_odometer": 0
        }))
        .await
        .json();
    v["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn create_and_list_reminders() {
    let app = common::spawn_app().await;
    let token = common::register_and_login(&app, "reminder_list@example.com").await;
    let vid = create_vehicle(&app, &token, "B 0001 REM").await;

    let resp = app
        .client
        .post(&format!("/vehicles/{vid}/reminders"))
        .authorization_bearer(&token)
        .json(&json!({
            "title": "Oil change",
            "reminder_type": "date",
            "due_date": "2026-12-01"
        }))
        .await;
    resp.assert_status(axum::http::StatusCode::CREATED);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["title"].as_str().unwrap(), "Oil change");
    assert_eq!(body["reminder_type"].as_str().unwrap(), "date");
    assert!(!body["is_completed"].as_bool().unwrap());

    let list: serde_json::Value = app
        .client
        .get(&format!("/vehicles/{vid}/reminders"))
        .authorization_bearer(&token)
        .await
        .json();
    assert_eq!(list["reminders"].as_array().unwrap().len(), 1);
    assert_eq!(
        list["reminders"][0]["title"].as_str().unwrap(),
        "Oil change"
    );
}

#[tokio::test]
async fn mark_reminder_complete() {
    let app = common::spawn_app().await;
    let token = common::register_and_login(&app, "reminder_patch@example.com").await;
    let vid = create_vehicle(&app, &token, "B 0002 REM").await;

    // Create a reminder
    let created: serde_json::Value = app
        .client
        .post(&format!("/vehicles/{vid}/reminders"))
        .authorization_bearer(&token)
        .json(&json!({
            "title": "Tire rotation",
            "reminder_type": "odometer",
            "due_odometer": 50000
        }))
        .await
        .json();
    let rid = created["id"].as_str().unwrap();

    // PATCH to mark complete
    let resp = app
        .client
        .patch(&format!("/vehicles/{vid}/reminders/{rid}"))
        .authorization_bearer(&token)
        .json(&json!({ "is_completed": true }))
        .await;
    resp.assert_status(axum::http::StatusCode::OK);
    let updated: serde_json::Value = resp.json();
    assert!(updated["is_completed"].as_bool().unwrap());

    // List confirms it's complete
    let list: serde_json::Value = app
        .client
        .get(&format!("/vehicles/{vid}/reminders"))
        .authorization_bearer(&token)
        .await
        .json();
    assert!(list["reminders"][0]["is_completed"].as_bool().unwrap());
}

#[tokio::test]
async fn date_reminder_without_due_date_returns_422() {
    let app = common::spawn_app().await;
    let token = common::register_and_login(&app, "reminder_422@example.com").await;
    let vid = create_vehicle(&app, &token, "B 0003 REM").await;

    let resp = app
        .client
        .post(&format!("/vehicles/{vid}/reminders"))
        .authorization_bearer(&token)
        .json(&json!({
            "title": "Missing due_date",
            "reminder_type": "date"
            // due_date intentionally omitted
        }))
        .await;
    resp.assert_status(axum::http::StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn odometer_reminder_without_due_odometer_returns_422() {
    let app = common::spawn_app().await;
    let token = common::register_and_login(&app, "reminder_422b@example.com").await;
    let vid = create_vehicle(&app, &token, "B 0004 REM").await;

    let resp = app
        .client
        .post(&format!("/vehicles/{vid}/reminders"))
        .authorization_bearer(&token)
        .json(&json!({
            "title": "Missing odometer",
            "reminder_type": "odometer"
            // due_odometer intentionally omitted
        }))
        .await;
    resp.assert_status(axum::http::StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn invalid_reminder_type_returns_422() {
    let app = common::spawn_app().await;
    let token = common::register_and_login(&app, "reminder_invalid_type@example.com").await;
    let vid = create_vehicle(&app, &token, "B 0005 REM").await;

    let resp = app
        .client
        .post(&format!("/vehicles/{vid}/reminders"))
        .authorization_bearer(&token)
        .json(&json!({
            "title": "Bad type",
            "reminder_type": "invalid_type",
            "due_date": "2026-12-01"
        }))
        .await;
    resp.assert_status(axum::http::StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn reminder_for_other_users_vehicle_returns_404() {
    let app = common::spawn_app().await;
    let token_a = common::register_and_login(&app, "reminder_owner@example.com").await;
    let token_b = common::register_and_login(&app, "reminder_intruder@example.com").await;

    let vid = create_vehicle(&app, &token_a, "B 0006 REM").await;

    let resp = app
        .client
        .post(&format!("/vehicles/{vid}/reminders"))
        .authorization_bearer(&token_b)
        .json(&json!({
            "title": "Unauthorised reminder",
            "reminder_type": "date",
            "due_date": "2026-12-01"
        }))
        .await;
    resp.assert_status(axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn patch_both_type_reminder_preserves_existing_due_date() {
    let app = common::spawn_app().await;
    let token = common::register_and_login(&app, "reminder_both_patch@example.com").await;
    let vid = create_vehicle(&app, &token, "B 0007 REM").await;

    // Create a both-type reminder with both triggers
    let created: serde_json::Value = app
        .client
        .post(&format!("/vehicles/{vid}/reminders"))
        .authorization_bearer(&token)
        .json(&json!({
            "title": "Full service",
            "reminder_type": "both",
            "due_date": "2027-01-01",
            "due_odometer": 60000
        }))
        .await
        .json();
    let rid = created["id"].as_str().unwrap();

    // PATCH only is_completed — due triggers preserved via merge
    let resp = app
        .client
        .patch(&format!("/vehicles/{vid}/reminders/{rid}"))
        .authorization_bearer(&token)
        .json(&json!({ "is_completed": true }))
        .await;
    resp.assert_status(axum::http::StatusCode::OK);
    let updated: serde_json::Value = resp.json();
    assert!(updated["is_completed"].as_bool().unwrap());
    assert_eq!(updated["due_date"].as_str().unwrap(), "2027-01-01");
    assert_eq!(updated["due_odometer"].as_u64().unwrap(), 60000);
}
