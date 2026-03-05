use axum::{
    Json,
    body::Body,
    extract::{Multipart, Path, State},
    http::{StatusCode, header},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::super::state::AppState;
use crate::domain::services::file_manager::{FileManager, FileManagerError};
use crate::domain::types::file::file_metadata::Metadata;

// ── Request DTOs ──────────────────────────────────────────────

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

#[derive(Debug, Deserialize)]
pub struct UpdateFileRequest {
    pub name: Option<String>,
    pub ext: Option<String>,
    pub tags: Option<Vec<String>>,
    pub mime: Option<String>,
    pub content: Option<String>,
}

// ── Response DTOs ─────────────────────────────────────────────

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

// ── Conversions ───────────────────────────────────────────────

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
            title: None,
        }
    }
}

fn error_status(e: &FileManagerError) -> StatusCode {
    match e {
        FileManagerError::FileNotFound(_) => StatusCode::NOT_FOUND,
        FileManagerError::FileAlreadyExists(_) => StatusCode::CONFLICT,
        FileManagerError::PermissionDenied(_) => StatusCode::FORBIDDEN,
        FileManagerError::ValidationError(_) => StatusCode::BAD_REQUEST,
        FileManagerError::IoError(_) | FileManagerError::UnknownError(_) => {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

fn error_response(e: FileManagerError) -> impl IntoResponse {
    let status = error_status(&e);
    (
        status,
        Json(ErrorResponse {
            error: e.to_string(),
        }),
    )
}

fn file_to_response(file: crate::domain::types::file::file::File) -> FileResponse {
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

fn metadata_to_file_response(m: Metadata, content: Option<String>) -> FileResponse {
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

// ── Handlers ──────────────────────────────────────────────────

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

/// Extract the first markdown H1 heading or YAML frontmatter title from content.
fn extract_title(content: &str) -> Option<String> {
    let mut in_frontmatter = false;
    let mut past_frontmatter = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Handle YAML frontmatter
        if trimmed == "---" {
            if !in_frontmatter && !past_frontmatter {
                in_frontmatter = true;
                continue;
            } else if in_frontmatter {
                in_frontmatter = false;
                past_frontmatter = true;
                continue;
            }
        }

        if in_frontmatter {
            if let Some(rest) = trimmed.strip_prefix("title:") {
                let title = rest.trim().trim_matches('"').trim_matches('\'').trim();
                if !title.is_empty() {
                    return Some(title.to_string());
                }
            }
            continue;
        }

        // Skip blank lines
        if trimmed.is_empty() {
            continue;
        }

        // Look for first # heading
        if let Some(rest) = trimmed.strip_prefix("# ") {
            let title = rest.trim();
            if !title.is_empty() {
                return Some(title.to_string());
            }
        }

        // Stop searching after the first non-empty, non-heading line past frontmatter
        if past_frontmatter || !in_frontmatter {
            break;
        }
    }

    None
}

/// GET /api/files
pub async fn list_files(State(state): State<AppState>) -> impl IntoResponse {
    match state.file_manager.list_files() {
        Ok(files) => {
            let resp: Vec<MetadataResponse> = files
                .into_iter()
                .map(|m| {
                    let title = if m.ext == "md" {
                        // Try to read file content and extract title
                        let id = m.id;
                        state
                            .file_manager
                            .read_file_bytes(id)
                            .ok()
                            .and_then(|(_, bytes)| {
                                String::from_utf8(bytes)
                                    .ok()
                                    .and_then(|c| extract_title(&c))
                            })
                    } else {
                        None
                    };
                    let mut resp: MetadataResponse = m.into();
                    resp.title = title;
                    resp
                })
                .collect();
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => error_response(e).into_response(),
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

/// DELETE /api/files/:id
pub async fn delete_file(State(state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match state.file_manager.delete_file(id) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => error_response(e).into_response(),
    }
}

/// GET /api/tags
pub async fn list_tags(State(state): State<AppState>) -> impl IntoResponse {
    match state.file_manager.list_all_tags() {
        Ok(tags) => (StatusCode::OK, Json(tags)).into_response(),
        Err(e) => error_response(e).into_response(),
    }
}

/// GET /api/files/:id/download
pub async fn download_file(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.file_manager.read_file_bytes(id) {
        Ok((metadata, data)) => {
            let filename = format!("{}.{}", metadata.name, metadata.ext);
            let content_type = metadata.mime.clone();

            (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, content_type),
                    (
                        header::CONTENT_DISPOSITION,
                        format!("attachment; filename=\"{}\"", filename),
                    ),
                ],
                Body::from(data),
            )
                .into_response()
        }
        Err(e) => error_response(e).into_response(),
    }
}

/// GET /api/files/:id/raw — serve raw file inline (for images etc.)
pub async fn serve_file_raw(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.file_manager.read_file_bytes(id) {
        Ok((metadata, data)) => {
            let content_type = metadata.mime.clone();

            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, content_type)],
                Body::from(data),
            )
                .into_response()
        }
        Err(e) => error_response(e).into_response(),
    }
}

/// POST /api/files/upload — multipart upload
pub async fn upload_file(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut file_name = String::new();
    let mut file_ext = String::new();
    let mut file_mime = String::from("application/octet-stream");
    let mut file_data: Option<Vec<u8>> = None;
    let mut tags: Vec<String> = Vec::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "file" => {
                if let Some(fname) = field.file_name() {
                    let fname = fname.to_string();
                    if let Some(dot_pos) = fname.rfind('.') {
                        file_name = fname[..dot_pos].to_string();
                        file_ext = fname[dot_pos + 1..].to_string();
                    } else {
                        file_name = fname;
                        file_ext = String::new();
                    }
                }
                if let Some(ct) = field.content_type() {
                    file_mime = ct.to_string();
                }
                match field.bytes().await {
                    Ok(bytes) => file_data = Some(bytes.to_vec()),
                    Err(e) => {
                        return (
                            StatusCode::BAD_REQUEST,
                            Json(ErrorResponse {
                                error: format!("Failed to read upload: {e}"),
                            }),
                        )
                            .into_response();
                    }
                }
            }
            "tags" => {
                if let Ok(text) = field.text().await {
                    for t in text.split(',') {
                        let t = t.trim().to_string();
                        if !t.is_empty() {
                            tags.push(t);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let data = match file_data {
        Some(d) => d,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "No file field in upload".to_string(),
                }),
            )
                .into_response();
        }
    };

    if file_name.is_empty() {
        file_name = "uploaded".to_string();
    }
    if file_ext.is_empty() {
        file_ext = "bin".to_string();
    }

    match state
        .file_manager
        .create_file_bytes(file_name, file_ext, file_mime, tags, data)
    {
        Ok(metadata) => {
            let resp: MetadataResponse = metadata.into();
            (StatusCode::CREATED, Json(resp)).into_response()
        }
        Err(e) => error_response(e).into_response(),
    }
}
