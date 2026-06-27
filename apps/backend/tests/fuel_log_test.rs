mod common;
use serde_json::json;

#[tokio::test]
async fn create_and_list_fuel_logs() {
    let app = common::spawn_app().await;
    let token = common::register_and_login(&app, "fuel@example.com").await;
    let auth_h: axum::http::HeaderValue = format!("Bearer {token}").parse().unwrap();
    let auth_n: axum::http::HeaderName = "Authorization".parse().unwrap();

    let v: serde_json::Value = app
        .client
        .post("/vehicles")
        .add_header(auth_n.clone(), auth_h.clone())
        .json(&json!({
            "brand": "Honda", "model": "Brio", "year": 2022,
            "plate_number": "B 0001 FUL", "fuel_type": "petrol", "current_odometer": 0
        }))
        .await
        .json();
    let vid = v["id"].as_str().unwrap();

    let resp = app
        .client
        .post(&format!("/vehicles/{vid}/fuel-logs"))
        .add_header(auth_n.clone(), auth_h.clone())
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
    assert_eq!(body["total_cost"].as_str().unwrap(), "400000.00");

    let list: serde_json::Value = app
        .client
        .get(&format!("/vehicles/{vid}/fuel-logs"))
        .add_header(auth_n, auth_h)
        .await
        .json();
    assert_eq!(list["logs"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn fuel_log_for_other_users_vehicle_returns_404() {
    let app = common::spawn_app().await;
    let token_a = common::register_and_login(&app, "owner@fuel.com").await;
    let token_b = common::register_and_login(&app, "intruder@fuel.com").await;

    let v: serde_json::Value = app
        .client
        .post("/vehicles")
        .authorization_bearer(&token_a)
        .json(&json!({
            "brand": "Honda", "model": "Brio", "year": 2022,
            "plate_number": "B 0002 FUL", "fuel_type": "petrol", "current_odometer": 0
        }))
        .await
        .json();
    let vid = v["id"].as_str().unwrap();

    let resp = app
        .client
        .post(&format!("/vehicles/{vid}/fuel-logs"))
        .authorization_bearer(&token_b)
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
