use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::inbound::server::state::AppState;
use crate::inbound::server::error_response::error_response;

#[derive(Debug, Deserialize)]
pub struct UpdateFileRequest {
    pub name: Option<String>,
    pub ext: Option<String>,
    pub tags: Option<Vec<String>>,
    pub mime: Option<String>,
    pub content: Option<String>,
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

/// PUT /api/files/:id
pub async fn update_file(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateFileRequest>,
) -> impl IntoResponse {
    use crate::domain::types::file::file::UpdateFile;
    use crate::domain::types::file::file_metadata::UpdateMetadata;

    let has_meta =
        body.name.is_some() || body.ext.is_some() || body.tags.is_some() || body.mime.is_some();

    let update = UpdateFile {
        metadata: if has_meta {
            Some(UpdateMetadata {
                name: body.name,
                ext: body.ext,
                tags: body.tags,
                mime: body.mime,
            })
        } else {
            None
        },
        content: body.content,
    };

    match state.file_manager.update_file(id, update) {
        Ok(file) => (StatusCode::OK, Json(file_to_response(file))).into_response(),
        Err(e) => error_response(e).into_response(),
    }
}
