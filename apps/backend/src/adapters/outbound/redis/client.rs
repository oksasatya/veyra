use anyhow::Context;
use fred::clients::Pool;
use fred::prelude::*;

/// Builds and initializes a fred connection pool from a `redis://` URL.
pub async fn build_pool(redis_url: &str) -> anyhow::Result<Pool> {
    let config = Config::from_url(redis_url).context("invalid REDIS_URL")?;
    let pool = Builder::from_config(config)
        .build_pool(8)
        .context("failed to build Redis pool")?;
    pool.init().await.context("failed to connect to Redis")?;
    Ok(pool)
}
