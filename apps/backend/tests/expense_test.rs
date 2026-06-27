mod common;
use serde_json::json;

#[tokio::test]
async fn create_and_list_expenses() {
    let app = common::spawn_app().await;
    let token = common::register_and_login(&app, "expense@example.com").await;
    let auth_h: axum::http::HeaderValue = format!("Bearer {token}").parse().unwrap();
    let auth_n: axum::http::HeaderName = "Authorization".parse().unwrap();

    let v: serde_json::Value = app
        .client
        .post("/vehicles")
        .add_header(auth_n.clone(), auth_h.clone())
        .json(&json!({
            "brand": "Toyota", "model": "Avanza", "year": 2021,
            "plate_number": "B 0001 EXP", "fuel_type": "petrol", "current_odometer": 0
        }))
        .await
        .json();
    let vid = v["id"].as_str().unwrap();

    let resp = app
        .client
        .post(&format!("/vehicles/{vid}/expenses"))
        .add_header(auth_n.clone(), auth_h.clone())
        .json(&json!({
            "expense_date": "2026-06-15",
            "category": "tire",
            "description": "Front tire replacement",
            "amount": "350000.00"
        }))
        .await;
    resp.assert_status(axum::http::StatusCode::CREATED);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["category"].as_str().unwrap(), "tire");
    assert_eq!(body["amount"].as_str().unwrap(), "350000.00");

    let list: serde_json::Value = app
        .client
        .get(&format!("/vehicles/{vid}/expenses"))
        .add_header(auth_n, auth_h)
        .await
        .json();
    assert_eq!(list["expenses"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn expense_invalid_category_returns_422() {
    let app = common::spawn_app().await;
    let token = common::register_and_login(&app, "exp_invalid@example.com").await;
    let auth_h: axum::http::HeaderValue = format!("Bearer {token}").parse().unwrap();
    let auth_n: axum::http::HeaderName = "Authorization".parse().unwrap();

    let v: serde_json::Value = app
        .client
        .post("/vehicles")
        .add_header(auth_n.clone(), auth_h.clone())
        .json(&json!({
            "brand": "Honda", "model": "Jazz", "year": 2020,
            "plate_number": "B 0002 EXP", "fuel_type": "petrol", "current_odometer": 0
        }))
        .await
        .json();
    let vid = v["id"].as_str().unwrap();

    let resp = app
        .client
        .post(&format!("/vehicles/{vid}/expenses"))
        .add_header(auth_n, auth_h)
        .json(&json!({
            "expense_date": "2026-06-15",
            "category": "INVALID_CATEGORY",
            "description": "Should fail",
            "amount": "100.00"
        }))
        .await;
    resp.assert_status(axum::http::StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn expense_for_other_users_vehicle_returns_404() {
    let app = common::spawn_app().await;
    let token_a = common::register_and_login(&app, "owner@expense.com").await;
    let token_b = common::register_and_login(&app, "intruder@expense.com").await;

    let v: serde_json::Value = app
        .client
        .post("/vehicles")
        .authorization_bearer(&token_a)
        .json(&json!({
            "brand": "Honda", "model": "Brio", "year": 2022,
            "plate_number": "B 0003 EXP", "fuel_type": "petrol", "current_odometer": 0
        }))
        .await
        .json();
    let vid = v["id"].as_str().unwrap();

    let resp = app
        .client
        .post(&format!("/vehicles/{vid}/expenses"))
        .authorization_bearer(&token_b)
        .json(&json!({
            "expense_date": "2026-06-15",
            "category": "battery",
            "description": "New battery",
            "amount": "500000.00"
        }))
        .await;
    resp.assert_status(axum::http::StatusCode::NOT_FOUND);
}
