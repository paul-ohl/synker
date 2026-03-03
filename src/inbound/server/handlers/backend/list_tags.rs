use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
};
use crate::inbound::server::state::AppState;
use crate::inbound::server::error_response::error_response;

/// GET /api/tags
pub async fn list_tags(State(state): State<AppState>) -> impl IntoResponse {
    match state.file_manager.list_all_tags() {
        Ok(tags) => (StatusCode::OK, Json(tags)).into_response(),
        Err(e) => error_response(e).into_response(),
    }
}
