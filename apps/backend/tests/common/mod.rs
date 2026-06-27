use axum::http::{HeaderName, HeaderValue};
use axum_test::TestServer;
use cookie::CookieJar;
use serde_json::json;
use sqlx::PgPool;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::redis::Redis;
use testcontainers_modules::testcontainers::{runners::AsyncRunner, ImageExt};
use veyra::adapters::inbound::http::cookies::X_CSRF_TOKEN;
use veyra::adapters::outbound::redis::{client::build_pool, session_store::RedisSessionStore};
use veyra::bootstrap::config::{Config, SameSiteCfg};
use veyra::{adapters::inbound::http::router, bootstrap::state::AppState};

pub struct TestApp {
    pub client: TestServer,
}

/// An authenticated session captured after register+login. Holds the user's
/// auth cookies (access/refresh/csrf) so a request can be scoped to *this* user
/// explicitly — robust even when several users exist in one test — plus the CSRF
/// token to send via the `X-CSRF-Token` header on mutating requests.
#[allow(dead_code)] // fields read by different test binaries
pub struct Session {
    pub csrf: String,
    pub cookies: CookieJar,
}

/// A test [`Config`]: insecure cookies (HTTP test transport), `SameSite=Strict`,
/// no domain, no CORS allowlist (same-origin), and the standard TTLs. The
/// refresh grace window is configurable so tests can exercise reuse detection.
fn test_config(refresh_grace_secs: u64) -> Config {
    Config {
        database_url: String::new(),
        redis_url: String::new(),
        jwt_secret: "test-secret-at-least-32-chars-long!!".into(),
        port: 3000,
        access_ttl_secs: 900,
        refresh_ttl_secs: 604_800,
        refresh_grace_secs,
        cookie_secure: false,
        cookie_samesite: SameSiteCfg::Strict,
        cookie_domain: None,
        cors_allowed_origins: Vec::new(),
    }
}

/// Register a user, then log in, and return the authenticated [`Session`]
/// (auth cookies + CSRF token). The login response's cookies are also stored in
/// the shared server jar (`save_cookies`), but tests scope each request to a
/// specific user via [`Session::cookies`] for correctness across multi-user
/// scenarios.
///
/// Password is always `"password123"` and name is `"User"`.
#[allow(dead_code)] // only some test binaries call this
pub async fn register_and_login(app: &TestApp, email: &str) -> Session {
    app.client
        .post("/auth/register")
        .json(&json!({
            "email": email,
            "password": "password123",
            "name": "User"
        }))
        .await;
    let resp = app
        .client
        .post("/auth/login")
        .json(&json!({
            "email": email,
            "password": "password123"
        }))
        .await;
    // The csrf cookie is readable (not HttpOnly); its value is the CSRF token.
    let csrf = resp.cookie("veyra_csrf").value().to_string();
    Session {
        csrf,
        cookies: resp.cookies(),
    }
}

/// Build the `(name, value)` pair for the `X-CSRF-Token` header. Attach to every
/// mutating request: `.add_header(name, value)`.
#[allow(dead_code)] // only some test binaries call this
pub fn csrf_header(token: &str) -> (HeaderName, HeaderValue) {
    let name = HeaderName::from_static(X_CSRF_TOKEN);
    let value = HeaderValue::from_str(token).expect("valid csrf header value");
    (name, value)
}

/// Spins up real Postgres AND Redis containers, runs migrations, builds the
/// Redis pool, and returns a [`TestApp`] backed by the full axum router with
/// cookie persistence enabled. Uses the default refresh grace window (10s).
#[allow(dead_code)] // only some test binaries call this
pub async fn spawn_app() -> TestApp {
    spawn_app_with_grace(10).await
}

/// Like [`spawn_app`] but with an explicit refresh grace window — `0` makes the
/// previous refresh secret immediately invalid (used to test reuse detection).
/// Containers are leaked intentionally so they outlive the test; process exit
/// cleans them up.
#[allow(dead_code)] // only some test binaries call this
pub async fn spawn_app_with_grace(refresh_grace_secs: u64) -> TestApp {
    let pg = Postgres::default()
        .with_tag("16-alpine")
        .start()
        .await
        .unwrap();
    let pg_port = pg.get_host_port_ipv4(5432).await.unwrap();
    let url = format!("postgres://postgres:postgres@127.0.0.1:{pg_port}/postgres");
    let pool = PgPool::connect(&url).await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    let redis = Redis::default().start().await.unwrap();
    let redis_port = redis.get_host_port_ipv4(6379).await.unwrap();
    let redis_pool = build_pool(&format!("redis://127.0.0.1:{redis_port}"))
        .await
        .unwrap();

    let state = AppState::new(pool, redis_pool, &test_config(refresh_grace_secs));
    let app = router::build(state);
    let client = TestServer::builder().save_cookies().build(app);

    // Leak the containers — they outlive the test; the process exit cleans up.
    std::mem::forget(pg);
    std::mem::forget(redis);

    TestApp { client }
}

/// Spins up a real Redis container and returns a `RedisSessionStore` (grace = 0)
/// along with the container handle. Drop the handle to stop the container.
#[allow(dead_code)]
pub async fn redis_store() -> (RedisSessionStore, impl Sized) {
    redis_store_with_grace(0).await
}

/// Like [`redis_store`] but with a configurable grace window in seconds.
#[allow(dead_code)]
pub async fn redis_store_with_grace(grace: u64) -> (RedisSessionStore, impl Sized) {
    let container = Redis::default().start().await.unwrap();
    let port = container.get_host_port_ipv4(6379).await.unwrap();
    let pool = build_pool(&format!("redis://127.0.0.1:{port}"))
        .await
        .unwrap();
    (RedisSessionStore::new(pool, 604_800, 900, grace), container)
}
