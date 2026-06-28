mod common;

use cookie::CookieJar;
use serde_json::json;

const ACCESS_COOKIE: &str = "veyra_access";
const REFRESH_COOKIE: &str = "veyra_refresh";
const CSRF_COOKIE: &str = "veyra_csrf";

#[tokio::test]
async fn health_still_works() {
    let app = common::spawn_app().await;
    let resp = app.client.get("/health").await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["status"].as_str().unwrap(), "ok");
}

#[tokio::test]
async fn register_returns_201_with_user_and_no_body_token() {
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
    // No token in the body — auth is carried entirely by cookies.
    assert!(
        body["data"]["token"].is_null(),
        "register must NOT return a body token"
    );
    assert_eq!(body["data"]["email"].as_str().unwrap(), "alice@example.com");
    assert_eq!(body["data"]["name"].as_str().unwrap(), "Alice");
    // Cookies set.
    assert!(resp.maybe_cookie(ACCESS_COOKIE).is_some());
    assert!(resp.maybe_cookie(REFRESH_COOKIE).is_some());
    assert!(resp.maybe_cookie(CSRF_COOKIE).is_some());
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
async fn login_sets_httponly_access_cookie_and_no_body_token() {
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
    assert!(
        body["data"]["token"].is_null(),
        "login must NOT return a body token"
    );

    let access = resp.cookie(ACCESS_COOKIE);
    assert_eq!(
        access.http_only(),
        Some(true),
        "access cookie must be HttpOnly"
    );
    assert!(!access.value().is_empty());

    // CSRF cookie must be readable by JS (NOT HttpOnly).
    let csrf = resp.cookie(CSRF_COOKIE);
    assert_ne!(csrf.http_only(), Some(true), "csrf cookie must be readable");
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
async fn me_returns_user_when_authenticated() {
    let app = common::spawn_app().await;
    // register_and_login stores the session cookies in the shared jar.
    common::register_and_login(&app, "me@example.com").await;

    let resp = app.client.get("/me").await;

    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["data"]["email"].as_str().unwrap(), "me@example.com");
}

#[tokio::test]
async fn me_without_token_returns_401() {
    let app = common::spawn_app().await;
    // Fresh server, no cookies in the jar.
    let resp = app.client.get("/me").await;
    resp.assert_status_unauthorized();
}

#[tokio::test]
async fn post_without_csrf_header_returns_403() {
    let app = common::spawn_app().await;
    // Authenticated (cookies in jar) but NO X-CSRF-Token header → CSRF rejects.
    common::register_and_login(&app, "csrf@example.com").await;

    let resp = app
        .client
        .post("/vehicles")
        .json(&json!({
            "brand": "Toyota", "model": "Avanza", "year": 2020,
            "plate_number": "B 4040 CSF", "fuel_type": "petrol", "current_odometer": 0
        }))
        .await;

    resp.assert_status(axum::http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn refresh_rotates_and_old_cookie_rejected_after_grace() {
    // grace = 0 → the previous refresh secret is invalid the instant it rotates.
    let app = common::spawn_app_with_grace(0).await;
    let login = app
        .client
        .post("/auth/register")
        .json(&json!({
            "email": "rotate@example.com",
            "password": "password123",
            "name": "Rotate"
        }))
        .await;

    // CSRF is now enforced on /auth/refresh — read the token from the login jar.
    let csrf = login.cookie(CSRF_COOKIE).value().to_string();
    let (csrf_name, csrf_value) = common::csrf_header(&csrf);

    // Capture the ORIGINAL refresh cookie before any rotation.
    let original_refresh = login.cookie(REFRESH_COOKIE);

    // First refresh: rotates successfully, issues a new refresh cookie.
    let r1 = app
        .client
        .post("/auth/refresh")
        .add_header(csrf_name.clone(), csrf_value.clone())
        .await;
    r1.assert_status_ok();
    let new_refresh = r1.cookie(REFRESH_COOKIE);
    assert_ne!(
        original_refresh.value(),
        new_refresh.value(),
        "refresh must rotate the secret"
    );

    // Replay the ORIGINAL (now-stale) refresh cookie → reuse detected → 401.
    // Carry the csrf cookie + header so CSRF passes and the reuse path is reached.
    let mut stale_jar = CookieJar::new();
    stale_jar.add(original_refresh.clone());
    stale_jar.add(login.cookie(CSRF_COOKIE).clone());
    let r2 = app
        .client
        .post("/auth/refresh")
        .add_cookies(stale_jar)
        .add_header(csrf_name, csrf_value)
        .do_not_save_cookies()
        .await;
    r2.assert_status_unauthorized();
}

#[tokio::test]
async fn refresh_without_csrf_token_returns_403() {
    let app = common::spawn_app().await;
    // Authenticated jar (register stores cookies) but NO X-CSRF-Token header.
    app.client
        .post("/auth/register")
        .json(&json!({
            "email": "refresh-nocsrf@example.com",
            "password": "password123",
            "name": "NoCsrf"
        }))
        .await;

    let resp = app.client.post("/auth/refresh").await;
    resp.assert_status(axum::http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn logout_without_csrf_token_returns_403() {
    let app = common::spawn_app().await;
    // Authenticated jar (register stores cookies) but NO X-CSRF-Token header.
    app.client
        .post("/auth/register")
        .json(&json!({
            "email": "logout-nocsrf@example.com",
            "password": "password123",
            "name": "NoCsrf"
        }))
        .await;

    let resp = app.client.post("/auth/logout").await;
    resp.assert_status(axum::http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn logout_revokes_session_then_me_returns_401() {
    let app = common::spawn_app().await;
    let login = app
        .client
        .post("/auth/register")
        .json(&json!({
            "email": "logout@example.com",
            "password": "password123",
            "name": "Logout"
        }))
        .await;
    // Capture the live access cookie before logout so we can prove the SERVER
    // rejects the (still-unexpired) token via sid revocation — not just that
    // the cookie was cleared from the jar.
    let access = login.cookie(ACCESS_COOKIE);
    let csrf = login.cookie(CSRF_COOKIE).value().to_string();
    let (csrf_name, csrf_value) = common::csrf_header(&csrf);

    // Authenticated /me works first.
    app.client.get("/me").await.assert_status_ok();

    // Logout (cookies in jar carry access sid + refresh family) — CSRF enforced.
    let out = app
        .client
        .post("/auth/logout")
        .add_header(csrf_name, csrf_value)
        .await;
    out.assert_status(axum::http::StatusCode::NO_CONTENT);

    // Re-send the original access cookie explicitly: the sid is now revoked, so
    // the server must reject it even though the JWT itself has not expired.
    let mut revoked_jar = CookieJar::new();
    revoked_jar.add(access.clone());
    let resp = app
        .client
        .get("/me")
        .add_cookies(revoked_jar)
        .do_not_save_cookies()
        .await;
    resp.assert_status_unauthorized();
}

#[tokio::test]
async fn logout_with_expired_access_still_revokes_session() {
    // Logout must work WITHOUT a valid access cookie: it derives the family_id
    // from the refresh cookie (sid == family_id) and revokes the family. Here we
    // send ONLY the refresh + csrf cookies (no access cookie) and assert the
    // family is revoked by replaying the refresh cookie afterwards → 401.
    let app = common::spawn_app().await;
    let login = app
        .client
        .post("/auth/register")
        .json(&json!({
            "email": "logout-expired@example.com",
            "password": "password123",
            "name": "Expired"
        }))
        .await;
    let refresh = login.cookie(REFRESH_COOKIE);
    let csrf_cookie = login.cookie(CSRF_COOKIE);
    let csrf = csrf_cookie.value().to_string();
    let (csrf_name, csrf_value) = common::csrf_header(&csrf);

    // Logout with ONLY refresh + csrf cookies present (no access cookie at all).
    let mut jar = CookieJar::new();
    jar.add(refresh.clone());
    jar.add(csrf_cookie.clone());
    let out = app
        .client
        .post("/auth/logout")
        .add_cookies(jar)
        .add_header(csrf_name.clone(), csrf_value.clone())
        .do_not_save_cookies()
        .await;
    out.assert_status(axum::http::StatusCode::NO_CONTENT);

    // The family must now be revoked: replaying the original refresh cookie on
    // /auth/refresh returns 401 (family gone).
    let mut refresh_jar = CookieJar::new();
    refresh_jar.add(refresh.clone());
    refresh_jar.add(csrf_cookie.clone());
    let r = app
        .client
        .post("/auth/refresh")
        .add_cookies(refresh_jar)
        .add_header(csrf_name, csrf_value)
        .do_not_save_cookies()
        .await;
    r.assert_status_unauthorized();
}

// ── Bearer mode (native mobile, ADR-0007) ────────────────────────────────────

#[tokio::test]
async fn bearer_login_returns_tokens_and_no_cookies() {
    let app = common::spawn_app().await;
    let (n, v) = common::auth_mode_header();
    app.client
        .post("/auth/register")
        .add_header(n.clone(), v.clone())
        .json(&json!({ "email": "b@e.com", "password": "password123", "name": "B" }))
        .await;
    let resp = app
        .client
        .post("/auth/login")
        .add_header(n, v)
        .json(&json!({ "email": "b@e.com", "password": "password123" }))
        .await;

    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert!(body["data"]["tokens"]["access_token"].is_string());
    assert!(body["data"]["tokens"]["refresh_token"].is_string());
    // Security invariant: bearer mode sets NO cookies.
    assert!(
        resp.maybe_cookie(ACCESS_COOKIE).is_none(),
        "bearer mode must not set the access cookie"
    );
    assert!(resp.maybe_cookie(REFRESH_COOKIE).is_none());
}

#[tokio::test]
async fn bearer_me_succeeds_with_authorization_header() {
    let app = common::spawn_app().await;
    let (access, _refresh) = common::register_and_login_bearer(&app, "bm@e.com").await;
    let (n, v) = common::bearer_header(&access);

    let resp = app.client.get("/me").add_header(n, v).await;

    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["data"]["email"].as_str().unwrap(), "bm@e.com");
}

#[tokio::test]
async fn bearer_mutation_needs_no_csrf_header() {
    let app = common::spawn_app().await;
    let (access, _refresh) = common::register_and_login_bearer(&app, "bmut@e.com").await;
    let (n, v) = common::bearer_header(&access);

    // POST a vehicle with Bearer auth and NO X-CSRF-Token header → CSRF bypassed.
    let resp = app
        .client
        .post("/vehicles")
        .add_header(n, v)
        .json(&json!({
            "brand": "Toyota", "model": "Avanza", "year": 2020,
            "plate_number": "B 1 ABC", "fuel_type": "petrol", "current_odometer": 1000
        }))
        .await;

    assert!(
        resp.status_code().is_success(),
        "bearer mutation should pass the CSRF bypass, got {}",
        resp.status_code()
    );
}

#[tokio::test]
async fn bearer_invalid_token_is_unauthorized_without_cookies() {
    let app = common::spawn_app().await;
    let (n, v) = common::bearer_header("not.a.real.jwt");

    let resp = app.client.get("/me").add_header(n, v).await;

    resp.assert_status_unauthorized();
    assert!(resp.maybe_cookie(ACCESS_COOKIE).is_none());
}

#[tokio::test]
async fn bearer_refresh_rotates_and_returns_tokens() {
    let app = common::spawn_app().await;
    let (_access, refresh) = common::register_and_login_bearer(&app, "br@e.com").await;
    let (n, v) = common::auth_mode_header();

    let resp = app
        .client
        .post("/auth/refresh")
        .add_header(n, v)
        .json(&json!({ "refresh_token": refresh }))
        .await;

    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert!(body["data"]["tokens"]["access_token"].is_string());
    assert!(body["data"]["tokens"]["refresh_token"].is_string());
}

#[tokio::test]
async fn bearer_refresh_reuse_is_rejected() {
    // grace = 0 → the old refresh secret is invalid the instant it rotates.
    let app = common::spawn_app_with_grace(0).await;
    let (_access, refresh) = common::register_and_login_bearer(&app, "brr@e.com").await;
    let (n, v) = common::auth_mode_header();

    app.client
        .post("/auth/refresh")
        .add_header(n.clone(), v.clone())
        .json(&json!({ "refresh_token": refresh.clone() }))
        .await
        .assert_status_ok();

    let resp = app
        .client
        .post("/auth/refresh")
        .add_header(n, v)
        .json(&json!({ "refresh_token": refresh }))
        .await;

    resp.assert_status_unauthorized();
}

#[tokio::test]
async fn bearer_logout_then_refresh_is_unauthorized() {
    let app = common::spawn_app().await;
    let (_access, refresh) = common::register_and_login_bearer(&app, "bl@e.com").await;
    let (n, v) = common::auth_mode_header();

    app.client
        .post("/auth/logout")
        .add_header(n.clone(), v.clone())
        .json(&json!({ "refresh_token": refresh.clone() }))
        .await
        .assert_status(axum::http::StatusCode::NO_CONTENT);

    let resp = app
        .client
        .post("/auth/refresh")
        .add_header(n, v)
        .json(&json!({ "refresh_token": refresh }))
        .await;

    resp.assert_status_unauthorized();
}
