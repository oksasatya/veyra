mod common;

use serde_json::json;

#[tokio::test]
async fn health_still_works() {
    let app = common::spawn_app().await;
    let resp = app.client.get("/health").await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["status"].as_str().unwrap(), "ok");
}

#[tokio::test]
async fn register_returns_201_with_user_and_tokens() {
    let app = common::spawn_app().await;

    let resp = app
        .client
        .post("/auth/register")
        .json(&json!({
            "email": "alice@example.com",
            "password": "password123",
            "name": "Alice"
        }))
        .await;

    resp.assert_status(axum::http::StatusCode::CREATED);
    let body: serde_json::Value = resp.json();
    assert_eq!(
        body["data"]["user"]["email"].as_str().unwrap(),
        "alice@example.com"
    );
    assert_eq!(body["data"]["user"]["name"].as_str().unwrap(), "Alice");
    assert!(body["data"]["tokens"]["access_token"].is_string());
    assert!(body["data"]["tokens"]["refresh_token"].is_string());
    // Bearer-only: no cookies are ever set.
    assert!(resp.maybe_cookie("veyra_access").is_none());
}

#[tokio::test]
async fn register_duplicate_email_returns_409() {
    let app = common::spawn_app().await;
    let payload = json!({
        "email": "dup@example.com",
        "password": "password123",
        "name": "Dup"
    });
    app.client.post("/auth/register").json(&payload).await;
    let resp = app.client.post("/auth/register").json(&payload).await;
    resp.assert_status_conflict();
}

#[tokio::test]
async fn login_returns_tokens() {
    let app = common::spawn_app().await;
    app.client
        .post("/auth/register")
        .json(&json!({
            "email": "login@example.com",
            "password": "password123",
            "name": "Login"
        }))
        .await;

    let resp = app
        .client
        .post("/auth/login")
        .json(&json!({
            "email": "login@example.com",
            "password": "password123"
        }))
        .await;

    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert!(body["data"]["tokens"]["access_token"].is_string());
    assert!(body["data"]["tokens"]["refresh_token"].is_string());
}

#[tokio::test]
async fn login_wrong_password_returns_401() {
    let app = common::spawn_app().await;
    app.client
        .post("/auth/register")
        .json(&json!({
            "email": "pass@example.com",
            "password": "password123",
            "name": "P"
        }))
        .await;

    let resp = app
        .client
        .post("/auth/login")
        .json(&json!({
            "email": "pass@example.com",
            "password": "wrongpassword"
        }))
        .await;

    resp.assert_status_unauthorized();
}

#[tokio::test]
async fn me_succeeds_with_authorization_header() {
    let app = common::spawn_app().await;
    let s = common::register_and_login(&app, "me@example.com").await;
    let (n, v) = common::bearer_header(&s.access);

    let resp = app.client.get("/me").add_header(n, v).await;

    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["data"]["email"].as_str().unwrap(), "me@example.com");
}

#[tokio::test]
async fn me_without_token_returns_401() {
    let app = common::spawn_app().await;
    let resp = app.client.get("/me").await;
    resp.assert_status_unauthorized();
}

#[tokio::test]
async fn me_with_invalid_token_returns_401() {
    let app = common::spawn_app().await;
    let (n, v) = common::bearer_header("not.a.real.jwt");

    let resp = app.client.get("/me").add_header(n, v).await;

    resp.assert_status_unauthorized();
}

#[tokio::test]
async fn mutation_with_bearer_needs_no_csrf() {
    let app = common::spawn_app().await;
    let s = common::register_and_login(&app, "mut@example.com").await;
    let (n, v) = common::bearer_header(&s.access);

    let resp = app
        .client
        .post("/vehicles")
        .add_header(n, v)
        .json(&json!({
            "brand": "Toyota", "model": "Avanza", "year": 2020,
            "plate_number": "B 1 ABC", "fuel_type": "petrol", "current_odometer": 1000
        }))
        .await;

    resp.assert_status(axum::http::StatusCode::CREATED);
}

#[tokio::test]
async fn refresh_rotates_and_returns_tokens() {
    let app = common::spawn_app().await;
    let s = common::register_and_login(&app, "br@e.com").await;

    let resp = app
        .client
        .post("/auth/refresh")
        .json(&json!({ "refresh_token": s.refresh }))
        .await;

    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert!(body["data"]["tokens"]["access_token"].is_string());
    assert!(body["data"]["tokens"]["refresh_token"].is_string());
}

#[tokio::test]
async fn refresh_missing_token_returns_401() {
    let app = common::spawn_app().await;
    let resp = app.client.post("/auth/refresh").await;
    resp.assert_status_unauthorized();
}

#[tokio::test]
async fn refresh_reuse_is_rejected_after_grace() {
    // grace = 0 → the old refresh secret is invalid the instant it rotates.
    let app = common::spawn_app_with_grace(0).await;
    let s = common::register_and_login(&app, "brr@e.com").await;

    app.client
        .post("/auth/refresh")
        .json(&json!({ "refresh_token": s.refresh.clone() }))
        .await
        .assert_status_ok();

    let resp = app
        .client
        .post("/auth/refresh")
        .json(&json!({ "refresh_token": s.refresh }))
        .await;

    resp.assert_status_unauthorized();
}

#[tokio::test]
async fn logout_then_refresh_is_unauthorized() {
    let app = common::spawn_app().await;
    let s = common::register_and_login(&app, "bl@e.com").await;

    app.client
        .post("/auth/logout")
        .json(&json!({ "refresh_token": s.refresh.clone() }))
        .await
        .assert_status(axum::http::StatusCode::NO_CONTENT);

    let resp = app
        .client
        .post("/auth/refresh")
        .json(&json!({ "refresh_token": s.refresh }))
        .await;

    resp.assert_status_unauthorized();
}

#[tokio::test]
async fn logout_revokes_session_then_me_returns_401() {
    let app = common::spawn_app().await;
    let s = common::register_and_login(&app, "logout@example.com").await;
    let (n, v) = common::bearer_header(&s.access);

    // Authenticated /me works first.
    app.client
        .get("/me")
        .add_header(n.clone(), v.clone())
        .await
        .assert_status_ok();

    // Logout revokes the session family + the access sid.
    app.client
        .post("/auth/logout")
        .json(&json!({ "refresh_token": s.refresh }))
        .await
        .assert_status(axum::http::StatusCode::NO_CONTENT);

    // The still-unexpired access token is now rejected via sid revocation.
    let resp = app.client.get("/me").add_header(n, v).await;
    resp.assert_status_unauthorized();
}
