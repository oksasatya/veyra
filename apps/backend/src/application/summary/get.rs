use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::application::errors::AppError;

/// Aggregated summary for a single vehicle, computed in one SQL pass.
#[derive(Debug)]
pub struct VehicleSummary {
    pub vehicle_id: Uuid,
    pub current_odometer: i32,
    pub total_services: i64,
    pub total_service_cost: Decimal,
    pub total_refuels: i64,
    pub total_fuel_cost: Decimal,
    pub total_expenses: Decimal,
    pub upcoming_reminders: i64,
}

/// Fetches an aggregated vehicle summary in a single SQL query.
///
/// Ownership is enforced by the `v.user_id = $2` predicate — no other
/// user can see another user's vehicle summary.
pub struct GetSummaryUseCase {
    pub pool: PgPool,
}

impl GetSummaryUseCase {
    pub async fn execute(&self, vehicle_id: Uuid, user_id: Uuid) -> Result<VehicleSummary, AppError> {
        // CTEs pre-aggregate each sub-table independently before joining to the
        // vehicle row. Without the CTEs a direct multi-table LEFT JOIN produces
        // a cartesian product across dimensions — e.g., 2 service records × 1
        // fuel log gives 2 fuel-log rows in the result set and inflates the sum.
        // Pre-aggregating eliminates the fan-out: O(1) round-trips, O(n) work
        // inside Postgres where n = rows per vehicle, all bounded and indexed.
        let row = sqlx::query!(
            r#"WITH
                svc AS (
                    SELECT vehicle_id,
                           COUNT(*)          AS cnt,
                           COALESCE(SUM(cost), 0) AS total
                    FROM   service_records
                    WHERE  vehicle_id = $1
                    GROUP BY vehicle_id
                ),
                fuel AS (
                    SELECT vehicle_id,
                           COUNT(*)               AS cnt,
                           COALESCE(SUM(total_cost), 0) AS total
                    FROM   fuel_logs
                    WHERE  vehicle_id = $1
                    GROUP BY vehicle_id
                ),
                exp AS (
                    SELECT vehicle_id,
                           COALESCE(SUM(amount), 0) AS total
                    FROM   expenses
                    WHERE  vehicle_id = $1
                    GROUP BY vehicle_id
                ),
                rem AS (
                    SELECT vehicle_id,
                           COUNT(*) AS upcoming
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
                COALESCE(svc.cnt,      0)  AS "total_services!: i64",
                COALESCE(svc.total,    0)  AS "total_service_cost!: Decimal",
                COALESCE(fuel.cnt,     0)  AS "total_refuels!: i64",
                COALESCE(fuel.total,   0)  AS "total_fuel_cost!: Decimal",
                COALESCE(exp.total,    0)  AS "total_expenses!: Decimal",
                COALESCE(rem.upcoming, 0)  AS "upcoming_reminders!: i64"
            FROM vehicles v
            LEFT JOIN svc  ON svc.vehicle_id  = v.id
            LEFT JOIN fuel ON fuel.vehicle_id = v.id
            LEFT JOIN exp  ON exp.vehicle_id  = v.id
            LEFT JOIN rem  ON rem.vehicle_id  = v.id
            WHERE v.id = $1 AND v.user_id = $2"#,
            vehicle_id,
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or(AppError::NotFound)?;

        Ok(VehicleSummary {
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
