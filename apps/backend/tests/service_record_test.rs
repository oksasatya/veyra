mod common;
use serde_json::json;

#[tokio::test]
async fn create_and_list_service_records() {
    let app = common::spawn_app().await;
    let s = common::register_and_login(&app, "svc@example.com").await;
    let (cn, cv) = common::csrf_header(&s.csrf);

    let v = app
        .client
        .post("/vehicles")
        .add_cookies(s.cookies.clone())
        .add_header(cn.clone(), cv.clone())
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
        .add_cookies(s.cookies.clone())
        .add_header(cn.clone(), cv.clone())
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
        .add_cookies(s.cookies.clone())
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
    let a = common::register_and_login(&app, "owner@svc.com").await;
    let b = common::register_and_login(&app, "intruder@svc.com").await;
    let (a_cn, a_cv) = common::csrf_header(&a.csrf);
    let (b_cn, b_cv) = common::csrf_header(&b.csrf);

    // Owner creates a vehicle
    let v = app
        .client
        .post("/vehicles")
        .add_cookies(a.cookies.clone())
        .add_header(a_cn, a_cv)
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
        .add_cookies(b.cookies.clone())
        .add_header(b_cn, b_cv)
        .json(&json!({
            "service_date": "2026-01-15",
            "odometer": 1000,
            "description": "Unauthorized service"
        }))
        .await;
    resp.assert_status(axum::http::StatusCode::NOT_FOUND);
}
