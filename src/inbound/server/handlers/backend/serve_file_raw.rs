use axum::{
    body::Body,
    extract::{Path, State},
    http::{StatusCode, header},
    response::IntoResponse,
};
use uuid::Uuid;
use crate::domain::services::file_manager::FileManager;
use crate::inbound::server::state::AppState;
use crate::inbound::server::error_response::error_response;

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
