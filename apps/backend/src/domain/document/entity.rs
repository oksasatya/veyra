use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Document {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub doc_type: DocType,
    pub title: String,
    pub expiry_date: Option<NaiveDate>,
    pub file_url: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Vehicle document type (Indonesian vehicle ownership context).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocType {
    Stnk,
    Bpkb,
    Insurance,
    Other,
}

impl DocType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Stnk => "stnk",
            Self::Bpkb => "bpkb",
            Self::Insurance => "insurance",
            Self::Other => "other",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "stnk" => Some(Self::Stnk),
            "bpkb" => Some(Self::Bpkb),
            "insurance" => Some(Self::Insurance),
            "other" => Some(Self::Other),
            _ => None,
        }
    }
}
