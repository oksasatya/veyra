use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::reminder::entity::{Reminder, ReminderType},
    ports::repositories::{
        CreateReminderParams, RepositoryError, RepositoryResult, ReminderRepository,
        UpdateReminderParams,
    },
};

use super::models::ReminderRow;

/// Column list for SELECT (no table prefix — used in single-table RETURNING).
const SELECT_COLS: &str =
    "id, vehicle_id, title, reminder_type, due_date, due_odometer, is_completed, notes, created_at";

/// Column list with `r.` prefix — used in JOINed queries.
const SELECT_COLS_PREFIXED: &str =
    "r.id, r.vehicle_id, r.title, r.reminder_type, r.due_date, r.due_odometer, r.is_completed, r.notes, r.created_at";

/// Postgres implementation of [`ReminderRepository`].
pub struct PgReminderRepo {
    pool: PgPool,
}

impl PgReminderRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Converts a `sqlx::Error` to `RepositoryError`, keeping the ports layer
/// free of sqlx (layer-boundary rule).
fn map_pg_err(e: sqlx::Error) -> RepositoryError {
    RepositoryError::Database(e.to_string())
}

/// Maps a `ReminderRow` (sqlx) to a domain `Reminder`.
///
/// `reminder_type` is stored as TEXT in Postgres. We default to `Date` if the
/// stored value is unrecognised — avoids a hard crash on a bad DB row.
fn row_to_reminder(r: ReminderRow) -> Reminder {
    Reminder {
        id: r.id,
        vehicle_id: r.vehicle_id,
        title: r.title,
        reminder_type: ReminderType::parse(&r.reminder_type).unwrap_or(ReminderType::Date),
        due_date: r.due_date,
        due_odometer: r.due_odometer.map(|v| v as u32),
        is_completed: r.is_completed,
        notes: r.notes,
        created_at: r.created_at,
    }
}

#[async_trait]
impl ReminderRepository for PgReminderRepo {
    /// Lists reminders for a vehicle, scoped via a JOIN to ensure the vehicle
    /// belongs to `user_id`. Ordered by `due_date ASC NULLS LAST`.
    async fn list_by_vehicle(
        &self,
        vehicle_id: Uuid,
        user_id: Uuid,
    ) -> RepositoryResult<Vec<Reminder>> {
        let query = format!(
            "SELECT {SELECT_COLS_PREFIXED} \
             FROM reminders r \
             JOIN vehicles v ON v.id = r.vehicle_id \
             WHERE r.vehicle_id = $1 AND v.user_id = $2 \
             ORDER BY r.due_date ASC NULLS LAST, r.created_at ASC"
        );
        let rows = sqlx::query_as::<_, ReminderRow>(&query)
            .bind(vehicle_id)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(map_pg_err)?;

        Ok(rows.into_iter().map(row_to_reminder).collect())
    }

    async fn insert(&self, p: CreateReminderParams) -> RepositoryResult<Reminder> {
        let query = format!(
            r#"INSERT INTO reminders
               (vehicle_id, title, reminder_type, due_date, due_odometer, notes)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING {SELECT_COLS}"#
        );
        let row = sqlx::query_as::<_, ReminderRow>(&query)
            .bind(p.vehicle_id)
            .bind(&p.title)
            .bind(&p.reminder_type)
            .bind(p.due_date)
            .bind(p.due_odometer.map(|v| v as i32))
            .bind(&p.notes)
            .fetch_one(&self.pool)
            .await
            .map_err(map_pg_err)?;

        Ok(row_to_reminder(row))
    }

