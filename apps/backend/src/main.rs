use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use veyra::adapters::inbound::http::router;
use veyra::adapters::outbound::redis::client::build_pool;
use veyra::bootstrap::{config::Config, state::AppState};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "veyra=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::load()?;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let redis_pool = build_pool(&config.redis_url).await?;

    let state = AppState::new(pool, redis_pool, &config);
    let app = router::build(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
