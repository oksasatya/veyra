use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::expense::entity::{Expense, ExpenseCategory},
    ports::repositories::{CreateExpenseParams, ExpenseRepository, RepositoryError, RepositoryResult},
};

use super::models::ExpenseRow;

/// Column list for SELECT (no table prefix — used in single-table RETURNING).
const SELECT_COLS: &str = "id, vehicle_id, expense_date, category, description, amount, created_at";

/// Column list with `e.` prefix — used in JOINed queries where vehicles also
/// has overlapping column names (e.g., `id`).
const SELECT_COLS_PREFIXED: &str =
    "e.id, e.vehicle_id, e.expense_date, e.category, e.description, e.amount, e.created_at";

/// Postgres implementation of [`ExpenseRepository`].
pub struct PgExpenseRepo {
    pool: PgPool,
}

impl PgExpenseRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Converts a `sqlx::Error` to `RepositoryError`, keeping the ports layer
/// free of sqlx (layer-boundary rule).
fn map_pg_err(e: sqlx::Error) -> RepositoryError {
    RepositoryError::Database(e.to_string())
}

/// Maps an `ExpenseRow` (sqlx) to a domain `Expense`.
///
/// Category is stored as TEXT in Postgres. We default to `Other` if the stored
/// value is somehow unrecognised — this avoids a hard crash on a bad DB row
/// while still surfacing the issue via the `description` field.
fn row_to_expense(r: ExpenseRow) -> Expense {
    Expense {
        id: r.id,
        vehicle_id: r.vehicle_id,
        expense_date: r.expense_date,
        category: ExpenseCategory::parse(&r.category).unwrap_or(ExpenseCategory::Other),
        description: r.description,
        amount: r.amount,
        created_at: r.created_at,
    }
}

#[async_trait]
impl ExpenseRepository for PgExpenseRepo {
    /// Lists expenses for a vehicle, scoped via a JOIN to ensure the vehicle
    /// belongs to `user_id` — defence-in-depth even though the use case already
    /// checks ownership. Ordered by `expense_date DESC`.
    async fn list_by_vehicle(
        &self,
        vehicle_id: Uuid,
        user_id: Uuid,
    ) -> RepositoryResult<Vec<Expense>> {
        let query = format!(
            "SELECT {SELECT_COLS_PREFIXED} \
             FROM expenses e \
             JOIN vehicles v ON v.id = e.vehicle_id \
             WHERE e.vehicle_id = $1 AND v.user_id = $2 \
             ORDER BY e.expense_date DESC"
        );
        let rows = sqlx::query_as::<_, ExpenseRow>(&query)
            .bind(vehicle_id)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(map_pg_err)?;

        Ok(rows.into_iter().map(row_to_expense).collect())
    }

    async fn insert(&self, p: CreateExpenseParams) -> RepositoryResult<Expense> {
        let query = format!(
            r#"INSERT INTO expenses
               (vehicle_id, expense_date, category, description, amount)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING {SELECT_COLS}"#
        );
        let row = sqlx::query_as::<_, ExpenseRow>(&query)
            .bind(p.vehicle_id)
            .bind(p.expense_date)
            .bind(&p.category)
            .bind(&p.description)
            .bind(p.amount)
            .fetch_one(&self.pool)
            .await
            .map_err(map_pg_err)?;

        Ok(row_to_expense(row))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use testcontainers_modules::postgres::Postgres;
    use testcontainers_modules::testcontainers::runners::AsyncRunner;
    use testcontainers_modules::testcontainers::RunnableImage;

    /// Spins up Postgres 16-alpine, runs all migrations, returns pool + container.
    /// Caller must bind `_container` to keep it alive for the pool's lifetime.
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

    /// Inserts a bare user row and returns its id.
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

    /// Inserts a bare vehicle row owned by `user_id` and returns its id.
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

    fn make_params(vehicle_id: Uuid) -> CreateExpenseParams {
        CreateExpenseParams {
            vehicle_id,
            expense_date: NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
            category: "tire".into(),
            description: "Front tire replacement".into(),
            amount: Decimal::new(35_000_000, 2), // 350000.00
        }
    }

    #[tokio::test]
    async fn insert_and_list_by_vehicle() {
        let (pool, _container) = setup_pool().await;
        let user_id = insert_user(&pool).await;
        let vehicle_id = insert_vehicle(&pool, user_id).await;
        let repo = PgExpenseRepo::new(pool);

        let expense = repo.insert(make_params(vehicle_id)).await.unwrap();

        assert_eq!(expense.category, ExpenseCategory::Tire);
        assert_eq!(expense.description, "Front tire replacement");
        assert_eq!(expense.amount.to_string(), "350000.00");

        let expenses = repo.list_by_vehicle(vehicle_id, user_id).await.unwrap();
        assert_eq!(expenses.len(), 1);
        assert_eq!(expenses[0].description, "Front tire replacement");
    }

    #[tokio::test]
    async fn list_by_vehicle_with_wrong_user_returns_empty() {
        let (pool, _container) = setup_pool().await;
        let owner_id = insert_user(&pool).await;
        let intruder_id = insert_user(&pool).await;
        let vehicle_id = insert_vehicle(&pool, owner_id).await;
        let repo = PgExpenseRepo::new(pool);

        repo.insert(make_params(vehicle_id)).await.unwrap();

        // Querying as a different user should return nothing (JOIN filters it out)
        let expenses = repo.list_by_vehicle(vehicle_id, intruder_id).await.unwrap();
        assert_eq!(expenses.len(), 0);
    }

    #[tokio::test]
    async fn list_returns_multiple_expenses_ordered_by_date_desc() {
        let (pool, _container) = setup_pool().await;
        let user_id = insert_user(&pool).await;
        let vehicle_id = insert_vehicle(&pool, user_id).await;
        let repo = PgExpenseRepo::new(pool);

        // Insert two expenses: older first, newer second
        let older = CreateExpenseParams {
            vehicle_id,
            expense_date: NaiveDate::from_ymd_opt(2025, 6, 1).unwrap(),
            category: "battery".into(),
            description: "Old battery".into(),
            amount: Decimal::new(30_000_000, 2),
        };
        let newer = CreateExpenseParams {
            vehicle_id,
            expense_date: NaiveDate::from_ymd_opt(2026, 1, 20).unwrap(),
            category: "tire".into(),
            description: "New tire".into(),
            amount: Decimal::new(35_000_000, 2),
        };
        repo.insert(older).await.unwrap();
        repo.insert(newer).await.unwrap();

        let expenses = repo.list_by_vehicle(vehicle_id, user_id).await.unwrap();
        assert_eq!(expenses.len(), 2);
        // Most recent date first
        assert_eq!(expenses[0].description, "New tire");
        assert_eq!(expenses[1].description, "Old battery");
    }
}
