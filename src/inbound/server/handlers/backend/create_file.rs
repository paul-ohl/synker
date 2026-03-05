use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use crate::domain::services::file_manager::FileManager;
use crate::inbound::server::state::AppState;
use crate::inbound::server::error_response::error_response;

#[derive(Debug, Deserialize)]
pub struct CreateFileRequest {
    pub name: String,
    pub ext: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_mime")]
    pub mime: String,
    pub content: Option<String>,
}

fn default_mime() -> String {
    "text/markdown".to_string()
}

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

pub fn file_to_response(file: crate::domain::types::file::file::File) -> FileResponse {
    FileResponse {
        id: file.id.to_string(),
        name: file.metadata.name,
        ext: file.metadata.ext,
        tags: file.metadata.tags,
        size: file.metadata.size,
        mime: file.metadata.mime,
        created_at: file.metadata.created_at,
        modified_at: file.metadata.modified_at,
        content: Some(file.content),
    }
}

/// POST /api/files
pub async fn create_file(
    State(state): State<AppState>,
    Json(body): Json<CreateFileRequest>,
) -> impl IntoResponse {
    use crate::domain::types::file::file::NewFile;
    use crate::domain::types::file::file_metadata::NewMetadata;

    let new_file = NewFile {
        metadata: NewMetadata {
            name: body.name,
            ext: body.ext,
            tags: body.tags,
            mime: body.mime,
        },
        content: body.content,
    };

    match state.file_manager.create_file(new_file) {
        Ok(file) => (StatusCode::CREATED, Json(file_to_response(file))).into_response(),
        Err(e) => error_response(e).into_response(),
    }
}
