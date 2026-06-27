use std::sync::Arc;

use sqlx::PgPool;

use crate::{
    adapters::outbound::postgres::{
        document_repo::PgDocumentRepo, expense_repo::PgExpenseRepo,
        fuel_log_repo::PgFuelLogRepo, jwt_auth::JwtAuth, reminder_repo::PgReminderRepo,
        service_record_repo::PgServiceRecordRepo, user_repo::PgUserRepo,
        vehicle_repo::PgVehicleRepo,
    },
    ports::{
        auth::AuthPort,
        repositories::{
            DocumentRepository, ExpenseRepository, FuelLogRepository, ReminderRepository,
            ServiceRecordRepository, UserRepository, VehicleRepository,
        },
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
    pub auth: Arc<dyn AuthPort>,
}

impl AppState {
    pub fn new(pool: PgPool, jwt_secret: String) -> Self {
        let user_repo = Arc::new(PgUserRepo::new(pool.clone()));
        let vehicle_repo = Arc::new(PgVehicleRepo::new(pool.clone()));
        let service_record_repo = Arc::new(PgServiceRecordRepo::new(pool.clone()));
        let fuel_log_repo = Arc::new(PgFuelLogRepo::new(pool.clone()));
        let expense_repo = Arc::new(PgExpenseRepo::new(pool.clone()));
        let reminder_repo = Arc::new(PgReminderRepo::new(pool.clone()));
        let document_repo = Arc::new(PgDocumentRepo::new(pool.clone()));
        let auth = Arc::new(JwtAuth::new(jwt_secret));
        Self {
            pool,
            user_repo,
            vehicle_repo,
            service_record_repo,
            fuel_log_repo,
            expense_repo,
            reminder_repo,
            document_repo,
            auth,
        }
    }
}
