use std::fmt;

use uuid::Uuid;

use crate::domain::types::file::{
    file::{File, NewFile, UpdateFile},
    file_metadata::Metadata,
    file_query::FileQuery,
};

/// Port: the primary interface for file management operations.
/// Implemented by outbound adapters (e.g., filesystem, database).
pub trait FileManager: Send + Sync {
    fn create_file(&self, new_file: NewFile) -> Result<File, FileManagerError>;
    fn create_file_bytes(
        &self,
        name: String,
        ext: String,
        mime: String,
        tags: Vec<String>,
        data: Vec<u8>,
    ) -> Result<Metadata, FileManagerError>;
    fn read_file(&self, file_id: Uuid) -> Result<File, FileManagerError>;
    fn read_file_bytes(&self, file_id: Uuid) -> Result<(Metadata, Vec<u8>), FileManagerError>;
    fn list_files(&self) -> Result<Vec<Metadata>, FileManagerError>;
    fn list_all_tags(&self) -> Result<Vec<String>, FileManagerError>;
    fn update_file(&self, file_id: Uuid, update: UpdateFile) -> Result<File, FileManagerError>;
    fn delete_file(&self, file_id: Uuid) -> Result<(), FileManagerError>;
    fn find(&self, query: FileQuery) -> Result<Vec<Metadata>, FileManagerError>;
}

#[derive(Debug)]
pub enum FileManagerError {
    FileNotFound(String),
    FileAlreadyExists(String),
    PermissionDenied(String),
    ValidationError(String),
    IoError(String),
    UnknownError(String),
}

impl fmt::Display for FileManagerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileNotFound(s) => write!(f, "File not found: {s}"),
            Self::FileAlreadyExists(s) => write!(f, "File already exists: {s}"),
            Self::PermissionDenied(s) => write!(f, "Permission denied: {s}"),
            Self::ValidationError(s) => write!(f, "Validation error: {s}"),
            Self::IoError(s) => write!(f, "IO error: {s}"),
            Self::UnknownError(s) => write!(f, "Unknown error: {s}"),
        }
    }
}

impl std::error::Error for FileManagerError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_file_already_exists() {
        let e = FileManagerError::FileAlreadyExists("foo.md".to_string());
        assert_eq!(e.to_string(), "File already exists: foo.md");
    }

    #[test]
    fn test_display_permission_denied() {
        let e = FileManagerError::PermissionDenied("secret.txt".to_string());
        assert_eq!(e.to_string(), "Permission denied: secret.txt");
    }
}
