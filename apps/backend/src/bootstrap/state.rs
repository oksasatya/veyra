use std::sync::Arc;

use fred::clients::Pool as RedisPool;
use sqlx::PgPool;

use crate::{
    adapters::outbound::{
        postgres::{
            document_repo::PgDocumentRepo, expense_repo::PgExpenseRepo,
            fuel_log_repo::PgFuelLogRepo, reminder_repo::PgReminderRepo,
            service_record_repo::PgServiceRecordRepo, summary_repo::PgSummaryRepo,
            user_repo::PgUserRepo, vehicle_repo::PgVehicleRepo,
        },
        redis::{
            cache::RedisCache, cached_summary_repo::CachedSummaryRepo,
            cached_vehicle_repo::CachedVehicleRepo, session_store::RedisSessionStore,
        },
        token::jwt_auth::JwtAuth,
    },
    bootstrap::config::Config,
    ports::{
        auth::AuthPort,
        repositories::{
            DocumentRepository, ExpenseRepository, FuelLogRepository, ReminderRepository,
            ServiceRecordRepository, SummaryRepository, UserRepository, VehicleRepository,
        },
        session::SessionStore,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub user_repo: Arc<dyn UserRepository>,
    pub vehicle_repo: Arc<dyn VehicleRepository>,
    pub service_record_repo: Arc<dyn ServiceRecordRepository>,
    pub fuel_log_repo: Arc<dyn FuelLogRepository>,
    pub expense_repo: Arc<dyn ExpenseRepository>,
    pub reminder_repo: Arc<dyn ReminderRepository>,
    pub document_repo: Arc<dyn DocumentRepository>,
    pub summary_repo: Arc<dyn SummaryRepository>,
    pub auth: Arc<dyn AuthPort>,
    pub sessions: Arc<dyn SessionStore>,
    pub access_ttl_secs: u64,
}

impl AppState {
    /// Compose the application state from the database pool, the Redis pool, and
    /// the loaded configuration.
    pub fn new(pool: PgPool, redis_pool: RedisPool, config: &Config) -> Self {
        let user_repo = Arc::new(PgUserRepo::new(pool.clone()));
        let vehicle_repo: Arc<dyn VehicleRepository> = Arc::new(CachedVehicleRepo::new(
            Arc::new(PgVehicleRepo::new(pool.clone())),
            RedisCache::new(redis_pool.clone()),
        ));
        let service_record_repo = Arc::new(PgServiceRecordRepo::new(pool.clone()));
        let fuel_log_repo = Arc::new(PgFuelLogRepo::new(pool.clone()));
        let expense_repo = Arc::new(PgExpenseRepo::new(pool.clone()));
        let reminder_repo = Arc::new(PgReminderRepo::new(pool.clone()));
        let document_repo = Arc::new(PgDocumentRepo::new(pool.clone()));
        let summary_repo: Arc<dyn SummaryRepository> = Arc::new(CachedSummaryRepo::new(
            Arc::new(PgSummaryRepo::new(pool.clone())),
            RedisCache::new(redis_pool.clone()),
        ));

        let auth = Arc::new(JwtAuth::new(
            config.jwt_secret.clone(),
            config.access_ttl_secs,
        ));
        let sessions = Arc::new(RedisSessionStore::new(
            redis_pool,
            config.refresh_ttl_secs,
            config.access_ttl_secs,
            config.refresh_grace_secs,
        ));
        Self {
            pool,
            user_repo,
            vehicle_repo,
            service_record_repo,
            fuel_log_repo,
            expense_repo,
            reminder_repo,
            document_repo,
            summary_repo,
            auth,
            sessions,
            access_ttl_secs: config.access_ttl_secs,
        }
    }
}
