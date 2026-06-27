use crate::bootstrap::state::AppState;
use axum::{routing::get, Router};

use super::handlers::health::health;

pub fn build(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .with_state(state)
}
