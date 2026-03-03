use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;
use crate::inbound::server::state::AppState;
use crate::inbound::server::error_response::error_response;

/// DELETE /api/files/:id
pub async fn delete_file(State(state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match state.file_manager.delete_file(id) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => error_response(e).into_response(),
    }
}
