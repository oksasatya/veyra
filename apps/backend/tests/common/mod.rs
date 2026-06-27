use axum_test::TestServer;
use serde_json::json;
use sqlx::PgPool;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::{runners::AsyncRunner, RunnableImage};
use veyra::{adapters::inbound::http::router, bootstrap::state::AppState};

pub struct TestApp {
    pub client: TestServer,
}

/// Register a user and return their JWT token.
///
/// Used by all integration test files that need an authenticated session.
/// Password is always `"password123"` and name is `"User"`.
#[allow(dead_code)] // only some test binaries call this
pub async fn register_and_login(app: &TestApp, email: &str) -> String {
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
    let body: serde_json::Value = resp.json();
    body["token"].as_str().unwrap().to_string()
}

/// Spins up a real Postgres container, runs migrations, and returns a TestApp
/// backed by the full axum router. The container is leaked intentionally so it
/// outlives the test — process exit cleans it up.
pub async fn spawn_app() -> TestApp {
    let image = RunnableImage::from(Postgres::default()).with_tag("16-alpine");
    let container = image.start().await;
    let port = container.get_host_port_ipv4(5432).await;
    let url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");
    let pool = PgPool::connect(&url).await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    let state = AppState::new(pool, "test-secret-at-least-32-chars-long!!".into());
    let app = router::build(state);
    let client = TestServer::new(app);

    // Leak the container — it outlives the test; the process exit cleans it up.
    std::mem::forget(container);

    TestApp { client }
}
