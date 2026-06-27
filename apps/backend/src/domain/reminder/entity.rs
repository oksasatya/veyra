use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Reminder {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub title: String,
    pub reminder_type: ReminderType,
    pub due_date: Option<NaiveDate>,
    pub due_odometer: Option<u32>,
    pub is_completed: bool,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// How the reminder is triggered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReminderType {
    Date,
    Odometer,
    Both,
}

impl ReminderType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Date => "date",
            Self::Odometer => "odometer",
            Self::Both => "both",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "date" => Some(Self::Date),
            "odometer" => Some(Self::Odometer),
            "both" => Some(Self::Both),
            _ => None,
        }
    }
}
