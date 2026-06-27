use std::sync::Arc;

use uuid::Uuid;

use crate::{application::errors::AppError, ports::repositories::VehicleRepository};

pub struct DeleteVehicleUseCase {
    pub repo: Arc<dyn VehicleRepository>,
}

impl DeleteVehicleUseCase {
    pub async fn execute(&self, id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        self.repo.delete(id, user_id).await.map_err(AppError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::vehicle::entity::Vehicle,
        ports::repositories::{
            CreateVehicleParams, RepositoryError, RepositoryResult, UpdateVehicleParams,
            VehicleRepository,
        },
    };
    use async_trait::async_trait;

    struct FakeVehicleRepo {
        // When Some, delete will succeed for this vehicle_id+user_id pair; else NotFound
        owned_id: Option<(Uuid, Uuid)>,
    }

    #[async_trait]
    impl VehicleRepository for FakeVehicleRepo {
        async fn list_by_user(&self, _user_id: Uuid) -> RepositoryResult<Vec<Vehicle>> {
            Ok(vec![])
        }

        async fn find_by_id(&self, _id: Uuid, _user_id: Uuid) -> RepositoryResult<Vehicle> {
            Err(RepositoryError::NotFound)
        }

        async fn insert(&self, _params: CreateVehicleParams) -> RepositoryResult<Vehicle> {
            Err(RepositoryError::NotFound)
        }

        async fn update(
            &self,
            _id: Uuid,
            _user_id: Uuid,
            _params: UpdateVehicleParams,
        ) -> RepositoryResult<Vehicle> {
            Err(RepositoryError::NotFound)
        }

        async fn delete(&self, id: Uuid, user_id: Uuid) -> RepositoryResult<()> {
            match self.owned_id {
                Some((oid, ouid)) if oid == id && ouid == user_id => Ok(()),
                _ => Err(RepositoryError::NotFound),
            }
        }
    }

    #[tokio::test]
    async fn delete_own_vehicle_succeeds() {
        let user_id = Uuid::new_v4();
        let vehicle_id = Uuid::new_v4();
        let repo = Arc::new(FakeVehicleRepo {
            owned_id: Some((vehicle_id, user_id)),
        });
        let uc = DeleteVehicleUseCase { repo };

        let result = uc.execute(vehicle_id, user_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn delete_other_user_vehicle_returns_not_found() {
        let owner_id = Uuid::new_v4();
        let intruder_id = Uuid::new_v4();
        let vehicle_id = Uuid::new_v4();
        let repo = Arc::new(FakeVehicleRepo {
            owned_id: Some((vehicle_id, owner_id)),
        });
        let uc = DeleteVehicleUseCase { repo };

        let result = uc.execute(vehicle_id, intruder_id).await;
        assert!(matches!(result, Err(AppError::NotFound)));
    }
}
