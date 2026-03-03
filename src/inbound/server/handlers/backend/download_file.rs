use axum::{
    body::Body,
    extract::{Path, State},
    http::{StatusCode, header},
    response::IntoResponse,
};
use uuid::Uuid;
use crate::inbound::server::state::AppState;
use crate::inbound::server::error_response::error_response;

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
