use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Expense {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub expense_date: NaiveDate,
    pub category: ExpenseCategory,
    pub description: String,
    pub amount: Decimal,
    pub created_at: DateTime<Utc>,
}

/// Expense category. The caller maps an unknown string to `AppError::Validation`
/// — hence `Option` rather than `Result` here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpenseCategory {
    Tire,
    Battery,
    Tax,
    Insurance,
    Other,
}

impl ExpenseCategory {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Tire => "tire",
            Self::Battery => "battery",
            Self::Tax => "tax",
            Self::Insurance => "insurance",
            Self::Other => "other",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "tire" => Some(Self::Tire),
            "battery" => Some(Self::Battery),
            "tax" => Some(Self::Tax),
            "insurance" => Some(Self::Insurance),
            "other" => Some(Self::Other),
            _ => None,
        }
    }
}
