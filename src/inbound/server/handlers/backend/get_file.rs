use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use uuid::Uuid;
use crate::domain::services::file_manager::FileManager;
use crate::inbound::server::state::AppState;
use crate::domain::types::file::file_metadata::Metadata;
use crate::inbound::server::error_response::error_response;

#[derive(Debug, Serialize)]
pub struct FileResponse {
    pub id: String,
    pub name: String,
    pub ext: String,
    pub tags: Vec<String>,
    pub size: u64,
    pub mime: String,
    pub created_at: String,
    pub modified_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

pub fn metadata_to_file_response(m: Metadata, content: Option<String>) -> FileResponse {
    FileResponse {
        id: m.id.to_string(),
        name: m.name,
        ext: m.ext,
        tags: m.tags,
        size: m.size,
        mime: m.mime,
        created_at: m.created_at,
        modified_at: m.modified_at,
        content,
    }
}

/// GET /api/files/:id
pub async fn get_file(State(state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match state.file_manager.read_file_bytes(id) {
        Ok((metadata, bytes)) => {
            // Try to interpret as UTF-8 text; if it fails, return no content (binary file)
            let content = String::from_utf8(bytes).ok();
            let resp = metadata_to_file_response(metadata, content);
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => error_response(e).into_response(),
    }
}
