use std::sync::Arc;

use sqlx::PgPool;

use crate::{
    adapters::outbound::postgres::{jwt_auth::JwtAuth, user_repo::PgUserRepo},
    ports::{auth::AuthPort, repositories::UserRepository},
};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub user_repo: Arc<dyn UserRepository>,
    pub auth: Arc<dyn AuthPort>,
}

impl AppState {
    pub fn new(pool: PgPool, jwt_secret: String) -> Self {
        let user_repo = Arc::new(PgUserRepo::new(pool.clone()));
        let auth = Arc::new(JwtAuth::new(jwt_secret));
        Self {
            pool,
            user_repo,
            auth,
        }
    }
}
