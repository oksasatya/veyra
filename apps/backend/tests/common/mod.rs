use axum::http::{header::AUTHORIZATION, HeaderName, HeaderValue};
use axum_test::TestServer;
use fred::clients::Pool as RedisPool;
use serde_json::json;
use sqlx::PgPool;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::redis::Redis;
use testcontainers_modules::testcontainers::{runners::AsyncRunner, ContainerAsync, ImageExt};
use veyra::adapters::outbound::redis::{client::build_pool, session_store::RedisSessionStore};
use veyra::bootstrap::config::Config;
use veyra::{adapters::inbound::http::router, bootstrap::state::AppState};

pub struct TestApp {
    pub client: TestServer,
    /// Raw Redis pool exposed so cache integration tests can run low-level
    /// commands (e.g. `TTL`, `GET`) against the same Redis instance.
    #[allow(dead_code)]
    pub redis_pool: RedisPool,
    // Container handles held for the TestApp's lifetime: the containers stay up for
    // the whole test and are stopped on Drop when it ends. They are NOT leaked — a
    // previous `mem::forget` left every test's containers running, accumulating across
    // the suite until the Docker daemon saturated. Holding them bounds the live
    // container count to the in-flight tests.
    #[allow(dead_code)]
    _pg: ContainerAsync<Postgres>,
    #[allow(dead_code)]
    _redis: ContainerAsync<Redis>,
}

/// An authenticated session captured after register+login. Holds the bearer
/// access + refresh tokens so a request can be scoped to *this* user explicitly —
/// robust even when several users exist in one test.
#[allow(dead_code)] // fields read by different test binaries
pub struct Session {
    pub access: String,
    pub refresh: String,
}

/// A test [`Config`] with the standard TTLs. The refresh grace window is
/// configurable so tests can exercise reuse detection.
fn test_config(refresh_grace_secs: u64) -> Config {
    Config {
        database_url: String::new(),
        redis_url: String::new(),
        jwt_secret: "test-secret-at-least-32-chars-long!!".into(),
        port: 8080,
        access_ttl_secs: 900,
        refresh_ttl_secs: 604_800,
        refresh_grace_secs,
    }
}

/// Register a user, then log in, and return the authenticated [`Session`]
/// (`access_token` + `refresh_token` read from the JSON body).
///
/// Password is always `"password123"` and name is `"User"`.
#[allow(dead_code)] // only some test binaries call this
pub async fn register_and_login(app: &TestApp, email: &str) -> Session {
    app.client
        .post("/auth/register")
        .json(&json!({ "email": email, "password": "password123", "name": "User" }))
        .await;
    let resp = app
        .client
        .post("/auth/login")
        .json(&json!({ "email": email, "password": "password123" }))
        .await;
    let body: serde_json::Value = resp.json();
    let access = body["data"]["tokens"]["access_token"]
        .as_str()
        .expect("login returns an access token")
        .to_string();
    let refresh = body["data"]["tokens"]["refresh_token"]
        .as_str()
        .expect("login returns a refresh token")
        .to_string();
    Session { access, refresh }
}

/// The `Authorization: Bearer <access>` header pair for protected requests.
/// Attach via `.add_header(name, value)`.
#[allow(dead_code)] // only some test binaries call this
pub fn bearer_header(access: &str) -> (HeaderName, HeaderValue) {
    let value =
        HeaderValue::from_str(&format!("Bearer {access}")).expect("valid bearer header value");
    (AUTHORIZATION, value)
}

/// Spins up real Postgres AND Redis containers, runs migrations, builds the
/// Redis pool, and returns a [`TestApp`] backed by the full axum router. Uses the
/// default refresh grace window (10s).
#[allow(dead_code)] // only some test binaries call this
pub async fn spawn_app() -> TestApp {
    spawn_app_with_grace(10).await
}

/// Like [`spawn_app`] but with an explicit refresh grace window — `0` makes the
/// previous refresh secret immediately invalid (used to test reuse detection).
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

    let state = AppState::new(pool, redis_pool.clone(), &test_config(refresh_grace_secs));
    let app = router::build(state);
    let client = TestServer::new(app);

    // Hold the container handles in TestApp so they live for the whole test and are
    // stopped on Drop when it ends — never leaked.
    TestApp {
        client,
        redis_pool,
        _pg: pg,
        _redis: redis,
    }
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
