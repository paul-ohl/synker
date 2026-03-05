use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use crate::domain::services::file_manager::FileManager;
use crate::inbound::server::state::AppState;
use crate::domain::types::file::file_metadata::Metadata;
use crate::inbound::server::error_response::error_response;

#[derive(Debug, Serialize)]
pub struct MetadataResponse {
    pub id: String,
    pub name: String,
    pub ext: String,
    pub tags: Vec<String>,
    pub size: u64,
    pub mime: String,
    pub created_at: String,
    pub modified_at: String,
}

impl From<Metadata> for MetadataResponse {
    fn from(m: Metadata) -> Self {
        Self {
            id: m.id.to_string(),
            name: m.name,
            ext: m.ext,
            tags: m.tags,
            size: m.size,
            mime: m.mime,
            created_at: m.created_at,
            modified_at: m.modified_at,
        }
    }
}

/// GET /api/files
pub async fn list_files(State(state): State<AppState>) -> impl IntoResponse {
    match state.file_manager.list_files() {
        Ok(files) => {
            let resp: Vec<MetadataResponse> = files.into_iter().map(Into::into).collect();
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => error_response(e).into_response(),
    }
}
