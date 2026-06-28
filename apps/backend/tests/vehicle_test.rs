mod common;
use serde_json::json;

#[tokio::test]
async fn create_vehicle_returns_201() {
    let app = common::spawn_app().await;
    let s = common::register_and_login(&app, "car@example.com").await;
    let (csrf_name, csrf_value) = common::csrf_header(&s.csrf);

    let resp = app
        .client
        .post("/vehicles")
        .add_cookies(s.cookies.clone())
        .add_header(csrf_name, csrf_value)
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
    assert_eq!(body["data"]["brand"].as_str().unwrap(), "Toyota");
}

#[tokio::test]
async fn list_vehicles_returns_only_own_vehicles() {
    let app = common::spawn_app().await;
    let a = common::register_and_login(&app, "alice@cars.com").await;
    let b = common::register_and_login(&app, "bob@cars.com").await;
    let (a_csrf_name, a_csrf_value) = common::csrf_header(&a.csrf);

    // Alice creates a vehicle
    app.client
        .post("/vehicles")
        .add_cookies(a.cookies.clone())
        .add_header(a_csrf_name, a_csrf_value)
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
        .add_cookies(b.cookies.clone())
        .await;

    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["data"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn get_vehicle_not_owned_returns_404() {
    let app = common::spawn_app().await;
    let a = common::register_and_login(&app, "owner@cars.com").await;
    let b = common::register_and_login(&app, "intruder@cars.com").await;
    let (a_csrf_name, a_csrf_value) = common::csrf_header(&a.csrf);

    let created = app
        .client
        .post("/vehicles")
        .add_cookies(a.cookies.clone())
        .add_header(a_csrf_name, a_csrf_value)
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
    let id = vehicle["data"]["id"].as_str().unwrap();

    let resp = app
        .client
        .get(&format!("/vehicles/{id}"))
        .add_cookies(b.cookies.clone())
        .await;

    resp.assert_status(axum::http::StatusCode::NOT_FOUND);
}
