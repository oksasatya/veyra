mod common;
use serde_json::json;

#[tokio::test]
async fn create_and_list_expenses() {
    let app = common::spawn_app().await;
    let s = common::register_and_login(&app, "expense@example.com").await;
    let (cn, cv) = common::csrf_header(&s.csrf);

    let v: serde_json::Value = app
        .client
        .post("/vehicles")
        .add_cookies(s.cookies.clone())
        .add_header(cn.clone(), cv.clone())
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
        .add_cookies(s.cookies.clone())
        .add_header(cn.clone(), cv.clone())
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
        .add_cookies(s.cookies.clone())
        .await
        .json();
    assert_eq!(list["expenses"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn expense_invalid_category_returns_422() {
    let app = common::spawn_app().await;
    let s = common::register_and_login(&app, "exp_invalid@example.com").await;
    let (cn, cv) = common::csrf_header(&s.csrf);

    let v: serde_json::Value = app
        .client
        .post("/vehicles")
        .add_cookies(s.cookies.clone())
        .add_header(cn.clone(), cv.clone())
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
        .add_cookies(s.cookies.clone())
        .add_header(cn, cv)
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
    let a = common::register_and_login(&app, "owner@expense.com").await;
    let b = common::register_and_login(&app, "intruder@expense.com").await;
    let (a_cn, a_cv) = common::csrf_header(&a.csrf);
    let (b_cn, b_cv) = common::csrf_header(&b.csrf);

    let v: serde_json::Value = app
        .client
        .post("/vehicles")
        .add_cookies(a.cookies.clone())
        .add_header(a_cn, a_cv)
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
        .add_cookies(b.cookies.clone())
        .add_header(b_cn, b_cv)
        .json(&json!({
            "expense_date": "2026-06-15",
            "category": "battery",
            "description": "New battery",
            "amount": "500000.00"
        }))
        .await;
    resp.assert_status(axum::http::StatusCode::NOT_FOUND);
}
