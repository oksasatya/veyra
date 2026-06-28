use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateExpenseRequest {
    pub expense_date: NaiveDate,
    pub category: String,
    pub description: String,
    pub amount: Decimal,
}

#[derive(Debug, Serialize)]
pub struct ExpenseResponse {
    pub id: String,
    pub vehicle_id: String,
    pub expense_date: NaiveDate,
    pub category: String,
    /// Serialised as a string to avoid floating-point precision loss.
    pub amount: String,
    pub description: String,
}
