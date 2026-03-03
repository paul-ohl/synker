use crate::domain::services::file_manager::FileManagerError;
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub fn error_status(e: &FileManagerError) -> StatusCode {
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

pub fn error_response(e: FileManagerError) -> impl IntoResponse {
    let status = error_status(&e);
    (
        status,
        Json(ErrorResponse {
            error: e.to_string(),
        }),
    )
}
