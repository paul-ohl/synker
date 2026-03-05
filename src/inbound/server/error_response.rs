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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[test]
    fn test_error_status_file_not_found() {
        let err = FileManagerError::FileNotFound("file.md".to_string());
        assert_eq!(error_status(&err), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_error_status_conflict() {
        let err = FileManagerError::FileAlreadyExists("file.md".to_string());
        assert_eq!(error_status(&err), StatusCode::CONFLICT);
    }

    #[test]
    fn test_error_status_forbidden() {
        let err = FileManagerError::PermissionDenied("no access".to_string());
        assert_eq!(error_status(&err), StatusCode::FORBIDDEN);
    }

    #[test]
    fn test_error_status_bad_request() {
        let err = FileManagerError::ValidationError("bad input".to_string());
        assert_eq!(error_status(&err), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_error_status_io_error() {
        let err = FileManagerError::IoError("disk full".to_string());
        assert_eq!(error_status(&err), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_error_status_unknown_error() {
        let err = FileManagerError::UnknownError("something went wrong".to_string());
        assert_eq!(error_status(&err), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
