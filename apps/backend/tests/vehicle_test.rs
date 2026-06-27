mod common;
use serde_json::json;

async fn register_and_login(app: &common::TestApp, email: &str) -> String {
    app.client
        .post("/auth/register")
        .json(&json!({
            "email": email,
            "password": "password123",
            "name": "User"
        }))
        .await;
    let resp = app
        .client
        .post("/auth/login")
        .json(&json!({
            "email": email,
            "password": "password123"
        }))
        .await;
    let body: serde_json::Value = resp.json();
    body["token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn create_vehicle_returns_201() {
    let app = common::spawn_app().await;
    let token = register_and_login(&app, "car@example.com").await;

    let resp = app
        .client
        .post("/vehicles")
        .authorization_bearer(&token)
        .json(&json!({
            "brand": "Toyota",
            "model": "Avanza",
            "year": 2020,
            "plate_number": "B 1234 XYZ",
            "fuel_type": "petrol",
            "current_odometer": 0
        }))
        .await;

    resp.assert_status(axum::http::StatusCode::CREATED);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["brand"].as_str().unwrap(), "Toyota");
}

#[tokio::test]
async fn list_vehicles_returns_only_own_vehicles() {
    let app = common::spawn_app().await;
    let token_a = register_and_login(&app, "alice@cars.com").await;
    let token_b = register_and_login(&app, "bob@cars.com").await;

    // Alice creates a vehicle
    app.client
        .post("/vehicles")
        .authorization_bearer(&token_a)
        .json(&json!({
            "brand": "Honda",
            "model": "Brio",
            "year": 2021,
            "plate_number": "B 9999 AAA",
            "fuel_type": "petrol",
            "current_odometer": 0
        }))
        .await;

    // Bob lists vehicles — should be empty
    let resp = app
        .client
        .get("/vehicles")
        .authorization_bearer(&token_b)
        .await;

    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["vehicles"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn get_vehicle_not_owned_returns_404() {
    let app = common::spawn_app().await;
    let token_a = register_and_login(&app, "owner@cars.com").await;
    let token_b = register_and_login(&app, "intruder@cars.com").await;

    let created = app
        .client
        .post("/vehicles")
        .authorization_bearer(&token_a)
        .json(&json!({
            "brand": "Daihatsu",
            "model": "Xenia",
            "year": 2022,
            "plate_number": "D 5555 ZZZ",
            "fuel_type": "petrol",
            "current_odometer": 0
        }))
        .await;
    let vehicle: serde_json::Value = created.json();
    let id = vehicle["id"].as_str().unwrap();

    let resp = app
        .client
        .get(&format!("/vehicles/{id}"))
        .authorization_bearer(&token_b)
        .await;

    resp.assert_status(axum::http::StatusCode::NOT_FOUND);
}
