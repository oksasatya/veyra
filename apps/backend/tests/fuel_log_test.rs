mod common;
use serde_json::json;

#[tokio::test]
async fn create_and_list_fuel_logs() {
    let app = common::spawn_app().await;
    let s = common::register_and_login(&app, "fuel@example.com").await;

    let v: serde_json::Value = app
        .client
        .post("/vehicles")
        .authorization_bearer(&s.access)
        .json(&json!({
            "brand": "Honda", "model": "Brio", "year": 2022,
            "plate_number": "B 0001 FUL", "fuel_type": "petrol", "current_odometer": 0
        }))
        .await
        .json();
    let vid = v["data"]["id"].as_str().unwrap();

    let resp = app
        .client
        .post(&format!("/vehicles/{vid}/fuel-logs"))
        .authorization_bearer(&s.access)
        .json(&json!({
            "log_date": "2026-01-20",
            "odometer": 10000,
            "liters": "40.0",
            "price_per_liter": "10000.0",
            "station": "Shell",
            "is_full_tank": true
        }))
        .await;
    resp.assert_status(axum::http::StatusCode::CREATED);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["data"]["total_cost"].as_str().unwrap(), "400000.00");

    let list: serde_json::Value = app
        .client
        .get(&format!("/vehicles/{vid}/fuel-logs"))
        .authorization_bearer(&s.access)
        .await
        .json();
    assert_eq!(list["data"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn fuel_log_for_other_users_vehicle_returns_404() {
    let app = common::spawn_app().await;
    let a = common::register_and_login(&app, "owner@fuel.com").await;
    let b = common::register_and_login(&app, "intruder@fuel.com").await;

    let v: serde_json::Value = app
        .client
        .post("/vehicles")
        .authorization_bearer(&a.access)
        .json(&json!({
            "brand": "Honda", "model": "Brio", "year": 2022,
            "plate_number": "B 0002 FUL", "fuel_type": "petrol", "current_odometer": 0
        }))
        .await
        .json();
    let vid = v["data"]["id"].as_str().unwrap();

    let resp = app
        .client
        .post(&format!("/vehicles/{vid}/fuel-logs"))
        .authorization_bearer(&b.access)
        .json(&json!({
            "log_date": "2026-01-20",
            "odometer": 5000,
            "liters": "30.0",
            "price_per_liter": "9000.0",
            "is_full_tank": false
        }))
        .await;
    resp.assert_status(axum::http::StatusCode::NOT_FOUND);
}
