use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::ports::repositories::{
    RepositoryError, RepositoryResult, SummaryRepository, VehicleSummaryData,
};

use super::models::SummaryRow;

/// CTE query that aggregates all vehicle stats in a single Postgres round-trip.
///
/// Each CTE pre-aggregates its sub-table before joining onto the vehicle row.
/// A direct multi-table LEFT JOIN would produce a cartesian product
/// (e.g., 2 service records × 3 fuel logs = 6 rows, inflating every sum).
/// Pre-aggregating eliminates the fan-out: O(1) round-trips, O(n) work inside
/// Postgres where n = rows per vehicle, all bounded and indexed on vehicle_id.
const SUMMARY_QUERY: &str = r#"
WITH
    svc AS (
        SELECT vehicle_id,
               COUNT(*)               AS total_services,
               COALESCE(SUM(cost), 0) AS total_service_cost
        FROM   service_records
        WHERE  vehicle_id = $1
        GROUP BY vehicle_id
    ),
    fuel AS (
        SELECT vehicle_id,
               COUNT(*)                    AS total_refuels,
               COALESCE(SUM(total_cost), 0) AS total_fuel_cost
        FROM   fuel_logs
        WHERE  vehicle_id = $1
        GROUP BY vehicle_id
    ),
    exp AS (
        SELECT vehicle_id,
               COALESCE(SUM(amount), 0) AS total_expenses
        FROM   expenses
        WHERE  vehicle_id = $1
        GROUP BY vehicle_id
    ),
    rem AS (
        SELECT vehicle_id,
               COUNT(*) AS upcoming_reminders
        FROM   reminders
        WHERE  vehicle_id = $1
          AND  is_completed = FALSE
          AND (
                due_date     <= CURRENT_DATE + INTERVAL '30 days'
             OR due_odometer <= (SELECT current_odometer FROM vehicles WHERE id = $1) + 500
          )
        GROUP BY vehicle_id
    )
SELECT
    v.id,
    v.current_odometer,
    COALESCE(svc.total_services,    0) AS total_services,
    COALESCE(svc.total_service_cost,0) AS total_service_cost,
    COALESCE(fuel.total_refuels,    0) AS total_refuels,
    COALESCE(fuel.total_fuel_cost,  0) AS total_fuel_cost,
    COALESCE(exp.total_expenses,    0) AS total_expenses,
    COALESCE(rem.upcoming_reminders,0) AS upcoming_reminders
FROM vehicles v
LEFT JOIN svc  ON svc.vehicle_id  = v.id
LEFT JOIN fuel ON fuel.vehicle_id = v.id
LEFT JOIN exp  ON exp.vehicle_id  = v.id
LEFT JOIN rem  ON rem.vehicle_id  = v.id
WHERE v.id = $1 AND v.user_id = $2
"#;

/// Postgres implementation of [`SummaryRepository`].
pub struct PgSummaryRepo {
    pool: PgPool,
}

impl PgSummaryRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SummaryRepository for PgSummaryRepo {
    async fn get_vehicle_summary(
        &self,
        vehicle_id: Uuid,
        user_id: Uuid,
    ) -> RepositoryResult<VehicleSummaryData> {
        let row = sqlx::query_as::<_, SummaryRow>(SUMMARY_QUERY)
            .bind(vehicle_id)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?
            .ok_or(RepositoryError::NotFound)?;

        Ok(VehicleSummaryData {
            vehicle_id: row.id,
            current_odometer: row.current_odometer,
            total_services: row.total_services,
            total_service_cost: row.total_service_cost,
            total_refuels: row.total_refuels,
            total_fuel_cost: row.total_fuel_cost,
            total_expenses: row.total_expenses,
            upcoming_reminders: row.upcoming_reminders,
        })
    }
}
