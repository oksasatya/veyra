use std::sync::Arc;

use sqlx::PgPool;

use crate::{
    adapters::outbound::postgres::{
        jwt_auth::JwtAuth,
        service_record_repo::PgServiceRecordRepo,
        user_repo::PgUserRepo,
        vehicle_repo::PgVehicleRepo,
    },
    ports::{
        auth::AuthPort,
        repositories::{ServiceRecordRepository, UserRepository, VehicleRepository},
    },
};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub user_repo: Arc<dyn UserRepository>,
    pub vehicle_repo: Arc<dyn VehicleRepository>,
    pub service_record_repo: Arc<dyn ServiceRecordRepository>,
    pub auth: Arc<dyn AuthPort>,
}

impl AppState {
    pub fn new(pool: PgPool, jwt_secret: String) -> Self {
        let user_repo = Arc::new(PgUserRepo::new(pool.clone()));
        let vehicle_repo = Arc::new(PgVehicleRepo::new(pool.clone()));
        let service_record_repo = Arc::new(PgServiceRecordRepo::new(pool.clone()));
        let auth = Arc::new(JwtAuth::new(jwt_secret));
        Self {
            pool,
            user_repo,
            vehicle_repo,
            service_record_repo,
            auth,
        }
    }
}