    /// Applies a partial update. Only fields provided in `params` are changed;
    /// `None` means "keep existing". The returned `Reminder` reflects the
    /// post-update state.
    async fn update(
        &self,
        id: Uuid,
        vehicle_id: Uuid,
        user_id: Uuid,
        p: UpdateReminderParams,
    ) -> RepositoryResult<Reminder> {
        let query = format!(
            r#"UPDATE reminders r
               SET
                 is_completed  = COALESCE($1, r.is_completed),
                 due_date      = COALESCE($2, r.due_date),
                 due_odometer  = COALESCE($3, r.due_odometer),
                 notes         = COALESCE($4, r.notes)
               FROM vehicles v
               WHERE r.id = $5
                 AND r.vehicle_id = $6
                 AND v.id = r.vehicle_id
                 AND v.user_id = $7
               RETURNING {SELECT_COLS_PREFIXED}"#
        );
        let row = sqlx::query_as::<_, ReminderRow>(&query)
            .bind(p.is_completed)
            .bind(p.due_date)
            .bind(p.due_odometer.map(|v| v as i32))
            .bind(&p.notes)
            .bind(id)
            .bind(vehicle_id)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_pg_err)?
            .ok_or(RepositoryError::NotFound)?;

        Ok(row_to_reminder(row))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use testcontainers_modules::postgres::Postgres;
    use testcontainers_modules::testcontainers::runners::AsyncRunner;
    use testcontainers_modules::testcontainers::RunnableImage;

    async fn setup_pool() -> (
        PgPool,
        testcontainers_modules::testcontainers::ContainerAsync<Postgres>,
    ) {
        let image = RunnableImage::from(Postgres::default()).with_tag("16-alpine");
        let container = image.start().await;
        let port = container.get_host_port_ipv4(5432).await;
        let url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");
        let pool = PgPool::connect(&url).await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        (pool, container)
    }

