mod common;
use serde_json::json;

#[tokio::test]
async fn create_and_list_documents() {
    let app = common::spawn_app().await;
    let s = common::register_and_login(&app, "doc@example.com").await;

    let v: serde_json::Value = app
        .client
        .post("/vehicles")
        .authorization_bearer(&s.access)
        .json(&json!({
            "brand": "Toyota", "model": "Avanza", "year": 2021,
            "plate_number": "B 0001 DOC", "fuel_type": "petrol", "current_odometer": 0
        }))
        .await
        .json();
    let vid = v["data"]["id"].as_str().unwrap();

    let resp = app
        .client
        .post(&format!("/vehicles/{vid}/documents"))
        .authorization_bearer(&s.access)
        .json(&json!({
            "doc_type": "stnk",
            "title": "STNK 2026",
            "expiry_date": "2026-12-31",
            "file_url": "https://storage.example.com/stnk.pdf"
        }))
        .await;
    resp.assert_status(axum::http::StatusCode::CREATED);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["data"]["doc_type"].as_str().unwrap(), "stnk");
    assert_eq!(body["data"]["title"].as_str().unwrap(), "STNK 2026");
    assert_eq!(body["data"]["expiry_date"].as_str().unwrap(), "2026-12-31");
    assert_eq!(
        body["data"]["file_url"].as_str().unwrap(),
        "https://storage.example.com/stnk.pdf"
    );

    let list: serde_json::Value = app
        .client
        .get(&format!("/vehicles/{vid}/documents"))
        .authorization_bearer(&s.access)
        .await
        .json();
    assert_eq!(list["data"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn document_invalid_doc_type_returns_422() {
    let app = common::spawn_app().await;
    let s = common::register_and_login(&app, "doc_invalid@example.com").await;

    let v: serde_json::Value = app
        .client
        .post("/vehicles")
        .authorization_bearer(&s.access)
        .json(&json!({
            "brand": "Honda", "model": "Jazz", "year": 2020,
            "plate_number": "B 0002 DOC", "fuel_type": "petrol", "current_odometer": 0
        }))
        .await
        .json();
    let vid = v["data"]["id"].as_str().unwrap();

    let resp = app
        .client
        .post(&format!("/vehicles/{vid}/documents"))
        .authorization_bearer(&s.access)
        .json(&json!({
            "doc_type": "INVALID_TYPE",
            "title": "Should fail"
        }))
        .await;
    resp.assert_status(axum::http::StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn document_for_other_users_vehicle_returns_404() {
    let app = common::spawn_app().await;
    let a = common::register_and_login(&app, "owner@doc.com").await;
    let b = common::register_and_login(&app, "intruder@doc.com").await;

    let v: serde_json::Value = app
        .client
        .post("/vehicles")
        .authorization_bearer(&a.access)
        .json(&json!({
            "brand": "Honda", "model": "Brio", "year": 2022,
            "plate_number": "B 0003 DOC", "fuel_type": "petrol", "current_odometer": 0
        }))
        .await
        .json();
    let vid = v["data"]["id"].as_str().unwrap();

    let resp = app
        .client
        .post(&format!("/vehicles/{vid}/documents"))
        .authorization_bearer(&b.access)
        .json(&json!({
            "doc_type": "bpkb",
            "title": "BPKB"
        }))
        .await;
    resp.assert_status(axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn document_without_optional_fields() {
    let app = common::spawn_app().await;
    let s = common::register_and_login(&app, "doc_minimal@example.com").await;

    let v: serde_json::Value = app
        .client
        .post("/vehicles")
        .authorization_bearer(&s.access)
        .json(&json!({
            "brand": "Suzuki", "model": "Ertiga", "year": 2023,
            "plate_number": "B 0004 DOC", "fuel_type": "petrol", "current_odometer": 0
        }))
        .await
        .json();
    let vid = v["data"]["id"].as_str().unwrap();

    let resp = app
        .client
        .post(&format!("/vehicles/{vid}/documents"))
        .authorization_bearer(&s.access)
        .json(&json!({
            "doc_type": "insurance",
            "title": "Insurance 2026"
        }))
        .await;
    resp.assert_status(axum::http::StatusCode::CREATED);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["data"]["doc_type"].as_str().unwrap(), "insurance");
    assert!(body["data"]["expiry_date"].is_null());
    assert!(body["data"]["file_url"].is_null());
}
