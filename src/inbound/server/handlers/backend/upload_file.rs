use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use crate::inbound::server::state::AppState;
use crate::inbound::server::error_response::{error_response, ErrorResponse};

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

impl From<crate::domain::types::file::file_metadata::Metadata> for MetadataResponse {
    fn from(m: crate::domain::types::file::file_metadata::Metadata) -> Self {
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