    async fn insert_user(pool: &PgPool) -> Uuid {
        let row: (Uuid,) = sqlx::query_as(
            "INSERT INTO users (email, password_hash, name) VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(format!("user_{}@test.com", Uuid::new_v4()))
        .bind("hash")
        .bind("Test User")
        .fetch_one(pool)
        .await
        .unwrap();
        row.0
    }

    async fn insert_vehicle(pool: &PgPool, user_id: Uuid) -> Uuid {
        let row: (Uuid,) = sqlx::query_as(
            "INSERT INTO vehicles (user_id, brand, model, year, plate_number, fuel_type, current_odometer) \
             VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id",
        )
        .bind(user_id)
        .bind("Toyota")
        .bind("Avanza")
        .bind(2020_i16)
        .bind(format!("B {} TST", Uuid::new_v4().simple()))
        .bind("petrol")
        .bind(0_i32)
        .fetch_one(pool)
        .await
        .unwrap();
        row.0
    }

    fn make_date_params(vehicle_id: Uuid) -> CreateReminderParams {
        CreateReminderParams {
            vehicle_id,
            title: "Oil change".into(),
            reminder_type: "date".into(),
            due_date: Some(NaiveDate::from_ymd_opt(2026, 12, 1).unwrap()),
            due_odometer: None,
            notes: None,
        }
    }

    fn make_odometer_params(vehicle_id: Uuid) -> CreateReminderParams {
        CreateReminderParams {
            vehicle_id,
            title: "Tire rotation".into(),
            reminder_type: "odometer".into(),
            due_date: None,
            due_odometer: Some(50_000),
            notes: None,
        }
    }

    #[tokio::test]
    async fn insert_and_list_by_vehicle() {
        let (pool, _container) = setup_pool().await;
        let user_id = insert_user(&pool).await;
        let vehicle_id = insert_vehicle(&pool, user_id).await;
        let repo = PgReminderRepo::new(pool);

        let reminder = repo.insert(make_date_params(vehicle_id)).await.unwrap();

        assert_eq!(reminder.title, "Oil change");
        assert_eq!(reminder.reminder_type, ReminderType::Date);
        assert!(!reminder.is_completed);
        assert_eq!(
            reminder.due_date,
            Some(NaiveDate::from_ymd_opt(2026, 12, 1).unwrap())
        );

        let reminders = repo.list_by_vehicle(vehicle_id, user_id).await.unwrap();
        assert_eq!(reminders.len(), 1);
        assert_eq!(reminders[0].title, "Oil change");
    }

    #[tokio::test]
    async fn list_by_vehicle_with_wrong_user_returns_empty() {
        let (pool, _container) = setup_pool().await;
        let owner_id = insert_user(&pool).await;
        let intruder_id = insert_user(&pool).await;
        let vehicle_id = insert_vehicle(&pool, owner_id).await;
        let repo = PgReminderRepo::new(pool);

        repo.insert(make_date_params(vehicle_id)).await.unwrap();

        let reminders = repo.list_by_vehicle(vehicle_id, intruder_id).await.unwrap();
        assert_eq!(reminders.len(), 0);
    }

    #[tokio::test]
    async fn update_marks_completed() {
        let (pool, _container) = setup_pool().await;
        let user_id = insert_user(&pool).await;
        let vehicle_id = insert_vehicle(&pool, user_id).await;
        let repo = PgReminderRepo::new(pool);

        let reminder = repo.insert(make_date_params(vehicle_id)).await.unwrap();
        assert!(!reminder.is_completed);

        let updated = repo
            .update(
                reminder.id,
                vehicle_id,
                user_id,
                UpdateReminderParams {
                    is_completed: Some(true),
                    due_date: None,
                    due_odometer: None,
                    notes: None,
                },
            )
            .await
            .unwrap();

        assert!(updated.is_completed);
        // due_date preserved via COALESCE
        assert_eq!(updated.due_date, reminder.due_date);
    }

    #[tokio::test]
    async fn update_with_wrong_user_returns_not_found() {
        let (pool, _container) = setup_pool().await;
        let owner_id = insert_user(&pool).await;
        let intruder_id = insert_user(&pool).await;
        let vehicle_id = insert_vehicle(&pool, owner_id).await;
        let repo = PgReminderRepo::new(pool);

        let reminder = repo.insert(make_date_params(vehicle_id)).await.unwrap();

        let result = repo
            .update(
                reminder.id,
                vehicle_id,
                intruder_id,
                UpdateReminderParams {
                    is_completed: Some(true),
                    due_date: None,
                    due_odometer: None,
                    notes: None,
                },
            )
            .await;

        assert!(matches!(result, Err(RepositoryError::NotFound)));
    }

    #[tokio::test]
    async fn insert_odometer_type_reminder() {
        let (pool, _container) = setup_pool().await;
        let user_id = insert_user(&pool).await;
        let vehicle_id = insert_vehicle(&pool, user_id).await;
        let repo = PgReminderRepo::new(pool);

        let reminder = repo.insert(make_odometer_params(vehicle_id)).await.unwrap();

        assert_eq!(reminder.reminder_type, ReminderType::Odometer);
        assert_eq!(reminder.due_odometer, Some(50_000));
        assert!(reminder.due_date.is_none());
    }

    #[tokio::test]
    async fn list_returns_multiple_reminders_ordered_by_due_date_asc() {
        let (pool, _container) = setup_pool().await;
        let user_id = insert_user(&pool).await;
        let vehicle_id = insert_vehicle(&pool, user_id).await;
        let repo = PgReminderRepo::new(pool);

        // Insert later date first, earlier date second
        let later = CreateReminderParams {
            vehicle_id,
            title: "Later service".into(),
            reminder_type: "date".into(),
            due_date: Some(NaiveDate::from_ymd_opt(2027, 6, 1).unwrap()),
            due_odometer: None,
            notes: None,
        };
        let earlier = CreateReminderParams {
            vehicle_id,
            title: "Earlier service".into(),
            reminder_type: "date".into(),
            due_date: Some(NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
            due_odometer: None,
            notes: None,
        };
        repo.insert(later).await.unwrap();
        repo.insert(earlier).await.unwrap();

        let reminders = repo.list_by_vehicle(vehicle_id, user_id).await.unwrap();
        assert_eq!(reminders.len(), 2);
        // Earlier due_date should come first
        assert_eq!(reminders[0].title, "Earlier service");
        assert_eq!(reminders[1].title, "Later service");
    }
}
