mod common;

#[tokio::test]
async fn health_still_works() {
    let app = common::spawn_app().await;
    let resp = app.client.get("/health").await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["status"].as_str().unwrap(), "ok");
}

#[tokio::test]
async fn register_returns_201_with_token() {
    let app = common::spawn_app().await;

    let resp = app
        .client
        .post("/auth/register")
        .json(&serde_json::json!({
            "email": "alice@example.com",
            "password": "password123",
            "name": "Alice"
        }))
        .await;

    resp.assert_status(axum::http::StatusCode::CREATED);
    let body: serde_json::Value = resp.json();
    assert!(body["token"].as_str().is_some());
}

#[tokio::test]
async fn register_duplicate_email_returns_409() {
    let app = common::spawn_app().await;
    let payload = serde_json::json!({
        "email": "dup@example.com",
        "password": "password123",
        "name": "Dup"
    });
    app.client.post("/auth/register").json(&payload).await;
    let resp = app.client.post("/auth/register").json(&payload).await;
    resp.assert_status_conflict();
}

#[tokio::test]
async fn login_returns_200_with_token() {
    let app = common::spawn_app().await;
    app.client
        .post("/auth/register")
        .json(&serde_json::json!({
            "email": "login@example.com",
            "password": "password123",
            "name": "Login"
        }))
        .await;

    let resp = app
        .client
        .post("/auth/login")
        .json(&serde_json::json!({
            "email": "login@example.com",
            "password": "password123"
        }))
        .await;

    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert!(body["token"].as_str().is_some());
}

#[tokio::test]
async fn login_wrong_password_returns_401() {
    let app = common::spawn_app().await;
    app.client
        .post("/auth/register")
        .json(&serde_json::json!({
            "email": "pass@example.com",
            "password": "password123",
            "name": "P"
        }))
        .await;

    let resp = app
        .client
        .post("/auth/login")
        .json(&serde_json::json!({
            "email": "pass@example.com",
            "password": "wrongpassword"
        }))
        .await;

    resp.assert_status_unauthorized();
}

#[tokio::test]
async fn me_returns_user_when_authenticated() {
    let app = common::spawn_app().await;
    let reg = app
        .client
        .post("/auth/register")
        .json(&serde_json::json!({
            "email": "me@example.com",
            "password": "password123",
            "name": "Me"
        }))
        .await;
    let token: serde_json::Value = reg.json();
    let token = token["token"].as_str().unwrap();

    let resp = app.client.get("/me").authorization_bearer(token).await;

    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["email"].as_str().unwrap(), "me@example.com");
}

#[tokio::test]
async fn me_without_token_returns_401() {
    let app = common::spawn_app().await;
    let resp = app.client.get("/me").await;
    resp.assert_status_unauthorized();
}
