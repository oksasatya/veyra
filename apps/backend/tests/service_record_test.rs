mod common;
use serde_json::json;

#[tokio::test]
async fn create_and_list_service_records() {
    let app = common::spawn_app().await;

    // Register + create vehicle
    app.client
        .post("/auth/register")
        .json(&json!({
            "email": "svc@example.com", "password": "password123", "name": "S"
        }))
        .await;
    let login = app
        .client
        .post("/auth/login")
        .json(&json!({
            "email": "svc@example.com", "password": "password123"
        }))
        .await;
    let login_body: serde_json::Value = login.json();
    let token = login_body["token"].as_str().unwrap();
    let auth_header: axum::http::HeaderValue = format!("Bearer {token}").parse().unwrap();
    let auth_name: axum::http::HeaderName = "Authorization".parse().unwrap();

    let v = app
        .client
        .post("/vehicles")
        .add_header(auth_name.clone(), auth_header.clone())
        .json(&json!({
            "brand": "Toyota", "model": "Avanza", "year": 2020,
            "plate_number": "B 0001 SVC", "fuel_type": "petrol", "current_odometer": 0
        }))
        .await;
    let v_body: serde_json::Value = v.json();
    let vid = v_body["id"].as_str().unwrap();

    // Create service record
    let resp = app
        .client
        .post(&format!("/vehicles/{vid}/services"))
        .add_header(auth_name.clone(), auth_header.clone())
        .json(&json!({
            "service_date": "2026-01-15",
            "odometer": 5000,
            "description": "Oil change",
            "workshop": "Fast Lube",
            "cost": "150.00"
        }))
        .await;
    resp.assert_status(axum::http::StatusCode::CREATED);

    // List service records
    let list = app
        .client
        .get(&format!("/vehicles/{vid}/services"))
        .add_header(auth_name, auth_header)
        .await;
    list.assert_status_ok();
    let list_body: serde_json::Value = list.json();
    assert_eq!(list_body["records"].as_array().unwrap().len(), 1);
    assert_eq!(
        list_body["records"][0]["description"].as_str().unwrap(),
        "Oil change"
    );
}

#[tokio::test]
async fn service_record_for_other_users_vehicle_returns_404() {
    let app = common::spawn_app().await;
    let token_a = common::register_and_login(&app, "owner@svc.com").await;
    let token_b = common::register_and_login(&app, "intruder@svc.com").await;

    // Owner creates a vehicle
    let v = app
        .client
        .post("/vehicles")
        .authorization_bearer(&token_a)
        .json(&json!({
            "brand": "Honda", "model": "Brio", "year": 2021,
            "plate_number": "B 0002 SVC", "fuel_type": "petrol", "current_odometer": 0
        }))
        .await;
    let v_body: serde_json::Value = v.json();
    let vid = v_body["id"].as_str().unwrap();

    // Intruder tries to create a service record on owner's vehicle
    let resp = app
        .client
        .post(&format!("/vehicles/{vid}/services"))
        .authorization_bearer(&token_b)
        .json(&json!({
            "service_date": "2026-01-15",
            "odometer": 1000,
            "description": "Unauthorized service"
        }))
        .await;
    resp.assert_status(axum::http::StatusCode::NOT_FOUND);
}
