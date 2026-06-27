use std::sync::Arc;

use chrono::NaiveDate;
use uuid::Uuid;

use crate::{
    application::errors::AppError,
    domain::document::entity::{DocType, Document},
    ports::repositories::{CreateDocumentParams, DocumentRepository, VehicleRepository},
};

pub struct CreateDocumentUseCase {
    pub repo: Arc<dyn DocumentRepository>,
    pub vehicle_repo: Arc<dyn VehicleRepository>,
}

pub struct CreateDocumentInput {
    pub vehicle_id: Uuid,
    pub user_id: Uuid,
    pub doc_type: String,
    pub title: String,
    pub expiry_date: Option<NaiveDate>,
    pub file_url: Option<String>,
    pub notes: Option<String>,
}

impl CreateDocumentUseCase {
    /// Validates `doc_type` against `DocType::parse`, verifies vehicle ownership,
    /// then inserts the document.
    ///
    /// Returns `AppError::Validation` for an unknown doc_type,
    /// `AppError::NotFound` when the vehicle is not owned by the caller.
    pub async fn execute(&self, input: CreateDocumentInput) -> Result<Document, AppError> {
        // Validate doc_type before hitting the database
        let _ = DocType::parse(&input.doc_type)
            .ok_or_else(|| AppError::Validation(format!("unknown doc_type: {}", input.doc_type)))?;

        // Ownership guard: vehicle must belong to the caller
        self.vehicle_repo
            .find_by_id(input.vehicle_id, input.user_id)
            .await
            .map_err(AppError::from)?;

        self.repo
            .insert(CreateDocumentParams {
                vehicle_id: input.vehicle_id,
                doc_type: input.doc_type,
                title: input.title,
                expiry_date: input.expiry_date,
                file_url: input.file_url,
                notes: input.notes,
            })
            .await
            .map_err(AppError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::{
            document::entity::Document,
            vehicle::{
                entity::Vehicle,
                value_objects::{FuelType, Odometer, PlateNumber},
            },
        },
        ports::repositories::{
            CreateDocumentParams, CreateVehicleParams, RepositoryError, RepositoryResult,
            UpdateVehicleParams,
        },
    };
    use async_trait::async_trait;
    use chrono::Utc;

    struct FakeVehicleRepo {
        owner_id: Uuid,
        vehicle_id: Uuid,
    }

    #[async_trait]
    impl VehicleRepository for FakeVehicleRepo {
        async fn list_by_user(&self, _user_id: Uuid) -> RepositoryResult<Vec<Vehicle>> {
            Ok(vec![])
        }

        async fn find_by_id(&self, id: Uuid, user_id: Uuid) -> RepositoryResult<Vehicle> {
            if id == self.vehicle_id && user_id == self.owner_id {
                Ok(fake_vehicle(self.vehicle_id, self.owner_id))
            } else {
                Err(RepositoryError::NotFound)
            }
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

        async fn delete(&self, _id: Uuid, _user_id: Uuid) -> RepositoryResult<()> {
            Err(RepositoryError::NotFound)
        }
    }

    struct FakeDocumentRepo {
        fail_with: Option<RepositoryError>,
    }

    #[async_trait]
    impl DocumentRepository for FakeDocumentRepo {
        async fn list_by_vehicle(
            &self,
            _vehicle_id: Uuid,
            _user_id: Uuid,
        ) -> RepositoryResult<Vec<Document>> {
            Ok(vec![])
        }

        async fn insert(&self, params: CreateDocumentParams) -> RepositoryResult<Document> {
            if let Some(ref e) = self.fail_with {
                return Err(match e {
                    RepositoryError::NotFound => RepositoryError::NotFound,
                    RepositoryError::Conflict(m) => RepositoryError::Conflict(m.clone()),
                    RepositoryError::Database(m) => RepositoryError::Database(m.clone()),
                });
            }
            Ok(Document {
                id: Uuid::new_v4(),
                vehicle_id: params.vehicle_id,
                doc_type: DocType::parse(&params.doc_type).unwrap_or(DocType::Other),
                title: params.title,
                expiry_date: params.expiry_date,
                file_url: params.file_url,
                notes: params.notes,
                created_at: Utc::now(),
            })
        }
    }

    fn fake_vehicle(id: Uuid, user_id: Uuid) -> Vehicle {
        Vehicle {
            id,
            user_id,
            brand: "Toyota".into(),
            model: "Avanza".into(),
            year: 2020,
            plate_number: PlateNumber::new("B 1234 XYZ".into()).unwrap(),
            color: None,
            fuel_type: FuelType::parse("petrol").unwrap(),
            current_odometer: Odometer::new(0),
            notes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_input(vehicle_id: Uuid, user_id: Uuid) -> CreateDocumentInput {
        CreateDocumentInput {
            vehicle_id,
            user_id,
            doc_type: "stnk".into(),
            title: "STNK 2026".into(),
            expiry_date: Some("2026-12-31".parse().unwrap()),
            file_url: Some("https://storage.example.com/stnk.pdf".into()),
            notes: None,
        }
    }

    #[tokio::test]
    async fn valid_input_creates_document() {
        let vehicle_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let uc = CreateDocumentUseCase {
            repo: Arc::new(FakeDocumentRepo { fail_with: None }),
            vehicle_repo: Arc::new(FakeVehicleRepo {
                owner_id: user_id,
                vehicle_id,
            }),
        };

        let result = uc.execute(make_input(vehicle_id, user_id)).await;
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.doc_type, DocType::Stnk);
    }

    #[tokio::test]
    async fn invalid_doc_type_returns_validation_error() {
        let vehicle_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let uc = CreateDocumentUseCase {
            repo: Arc::new(FakeDocumentRepo { fail_with: None }),
            vehicle_repo: Arc::new(FakeVehicleRepo {
                owner_id: user_id,
                vehicle_id,
            }),
        };

        let mut input = make_input(vehicle_id, user_id);
        input.doc_type = "unknown_type".into();

        let result = uc.execute(input).await;
        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[tokio::test]
    async fn wrong_owner_returns_not_found() {
        let vehicle_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let intruder_id = Uuid::new_v4();

        let uc = CreateDocumentUseCase {
            repo: Arc::new(FakeDocumentRepo { fail_with: None }),
            vehicle_repo: Arc::new(FakeVehicleRepo {
                owner_id,
                vehicle_id,
            }),
        };

        let result = uc.execute(make_input(vehicle_id, intruder_id)).await;
        assert!(matches!(result, Err(AppError::NotFound)));
    }

    #[tokio::test]
    async fn repo_database_error_maps_to_internal() {
        let vehicle_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let uc = CreateDocumentUseCase {
            repo: Arc::new(FakeDocumentRepo {
                fail_with: Some(RepositoryError::Database("db error".into())),
            }),
            vehicle_repo: Arc::new(FakeVehicleRepo {
                owner_id: user_id,
                vehicle_id,
            }),
        };

        let result = uc.execute(make_input(vehicle_id, user_id)).await;
        assert!(matches!(result, Err(AppError::Internal(_))));
    }
}
