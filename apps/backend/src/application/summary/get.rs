use std::sync::Arc;

use uuid::Uuid;

use crate::{
    application::errors::AppError,
    ports::repositories::{SummaryRepository, VehicleSummaryData},
};

pub struct GetSummaryUseCase {
    pub repo: Arc<dyn SummaryRepository>,
}

impl GetSummaryUseCase {
    pub async fn execute(
        &self,
        vehicle_id: Uuid,
        user_id: Uuid,
    ) -> Result<VehicleSummaryData, AppError> {
        self.repo
            .get_vehicle_summary(vehicle_id, user_id)
            .await
            .map_err(AppError::from)
    }
}
