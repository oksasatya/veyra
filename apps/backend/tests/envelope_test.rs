mod common;

use serde_json::json;

/// A freshly-registered user defaults to English until they change it.
#[tokio::test]
async fn me_returns_preferred_language_default_en() {
    let app = common::spawn_app().await;
    let s = common::register_and_login(&app, "lang-default@example.com").await;

    let resp = app.client.get("/me").authorization_bearer(&s.access).await;

    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["data"]["preferred_language"].as_str().unwrap(), "en");
    // The success envelope always carries a request id.
    assert!(body["meta"]["request_id"].is_string());
}

/// PATCH /me updates the preferred language; the change persists to a later GET.
#[tokio::test]
async fn patch_me_updates_preferred_language() {
    let app = common::spawn_app().await;
    let s = common::register_and_login(&app, "lang-update@example.com").await;

    let resp = app
        .client
        .patch("/me")
        .authorization_bearer(&s.access)
        .json(&json!({ "preferred_language": "id" }))
        .await;

    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["data"]["preferred_language"].as_str().unwrap(), "id");

    // Persisted: a subsequent GET /me reflects the new language.
    let me = app.client.get("/me").authorization_bearer(&s.access).await;
    let me_body: serde_json::Value = me.json();
    assert_eq!(
        me_body["data"]["preferred_language"].as_str().unwrap(),
        "id"
    );
}

/// An unsupported language code is rejected as a 422 with the stable code.
#[tokio::test]
async fn patch_me_rejects_unsupported_language() {
    let app = common::spawn_app().await;
    let s = common::register_and_login(&app, "lang-bad@example.com").await;

    let resp = app
        .client
        .patch("/me")
        .authorization_bearer(&s.access)
        .json(&json!({ "preferred_language": "fr" }))
        .await;

    resp.assert_status(axum::http::StatusCode::UNPROCESSABLE_ENTITY);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["error"]["code"].as_str().unwrap(), "INVALID_LANGUAGE");
    assert!(body["error"]["message"].is_string());
    assert!(body["meta"]["request_id"].is_string());
}

/// Error responses use the envelope: a stable `error.code`, a `meta.request_id`,
/// and no `data` key.
#[tokio::test]
async fn not_found_uses_error_envelope() {
    let app = common::spawn_app().await;
    let s = common::register_and_login(&app, "envelope-404@example.com").await;

    // A well-formed UUID that the just-registered user does not own.
    let missing = "11111111-1111-1111-1111-111111111111";
    let resp = app
        .client
        .get(&format!("/vehicles/{missing}"))
        .authorization_bearer(&s.access)
        .await;

    resp.assert_status(axum::http::StatusCode::NOT_FOUND);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["error"]["code"].as_str().unwrap(), "NOT_FOUND");
    assert!(body["meta"]["request_id"].is_string());
    assert!(body["data"].is_null());
}

/// Every response echoes the request id as an `X-Request-Id` header.
#[tokio::test]
async fn response_carries_request_id_header() {
    let app = common::spawn_app().await;
    let resp = app.client.get("/health").await;
    assert!(resp.headers().get("x-request-id").is_some());
}
