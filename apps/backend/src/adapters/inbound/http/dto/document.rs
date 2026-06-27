use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateDocumentRequest {
    pub doc_type: String,
    pub title: String,
    pub expiry_date: Option<NaiveDate>,
    pub file_url: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DocumentResponse {
    pub id: String,
    pub vehicle_id: String,
    pub doc_type: String,
    pub title: String,
    pub expiry_date: Option<NaiveDate>,
    pub file_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DocumentListResponse {
    pub documents: Vec<DocumentResponse>,
}
