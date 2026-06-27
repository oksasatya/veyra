use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateReminderRequest {
    pub title: String,
    pub reminder_type: String,
    pub due_date: Option<NaiveDate>,
    pub due_odometer: Option<u32>,
    pub notes: Option<String>,
}

/// Partial update — all fields are optional; only provided fields are applied.
#[derive(Debug, Deserialize)]
pub struct PatchReminderRequest {
    pub is_completed: Option<bool>,
    pub due_date: Option<NaiveDate>,
    pub due_odometer: Option<u32>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ReminderResponse {
    pub id: String,
    pub vehicle_id: String,
    pub title: String,
    pub reminder_type: String,
    pub due_date: Option<NaiveDate>,
    pub due_odometer: Option<u32>,
    pub is_completed: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ReminderListResponse {
    pub reminders: Vec<ReminderResponse>,
}
